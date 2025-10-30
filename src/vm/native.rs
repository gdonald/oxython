use crate::object::{ClassObject, Object, ObjectType};
use std::collections::HashMap;
use std::rc::Rc;

/// Registers all builtin native functions into the global namespace.
///
/// This function populates the globals HashMap with native function implementations
/// that are available to all Python code. Currently, this includes:
/// - `super()` - Access parent class methods in inheritance hierarchies
///
/// # Arguments
/// * `globals` - Mutable reference to the VM's global namespace
pub fn register_builtins(globals: &mut HashMap<String, Object>) {
    // Register the super() builtin
    globals.insert(
        "super".to_string(),
        Rc::new(ObjectType::NativeFunction(
            "super".to_string(),
            native_super,
        )),
    );
}

/// Native implementation of the super() builtin function.
///
/// The `super()` function is used to access methods from a parent class in an
/// inheritance hierarchy. It returns a SuperProxy object that allows method
/// lookups in the parent class while maintaining the current instance context.
///
/// # Arguments
/// * `args` - Slice of arguments passed to super(). Should contain exactly one argument: self
/// * `class_context` - The class context from which super() is being called
///
/// # Returns
/// A Result containing either:
/// - Ok(SuperProxy) - A proxy object for accessing parent class methods
/// - Err(String) - An error message if the call is invalid
///
/// # Errors
/// This function will return an error if:
/// - Called with no arguments (no access to self)
/// - Called with more than 1 argument
/// - Called outside a method context (no class_context)
/// - Called in a class with no parent class
///
/// # Example Python Usage
/// ```python
/// class Parent:
///     def greet(self):
///         print("Hello from Parent")
///
/// class Child(Parent):
///     def greet(self):
///         super().greet()  # Calls Parent.greet()
///         print("Hello from Child")
/// ```
pub fn native_super(
    args: &[Object],
    class_context: Option<Rc<ClassObject>>,
) -> Result<Object, String> {
    // super() should be called with no arguments or with instance explicitly
    if args.is_empty() {
        return Err("super() requires access to self".to_string());
    }

    if args.len() != 1 {
        return Err("super() takes at most 1 argument (self)".to_string());
    }

    // Get the current class context
    let current_class =
        class_context.ok_or_else(|| "super() can only be called inside a method".to_string())?;

    // Get the parent class
    let parent_class = current_class
        .parent
        .clone()
        .ok_or_else(|| "super() called in class with no parent".to_string())?;

    // Get self (instance) from the argument
    let instance = args[0].clone();

    // Return a SuperProxy that will handle attribute lookups in the parent class
    Ok(Rc::new(ObjectType::SuperProxy(instance, parent_class)))
}
