use crate::object::{ClassObject, FunctionObject, InstanceObject, Object, ObjectType};
use crate::vm::call_frame::{CallFrame, FRAMES_MAX};
use crate::vm::VM;
use std::cell::RefCell;
use std::rc::Rc;

impl VM {
    pub(super) fn call_value(&mut self, arg_count: usize) -> bool {
        if self.stack.top() < arg_count + 1 {
            return false;
        }
        let callee_index = self.stack.top() - arg_count - 1;
        let callee = self.stack.get(callee_index).clone();
        match &*callee {
            ObjectType::Function(function) => {
                self.call_function(function.clone(), callee_index, arg_count, None, None)
            }
            ObjectType::NativeFunction(name, func) => {
                // Special handling for super() - it needs access to self
                if name == "super" {
                    // Get the class context from the current frame
                    let class_context = self.frames.last().and_then(|f| f.class_context.clone());

                    // Get 'self' from the current frame's first local variable (slot + 1)
                    let self_instance = if let Some(frame) = self.frames.last() {
                        self.stack.get(frame.slot + 1).clone()
                    } else {
                        return false;
                    };

                    // Call the native function with self as an argument
                    let args = [self_instance];
                    match func(&args, class_context) {
                        Ok(result) => {
                            self.stack.set_top(callee_index);
                            self.push(result);
                            true
                        }
                        Err(_) => false,
                    }
                } else {
                    // General native function call
                    let class_context = self.frames.last().and_then(|f| f.class_context.clone());
                    let args: Vec<Object> = (0..arg_count)
                        .map(|i| self.stack.get(callee_index + 1 + i).clone())
                        .collect();
                    match func(&args, class_context) {
                        Ok(result) => {
                            self.stack.set_top(callee_index);
                            self.push(result);
                            true
                        }
                        Err(_) => false,
                    }
                }
            }
            ObjectType::Class(class) => {
                // Create instance
                let instance = Rc::new(RefCell::new(InstanceObject::new(class.clone())));
                let instance_obj = Rc::new(ObjectType::Instance(instance.clone()));

                // Look for __init__ method (traverses inheritance chain)
                if let Some(init_method) = class.get_method("__init__") {
                    if let ObjectType::Function(init_func) = &*init_method {
                        // Stack layout: [class, arg1, arg2, ...]
                        // We want: [instance, instance, arg1, arg2, ...] so that after __init__ returns,
                        // one instance remains

                        // Push instance at the callee position
                        self.stack.set(callee_index, instance_obj.clone());

                        // Insert instance as self parameter
                        // Shift arguments up by one to make room for self
                        let stack_top = self.stack.top();
                        for i in (callee_index + 1..stack_top).rev() {
                            let value = self.stack.get(i).clone();
                            self.stack.set(i + 1, value);
                        }
                        self.stack.set(callee_index + 1, instance_obj.clone());
                        self.stack.set_top(stack_top + 1);

                        // Save instance beyond the call frame so it doesn't get overwritten
                        let new_top = self.stack.top();
                        self.stack.set(new_top, instance_obj.clone());
                        let saved_instance_slot = new_top;
                        self.stack.set_top(new_top + 1);

                        // Now call __init__: [instance, self(instance), arg1, arg2, ..., saved_instance]
                        // Pass the saved_instance_slot to call_function so handle_return can restore it
                        // Pass the class as context for super() to work in __init__
                        return self.call_function(
                            init_func.clone(),
                            callee_index,
                            arg_count + 1,
                            Some(saved_instance_slot),
                            Some(class.clone()),
                        );
                    }
                }

                // No __init__, just return the instance
                self.stack.set(callee_index, instance_obj);
                self.stack.set_top(callee_index + 1);
                true
            }
            ObjectType::BoundMethod(instance, method) => {
                // Insert the instance as first parameter
                // Stack layout: [bound_method, arg1, arg2, ...]
                // Need: [bound_method, instance(self), arg1, arg2, ...]

                // Shift arguments to make room for self
                let stack_top = self.stack.top();
                for i in (callee_index + 1..stack_top).rev() {
                    let value = self.stack.get(i).clone();
                    self.stack.set(i + 1, value);
                }
                // Insert instance as first parameter
                self.stack.set(callee_index + 1, instance.clone());
                self.stack.set_top(stack_top + 1);

                // Get the class context from the instance for super() support
                let class_context = if let ObjectType::Instance(inst_ref) = &**instance {
                    Some(inst_ref.borrow().class.clone())
                } else {
                    None
                };

                match &**method {
                    ObjectType::Function(function) => {
                        // Call with arg_count + 1 (including self)
                        // slot points to bound_method, parameters start at slot+1
                        self.call_function(
                            function.clone(),
                            callee_index,
                            arg_count + 1,
                            None,
                            class_context,
                        )
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub(super) fn call_function(
        &mut self,
        function: Rc<FunctionObject>,
        callee_index: usize,
        arg_count: usize,
        instance_slot: Option<usize>,
        class_context: Option<Rc<ClassObject>>,
    ) -> bool {
        // Validate argument count with default parameters support
        // arg_count must be between required_args and arity (inclusive)
        if arg_count < function.required_args || arg_count > function.arity {
            return false;
        }

        if self.frames.len() >= FRAMES_MAX {
            return false;
        }

        // Fill in missing arguments with default values
        if arg_count < function.arity {
            // Push default values for missing parameters
            for i in arg_count..function.arity {
                if let Some(Some(default_value)) = function.default_values.get(i) {
                    self.push(default_value.clone());
                } else {
                    // This should not happen if required_args is calculated correctly
                    return false;
                }
            }
        }

        self.frames.push(CallFrame::new(
            function,
            callee_index,
            instance_slot,
            class_context,
        ));
        true
    }
}
