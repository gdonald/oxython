#![allow(dead_code)]

use crate::object::{Object, ObjectType};
use crate::vm::collections::collect_iterable;
use crate::vm::InterpretResult;
use std::rc::Rc;

/// Handle OpRound - Round a float to specified decimal places
pub fn op_round(
    value: Rc<ObjectType>,
    digits: Rc<ObjectType>,
) -> Result<Rc<ObjectType>, InterpretResult> {
    let digits = match &*digits {
        ObjectType::Integer(v) => *v as i32,
        _ => return Err(InterpretResult::RuntimeError),
    };

    let number = match &*value {
        ObjectType::Integer(v) => *v as f64,
        ObjectType::Float(v) => *v,
        _ => return Err(InterpretResult::RuntimeError),
    };

    let factor = 10f64.powi(digits.max(0));
    let rounded = (number * factor).round() / factor;
    Ok(Rc::new(ObjectType::Float(rounded)))
}

/// Handle OpToList - Convert an iterable to a list
pub fn op_to_list(value: Rc<ObjectType>) -> Result<Rc<ObjectType>, InterpretResult> {
    match collect_iterable(&value) {
        Some(elements) => Ok(Rc::new(ObjectType::List(elements))),
        None => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpZip - Zip multiple iterables together
/// Returns a list of tuples
pub fn op_zip(args: Vec<Object>, star_mask: u16) -> Result<Rc<ObjectType>, InterpretResult> {
    if args.is_empty() {
        return Ok(Rc::new(ObjectType::List(Vec::new())));
    }

    let mut iterables: Vec<Vec<Object>> = Vec::new();

    for (index, arg) in args.into_iter().enumerate() {
        if (star_mask & (1 << index)) != 0 {
            match &*arg {
                ObjectType::List(values) => {
                    for item in values {
                        if let Some(collected) = collect_iterable(item) {
                            iterables.push(collected);
                        } else {
                            return Err(InterpretResult::RuntimeError);
                        }
                    }
                }
                _ => return Err(InterpretResult::RuntimeError),
            }
        } else if let Some(collected) = collect_iterable(&arg) {
            iterables.push(collected);
        } else {
            return Err(InterpretResult::RuntimeError);
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

    Ok(Rc::new(ObjectType::List(zipped)))
}
