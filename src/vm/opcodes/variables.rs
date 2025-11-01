//! Variable access operation handlers for the VM.
//!
//! This module provides handlers for variable-related opcodes:
//! - Global variables: `OpDefineGlobal`, `OpGetGlobal`, `OpSetGlobal`
//! - Local variables: `OpGetLocal`, `OpSetLocal`
//! - Upvalues (closures): `OpGetUpvalue`, `OpSetUpvalue`

use crate::object::Object;
use crate::vm::call_frame::CallFrame;
use crate::vm::stack_ops::Stack;
use crate::vm::InterpretResult;
use std::collections::HashMap;

/// Handler for OpDefineGlobal - defines a new global variable.
///
/// Reads a constant name from the bytecode and creates a new global variable
/// with the value on top of the stack.
pub fn op_define_global(name: String, value: Object, globals: &mut HashMap<String, Object>) {
    globals.insert(name, value);
}

/// Handler for OpGetGlobal - gets the value of a global variable.
///
/// Reads a constant name from the bytecode and pushes the global variable's value
/// onto the stack. Returns RuntimeError if the variable doesn't exist.
pub fn op_get_global(
    name: &str,
    globals: &HashMap<String, Object>,
) -> Result<Object, InterpretResult> {
    globals
        .get(name)
        .cloned()
        .ok_or(InterpretResult::RuntimeError)
}

/// Handler for OpSetGlobal - sets the value of an existing global variable.
///
/// Updates an existing global variable with the value on top of the stack.
/// Returns RuntimeError if the variable doesn't exist (assignment to undefined variable).
pub fn op_set_global(
    name: &str,
    value: Object,
    globals: &mut HashMap<String, Object>,
) -> Result<(), InterpretResult> {
    if globals.contains_key(name) {
        globals.insert(name.to_string(), value);
        Ok(())
    } else {
        Err(InterpretResult::RuntimeError)
    }
}

/// Handler for OpGetLocal - gets the value of a local variable.
///
/// Reads a slot index from the bytecode and pushes the local variable's value
/// from the current frame's stack window.
pub fn op_get_local(
    slot: usize,
    stack: &Stack,
    frames: &[CallFrame],
) -> Result<Object, InterpretResult> {
    if let Some(frame) = frames.last() {
        let index = frame.slot + slot;
        Ok(stack.get(index).clone())
    } else {
        Err(InterpretResult::RuntimeError)
    }
}

/// Handler for OpSetLocal - sets the value of a local variable.
///
/// Reads a slot index from the bytecode and updates the local variable
/// in the current frame's stack window.
pub fn op_set_local(
    slot: usize,
    value: Object,
    stack: &mut Stack,
    frames: &[CallFrame],
) -> Result<(), InterpretResult> {
    if let Some(frame) = frames.last() {
        let index = frame.slot + slot;
        stack.set(index, value);
        Ok(())
    } else {
        Err(InterpretResult::RuntimeError)
    }
}

/// Handler for OpGetUpvalue - gets the value of an upvalue (captured variable).
///
/// Reads an upvalue slot from the bytecode and retrieves the value from
/// either the stack (if still open) or the closed-over value.
pub fn op_get_upvalue(
    slot: usize,
    stack: &Stack,
    frames: &[CallFrame],
) -> Result<Object, InterpretResult> {
    let upvalue_ref = frames
        .last()
        .and_then(|frame| frame.function.upvalues.get(slot).cloned())
        .ok_or(InterpretResult::RuntimeError)?;

    let value = {
        let upvalue = upvalue_ref.borrow();
        if upvalue.is_closed {
            upvalue.closed.clone()
        } else {
            stack.get(upvalue.location).clone()
        }
    };
    Ok(value)
}

/// Handler for OpSetUpvalue - sets the value of an upvalue (captured variable).
///
/// Reads an upvalue slot from the bytecode and updates the value either
/// on the stack (if still open) or in the closed-over storage.
pub fn op_set_upvalue(
    slot: usize,
    value: Object,
    stack: &mut Stack,
    frames: &[CallFrame],
) -> Result<(), InterpretResult> {
    let upvalue_ref = frames
        .last()
        .and_then(|frame| frame.function.upvalues.get(slot).cloned())
        .ok_or(InterpretResult::RuntimeError)?;

    let mut upvalue = upvalue_ref.borrow_mut();
    if upvalue.is_closed {
        upvalue.closed = value;
    } else {
        stack.set(upvalue.location, value);
    }
    Ok(())
}
