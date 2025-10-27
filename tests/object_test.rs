use oxython::object::{ClassObject, InstanceObject, ObjectType, Type};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn display_formats_boolean_and_nil() {
    assert_eq!(format!("{}", ObjectType::Boolean(true)), "True");
    assert_eq!(format!("{}", ObjectType::Boolean(false)), "False");
    assert_eq!(format!("{}", ObjectType::Nil), "nil");
}

#[test]
fn display_formats_float() {
    assert_eq!(format!("{}", ObjectType::Float(3.15)), "3.15");
}

#[test]
fn type_name_method_returns_correct_names() {
    assert_eq!(Type::Int.name(), "int");
    assert_eq!(Type::Float.name(), "float");
    assert_eq!(Type::Str.name(), "str");
    assert_eq!(Type::Bool.name(), "bool");
    assert_eq!(Type::List.name(), "list");
    assert_eq!(Type::Dict.name(), "dict");
    assert_eq!(Type::Tuple.name(), "tuple");
    assert_eq!(Type::Class("MyClass".to_string()).name(), "MyClass");
    assert_eq!(Type::Any.name(), "Any");
    assert_eq!(Type::None.name(), "None");
}

#[test]
fn type_display_shows_type_name() {
    assert_eq!(format!("{}", Type::Int), "int");
    assert_eq!(format!("{}", Type::Float), "float");
    assert_eq!(format!("{}", Type::Str), "str");
    assert_eq!(format!("{}", Type::Bool), "bool");
    assert_eq!(format!("{}", Type::List), "list");
    assert_eq!(format!("{}", Type::Dict), "dict");
    assert_eq!(format!("{}", Type::Tuple), "tuple");
    assert_eq!(format!("{}", Type::Class("Person".to_string())), "Person");
    assert_eq!(format!("{}", Type::Any), "Any");
    assert_eq!(format!("{}", Type::None), "None");
}

#[test]
fn type_equality_works() {
    assert_eq!(Type::Int, Type::Int);
    assert_eq!(Type::Float, Type::Float);
    assert_eq!(Type::Str, Type::Str);
    assert_eq!(
        Type::Class("MyClass".to_string()),
        Type::Class("MyClass".to_string())
    );
    assert_ne!(Type::Int, Type::Float);
    assert_ne!(Type::Class("A".to_string()), Type::Class("B".to_string()));
}

#[test]
fn object_type_get_type_returns_correct_type() {
    assert_eq!(ObjectType::Integer(42).get_type(), Type::Int);
    assert_eq!(ObjectType::Float(3.14).get_type(), Type::Float);
    assert_eq!(
        ObjectType::String("hello".to_string()).get_type(),
        Type::Str
    );
    assert_eq!(ObjectType::Boolean(true).get_type(), Type::Bool);
    assert_eq!(ObjectType::List(vec![]).get_type(), Type::List);
    assert_eq!(ObjectType::Tuple(vec![]).get_type(), Type::Tuple);
    assert_eq!(ObjectType::Dict(vec![]).get_type(), Type::Dict);
    assert_eq!(ObjectType::Nil.get_type(), Type::None);
}

#[test]
fn object_type_get_type_for_class() {
    let class = Rc::new(ClassObject::new("TestClass".to_string(), HashMap::new()));
    let obj = ObjectType::Class(class);
    assert_eq!(obj.get_type(), Type::Class("TestClass".to_string()));
}

#[test]
fn object_type_get_type_for_instance() {
    let class = Rc::new(ClassObject::new("TestClass".to_string(), HashMap::new()));
    let instance = InstanceObject::new(class);
    let obj = ObjectType::Instance(Rc::new(RefCell::new(instance)));
    assert_eq!(obj.get_type(), Type::Class("TestClass".to_string()));
}

#[test]
fn object_type_type_name_returns_correct_names() {
    assert_eq!(ObjectType::Integer(42).type_name(), "int");
    assert_eq!(ObjectType::Float(3.14).type_name(), "float");
    assert_eq!(ObjectType::String("hello".to_string()).type_name(), "str");
    assert_eq!(ObjectType::Boolean(true).type_name(), "bool");
    assert_eq!(ObjectType::List(vec![]).type_name(), "list");
    assert_eq!(ObjectType::Tuple(vec![]).type_name(), "tuple");
    assert_eq!(ObjectType::Dict(vec![]).type_name(), "dict");
    assert_eq!(ObjectType::Nil.type_name(), "NoneType");
}

#[test]
fn object_type_type_name_for_class() {
    let class = Rc::new(ClassObject::new("TestClass".to_string(), HashMap::new()));
    let obj = ObjectType::Class(class);
    assert_eq!(obj.type_name(), "type");
}

#[test]
fn object_type_type_name_for_instance() {
    let class = Rc::new(ClassObject::new("Person".to_string(), HashMap::new()));
    let instance = InstanceObject::new(class);
    let obj = ObjectType::Instance(Rc::new(RefCell::new(instance)));
    assert_eq!(obj.type_name(), "Person");
}

#[test]
fn type_clone_works() {
    let t1 = Type::Int;
    let t2 = t1.clone();
    assert_eq!(t1, t2);

    let t3 = Type::Class("MyClass".to_string());
    let t4 = t3.clone();
    assert_eq!(t3, t4);
}

#[test]
fn type_debug_format() {
    let int_type = Type::Int;
    let debug_str = format!("{:?}", int_type);
    assert!(debug_str.contains("Int"));

    let class_type = Type::Class("MyClass".to_string());
    let debug_str = format!("{:?}", class_type);
    assert!(debug_str.contains("Class"));
    assert!(debug_str.contains("MyClass"));
}
