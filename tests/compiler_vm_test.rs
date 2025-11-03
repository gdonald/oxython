use oxython::compiler::Compiler;
use oxython::object::ObjectType;
use oxython::vm::{InterpretResult, VM};
use std::rc::Rc;

fn run_code(source: &str) -> (InterpretResult, Rc<ObjectType>) {
    let chunk = Compiler::compile(source).expect("Compilation failed");
    let mut vm = VM::new();
    let result = vm.interpret(chunk.clone());

    // If the last instruction was not a print/return, the result of the last expression
    // is on top of the stack. Otherwise, we check the last popped element.
    let last_popped = vm
        .peek_stack()
        .unwrap_or_else(|| vm.last_popped_stack_elem());
    (result, last_popped)
}

#[test]
fn test_integer_arithmetic() {
    let source = "1 + 2";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(3));
}

#[test]
fn test_global_variable_definition_and_retrieval() {
    let source = "a = 10; a";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(10));
}

#[test]
fn test_type_mismatch_runtime_error() {
    let source = "1 + 'hello'";
    let (result, _) = run_code(source);
    assert_eq!(result, InterpretResult::RuntimeError);
}

#[test]
fn test_function_definition_and_call() {
    let source = "
def add(a, b):
    return a + b

add(2, 5)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(7));
}

#[test]
fn test_recursive_function_call_frames() {
    let source = "
def factorial(n):
    if n < 2:
        return 1
    return n * factorial(n - 1)

factorial(5)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(120));
}

#[test]
fn test_function_invocation_chain() {
    let source = "
def double(n):
    return n * 2

def apply_twice(value):
    return double(double(value))

apply_twice(3)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(12));
}

#[test]
fn test_function_return_without_value_pushes_nil() {
    let source = "
def noop():
    return

noop()
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Nil);
}

#[test]
fn test_function_with_local_variable() {
    let source = "
def compute(a):
    b = a + 1
    return b

compute(4)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(5));
}

#[test]
fn test_function_uses_globals_without_leaking_locals() {
    let source = "
value = 10

def use_locals(x):
    temp = x + value
    temp = temp * 2
    return temp

value = value + 5
use_locals(5)
value
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(15));
}

#[test]
fn test_function_for_loop_local_variable() {
    let source = "
def sum_range(n):
    total = 0
    for i in range(0, n):
        total = total + i
    return total

sum_range(5)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(10));
}

#[test]
fn test_function_list_comprehension_local_scope() {
    let source = "
def build_doubles(n):
    return [i * 2 for i in range(0, n)]

build_doubles(4)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    match &*last_popped {
        ObjectType::List(values) => {
            let ints: Vec<i64> = values
                .iter()
                .map(|value| match &**value {
                    ObjectType::Integer(v) => *v,
                    other => panic!("expected integer in list, got {:?}", other),
                })
                .collect();
            assert_eq!(ints, vec![0, 2, 4, 6]);
        }
        other => panic!("expected list result, got {:?}", other),
    }
}

#[test]
fn test_variable_type_annotation() {
    let source = "
x: int = 42
y: str = 'hello'
z: float = 3.14
x + 1
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(43));
}

#[test]
fn test_function_parameter_type_annotations() {
    let source = "
def greet(name: str, age: int) -> str:
    return 'hello'

greet('Alice', 30)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("hello".to_string()));
}

#[test]
fn test_function_return_type_annotation() {
    let source = "
def add(a: int, b: int) -> int:
    return a + b

add(10, 20)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(30));
}

#[test]
fn test_mixed_annotated_and_unannotated_vars() {
    let source = "
x: int = 5
y = 10
z: float = 2.5
x + y
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(15));
}

#[test]
fn test_local_variable_type_annotation() {
    let source = "
def test():
    x: int = 100
    y: str = 'test'
    return x + 50

test()
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(150));
}

#[test]
fn test_global_variable_type_annotations() {
    // Test that global variables with type annotations compile and execute correctly
    let source = "
x: int = 10
y: str = 'hello'
z: float = 3.14
result: int = x + 5
result
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(15));
}

#[test]
fn test_mixed_global_local_type_annotations() {
    // Test mix of global and local type annotations
    let source = "
global_var: int = 100

def test():
    local_var: int = 50
    return global_var + local_var

test()
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(150));
}

#[test]
fn test_reassignment_with_type_annotation() {
    // Test that we can reassign a variable that was initially declared with a type annotation
    let source = "
x: int = 10
x = 20
x = x + 5
x
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(25));
}

#[test]
fn test_multiple_type_annotations_different_types() {
    // Test multiple global variables with different type annotations
    let source = "
a: int = 42
b: float = 2.5
c: str = 'test'
d: bool = True
e: list = [1, 2, 3]
len(e)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::Integer(3));
}

#[test]
fn test_type_builtin_integer() {
    let source = "type(42)";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("int".to_string()));
}

#[test]
fn test_type_builtin_float() {
    let source = "type(3.14)";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("float".to_string()));
}

#[test]
fn test_type_builtin_string() {
    let source = "type('hello')";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("str".to_string()));
}

#[test]
fn test_type_builtin_boolean() {
    let source = "type(True)";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("bool".to_string()));
}

#[test]
fn test_type_builtin_list() {
    let source = "type([1, 2, 3])";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("list".to_string()));
}

#[test]
fn test_type_builtin_dict() {
    let source = "type({'a': 1})";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("dict".to_string()));
}

#[test]
fn test_type_builtin_none() {
    let source = "
def foo():
    return
type(foo())
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("None".to_string()));
}

#[test]
fn test_type_builtin_variable() {
    let source = "
x = 42
type(x)
";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("int".to_string()));
}

#[test]
fn test_type_builtin_expression() {
    let source = "type(10 + 20)";
    let (result, last_popped) = run_code(source);
    assert_eq!(result, InterpretResult::Ok);
    assert_eq!(*last_popped, ObjectType::String("int".to_string()));
}
