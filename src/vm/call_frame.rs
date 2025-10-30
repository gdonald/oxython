use crate::object::{ClassObject, FunctionObject};
use std::rc::Rc;

pub const FRAMES_MAX: usize = 64;

pub struct CallFrame {
    pub function: Rc<FunctionObject>,
    pub ip: usize,
    pub slot: usize,
    pub instance_slot: Option<usize>, // For __init__ calls, where to find the instance to return
    pub class_context: Option<Rc<ClassObject>>, // For tracking which class a method belongs to (for super())
}

impl CallFrame {
    pub fn new(
        function: Rc<FunctionObject>,
        slot: usize,
        instance_slot: Option<usize>,
        class_context: Option<Rc<ClassObject>>,
    ) -> Self {
        Self {
            function,
            ip: 0,
            slot,
            instance_slot,
            class_context,
        }
    }
}
