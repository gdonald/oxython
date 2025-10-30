use crate::object::ObjectType;
use crate::vm::InterpretResult;
use std::rc::Rc;

/// Handle OpAdd - Add two values (integers, floats, or strings)
pub fn op_add(a: Rc<ObjectType>, b: Rc<ObjectType>) -> Result<Rc<ObjectType>, InterpretResult> {
    match (&*a, &*b) {
        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => {
            Ok(Rc::new(ObjectType::Integer(val_a + val_b)))
        }
        (ObjectType::Float(val_a), ObjectType::Float(val_b)) => {
            Ok(Rc::new(ObjectType::Float(val_a + val_b)))
        }
        (ObjectType::Integer(val_a), ObjectType::Float(val_b)) => {
            Ok(Rc::new(ObjectType::Float(*val_a as f64 + val_b)))
        }
        (ObjectType::Float(val_a), ObjectType::Integer(val_b)) => {
            Ok(Rc::new(ObjectType::Float(val_a + *val_b as f64)))
        }
        (ObjectType::String(val_a), ObjectType::String(val_b)) => {
            let mut combined = val_a.clone();
            combined.push_str(val_b);
            Ok(Rc::new(ObjectType::String(combined)))
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpSubtract - Subtract two numeric values
pub fn op_subtract(
    a: Rc<ObjectType>,
    b: Rc<ObjectType>,
) -> Result<Rc<ObjectType>, InterpretResult> {
    match (&*a, &*b) {
        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => {
            Ok(Rc::new(ObjectType::Integer(val_a - val_b)))
        }
        (ObjectType::Float(val_a), ObjectType::Float(val_b)) => {
            Ok(Rc::new(ObjectType::Float(val_a - val_b)))
        }
        (ObjectType::Integer(val_a), ObjectType::Float(val_b)) => {
            Ok(Rc::new(ObjectType::Float(*val_a as f64 - val_b)))
        }
        (ObjectType::Float(val_a), ObjectType::Integer(val_b)) => {
            Ok(Rc::new(ObjectType::Float(val_a - *val_b as f64)))
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpMultiply - Multiply two numeric values
pub fn op_multiply(
    a: Rc<ObjectType>,
    b: Rc<ObjectType>,
) -> Result<Rc<ObjectType>, InterpretResult> {
    match (&*a, &*b) {
        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => {
            Ok(Rc::new(ObjectType::Integer(val_a * val_b)))
        }
        (ObjectType::Float(val_a), ObjectType::Float(val_b)) => {
            Ok(Rc::new(ObjectType::Float(val_a * val_b)))
        }
        (ObjectType::Integer(val_a), ObjectType::Float(val_b)) => {
            Ok(Rc::new(ObjectType::Float(*val_a as f64 * val_b)))
        }
        (ObjectType::Float(val_a), ObjectType::Integer(val_b)) => {
            Ok(Rc::new(ObjectType::Float(val_a * *val_b as f64)))
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Handle OpDivide - Divide two numeric values
pub fn op_divide(a: Rc<ObjectType>, b: Rc<ObjectType>) -> Result<Rc<ObjectType>, InterpretResult> {
    let lhs = match &*a {
        ObjectType::Integer(v) => *v as f64,
        ObjectType::Float(v) => *v,
        _ => return Err(InterpretResult::RuntimeError),
    };
    let rhs = match &*b {
        ObjectType::Integer(v) => *v as f64,
        ObjectType::Float(v) => *v,
        _ => return Err(InterpretResult::RuntimeError),
    };
    if rhs == 0.0 {
        return Err(InterpretResult::RuntimeError);
    }
    Ok(Rc::new(ObjectType::Float(lhs / rhs)))
}

/// Handle OpModulo - Modulo operation on integers
pub fn op_modulo(a: Rc<ObjectType>, b: Rc<ObjectType>) -> Result<Rc<ObjectType>, InterpretResult> {
    match (&*a, &*b) {
        (ObjectType::Integer(val_a), ObjectType::Integer(val_b)) => {
            if *val_b == 0 {
                return Err(InterpretResult::RuntimeError);
            }
            Ok(Rc::new(ObjectType::Integer(val_a % val_b)))
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}
