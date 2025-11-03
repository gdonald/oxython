use crate::object::ObjectType;
use crate::vm::upvalues;
use crate::vm::VM;
use std::rc::Rc;

impl VM {
    pub(super) fn handle_return(&mut self) -> bool {
        let (frame_slot, frame_arity, instance_slot) = if let Some(frame) = self.frames.last() {
            (frame.slot, frame.function.arity, frame.instance_slot)
        } else {
            (0, 0, None)
        };
        // Stack layout: [callee/function, params...] [return_value?]
        // frame_slot points to callee, params start at frame_slot+1
        // Return value (if any) is at frame_slot + arity + 1
        // So if there's a return value, stack_top > frame_slot + arity + 1
        let minimum_stack = frame_slot + frame_arity + 1;
        let result = if self.stack.top() > minimum_stack {
            Some(self.pop())
        } else {
            None
        };

        // Save the instance BEFORE resetting stack_top
        let saved_instance = instance_slot.map(|slot| self.stack.get(slot).clone());

        upvalues::close_upvalues(&mut self.open_upvalues, &self.stack, frame_slot);

        self.frames.pop();
        self.stack.set_top(frame_slot);

        if self.frames.is_empty() {
            if let Some(value) = result {
                self.stack.set_last_popped(value.clone());
                self.push(value);
            }
            true
        } else {
            // Check if this was an __init__ call
            let value = if let Some(instance) = saved_instance {
                // Return the saved instance instead of the function's return value
                instance
            } else {
                result.unwrap_or_else(|| Rc::new(ObjectType::Nil))
            };
            self.stack.set_last_popped(value.clone());
            self.push(value);
            false
        }
    }
}
