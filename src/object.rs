use std::fmt;
use std::rc::Rc;

/// A type alias for a reference-counted Object.
/// Using Rc allows multiple parts of the interpreter to "own" the same object,
/// which is essential for a dynamically-typed language with variables and data structures.
pub type Object = Rc<ObjectType>;

/// Represents all possible data types that can exist in the oxython language.
/// By wrapping primitive Rust types, we create a unified object model.
#[derive(Debug, PartialEq)]
pub enum ObjectType {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Object>),
    Dict(Vec<(String, Object)>),
    Nil,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectType::Integer(val) => write!(f, "{}", val),
            ObjectType::Float(val) => write!(f, "{}", val),
            ObjectType::String(val) => write!(f, "{}", val),
            ObjectType::Boolean(val) => write!(f, "{}", val),
            ObjectType::List(values) => {
                write!(f, "[")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
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
            ObjectType::Nil => write!(f, "nil"),
        }
    }
}
