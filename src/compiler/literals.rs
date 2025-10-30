//! Literal and collection parsing for the compiler.
//!
//! This module contains functions for parsing literal values including
//! lists, dictionaries, and list comprehensions.

use crate::bytecode::OpCode;
use crate::object::{Object, ObjectType};
use crate::token::Token;
use std::rc::Rc;

use super::types::ComprehensionEnd;

impl super::Compiler<'_> {
    /// Checks if the current position in the token stream is a list comprehension.
    /// A list comprehension contains a 'for' keyword at the top level before the closing bracket.
    pub(super) fn is_list_comprehension(&self) -> bool {
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

    /// Parses a list comprehension expression: [expr for var in iterable]
    /// or [expr for var in iterable if condition]
    pub(super) fn parse_list_comprehension(&mut self) -> bool {
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

    /// Compiles a comprehension (list or generator) into bytecode.
    /// Handles the iteration, optional filtering, and result accumulation.
    pub(super) fn compile_comprehension(
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

    /// Parses a list literal: [1, 2, 3] or ["a", "b", "c"]
    /// Supports nested lists and dictionaries as elements.
    pub(super) fn parse_list_literal(&mut self) -> Option<Object> {
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

    /// Parses a dictionary literal: {"key1": value1, "key2": value2}
    /// Keys must be strings, values can be integers, floats, or strings.
    /// Duplicate keys are handled by keeping the last value.
    pub(super) fn parse_dict_literal(&mut self) -> Option<Object> {
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

    /// Generates a unique name for a list comprehension result variable.
    pub(super) fn next_list_comp_result_name(&mut self) -> String {
        let name = format!("__list_comp_result_{}", self.list_comp_counter);
        self.list_comp_counter += 1;
        name
    }
}
