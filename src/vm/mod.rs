mod bytecode_reader;
mod call_frame;
pub mod collections;
mod function_calls;
pub mod native;
mod opcode_dispatcher;
pub mod opcodes;
mod stack_ops;
mod string_repr;
mod upvalues;
pub mod values;

use crate::bytecode::{Chunk, OpCode};
use crate::object::{FunctionObject, Object, ObjectType, UpvalueRef};
use call_frame::CallFrame;
use stack_ops::Stack;
use std::collections::HashMap;
use std::rc::Rc;

pub struct VM {
    stack: Stack,
    globals: HashMap<String, Object>,
    frames: Vec<CallFrame>,
    open_upvalues: Vec<UpvalueRef>,
}

#[derive(Debug, PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            stack: Stack::new(),
            globals: HashMap::new(),
            frames: Vec::new(),
            open_upvalues: Vec::new(),
        };
        vm.register_builtins();
        vm
    }

    fn register_builtins(&mut self) {
        native::register_builtins(&mut self.globals);
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.stack.reset();
        self.frames.clear();
        self.open_upvalues.clear();

        let script_function = Rc::new(FunctionObject::new(
            "<script>".to_string(),
            0,
            chunk,
            Vec::new(),
            "<script>".to_string(),
        ));
        self.push(Rc::new(ObjectType::Function(script_function.clone())));
        self.frames
            .push(CallFrame::new(script_function, 0, None, None));

        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if self.frames.is_empty() {
                return InterpretResult::Ok;
            }

            let instruction = OpCode::from(self.read_byte());
            let result = self.dispatch_opcode(instruction);

            if result != InterpretResult::Ok {
                return result;
            }
        }
    }

    fn handle_return(&mut self) -> bool {
        let (frame_slot, frame_arity, instance_slot) = if let Some(frame) = self.frames.last() {
            (frame.slot, frame.function.arity, frame.instance_slot)
        } else {
            (0, 0, None)
        };
        // Stack layout: [callee/function, params...] [return_value?]
        // frame_slot points to callee, params start at frame_slot+1
        // Return value (if any) is at frame_slot + arity + 1
        // So if there's a return value, stack_top > frame_slot + arity + 1
        let minimum_stack = frame_slot + frame_arity + 1;
        let result = if self.stack.top() > minimum_stack {
            Some(self.pop())
        } else {
            None
        };

        // Save the instance BEFORE resetting stack_top
        let saved_instance = instance_slot.map(|slot| self.stack.get(slot).clone());

        upvalues::close_upvalues(&mut self.open_upvalues, &self.stack, frame_slot);

        self.frames.pop();
        self.stack.set_top(frame_slot);

        if self.frames.is_empty() {
            if let Some(value) = result {
                self.stack.set_last_popped(value.clone());
                self.push(value);
            }
            true
        } else {
            // Check if this was an __init__ call
            let value = if let Some(instance) = saved_instance {
                // Return the saved instance instead of the function's return value
                instance
            } else {
                result.unwrap_or_else(|| Rc::new(ObjectType::Nil))
            };
            self.stack.set_last_popped(value.clone());
            self.push(value);
            false
        }
    }
}
