use crate::bytecode::Chunk;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// A type alias for a reference-counted Object.
/// Using Rc allows multiple parts of the interpreter to "own" the same object,
/// which is essential for a dynamically-typed language with variables and data structures.
pub type Object = Rc<ObjectType>;

/// Describes how a closure captures a variable from an outer scope.
#[derive(Clone, Debug, PartialEq)]
pub struct UpvalueDescriptor {
    pub is_local: bool,
    pub index: usize,
}

/// Represents a live upvalue captured by a closure.
#[derive(Debug)]
pub struct Upvalue {
    pub location: usize,
    pub closed: Object,
    pub is_closed: bool,
}

impl Upvalue {
    pub fn new(location: usize, closed: Object) -> Self {
        Upvalue {
            location,
            closed,
            is_closed: false,
        }
    }
}

pub type UpvalueRef = Rc<RefCell<Upvalue>>;

/// Represents a compiled function object.
#[derive(Clone, Debug)]
pub struct FunctionObject {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
    pub upvalues: Vec<UpvalueRef>,
}

impl FunctionObject {
    pub fn new(name: String, arity: usize, chunk: Chunk, upvalues: Vec<UpvalueRef>) -> Self {
        FunctionObject {
            name,
            arity,
            chunk,
            upvalues,
        }
    }
}

impl PartialEq for FunctionObject {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}

/// Prototype stored in constants that the VM turns into runtime function objects.
#[derive(Clone, Debug)]
pub struct FunctionPrototype {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
    pub upvalues: Vec<UpvalueDescriptor>,
}

impl FunctionPrototype {
    pub fn new(name: String, arity: usize, chunk: Chunk, upvalues: Vec<UpvalueDescriptor>) -> Self {
        FunctionPrototype {
            name,
            arity,
            chunk,
            upvalues,
        }
    }
}

impl PartialEq for FunctionPrototype {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity && self.upvalues == other.upvalues
    }
}

/// Represents a class definition.
#[derive(Clone, Debug)]
pub struct ClassObject {
    pub name: String,
    pub methods: HashMap<String, Object>,
    pub parent: Option<Rc<ClassObject>>,
}

impl ClassObject {
    pub fn new(name: String, methods: HashMap<String, Object>) -> Self {
        ClassObject {
            name,
            methods,
            parent: None,
        }
    }

    pub fn new_with_parent(
        name: String,
        methods: HashMap<String, Object>,
        parent: Rc<ClassObject>,
    ) -> Self {
        ClassObject {
            name,
            methods,
            parent: Some(parent),
        }
    }

    /// Looks up a method in this class or its parent chain
    pub fn get_method(&self, name: &str) -> Option<Object> {
        // First check this class's methods
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        }

        // Then check parent class if it exists
        if let Some(ref parent) = self.parent {
            return parent.get_method(name);
        }

        None
    }
}

impl PartialEq for ClassObject {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Represents an instance of a class.
#[derive(Clone, Debug)]
pub struct InstanceObject {
    pub class: Rc<ClassObject>,
    pub fields: Vec<(String, Object)>,
}

impl InstanceObject {
    pub fn new(class: Rc<ClassObject>) -> Self {
        InstanceObject {
            class,
            fields: Vec::new(),
        }
    }

    pub fn get_field(&self, name: &str) -> Option<Object> {
        self.fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }

    pub fn set_field(&mut self, name: String, value: Object) {
        if let Some((_, existing)) = self.fields.iter_mut().find(|(k, _)| k == &name) {
            *existing = value;
        } else {
            self.fields.push((name, value));
        }
    }
}

impl PartialEq for InstanceObject {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.class, &other.class) && self.fields == other.fields
    }
}

/// Type alias for native (Rust-implemented) functions
/// The function receives:
/// - args: The arguments passed to the function
/// - class_context: The class context from the current call frame (for super())
pub type NativeFn =
    fn(args: &[Object], class_context: Option<Rc<ClassObject>>) -> Result<Object, String>;

/// Represents all possible data types that can exist in the oxython language.
/// By wrapping primitive Rust types, we create a unified object model.
#[derive(Debug)]
pub enum ObjectType {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Object>),
    Tuple(Vec<Object>),
    Dict(Vec<(String, Object)>),
    FunctionPrototype(Rc<FunctionPrototype>),
    Function(Rc<FunctionObject>),
    NativeFunction(String, NativeFn), // (name, function pointer)
    Class(Rc<ClassObject>),
    Instance(Rc<RefCell<InstanceObject>>),
    BoundMethod(Object, Object),         // (instance, method function)
    SuperProxy(Object, Rc<ClassObject>), // (instance, parent class to lookup methods in)
    Nil,
}

impl PartialEq for ObjectType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjectType::Integer(a), ObjectType::Integer(b)) => a == b,
            (ObjectType::Float(a), ObjectType::Float(b)) => a == b,
            (ObjectType::String(a), ObjectType::String(b)) => a == b,
            (ObjectType::Boolean(a), ObjectType::Boolean(b)) => a == b,
            (ObjectType::List(a), ObjectType::List(b)) => a == b,
            (ObjectType::Tuple(a), ObjectType::Tuple(b)) => a == b,
            (ObjectType::Dict(a), ObjectType::Dict(b)) => a == b,
            (ObjectType::FunctionPrototype(a), ObjectType::FunctionPrototype(b)) => a == b,
            (ObjectType::Function(a), ObjectType::Function(b)) => a == b,
            (ObjectType::NativeFunction(name_a, _), ObjectType::NativeFunction(name_b, _)) => {
                name_a == name_b
            }
            (ObjectType::Class(a), ObjectType::Class(b)) => a == b,
            (ObjectType::Instance(a), ObjectType::Instance(b)) => a == b,
            (ObjectType::BoundMethod(inst_a, meth_a), ObjectType::BoundMethod(inst_b, meth_b)) => {
                inst_a == inst_b && meth_a == meth_b
            }
            (ObjectType::SuperProxy(inst_a, class_a), ObjectType::SuperProxy(inst_b, class_b)) => {
                inst_a == inst_b && class_a == class_b
            }
            (ObjectType::Nil, ObjectType::Nil) => true,
            _ => false,
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectType::Integer(val) => write!(f, "{}", val),
            ObjectType::Float(val) => write!(f, "{}", val),
            ObjectType::String(val) => write!(f, "{}", val),
            ObjectType::Boolean(val) => {
                if *val {
                    write!(f, "True")
                } else {
                    write!(f, "False")
                }
            }
            ObjectType::List(values) => {
                write!(f, "[")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    match &**value {
                        ObjectType::String(text) => write!(f, "'{}'", text)?,
                        _ => write!(f, "{}", value)?,
                    }
                }
                write!(f, "]")
            }
            ObjectType::Tuple(values) => {
                write!(f, "(")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    match &**value {
                        ObjectType::String(text) => write!(f, "'{}'", text)?,
                        _ => write!(f, "{}", value)?,
                    }
                }
                if values.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
            }
            ObjectType::Dict(entries) => {
                write!(f, "{{")?;
                for (idx, (key, value)) in entries.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "'{}': {}", key, value)?;
                }
                write!(f, "}}")
            }
            ObjectType::FunctionPrototype(proto) => write!(f, "<fn {}>", proto.name),
            ObjectType::Function(func) => write!(f, "<function {}>", func.name),
            ObjectType::NativeFunction(name, _) => write!(f, "<built-in function {}>", name),
            ObjectType::Class(class) => write!(f, "<class '{}'>", class.name),
            ObjectType::Instance(instance) => {
                write!(f, "<{} instance>", instance.borrow().class.name)
            }
            ObjectType::BoundMethod(_, method) => match &**method {
                ObjectType::Function(func) => write!(f, "<bound method {}>", func.name),
                _ => write!(f, "<bound method>"),
            },
            ObjectType::SuperProxy(_, _) => write!(f, "<super>"),
            ObjectType::Nil => write!(f, "nil"),
        }
    }
}
