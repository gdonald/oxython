//! Opcode dispatching for the VM.
//!
//! This module implements the main opcode dispatch logic that executes
//! individual bytecode instructions.

use super::VM;
use crate::bytecode::OpCode;
use crate::object::ObjectType;
use crate::vm::{opcodes, InterpretResult};
use std::rc::Rc;

impl VM {
    /// Dispatch and execute a single opcode instruction.
    pub(super) fn dispatch_opcode(&mut self, instruction: OpCode) -> InterpretResult {
        match instruction {
            OpCode::OpConstant => {
                let const_idx = self.read_byte() as usize;
                let constant = self.current_chunk().constants[const_idx].clone();
                self.push(constant);
            }
            OpCode::OpAdd => {
                let b = self.pop();
                let a = self.pop();
                match opcodes::arithmetic::op_add(a, b) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpDivide => {
                let b = self.pop();
                let a = self.pop();
                match opcodes::arithmetic::op_divide(a, b) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpSubtract => {
                let b = self.pop();
                let a = self.pop();
                match opcodes::arithmetic::op_subtract(a, b) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpMultiply => {
                let b = self.pop();
                let a = self.pop();
                match opcodes::arithmetic::op_multiply(a, b) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpModulo => {
                let b = self.pop();
                let a = self.pop();
                match opcodes::arithmetic::op_modulo(a, b) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpDefineGlobal => {
                let name_idx = self.read_byte() as usize;
                if let ObjectType::String(name) = &*self.current_chunk().constants[name_idx] {
                    let value = self.peek(0).clone();
                    opcodes::variables::op_define_global(name.clone(), value, &mut self.globals);
                    self.pop();
                }
            }
            OpCode::OpGetGlobal => {
                let name_idx = self.read_byte() as usize;
                if let ObjectType::String(name) = &*self.current_chunk().constants[name_idx] {
                    match opcodes::variables::op_get_global(name, &self.globals) {
                        Ok(value) => self.push(value),
                        Err(e) => return e,
                    }
                }
            }
            OpCode::OpSetGlobal => {
                let name_idx = self.read_byte() as usize;
                let name = match &*self.current_chunk().constants[name_idx] {
                    ObjectType::String(name) => name.clone(),
                    _ => return InterpretResult::RuntimeError,
                };
                let value = self.peek(0).clone();
                match opcodes::variables::op_set_global(&name, value, &mut self.globals) {
                    Ok(()) => {}
                    Err(e) => return e,
                }
            }
            OpCode::OpCall => {
                let arg_count = self.read_byte() as usize;
                if !self.call_value(arg_count) {
                    return InterpretResult::RuntimeError;
                }
            }
            OpCode::OpMakeFunction => {
                let proto_idx = self.read_byte() as usize;
                let proto = match &*self.current_chunk().constants[proto_idx] {
                    ObjectType::FunctionPrototype(proto) => proto.clone(),
                    _ => return InterpretResult::RuntimeError,
                };
                let (frame_slot, parent_upvalues) = if let Some(frame) = self.frames.last() {
                    (frame.slot, frame.function.upvalues.clone())
                } else {
                    (0, Vec::new())
                };
                match opcodes::functions::op_make_function(
                    proto,
                    frame_slot,
                    &parent_upvalues,
                    self.globals.clone(),
                    &mut self.open_upvalues,
                ) {
                    Ok(function) => self.push(function),
                    Err(e) => return e,
                }
            }
            OpCode::OpGetLocal => {
                let slot = self.read_byte() as usize;
                match opcodes::variables::op_get_local(slot, &self.stack, &self.frames) {
                    Ok(value) => self.push(value),
                    Err(e) => return e,
                }
            }
            OpCode::OpSetLocal => {
                let slot = self.read_byte() as usize;
                let value = self.peek(0).clone();
                match opcodes::variables::op_set_local(slot, value, &mut self.stack, &self.frames) {
                    Ok(()) => {}
                    Err(e) => return e,
                }
            }
            OpCode::OpGetUpvalue => {
                let slot = self.read_byte() as usize;
                match opcodes::variables::op_get_upvalue(slot, &self.stack, &self.frames) {
                    Ok(value) => self.push(value),
                    Err(e) => return e,
                }
            }
            OpCode::OpSetUpvalue => {
                let slot = self.read_byte() as usize;
                let value = self.peek(0).clone();
                match opcodes::variables::op_set_upvalue(slot, value, &mut self.stack, &self.frames)
                {
                    Ok(()) => {}
                    Err(e) => return e,
                }
            }
            OpCode::OpPrintSpaced => {
                let value = self.pop();
                opcodes::io::op_print_spaced(value, &mut self.stack, &mut self.frames);
            }
            OpCode::OpPrint => {
                let value = self.pop();
                opcodes::io::op_print(value, &mut self.stack, &mut self.frames);
            }
            OpCode::OpPrintln => {
                opcodes::io::op_println();
            }
            OpCode::OpIndex => {
                let index = self.pop();
                let collection = self.pop();
                match opcodes::collections::op_index(collection, index) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpLen => {
                let value = self.pop();
                match opcodes::collections::op_len(value) {
                    Ok(len) => self.push(Rc::new(ObjectType::Integer(len))),
                    Err(e) => return e,
                }
            }
            OpCode::OpToList => {
                let value = self.pop();
                match opcodes::builtins::op_to_list(value) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpAppend => {
                let value = self.pop();
                let collection = self.pop();
                match opcodes::collections::op_append(collection, value) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpRange => {
                let end = self.pop();
                let start = self.pop();
                match opcodes::collections::op_range(start, end) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpLess => {
                let b = self.pop();
                let a = self.pop();
                match opcodes::comparison::op_less(a, b) {
                    Ok(result) => self.push(Rc::new(ObjectType::Boolean(result))),
                    Err(e) => return e,
                }
            }
            OpCode::OpEqual => {
                let b = self.pop();
                let a = self.pop();
                let result = opcodes::comparison::op_equal(a, b);
                self.push(Rc::new(ObjectType::Boolean(result)));
            }
            OpCode::OpSlice => {
                let step = self.pop();
                let end = self.pop();
                let start = self.pop();
                let collection = self.pop();
                match opcodes::collections::op_slice(collection, start, end, step) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpStrLower => {
                let value = self.pop();
                match opcodes::strings::op_str_lower(value) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpStrIsAlnum => {
                let value = self.pop();
                match opcodes::strings::op_str_is_alnum(value) {
                    Ok(result) => self.push(Rc::new(ObjectType::Boolean(result))),
                    Err(e) => return e,
                }
            }
            OpCode::OpStrJoin => {
                let iterable = self.pop();
                let separator = self.pop();
                match opcodes::strings::op_str_join(separator, iterable) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpRound => {
                let digits = self.pop();
                let value = self.pop();
                match opcodes::builtins::op_round(value, digits) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpZip => {
                let arg_count = self.read_byte() as usize;
                let star_mask = self.read_u16() as u16;

                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.pop());
                }
                args.reverse();

                match opcodes::builtins::op_zip(args, star_mask) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpReturn => {
                if self.handle_return() {
                    return InterpretResult::Ok;
                }
            }
            OpCode::OpPop => {
                self.pop();
            }
            OpCode::OpIterNext => {
                let offset = self.read_u16();
                let index = self.pop();
                let collection = self.pop();

                match opcodes::control_flow::op_iter_next(
                    offset,
                    index,
                    collection,
                    &mut self.frames,
                ) {
                    Ok(Some((coll, idx, elem))) => {
                        self.push(coll);
                        self.push(idx);
                        self.push(elem);
                    }
                    Ok(None) => {
                        // Iteration finished, frame IP already updated
                    }
                    Err(e) => return e,
                }
            }
            OpCode::OpLoop => {
                let offset = self.read_u16();
                opcodes::control_flow::op_loop(offset, &mut self.frames);
            }
            OpCode::OpJumpIfFalse => {
                let offset = self.read_u16();
                let condition = self.peek(0).clone();
                opcodes::control_flow::op_jump_if_false(offset, &condition, &mut self.frames);
            }
            OpCode::OpJump => {
                let offset = self.read_u16();
                opcodes::control_flow::op_jump(offset, &mut self.frames);
            }
            OpCode::OpSetIndex => {
                let value = self.pop();
                let index = self.pop();
                let collection = self.pop();
                match opcodes::collections::op_set_index(collection, index, value) {
                    Ok(result) => self.push(result),
                    Err(e) => return e,
                }
            }
            OpCode::OpDup => {
                let value = self.peek(0).clone();
                self.push(value);
            }
            OpCode::OpContains => {
                let collection = self.pop();
                let item = self.pop();
                match opcodes::collections::op_contains(item, collection) {
                    Ok(result) => self.push(Rc::new(ObjectType::Boolean(result))),
                    Err(e) => return e,
                }
            }
            OpCode::OpSwap => {
                let base = self.frames.last().map(|frame| frame.slot + 1).unwrap_or(0);
                if self.stack.top() < base + 2 {
                    return InterpretResult::RuntimeError;
                }
                self.stack.swap(self.stack.top() - 1, self.stack.top() - 2);
            }
            OpCode::OpMakeClass => {
                let method_count = self.read_byte() as usize;

                // Pop class name
                let class_name = match &*self.pop() {
                    ObjectType::String(name) => name.clone(),
                    _ => return InterpretResult::RuntimeError,
                };

                // Pop method names (in reverse order)
                let mut method_names = Vec::with_capacity(method_count);
                for _ in 0..method_count {
                    let method_name = match &*self.pop() {
                        ObjectType::String(name) => name.clone(),
                        _ => return InterpretResult::RuntimeError,
                    };
                    method_names.push(method_name);
                }

                // Method names are in reverse order, so reverse them back
                method_names.reverse();

                // Now pop method functions (they're below the names on stack)
                let mut method_funcs = Vec::with_capacity(method_count);
                for _ in 0..method_count {
                    method_funcs.push(self.pop());
                }

                // Functions are in reverse order too
                method_funcs.reverse();

                match opcodes::classes::op_make_class(class_name, method_names, method_funcs) {
                    Ok(class) => self.push(class),
                    Err(e) => return e,
                }
            }
            OpCode::OpGetAttr => {
                let attr_idx = self.read_byte() as usize;
                let attr_name = match &*self.current_chunk().constants[attr_idx] {
                    ObjectType::String(name) => name.clone(),
                    _ => return InterpretResult::RuntimeError,
                };

                let object = self.pop();
                match opcodes::attributes::op_get_attr(object, &attr_name, &self.stack) {
                    Ok(value) => self.push(value),
                    Err(e) => return e,
                }
            }
            OpCode::OpSetAttr => {
                let attr_idx = self.read_byte() as usize;
                let attr_name = match &*self.current_chunk().constants[attr_idx] {
                    ObjectType::String(name) => name.clone(),
                    _ => return InterpretResult::RuntimeError,
                };

                let value = self.pop();
                let object = self.pop();

                match opcodes::attributes::op_set_attr(object, attr_name, value) {
                    Ok(()) => {}
                    Err(e) => return e,
                }
            }
            OpCode::OpInherit => {
                // Stack: [child_class, parent_class]
                let parent = self.pop();
                let child = self.pop();

                match opcodes::classes::op_inherit(child, parent) {
                    Ok(class) => self.push(class),
                    Err(e) => return e,
                }
            }
            OpCode::OpType => {
                let value = self.pop();
                let type_name = opcodes::builtins::op_type(value);
                self.push(type_name);
            }
        }

        InterpretResult::Ok
    }
}
