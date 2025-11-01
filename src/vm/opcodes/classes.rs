// Class and OOP-related opcodes
//
// OpGetAttr, OpSetAttr - moved to opcodes/attributes.rs
//
// Note: Class operations involve complex object manipulation and method resolution.

use crate::object::{ClassObject, Object, ObjectType};
use crate::vm::InterpretResult;
use std::collections::HashMap;
use std::rc::Rc;

/// Create a class object from a class name and methods.
///
/// # Arguments
/// * `class_name` - The name of the class
/// * `method_names` - Names of the class methods
/// * `method_funcs` - Function objects for each method
///
/// # Returns
/// * `Ok(Object)` - The created class object
/// * `Err(InterpretResult)` - Runtime error if inputs are invalid
pub fn op_make_class(
    class_name: String,
    method_names: Vec<String>,
    method_funcs: Vec<Object>,
) -> Result<Object, InterpretResult> {
    if method_names.len() != method_funcs.len() {
        return Err(InterpretResult::RuntimeError);
    }

    let mut methods = HashMap::new();
    for (name, func) in method_names.into_iter().zip(method_funcs.into_iter()) {
        methods.insert(name, func);
    }

    let class = Rc::new(ClassObject::new(class_name, methods));
    Ok(Rc::new(ObjectType::Class(class)))
}

/// Set up class inheritance by creating a new child class with a parent.
///
/// # Arguments
/// * `child` - The child class object
/// * `parent` - The parent class object
///
/// # Returns
/// * `Ok(Object)` - The child class with parent set
/// * `Err(InterpretResult)` - Runtime error if either argument is not a class
pub fn op_inherit(child: Object, parent: Object) -> Result<Object, InterpretResult> {
    let parent_class = match &*parent {
        ObjectType::Class(class) => class.clone(),
        _ => return Err(InterpretResult::RuntimeError),
    };

    let child_class = match &*child {
        ObjectType::Class(class) => class,
        _ => return Err(InterpretResult::RuntimeError),
    };

    let new_child = Rc::new(ClassObject::new_with_parent(
        child_class.name.clone(),
        child_class.methods.clone(),
        parent_class,
    ));

    Ok(Rc::new(ObjectType::Class(new_child)))
}
