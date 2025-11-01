//! Assignment statement parsing.

use crate::bytecode::OpCode;
use crate::object::ObjectType;
use crate::token::Token;
use std::rc::Rc;

use super::super::types::*;

impl super::super::Compiler<'_> {
    pub(super) fn detect_assignment_kind(&self) -> Option<AssignmentKind> {
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

    pub(super) fn parse_assignment_statement(&mut self, kind: AssignmentKind) {
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
}
