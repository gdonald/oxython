//! Bytecode reading utilities for the VM.
//!
//! This module provides helper methods for reading bytecode instructions
//! and operands from the current execution frame.

use super::VM;
use crate::bytecode::Chunk;

impl VM {
    /// Get a reference to the current call frame's bytecode chunk.
    pub(super) fn current_chunk(&self) -> &Chunk {
        &self
            .frames
            .last()
            .expect("expected active call frame")
            .function
            .chunk
    }

    /// Read a single byte from the current instruction pointer and advance it.
    pub(super) fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("expected active call frame");
        let byte = frame.function.chunk.code[frame.ip];
        frame.ip += 1;
        byte
    }

    /// Read a 16-bit unsigned integer (big-endian) from the instruction stream.
    pub(super) fn read_u16(&mut self) -> usize {
        let high = self.read_byte() as usize;
        let low = self.read_byte() as usize;
        (high << 8) | low
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{Chunk, OpCode};
    use crate::object::FunctionObject;
    use std::rc::Rc;

    #[test]
    fn test_read_byte() {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();
        chunk.code.push(OpCode::OpConstant as u8);
        chunk.code.push(42);
        chunk.code.push(OpCode::OpReturn as u8);

        let function = Rc::new(FunctionObject::new(
            "test".to_string(),
            0,
            chunk,
            Vec::new(),
            "test".to_string(),
        ));

        vm.frames.push(crate::vm::call_frame::CallFrame::new(
            function, 0, None, None,
        ));

        assert_eq!(vm.read_byte(), OpCode::OpConstant as u8);
        assert_eq!(vm.read_byte(), 42);
        assert_eq!(vm.read_byte(), OpCode::OpReturn as u8);
    }

    #[test]
    fn test_read_u16() {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();
        // Write 0x1234 as two bytes (big-endian)
        chunk.code.push(0x12);
        chunk.code.push(0x34);

        let function = Rc::new(FunctionObject::new(
            "test".to_string(),
            0,
            chunk,
            Vec::new(),
            "test".to_string(),
        ));

        vm.frames.push(crate::vm::call_frame::CallFrame::new(
            function, 0, None, None,
        ));

        assert_eq!(vm.read_u16(), 0x1234);
    }
}
