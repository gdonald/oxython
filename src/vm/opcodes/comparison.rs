use crate::object::ObjectType;
use crate::vm::InterpretResult;
use std::rc::Rc;

/// Handle OpLess - Less than comparison
pub fn op_less(a: Rc<ObjectType>, b: Rc<ObjectType>) -> Result<bool, InterpretResult> {
    match (&*a, &*b) {
        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => Ok(val_a < val_b),
        (ObjectType::Float(val_a), ObjectType::Float(val_b)) => Ok(val_a < val_b),
        (ObjectType::Integer(val_a), ObjectType::Float(val_b)) => Ok((*val_a as f64) < *val_b),
        (ObjectType::Float(val_a), ObjectType::Integer(val_b)) => Ok(*val_a < (*val_b as f64)),
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpEqual - Equality comparison
pub fn op_equal(a: Rc<ObjectType>, b: Rc<ObjectType>) -> bool {
    *a == *b
}
