use oxython::object::{ClassObject, ObjectType};
use oxython::vm::collections::{collect_iterable, slice_indices};
use oxython::vm::native::native_super;
use oxython::vm::opcodes::builtins::{op_round, op_to_list, op_zip};
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

// ============================================================================
// Builtin Operations Tests
// ============================================================================

#[test]
fn test_op_round_rounds_float_to_decimal_places() {
    let value = Rc::new(ObjectType::Float(3.14159));
    let digits = Rc::new(ObjectType::Integer(2));
    let result = op_round(value, digits).unwrap();
    match &*result {
        ObjectType::Float(v) => assert!((v - 3.14).abs() < 1e-6),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_op_round_rounds_integer_to_float() {
    let value = Rc::new(ObjectType::Integer(42));
    let digits = Rc::new(ObjectType::Integer(2));
    let result = op_round(value, digits).unwrap();
    match &*result {
        ObjectType::Float(v) => assert!((v - 42.0).abs() < 1e-6),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_op_round_with_zero_digits() {
    let value = Rc::new(ObjectType::Float(3.7));
    let digits = Rc::new(ObjectType::Integer(0));
    let result = op_round(value, digits).unwrap();
    match &*result {
        ObjectType::Float(v) => assert!((v - 4.0).abs() < 1e-6),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_op_round_with_negative_digits_uses_zero() {
    let value = Rc::new(ObjectType::Float(123.456));
    let digits = Rc::new(ObjectType::Integer(-2));
    let result = op_round(value, digits).unwrap();
    match &*result {
        // negative digits are clamped to 0 via max(0), so factor = 10^0 = 1
        // This rounds to nearest integer
        ObjectType::Float(v) => assert!((v - 123.0).abs() < 1e-6),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_op_round_errors_on_non_numeric_value() {
    let value = Rc::new(ObjectType::String("test".to_string()));
    let digits = Rc::new(ObjectType::Integer(2));
    let result = op_round(value, digits);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

#[test]
fn test_op_round_errors_on_non_integer_digits() {
    let value = Rc::new(ObjectType::Float(3.14));
    let digits = Rc::new(ObjectType::String("two".to_string()));
    let result = op_round(value, digits);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

#[test]
fn test_op_to_list_converts_list_to_list() {
    let list = Rc::new(ObjectType::List(vec![
        Rc::new(ObjectType::Integer(1)),
        Rc::new(ObjectType::Integer(2)),
    ]));
    let result = op_to_list(list).unwrap();
    match &*result {
        ObjectType::List(values) => assert_eq!(values.len(), 2),
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_op_to_list_converts_string_to_list_of_chars() {
    let string = Rc::new(ObjectType::String("abc".to_string()));
    let result = op_to_list(string).unwrap();
    match &*result {
        ObjectType::List(values) => {
            assert_eq!(values.len(), 3);
            match &*values[0] {
                ObjectType::String(s) => assert_eq!(s, "a"),
                _ => panic!("Expected string"),
            }
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_op_to_list_converts_tuple_to_list() {
    let tuple = Rc::new(ObjectType::Tuple(vec![
        Rc::new(ObjectType::Integer(1)),
        Rc::new(ObjectType::Integer(2)),
    ]));
    let result = op_to_list(tuple).unwrap();
    match &*result {
        ObjectType::List(values) => assert_eq!(values.len(), 2),
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_op_to_list_errors_on_non_iterable() {
    let integer = Rc::new(ObjectType::Integer(42));
    let result = op_to_list(integer);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

#[test]
fn test_op_zip_with_no_args_returns_empty_list() {
    let result = op_zip(vec![], 0).unwrap();
    match &*result {
        ObjectType::List(values) => assert_eq!(values.len(), 0),
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_op_zip_zips_two_lists() {
    let list1 = Rc::new(ObjectType::List(vec![
        Rc::new(ObjectType::Integer(1)),
        Rc::new(ObjectType::Integer(2)),
    ]));
    let list2 = Rc::new(ObjectType::List(vec![
        Rc::new(ObjectType::String("a".to_string())),
        Rc::new(ObjectType::String("b".to_string())),
    ]));
    let result = op_zip(vec![list1, list2], 0).unwrap();
    match &*result {
        ObjectType::List(values) => {
            assert_eq!(values.len(), 2);
            match &*values[0] {
                ObjectType::Tuple(items) => assert_eq!(items.len(), 2),
                _ => panic!("Expected tuple"),
            }
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_op_zip_stops_at_shortest_iterable() {
    let list1 = Rc::new(ObjectType::List(vec![
        Rc::new(ObjectType::Integer(1)),
        Rc::new(ObjectType::Integer(2)),
        Rc::new(ObjectType::Integer(3)),
    ]));
    let list2 = Rc::new(ObjectType::List(vec![Rc::new(ObjectType::String(
        "a".to_string(),
    ))]));
    let result = op_zip(vec![list1, list2], 0).unwrap();
    match &*result {
        ObjectType::List(values) => assert_eq!(values.len(), 1),
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_op_zip_with_star_unpacking() {
    // Create a list of lists to be unpacked
    let inner1 = Rc::new(ObjectType::List(vec![
        Rc::new(ObjectType::Integer(1)),
        Rc::new(ObjectType::Integer(2)),
    ]));
    let inner2 = Rc::new(ObjectType::List(vec![
        Rc::new(ObjectType::Integer(3)),
        Rc::new(ObjectType::Integer(4)),
    ]));
    let outer = Rc::new(ObjectType::List(vec![inner1, inner2]));

    // star_mask = 1 means unpack the first argument
    let result = op_zip(vec![outer], 1).unwrap();
    match &*result {
        ObjectType::List(values) => {
            assert_eq!(values.len(), 2); // Shortest of the two inner lists
            match &*values[0] {
                ObjectType::Tuple(items) => assert_eq!(items.len(), 2), // Two lists unpacked
                _ => panic!("Expected tuple"),
            }
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_op_zip_errors_on_non_iterable() {
    let integer = Rc::new(ObjectType::Integer(42));
    let result = op_zip(vec![integer], 0);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

#[test]
fn test_op_zip_errors_on_star_with_non_list() {
    let integer = Rc::new(ObjectType::Integer(42));
    // star_mask = 1 means try to unpack the first argument
    let result = op_zip(vec![integer], 1);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

#[test]
fn test_op_zip_errors_on_star_with_non_iterable_items() {
    let bad_list = Rc::new(ObjectType::List(vec![Rc::new(ObjectType::Integer(42))]));
    // star_mask = 1 means unpack the first argument, but it contains non-iterables
    let result = op_zip(vec![bad_list], 1);
    assert_eq!(result, Err(InterpretResult::RuntimeError));
}

// ============================================================================
// Collection Utilities Tests
// ============================================================================

#[test]
fn test_collect_iterable_from_list() {
    let list = Rc::new(ObjectType::List(vec![
        Rc::new(ObjectType::Integer(1)),
        Rc::new(ObjectType::Integer(2)),
    ]));
    let result = collect_iterable(&list);
    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn test_collect_iterable_from_tuple() {
    let tuple = Rc::new(ObjectType::Tuple(vec![
        Rc::new(ObjectType::Integer(1)),
        Rc::new(ObjectType::Integer(2)),
    ]));
    let result = collect_iterable(&tuple);
    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn test_collect_iterable_from_string() {
    let string = Rc::new(ObjectType::String("abc".to_string()));
    let result = collect_iterable(&string);
    assert!(result.is_some());
    let chars = result.unwrap();
    assert_eq!(chars.len(), 3);
    match &*chars[0] {
        ObjectType::String(s) => assert_eq!(s, "a"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_collect_iterable_from_non_iterable() {
    let integer = Rc::new(ObjectType::Integer(42));
    let result = collect_iterable(&integer);
    assert!(result.is_none());
}

#[test]
fn test_slice_indices_basic_slice() {
    let indices = slice_indices(10, Some(2), Some(5), 1).unwrap();
    assert_eq!(indices, vec![2, 3, 4]);
}

#[test]
fn test_slice_indices_full_slice() {
    let indices = slice_indices(5, None, None, 1).unwrap();
    assert_eq!(indices, vec![0, 1, 2, 3, 4]);
}

#[test]
fn test_slice_indices_with_step() {
    let indices = slice_indices(10, None, None, 2).unwrap();
    assert_eq!(indices, vec![0, 2, 4, 6, 8]);
}

#[test]
fn test_slice_indices_negative_step() {
    let indices = slice_indices(5, None, None, -1).unwrap();
    assert_eq!(indices, vec![4, 3, 2, 1, 0]);
}

#[test]
fn test_slice_indices_negative_start() {
    let indices = slice_indices(10, Some(-3), None, 1).unwrap();
    assert_eq!(indices, vec![7, 8, 9]);
}

#[test]
fn test_slice_indices_negative_end() {
    let indices = slice_indices(10, Some(2), Some(-2), 1).unwrap();
    assert_eq!(indices, vec![2, 3, 4, 5, 6, 7]);
}

#[test]
fn test_slice_indices_zero_step_returns_none() {
    let result = slice_indices(10, Some(0), Some(5), 0);
    assert!(result.is_none());
}

#[test]
fn test_slice_indices_empty_range() {
    let indices = slice_indices(10, Some(5), Some(2), 1).unwrap();
    assert_eq!(indices, vec![]); // start > end with positive step
}

#[test]
fn test_slice_indices_on_empty_sequence() {
    let indices = slice_indices(0, None, None, 1).unwrap();
    assert_eq!(indices, vec![]);
}

#[test]
fn test_slice_indices_out_of_bounds_start() {
    let indices = slice_indices(5, Some(10), None, 1).unwrap();
    assert_eq!(indices, vec![]); // clamped to length
}

#[test]
fn test_slice_indices_out_of_bounds_negative() {
    let indices = slice_indices(5, Some(-10), None, 1).unwrap();
    assert_eq!(indices, vec![0, 1, 2, 3, 4]); // clamped to 0
}

#[test]
fn test_slice_indices_negative_step_with_bounds() {
    let indices = slice_indices(10, Some(7), Some(2), -1).unwrap();
    assert_eq!(indices, vec![7, 6, 5, 4, 3]); // 7 down to 3 (exclusive of 2)
}

#[test]
fn test_slice_indices_step_skips_elements() {
    let indices = slice_indices(10, Some(1), Some(8), 3).unwrap();
    assert_eq!(indices, vec![1, 4, 7]);
}
