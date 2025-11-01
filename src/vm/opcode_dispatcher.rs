//! Opcode dispatching for the VM.
//!
//! This module implements the main opcode dispatch logic that executes
//! individual bytecode instructions.

use super::VM;
use crate::bytecode::OpCode;
use crate::object::{ClassObject, FunctionObject, Object, ObjectType, UpvalueRef};
use crate::vm::{opcodes, upvalues, InterpretResult};
use std::collections::HashMap;
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
                let mut captured: Vec<UpvalueRef> = Vec::with_capacity(proto.upvalues.len());
                for descriptor in proto.upvalues.iter() {
                    if descriptor.is_local {
                        let stack_index = frame_slot + descriptor.index;
                        let upvalue =
                            upvalues::capture_upvalue(&mut self.open_upvalues, stack_index);
                        captured.push(upvalue);
                    } else {
                        let upvalue = match parent_upvalues.get(descriptor.index) {
                            Some(value) => value.clone(),
                            None => return InterpretResult::RuntimeError,
                        };
                        captured.push(upvalue);
                    }
                }
                let type_info = crate::object::TypeInfo {
                    parameter_names: proto.parameter_names.clone(),
                    parameter_types: proto.parameter_types.clone(),
                    return_type: proto.return_type.clone(),
                    default_values: proto.default_values.clone(),
                };
                let mut function = FunctionObject::new_with_types(
                    proto.name.clone(),
                    proto.arity,
                    proto.chunk.clone(),
                    captured,
                    type_info,
                    proto.module.clone(),
                );
                function.doc = proto.doc.clone();
                function.qualname = proto.qualname.clone();
                // Capture a snapshot of the global namespace at function definition time
                function.globals = self.globals.clone();
                self.push(Rc::new(ObjectType::Function(Rc::new(function))));
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

                // Pair up names and functions
                let mut methods = HashMap::new();
                for (name, func) in method_names.into_iter().zip(method_funcs.into_iter()) {
                    methods.insert(name, func);
                }

                let class = Rc::new(ClassObject::new(class_name, methods));
                self.push(Rc::new(ObjectType::Class(class)));
            }
            OpCode::OpGetAttr => {
                let attr_idx = self.read_byte() as usize;
                let attr_name = match &*self.current_chunk().constants[attr_idx] {
                    ObjectType::String(name) => name.clone(),
                    _ => return InterpretResult::RuntimeError,
                };

                let object = self.pop();
                match &*object {
                    ObjectType::Instance(instance_ref) => {
                        let instance = instance_ref.borrow();

                        // First check instance fields
                        if let Some(value) = instance.get_field(&attr_name) {
                            self.push(value);
                        } else if let Some(method) = instance.class.get_method(&attr_name) {
                            // Create a bound method (using inheritance chain)
                            let bound =
                                Rc::new(ObjectType::BoundMethod(object.clone(), method.clone()));
                            self.push(bound);
                        } else {
                            return InterpretResult::RuntimeError;
                        }
                    }
                    ObjectType::Class(class) => {
                        // Access method from class directly (using inheritance chain)
                        if let Some(method) = class.get_method(&attr_name) {
                            self.push(method.clone());
                        } else {
                            return InterpretResult::RuntimeError;
                        }
                    }
                    ObjectType::SuperProxy(instance, parent_class) => {
                        // Look up method in the parent class only (not the full chain)
                        if let Some(method) = parent_class.get_method(&attr_name) {
                            // Create a bound method with the instance
                            let bound =
                                Rc::new(ObjectType::BoundMethod(instance.clone(), method.clone()));
                            self.push(bound);
                        } else {
                            return InterpretResult::RuntimeError;
                        }
                    }
                    ObjectType::Function(func) => {
                        // Function introspection attributes
                        match attr_name.as_str() {
                            "__name__" => {
                                let name = Rc::new(ObjectType::String(func.name.clone()));
                                self.push(name);
                            }
                            "__module__" => {
                                let module = Rc::new(ObjectType::String(func.module.clone()));
                                self.push(module);
                            }
                            "__doc__" => {
                                let doc = match &func.doc {
                                    Some(docstring) => {
                                        Rc::new(ObjectType::String(docstring.clone()))
                                    }
                                    None => Rc::new(ObjectType::Nil),
                                };
                                self.push(doc);
                            }
                            "__annotations__" => {
                                // Build a Dict with parameter names and type annotations
                                let mut annotations: Vec<(String, Object)> = Vec::new();

                                // Add parameter type annotations
                                for (i, param_name) in func.parameter_names.iter().enumerate() {
                                    if let Some(Some(param_type)) = func.parameter_types.get(i) {
                                        let type_str = Rc::new(ObjectType::String(
                                            param_type.name().to_string(),
                                        ));
                                        annotations.push((param_name.clone(), type_str));
                                    }
                                }

                                // Add return type annotation with 'return' key
                                if let Some(return_type) = &func.return_type {
                                    let type_str =
                                        Rc::new(ObjectType::String(return_type.name().to_string()));
                                    annotations.push(("return".to_string(), type_str));
                                }

                                let annotations_dict = Rc::new(ObjectType::Dict(annotations));
                                self.push(annotations_dict);
                            }
                            "__code__" => {
                                // Return a reference to the function's bytecode chunk
                                let code_obj = Rc::new(ObjectType::CodeObject(func.chunk.clone()));
                                self.push(code_obj);
                            }
                            "__qualname__" => {
                                let qualname = Rc::new(ObjectType::String(func.qualname.clone()));
                                self.push(qualname);
                            }
                            "__globals__" => {
                                // Convert HashMap to Dict format (Vec<(String, Object)>)
                                let globals_vec: Vec<(String, Object)> = func
                                    .globals
                                    .iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect();
                                let globals_dict = Rc::new(ObjectType::Dict(globals_vec));
                                self.push(globals_dict);
                            }
                            "__closure__" => {
                                // Return a tuple of cell objects (upvalues), or None if no closure
                                if func.upvalues.is_empty() {
                                    self.push(Rc::new(ObjectType::Nil));
                                } else {
                                    // Create a tuple containing the closed-over values
                                    let cell_values: Vec<Object> = func
                                        .upvalues
                                        .iter()
                                        .map(|upvalue_ref| {
                                            let upvalue = upvalue_ref.borrow();
                                            if upvalue.is_closed {
                                                // Use the closed value
                                                upvalue.closed.clone()
                                            } else {
                                                // Read from stack at location
                                                self.stack.get(upvalue.location).clone()
                                            }
                                        })
                                        .collect();
                                    let closure_tuple = Rc::new(ObjectType::Tuple(cell_values));
                                    self.push(closure_tuple);
                                }
                            }
                            "__defaults__" => {
                                // Return a tuple of default values for parameters, or None if no defaults
                                // Collect only the default values (not None placeholders)
                                let defaults: Vec<Object> = func
                                    .default_values
                                    .iter()
                                    .filter_map(|opt| opt.clone())
                                    .collect();

                                if defaults.is_empty() {
                                    self.push(Rc::new(ObjectType::Nil));
                                } else {
                                    let defaults_tuple = Rc::new(ObjectType::Tuple(defaults));
                                    self.push(defaults_tuple);
                                }
                            }
                            _ => return InterpretResult::RuntimeError,
                        }
                    }
                    ObjectType::FunctionPrototype(proto) => {
                        // Function prototype introspection attributes
                        match attr_name.as_str() {
                            "__name__" => {
                                let name = Rc::new(ObjectType::String(proto.name.clone()));
                                self.push(name);
                            }
                            "__module__" => {
                                let module = Rc::new(ObjectType::String(proto.module.clone()));
                                self.push(module);
                            }
                            "__doc__" => {
                                let doc = match &proto.doc {
                                    Some(docstring) => {
                                        Rc::new(ObjectType::String(docstring.clone()))
                                    }
                                    None => Rc::new(ObjectType::Nil),
                                };
                                self.push(doc);
                            }
                            "__annotations__" => {
                                // Build a Dict with parameter names and type annotations
                                let mut annotations: Vec<(String, Object)> = Vec::new();

                                // Add parameter type annotations
                                for (i, param_name) in proto.parameter_names.iter().enumerate() {
                                    if let Some(Some(param_type)) = proto.parameter_types.get(i) {
                                        let type_str = Rc::new(ObjectType::String(
                                            param_type.name().to_string(),
                                        ));
                                        annotations.push((param_name.clone(), type_str));
                                    }
                                }

                                // Add return type annotation with 'return' key
                                if let Some(return_type) = &proto.return_type {
                                    let type_str =
                                        Rc::new(ObjectType::String(return_type.name().to_string()));
                                    annotations.push(("return".to_string(), type_str));
                                }

                                let annotations_dict = Rc::new(ObjectType::Dict(annotations));
                                self.push(annotations_dict);
                            }
                            "__code__" => {
                                // Return a reference to the prototype's bytecode chunk
                                let code_obj = Rc::new(ObjectType::CodeObject(proto.chunk.clone()));
                                self.push(code_obj);
                            }
                            "__qualname__" => {
                                let qualname = Rc::new(ObjectType::String(proto.qualname.clone()));
                                self.push(qualname);
                            }
                            "__globals__" => {
                                // Prototypes don't have globals captured yet - return empty dict
                                let empty_dict = Rc::new(ObjectType::Dict(Vec::new()));
                                self.push(empty_dict);
                            }
                            "__closure__" => {
                                // Prototypes are templates, not runtime closures
                                // Return None since closures are only created at runtime
                                self.push(Rc::new(ObjectType::Nil));
                            }
                            "__defaults__" => {
                                // Return a tuple of default values for parameters, or None if no defaults
                                // Collect only the default values (not None placeholders)
                                let defaults: Vec<Object> = proto
                                    .default_values
                                    .iter()
                                    .filter_map(|opt| opt.clone())
                                    .collect();

                                if defaults.is_empty() {
                                    self.push(Rc::new(ObjectType::Nil));
                                } else {
                                    let defaults_tuple = Rc::new(ObjectType::Tuple(defaults));
                                    self.push(defaults_tuple);
                                }
                            }
                            _ => return InterpretResult::RuntimeError,
                        }
                    }
                    _ => return InterpretResult::RuntimeError,
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

                match &*object {
                    ObjectType::Instance(instance_ref) => {
                        instance_ref.borrow_mut().set_field(attr_name, value);
                    }
                    _ => return InterpretResult::RuntimeError,
                }
            }
            OpCode::OpInherit => {
                // Stack: [child_class, parent_class]
                let parent = self.pop();
                let child = self.pop();

                // Ensure parent is a class
                let parent_class = match &*parent {
                    ObjectType::Class(class) => class.clone(),
                    _ => return InterpretResult::RuntimeError,
                };

                // Ensure child is a class
                let child_class = match &*child {
                    ObjectType::Class(class) => class,
                    _ => return InterpretResult::RuntimeError,
                };

                // Create new class with parent set
                let new_child = Rc::new(ClassObject::new_with_parent(
                    child_class.name.clone(),
                    child_class.methods.clone(),
                    parent_class,
                ));

                // Push the updated child class back
                self.push(Rc::new(ObjectType::Class(new_child)));
            }
        }

        InterpretResult::Ok
    }
}
