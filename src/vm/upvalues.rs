use crate::object::{ObjectType, Upvalue, UpvalueRef};
use std::cell::RefCell;
use std::rc::Rc;

use super::stack::Stack;

/// Captures an upvalue for a given stack index.
///
/// If an upvalue already exists for this stack index in the open upvalues list,
/// it returns a reference to that existing upvalue. Otherwise, it creates a new
/// upvalue and adds it to the open upvalues list.
///
/// # Arguments
/// * `open_upvalues` - Mutable reference to the list of open upvalues
/// * `index` - The stack index to capture
///
/// # Returns
/// A reference-counted, interior-mutable Upvalue
pub fn capture_upvalue(open_upvalues: &mut Vec<UpvalueRef>, index: usize) -> UpvalueRef {
    for upvalue_ref in open_upvalues.iter() {
        let should_take = {
            let upvalue = upvalue_ref.borrow();
            !upvalue.is_closed && upvalue.location == index
        };
        if should_take {
            return upvalue_ref.clone();
        }
    }

    let upvalue = Rc::new(RefCell::new(Upvalue::new(index, Rc::new(ObjectType::Nil))));
    open_upvalues.push(upvalue.clone());
    upvalue
}

/// Closes upvalues that reference stack positions at or above `from_index`.
///
/// When a function returns, any local variables that were captured by closures
/// need to be "closed over" - meaning we copy their values from the stack into
/// the upvalue itself, and mark the upvalue as closed.
///
/// # Arguments
/// * `open_upvalues` - Mutable reference to the list of open upvalues
/// * `stack` - Reference to the VM stack to read values from
/// * `from_index` - The stack index from which to close upvalues
pub fn close_upvalues(open_upvalues: &mut Vec<UpvalueRef>, stack: &Stack, from_index: usize) {
    let mut to_remove = Vec::new();
    for (idx, upvalue_ref) in open_upvalues.iter().enumerate() {
        let mut upvalue = upvalue_ref.borrow_mut();
        if !upvalue.is_closed && upvalue.location >= from_index {
            upvalue.closed = stack.get(upvalue.location).clone();
            upvalue.is_closed = true;
            to_remove.push(idx);
        }
    }

    for idx in to_remove.into_iter().rev() {
        open_upvalues.remove(idx);
    }
}
