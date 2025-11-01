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
