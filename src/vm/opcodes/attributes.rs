//! Attribute access opcodes (OpGetAttr, OpSetAttr).
//!
//! Handles attribute access for instances, classes, and function introspection.

use crate::object::{FunctionObject, FunctionPrototype, Object, ObjectType};
use crate::vm::stack_ops::Stack;
use crate::vm::InterpretResult;
use std::rc::Rc;

/// Get an attribute from an object.
///
/// Handles instance fields, class methods, super proxy, and function introspection.
pub fn op_get_attr(
    object: Object,
    attr_name: &str,
    stack: &Stack,
) -> Result<Object, InterpretResult> {
    match &*object {
        ObjectType::Instance(instance_ref) => {
            get_instance_attr(object.clone(), instance_ref, attr_name)
        }
        ObjectType::Class(class) => get_class_attr(class, attr_name),
        ObjectType::SuperProxy(instance, parent_class) => {
            get_super_proxy_attr(instance.clone(), parent_class, attr_name)
        }
        ObjectType::Function(func) => get_function_attr(func, attr_name, stack),
        ObjectType::FunctionPrototype(proto) => get_function_prototype_attr(proto, attr_name),
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Get an attribute from an instance (field or method).
fn get_instance_attr(
    object: Object,
    instance_ref: &std::cell::RefCell<crate::object::InstanceObject>,
    attr_name: &str,
) -> Result<Object, InterpretResult> {
    let instance = instance_ref.borrow();

    // First check instance fields
    if let Some(value) = instance.get_field(attr_name) {
        Ok(value)
    } else if let Some(method) = instance.class.get_method(attr_name) {
        // Create a bound method (using inheritance chain)
        Ok(Rc::new(ObjectType::BoundMethod(object, method.clone())))
    } else {
        Err(InterpretResult::RuntimeError)
    }
}

/// Get a method from a class directly.
fn get_class_attr(
    class: &Rc<crate::object::ClassObject>,
    attr_name: &str,
) -> Result<Object, InterpretResult> {
    // Access method from class directly (using inheritance chain)
    class
        .get_method(attr_name)
        .ok_or(InterpretResult::RuntimeError)
}

/// Get an attribute from a super proxy (parent class method).
fn get_super_proxy_attr(
    instance: Object,
    parent_class: &Rc<crate::object::ClassObject>,
    attr_name: &str,
) -> Result<Object, InterpretResult> {
    // Look up method in the parent class only (not the full chain)
    if let Some(method) = parent_class.get_method(attr_name) {
        // Create a bound method with the instance
        Ok(Rc::new(ObjectType::BoundMethod(instance, method.clone())))
    } else {
        Err(InterpretResult::RuntimeError)
    }
}

/// Get introspection attribute from a function object.
fn get_function_attr(
    func: &Rc<FunctionObject>,
    attr_name: &str,
    stack: &Stack,
) -> Result<Object, InterpretResult> {
    match attr_name {
        "__name__" => Ok(Rc::new(ObjectType::String(func.name.clone()))),
        "__module__" => Ok(Rc::new(ObjectType::String(func.module.clone()))),
        "__doc__" => Ok(match &func.doc {
            Some(docstring) => Rc::new(ObjectType::String(docstring.clone())),
            None => Rc::new(ObjectType::Nil),
        }),
        "__annotations__" => {
            let mut annotations: Vec<(String, Object)> = Vec::new();

            // Add parameter type annotations
            for (i, param_name) in func.parameter_names.iter().enumerate() {
                if let Some(Some(param_type)) = func.parameter_types.get(i) {
                    let type_str = Rc::new(ObjectType::String(param_type.name().to_string()));
                    annotations.push((param_name.clone(), type_str));
                }
            }

            // Add return type annotation with 'return' key
            if let Some(return_type) = &func.return_type {
                let type_str = Rc::new(ObjectType::String(return_type.name().to_string()));
                annotations.push(("return".to_string(), type_str));
            }

            Ok(Rc::new(ObjectType::Dict(annotations)))
        }
        "__code__" => Ok(Rc::new(ObjectType::CodeObject(func.chunk.clone()))),
        "__qualname__" => Ok(Rc::new(ObjectType::String(func.qualname.clone()))),
        "__globals__" => {
            // Convert HashMap to Dict format (Vec<(String, Object)>)
            let globals_vec: Vec<(String, Object)> = func
                .globals
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Ok(Rc::new(ObjectType::Dict(globals_vec)))
        }
        "__closure__" => {
            // Return a tuple of cell objects (upvalues), or None if no closure
            if func.upvalues.is_empty() {
                Ok(Rc::new(ObjectType::Nil))
            } else {
                // Create a tuple containing the closed-over values
                let cell_values: Vec<Object> = func
                    .upvalues
                    .iter()
                    .map(|upvalue_ref| {
                        let upvalue = upvalue_ref.borrow();
                        if upvalue.is_closed {
                            upvalue.closed.clone()
                        } else {
                            stack.get(upvalue.location).clone()
                        }
                    })
                    .collect();
                Ok(Rc::new(ObjectType::Tuple(cell_values)))
            }
        }
        "__defaults__" => {
            // Return a tuple of default values for parameters, or None if no defaults
            let defaults: Vec<Object> = func
                .default_values
                .iter()
                .filter_map(|opt| opt.clone())
                .collect();

            if defaults.is_empty() {
                Ok(Rc::new(ObjectType::Nil))
            } else {
                Ok(Rc::new(ObjectType::Tuple(defaults)))
            }
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Get introspection attribute from a function prototype.
fn get_function_prototype_attr(
    proto: &Rc<FunctionPrototype>,
    attr_name: &str,
) -> Result<Object, InterpretResult> {
    match attr_name {
        "__name__" => Ok(Rc::new(ObjectType::String(proto.name.clone()))),
        "__module__" => Ok(Rc::new(ObjectType::String(proto.module.clone()))),
        "__doc__" => Ok(match &proto.doc {
            Some(docstring) => Rc::new(ObjectType::String(docstring.clone())),
            None => Rc::new(ObjectType::Nil),
        }),
        "__annotations__" => {
            let mut annotations: Vec<(String, Object)> = Vec::new();

            // Add parameter type annotations
            for (i, param_name) in proto.parameter_names.iter().enumerate() {
                if let Some(Some(param_type)) = proto.parameter_types.get(i) {
                    let type_str = Rc::new(ObjectType::String(param_type.name().to_string()));
                    annotations.push((param_name.clone(), type_str));
                }
            }

            // Add return type annotation with 'return' key
            if let Some(return_type) = &proto.return_type {
                let type_str = Rc::new(ObjectType::String(return_type.name().to_string()));
                annotations.push(("return".to_string(), type_str));
            }

            Ok(Rc::new(ObjectType::Dict(annotations)))
        }
        "__code__" => Ok(Rc::new(ObjectType::CodeObject(proto.chunk.clone()))),
        "__qualname__" => Ok(Rc::new(ObjectType::String(proto.qualname.clone()))),
        "__globals__" => {
            // Prototypes don't have globals captured yet - return empty dict
            Ok(Rc::new(ObjectType::Dict(Vec::new())))
        }
        "__closure__" => {
            // Prototypes are templates, not runtime closures
            Ok(Rc::new(ObjectType::Nil))
        }
        "__defaults__" => {
            // Return a tuple of default values for parameters, or None if no defaults
            let defaults: Vec<Object> = proto
                .default_values
                .iter()
                .filter_map(|opt| opt.clone())
                .collect();

            if defaults.is_empty() {
                Ok(Rc::new(ObjectType::Nil))
            } else {
                Ok(Rc::new(ObjectType::Tuple(defaults)))
            }
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}

/// Set an attribute on an object (currently only instances).
pub fn op_set_attr(
    object: Object,
    attr_name: String,
    value: Object,
) -> Result<(), InterpretResult> {
    match &*object {
        ObjectType::Instance(instance_ref) => {
            instance_ref.borrow_mut().set_field(attr_name, value);
            Ok(())
        }
        _ => Err(InterpretResult::RuntimeError),
    }
}
