//! Expression parsing for the compiler.
//!
//! This module contains functions for parsing expressions, terms, and postfix
//! operations including function calls, indexing, slicing, and method calls.

use crate::bytecode::OpCode;
use crate::object::ObjectType;
use crate::token::Token;
use std::rc::Rc;

use super::types::VariableTarget;

impl super::Compiler<'_> {
    /// Parses a simple binary expression: term (operator term)*
    /// Supports operators: +, -, *, /, %, <, ==, in
    pub(super) fn parse_expression(&mut self) -> bool {
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

    /// Parses a term (primary expression) including literals, identifiers,
    /// unary minus, built-in functions, and collection literals.
    pub(super) fn parse_term(&mut self, token: Token) -> bool {
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

                if name == "type" && self.lexer.clone().next() == Some(Ok(Token::LParen)) {
                    self.lexer.next(); // consume '('
                    if !self.parse_expression() {
                        self.had_error = true;
                        return false;
                    }
                    if self.lexer.next() != Some(Ok(Token::RParen)) {
                        self.had_error = true;
                        return false;
                    }
                    self.chunk.code.push(OpCode::OpType as u8);
                    true
                } else if name == "len" && self.lexer.clone().next() == Some(Ok(Token::LParen)) {
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

    /// Parses postfix operations: function calls, indexing, slicing, and attribute access.
    /// Handles chained operations like `obj.method()[0].attr`.
    pub(super) fn parse_postfix(&mut self, mut base_name_idx: Option<usize>) -> bool {
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
                    let attr_name = match self.lexer.next() {
                        Some(Ok(Token::Identifier(name))) => name,
                        _ => {
                            self.had_error = true;
                            return false;
                        }
                    };

                    // Check for built-in methods first (for lists, strings, etc.)
                    match attr_name.as_str() {
                        "append" => {
                            // Handle list.append(value) -> OpAppend + OpSetGlobal
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
                            // Handle string.lower() -> OpStrLower
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
                        _ => {
                            // General attribute access for user-defined classes
                            let attr_idx =
                                self.add_constant(Rc::new(ObjectType::String(attr_name)));
                            self.chunk.code.push(OpCode::OpGetAttr as u8);
                            self.chunk.code.push(attr_idx as u8);
                            base_name_idx = None;
                        }
                    }
                }
                _ => break,
            }
        }

        true
    }
}
