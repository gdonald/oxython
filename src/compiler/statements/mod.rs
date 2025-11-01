//! Statement parsing for the compiler.
//!
//! This module contains functions for parsing Python statements including
//! function definitions, class definitions, control flow statements (if, while, for),
//! assignments, and expression statements.

mod assignments;
mod control_flow;
mod definitions;
mod simple;

use crate::token::Token;

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

    pub(super) fn has_newline_between(&self, start: usize, end: usize) -> bool {
        self.source[start..end].chars().any(|ch| ch == '\n')
    }

    pub(super) fn parse_suite(&mut self, parent_indent: usize, colon_end: usize) -> bool {
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
}
