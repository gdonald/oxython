use crate::bytecode::{Chunk, OpCode};
use crate::object::{
    ClassObject, FunctionObject, InstanceObject, Object, ObjectType, Upvalue, UpvalueRef,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const STACK_MAX: usize = 256;
const FRAMES_MAX: usize = 64;

struct CallFrame {
    function: Rc<FunctionObject>,
    ip: usize,
    slot: usize,
    instance_slot: Option<usize>, // For __init__ calls, where to find the instance to return
    class_context: Option<Rc<ClassObject>>, // For tracking which class a method belongs to (for super())
}

pub struct VM {
    stack: [Object; STACK_MAX],
    stack_top: usize,
    globals: HashMap<String, Object>,
    last_popped: Object,
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
        // The default for Rc<ObjectType> is needed to initialize the array.
        // We can achieve this by making ObjectType derivable from Default.
        // For now, let's create a default Nil object.
        let default_obj = Rc::new(ObjectType::Nil);
        let mut vm = VM {
            stack: [(); STACK_MAX].map(|_| default_obj.clone()),
            stack_top: 0,
            globals: HashMap::new(),
            last_popped: default_obj,
            frames: Vec::new(),
            open_upvalues: Vec::new(),
        };
        vm.register_builtins();
        vm
    }

    fn register_builtins(&mut self) {
        // Register the super() builtin
        self.globals.insert(
            "super".to_string(),
            Rc::new(ObjectType::NativeFunction(
                "super".to_string(),
                native_super,
            )),
        );
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.stack_top = 0;
        self.frames.clear();
        self.last_popped = Rc::new(ObjectType::Nil);
        self.open_upvalues.clear();

        let script_function = Rc::new(FunctionObject::new(
            "<script>".to_string(),
            0,
            chunk,
            Vec::new(),
        ));
        self.push(Rc::new(ObjectType::Function(script_function.clone())));
        self.frames.push(CallFrame {
            function: script_function,
            ip: 0,
            slot: 0,
            instance_slot: None,
            class_context: None,
        });

        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if self.frames.is_empty() {
                return InterpretResult::Ok;
            }

            let instruction = OpCode::from(self.read_byte());

            match instruction {
                OpCode::OpConstant => {
                    let const_idx = self.read_byte() as usize;
                    let constant = self.current_chunk().constants[const_idx].clone();
                    self.push(constant);
                }
                OpCode::OpAdd => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&*a, &*b) {
                        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => {
                            self.push(Rc::new(ObjectType::Integer(val_a + val_b)));
                        }
                        (ObjectType::Float(val_a), ObjectType::Float(val_b)) => {
                            self.push(Rc::new(ObjectType::Float(val_a + val_b)));
                        }
                        (ObjectType::Integer(val_a), ObjectType::Float(val_b)) => {
                            self.push(Rc::new(ObjectType::Float(*val_a as f64 + val_b)));
                        }
                        (ObjectType::Float(val_a), ObjectType::Integer(val_b)) => {
                            self.push(Rc::new(ObjectType::Float(val_a + *val_b as f64)));
                        }
                        (ObjectType::String(val_a), ObjectType::String(val_b)) => {
                            let mut combined = val_a.clone();
                            combined.push_str(val_b);
                            self.push(Rc::new(ObjectType::String(combined)));
                        }
                        _ => {
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpDivide => {
                    let b = self.pop();
                    let a = self.pop();
                    let lhs = match &*a {
                        ObjectType::Integer(v) => *v as f64,
                        ObjectType::Float(v) => *v,
                        _ => return InterpretResult::RuntimeError,
                    };
                    let rhs = match &*b {
                        ObjectType::Integer(v) => *v as f64,
                        ObjectType::Float(v) => *v,
                        _ => return InterpretResult::RuntimeError,
                    };
                    if rhs == 0.0 {
                        return InterpretResult::RuntimeError;
                    }
                    self.push(Rc::new(ObjectType::Float(lhs / rhs)));
                }
                OpCode::OpSubtract => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&*a, &*b) {
                        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => {
                            self.push(Rc::new(ObjectType::Integer(val_a - val_b)));
                        }
                        (ObjectType::Float(val_a), ObjectType::Float(val_b)) => {
                            self.push(Rc::new(ObjectType::Float(val_a - val_b)));
                        }
                        (ObjectType::Integer(val_a), ObjectType::Float(val_b)) => {
                            self.push(Rc::new(ObjectType::Float(*val_a as f64 - val_b)));
                        }
                        (ObjectType::Float(val_a), ObjectType::Integer(val_b)) => {
                            self.push(Rc::new(ObjectType::Float(val_a - *val_b as f64)));
                        }
                        _ => {
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpMultiply => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&*a, &*b) {
                        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => {
                            self.push(Rc::new(ObjectType::Integer(val_a * val_b)));
                        }
                        (ObjectType::Float(val_a), ObjectType::Float(val_b)) => {
                            self.push(Rc::new(ObjectType::Float(val_a * val_b)));
                        }
                        (ObjectType::Integer(val_a), ObjectType::Float(val_b)) => {
                            self.push(Rc::new(ObjectType::Float(*val_a as f64 * val_b)));
                        }
                        (ObjectType::Float(val_a), ObjectType::Integer(val_b)) => {
                            self.push(Rc::new(ObjectType::Float(val_a * *val_b as f64)));
                        }
                        _ => {
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpModulo => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&*a, &*b) {
                        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => {
                            if *val_b == 0 {
                                return InterpretResult::RuntimeError;
                            }
                            self.push(Rc::new(ObjectType::Integer(val_a % val_b)));
                        }
                        _ => {
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpDefineGlobal => {
                    let name_idx = self.read_byte() as usize;
                    if let ObjectType::String(name) = &*self.current_chunk().constants[name_idx] {
                        self.globals.insert(name.clone(), self.peek(0).clone());
                        self.pop();
                    }
                }
                OpCode::OpGetGlobal => {
                    let name_idx = self.read_byte() as usize;
                    if let ObjectType::String(name) = &*self.current_chunk().constants[name_idx] {
                        if let Some(value) = self.globals.get(name) {
                            self.push(value.clone());
                        } else {
                            // Runtime error: undefined variable
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpSetGlobal => {
                    let name_idx = self.read_byte() as usize;
                    if let ObjectType::String(name) = &*self.current_chunk().constants[name_idx] {
                        if self.globals.contains_key(name) {
                            let value = self.peek(0).clone();
                            self.globals.insert(name.clone(), value);
                        } else {
                            return InterpretResult::RuntimeError;
                        }
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
                            let upvalue = self.capture_upvalue(stack_index);
                            captured.push(upvalue);
                        } else {
                            let upvalue = match parent_upvalues.get(descriptor.index) {
                                Some(value) => value.clone(),
                                None => return InterpretResult::RuntimeError,
                            };
                            captured.push(upvalue);
                        }
                    }
                    let function = Rc::new(FunctionObject::new(
                        proto.name.clone(),
                        proto.arity,
                        proto.chunk.clone(),
                        captured,
                    ));
                    self.push(Rc::new(ObjectType::Function(function)));
                }
                OpCode::OpGetLocal => {
                    let slot = self.read_byte() as usize;
                    if let Some(frame) = self.frames.last() {
                        let index = frame.slot + slot;
                        let value = self.stack[index].clone();
                        self.push(value);
                    } else {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpSetLocal => {
                    let slot = self.read_byte() as usize;
                    if let Some(frame) = self.frames.last() {
                        let index = frame.slot + slot;
                        let value = self.peek(0).clone();
                        self.stack[index] = value;
                    } else {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpGetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let upvalue_ref = if let Some(frame) = self.frames.last() {
                        frame.function.upvalues.get(slot).cloned()
                    } else {
                        None
                    };
                    if let Some(upvalue_ref) = upvalue_ref {
                        let value = {
                            let upvalue = upvalue_ref.borrow();
                            if upvalue.is_closed {
                                upvalue.closed.clone()
                            } else {
                                self.stack[upvalue.location].clone()
                            }
                        };
                        self.push(value);
                    } else {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpSetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let value = self.peek(0).clone();
                    let upvalue_ref = if let Some(frame) = self.frames.last() {
                        frame.function.upvalues.get(slot).cloned()
                    } else {
                        None
                    };
                    if let Some(upvalue_ref) = upvalue_ref {
                        let mut upvalue = upvalue_ref.borrow_mut();
                        if upvalue.is_closed {
                            upvalue.closed = value;
                        } else {
                            self.stack[upvalue.location] = value;
                        }
                    } else {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpPrintSpaced => {
                    print!("{} ", self.pop());
                }
                OpCode::OpPrint => {
                    print!("{}", self.pop());
                }
                OpCode::OpPrintln => {
                    println!();
                }
                OpCode::OpIndex => {
                    let index = self.pop();
                    let collection = self.pop();

                    match (&*collection, &*index) {
                        (ObjectType::List(values), ObjectType::Integer(idx))
                        | (ObjectType::Tuple(values), ObjectType::Integer(idx)) => {
                            let mut idx_isize = *idx as isize;
                            let len = values.len() as isize;
                            if idx_isize < 0 {
                                idx_isize += len;
                            }
                            if idx_isize < 0 || idx_isize as usize >= values.len() {
                                return InterpretResult::RuntimeError;
                            }
                            let element = values[idx_isize as usize].clone();
                            self.push(element);
                        }
                        (ObjectType::Dict(entries), ObjectType::String(key)) => {
                            if let Some((_, value)) =
                                entries.iter().find(|(existing_key, _)| existing_key == key)
                            {
                                self.push(value.clone());
                            } else {
                                return InterpretResult::RuntimeError;
                            }
                        }
                        _ => return InterpretResult::RuntimeError,
                    }
                }
                OpCode::OpLen => {
                    let value = self.pop();
                    match &*value {
                        ObjectType::List(values) => {
                            self.push(Rc::new(ObjectType::Integer(values.len() as i64)));
                        }
                        ObjectType::Tuple(values) => {
                            self.push(Rc::new(ObjectType::Integer(values.len() as i64)));
                        }
                        ObjectType::String(text) => {
                            self.push(Rc::new(ObjectType::Integer(text.chars().count() as i64)));
                        }
                        _ => return InterpretResult::RuntimeError,
                    }
                }
                OpCode::OpToList => {
                    let value = self.pop();
                    let elements = match VM::collect_iterable(&value) {
                        Some(elements) => elements,
                        None => return InterpretResult::RuntimeError,
                    };
                    self.push(Rc::new(ObjectType::List(elements)));
                }
                OpCode::OpAppend => {
                    let value = self.pop();
                    let collection = self.pop();

                    if let ObjectType::List(elements) = &*collection {
                        let mut new_elements = elements.clone();
                        new_elements.push(value.clone());
                        self.push(Rc::new(ObjectType::List(new_elements)));
                    } else {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpRange => {
                    let end = self.pop();
                    let start = self.pop();

                    let (start_val, end_val) = match (&*start, &*end) {
                        (ObjectType::Integer(start_int), ObjectType::Integer(end_int)) => {
                            (*start_int, *end_int)
                        }
                        _ => return InterpretResult::RuntimeError,
                    };

                    let mut elements: Vec<Object> = Vec::new();
                    if start_val < end_val {
                        for value in start_val..end_val {
                            elements.push(Rc::new(ObjectType::Integer(value)));
                        }
                    }

                    self.push(Rc::new(ObjectType::List(elements)));
                }
                OpCode::OpLess => {
                    let b = self.pop();
                    let a = self.pop();

                    let result = match (&*a, &*b) {
                        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => val_a < val_b,
                        (ObjectType::Float(val_a), ObjectType::Float(val_b)) => val_a < val_b,
                        (ObjectType::Integer(val_a), ObjectType::Float(val_b)) => {
                            (*val_a as f64) < *val_b
                        }
                        (ObjectType::Float(val_a), ObjectType::Integer(val_b)) => {
                            *val_a < (*val_b as f64)
                        }
                        _ => return InterpretResult::RuntimeError,
                    };

                    self.push(Rc::new(ObjectType::Boolean(result)));
                }
                OpCode::OpEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = *a == *b;
                    self.push(Rc::new(ObjectType::Boolean(result)));
                }
                OpCode::OpSlice => {
                    let step = self.pop();
                    let end = self.pop();
                    let start = self.pop();
                    let collection = self.pop();

                    let start_idx = match &*start {
                        ObjectType::Integer(v) => Some(*v),
                        ObjectType::Nil => None,
                        _ => return InterpretResult::RuntimeError,
                    };

                    let end_idx = match &*end {
                        ObjectType::Integer(v) => Some(*v),
                        ObjectType::Nil => None,
                        _ => return InterpretResult::RuntimeError,
                    };

                    let step_value = match &*step {
                        ObjectType::Integer(v) => *v,
                        ObjectType::Nil => 1,
                        _ => return InterpretResult::RuntimeError,
                    };

                    if step_value == 0 {
                        return InterpretResult::RuntimeError;
                    }

                    match &*collection {
                        ObjectType::List(values) => {
                            let indices =
                                match slice_indices(values.len(), start_idx, end_idx, step_value) {
                                    Some(idxs) => idxs,
                                    None => return InterpretResult::RuntimeError,
                                };
                            let slice: Vec<Object> =
                                indices.into_iter().map(|idx| values[idx].clone()).collect();
                            self.push(Rc::new(ObjectType::List(slice)));
                        }
                        ObjectType::String(text) => {
                            let chars: Vec<char> = text.chars().collect();
                            let indices =
                                match slice_indices(chars.len(), start_idx, end_idx, step_value) {
                                    Some(idxs) => idxs,
                                    None => return InterpretResult::RuntimeError,
                                };
                            let slice: String = indices.into_iter().map(|idx| chars[idx]).collect();
                            self.push(Rc::new(ObjectType::String(slice)));
                        }
                        _ => return InterpretResult::RuntimeError,
                    }
                }
                OpCode::OpStrLower => {
                    let value = self.pop();
                    match &*value {
                        ObjectType::String(text) => {
                            self.push(Rc::new(ObjectType::String(text.to_lowercase())));
                        }
                        _ => return InterpretResult::RuntimeError,
                    }
                }
                OpCode::OpStrIsAlnum => {
                    let value = self.pop();
                    match &*value {
                        ObjectType::String(text) => {
                            let is_alnum =
                                !text.is_empty() && text.chars().all(|ch| ch.is_alphanumeric());
                            self.push(Rc::new(ObjectType::Boolean(is_alnum)));
                        }
                        _ => return InterpretResult::RuntimeError,
                    }
                }
                OpCode::OpStrJoin => {
                    let iterable = self.pop();
                    let separator = self.pop();

                    match (&*separator, &*iterable) {
                        (ObjectType::String(sep), ObjectType::List(values)) => {
                            let mut parts = Vec::with_capacity(values.len());
                            for value in values {
                                match &**value {
                                    ObjectType::String(text) => parts.push(text.clone()),
                                    _ => return InterpretResult::RuntimeError,
                                }
                            }
                            let joined = parts.join(sep);
                            self.push(Rc::new(ObjectType::String(joined)));
                        }
                        (ObjectType::String(sep), ObjectType::String(text)) => {
                            let chars: Vec<String> =
                                text.chars().map(|ch| ch.to_string()).collect();
                            let joined = chars.join(sep);
                            self.push(Rc::new(ObjectType::String(joined)));
                        }
                        _ => return InterpretResult::RuntimeError,
                    }
                }
                OpCode::OpRound => {
                    let digits = self.pop();
                    let value = self.pop();

                    let digits = match &*digits {
                        ObjectType::Integer(v) => *v as i32,
                        _ => return InterpretResult::RuntimeError,
                    };

                    let number = match &*value {
                        ObjectType::Integer(v) => *v as f64,
                        ObjectType::Float(v) => *v,
                        _ => return InterpretResult::RuntimeError,
                    };

                    let factor = 10f64.powi(digits.max(0));
                    let rounded = (number * factor).round() / factor;
                    self.push(Rc::new(ObjectType::Float(rounded)));
                }
                OpCode::OpZip => {
                    let arg_count = self.read_byte() as usize;
                    let star_mask = self.read_u16() as u16;

                    if arg_count == 0 {
                        self.push(Rc::new(ObjectType::List(Vec::new())));
                        continue;
                    }

                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(self.pop());
                    }
                    args.reverse();

                    let mut iterables: Vec<Vec<Object>> = Vec::new();

                    for (index, arg) in args.into_iter().enumerate() {
                        if (star_mask & (1 << index)) != 0 {
                            match &*arg {
                                ObjectType::List(values) => {
                                    for item in values {
                                        if let Some(collected) = VM::collect_iterable(item) {
                                            iterables.push(collected);
                                        } else {
                                            return InterpretResult::RuntimeError;
                                        }
                                    }
                                }
                                _ => return InterpretResult::RuntimeError,
                            }
                        } else if let Some(collected) = VM::collect_iterable(&arg) {
                            iterables.push(collected);
                        } else {
                            return InterpretResult::RuntimeError;
                        }
                    }

                    let min_len = iterables.iter().map(|items| items.len()).min().unwrap_or(0);

                    let mut zipped = Vec::with_capacity(min_len);
                    for idx in 0..min_len {
                        let mut row = Vec::with_capacity(iterables.len());
                        for iterable in &iterables {
                            row.push(iterable[idx].clone());
                        }
                        zipped.push(Rc::new(ObjectType::Tuple(row)));
                    }

                    self.push(Rc::new(ObjectType::List(zipped)));
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

                    match (&*collection, &*index) {
                        (ObjectType::List(values), ObjectType::Integer(idx))
                        | (ObjectType::Tuple(values), ObjectType::Integer(idx)) => {
                            if *idx < 0 {
                                return InterpretResult::RuntimeError;
                            }
                            let idx_usize = *idx as usize;
                            if idx_usize >= values.len() {
                                // Iteration finished; skip body.
                                if let Some(frame) = self.frames.last_mut() {
                                    frame.ip += offset;
                                }
                            } else {
                                let element = values[idx_usize].clone();
                                let next_index = (idx_usize + 1) as i64;
                                self.push(collection.clone());
                                self.push(Rc::new(ObjectType::Integer(next_index)));
                                self.push(element);
                            }
                        }
                        (ObjectType::String(text), ObjectType::Integer(idx)) => {
                            if *idx < 0 {
                                return InterpretResult::RuntimeError;
                            }
                            let chars: Vec<char> = text.chars().collect();
                            let idx_usize = *idx as usize;
                            if idx_usize >= chars.len() {
                                if let Some(frame) = self.frames.last_mut() {
                                    frame.ip += offset;
                                }
                            } else {
                                let ch = chars[idx_usize];
                                let next_index = (idx_usize + 1) as i64;
                                self.push(collection.clone());
                                self.push(Rc::new(ObjectType::Integer(next_index)));
                                self.push(Rc::new(ObjectType::String(ch.to_string())));
                            }
                        }
                        _ => {
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpLoop => {
                    let offset = self.read_u16();
                    if let Some(frame) = self.frames.last_mut() {
                        frame.ip -= offset;
                    }
                }
                OpCode::OpJumpIfFalse => {
                    let offset = self.read_u16();
                    let condition = self.peek(0).clone();
                    if !Self::is_truthy(&condition) {
                        if let Some(frame) = self.frames.last_mut() {
                            frame.ip += offset;
                        }
                    }
                }
                OpCode::OpJump => {
                    let offset = self.read_u16();
                    if let Some(frame) = self.frames.last_mut() {
                        frame.ip += offset;
                    }
                }
                OpCode::OpSetIndex => {
                    let value = self.pop();
                    let index = self.pop();
                    let collection = self.pop();

                    match (&*collection, &*index) {
                        (ObjectType::List(elements), ObjectType::Integer(idx)) => {
                            let idx_isize = *idx as isize;
                            if idx_isize < 0 || idx_isize as usize >= elements.len() {
                                return InterpretResult::RuntimeError;
                            }
                            let mut new_elements = elements.clone();
                            new_elements[idx_isize as usize] = value.clone();
                            self.push(Rc::new(ObjectType::List(new_elements)));
                        }
                        (ObjectType::Dict(entries), ObjectType::String(key)) => {
                            let mut new_entries = entries.clone();
                            if let Some(position) = new_entries
                                .iter()
                                .position(|(existing_key, _)| existing_key == key)
                            {
                                new_entries[position].1 = value.clone();
                            } else {
                                new_entries.push((key.clone(), value.clone()));
                            }
                            self.push(Rc::new(ObjectType::Dict(new_entries)));
                        }
                        _ => return InterpretResult::RuntimeError,
                    }
                }
                OpCode::OpDup => {
                    let value = self.peek(0).clone();
                    self.push(value);
                }
                OpCode::OpContains => {
                    let collection = self.pop();
                    let item = self.pop();

                    let result = match (&*collection, &*item) {
                        (ObjectType::Dict(entries), ObjectType::String(key)) => {
                            entries.iter().any(|(existing_key, _)| existing_key == key)
                        }
                        (ObjectType::List(values), _) | (ObjectType::Tuple(values), _) => {
                            values.iter().any(|element| **element == *item)
                        }
                        (ObjectType::String(text), ObjectType::String(pattern)) => {
                            text.contains(pattern)
                        }
                        _ => return InterpretResult::RuntimeError,
                    };

                    self.push(Rc::new(ObjectType::Boolean(result)));
                }
                OpCode::OpSwap => {
                    let base = self.frames.last().map(|frame| frame.slot + 1).unwrap_or(0);
                    if self.stack_top < base + 2 {
                        return InterpretResult::RuntimeError;
                    }
                    self.stack.swap(self.stack_top - 1, self.stack_top - 2);
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
                                let bound = Rc::new(ObjectType::BoundMethod(
                                    object.clone(),
                                    method.clone(),
                                ));
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
                                let bound = Rc::new(ObjectType::BoundMethod(
                                    instance.clone(),
                                    method.clone(),
                                ));
                                self.push(bound);
                            } else {
                                return InterpretResult::RuntimeError;
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
        }
    }

    fn push(&mut self, value: Object) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Object {
        self.stack_top -= 1;
        self.last_popped = self.stack[self.stack_top].clone();
        self.last_popped.clone()
    }

    fn peek(&self, distance: usize) -> &Object {
        &self.stack[self.stack_top - 1 - distance]
    }

    pub fn last_popped_stack_elem(&self) -> Rc<ObjectType> {
        self.last_popped.clone()
    }

    // Helper for testing to inspect the top of the stack without popping.
    pub fn peek_stack(&self) -> Option<Rc<ObjectType>> {
        (self.stack_top > 0).then(|| self.stack[self.stack_top - 1].clone())
    }

    fn is_truthy(value: &ObjectType) -> bool {
        match value {
            ObjectType::Nil => false,
            ObjectType::Boolean(b) => *b,
            _ => true,
        }
    }

    fn collect_iterable(value: &Object) -> Option<Vec<Object>> {
        match &**value {
            ObjectType::List(elements) => Some(elements.clone()),
            ObjectType::Tuple(elements) => Some(elements.clone()),
            ObjectType::String(text) => Some(
                text.chars()
                    .map(|ch| Rc::new(ObjectType::String(ch.to_string())))
                    .collect(),
            ),
            _ => None,
        }
    }

    fn current_chunk(&self) -> &Chunk {
        &self
            .frames
            .last()
            .expect("expected active call frame")
            .function
            .chunk
    }

    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("expected active call frame");
        let byte = frame.function.chunk.code[frame.ip];
        frame.ip += 1;
        byte
    }

    fn call_value(&mut self, arg_count: usize) -> bool {
        if self.stack_top < arg_count + 1 {
            return false;
        }
        let callee_index = self.stack_top - arg_count - 1;
        let callee = self.stack[callee_index].clone();
        match &*callee {
            ObjectType::Function(function) => {
                self.call_function(function.clone(), callee_index, arg_count, None, None)
            }
            ObjectType::NativeFunction(name, func) => {
                // Special handling for super() - it needs access to self
                if name == "super" {
                    // Get the class context from the current frame
                    let class_context = self.frames.last().and_then(|f| f.class_context.clone());

                    // Get 'self' from the current frame's first local variable (slot + 1)
                    let self_instance = if let Some(frame) = self.frames.last() {
                        self.stack[frame.slot + 1].clone()
                    } else {
                        return false;
                    };

                    // Call the native function with self as an argument
                    let args = [self_instance];
                    match func(&args, class_context) {
                        Ok(result) => {
                            self.stack_top = callee_index;
                            self.push(result);
                            true
                        }
                        Err(_) => false,
                    }
                } else {
                    // General native function call
                    let class_context = self.frames.last().and_then(|f| f.class_context.clone());
                    let args: Vec<Object> = (0..arg_count)
                        .map(|i| self.stack[callee_index + 1 + i].clone())
                        .collect();
                    match func(&args, class_context) {
                        Ok(result) => {
                            self.stack_top = callee_index;
                            self.push(result);
                            true
                        }
                        Err(_) => false,
                    }
                }
            }
            ObjectType::Class(class) => {
                // Create instance
                let instance = Rc::new(RefCell::new(InstanceObject::new(class.clone())));
                let instance_obj = Rc::new(ObjectType::Instance(instance.clone()));

                // Look for __init__ method (traverses inheritance chain)
                if let Some(init_method) = class.get_method("__init__") {
                    if let ObjectType::Function(init_func) = &*init_method {
                        // Stack layout: [class, arg1, arg2, ...]
                        // We want: [instance, instance, arg1, arg2, ...] so that after __init__ returns,
                        // one instance remains

                        // Push instance at the callee position
                        self.stack[callee_index] = instance_obj.clone();

                        // Insert instance as self parameter
                        // Shift arguments up by one to make room for self
                        for i in (callee_index + 1..self.stack_top).rev() {
                            self.stack[i + 1] = self.stack[i].clone();
                        }
                        self.stack[callee_index + 1] = instance_obj.clone();
                        self.stack_top += 1;

                        // Save instance beyond the call frame so it doesn't get overwritten
                        self.stack[self.stack_top] = instance_obj.clone();
                        let saved_instance_slot = self.stack_top;
                        self.stack_top += 1;

                        // Now call __init__: [instance, self(instance), arg1, arg2, ..., saved_instance]
                        // Pass the saved_instance_slot to call_function so handle_return can restore it
                        // Pass the class as context for super() to work in __init__
                        return self.call_function(
                            init_func.clone(),
                            callee_index,
                            arg_count + 1,
                            Some(saved_instance_slot),
                            Some(class.clone()),
                        );
                    }
                }

                // No __init__, just return the instance
                self.stack[callee_index] = instance_obj;
                self.stack_top = callee_index + 1;
                true
            }
            ObjectType::BoundMethod(instance, method) => {
                // Insert the instance as first parameter
                // Stack layout: [bound_method, arg1, arg2, ...]
                // Need: [bound_method, instance(self), arg1, arg2, ...]

                // Shift arguments to make room for self
                for i in (callee_index + 1..self.stack_top).rev() {
                    self.stack[i + 1] = self.stack[i].clone();
                }
                // Insert instance as first parameter
                self.stack[callee_index + 1] = instance.clone();
                self.stack_top += 1;

                // Get the class context from the instance for super() support
                let class_context = if let ObjectType::Instance(inst_ref) = &**instance {
                    Some(inst_ref.borrow().class.clone())
                } else {
                    None
                };

                match &**method {
                    ObjectType::Function(function) => {
                        // Call with arg_count + 1 (including self)
                        // slot points to bound_method, parameters start at slot+1
                        self.call_function(
                            function.clone(),
                            callee_index,
                            arg_count + 1,
                            None,
                            class_context,
                        )
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn call_function(
        &mut self,
        function: Rc<FunctionObject>,
        callee_index: usize,
        arg_count: usize,
        instance_slot: Option<usize>,
        class_context: Option<Rc<ClassObject>>,
    ) -> bool {
        if function.arity != arg_count {
            return false;
        }
        if self.frames.len() >= FRAMES_MAX {
            return false;
        }

        self.frames.push(CallFrame {
            function,
            ip: 0,
            slot: callee_index,
            instance_slot,
            class_context,
        });
        true
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
        let result = if self.stack_top > minimum_stack {
            Some(self.pop())
        } else {
            None
        };

        // Save the instance BEFORE resetting stack_top
        let saved_instance = instance_slot.map(|slot| self.stack[slot].clone());

        self.close_upvalues(frame_slot);

        self.frames.pop();
        self.stack_top = frame_slot;

        if self.frames.is_empty() {
            if let Some(value) = result {
                self.last_popped = value.clone();
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
            self.last_popped = value.clone();
            self.push(value);
            false
        }
    }

    fn read_u16(&mut self) -> usize {
        let high = self.read_byte() as usize;
        let low = self.read_byte() as usize;
        (high << 8) | low
    }

    fn capture_upvalue(&mut self, index: usize) -> UpvalueRef {
        for upvalue_ref in &self.open_upvalues {
            let should_take = {
                let upvalue = upvalue_ref.borrow();
                !upvalue.is_closed && upvalue.location == index
            };
            if should_take {
                return upvalue_ref.clone();
            }
        }

        let upvalue = Rc::new(RefCell::new(Upvalue::new(index, Rc::new(ObjectType::Nil))));
        self.open_upvalues.push(upvalue.clone());
        upvalue
    }

    fn close_upvalues(&mut self, from_index: usize) {
        let mut to_remove = Vec::new();
        for (idx, upvalue_ref) in self.open_upvalues.iter().enumerate() {
            let mut upvalue = upvalue_ref.borrow_mut();
            if !upvalue.is_closed && upvalue.location >= from_index {
                upvalue.closed = self.stack[upvalue.location].clone();
                upvalue.is_closed = true;
                to_remove.push(idx);
            }
        }

        for idx in to_remove.into_iter().rev() {
            self.open_upvalues.remove(idx);
        }
    }
}

fn slice_indices(
    len: usize,
    start: Option<i64>,
    end: Option<i64>,
    step: i64,
) -> Option<Vec<usize>> {
    if step == 0 {
        return None;
    }

    if len == 0 {
        return Some(Vec::new());
    }

    let len_isize = len as isize;
    let step_isize = step as isize;
    let step_positive = step_isize > 0;

    let start_idx = adjust_index(start, len_isize, false, step_positive);
    let end_idx = adjust_index(end, len_isize, true, step_positive);

    let mut indices = Vec::new();

    if step_positive {
        let mut idx = start_idx;
        while idx < end_idx {
            if idx >= 0 && idx < len_isize {
                indices.push(idx as usize);
            }
            idx += step_isize;
        }
    } else {
        let mut idx = start_idx;
        while idx > end_idx {
            if idx >= 0 && idx < len_isize {
                indices.push(idx as usize);
            }
            idx += step_isize;
        }
    }

    Some(indices)
}

fn adjust_index(index: Option<i64>, len: isize, is_end: bool, step_positive: bool) -> isize {
    let len_i64 = len as i64;
    match index {
        Some(mut value) => {
            if value < 0 {
                value += len_i64;
            }
            if step_positive {
                if value < 0 {
                    value = 0;
                }
                if value > len_i64 {
                    value = len_i64;
                }
            } else {
                if value < -1 {
                    value = -1;
                }
                if value >= len_i64 {
                    value = len_i64 - 1;
                }
            }
            value as isize
        }
        None => {
            if step_positive {
                if is_end {
                    len
                } else {
                    0
                }
            } else if is_end || len_i64 <= 0 {
                -1
            } else {
                len - 1
            }
        }
    }
}

/// Native implementation of the super() builtin function
fn native_super(args: &[Object], class_context: Option<Rc<ClassObject>>) -> Result<Object, String> {
    // super() should be called with no arguments or with instance explicitly
    if args.is_empty() {
        return Err("super() requires access to self".to_string());
    }

    if args.len() != 1 {
        return Err("super() takes at most 1 argument (self)".to_string());
    }

    // Get the current class context
    let current_class =
        class_context.ok_or_else(|| "super() can only be called inside a method".to_string())?;

    // Get the parent class
    let parent_class = current_class
        .parent
        .clone()
        .ok_or_else(|| "super() called in class with no parent".to_string())?;

    // Get self (instance) from the argument
    let instance = args[0].clone();

    // Return a SuperProxy that will handle attribute lookups in the parent class
    Ok(Rc::new(ObjectType::SuperProxy(instance, parent_class)))
}
