//! I/O operation handlers for the VM.
//!
//! This module provides handlers for I/O-related opcodes:
//! - `OpPrint`: Print a value without trailing space
//! - `OpPrintSpaced`: Print a value with trailing space
//! - `OpPrintln`: Print a newline

use crate::object::Object;
use crate::vm::call_frame::CallFrame;
use crate::vm::stack_ops::Stack;
use crate::vm::string_repr;

/// Handler for OpPrintSpaced - prints a value with a trailing space.
///
/// Pops a value from the stack and prints it with a space after it.
/// Uses the __str__ method if available on instances.
pub fn op_print_spaced(value: Object, stack: &mut Stack, frames: &mut Vec<CallFrame>) {
    let string_repr = string_repr::get_string_representation(value.clone(), stack, frames);
    if let Some(repr) = string_repr {
        print!("{} ", repr);
    } else {
        // Fallback to default Display implementation
        print!("{} ", value);
    }
}

/// Handler for OpPrint - prints a value without trailing space.
///
/// Pops a value from the stack and prints it.
/// Uses the __str__ method if available on instances.
pub fn op_print(value: Object, stack: &mut Stack, frames: &mut Vec<CallFrame>) {
    let string_repr = string_repr::get_string_representation(value.clone(), stack, frames);
    if let Some(repr) = string_repr {
        print!("{}", repr);
    } else {
        // Fallback to default Display implementation
        print!("{}", value);
    }
}

/// Handler for OpPrintln - prints a newline character.
pub fn op_println() {
    println!();
}
