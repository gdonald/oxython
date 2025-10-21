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
