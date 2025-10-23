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
