use crate::bytecode::{Chunk, OpCode};
use crate::object::{Object, ObjectType};
use crate::token::Token;
use logos::{Lexer, Logos};
use std::rc::Rc;

pub struct Compiler<'a> {
    lexer: Lexer<'a, Token>,
    chunk: Chunk,
    had_error: bool,
    list_comp_counter: usize,
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

impl<'a> Compiler<'a> {
    pub fn compile(source: &'a str) -> Option<Chunk> {
        let mut compiler = Compiler {
            lexer: Token::lexer(source),
            chunk: Chunk::new(),
            had_error: false,
            list_comp_counter: 0,
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

    fn parse_statement(&mut self) {
        if let Some(Ok(token)) = self.lexer.clone().next() {
            match token {
                Token::Print => self.parse_print_statement(),
                Token::For => self.parse_for_statement(),
                Token::While => self.parse_while_statement(),
                Token::If => self.parse_if_statement(),
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

        let var_const_idx = self.add_constant(Rc::new(ObjectType::String(loop_var.clone())));
        let nil_idx = self.add_constant(Rc::new(ObjectType::Nil));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(nil_idx as u8);
        self.chunk.code.push(OpCode::OpDefineGlobal as u8);
        self.chunk.code.push(var_const_idx as u8);

        if !self.parse_expression() {
            self.had_error = true;
            return;
        }

        if self.lexer.next() != Some(Ok(Token::Colon)) {
            self.had_error = true;
            return;
        }

        let zero_idx = self.add_constant(Rc::new(ObjectType::Integer(0)));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(zero_idx as u8);

        let loop_start = self.chunk.code.len();
        self.chunk.code.push(OpCode::OpIterNext as u8);
        let iter_jump_pos = self.chunk.code.len();
        self.chunk.code.push(0);
        self.chunk.code.push(0);

        self.chunk.code.push(OpCode::OpSetGlobal as u8);
        self.chunk.code.push(var_const_idx as u8);
        self.chunk.code.push(OpCode::OpPop as u8);

        self.parse_statement();

        self.emit_loop(loop_start);
        self.patch_jump(iter_jump_pos);
    }

    fn parse_if_statement(&mut self) {
        self.lexer.next(); // Consume 'if'

        if !self.parse_expression() {
            self.had_error = true;
            return;
        }

        if self.lexer.next() != Some(Ok(Token::Colon)) {
            self.had_error = true;
            return;
        }

        let then_jump = self.emit_jump(OpCode::OpJumpIfFalse);
        self.chunk.code.push(OpCode::OpPop as u8);

        self.parse_statement();

        if self.lexer.clone().next() == Some(Ok(Token::Else)) {
            let else_jump = self.emit_jump(OpCode::OpJump);
            self.patch_jump(then_jump);
            self.chunk.code.push(OpCode::OpPop as u8);

            self.lexer.next(); // Consume 'else'
            if self.lexer.next() != Some(Ok(Token::Colon)) {
                self.had_error = true;
                return;
            }

            self.parse_statement();
            self.patch_jump(else_jump);
        } else {
            self.patch_jump(then_jump);
            self.chunk.code.push(OpCode::OpPop as u8);
        }
    }

    fn parse_while_statement(&mut self) {
        self.lexer.next(); // Consume 'while'

        let loop_start = self.chunk.code.len();

        if !self.parse_expression() {
            self.had_error = true;
            return;
        }

        if self.lexer.next() != Some(Ok(Token::Colon)) {
            self.had_error = true;
            return;
        }

        let exit_jump = self.emit_jump(OpCode::OpJumpIfFalse);
        self.chunk.code.push(OpCode::OpPop as u8);

        self.parse_statement();

        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.chunk.code.push(OpCode::OpPop as u8);
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
                    let const_idx = self.add_constant(Rc::new(ObjectType::String(name.clone())));
                    self.chunk.code.push(OpCode::OpGetGlobal as u8);
                    self.chunk.code.push(const_idx as u8);
                    self.parse_postfix(Some(const_idx))
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
        element_code: Vec<u8>,
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
        let nil_idx = self.add_constant(Rc::new(ObjectType::Nil));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(nil_idx as u8);
        self.chunk.code.push(OpCode::OpDefineGlobal as u8);
        self.chunk.code.push(loop_var_idx as u8);

        let result_name = self.next_list_comp_result_name();
        let result_name_idx = self.add_constant(Rc::new(ObjectType::String(result_name.clone())));
        let empty_list_idx = self.add_constant(Rc::new(ObjectType::List(Vec::new())));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(empty_list_idx as u8);
        self.chunk.code.push(OpCode::OpDefineGlobal as u8);
        self.chunk.code.push(result_name_idx as u8);

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

        self.chunk.code.push(OpCode::OpSetGlobal as u8);
        self.chunk.code.push(loop_var_idx as u8);
        self.chunk.code.push(OpCode::OpPop as u8);

        if let Some(code) = filter_code {
            self.chunk.code.extend_from_slice(&code);
            let skip_append = self.emit_jump(OpCode::OpJumpIfFalse);
            self.chunk.code.push(OpCode::OpPop as u8);

            self.chunk.code.push(OpCode::OpGetGlobal as u8);
            self.chunk.code.push(result_name_idx as u8);
            self.chunk.code.extend_from_slice(&element_code);
            self.chunk.code.push(OpCode::OpAppend as u8);
            self.chunk.code.push(OpCode::OpSetGlobal as u8);
            self.chunk.code.push(result_name_idx as u8);
            self.chunk.code.push(OpCode::OpPop as u8);

            let after_append = self.emit_jump(OpCode::OpJump);
            self.patch_jump(skip_append);
            self.chunk.code.push(OpCode::OpPop as u8);
            self.patch_jump(after_append);
        } else {
            self.chunk.code.push(OpCode::OpGetGlobal as u8);
            self.chunk.code.push(result_name_idx as u8);
            self.chunk.code.extend_from_slice(&element_code);
            self.chunk.code.push(OpCode::OpAppend as u8);
            self.chunk.code.push(OpCode::OpSetGlobal as u8);
            self.chunk.code.push(result_name_idx as u8);
            self.chunk.code.push(OpCode::OpPop as u8);
        }

        self.emit_loop(loop_start);
        self.patch_jump(iter_jump_pos);

        self.chunk.code.push(OpCode::OpGetGlobal as u8);
        self.chunk.code.push(result_name_idx as u8);

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
        let name_idx = self.add_constant(Rc::new(ObjectType::String(name.clone())));

        let mut has_subscript = false;
        let mut index_expression_code: Option<Vec<u8>> = None;

        if self.lexer.clone().next() == Some(Ok(Token::LBracket)) {
            has_subscript = true;
            self.lexer.next(); // Consume '['

            match kind {
                AssignmentKind::Simple => {
                    self.chunk.code.push(OpCode::OpGetGlobal as u8);
                    self.chunk.code.push(name_idx as u8);
                }
                AssignmentKind::AddAssign | AssignmentKind::MultiplyAssign => {
                    self.chunk.code.push(OpCode::OpGetGlobal as u8);
                    self.chunk.code.push(name_idx as u8);
                    self.chunk.code.push(OpCode::OpGetGlobal as u8);
                    self.chunk.code.push(name_idx as u8);
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
                    self.chunk.code.push(OpCode::OpSetGlobal as u8);
                    self.chunk.code.push(name_idx as u8);
                    self.chunk.code.push(OpCode::OpPop as u8);
                } else {
                    if !self.parse_expression() {
                        self.had_error = true;
                        return;
                    }

                    self.chunk.code.push(OpCode::OpDefineGlobal as u8);
                    self.chunk.code.push(name_idx as u8);
                }
            }
            AssignmentKind::AddAssign | AssignmentKind::MultiplyAssign => {
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
                    self.chunk.code.push(OpCode::OpSetGlobal as u8);
                    self.chunk.code.push(name_idx as u8);
                    self.chunk.code.push(OpCode::OpPop as u8);
                } else {
                    self.chunk.code.push(OpCode::OpGetGlobal as u8);
                    self.chunk.code.push(name_idx as u8);

                    if !self.parse_expression() {
                        self.had_error = true;
                        return;
                    }

                    self.chunk.code.push(arithmetic_opcode as u8);
                    self.chunk.code.push(OpCode::OpSetGlobal as u8);
                    self.chunk.code.push(name_idx as u8);
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
