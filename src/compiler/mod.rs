//! Compiler module for converting Python source code to bytecode.
//!
//! This module implements a single-pass compiler that parses Python syntax
//! and generates bytecode instructions for the virtual machine.

mod builtins;
mod codegen;
mod expressions;
mod literals;
mod scope;
mod statements;
mod types;

use crate::bytecode::{Chunk, OpCode};
use crate::object::Type;
use crate::token::Token;
use logos::{Lexer, Logos};
use std::collections::HashMap;
use types::*;

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
    module: String,
    function_name_stack: Vec<String>, // Stack of function names for building qualified names
    global_type_annotations: HashMap<String, Type>, // Type annotations for global variables
}

impl<'a> Compiler<'a> {
    pub fn compile(source: &'a str) -> Option<Chunk> {
        Self::compile_with_module(source, "<script>")
    }

    pub fn compile_with_module(source: &'a str, module: &str) -> Option<Chunk> {
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
            module: module.to_string(),
            function_name_stack: Vec::new(),
            global_type_annotations: HashMap::new(),
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

    /// Peeks at the next token along with its indentation information.
    #[allow(private_interfaces)]
    pub(super) fn peek_token_with_indent(&self) -> Option<(Result<Token, ()>, TokenInfo)> {
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

    /// Calculates the indentation level at a given position in the source code.
    pub(super) fn indent_at(&self, position: usize) -> usize {
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
}
