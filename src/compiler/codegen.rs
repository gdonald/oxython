//! Code generation utilities for the compiler.
//!
//! This module contains functions for emitting bytecode instructions,
//! managing jumps, and manipulating the constant pool.

use crate::bytecode::OpCode;
use crate::object::{Object, ObjectType};
use std::rc::Rc;

use super::types::VariableTarget;

impl super::Compiler<'_> {
    /// Emits bytecode to push nil onto the stack.
    pub(super) fn emit_nil(&mut self) {
        let nil_idx = self.add_constant(Rc::new(ObjectType::Nil));
        self.chunk.code.push(OpCode::OpConstant as u8);
        self.chunk.code.push(nil_idx as u8);
    }

    /// Emits bytecode to get a variable value.
    pub(super) fn emit_get_variable(&mut self, name_idx: usize, target: VariableTarget) {
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

    /// Emits bytecode to set a variable value.
    pub(super) fn emit_set_variable(&mut self, name_idx: usize, target: VariableTarget) {
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

    /// Emits bytecode to define a variable (set and optionally pop).
    pub(super) fn emit_define_variable(&mut self, name_idx: usize, target: VariableTarget) {
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

    /// Emits a jump instruction and returns the index where the operand should be patched.
    pub(super) fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.chunk.code.push(instruction as u8);
        let operand_index = self.chunk.code.len();
        self.chunk.code.push(0);
        self.chunk.code.push(0);
        operand_index
    }

    /// Emits a loop instruction that jumps backward to the given position.
    pub(super) fn emit_loop(&mut self, loop_start: usize) {
        self.chunk.code.push(OpCode::OpLoop as u8);
        let operand_index = self.chunk.code.len();
        self.chunk.code.push(0);
        self.chunk.code.push(0);
        let offset = self.chunk.code.len() - loop_start;
        self.chunk.code[operand_index] = ((offset >> 8) & 0xff) as u8;
        self.chunk.code[operand_index + 1] = (offset & 0xff) as u8;
    }

    /// Patches a previously emitted jump instruction with the correct offset.
    pub(super) fn patch_jump(&mut self, operand_index: usize) {
        let jump = self.chunk.code.len() - (operand_index + 2);
        self.chunk.code[operand_index] = ((jump >> 8) & 0xff) as u8;
        self.chunk.code[operand_index + 1] = (jump & 0xff) as u8;
    }

    /// Returns all constant indices that contain the given string.
    pub(super) fn constant_indices_for_string(&self, name: &str) -> Vec<usize> {
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

    /// Rewrites OpGetGlobal and OpSetGlobal instructions to use local variables instead.
    /// This is used for list comprehensions where globals need to become locals.
    pub(super) fn rewrite_globals_to_local(
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

    /// Returns the number of operand bytes for a given opcode.
    pub(super) fn opcode_operand_width(opcode: OpCode) -> usize {
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

    /// Adds a constant to the constant pool and returns its index.
    pub(super) fn add_constant(&mut self, value: Object) -> usize {
        self.chunk.constants.push(value);
        self.chunk.constants.len() - 1
    }
}
