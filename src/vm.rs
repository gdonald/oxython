use crate::bytecode::{Chunk, OpCode};
use crate::object::{Object, ObjectType};
use std::collections::HashMap;
use std::rc::Rc;

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip: usize, // Instruction Pointer
    stack: [Object; STACK_MAX],
    stack_top: usize,
    globals: HashMap<String, Object>,
    last_popped: Object,
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
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: [(); STACK_MAX].map(|_| default_obj.clone()),
            stack_top: 0,
            globals: HashMap::new(),
            last_popped: default_obj,
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = self.chunk.code[self.ip];
            self.ip += 1;

            match OpCode::from(instruction) {
                OpCode::OpConstant => {
                    let const_idx = self.chunk.code[self.ip] as usize;
                    self.ip += 1;
                    let constant = self.chunk.constants[const_idx].clone();
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
                    let name_idx = self.chunk.code[self.ip] as usize;
                    self.ip += 1;
                    if let ObjectType::String(name) = &*self.chunk.constants[name_idx] {
                        self.globals.insert(name.clone(), self.peek(0).clone());
                        self.pop();
                    }
                }
                OpCode::OpGetGlobal => {
                    let name_idx = self.chunk.code[self.ip] as usize;
                    self.ip += 1;
                    if let ObjectType::String(name) = &*self.chunk.constants[name_idx] {
                        if let Some(value) = self.globals.get(name) {
                            self.push(value.clone());
                        } else {
                            // Runtime error: undefined variable
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpSetGlobal => {
                    let name_idx = self.chunk.code[self.ip] as usize;
                    self.ip += 1;
                    if let ObjectType::String(name) = &*self.chunk.constants[name_idx] {
                        if self.globals.contains_key(name) {
                            let value = self.peek(0).clone();
                            self.globals.insert(name.clone(), value);
                        } else {
                            return InterpretResult::RuntimeError;
                        }
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
                        (ObjectType::List(values), ObjectType::Integer(idx)) => {
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
                    let arg_count = self.chunk.code[self.ip] as usize;
                    self.ip += 1;
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
                        zipped.push(Rc::new(ObjectType::List(row)));
                    }

                    self.push(Rc::new(ObjectType::List(zipped)));
                }
                OpCode::OpReturn => {
                    // OpReturn simply ends the execution loop.
                    // The final value of the script (if any) would have been left on the stack.
                    return InterpretResult::Ok;
                }
                OpCode::OpPop => {
                    self.pop();
                }
                OpCode::OpIterNext => {
                    let offset = self.read_u16();
                    let index = self.pop();
                    let collection = self.pop();

                    match (&*collection, &*index) {
                        (ObjectType::List(values), ObjectType::Integer(idx)) => {
                            if *idx < 0 {
                                return InterpretResult::RuntimeError;
                            }
                            let idx_usize = *idx as usize;
                            if idx_usize >= values.len() {
                                // Iteration finished; skip body.
                                self.ip += offset;
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
                                self.ip += offset;
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
                    self.ip -= offset;
                }
                OpCode::OpJumpIfFalse => {
                    let offset = self.read_u16();
                    let condition = self.peek(0).clone();
                    if !Self::is_truthy(&condition) {
                        self.ip += offset;
                    }
                }
                OpCode::OpJump => {
                    let offset = self.read_u16();
                    self.ip += offset;
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
                        (ObjectType::List(values), _) => {
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
                    if self.stack_top < 2 {
                        return InterpretResult::RuntimeError;
                    }
                    self.stack.swap(self.stack_top - 1, self.stack_top - 2);
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
            ObjectType::String(text) => Some(
                text.chars()
                    .map(|ch| Rc::new(ObjectType::String(ch.to_string())))
                    .collect(),
            ),
            _ => None,
        }
    }

    fn read_u16(&mut self) -> usize {
        let high = self.chunk.code[self.ip] as usize;
        let low = self.chunk.code[self.ip + 1] as usize;
        self.ip += 2;
        (high << 8) | low
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
