#![allow(dead_code)]

use crate::object::ObjectType;
use crate::vm::InterpretResult;
use std::rc::Rc;

/// Handle OpStrLower - Convert string to lowercase
pub fn op_str_lower(value: Rc<ObjectType>) -> Result<Rc<ObjectType>, InterpretResult> {
    match &*value {
        ObjectType::String(text) => Ok(Rc::new(ObjectType::String(text.to_lowercase()))),
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpStrIsAlnum - Check if string is alphanumeric
pub fn op_str_is_alnum(value: Rc<ObjectType>) -> Result<bool, InterpretResult> {
    match &*value {
        ObjectType::String(text) => {
            let is_alnum = !text.is_empty() && text.chars().all(|ch| ch.is_alphanumeric());
            Ok(is_alnum)
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpStrJoin - Join strings with separator
pub fn op_str_join(
    separator: Rc<ObjectType>,
    iterable: Rc<ObjectType>,
) -> Result<Rc<ObjectType>, InterpretResult> {
    match (&*separator, &*iterable) {
        (ObjectType::String(sep), ObjectType::List(values)) => {
            let mut parts = Vec::with_capacity(values.len());
            for value in values {
                match &**value {
                    ObjectType::String(text) => parts.push(text.clone()),
                    _ => return Err(InterpretResult::RuntimeError),
                }
            }
            let joined = parts.join(sep);
            Ok(Rc::new(ObjectType::String(joined)))
        }
        (ObjectType::String(sep), ObjectType::String(text)) => {
            let chars: Vec<String> = text.chars().map(|ch| ch.to_string()).collect();
            let joined = chars.join(sep);
            Ok(Rc::new(ObjectType::String(joined)))
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}
