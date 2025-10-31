//! Statement parsing for the compiler.
//!
//! This module contains functions for parsing Python statements including
//! function definitions, class definitions, control flow statements (if, while, for),
//! assignments, and expression statements.

use crate::bytecode::OpCode;
use crate::object::{FunctionPrototype, ObjectType, Type};
use crate::token::Token;
use std::rc::Rc;

use super::types::*;

impl super::Compiler<'_> {
    /// Main statement dispatcher. Parses a single statement based on the current token.
    pub(super) fn parse_statement(&mut self) {
        let (token_result, info) = match self.peek_token_with_indent() {
            Some(value) => value,
            None => return,
        };

        self.current_indent = info.indent;

        let token = match token_result {
            Ok(token) => token,
            Err(_) => {
                self.lexer.next(); // Consume the unknown token.
                self.had_error = true;
                return;
            }
        };

        match token {
            Token::Print => self.parse_print_statement(),
            Token::For => self.parse_for_statement(),
            Token::While => self.parse_while_statement(),
            Token::If => self.parse_if_statement(),
            Token::Def => self.parse_function_statement(),
            Token::Class => self.parse_class_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Break => self.parse_break_statement(),
            Token::Nonlocal => self.parse_nonlocal_statement(),
            Token::Semicolon => {
                self.lexer.next(); // Consume stray semicolons.
            }
            Token::Unknown => {
                self.lexer.next(); // Skip unrecognized tokens to prevent infinite loops.
                self.had_error = true;
            }
            Token::Identifier(_) => {
                if let Some(kind) = self.detect_assignment_kind() {
                    self.parse_assignment_statement(kind);
                } else {
                    self.parse_expression_statement(true);
                }
            }
            _ => self.parse_expression_statement(true),
        }
    }

    fn has_newline_between(&self, start: usize, end: usize) -> bool {
        self.source[start..end].chars().any(|ch| ch == '\n')
    }

    fn parse_suite(&mut self, parent_indent: usize, colon_end: usize) -> bool {
        let Some((token_result, info)) = self.peek_token_with_indent() else {
            return false;
        };

        let inline = !self.has_newline_between(colon_end, info.start);

        if inline {
            match token_result {
                Ok(_) => {
                    self.parse_statement();
                    !self.had_error
                }
                Err(_) => {
                    self.lexer.next();
                    self.had_error = true;
                    false
                }
            }
        } else {
            if info.indent <= parent_indent {
                self.had_error = true;
                return false;
            }

            let mut had_statement = false;

            while let Some((token_result, next_info)) = self.peek_token_with_indent() {
                if next_info.indent <= parent_indent {
                    break;
                }

                match token_result {
                    Ok(_) => {
                        self.parse_statement();
                        had_statement = true;
                        if self.had_error {
                            break;
                        }
                    }
                    Err(_) => {
                        self.lexer.next();
                        self.had_error = true;
                        break;
                    }
                }

                if self.had_error {
                    break;
                }
            }

            had_statement
        }
    }

    fn parse_function_statement(&mut self) {
        self.lexer.next(); // consume 'def'

        let name = match self.lexer.next() {
            Some(Ok(Token::Identifier(identifier))) => identifier,
            _ => {
                self.had_error = true;
                return;
            }
        };

        if self.lexer.next() != Some(Ok(Token::LParen)) {
            self.had_error = true;
            return;
        }

        let mut parameters: Vec<Parameter> = Vec::new();
        if self.lexer.clone().next() != Some(Ok(Token::RParen)) {
            loop {
                let param_name = match self.lexer.next() {
                    Some(Ok(Token::Identifier(param))) => param,
                    _ => {
                        self.had_error = true;
                        return;
                    }
                };

                // Check for type annotation: param: type
                let type_annotation = if matches!(self.lexer.clone().next(), Some(Ok(Token::Colon)))
                {
                    self.lexer.next(); // consume ':'
                    self.parse_type_annotation()
                } else {
                    None
                };

                // Check for default value: param = value
                let default_value = if matches!(self.lexer.clone().next(), Some(Ok(Token::Assign)))
                {
                    self.lexer.next(); // consume '='
                    self.parse_constant_default_value()
                } else {
                    None
                };

                // Validate: non-default parameters cannot follow default parameters
                let has_default_params = parameters.iter().any(|p| p.default_value.is_some());
                if has_default_params && default_value.is_none() {
                    // Error: non-default parameter after default parameter
                    self.had_error = true;
                    return;
                }

                if let Some(default) = default_value {
                    parameters.push(Parameter::new_with_default(
                        param_name,
                        type_annotation,
                        default,
                    ));
                } else {
                    parameters.push(Parameter::new(param_name, type_annotation));
                }

                match self.lexer.clone().next() {
                    Some(Ok(Token::Comma)) => {
                        self.lexer.next();
                    }
                    Some(Ok(Token::RParen)) => break,
                    _ => {
                        self.had_error = true;
                        return;
                    }
                }
            }
        }

        if self.lexer.next() != Some(Ok(Token::RParen)) {
            self.had_error = true;
            return;
        }

        if parameters.len() > u8::MAX as usize {
            self.had_error = true;
            return;
        }

        // Check for return type annotation: -> type
        let return_type = if matches!(self.lexer.clone().next(), Some(Ok(Token::Arrow))) {
            self.lexer.next(); // consume '->'
            self.parse_type_annotation()
        } else {
            None
        };

        let colon_end = if let Some(Ok(Token::Colon)) = self.lexer.next() {
            self.lexer.span().end
        } else {
            self.had_error = true;
            return;
        };

        let outer_chunk = std::mem::take(&mut self.chunk);
        let outer_loop_stack = std::mem::take(&mut self.loop_stack);
        let parent_indent = self.current_indent;

        // Build qualified name by joining function names with '.'
        self.function_name_stack.push(name.clone());
        let qualname = self.function_name_stack.join(".");

        self.function_scopes
            .push(FunctionScope::new_with_params(parameters.clone()));
        self.function_depth += 1;

        let body_had_statement = self.parse_suite(parent_indent, colon_end);

        self.function_depth -= 1;
        self.function_name_stack.pop();
        let captured_upvalues = self
            .function_scopes
            .pop()
            .map(|scope| scope.upvalues)
            .unwrap_or_default();

        if !body_had_statement && !self.had_error {
            self.had_error = true;
        }

        if !self.had_error && self.chunk.code.last() != Some(&(OpCode::OpReturn as u8)) {
            self.chunk.code.push(OpCode::OpReturn as u8);
        }

        let function_chunk = std::mem::replace(&mut self.chunk, outer_chunk);
        self.loop_stack = outer_loop_stack;
        self.current_indent = parent_indent;

        if self.had_error {
            return;
        }

        // Extract parameter names, types, and default values for the prototype
        let parameter_names: Vec<String> = parameters.iter().map(|p| p.name.clone()).collect();
        let parameter_types: Vec<Option<Type>> = parameters
            .iter()
            .map(|p| p.type_annotation.clone())
            .collect();
        let default_values: Vec<Option<crate::object::Object>> =
            parameters.iter().map(|p| p.default_value.clone()).collect();

        let type_info = crate::object::TypeInfo {
            parameter_names,
            parameter_types,
            return_type,
            default_values,
        };

        let mut prototype = FunctionPrototype::new_with_types(
            name.clone(),
            parameters.len(),
            function_chunk,
            captured_upvalues,
            type_info,
            self.module.clone(),
        );
        prototype.qualname = qualname;
        let prototype_value = Rc::new(ObjectType::FunctionPrototype(Rc::new(prototype)));
        let prototype_const_idx = self.add_constant(prototype_value);
        self.chunk.code.push(OpCode::OpMakeFunction as u8);
        self.chunk.code.push(prototype_const_idx as u8);

        let name_idx = self.add_constant(Rc::new(ObjectType::String(name)));
        self.chunk.code.push(OpCode::OpDefineGlobal as u8);
        self.chunk.code.push(name_idx as u8);
    }

    fn parse_class_statement(&mut self) {
        self.lexer.next(); // consume 'class'

        let class_name = match self.lexer.next() {
            Some(Ok(Token::Identifier(identifier))) => identifier,
            _ => {
                self.had_error = true;
                return;
            }
        };

        // Check for inheritance: class Child(Parent):
        let has_parent = matches!(self.lexer.clone().next(), Some(Ok(Token::LParen)));
        let parent_name = if has_parent {
            self.lexer.next(); // consume '('
            let parent = match self.lexer.next() {
                Some(Ok(Token::Identifier(identifier))) => identifier,
                _ => {
                    self.had_error = true;
                    return;
                }
            };
            if self.lexer.next() != Some(Ok(Token::RParen)) {
                self.had_error = true;
                return;
            }
            Some(parent)
        } else {
            None
        };

        let colon_end = if let Some(Ok(Token::Colon)) = self.lexer.next() {
            self.lexer.span().end
        } else {
            self.had_error = true;
            return;
        };

        let parent_indent = self.current_indent;
        let class_body_indent = parent_indent + 4; // Methods are indented 4 spaces from class

        // Parse class body - collect methods
        let mut method_names: Vec<String> = Vec::new();

        loop {
            let Some((token_result, info)) = self.peek_token_with_indent() else {
                break;
            };

            if info.indent <= parent_indent {
                break;
            }

            if self.has_newline_between(colon_end, info.start) && info.indent != class_body_indent {
                self.had_error = true;
                return;
            }

            let Ok(token) = token_result else {
                self.lexer.next();
                self.had_error = true;
                return;
            };

            // Only methods (def statements) are allowed in class body
            if matches!(token, Token::Def) {
                // Parse the method as a function
                self.lexer.next(); // consume 'def'

                let method_name = match self.lexer.next() {
                    Some(Ok(Token::Identifier(identifier))) => identifier,
                    _ => {
                        self.had_error = true;
                        return;
                    }
                };

                // Now parse rest of function exactly like parse_function_statement
                if self.lexer.next() != Some(Ok(Token::LParen)) {
                    self.had_error = true;
                    return;
                }

                let mut parameters: Vec<Parameter> = Vec::new();
                if self.lexer.clone().next() != Some(Ok(Token::RParen)) {
                    loop {
                        let param_name = match self.lexer.next() {
                            Some(Ok(Token::Identifier(param))) => param,
                            _ => {
                                self.had_error = true;
                                return;
                            }
                        };

                        // Check for type annotation: param: type
                        let type_annotation =
                            if matches!(self.lexer.clone().next(), Some(Ok(Token::Colon))) {
                                // Need to differentiate between parameter type annotation and method end colon
                                // Look ahead to see if this is followed by a type or if it's the method colon
                                let mut lookahead = self.lexer.clone();
                                lookahead.next(); // consume ':'
                                match lookahead.next() {
                                    Some(Ok(Token::Identifier(_))) => {
                                        self.lexer.next(); // consume ':'
                                        self.parse_type_annotation()
                                    }
                                    _ => None,
                                }
                            } else {
                                None
                            };

                        // Check for default value: param = value
                        let default_value =
                            if matches!(self.lexer.clone().next(), Some(Ok(Token::Assign))) {
                                self.lexer.next(); // consume '='
                                self.parse_constant_default_value()
                            } else {
                                None
                            };

                        // Validate: non-default parameters cannot follow default parameters
                        let has_default_params =
                            parameters.iter().any(|p| p.default_value.is_some());
                        if has_default_params && default_value.is_none() {
                            // Error: non-default parameter after default parameter
                            self.had_error = true;
                            return;
                        }

                        if let Some(default) = default_value {
                            parameters.push(Parameter::new_with_default(
                                param_name,
                                type_annotation,
                                default,
                            ));
                        } else {
                            parameters.push(Parameter::new(param_name, type_annotation));
                        }

                        match self.lexer.clone().next() {
                            Some(Ok(Token::Comma)) => {
                                self.lexer.next();
                            }
                            Some(Ok(Token::RParen)) => break,
                            _ => {
                                self.had_error = true;
                                return;
                            }
                        }
                    }
                }

                if self.lexer.next() != Some(Ok(Token::RParen)) {
                    self.had_error = true;
                    return;
                }

                if parameters.len() > u8::MAX as usize {
                    self.had_error = true;
                    return;
                }

                // Check for return type annotation: -> type
                let return_type = if matches!(self.lexer.clone().next(), Some(Ok(Token::Arrow))) {
                    self.lexer.next(); // consume '->'
                    self.parse_type_annotation()
                } else {
                    None
                };

                let method_colon_end = if let Some(Ok(Token::Colon)) = self.lexer.next() {
                    self.lexer.span().end
                } else {
                    self.had_error = true;
                    return;
                };

                let outer_chunk = std::mem::take(&mut self.chunk);
                let outer_loop_stack = std::mem::take(&mut self.loop_stack);

                // Build qualified name for method: ClassName.method_name
                self.function_name_stack.push(method_name.clone());
                let qualname = format!("{}.{}", class_name, method_name);

                self.function_scopes
                    .push(FunctionScope::new_with_params(parameters.clone()));
                self.function_depth += 1;

                // Method body should be indented relative to the method definition (at class_body_indent)
                let body_had_statement = self.parse_suite(class_body_indent, method_colon_end);

                self.function_depth -= 1;
                self.function_name_stack.pop();
                let captured_upvalues = self
                    .function_scopes
                    .pop()
                    .map(|scope| scope.upvalues)
                    .unwrap_or_default();

                if !body_had_statement && !self.had_error {
                    self.had_error = true;
                }

                if !self.had_error && self.chunk.code.last() != Some(&(OpCode::OpReturn as u8)) {
                    self.chunk.code.push(OpCode::OpReturn as u8);
                }

                let function_chunk = std::mem::replace(&mut self.chunk, outer_chunk);
                self.loop_stack = outer_loop_stack;
                self.current_indent = class_body_indent;

                if self.had_error {
                    return;
                }

                // Extract parameter names, types, and default values for the prototype
                let parameter_names: Vec<String> =
                    parameters.iter().map(|p| p.name.clone()).collect();
                let parameter_types: Vec<Option<Type>> = parameters
                    .iter()
                    .map(|p| p.type_annotation.clone())
                    .collect();
                let default_values: Vec<Option<crate::object::Object>> =
                    parameters.iter().map(|p| p.default_value.clone()).collect();

                let type_info = crate::object::TypeInfo {
                    parameter_names,
                    parameter_types,
                    return_type,
                    default_values,
                };

                let mut prototype = FunctionPrototype::new_with_types(
                    method_name.clone(),
                    parameters.len(),
                    function_chunk,
                    captured_upvalues,
                    type_info,
                    self.module.clone(),
                );
                prototype.qualname = qualname;
                let prototype_value = Rc::new(ObjectType::FunctionPrototype(Rc::new(prototype)));
                let prototype_const_idx = self.add_constant(prototype_value);
                self.chunk.code.push(OpCode::OpMakeFunction as u8);
                self.chunk.code.push(prototype_const_idx as u8);

                method_names.push(method_name);
            } else {
                // Skip non-def tokens in class body
                self.had_error = true;
                return;
            }
        }

        if self.had_error {
            return;
        }

        // Methods are already on the stack as functions
        // Now emit each method name after its function
        // Stack will be: [func1] [name1] [func2] [name2] ... [class_name]
        // But methods are already pushed in order during parsing
        // So we need a different approach

        // Actually, let's emit name constants after all functions
        // Stack layout will be: [func1, func2, ...] then we push [name1, name2, ...] [class_name]
        for method_name in method_names.iter() {
            let name_idx = self.add_constant(Rc::new(ObjectType::String(method_name.clone())));
            self.chunk.code.push(OpCode::OpConstant as u8);
            self.chunk.code.push(name_idx as u8);
        }

        // Emit class name constant
        let class_name_idx = self.add_constant(Rc::new(ObjectType::String(class_name.clone())));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(class_name_idx as u8);

        // Emit OpMakeClass with method count
        self.chunk.code.push(OpCode::OpMakeClass as u8);
        self.chunk.code.push(method_names.len() as u8);

        // If there's a parent class, emit OpInherit
        if let Some(parent_name) = parent_name {
            // Get the parent class from globals
            let parent_idx = self.add_constant(Rc::new(ObjectType::String(parent_name)));
            self.chunk.code.push(OpCode::OpGetGlobal as u8);
            self.chunk.code.push(parent_idx as u8);

            // Now we have [class, parent] on stack
            self.chunk.code.push(OpCode::OpInherit as u8);
        }

        // Define class as global
        let define_name_idx = self.add_constant(Rc::new(ObjectType::String(class_name)));
        self.chunk.code.push(OpCode::OpDefineGlobal as u8);
        self.chunk.code.push(define_name_idx as u8);
    }

    fn parse_return_statement(&mut self) {
        self.lexer.next(); // consume 'return'

        if self.function_depth == 0 {
            self.had_error = true;
            return;
        }

        let return_end = self.lexer.span().end;

        let mut has_expression = false;
        if let Some((token_result, info)) = self.peek_token_with_indent() {
            if !self.has_newline_between(return_end, info.start) {
                if let Ok(token) = token_result {
                    if !matches!(token, Token::Semicolon) {
                        has_expression = true;
                    }
                }
            }
        }

        if has_expression {
            if !self.parse_expression() {
                self.had_error = true;
                return;
            }
        } else {
            self.emit_nil();
        }

        self.chunk.code.push(OpCode::OpReturn as u8);
    }

    fn parse_nonlocal_statement(&mut self) {
        self.lexer.next(); // consume 'nonlocal'

        if self.function_depth == 0 {
            self.had_error = true;
            return;
        }

        // Parse comma-separated list of identifiers
        loop {
            match self.lexer.next() {
                Some(Ok(Token::Identifier(name))) => {
                    // Add to nonlocals set in current function scope
                    // The variable doesn't need to exist yet - it will be resolved
                    // when actually used or assigned
                    if let Some(scope) = self.function_scopes.last_mut() {
                        scope.nonlocals.insert(name);
                    }
                }
                _ => {
                    self.had_error = true;
                    return;
                }
            }

            // Check for comma (more variables) or end of statement
            let mut lookahead = self.lexer.clone();
            match lookahead.next() {
                Some(Ok(Token::Comma)) => {
                    self.lexer.next(); // consume comma
                    continue;
                }
                _ => break,
            }
        }
    }

    fn detect_assignment_kind(&self) -> Option<AssignmentKind> {
        let mut lookahead = self.lexer.clone();
        match lookahead.next()? {
            Ok(Token::Identifier(_)) => {}
            _ => return None,
        }

        let mut bracket_depth = 0;
        let mut has_dot = false;

        while let Some(token_result) = lookahead.next() {
            match token_result {
                Ok(Token::LBracket) => bracket_depth += 1,
                Ok(Token::RBracket) => {
                    if bracket_depth == 0 {
                        return None;
                    }
                    bracket_depth -= 1;
                }
                Ok(Token::Dot) if bracket_depth == 0 => {
                    has_dot = true;
                    // Continue to see if there's an assignment after the dot and identifier
                }
                Ok(Token::Assign) if bracket_depth == 0 => return Some(AssignmentKind::Simple),
                Ok(Token::PlusEqual) if bracket_depth == 0 => {
                    return Some(AssignmentKind::AddAssign)
                }
                Ok(Token::StarEqual) if bracket_depth == 0 => {
                    return Some(AssignmentKind::MultiplyAssign)
                }
                Ok(Token::LParen) if bracket_depth == 0 && !has_dot => return None,
                Ok(Token::Comma) | Ok(Token::Semicolon) if bracket_depth == 0 => return None,
                Ok(Token::Plus) | Ok(Token::Slash) | Ok(Token::Star) | Ok(Token::Minus)
                | Ok(Token::In)
                    if bracket_depth == 0 =>
                {
                    return None
                }
                Ok(Token::RParen) if bracket_depth == 0 => return None,
                Ok(Token::Colon) if bracket_depth == 0 => {
                    // Check if this is a type annotation (x: int = value)
                    // Skip the type annotation token if present
                    match lookahead.next() {
                        Some(Ok(Token::Identifier(_))) => {
                            // Continue to check for assignment operator after type annotation
                        }
                        _ => return None,
                    }
                }
                Err(_) => return None,
                _ => {}
            }
        }

        None
    }

    fn parse_print_statement(&mut self) {
        self.lexer.next(); // Consume 'print'
        self.lexer.next(); // Consume '('

        let mut first = true;
        while self.lexer.clone().next() != Some(Ok(Token::RParen)) {
            if !first {
                // Consume the comma separating arguments.
                if self.lexer.clone().next() == Some(Ok(Token::Comma)) {
                    self.lexer.next();
                } else {
                    break;
                }
            }

            if !self.parse_expression() {
                self.had_error = true;
                return;
            }

            if self.lexer.clone().next() == Some(Ok(Token::Comma)) {
                self.chunk.code.push(OpCode::OpPrintSpaced as u8);
            } else {
                self.chunk.code.push(OpCode::OpPrint as u8);
                break;
            }

            first = false;
        }

        self.lexer.next(); // Consume ')'
        self.chunk.code.push(OpCode::OpPrintln as u8);
    }

    fn parse_for_statement(&mut self) {
        self.lexer.next(); // Consume 'for'

        let loop_indent = self.current_indent;

        let loop_var = if let Some(Ok(Token::Identifier(name))) = self.lexer.next() {
            name
        } else {
            self.had_error = true;
            return;
        };

        if self.lexer.next() != Some(Ok(Token::In)) {
            self.had_error = true;
            return;
        }

        let mut loop_var_local = None;
        let mut loop_var_const_idx = None;

        if self.function_depth > 0 {
            if let Some((idx, is_new)) = self.declare_local(loop_var.clone()) {
                loop_var_local = Some(idx);
                if is_new {
                    self.emit_nil();
                }
            }
            if self.had_error {
                return;
            }
        }

        if loop_var_local.is_none() {
            loop_var_const_idx =
                Some(self.add_constant(Rc::new(ObjectType::String(loop_var.clone()))));
        }

        let nil_idx = self.add_constant(Rc::new(ObjectType::Nil));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(nil_idx as u8);
        match loop_var_local {
            Some(local_idx) => {
                self.chunk.code.push(OpCode::OpSetLocal as u8);
                self.chunk.code.push(local_idx as u8);
                self.chunk.code.push(OpCode::OpPop as u8);
            }
            None => {
                let name_idx = loop_var_const_idx.expect("global loop variable must have name");
                self.chunk.code.push(OpCode::OpDefineGlobal as u8);
                self.chunk.code.push(name_idx as u8);
            }
        }

        if !self.parse_expression() {
            self.had_error = true;
            return;
        }

        let colon_end = if let Some(Ok(Token::Colon)) = self.lexer.next() {
            self.lexer.span().end
        } else {
            self.had_error = true;
            return;
        };

        let zero_idx = self.add_constant(Rc::new(ObjectType::Integer(0)));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(zero_idx as u8);

        let loop_start = self.chunk.code.len();
        self.chunk.code.push(OpCode::OpIterNext as u8);
        let iter_jump_pos = self.chunk.code.len();
        self.chunk.code.push(0);
        self.chunk.code.push(0);

        match loop_var_local {
            Some(local_idx) => {
                self.chunk.code.push(OpCode::OpSetLocal as u8);
                self.chunk.code.push(local_idx as u8);
            }
            None => {
                let name_idx = loop_var_const_idx.expect("global loop variable must have name");
                self.chunk.code.push(OpCode::OpSetGlobal as u8);
                self.chunk.code.push(name_idx as u8);
            }
        }
        self.chunk.code.push(OpCode::OpPop as u8);

        self.loop_stack.push(LoopContext::new(2));
        let body_had_statement = self.parse_suite(loop_indent, colon_end);
        let context = self.loop_stack.pop().unwrap_or_else(|| LoopContext::new(2));

        if !body_had_statement && !self.had_error {
            self.had_error = true;
        }

        self.emit_loop(loop_start);
        self.patch_jump(iter_jump_pos);

        for jump in context.break_jumps {
            self.patch_jump(jump);
        }
    }

    fn parse_if_statement(&mut self) {
        self.lexer.next(); // Consume 'if'

        let if_indent = self.current_indent;

        if !self.parse_expression() {
            self.had_error = true;
            return;
        }

        let colon_end = if let Some(Ok(Token::Colon)) = self.lexer.next() {
            self.lexer.span().end
        } else {
            self.had_error = true;
            return;
        };

        let then_jump = self.emit_jump(OpCode::OpJumpIfFalse);
        self.chunk.code.push(OpCode::OpPop as u8);

        let then_had_statement = self.parse_suite(if_indent, colon_end);
        if !then_had_statement && !self.had_error {
            self.had_error = true;
            return;
        }

        let else_is_next = matches!(
            self.peek_token_with_indent(),
            Some((Ok(Token::Else), info)) if info.indent == if_indent
        );

        if else_is_next {
            let else_jump = self.emit_jump(OpCode::OpJump);
            self.patch_jump(then_jump);
            self.chunk.code.push(OpCode::OpPop as u8);

            self.lexer.next(); // Consume 'else'
            let else_colon_end = if let Some(Ok(Token::Colon)) = self.lexer.next() {
                self.lexer.span().end
            } else {
                self.had_error = true;
                return;
            };

            let else_had_statement = self.parse_suite(if_indent, else_colon_end);
            if !else_had_statement && !self.had_error {
                self.had_error = true;
            }

            self.patch_jump(else_jump);
        } else {
            let end_jump = self.emit_jump(OpCode::OpJump);
            self.patch_jump(then_jump);
            self.chunk.code.push(OpCode::OpPop as u8);
            self.patch_jump(end_jump);
        }
    }

    fn parse_while_statement(&mut self) {
        self.lexer.next(); // Consume 'while'

        let loop_indent = self.current_indent;

        let loop_start = self.chunk.code.len();

        if !self.parse_expression() {
            self.had_error = true;
            return;
        }

        let colon_end = if let Some(Ok(Token::Colon)) = self.lexer.next() {
            self.lexer.span().end
        } else {
            self.had_error = true;
            return;
        };

        let exit_jump = self.emit_jump(OpCode::OpJumpIfFalse);
        self.chunk.code.push(OpCode::OpPop as u8);

        self.loop_stack.push(LoopContext::new(0));
        let body_had_statement = self.parse_suite(loop_indent, colon_end);
        let context = self.loop_stack.pop().unwrap_or_else(|| LoopContext::new(0));

        if !body_had_statement && !self.had_error {
            self.had_error = true;
        }

        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.chunk.code.push(OpCode::OpPop as u8);

        for jump in context.break_jumps {
            self.patch_jump(jump);
        }
    }

    fn parse_break_statement(&mut self) {
        self.lexer.next(); // Consume 'break'

        let cleanup_depth = if let Some(context) = self.loop_stack.last() {
            context.cleanup_depth
        } else {
            self.had_error = true;
            return;
        };

        for _ in 0..cleanup_depth {
            self.chunk.code.push(OpCode::OpPop as u8);
        }

        let jump_pos = self.emit_jump(OpCode::OpJump);

        if let Some(context) = self.loop_stack.last_mut() {
            context.break_jumps.push(jump_pos);
        }
    }

    fn parse_assignment_statement(&mut self, kind: AssignmentKind) {
        let name = if let Some(Ok(Token::Identifier(s))) = self.lexer.next() {
            s
        } else {
            return; // Should not happen
        };

        // Check for type annotation: name: type
        let type_annotation = if matches!(self.lexer.clone().next(), Some(Ok(Token::Colon))) {
            self.lexer.next(); // consume ':'
            self.parse_type_annotation()
        } else {
            None
        };

        let mut target = self.resolve_variable(&name);
        let name_idx = self.add_constant(Rc::new(ObjectType::String(name.clone())));

        let mut has_subscript = false;
        let mut has_attribute = false;
        let mut attr_name: Option<String> = None;
        let mut index_expression_code: Option<Vec<u8>> = None;

        // Check for attribute assignment (obj.attr = value)
        if self.lexer.clone().next() == Some(Ok(Token::Dot)) {
            self.lexer.next(); // consume '.'
            attr_name = match self.lexer.next() {
                Some(Ok(Token::Identifier(attr))) => Some(attr),
                _ => {
                    self.had_error = true;
                    return;
                }
            };
            has_attribute = true;

            // Load the object onto the stack
            self.emit_get_variable(name_idx, target);
        } else if self.lexer.clone().next() == Some(Ok(Token::LBracket)) {
            has_subscript = true;
            self.lexer.next(); // Consume '['

            match kind {
                AssignmentKind::Simple => {
                    self.emit_get_variable(name_idx, target);
                }
                AssignmentKind::AddAssign | AssignmentKind::MultiplyAssign => {
                    self.emit_get_variable(name_idx, target);
                    self.emit_get_variable(name_idx, target);
                }
            }

            let expr_start = self.chunk.code.len();
            if !self.parse_expression() {
                self.had_error = true;
                return;
            }
            let expr_end = self.chunk.code.len();
            index_expression_code = Some(self.chunk.code[expr_start..expr_end].to_vec());

            if self.lexer.next() != Some(Ok(Token::RBracket)) {
                self.had_error = true;
                return;
            }
        }

        match kind {
            AssignmentKind::Simple => {
                if self.lexer.next() != Some(Ok(Token::Assign)) {
                    self.had_error = true;
                    return;
                }

                if has_attribute {
                    // Handle attribute assignment: obj.attr = value
                    // Stack already has the object loaded
                    let attr_name_str = attr_name.expect("attribute name");
                    let attr_idx = self.add_constant(Rc::new(ObjectType::String(attr_name_str)));

                    if !self.parse_expression() {
                        self.had_error = true;
                        return;
                    }

                    // Stack: [object, value]
                    self.chunk.code.push(OpCode::OpSetAttr as u8);
                    self.chunk.code.push(attr_idx as u8);
                } else if has_subscript {
                    if !self.parse_expression() {
                        self.had_error = true;
                        return;
                    }

                    self.chunk.code.push(OpCode::OpSetIndex as u8);
                    self.emit_set_variable(name_idx, target);
                    self.chunk.code.push(OpCode::OpPop as u8);
                } else {
                    if self.function_depth > 0 && matches!(target, VariableTarget::Global) {
                        // Check if variable is declared as nonlocal
                        let is_nonlocal = self
                            .function_scopes
                            .last()
                            .map(|scope| scope.nonlocals.contains(&name))
                            .unwrap_or(false);

                        if !is_nonlocal {
                            // Not nonlocal, so create as local variable
                            if let Some((idx, is_new)) =
                                self.declare_local_with_type(name.clone(), type_annotation)
                            {
                                target = VariableTarget::Local(idx);
                                if is_new {
                                    self.emit_nil();
                                }
                            }
                            if self.had_error {
                                return;
                            }
                        } else {
                            // Variable is nonlocal, re-resolve to get upvalue target
                            target = self.resolve_variable(&name);
                        }
                    }

                    if !self.parse_expression() {
                        self.had_error = true;
                        return;
                    }

                    self.emit_define_variable(name_idx, target);
                }
            }
            AssignmentKind::AddAssign | AssignmentKind::MultiplyAssign => {
                if self.function_depth > 0
                    && matches!(target, VariableTarget::Global)
                    && !has_subscript
                {
                    self.had_error = true;
                    return;
                }

                let expected_token = if matches!(kind, AssignmentKind::AddAssign) {
                    Token::PlusEqual
                } else {
                    Token::StarEqual
                };

                if self.lexer.next() != Some(Ok(expected_token)) {
                    self.had_error = true;
                    return;
                }

                let arithmetic_opcode = if matches!(kind, AssignmentKind::AddAssign) {
                    OpCode::OpAdd
                } else {
                    OpCode::OpMultiply
                };

                if has_subscript {
                    self.chunk.code.push(OpCode::OpIndex as u8);

                    if !self.parse_expression() {
                        self.had_error = true;
                        return;
                    }

                    self.chunk.code.push(arithmetic_opcode as u8);

                    if let Some(code) = &index_expression_code {
                        self.chunk.code.extend_from_slice(code);
                    } else {
                        self.had_error = true;
                        return;
                    }

                    self.chunk.code.push(OpCode::OpSwap as u8);
                    self.chunk.code.push(OpCode::OpSetIndex as u8);
                    self.emit_set_variable(name_idx, target);
                    self.chunk.code.push(OpCode::OpPop as u8);
                } else {
                    self.emit_get_variable(name_idx, target);

                    if !self.parse_expression() {
                        self.had_error = true;
                        return;
                    }

                    self.chunk.code.push(arithmetic_opcode as u8);
                    self.emit_set_variable(name_idx, target);
                    self.chunk.code.push(OpCode::OpPop as u8);
                }
            }
        }
    }

    fn parse_expression_statement(&mut self, discard_result: bool) {
        let produced = self.parse_expression();
        if discard_result && produced {
            self.chunk.code.push(OpCode::OpPop as u8);
        } else if !produced {
            self.had_error = true;
        }
    }

    /// Parses a constant default value for a function parameter.
    /// Only supports literal values: integers, floats, strings, True, False, None.
    /// Returns None if the token is not a valid constant.
    fn parse_constant_default_value(&mut self) -> Option<crate::object::Object> {
        match self.lexer.next() {
            Some(Ok(Token::Integer(val))) => Some(Rc::new(ObjectType::Integer(val))),
            Some(Ok(Token::Float(val))) => Some(Rc::new(ObjectType::Float(val))),
            Some(Ok(Token::String(val))) => Some(Rc::new(ObjectType::String(val))),
            Some(Ok(Token::True)) => Some(Rc::new(ObjectType::Boolean(true))),
            Some(Ok(Token::False)) => Some(Rc::new(ObjectType::Boolean(false))),
            Some(Ok(Token::Identifier(ref s))) if s == "None" => Some(Rc::new(ObjectType::Nil)),
            _ => {
                self.had_error = true;
                None
            }
        }
    }
}
