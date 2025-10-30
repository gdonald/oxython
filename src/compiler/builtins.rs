//! Built-in function call parsing for the compiler.
//!
//! This module contains functions for parsing built-in Python functions
//! like join(), zip(), list(), and f-string literals.

use crate::bytecode::OpCode;
use crate::object::ObjectType;
use crate::token::Token;
use std::rc::Rc;

use super::types::{ComprehensionEnd, FStringSegment};

impl super::Compiler<'_> {
    /// Parses a str.join() call or join(iterable) comprehension.
    /// Supports both simple calls and comprehension expressions.
    #[allow(dead_code)]
    pub(super) fn parse_join_call(&mut self) -> bool {
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

    /// Parses an f-string literal: f"Hello {name}!"
    /// Interpolates variables into the string by converting {name} to variable lookups.
    pub(super) fn parse_f_string_literal(&mut self) -> bool {
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

    /// Parses a list() constructor call: list(iterable)
    /// Converts an iterable to a list.
    pub(super) fn parse_list_call(&mut self) -> bool {
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

    /// Parses a zip() call: zip(iter1, iter2, *iter3)
    /// Supports unpacking iterables with the * operator.
    pub(super) fn parse_zip_call(&mut self) -> bool {
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

    /// Parses an f-string template into segments of literals and identifiers.
    /// Handles escape sequences like {{ and }}.
    pub(super) fn f_string_segments(template: &str) -> Result<Vec<FStringSegment>, ()> {
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

    /// Checks if a string is a valid Python identifier.
    /// Must start with a letter or underscore, followed by letters, digits, or underscores.
    pub(super) fn is_valid_identifier(name: &str) -> bool {
        let mut chars = name.chars();
        match chars.next() {
            Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {}
            _ => return false,
        }

        chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    }
}
