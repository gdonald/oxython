//! Function and class definition parsing.

use crate::bytecode::OpCode;
use crate::object::{FunctionPrototype, ObjectType, Type, UpvalueDescriptor};
use crate::token::Token;
use std::rc::Rc;

use super::super::types::*;

impl super::super::Compiler<'_> {
    pub(super) fn parse_function_statement(&mut self) {
        self.lexer.next(); // consume 'def'

        let name = match self.lexer.next() {
            Some(Ok(Token::Identifier(identifier))) => identifier,
            _ => {
                self.had_error = true;
                return;
            }
        };

        let (
            parameters,
            return_type,
            _body_had_statement,
            function_chunk,
            captured_upvalues,
            qualname,
        ) = match self.parse_function_definition(name.clone(), None, None) {
            Some(result) => result,
            None => return,
        };

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

    pub(super) fn parse_class_statement(&mut self) {
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

                // Parse the method using shared function parsing helper
                let (
                    parameters,
                    return_type,
                    _body_had_statement,
                    function_chunk,
                    captured_upvalues,
                    qualname,
                ) = match self.parse_function_definition(
                    method_name.clone(),
                    Some(class_body_indent),
                    Some(&class_name),
                ) {
                    Some(result) => result,
                    None => return,
                };

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
        // Emit name constants after all functions
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

    /// Parses a function or method definition, extracting the shared logic.
    /// Returns: (parameters, return_type, body_had_statement, function_chunk, captured_upvalues, qualname)
    /// If method_indent is Some, this is a method and the body should be parsed at that indent level.
    /// If class_name is Some, builds qualname as ClassName.method_name instead of using function_name_stack.
    #[allow(clippy::type_complexity)]
    fn parse_function_definition(
        &mut self,
        name: String,
        method_indent: Option<usize>,
        class_name: Option<&str>,
    ) -> Option<(
        Vec<Parameter>,
        Option<Type>,
        bool,
        crate::bytecode::Chunk,
        Vec<UpvalueDescriptor>,
        String,
    )> {
        if self.lexer.next() != Some(Ok(Token::LParen)) {
            self.had_error = true;
            return None;
        }

        let mut parameters: Vec<Parameter> = Vec::new();
        if self.lexer.clone().next() != Some(Ok(Token::RParen)) {
            loop {
                let param_name = match self.lexer.next() {
                    Some(Ok(Token::Identifier(param))) => param,
                    _ => {
                        self.had_error = true;
                        return None;
                    }
                };

                // Check for type annotation: param: type
                let type_annotation = if matches!(self.lexer.clone().next(), Some(Ok(Token::Colon)))
                {
                    // For methods, need to differentiate between parameter type annotation and method end colon
                    if method_indent.is_some() {
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
                        self.lexer.next(); // consume ':'
                        self.parse_type_annotation()
                    }
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
                    return None;
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
                        return None;
                    }
                }
            }
        }

        if self.lexer.next() != Some(Ok(Token::RParen)) {
            self.had_error = true;
            return None;
        }

        if parameters.len() > u8::MAX as usize {
            self.had_error = true;
            return None;
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
            return None;
        };

        let outer_chunk = std::mem::take(&mut self.chunk);
        let outer_loop_stack = std::mem::take(&mut self.loop_stack);
        let parent_indent = method_indent.unwrap_or(self.current_indent);

        // Build qualified name by pushing function name
        self.function_name_stack.push(name.clone());

        self.function_scopes
            .push(FunctionScope::new_with_params(parameters.clone()));
        self.function_depth += 1;

        let body_had_statement = self.parse_suite(parent_indent, colon_end);

        self.function_depth -= 1;

        // Build qualified name BEFORE popping from the stack
        let qualname = if let Some(class_name) = class_name {
            format!("{}.{}", class_name, name)
        } else {
            self.function_name_stack.join(".")
        };

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
        self.current_indent = method_indent.unwrap_or(parent_indent);

        if self.had_error {
            return None;
        }

        Some((
            parameters,
            return_type,
            body_had_statement,
            function_chunk,
            captured_upvalues,
            qualname,
        ))
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
