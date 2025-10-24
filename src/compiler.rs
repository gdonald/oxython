use crate::bytecode::{Chunk, OpCode};
use crate::object::{FunctionPrototype, Object, ObjectType, UpvalueDescriptor};
use crate::token::Token;
use logos::{Lexer, Logos};
use std::collections::HashMap;
use std::rc::Rc;

pub struct Compiler<'a> {
    lexer: Lexer<'a, Token>,
    source: &'a str,
    chunk: Chunk,
    had_error: bool,
    list_comp_counter: usize,
    loop_stack: Vec<LoopContext>,
    current_indent: usize,
    function_depth: usize,
    function_scopes: Vec<FunctionScope>,
}

enum AssignmentKind {
    Simple,
    AddAssign,
    MultiplyAssign,
}

enum FStringSegment {
    Literal(String),
    Identifier(String),
}

enum ComprehensionEnd {
    RBracket,
    RParen,
}

#[derive(Clone, Copy)]
enum VariableTarget {
    Local(usize),
    Upvalue(usize),
    Global,
}

struct LoopContext {
    break_jumps: Vec<usize>,
    cleanup_depth: usize,
}

impl LoopContext {
    fn new(cleanup_depth: usize) -> Self {
        LoopContext {
            break_jumps: Vec::new(),
            cleanup_depth,
        }
    }
}

struct TokenInfo {
    indent: usize,
    start: usize,
}

struct FunctionScope {
    parameters: Vec<String>,
    locals: Vec<String>,
    upvalues: Vec<UpvalueDescriptor>,
    upvalue_map: HashMap<String, usize>,
}

impl FunctionScope {
    fn new(parameters: Vec<String>) -> Self {
        FunctionScope {
            parameters,
            locals: Vec::new(),
            upvalues: Vec::new(),
            upvalue_map: HashMap::new(),
        }
    }

    fn resolve(&self, name: &str) -> Option<usize> {
        if let Some(idx) = self.parameters.iter().position(|param| param == name) {
            return Some(idx + 1);
        }

        if let Some(idx) = self.locals.iter().position(|local| local == name) {
            return Some(self.parameters.len() + 1 + idx);
        }

        None
    }

    fn declare(&mut self, name: String) -> (usize, bool) {
        if let Some(idx) = self.parameters.iter().position(|param| param == &name) {
            return (idx + 1, false);
        }

        if let Some(idx) = self.locals.iter().position(|local| local == &name) {
            return (self.parameters.len() + 1 + idx, false);
        }

        self.locals.push(name);
        let idx = self.locals.len() - 1;
        (self.parameters.len() + 1 + idx, true)
    }

    fn add_upvalue(&mut self, name: String, is_local: bool, index: usize) -> usize {
        if let Some(existing) = self.upvalue_map.get(&name) {
            return *existing;
        }

        let upvalue_index = self.upvalues.len();
        self.upvalues.push(UpvalueDescriptor { is_local, index });
        self.upvalue_map.insert(name, upvalue_index);
        upvalue_index
    }

    fn resolve_upvalue(&self, name: &str) -> Option<usize> {
        self.upvalue_map.get(name).copied()
    }
}

impl<'a> Compiler<'a> {
    pub fn compile(source: &'a str) -> Option<Chunk> {
        let mut compiler = Compiler {
            lexer: Token::lexer(source),
            source,
            chunk: Chunk::new(),
            had_error: false,
            list_comp_counter: 0,
            loop_stack: Vec::new(),
            current_indent: 0,
            function_depth: 0,
            function_scopes: Vec::new(),
        };

        // Loop until we run out of tokens
        while let Some(next_token) = compiler.lexer.clone().next() {
            match next_token {
                Ok(_) => {
                    compiler.parse_statement();

                    // Consume any trailing semicolons after a statement
                    while compiler.lexer.clone().next() == Some(Ok(Token::Semicolon)) {
                        compiler.lexer.next();
                    }
                }
                Err(_) => {
                    // Advance past unrecognized input to avoid infinite loops.
                    compiler.lexer.next();
                    compiler.had_error = true;
                }
            }
        }
        if compiler.had_error {
            return None;
        }

        compiler.chunk.code.push(OpCode::OpReturn as u8);

        Some(compiler.chunk)
    }

    fn peek_token_with_indent(&self) -> Option<(Result<Token, ()>, TokenInfo)> {
        let mut lookahead = self.lexer.clone();
        let token = lookahead.next()?;
        let span = lookahead.span();
        let indent = self.indent_at(span.start);
        Some((
            token,
            TokenInfo {
                indent,
                start: span.start,
            },
        ))
    }

    fn indent_at(&self, position: usize) -> usize {
        let line_start = self.source[..position]
            .rfind('\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let mut indent = 0;
        for ch in self.source[line_start..position].chars() {
            match ch {
                ' ' => indent += 1,
                '\t' => indent += 4,
                _ => break,
            }
        }
        indent
    }

    fn parse_statement(&mut self) {
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
            Token::Return => self.parse_return_statement(),
            Token::Break => self.parse_break_statement(),
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

        let mut parameters: Vec<String> = Vec::new();
        if self.lexer.clone().next() != Some(Ok(Token::RParen)) {
            loop {
                match self.lexer.next() {
                    Some(Ok(Token::Identifier(param))) => parameters.push(param),
                    _ => {
                        self.had_error = true;
                        return;
                    }
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

        let colon_end = if let Some(Ok(Token::Colon)) = self.lexer.next() {
            self.lexer.span().end
        } else {
            self.had_error = true;
            return;
        };

        let outer_chunk = std::mem::take(&mut self.chunk);
        let outer_loop_stack = std::mem::take(&mut self.loop_stack);
        let parent_indent = self.current_indent;

        self.function_scopes
            .push(FunctionScope::new(parameters.clone()));
        self.function_depth += 1;

        let body_had_statement = self.parse_suite(parent_indent, colon_end);

        self.function_depth -= 1;
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

        let prototype_value = Rc::new(ObjectType::FunctionPrototype(Rc::new(
            FunctionPrototype::new(
                name.clone(),
                parameters.len(),
                function_chunk,
                captured_upvalues,
            ),
        )));
        let prototype_const_idx = self.add_constant(prototype_value);
        self.chunk.code.push(OpCode::OpMakeFunction as u8);
        self.chunk.code.push(prototype_const_idx as u8);

        let name_idx = self.add_constant(Rc::new(ObjectType::String(name)));
        self.chunk.code.push(OpCode::OpDefineGlobal as u8);
        self.chunk.code.push(name_idx as u8);
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

    fn detect_assignment_kind(&self) -> Option<AssignmentKind> {
        let mut lookahead = self.lexer.clone();
        match lookahead.next()? {
            Ok(Token::Identifier(_)) => {}
            _ => return None,
        }

        let mut bracket_depth = 0;

        for token_result in lookahead {
            match token_result {
                Ok(Token::LBracket) => bracket_depth += 1,
                Ok(Token::RBracket) => {
                    if bracket_depth == 0 {
                        return None;
                    }
                    bracket_depth -= 1;
                }
                Ok(Token::Assign) if bracket_depth == 0 => return Some(AssignmentKind::Simple),
                Ok(Token::PlusEqual) if bracket_depth == 0 => {
                    return Some(AssignmentKind::AddAssign)
                }
                Ok(Token::StarEqual) if bracket_depth == 0 => {
                    return Some(AssignmentKind::MultiplyAssign)
                }
                Ok(Token::LParen) | Ok(Token::Dot) if bracket_depth == 0 => return None,
                Ok(Token::Comma) | Ok(Token::Semicolon) if bracket_depth == 0 => return None,
                Ok(Token::Plus) | Ok(Token::Slash) | Ok(Token::Star) | Ok(Token::Minus)
                | Ok(Token::In)
                    if bracket_depth == 0 =>
                {
                    return None
                }
                Ok(Token::RParen) if bracket_depth == 0 => return None,
                Ok(Token::Colon) if bracket_depth == 0 => return None,
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

    fn parse_expression(&mut self) -> bool {
        // Extremely simplified expression parser for "value" or "value + value"
        let mut operands = 0;

        if let Some(Ok(token)) = self.lexer.next() {
            if self.parse_term(token) {
                operands += 1;
            } else {
                self.had_error = true;
                return false;
            }
        } else {
            return false;
        }

        while let Some(Ok(next_token)) = self.lexer.clone().next() {
            let opcode = match next_token {
                Token::Plus => Some(OpCode::OpAdd),
                Token::Slash => Some(OpCode::OpDivide),
                Token::Star => Some(OpCode::OpMultiply),
                Token::Minus => Some(OpCode::OpSubtract),
                Token::Less => Some(OpCode::OpLess),
                Token::In => Some(OpCode::OpContains),
                Token::Percent => Some(OpCode::OpModulo),
                Token::EqualEqual => Some(OpCode::OpEqual),
                _ => None,
            };

            let Some(opcode) = opcode else { break };

            self.lexer.next(); // Consume the operator
            let mut term_produced = false;
            if let Some(Ok(token)) = self.lexer.next() {
                term_produced = self.parse_term(token); // Parse the next term
                if term_produced {
                    operands += 1;
                }
            }
            if term_produced && operands >= 2 {
                self.chunk.code.push(opcode as u8);
                operands -= 1; // Two operands consumed, one result pushed.
            } else {
                self.had_error = true;
                break;
            }
        }

        if operands == 0 {
            self.had_error = true;
        }

        operands > 0
    }

    fn parse_term(&mut self, token: Token) -> bool {
        match token {
            Token::Minus => {
                let zero_idx = self.add_constant(Rc::new(ObjectType::Integer(0)));
                self.chunk.code.push(OpCode::OpConstant as u8);
                self.chunk.code.push(zero_idx as u8);

                if let Some(Ok(next)) = self.lexer.next() {
                    if self.parse_term(next) {
                        self.chunk.code.push(OpCode::OpSubtract as u8);
                        true
                    } else {
                        self.had_error = true;
                        false
                    }
                } else {
                    self.had_error = true;
                    false
                }
            }
            Token::Integer(val) => {
                let const_idx = self.add_constant(Rc::new(ObjectType::Integer(val)));
                self.chunk.code.push(OpCode::OpConstant as u8);
                self.chunk.code.push(const_idx as u8);
                true
            }
            Token::True => {
                let const_idx = self.add_constant(Rc::new(ObjectType::Boolean(true)));
                self.chunk.code.push(OpCode::OpConstant as u8);
                self.chunk.code.push(const_idx as u8);
                true
            }
            Token::False => {
                let const_idx = self.add_constant(Rc::new(ObjectType::Boolean(false)));
                self.chunk.code.push(OpCode::OpConstant as u8);
                self.chunk.code.push(const_idx as u8);
                true
            }
            Token::Identifier(name) => {
                if name == "f" && matches!(self.lexer.clone().next(), Some(Ok(Token::String(_)))) {
                    return self.parse_f_string_literal();
                } else if name == "list"
                    && matches!(self.lexer.clone().next(), Some(Ok(Token::LParen)))
                {
                    return self.parse_list_call();
                } else if name == "zip"
                    && matches!(self.lexer.clone().next(), Some(Ok(Token::LParen)))
                {
                    return self.parse_zip_call();
                }

                if name == "len" && self.lexer.clone().next() == Some(Ok(Token::LParen)) {
                    self.lexer.next(); // consume '('
                    if !self.parse_expression() {
                        self.had_error = true;
                        return false;
                    }
                    if self.lexer.next() != Some(Ok(Token::RParen)) {
                        self.had_error = true;
                        return false;
                    }
                    self.chunk.code.push(OpCode::OpLen as u8);
                    true
                } else if name == "round" && self.lexer.clone().next() == Some(Ok(Token::LParen)) {
                    self.lexer.next(); // consume '('
                    if !self.parse_expression() {
                        self.had_error = true;
                        return false;
                    }
                    if self.lexer.next() != Some(Ok(Token::Comma)) {
                        self.had_error = true;
                        return false;
                    }
                    if !self.parse_expression() {
                        self.had_error = true;
                        return false;
                    }
                    if self.lexer.next() != Some(Ok(Token::RParen)) {
                        self.had_error = true;
                        return false;
                    }
                    self.chunk.code.push(OpCode::OpRound as u8);
                    true
                } else if name == "range" && self.lexer.clone().next() == Some(Ok(Token::LParen)) {
                    self.lexer.next(); // consume '('
                    if !self.parse_expression() {
                        self.had_error = true;
                        return false;
                    }
                    if self.lexer.next() != Some(Ok(Token::Comma)) {
                        self.had_error = true;
                        return false;
                    }
                    if !self.parse_expression() {
                        self.had_error = true;
                        return false;
                    }
                    if self.lexer.next() != Some(Ok(Token::RParen)) {
                        self.had_error = true;
                        return false;
                    }
                    self.chunk.code.push(OpCode::OpRange as u8);
                    true
                } else {
                    match self.resolve_variable(&name) {
                        VariableTarget::Local(local_index) => {
                            self.chunk.code.push(OpCode::OpGetLocal as u8);
                            self.chunk.code.push(local_index as u8);
                            self.parse_postfix(None)
                        }
                        VariableTarget::Upvalue(upvalue_index) => {
                            self.chunk.code.push(OpCode::OpGetUpvalue as u8);
                            self.chunk.code.push(upvalue_index as u8);
                            self.parse_postfix(None)
                        }
                        VariableTarget::Global => {
                            let const_idx =
                                self.add_constant(Rc::new(ObjectType::String(name.clone())));
                            self.chunk.code.push(OpCode::OpGetGlobal as u8);
                            self.chunk.code.push(const_idx as u8);
                            self.parse_postfix(Some(const_idx))
                        }
                    }
                }
            }
            Token::String(val) => {
                let const_idx = self.add_constant(Rc::new(ObjectType::String(val)));
                self.chunk.code.push(OpCode::OpConstant as u8);
                self.chunk.code.push(const_idx as u8);
                self.parse_postfix(None)
            }
            Token::Float(val) => {
                let const_idx = self.add_constant(Rc::new(ObjectType::Float(val)));
                self.chunk.code.push(OpCode::OpConstant as u8);
                self.chunk.code.push(const_idx as u8);
                true
            }
            Token::LBrace => {
                if let Some(dict_constant) = self.parse_dict_literal() {
                    let const_idx = self.add_constant(dict_constant);
                    self.chunk.code.push(OpCode::OpConstant as u8);
                    self.chunk.code.push(const_idx as u8);
                    true
                } else {
                    false
                }
            }
            Token::LBracket => {
                if self.is_list_comprehension() {
                    self.parse_list_comprehension()
                } else if let Some(list_constant) = self.parse_list_literal() {
                    let const_idx = self.add_constant(list_constant);
                    self.chunk.code.push(OpCode::OpConstant as u8);
                    self.chunk.code.push(const_idx as u8);
                    true
                } else {
                    false
                }
            }
            _ => false, // Not a term, do nothing
        }
    }

    fn parse_postfix(&mut self, mut base_name_idx: Option<usize>) -> bool {
        loop {
            match self.lexer.clone().next() {
                Some(Ok(Token::LParen)) => {
                    self.lexer.next(); // consume '('
                    let mut arg_count: u8 = 0;

                    if self.lexer.clone().next() != Some(Ok(Token::RParen)) {
                        loop {
                            if !self.parse_expression() {
                                self.had_error = true;
                                return false;
                            }
                            if arg_count == u8::MAX {
                                self.had_error = true;
                                return false;
                            }
                            arg_count += 1;

                            match self.lexer.clone().next() {
                                Some(Ok(Token::Comma)) => {
                                    self.lexer.next();
                                }
                                Some(Ok(Token::RParen)) => break,
                                _ => {
                                    self.had_error = true;
                                    return false;
                                }
                            }
                        }
                    }

                    if self.lexer.next() != Some(Ok(Token::RParen)) {
                        self.had_error = true;
                        return false;
                    }

                    self.chunk.code.push(OpCode::OpCall as u8);
                    self.chunk.code.push(arg_count);
                    base_name_idx = None;
                }
                Some(Ok(Token::LBracket)) => {
                    self.lexer.next(); // consume '['
                    base_name_idx = None;

                    let mut start_pushed = false;
                    let next_token = self.lexer.clone().next();
                    if next_token != Some(Ok(Token::Colon))
                        && next_token != Some(Ok(Token::RBracket))
                    {
                        if !self.parse_expression() {
                            self.had_error = true;
                            return false;
                        }
                        start_pushed = true;
                    }

                    if self.lexer.clone().next() == Some(Ok(Token::Colon)) {
                        self.lexer.next(); // consume ':'

                        if !start_pushed {
                            let nil_idx = self.add_constant(Rc::new(ObjectType::Nil));
                            self.chunk.code.push(OpCode::OpConstant as u8);
                            self.chunk.code.push(nil_idx as u8);
                        }

                        let mut end_pushed = false;
                        match self.lexer.clone().next() {
                            Some(Ok(Token::RBracket)) | Some(Ok(Token::Colon)) => {}
                            _ => {
                                if !self.parse_expression() {
                                    self.had_error = true;
                                    return false;
                                }
                                end_pushed = true;
                            }
                        }

                        if !end_pushed {
                            let nil_idx = self.add_constant(Rc::new(ObjectType::Nil));
                            self.chunk.code.push(OpCode::OpConstant as u8);
                            self.chunk.code.push(nil_idx as u8);
                        }

                        let mut step_pushed = false;
                        if self.lexer.clone().next() == Some(Ok(Token::Colon)) {
                            self.lexer.next(); // consume second ':'
                            if self.lexer.clone().next() != Some(Ok(Token::RBracket)) {
                                if !self.parse_expression() {
                                    self.had_error = true;
                                    return false;
                                }
                                step_pushed = true;
                            }
                        }

                        if !step_pushed {
                            let nil_idx = self.add_constant(Rc::new(ObjectType::Nil));
                            self.chunk.code.push(OpCode::OpConstant as u8);
                            self.chunk.code.push(nil_idx as u8);
                        }

                        if self.lexer.next() != Some(Ok(Token::RBracket)) {
                            self.had_error = true;
                            return false;
                        }

                        self.chunk.code.push(OpCode::OpSlice as u8);
                    } else {
                        if self.lexer.next() != Some(Ok(Token::RBracket)) {
                            self.had_error = true;
                            return false;
                        }

                        self.chunk.code.push(OpCode::OpIndex as u8);
                    }
                }
                Some(Ok(Token::Dot)) => {
                    self.lexer.next(); // consume '.'
                    let method = match self.lexer.next() {
                        Some(Ok(Token::Identifier(m))) => m,
                        _ => {
                            self.had_error = true;
                            return false;
                        }
                    };

                    match method.as_str() {
                        "append" => {
                            if base_name_idx.is_none() {
                                self.had_error = true;
                                return false;
                            }
                            if self.lexer.next() != Some(Ok(Token::LParen)) {
                                self.had_error = true;
                                return false;
                            }
                            if !self.parse_expression() {
                                self.had_error = true;
                                return false;
                            }
                            if self.lexer.next() != Some(Ok(Token::RParen)) {
                                self.had_error = true;
                                return false;
                            }
                            self.chunk.code.push(OpCode::OpAppend as u8);
                            self.chunk.code.push(OpCode::OpSetGlobal as u8);
                            self.chunk
                                .code
                                .push(base_name_idx.expect("base variable index") as u8);
                        }
                        "lower" => {
                            if self.lexer.next() != Some(Ok(Token::LParen)) {
                                self.had_error = true;
                                return false;
                            }
                            if self.lexer.next() != Some(Ok(Token::RParen)) {
                                self.had_error = true;
                                return false;
                            }
                            self.chunk.code.push(OpCode::OpStrLower as u8);
                            base_name_idx = None;
                        }
                        "isalnum" => {
                            if self.lexer.next() != Some(Ok(Token::LParen)) {
                                self.had_error = true;
                                return false;
                            }
                            if self.lexer.next() != Some(Ok(Token::RParen)) {
                                self.had_error = true;
                                return false;
                            }
                            self.chunk.code.push(OpCode::OpStrIsAlnum as u8);
                            base_name_idx = None;
                        }
                        "join" => {
                            if !self.parse_join_call() {
                                return false;
                            }
                            base_name_idx = None;
                        }
                        _ => {
                            self.had_error = true;
                            return false;
                        }
                    }
                }
                _ => break,
            }
        }

        true
    }

    fn parse_join_call(&mut self) -> bool {
        if self.lexer.next() != Some(Ok(Token::LParen)) {
            self.had_error = true;
            return false;
        }

        let arg_start = self.chunk.code.len();
        if !self.parse_expression() {
            self.had_error = true;
            return false;
        }
        let arg_end = self.chunk.code.len();
        let element_code = self.chunk.code[arg_start..arg_end].to_vec();
        self.chunk.code.truncate(arg_start);

        if self.lexer.clone().next() == Some(Ok(Token::For)) {
            if !self.compile_comprehension(element_code, ComprehensionEnd::RParen) {
                return false;
            }
        } else {
            self.chunk.code.extend_from_slice(&element_code);
            if self.lexer.next() != Some(Ok(Token::RParen)) {
                self.had_error = true;
                return false;
            }
        }

        self.chunk.code.push(OpCode::OpStrJoin as u8);
        true
    }

    fn parse_f_string_literal(&mut self) -> bool {
        let template = match self.lexer.next() {
            Some(Ok(Token::String(template))) => template,
            _ => {
                self.had_error = true;
                return false;
            }
        };

        let segments = match Self::f_string_segments(&template) {
            Ok(segments) => segments,
            Err(_) => {
                self.had_error = true;
                return false;
            }
        };

        if segments.is_empty() {
            let const_idx = self.add_constant(Rc::new(ObjectType::String(String::new())));
            self.chunk.code.push(OpCode::OpConstant as u8);
            self.chunk.code.push(const_idx as u8);
            return true;
        }

        for (index, segment) in segments.into_iter().enumerate() {
            match segment {
                FStringSegment::Literal(text) => {
                    let const_idx = self.add_constant(Rc::new(ObjectType::String(text)));
                    self.chunk.code.push(OpCode::OpConstant as u8);
                    self.chunk.code.push(const_idx as u8);
                }
                FStringSegment::Identifier(identifier) => {
                    let name_idx = self.add_constant(Rc::new(ObjectType::String(identifier)));
                    self.chunk.code.push(OpCode::OpGetGlobal as u8);
                    self.chunk.code.push(name_idx as u8);
                }
            }

            if index > 0 {
                self.chunk.code.push(OpCode::OpAdd as u8);
            }
        }

        true
    }

    fn parse_list_call(&mut self) -> bool {
        self.lexer.next(); // consume '('

        if self.lexer.clone().next() == Some(Ok(Token::RParen)) {
            self.lexer.next(); // consume ')'
            let empty_list_idx = self.add_constant(Rc::new(ObjectType::List(Vec::new())));
            self.chunk.code.push(OpCode::OpConstant as u8);
            self.chunk.code.push(empty_list_idx as u8);
            return true;
        }

        if !self.parse_expression() {
            self.had_error = true;
            return false;
        }

        if self.lexer.clone().next() == Some(Ok(Token::Comma)) {
            self.had_error = true;
            return false;
        }

        if self.lexer.next() != Some(Ok(Token::RParen)) {
            self.had_error = true;
            return false;
        }

        self.chunk.code.push(OpCode::OpToList as u8);
        true
    }

    fn parse_zip_call(&mut self) -> bool {
        self.lexer.next(); // consume '('

        if self.lexer.clone().next() == Some(Ok(Token::RParen)) {
            self.lexer.next(); // consume ')'
            let empty_list_idx = self.add_constant(Rc::new(ObjectType::List(Vec::new())));
            self.chunk.code.push(OpCode::OpConstant as u8);
            self.chunk.code.push(empty_list_idx as u8);
            return true;
        }

        let mut arg_count: u16 = 0;
        let mut star_mask: u16 = 0;

        loop {
            let is_star = matches!(self.lexer.clone().next(), Some(Ok(Token::Star)));
            if is_star {
                self.lexer.next(); // consume '*'
            }

            if !self.parse_expression() {
                self.had_error = true;
                return false;
            }

            if arg_count >= 16 {
                self.had_error = true;
                return false;
            }

            if is_star {
                star_mask |= 1 << arg_count;
            }

            arg_count += 1;

            if self.lexer.clone().next() == Some(Ok(Token::Comma)) {
                self.lexer.next(); // consume ','
                continue;
            }
            break;
        }

        if self.lexer.next() != Some(Ok(Token::RParen)) {
            self.had_error = true;
            return false;
        }

        self.chunk.code.push(OpCode::OpZip as u8);
        self.chunk.code.push(arg_count as u8);
        self.chunk.code.push(((star_mask >> 8) & 0xff) as u8);
        self.chunk.code.push((star_mask & 0xff) as u8);
        true
    }

    fn f_string_segments(template: &str) -> Result<Vec<FStringSegment>, ()> {
        let mut segments = Vec::new();
        let mut chars = template.chars().peekable();
        let mut current_literal = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    if chars.peek() == Some(&'{') {
                        chars.next();
                        current_literal.push('{');
                        continue;
                    }

                    if !current_literal.is_empty() {
                        segments.push(FStringSegment::Literal(std::mem::take(
                            &mut current_literal,
                        )));
                    }

                    let mut expr = String::new();
                    let mut found_closing = false;
                    for next_ch in chars.by_ref() {
                        if next_ch == '}' {
                            found_closing = true;
                            break;
                        }
                        if next_ch == '{' {
                            return Err(());
                        }
                        expr.push(next_ch);
                    }

                    if !found_closing {
                        return Err(());
                    }

                    let trimmed = expr.trim();
                    if trimmed.is_empty() || !Self::is_valid_identifier(trimmed) {
                        return Err(());
                    }

                    segments.push(FStringSegment::Identifier(trimmed.to_string()));
                }
                '}' => {
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        current_literal.push('}');
                    } else {
                        return Err(());
                    }
                }
                _ => current_literal.push(ch),
            }
        }

        if !current_literal.is_empty() {
            segments.push(FStringSegment::Literal(current_literal));
        }

        Ok(segments)
    }

    fn is_valid_identifier(name: &str) -> bool {
        let mut chars = name.chars();
        match chars.next() {
            Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {}
            _ => return false,
        }

        chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    }

    fn is_list_comprehension(&self) -> bool {
        let lookahead = self.lexer.clone();
        let mut bracket_depth = 0;

        for token_result in lookahead {
            match token_result {
                Ok(Token::For) if bracket_depth == 0 => return true,
                Ok(Token::RBracket) if bracket_depth == 0 => return false,
                Ok(Token::LBracket) => bracket_depth += 1,
                Ok(Token::RBracket) => {
                    if bracket_depth == 0 {
                        return false;
                    }
                    bracket_depth -= 1;
                }
                Err(_) => return false,
                _ => {}
            }
        }

        false
    }

    fn parse_list_comprehension(&mut self) -> bool {
        let element_start = self.chunk.code.len();
        if !self.parse_expression() {
            self.had_error = true;
            return false;
        }
        let element_end = self.chunk.code.len();
        let element_code = self.chunk.code[element_start..element_end].to_vec();
        self.chunk.code.truncate(element_start);

        self.compile_comprehension(element_code, ComprehensionEnd::RBracket)
    }

    fn compile_comprehension(
        &mut self,
        mut element_code: Vec<u8>,
        terminator: ComprehensionEnd,
    ) -> bool {
        if self.lexer.next() != Some(Ok(Token::For)) {
            self.had_error = true;
            return false;
        }

        let loop_var = match self.lexer.next() {
            Some(Ok(Token::Identifier(name))) => name,
            _ => {
                self.had_error = true;
                return false;
            }
        };

        if self.lexer.next() != Some(Ok(Token::In)) {
            self.had_error = true;
            return false;
        }

        let loop_var_idx = self.add_constant(Rc::new(ObjectType::String(loop_var.clone())));
        let mut loop_var_local = None;
        if self.function_depth > 0 {
            if let Some((idx, is_new)) = self.declare_local(loop_var.clone()) {
                loop_var_local = Some(idx);
                if is_new {
                    self.emit_nil();
                }
            }
            if self.had_error {
                return false;
            }
        }
        let target_indices = self.constant_indices_for_string(&loop_var);
        if let Some(local_idx) = loop_var_local {
            self.rewrite_globals_to_local(&mut element_code, &target_indices, local_idx);
        }

        let nil_idx = self.add_constant(Rc::new(ObjectType::Nil));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(nil_idx as u8);
        if let Some(local_idx) = loop_var_local {
            self.chunk.code.push(OpCode::OpSetLocal as u8);
            self.chunk.code.push(local_idx as u8);
            self.chunk.code.push(OpCode::OpPop as u8);
        } else {
            self.chunk.code.push(OpCode::OpDefineGlobal as u8);
            self.chunk.code.push(loop_var_idx as u8);
        }

        let result_name = self.next_list_comp_result_name();
        let result_name_idx = self.add_constant(Rc::new(ObjectType::String(result_name.clone())));
        let mut result_local = None;
        if self.function_depth > 0 {
            if let Some((idx, is_new)) = self.declare_local(result_name.clone()) {
                result_local = Some(idx);
                if is_new {
                    self.emit_nil();
                }
            }
            if self.had_error {
                return false;
            }
        }
        let empty_list_idx = self.add_constant(Rc::new(ObjectType::List(Vec::new())));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(empty_list_idx as u8);
        if let Some(local_idx) = result_local {
            self.chunk.code.push(OpCode::OpSetLocal as u8);
            self.chunk.code.push(local_idx as u8);
            self.chunk.code.push(OpCode::OpPop as u8);
        } else {
            self.chunk.code.push(OpCode::OpDefineGlobal as u8);
            self.chunk.code.push(result_name_idx as u8);
        }

        if !self.parse_expression() {
            self.had_error = true;
            return false;
        }

        let filter_code = if self.lexer.clone().next() == Some(Ok(Token::If)) {
            self.lexer.next(); // consume 'if'
            let filter_start = self.chunk.code.len();
            if !self.parse_expression() {
                self.had_error = true;
                return false;
            }
            let filter_end = self.chunk.code.len();
            let code = self.chunk.code[filter_start..filter_end].to_vec();
            self.chunk.code.truncate(filter_start);
            let mut code = code;
            if let Some(local_idx) = loop_var_local {
                self.rewrite_globals_to_local(&mut code, &target_indices, local_idx);
            }
            Some(code)
        } else {
            None
        };

        let expected_terminator = match terminator {
            ComprehensionEnd::RBracket => Token::RBracket,
            ComprehensionEnd::RParen => Token::RParen,
        };

        if self.lexer.next() != Some(Ok(expected_terminator)) {
            self.had_error = true;
            return false;
        }

        let zero_idx = self.add_constant(Rc::new(ObjectType::Integer(0)));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(zero_idx as u8);

        let loop_start = self.chunk.code.len();
        self.chunk.code.push(OpCode::OpIterNext as u8);
        let iter_jump_pos = self.chunk.code.len();
        self.chunk.code.push(0);
        self.chunk.code.push(0);

        if let Some(local_idx) = loop_var_local {
            self.chunk.code.push(OpCode::OpSetLocal as u8);
            self.chunk.code.push(local_idx as u8);
        } else {
            self.chunk.code.push(OpCode::OpSetGlobal as u8);
            self.chunk.code.push(loop_var_idx as u8);
        }
        self.chunk.code.push(OpCode::OpPop as u8);

        if let Some(code) = filter_code {
            self.chunk.code.extend_from_slice(&code);
            let skip_append = self.emit_jump(OpCode::OpJumpIfFalse);
            self.chunk.code.push(OpCode::OpPop as u8);

            if let Some(local_idx) = result_local {
                self.chunk.code.push(OpCode::OpGetLocal as u8);
                self.chunk.code.push(local_idx as u8);
            } else {
                self.chunk.code.push(OpCode::OpGetGlobal as u8);
                self.chunk.code.push(result_name_idx as u8);
            }
            self.chunk.code.extend_from_slice(&element_code);
            self.chunk.code.push(OpCode::OpAppend as u8);
            if let Some(local_idx) = result_local {
                self.chunk.code.push(OpCode::OpSetLocal as u8);
                self.chunk.code.push(local_idx as u8);
            } else {
                self.chunk.code.push(OpCode::OpSetGlobal as u8);
                self.chunk.code.push(result_name_idx as u8);
            }
            self.chunk.code.push(OpCode::OpPop as u8);

            let after_append = self.emit_jump(OpCode::OpJump);
            self.patch_jump(skip_append);
            self.chunk.code.push(OpCode::OpPop as u8);
            self.patch_jump(after_append);
        } else {
            if let Some(local_idx) = result_local {
                self.chunk.code.push(OpCode::OpGetLocal as u8);
                self.chunk.code.push(local_idx as u8);
            } else {
                self.chunk.code.push(OpCode::OpGetGlobal as u8);
                self.chunk.code.push(result_name_idx as u8);
            }
            self.chunk.code.extend_from_slice(&element_code);
            self.chunk.code.push(OpCode::OpAppend as u8);
            if let Some(local_idx) = result_local {
                self.chunk.code.push(OpCode::OpSetLocal as u8);
                self.chunk.code.push(local_idx as u8);
            } else {
                self.chunk.code.push(OpCode::OpSetGlobal as u8);
                self.chunk.code.push(result_name_idx as u8);
            }
            self.chunk.code.push(OpCode::OpPop as u8);
        }

        self.emit_loop(loop_start);
        self.patch_jump(iter_jump_pos);

        if let Some(local_idx) = result_local {
            self.chunk.code.push(OpCode::OpGetLocal as u8);
            self.chunk.code.push(local_idx as u8);
        } else {
            self.chunk.code.push(OpCode::OpGetGlobal as u8);
            self.chunk.code.push(result_name_idx as u8);
        }

        true
    }

    fn parse_list_literal(&mut self) -> Option<Object> {
        let mut elements: Vec<Object> = Vec::new();

        loop {
            match self.lexer.clone().next() {
                Some(Ok(Token::RBracket)) => {
                    self.lexer.next(); // consume closing bracket
                    break;
                }
                Some(Ok(Token::Comma)) => {
                    self.lexer.next(); // consume comma between elements
                }
                Some(Ok(Token::Integer(value))) => {
                    self.lexer.next(); // consume integer
                    elements.push(Rc::new(ObjectType::Integer(value)));
                }
                Some(Ok(Token::Float(value))) => {
                    self.lexer.next(); // consume float
                    elements.push(Rc::new(ObjectType::Float(value)));
                }
                Some(Ok(Token::String(value))) => {
                    self.lexer.next(); // consume string
                    elements.push(Rc::new(ObjectType::String(value)));
                }
                Some(Ok(Token::LBracket)) => {
                    self.lexer.next(); // consume '['
                    if let Some(nested) = self.parse_list_literal() {
                        elements.push(nested);
                    } else {
                        return None;
                    }
                }
                Some(Ok(Token::LBrace)) => {
                    self.lexer.next(); // consume '{'
                    if let Some(dict) = self.parse_dict_literal() {
                        elements.push(dict);
                    } else {
                        return None;
                    }
                }
                Some(Ok(Token::Identifier(_))) => {
                    self.had_error = true;
                    return None;
                }
                _ => {
                    self.had_error = true;
                    return None;
                }
            }
        }

        Some(Rc::new(ObjectType::List(elements)))
    }

    fn parse_dict_literal(&mut self) -> Option<Object> {
        let mut entries: Vec<(String, Object)> = Vec::new();

        loop {
            match self.lexer.clone().next() {
                Some(Ok(Token::RBrace)) => {
                    self.lexer.next(); // consume closing brace
                    break;
                }
                Some(Ok(Token::Comma)) => {
                    self.lexer.next(); // consume comma between entries
                }
                Some(Ok(Token::String(key))) => {
                    self.lexer.next(); // consume key

                    if self.lexer.next() != Some(Ok(Token::Colon)) {
                        self.had_error = true;
                        return None;
                    }

                    let value = match self.lexer.next() {
                        Some(Ok(Token::Integer(value))) => Rc::new(ObjectType::Integer(value)),
                        Some(Ok(Token::Float(value))) => Rc::new(ObjectType::Float(value)),
                        Some(Ok(Token::String(value))) => Rc::new(ObjectType::String(value)),
                        _ => {
                            self.had_error = true;
                            return None;
                        }
                    };

                    if let Some(position) = entries
                        .iter()
                        .position(|(existing_key, _)| existing_key == &key)
                    {
                        entries[position].1 = value;
                    } else {
                        entries.push((key, value));
                    }
                }
                _ => {
                    self.had_error = true;
                    return None;
                }
            }
        }

        Some(Rc::new(ObjectType::Dict(entries)))
    }

    fn parse_assignment_statement(&mut self, kind: AssignmentKind) {
        let name = if let Some(Ok(Token::Identifier(s))) = self.lexer.next() {
            s
        } else {
            return; // Should not happen
        };
        let mut target = self.resolve_variable(&name);
        let name_idx = self.add_constant(Rc::new(ObjectType::String(name.clone())));

        let mut has_subscript = false;
        let mut index_expression_code: Option<Vec<u8>> = None;

        if self.lexer.clone().next() == Some(Ok(Token::LBracket)) {
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

                if has_subscript {
                    if !self.parse_expression() {
                        self.had_error = true;
                        return;
                    }

                    self.chunk.code.push(OpCode::OpSetIndex as u8);
                    self.emit_set_variable(name_idx, target);
                    self.chunk.code.push(OpCode::OpPop as u8);
                } else {
                    if self.function_depth > 0 && matches!(target, VariableTarget::Global) {
                        if let Some((idx, is_new)) = self.declare_local(name.clone()) {
                            target = VariableTarget::Local(idx);
                            if is_new {
                                self.emit_nil();
                            }
                        }
                        if self.had_error {
                            return;
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

    fn resolve_local(&self, name: &str) -> Option<usize> {
        self.function_scopes
            .last()
            .and_then(|scope| scope.resolve(name))
    }

    fn resolve_variable(&mut self, name: &str) -> VariableTarget {
        if let Some(local) = self.resolve_local(name) {
            VariableTarget::Local(local)
        } else if let Some(upvalue) = self.resolve_upvalue(name) {
            VariableTarget::Upvalue(upvalue)
        } else {
            VariableTarget::Global
        }
    }

    fn resolve_upvalue(&mut self, name: &str) -> Option<usize> {
        if self.function_scopes.len() < 2 {
            return None;
        }

        let current_index = self.function_scopes.len() - 1;

        if let Some(existing) = self.function_scopes[current_index].resolve_upvalue(name) {
            return Some(existing);
        }

        self.resolve_upvalue_recursive(current_index, name)
    }

    fn resolve_upvalue_recursive(&mut self, scope_index: usize, name: &str) -> Option<usize> {
        if scope_index == 0 {
            return None;
        }

        let parent_index = scope_index - 1;
        let parent_local = {
            let parent_scope = &self.function_scopes[parent_index];
            parent_scope.resolve(name)
        };

        if let Some(local_index) = parent_local {
            let name_owned = name.to_string();
            let scope = self
                .function_scopes
                .get_mut(scope_index)
                .expect("scope should exist");
            return Some(scope.add_upvalue(name_owned, true, local_index));
        }

        let parent_upvalue_index = self.resolve_upvalue_recursive(parent_index, name)?;

        let name_owned = name.to_string();
        let scope = self
            .function_scopes
            .get_mut(scope_index)
            .expect("scope should exist");
        Some(scope.add_upvalue(name_owned, false, parent_upvalue_index))
    }

    fn declare_local(&mut self, name: String) -> Option<(usize, bool)> {
        let scope = self.function_scopes.last_mut()?;

        if scope.parameters.len() + scope.locals.len() + 1 >= u8::MAX as usize {
            self.had_error = true;
            return None;
        }

        Some(scope.declare(name))
    }

    fn emit_nil(&mut self) {
        let nil_idx = self.add_constant(Rc::new(ObjectType::Nil));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(nil_idx as u8);
    }

    fn emit_get_variable(&mut self, name_idx: usize, target: VariableTarget) {
        match target {
            VariableTarget::Local(local) => {
                self.chunk.code.push(OpCode::OpGetLocal as u8);
                self.chunk.code.push(local as u8);
            }
            VariableTarget::Upvalue(upvalue) => {
                self.chunk.code.push(OpCode::OpGetUpvalue as u8);
                self.chunk.code.push(upvalue as u8);
            }
            VariableTarget::Global => {
                self.chunk.code.push(OpCode::OpGetGlobal as u8);
                self.chunk.code.push(name_idx as u8);
            }
        }
    }

    fn emit_set_variable(&mut self, name_idx: usize, target: VariableTarget) {
        match target {
            VariableTarget::Local(local) => {
                self.chunk.code.push(OpCode::OpSetLocal as u8);
                self.chunk.code.push(local as u8);
            }
            VariableTarget::Upvalue(upvalue) => {
                self.chunk.code.push(OpCode::OpSetUpvalue as u8);
                self.chunk.code.push(upvalue as u8);
            }
            VariableTarget::Global => {
                self.chunk.code.push(OpCode::OpSetGlobal as u8);
                self.chunk.code.push(name_idx as u8);
            }
        }
    }

    fn emit_define_variable(&mut self, name_idx: usize, target: VariableTarget) {
        match target {
            VariableTarget::Local(local) => {
                self.chunk.code.push(OpCode::OpSetLocal as u8);
                self.chunk.code.push(local as u8);
                self.chunk.code.push(OpCode::OpPop as u8);
            }
            VariableTarget::Upvalue(upvalue) => {
                self.chunk.code.push(OpCode::OpSetUpvalue as u8);
                self.chunk.code.push(upvalue as u8);
                self.chunk.code.push(OpCode::OpPop as u8);
            }
            VariableTarget::Global => {
                self.chunk.code.push(OpCode::OpDefineGlobal as u8);
                self.chunk.code.push(name_idx as u8);
            }
        }
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.chunk.code.push(instruction as u8);
        let operand_index = self.chunk.code.len();
        self.chunk.code.push(0);
        self.chunk.code.push(0);
        operand_index
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.chunk.code.push(OpCode::OpLoop as u8);
        let operand_index = self.chunk.code.len();
        self.chunk.code.push(0);
        self.chunk.code.push(0);
        let offset = self.chunk.code.len() - loop_start;
        self.chunk.code[operand_index] = ((offset >> 8) & 0xff) as u8;
        self.chunk.code[operand_index + 1] = (offset & 0xff) as u8;
    }

    fn patch_jump(&mut self, operand_index: usize) {
        let jump = self.chunk.code.len() - (operand_index + 2);
        self.chunk.code[operand_index] = ((jump >> 8) & 0xff) as u8;
        self.chunk.code[operand_index + 1] = (jump & 0xff) as u8;
    }

    fn constant_indices_for_string(&self, name: &str) -> Vec<usize> {
        self.chunk
            .constants
            .iter()
            .enumerate()
            .filter_map(|(idx, value)| match &**value {
                ObjectType::String(existing) if existing == name => Some(idx),
                _ => None,
            })
            .collect()
    }

    fn rewrite_globals_to_local(
        &self,
        code: &mut [u8],
        target_indices: &[usize],
        local_slot: usize,
    ) {
        if target_indices.is_empty() {
            return;
        }

        let mut i = 0;
        while i < code.len() {
            let opcode = OpCode::from(code[i]);
            match opcode {
                OpCode::OpGetGlobal => {
                    if i + 1 < code.len() {
                        let idx = code[i + 1] as usize;
                        if target_indices.contains(&idx) {
                            code[i] = OpCode::OpGetLocal as u8;
                            code[i + 1] = local_slot as u8;
                        }
                    }
                    i += 1 + 1;
                }
                OpCode::OpSetGlobal => {
                    if i + 1 < code.len() {
                        let idx = code[i + 1] as usize;
                        if target_indices.contains(&idx) {
                            code[i] = OpCode::OpSetLocal as u8;
                            code[i + 1] = local_slot as u8;
                        }
                    }
                    i += 1 + 1;
                }
                _ => {
                    i += 1 + Self::opcode_operand_width(opcode);
                }
            }
        }
    }

    fn opcode_operand_width(opcode: OpCode) -> usize {
        match opcode {
            OpCode::OpConstant
            | OpCode::OpDefineGlobal
            | OpCode::OpGetGlobal
            | OpCode::OpSetGlobal
            | OpCode::OpCall
            | OpCode::OpGetLocal
            | OpCode::OpSetLocal
            | OpCode::OpGetUpvalue
            | OpCode::OpSetUpvalue
            | OpCode::OpMakeFunction => 1,
            OpCode::OpIterNext | OpCode::OpLoop | OpCode::OpJumpIfFalse | OpCode::OpJump => 2,
            OpCode::OpZip => 3,
            _ => 0,
        }
    }

    fn next_list_comp_result_name(&mut self) -> String {
        let name = format!("__list_comp_result_{}", self.list_comp_counter);
        self.list_comp_counter += 1;
        name
    }

    fn add_constant(&mut self, value: Object) -> usize {
        self.chunk.constants.push(value);
        self.chunk.constants.len() - 1
    }
}
