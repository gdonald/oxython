// Function-related opcodes
//
// OpCall - handled inline in VM (calls VM::call_value method)
// OpReturn - handled inline in VM (calls VM::handle_return method)
//
// Note: Function call operations are deeply integrated with the VM's call frame
// management and remain in the main VM run() loop.

use crate::object::{FunctionObject, FunctionPrototype, Object, ObjectType, UpvalueRef};
use crate::vm::upvalues;
use crate::vm::InterpretResult;
use std::collections::HashMap;
use std::rc::Rc;

/// Create a function object from a function prototype.
/// Captures upvalues from the current execution context.
///
/// # Arguments
/// * `proto` - The function prototype containing bytecode and metadata
/// * `frame_slot` - The current call frame's base stack slot
/// * `parent_upvalues` - Upvalues from the parent function (for nested closures)
/// * `globals` - The global namespace to capture
/// * `open_upvalues` - Mutable reference to the VM's open upvalues list
///
/// # Returns
/// * `Ok(Object)` - The created function object
/// * `Err(InterpretResult)` - Runtime error if upvalue capture fails
pub fn op_make_function(
    proto: Rc<FunctionPrototype>,
    frame_slot: usize,
    parent_upvalues: &[UpvalueRef],
    globals: HashMap<String, Object>,
    open_upvalues: &mut Vec<UpvalueRef>,
) -> Result<Object, InterpretResult> {
    let mut captured: Vec<UpvalueRef> = Vec::with_capacity(proto.upvalues.len());

    for descriptor in proto.upvalues.iter() {
        if descriptor.is_local {
            let stack_index = frame_slot + descriptor.index;
            let upvalue = upvalues::capture_upvalue(open_upvalues, stack_index);
            captured.push(upvalue);
        } else {
            let upvalue = parent_upvalues
                .get(descriptor.index)
                .ok_or(InterpretResult::RuntimeError)?
                .clone();
            captured.push(upvalue);
        }
    }

    let type_info = crate::object::TypeInfo {
        parameter_names: proto.parameter_names.clone(),
        parameter_types: proto.parameter_types.clone(),
        return_type: proto.return_type.clone(),
        default_values: proto.default_values.clone(),
    };

    let mut function = FunctionObject::new_with_types(
        proto.name.clone(),
        proto.arity,
        proto.chunk.clone(),
        captured,
        type_info,
        proto.module.clone(),
    );

    function.doc = proto.doc.clone();
    function.qualname = proto.qualname.clone();
    function.globals = globals;

    Ok(Rc::new(ObjectType::Function(Rc::new(function))))
}
