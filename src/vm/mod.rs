mod bytecode_reader;
mod call_frame;
pub mod collections;
mod function_calls;
pub mod native;
mod opcode_dispatcher;
pub mod opcodes;
mod return_handler;
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
}
