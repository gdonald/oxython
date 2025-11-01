use crate::bytecode::OpCode;
use crate::object::{Object, ObjectType};
use std::rc::Rc;

use super::call_frame::CallFrame;
use super::stack_ops::Stack;

/// Get string representation of an object, checking for __str__ and __repr__ special methods.
///
/// This function implements Python's string representation protocol:
/// 1. For instances, check for `__str__` method first
/// 2. If not found, check for `__repr__` method
/// 3. If neither exists, use default representation
/// 4. For non-instance objects, use the Display trait
///
/// # Arguments
/// * `value` - The object to get string representation for
/// * `stack` - Mutable reference to the VM stack
/// * `frames` - Mutable reference to the call frames
///
/// # Returns
/// Optional string representation, or None if the method call failed
pub fn get_string_representation(
    value: Object,
    stack: &mut Stack,
    frames: &mut Vec<CallFrame>,
) -> Option<String> {
    match &*value {
        ObjectType::Instance(instance_ref) => {
            let instance = instance_ref.borrow();

            // Check for __str__ method first
            if let Some(str_method) = instance.class.get_method("__str__") {
                drop(instance); // Release borrow before calling method

                // Call __str__ on the instance
                let result = call_str_method(value.clone(), str_method, stack, frames)?;

                // Extract string from result
                if let ObjectType::String(s) = &*result {
                    return Some(s.clone());
                }
                return None;
            }

            // Check for __repr__ method as fallback
            if let Some(repr_method) = instance.class.get_method("__repr__") {
                drop(instance); // Release borrow before calling method

                // Call __repr__ on the instance
                let result = call_str_method(value.clone(), repr_method, stack, frames)?;

                // Extract string from result
                if let ObjectType::String(s) = &*result {
                    return Some(s.clone());
                }
                return None;
            }

            // No __str__ or __repr__, use default representation
            let class_name = instance.class.name.clone();
            drop(instance);
            Some(format!("<{} instance>", class_name))
        }
        _ => {
            // For non-instance objects, use the Display trait
            Some(format!("{}", value))
        }
    }
}

/// Helper to call __str__ or __repr__ method on an instance.
///
/// This function executes a mini-event loop to run the method and capture its result.
/// It handles the complexity of calling a method from within the VM without disrupting
/// the current execution state.
///
/// # Arguments
/// * `instance` - The instance object
/// * `method` - The method to call (__str__ or __repr__)
/// * `stack` - Mutable reference to the VM stack
/// * `frames` - Mutable reference to the call frames
///
/// # Returns
/// The result object from the method call, or None if the call failed
fn call_str_method(
    instance: Object,
    method: Object,
    stack: &mut Stack,
    frames: &mut Vec<CallFrame>,
) -> Option<Object> {
    // Save current stack state
    let saved_stack_top = stack.top();
    let saved_frame_count = frames.len();

    // Create a bound method
    let bound_method = Rc::new(ObjectType::BoundMethod(instance, method));

    // Push bound method onto stack
    stack.push(bound_method);

    // Call with 0 arguments (just self)
    if !call_value_for_str(0, stack, frames) {
        stack.set_top(saved_stack_top);
        return None;
    }

    // Execute until the method returns by running a mini event loop
    loop {
        // Check if we've returned from the __str__ call
        // We pushed a new frame, so frame_count increased. When we return, it should go back down.
        if frames.len() <= saved_frame_count {
            // Method has returned, get result
            if stack.top() > saved_stack_top {
                let result = stack.pop();
                stack.set_top(saved_stack_top);
                return Some(result);
            } else {
                stack.set_top(saved_stack_top);
                return None;
            }
        }

        if frames.is_empty() {
            stack.set_top(saved_stack_top);
            return None;
        }

        // Read and execute one instruction
        let instruction = OpCode::from(read_byte(frames));

        // We need to handle all possible opcodes that __str__ might use
        // For simplicity, let's handle the most common ones
        let should_bail = match instruction {
            OpCode::OpConstant => {
                let const_idx = read_byte(frames) as usize;
                let constant = current_chunk(frames).constants[const_idx].clone();
                stack.push(constant);
                false
            }
            OpCode::OpGetLocal => {
                let slot = read_byte(frames) as usize;
                if let Some(frame) = frames.last() {
                    let index = frame.slot + slot;
                    let value = stack.get(index).clone();
                    stack.push(value);
                }
                false
            }
            OpCode::OpGetAttr => {
                let attr_idx = read_byte(frames) as usize;
                let attr_name =
                    if let ObjectType::String(name) = &*current_chunk(frames).constants[attr_idx] {
                        name.clone()
                    } else {
                        stack.set_top(saved_stack_top);
                        return None;
                    };
                let object = stack.pop();
                if let ObjectType::Instance(instance_ref) = &*object {
                    let instance = instance_ref.borrow();
                    if let Some(value) = instance.get_field(&attr_name) {
                        stack.push(value);
                    } else {
                        stack.set_top(saved_stack_top);
                        return None;
                    }
                } else {
                    stack.set_top(saved_stack_top);
                    return None;
                }
                false
            }
            OpCode::OpAdd => {
                let b = stack.pop();
                let a = stack.pop();
                match (&*a, &*b) {
                    (ObjectType::String(val_a), ObjectType::String(val_b)) => {
                        let mut combined = val_a.clone();
                        combined.push_str(val_b);
                        stack.push(Rc::new(ObjectType::String(combined)));
                        false
                    }
                    _ => true,
                }
            }
            OpCode::OpReturn => {
                if handle_return_for_str(stack, frames) {
                    stack.set_top(saved_stack_top);
                    return None;
                }
                false
            }
            _ => {
                // Unsupported opcode in __str__ method
                true
            }
        };

        if should_bail {
            stack.set_top(saved_stack_top);
            return None;
        }
    }
}

/// Helper function to get the current chunk from the active frame
fn current_chunk(frames: &[CallFrame]) -> &crate::bytecode::Chunk {
    &frames
        .last()
        .expect("expected active call frame")
        .function
        .chunk
}

/// Helper function to read a byte from the current frame
fn read_byte(frames: &mut [CallFrame]) -> u8 {
    let frame = frames.last_mut().expect("expected active call frame");
    let byte = frame.function.chunk.code[frame.ip];
    frame.ip += 1;
    byte
}

/// Simplified call_value for __str__ method execution
fn call_value_for_str(arg_count: usize, stack: &mut Stack, frames: &mut Vec<CallFrame>) -> bool {
    if stack.top() < arg_count + 1 {
        return false;
    }
    let callee_index = stack.top() - arg_count - 1;
    let callee = stack.get(callee_index).clone();
    match &*callee {
        ObjectType::BoundMethod(instance, method) => {
            // Insert the instance as first parameter
            // Stack layout: [bound_method, arg1, arg2, ...]
            // Need: [bound_method, instance(self), arg1, arg2, ...]

            // Shift arguments to make room for self
            let stack_top = stack.top();
            for i in (callee_index + 1..stack_top).rev() {
                let value = stack.get(i).clone();
                stack.set(i + 1, value);
            }
            // Insert instance as first parameter
            stack.set(callee_index + 1, instance.clone());
            stack.set_top(stack_top + 1);

            match &**method {
                ObjectType::Function(function) => {
                    // Call with arg_count + 1 (including self)
                    call_function_for_str(function.clone(), callee_index, arg_count + 1, frames)
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Simplified call_function for __str__ method execution
fn call_function_for_str(
    function: Rc<crate::object::FunctionObject>,
    callee_index: usize,
    arg_count: usize,
    frames: &mut Vec<CallFrame>,
) -> bool {
    if function.arity != arg_count {
        return false;
    }
    if frames.len() >= super::call_frame::FRAMES_MAX {
        return false;
    }

    frames.push(CallFrame::new(function, callee_index, None, None));
    true
}

/// Simplified handle_return for __str__ method execution
fn handle_return_for_str(stack: &mut Stack, frames: &mut Vec<CallFrame>) -> bool {
    let (frame_slot, frame_arity) = if let Some(frame) = frames.last() {
        (frame.slot, frame.function.arity)
    } else {
        (0, 0)
    };

    let minimum_stack = frame_slot + frame_arity + 1;
    let result = if stack.top() > minimum_stack {
        Some(stack.pop())
    } else {
        None
    };

    frames.pop();
    stack.set_top(frame_slot);

    if frames.is_empty() {
        if let Some(value) = result {
            stack.set_last_popped(value.clone());
            stack.push(value);
        }
        true
    } else {
        let value = result.unwrap_or_else(|| Rc::new(ObjectType::Nil));
        stack.set_last_popped(value.clone());
        stack.push(value);
        false
    }
}
