//! Control flow operation handlers for the VM.
//!
//! This module provides handlers for control flow opcodes:
//! - Unconditional jumps: `OpJump`
//! - Conditional jumps: `OpJumpIfFalse`
//! - Loops: `OpLoop`
//! - Iteration: `OpIterNext`

use crate::object::{Object, ObjectType};
use crate::vm::call_frame::CallFrame;
use crate::vm::InterpretResult;
use std::rc::Rc;

/// Handler for OpJump - unconditional forward jump.
///
/// Advances the instruction pointer by the given offset.
pub fn op_jump(offset: usize, frames: &mut [CallFrame]) {
    if let Some(frame) = frames.last_mut() {
        frame.ip += offset;
    }
}

/// Handler for OpJumpIfFalse - conditional forward jump.
///
/// If the value on top of the stack is falsey, advances the instruction pointer
/// by the given offset. Does not pop the condition value.
pub fn op_jump_if_false(offset: usize, condition: &Object, frames: &mut [CallFrame]) {
    if !crate::vm::values::is_truthy(condition) {
        if let Some(frame) = frames.last_mut() {
            frame.ip += offset;
        }
    }
}

/// Handler for OpLoop - backward jump for loops.
///
/// Moves the instruction pointer backward by the given offset.
pub fn op_loop(offset: usize, frames: &mut [CallFrame]) {
    if let Some(frame) = frames.last_mut() {
        frame.ip -= offset;
    }
}

/// Handler for OpIterNext - iteration step for for-loops.
///
/// Manages iteration over collections (lists, tuples, strings).
/// Stack layout: [collection, index]
/// Returns: [collection, next_index, element] or jumps past loop body if done
pub fn op_iter_next(
    offset: usize,
    index: Object,
    collection: Object,
    frames: &mut [CallFrame],
) -> Result<Option<(Object, Object, Object)>, InterpretResult> {
    match (&*collection, &*index) {
        (ObjectType::List(values), ObjectType::Integer(idx))
        | (ObjectType::Tuple(values), ObjectType::Integer(idx)) => {
            if *idx < 0 {
                return Err(InterpretResult::RuntimeError);
            }
            let idx_usize = *idx as usize;
            if idx_usize >= values.len() {
                // Iteration finished; skip body
                if let Some(frame) = frames.last_mut() {
                    frame.ip += offset;
                }
                Ok(None)
            } else {
                let element = values[idx_usize].clone();
                let next_index = (idx_usize + 1) as i64;
                Ok(Some((
                    collection,
                    Rc::new(ObjectType::Integer(next_index)),
                    element,
                )))
            }
        }
        (ObjectType::String(text), ObjectType::Integer(idx)) => {
            if *idx < 0 {
                return Err(InterpretResult::RuntimeError);
            }
            let chars: Vec<char> = text.chars().collect();
            let idx_usize = *idx as usize;
            if idx_usize >= chars.len() {
                if let Some(frame) = frames.last_mut() {
                    frame.ip += offset;
                }
                Ok(None)
            } else {
                let ch = chars[idx_usize];
                let next_index = (idx_usize + 1) as i64;
                Ok(Some((
                    collection,
                    Rc::new(ObjectType::Integer(next_index)),
                    Rc::new(ObjectType::String(ch.to_string())),
                )))
            }
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}
