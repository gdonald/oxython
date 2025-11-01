//! Control flow statement parsing (if, while, for, break).

use crate::bytecode::OpCode;
use crate::object::ObjectType;
use crate::token::Token;
use std::rc::Rc;

use super::super::types::*;

impl super::super::Compiler<'_> {
    pub(super) fn parse_if_statement(&mut self) {
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

    pub(super) fn parse_while_statement(&mut self) {
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

    pub(super) fn parse_for_statement(&mut self) {
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

    pub(super) fn parse_break_statement(&mut self) {
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
}
