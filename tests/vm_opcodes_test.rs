use oxython::object::{ClassObject, ObjectType};
use oxython::vm::native::native_super;
use oxython::vm::opcodes::strings::{op_str_is_alnum, op_str_join, op_str_lower};
use oxython::vm::values::is_truthy;
use oxython::vm::InterpretResult;
use std::collections::HashMap;
use std::rc::Rc;

// ============================================================================
// String Operations Tests
// ============================================================================

#[test]
fn test_op_str_lower_converts_to_lowercase() {
    let input = Rc::new(ObjectType::String("HELLO World".to_string()));
    let result = op_str_lower(input).unwrap();
    match &*result {
        ObjectType::String(text) => assert_eq!(text, "hello world"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_op_str_lower_handles_empty_string() {
    let input = Rc::new(ObjectType::String("".to_string()));
    let result = op_str_lower(input).unwrap();
    match &*result {
        ObjectType::String(text) => assert_eq!(text, ""),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_op_str_lower_errors_on_non_string() {
    let input = Rc::new(ObjectType::Integer(42));
    let result = op_str_lower(input);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

#[test]
fn test_op_str_is_alnum_returns_true_for_alphanumeric() {
    let input = Rc::new(ObjectType::String("abc123".to_string()));
    let result = op_str_is_alnum(input).unwrap();
    assert!(result);
}

#[test]
fn test_op_str_is_alnum_returns_false_for_non_alphanumeric() {
    let input = Rc::new(ObjectType::String("abc-123".to_string()));
    let result = op_str_is_alnum(input).unwrap();
    assert!(!result);
}

#[test]
fn test_op_str_is_alnum_returns_false_for_empty_string() {
    let input = Rc::new(ObjectType::String("".to_string()));
    let result = op_str_is_alnum(input).unwrap();
    assert!(!result);
}

#[test]
fn test_op_str_is_alnum_returns_true_for_letters_only() {
    let input = Rc::new(ObjectType::String("abcXYZ".to_string()));
    let result = op_str_is_alnum(input).unwrap();
    assert!(result);
}

#[test]
fn test_op_str_is_alnum_returns_true_for_numbers_only() {
    let input = Rc::new(ObjectType::String("123456".to_string()));
    let result = op_str_is_alnum(input).unwrap();
    assert!(result);
}

#[test]
fn test_op_str_is_alnum_errors_on_non_string() {
    let input = Rc::new(ObjectType::Integer(42));
    let result = op_str_is_alnum(input);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

#[test]
fn test_op_str_join_joins_list_of_strings() {
    let sep = Rc::new(ObjectType::String(", ".to_string()));
    let items = Rc::new(ObjectType::List(vec![
        Rc::new(ObjectType::String("a".to_string())),
        Rc::new(ObjectType::String("b".to_string())),
        Rc::new(ObjectType::String("c".to_string())),
    ]));
    let result = op_str_join(sep, items).unwrap();
    match &*result {
        ObjectType::String(text) => assert_eq!(text, "a, b, c"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_op_str_join_handles_empty_list() {
    let sep = Rc::new(ObjectType::String(", ".to_string()));
    let items = Rc::new(ObjectType::List(vec![]));
    let result = op_str_join(sep, items).unwrap();
    match &*result {
        ObjectType::String(text) => assert_eq!(text, ""),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_op_str_join_joins_string_chars() {
    let sep = Rc::new(ObjectType::String("-".to_string()));
    let text = Rc::new(ObjectType::String("abc".to_string()));
    let result = op_str_join(sep, text).unwrap();
    match &*result {
        ObjectType::String(joined) => assert_eq!(joined, "a-b-c"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_op_str_join_handles_empty_string() {
    let sep = Rc::new(ObjectType::String("-".to_string()));
    let text = Rc::new(ObjectType::String("".to_string()));
    let result = op_str_join(sep, text).unwrap();
    match &*result {
        ObjectType::String(joined) => assert_eq!(joined, ""),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_op_str_join_errors_on_list_with_non_string() {
    let sep = Rc::new(ObjectType::String(", ".to_string()));
    let items = Rc::new(ObjectType::List(vec![
        Rc::new(ObjectType::String("a".to_string())),
        Rc::new(ObjectType::Integer(42)),
    ]));
    let result = op_str_join(sep, items);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

#[test]
fn test_op_str_join_errors_on_non_string_separator() {
    let sep = Rc::new(ObjectType::Integer(42));
    let items = Rc::new(ObjectType::List(vec![Rc::new(ObjectType::String(
        "a".to_string(),
    ))]));
    let result = op_str_join(sep, items);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

#[test]
fn test_op_str_join_errors_on_invalid_iterable() {
    let sep = Rc::new(ObjectType::String(", ".to_string()));
    let items = Rc::new(ObjectType::Integer(42));
    let result = op_str_join(sep, items);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

// ============================================================================
// Value Truthiness Tests
// ============================================================================

#[test]
fn test_is_truthy_nil_is_falsy() {
    let nil = ObjectType::Nil;
    assert!(!is_truthy(&nil));
}

#[test]
fn test_is_truthy_false_is_falsy() {
    let false_val = ObjectType::Boolean(false);
    assert!(!is_truthy(&false_val));
}

#[test]
fn test_is_truthy_true_is_truthy() {
    let true_val = ObjectType::Boolean(true);
    assert!(is_truthy(&true_val));
}

#[test]
fn test_is_truthy_integer_is_truthy() {
    let zero = ObjectType::Integer(0);
    assert!(is_truthy(&zero));
    let positive = ObjectType::Integer(42);
    assert!(is_truthy(&positive));
    let negative = ObjectType::Integer(-1);
    assert!(is_truthy(&negative));
}

#[test]
fn test_is_truthy_float_is_truthy() {
    let zero = ObjectType::Float(0.0);
    assert!(is_truthy(&zero));
    let positive = ObjectType::Float(3.14);
    assert!(is_truthy(&positive));
}

#[test]
fn test_is_truthy_string_is_truthy() {
    let empty = ObjectType::String("".to_string());
    assert!(is_truthy(&empty));
    let non_empty = ObjectType::String("hello".to_string());
    assert!(is_truthy(&non_empty));
}

#[test]
fn test_is_truthy_list_is_truthy() {
    let empty = ObjectType::List(vec![]);
    assert!(is_truthy(&empty));
    let non_empty = ObjectType::List(vec![Rc::new(ObjectType::Integer(1))]);
    assert!(is_truthy(&non_empty));
}

// ============================================================================
// Native Function Tests
// ============================================================================

#[test]
fn test_native_super_errors_with_no_args() {
    let result = native_super(&[], None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "super() requires access to self");
}

#[test]
fn test_native_super_errors_with_too_many_args() {
    let instance = Rc::new(ObjectType::Integer(1));
    let extra = Rc::new(ObjectType::Integer(2));
    let result = native_super(&[instance, extra], None);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "super() takes at most 1 argument (self)"
    );
}

#[test]
fn test_native_super_errors_without_class_context() {
    let instance = Rc::new(ObjectType::Integer(1));
    let result = native_super(&[instance], None);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "super() can only be called inside a method"
    );
}

#[test]
fn test_native_super_errors_with_no_parent_class() {
    let class = Rc::new(ClassObject::new("Child".to_string(), HashMap::new()));
    let instance = Rc::new(ObjectType::Integer(1));
    let result = native_super(&[instance], Some(class));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "super() called in class with no parent"
    );
}

#[test]
fn test_native_super_returns_super_proxy_with_parent() {
    let parent = Rc::new(ClassObject::new("Parent".to_string(), HashMap::new()));
    let child = Rc::new(ClassObject::new_with_parent(
        "Child".to_string(),
        HashMap::new(),
        parent.clone(),
    ));
    let instance = Rc::new(ObjectType::Integer(42));
    let result = native_super(&[instance.clone()], Some(child));
    assert!(result.is_ok());
    match &*result.unwrap() {
        ObjectType::SuperProxy(inst, parent_class) => {
            assert!(Rc::ptr_eq(inst, &instance));
            assert_eq!(parent_class.name, "Parent");
        }
        _ => panic!("Expected SuperProxy"),
    }
}
