use crate::bytecode::Chunk;
use std::cell::RefCell;
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

/// Represents all possible data types that can exist in the oxython language.
/// By wrapping primitive Rust types, we create a unified object model.
#[derive(Debug, PartialEq)]
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
    Nil,
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
            ObjectType::Nil => write!(f, "nil"),
        }
    }
}
