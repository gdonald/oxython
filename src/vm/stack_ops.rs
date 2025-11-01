use crate::object::Object;
use crate::object::ObjectType;
use std::rc::Rc;

pub const STACK_MAX: usize = 256;

/// Stack structure for the VM
/// Manages the operand stack used during bytecode execution
pub struct Stack {
    data: [Object; STACK_MAX],
    top: usize,
    last_popped: Object,
}

impl Stack {
    /// Create a new stack initialized with Nil values
    pub fn new() -> Self {
        let default_obj = Rc::new(ObjectType::Nil);
        Stack {
            data: [(); STACK_MAX].map(|_| default_obj.clone()),
            top: 0,
            last_popped: default_obj,
        }
    }

    /// Push a value onto the stack
    #[inline]
    pub fn push(&mut self, value: Object) {
        self.data[self.top] = value;
        self.top += 1;
    }

    /// Pop a value from the stack and return it
    #[inline]
    pub fn pop(&mut self) -> Object {
        self.top -= 1;
        self.last_popped = self.data[self.top].clone();
        self.last_popped.clone()
    }

    /// Peek at a value on the stack without removing it
    /// Distance 0 = top of stack, 1 = second from top, etc.
    #[inline]
    pub fn peek(&self, distance: usize) -> &Object {
        &self.data[self.top - 1 - distance]
    }

    /// Get the last popped value (used by the VM for tracking expression results)
    #[inline]
    pub fn last_popped(&self) -> Object {
        self.last_popped.clone()
    }

    /// Peek at the top of the stack (returns None if stack is empty)
    /// Helper for testing to inspect the top of the stack without popping
    pub fn peek_top(&self) -> Option<Object> {
        (self.top > 0).then(|| self.data[self.top - 1].clone())
    }

    /// Get the current stack top index
    #[inline]
    pub fn top(&self) -> usize {
        self.top
    }

    /// Set the stack top index (used for frame management)
    #[inline]
    pub fn set_top(&mut self, top: usize) {
        self.top = top;
    }

    /// Get a reference to a specific stack slot
    #[inline]
    pub fn get(&self, index: usize) -> &Object {
        &self.data[index]
    }

    /// Set a specific stack slot to a value
    #[inline]
    pub fn set(&mut self, index: usize, value: Object) {
        self.data[index] = value;
    }

    /// Swap two values on the stack
    pub fn swap(&mut self, a: usize, b: usize) {
        self.data.swap(a, b);
    }

    /// Reset the stack to initial state
    pub fn reset(&mut self) {
        self.top = 0;
        self.last_popped = Rc::new(ObjectType::Nil);
    }

    /// Set the last_popped value (used by handle_return)
    #[inline]
    pub fn set_last_popped(&mut self, value: Object) {
        self.last_popped = value;
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

// VM stack operation methods
use super::VM;

impl VM {
    /// Push a value onto the VM's operand stack.
    #[inline]
    pub(super) fn push(&mut self, value: Object) {
        self.stack.push(value);
    }

    /// Pop a value from the VM's operand stack.
    #[inline]
    pub(super) fn pop(&mut self) -> Object {
        self.stack.pop()
    }

    /// Peek at a value on the stack without removing it.
    /// Distance 0 = top of stack, 1 = second from top, etc.
    #[inline]
    pub(super) fn peek(&self, distance: usize) -> &Object {
        self.stack.peek(distance)
    }

    /// Get the last popped value from the stack.
    /// This is used by the REPL to display expression results.
    pub fn last_popped_stack_elem(&self) -> Object {
        self.stack.last_popped()
    }

    /// Helper for testing to inspect the top of the stack without popping.
    pub fn peek_stack(&self) -> Option<Object> {
        self.stack.peek_top()
    }
}
