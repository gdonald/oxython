//! Simple statement parsing (print, return, nonlocal, expression).

use crate::bytecode::OpCode;
use crate::token::Token;

impl super::super::Compiler<'_> {
    pub(super) fn parse_print_statement(&mut self) {
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

    pub(super) fn parse_return_statement(&mut self) {
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

    pub(super) fn parse_nonlocal_statement(&mut self) {
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

    pub(super) fn parse_expression_statement(&mut self, discard_result: bool) {
        let produced = self.parse_expression();
        if discard_result && produced {
            self.chunk.code.push(OpCode::OpPop as u8);
        } else if !produced {
            self.had_error = true;
        }
    }
}
