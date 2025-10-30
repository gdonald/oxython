#![allow(dead_code)]

use crate::object::{Object, ObjectType};
use crate::vm::collections::slice_indices;
use crate::vm::InterpretResult;
use std::rc::Rc;

/// Handle OpIndex - Index into a collection (list, tuple, dict)
pub fn op_index(
    collection: Rc<ObjectType>,
    index: Rc<ObjectType>,
) -> Result<Rc<ObjectType>, InterpretResult> {
    match (&*collection, &*index) {
        (ObjectType::List(values), ObjectType::Integer(idx))
        | (ObjectType::Tuple(values), ObjectType::Integer(idx)) => {
            let mut idx_isize = *idx as isize;
            let len = values.len() as isize;
            if idx_isize < 0 {
                idx_isize += len;
            }
            if idx_isize < 0 || idx_isize as usize >= values.len() {
                return Err(InterpretResult::RuntimeError);
            }
            let element = values[idx_isize as usize].clone();
            Ok(element)
        }
        (ObjectType::Dict(entries), ObjectType::String(key)) => {
            if let Some((_, value)) = entries.iter().find(|(existing_key, _)| existing_key == key) {
                Ok(value.clone())
            } else {
                Err(InterpretResult::RuntimeError)
            }
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpSetIndex - Set a value in a collection (list, dict)
pub fn op_set_index(
    collection: Rc<ObjectType>,
    index: Rc<ObjectType>,
    value: Rc<ObjectType>,
) -> Result<Rc<ObjectType>, InterpretResult> {
    match (&*collection, &*index) {
        (ObjectType::List(elements), ObjectType::Integer(idx)) => {
            let idx_isize = *idx as isize;
            if idx_isize < 0 || idx_isize as usize >= elements.len() {
                return Err(InterpretResult::RuntimeError);
            }
            let mut new_elements = elements.clone();
            new_elements[idx_isize as usize] = value.clone();
            Ok(Rc::new(ObjectType::List(new_elements)))
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
            Ok(Rc::new(ObjectType::Dict(new_entries)))
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpLen - Get length of a collection
pub fn op_len(value: Rc<ObjectType>) -> Result<i64, InterpretResult> {
    match &*value {
        ObjectType::List(values) => Ok(values.len() as i64),
        ObjectType::Tuple(values) => Ok(values.len() as i64),
        ObjectType::String(text) => Ok(text.chars().count() as i64),
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpAppend - Append a value to a list
pub fn op_append(
    collection: Rc<ObjectType>,
    value: Rc<ObjectType>,
) -> Result<Rc<ObjectType>, InterpretResult> {
    if let ObjectType::List(elements) = &*collection {
        let mut new_elements = elements.clone();
        new_elements.push(value.clone());
        Ok(Rc::new(ObjectType::List(new_elements)))
    } else {
        Err(InterpretResult::RuntimeError)
    }
}

/// Handle OpRange - Create a range of integers
pub fn op_range(
    start: Rc<ObjectType>,
    end: Rc<ObjectType>,
) -> Result<Rc<ObjectType>, InterpretResult> {
    let (start_val, end_val) = match (&*start, &*end) {
        (ObjectType::Integer(start_int), ObjectType::Integer(end_int)) => (*start_int, *end_int),
        _ => return Err(InterpretResult::RuntimeError),
    };

    let mut elements: Vec<Object> = Vec::new();
    if start_val < end_val {
        for value in start_val..end_val {
            elements.push(Rc::new(ObjectType::Integer(value)));
        }
    }

    Ok(Rc::new(ObjectType::List(elements)))
}

/// Handle OpContains - Check if item is in collection
pub fn op_contains(
    item: Rc<ObjectType>,
    collection: Rc<ObjectType>,
) -> Result<bool, InterpretResult> {
    let result = match (&*collection, &*item) {
        (ObjectType::Dict(entries), ObjectType::String(key)) => {
            entries.iter().any(|(existing_key, _)| existing_key == key)
        }
        (ObjectType::List(values), _) | (ObjectType::Tuple(values), _) => {
            values.iter().any(|element| **element == *item)
        }
        (ObjectType::String(text), ObjectType::String(pattern)) => text.contains(pattern),
        _ => return Err(InterpretResult::RuntimeError),
    };

    Ok(result)
}

/// Handle OpSlice - Slice a collection (list or string)
pub fn op_slice(
    collection: Rc<ObjectType>,
    start: Rc<ObjectType>,
    end: Rc<ObjectType>,
    step: Rc<ObjectType>,
) -> Result<Rc<ObjectType>, InterpretResult> {
    let start_idx = match &*start {
        ObjectType::Integer(v) => Some(*v),
        ObjectType::Nil => None,
        _ => return Err(InterpretResult::RuntimeError),
    };

    let end_idx = match &*end {
        ObjectType::Integer(v) => Some(*v),
        ObjectType::Nil => None,
        _ => return Err(InterpretResult::RuntimeError),
    };

    let step_value = match &*step {
        ObjectType::Integer(v) => *v,
        ObjectType::Nil => 1,
        _ => return Err(InterpretResult::RuntimeError),
    };

    if step_value == 0 {
        return Err(InterpretResult::RuntimeError);
    }

    match &*collection {
        ObjectType::List(values) => {
            let indices = match slice_indices(values.len(), start_idx, end_idx, step_value) {
                Some(idxs) => idxs,
                None => return Err(InterpretResult::RuntimeError),
            };
            let slice: Vec<Object> = indices.into_iter().map(|idx| values[idx].clone()).collect();
            Ok(Rc::new(ObjectType::List(slice)))
        }
        ObjectType::String(text) => {
            let chars: Vec<char> = text.chars().collect();
            let indices = match slice_indices(chars.len(), start_idx, end_idx, step_value) {
                Some(idxs) => idxs,
                None => return Err(InterpretResult::RuntimeError),
            };
            let slice: String = indices.into_iter().map(|idx| chars[idx]).collect();
            Ok(Rc::new(ObjectType::String(slice)))
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}
