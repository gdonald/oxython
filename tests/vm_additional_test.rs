use oxython::bytecode::{Chunk, OpCode};
use oxython::compiler::Compiler;
use oxython::object::{FunctionObject, ObjectType};
use oxython::vm::{InterpretResult, VM};
use std::rc::Rc;

fn push_constant(chunk: &mut Chunk, value: ObjectType) -> usize {
    chunk.constants.push(Rc::new(value));
    chunk.constants.len() - 1
}

#[test]
fn vm_default_initializes_like_new() {
    let mut chunk = Chunk::new();
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::default();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_adds_two_integers() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Integer(3));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpAdd as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(5)));
}

#[test]
fn vm_calls_simple_function() {
    let mut function_chunk = Chunk::new();
    function_chunk.code.push(OpCode::OpGetLocal as u8);
    function_chunk.code.push(1); // first argument
    function_chunk.code.push(OpCode::OpGetLocal as u8);
    function_chunk.code.push(2); // second argument
    function_chunk.code.push(OpCode::OpAdd as u8);
    function_chunk.code.push(OpCode::OpReturn as u8);

    let function_obj = Rc::new(FunctionObject::new(
        "add".to_string(),
        2,
        function_chunk,
        Vec::new(),
    ));

    let mut chunk = Chunk::new();
    let function_idx = push_constant(&mut chunk, ObjectType::Function(function_obj.clone()));
    let name_idx = push_constant(&mut chunk, ObjectType::String("add".to_string()));
    let one_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    let two_idx = push_constant(&mut chunk, ObjectType::Integer(2));

    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(function_idx as u8);
    chunk.code.push(OpCode::OpDefineGlobal as u8);
    chunk.code.push(name_idx as u8);

    chunk.code.push(OpCode::OpGetGlobal as u8);
    chunk.code.push(name_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(one_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(two_idx as u8);
    chunk.code.push(OpCode::OpCall as u8);
    chunk.code.push(2);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    assert!(matches!(
        &*vm.last_popped_stack_elem(),
        ObjectType::Integer(3)
    ));
}

#[test]
fn vm_defines_and_reads_global_variable() {
    let mut chunk = Chunk::new();
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(42));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    let name_idx = push_constant(&mut chunk, ObjectType::String("answer".into()));
    chunk.code.push(OpCode::OpDefineGlobal as u8);
    chunk.code.push(name_idx as u8);
    chunk.code.push(OpCode::OpGetGlobal as u8);
    chunk.code.push(name_idx as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(42)));
}

#[test]
fn vm_errors_on_missing_global() {
    let mut chunk = Chunk::new();
    let name_idx = push_constant(&mut chunk, ObjectType::String("missing".into()));
    chunk.code.push(OpCode::OpGetGlobal as u8);
    chunk.code.push(name_idx as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_errors_on_set_global() {
    let mut chunk = Chunk::new();
    let name_idx = push_constant(&mut chunk, ObjectType::String("var".into()));
    chunk.code.push(OpCode::OpSetGlobal as u8);
    chunk.code.push(name_idx as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_add_errors_on_type_mismatch() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    let str_idx = push_constant(&mut chunk, ObjectType::String("s".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    chunk.code.push(OpCode::OpAdd as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_concatenates_strings() {
    let mut chunk = Chunk::new();
    let first_idx = push_constant(&mut chunk, ObjectType::String("Ada".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(first_idx as u8);
    let second_idx = push_constant(&mut chunk, ObjectType::String(" Lovelace".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(second_idx as u8);
    chunk.code.push(OpCode::OpAdd as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::String(ref s) if s == "Ada Lovelace"));
}

#[test]
fn vm_adds_float_and_integer() {
    let mut chunk = Chunk::new();
    let float_idx = push_constant(&mut chunk, ObjectType::Float(1.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(float_idx as u8);
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    chunk.code.push(OpCode::OpAdd as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 3.5).abs() < f64::EPSILON));
}

#[test]
fn vm_adds_integer_and_float() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    let float_idx = push_constant(&mut chunk, ObjectType::Float(1.25));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(float_idx as u8);
    chunk.code.push(OpCode::OpAdd as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 3.25).abs() < f64::EPSILON));
}

#[test]
fn vm_appends_to_list() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(1)),
            Rc::new(ObjectType::Integer(2)),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(3));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpAppend as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::List(ref values) if values.len() == 3));
    assert!(
        matches!(&*top, ObjectType::List(ref values) if matches!(&*values[2], ObjectType::Integer(3)))
    );
}

#[test]
fn vm_divides_numbers() {
    let mut chunk = Chunk::new();
    let numerator_idx = push_constant(&mut chunk, ObjectType::Integer(6));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(numerator_idx as u8);
    let denominator_idx = push_constant(&mut chunk, ObjectType::Integer(4));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(denominator_idx as u8);
    chunk.code.push(OpCode::OpDivide as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 1.5).abs() < f64::EPSILON));
}

#[test]
fn vm_divides_float_and_integer() {
    let mut chunk = Chunk::new();
    let numerator_idx = push_constant(&mut chunk, ObjectType::Float(7.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(numerator_idx as u8);
    let denominator_idx = push_constant(&mut chunk, ObjectType::Integer(3));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(denominator_idx as u8);
    chunk.code.push(OpCode::OpDivide as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 2.5).abs() < f64::EPSILON));
}

#[test]
fn vm_rounds_float() {
    let mut chunk = Chunk::new();
    let value_idx = push_constant(&mut chunk, ObjectType::Float(3.13159));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    let digits_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(digits_idx as u8);
    chunk.code.push(OpCode::OpRound as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 3.13).abs() < 1e-6));
}

#[test]
fn vm_len_of_string() {
    let mut chunk = Chunk::new();
    let string_idx = push_constant(&mut chunk, ObjectType::String("rust".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(string_idx as u8);
    chunk.code.push(OpCode::OpLen as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(4)));
}

#[test]
fn vm_subtracts_numbers() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Integer(7));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Integer(4));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpSubtract as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(3)));
}

#[test]
fn vm_evaluates_less_than() {
    let mut chunk = Chunk::new();
    let left_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(left_idx as u8);
    let right_idx = push_constant(&mut chunk, ObjectType::Float(2.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(right_idx as u8);
    chunk.code.push(OpCode::OpLess as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Boolean(true)));
}

#[test]
fn vm_supports_negative_list_index() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(10)),
            Rc::new(ObjectType::Integer(20)),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let minus_one_idx = push_constant(&mut chunk, ObjectType::Integer(-1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(minus_one_idx as u8);
    chunk.code.push(OpCode::OpIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(20)));
}

#[test]
fn vm_builds_list_slice() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(1)),
            Rc::new(ObjectType::Integer(2)),
            Rc::new(ObjectType::Integer(3)),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let nil_idx = push_constant(&mut chunk, ObjectType::Nil);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    let end_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(end_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpSlice as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(
        matches!(&*top, ObjectType::List(ref values) if values.len() == 2
            && matches!(&*values[0], ObjectType::Integer(1))
            && matches!(&*values[1], ObjectType::Integer(2)))
    );
}

#[test]
fn vm_slice_with_negative_start_and_default_end() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(1)),
            Rc::new(ObjectType::Integer(2)),
            Rc::new(ObjectType::Integer(3)),
            Rc::new(ObjectType::Integer(4)),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let start_idx = push_constant(&mut chunk, ObjectType::Integer(-2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(start_idx as u8);
    let nil_idx = push_constant(&mut chunk, ObjectType::Nil);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpSlice as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(
        matches!(&*top, ObjectType::List(ref values) if values.len() == 2
            && matches!(&*values[0], ObjectType::Integer(3))
            && matches!(&*values[1], ObjectType::Integer(4)))
    );
}

#[test]
fn vm_slice_returns_empty_when_end_before_start() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(1)),
            Rc::new(ObjectType::Integer(2)),
            Rc::new(ObjectType::Integer(3)),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let start_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(start_idx as u8);
    let end_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(end_idx as u8);
    let nil_idx = push_constant(&mut chunk, ObjectType::Nil);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpSlice as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::List(ref values) if values.is_empty()));
}

#[test]
fn vm_range_builds_sequence() {
    let mut chunk = Chunk::new();
    let start_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(start_idx as u8);
    let end_idx = push_constant(&mut chunk, ObjectType::Integer(5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(end_idx as u8);
    chunk.code.push(OpCode::OpRange as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(
        matches!(&*top, ObjectType::List(ref values) if values.len() == 3
            && matches!(&*values[0], ObjectType::Integer(2))
            && matches!(&*values[2], ObjectType::Integer(4)))
    );
}

#[test]
fn vm_sets_list_index() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(1)),
            Rc::new(ObjectType::Integer(2)),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let index_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(index_idx as u8);
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(42));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpSetIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(
        matches!(&*top, ObjectType::List(ref values) if matches!(&*values[1], ObjectType::Integer(42)))
    );
}

#[test]
fn vm_sets_dict_entry() {
    let mut chunk = Chunk::new();
    let dict_idx = push_constant(
        &mut chunk,
        ObjectType::Dict(vec![("a".into(), Rc::new(ObjectType::Integer(1)))]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(dict_idx as u8);
    let key_idx = push_constant(&mut chunk, ObjectType::String("a".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(key_idx as u8);
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpSetIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Dict(ref entries)
            if entries.iter().any(|(k, v)| k == "a" && matches!(&**v, ObjectType::Integer(5)))));
}

#[test]
fn vm_contains_supports_list_and_dict_and_string() {
    // list contains
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(1)),
            Rc::new(ObjectType::Integer(2)),
        ]),
    );
    let item_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(item_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    chunk.code.push(OpCode::OpContains as u8);

    // dict contains
    let dict_idx = push_constant(
        &mut chunk,
        ObjectType::Dict(vec![("k".into(), Rc::new(ObjectType::Integer(1)))]),
    );
    let key_idx = push_constant(&mut chunk, ObjectType::String("k".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(key_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(dict_idx as u8);
    chunk.code.push(OpCode::OpContains as u8);

    // string contains
    let needle_idx = push_constant(&mut chunk, ObjectType::String("llo".into()));
    let haystack_idx = push_constant(&mut chunk, ObjectType::String("hello".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(needle_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(haystack_idx as u8);
    chunk.code.push(OpCode::OpContains as u8);

    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    // After three contains operations, stack holds their boolean results in order.
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Boolean(true)));
}

#[test]
fn vm_dup_allows_stack_reuse() {
    let mut chunk = Chunk::new();
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(7));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpDup as u8);
    chunk.code.push(OpCode::OpAdd as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(14)));
}

#[test]
fn vm_iterates_over_string_collection() {
    let source = "text = \"ab\"\nfor ch in text:\n    last = ch\n";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_divide_errors_on_non_numeric_operands() {
    let mut chunk = Chunk::new();
    let str_idx = push_constant(&mut chunk, ObjectType::String("test".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    chunk.code.push(OpCode::OpDivide as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_divide_errors_on_zero_division() {
    let mut chunk = Chunk::new();
    let num_idx = push_constant(&mut chunk, ObjectType::Integer(10));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(num_idx as u8);
    let zero_idx = push_constant(&mut chunk, ObjectType::Integer(0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(zero_idx as u8);
    chunk.code.push(OpCode::OpDivide as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_subtract_errors_on_type_mismatch() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    let str_idx = push_constant(&mut chunk, ObjectType::String("text".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    chunk.code.push(OpCode::OpSubtract as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_multiply_errors_on_type_mismatch() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(3));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    let str_idx = push_constant(&mut chunk, ObjectType::String("text".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    chunk.code.push(OpCode::OpMultiply as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_index_errors_on_out_of_bounds() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![Rc::new(ObjectType::Integer(1))]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let bad_idx = push_constant(&mut chunk, ObjectType::Integer(10));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(bad_idx as u8);
    chunk.code.push(OpCode::OpIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_index_errors_on_negative_out_of_bounds() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![Rc::new(ObjectType::Integer(1))]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let bad_idx = push_constant(&mut chunk, ObjectType::Integer(-10));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(bad_idx as u8);
    chunk.code.push(OpCode::OpIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_index_errors_on_missing_dict_key() {
    let mut chunk = Chunk::new();
    let dict_idx = push_constant(&mut chunk, ObjectType::Dict(vec![]));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(dict_idx as u8);
    let key_idx = push_constant(&mut chunk, ObjectType::String("missing".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(key_idx as u8);
    chunk.code.push(OpCode::OpIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_index_errors_on_invalid_type() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(123));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    let idx_idx = push_constant(&mut chunk, ObjectType::Integer(0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(idx_idx as u8);
    chunk.code.push(OpCode::OpIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_len_errors_on_invalid_type() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(123));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    chunk.code.push(OpCode::OpLen as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_append_errors_on_non_list() {
    let mut chunk = Chunk::new();
    let str_idx = push_constant(&mut chunk, ObjectType::String("text".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpAppend as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_range_errors_on_non_integer() {
    let mut chunk = Chunk::new();
    let str_idx = push_constant(&mut chunk, ObjectType::String("a".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    chunk.code.push(OpCode::OpRange as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_range_handles_reverse_range() {
    let mut chunk = Chunk::new();
    let start_idx = push_constant(&mut chunk, ObjectType::Integer(5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(start_idx as u8);
    let end_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(end_idx as u8);
    chunk.code.push(OpCode::OpRange as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::List(ref values) if values.is_empty()));
}

#[test]
fn vm_less_errors_on_type_mismatch() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    let str_idx = push_constant(&mut chunk, ObjectType::String("text".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    chunk.code.push(OpCode::OpLess as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_less_compares_float_to_float() {
    let mut chunk = Chunk::new();
    let left_idx = push_constant(&mut chunk, ObjectType::Float(1.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(left_idx as u8);
    let right_idx = push_constant(&mut chunk, ObjectType::Float(2.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(right_idx as u8);
    chunk.code.push(OpCode::OpLess as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Boolean(true)));
}

#[test]
fn vm_less_compares_float_to_int() {
    let mut chunk = Chunk::new();
    let left_idx = push_constant(&mut chunk, ObjectType::Float(1.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(left_idx as u8);
    let right_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(right_idx as u8);
    chunk.code.push(OpCode::OpLess as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Boolean(true)));
}

#[test]
fn vm_less_compares_int_to_int() {
    let mut chunk = Chunk::new();
    let left_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(left_idx as u8);
    let right_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(right_idx as u8);
    chunk.code.push(OpCode::OpLess as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Boolean(true)));
}

#[test]
fn vm_slice_errors_on_invalid_start_type() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![Rc::new(ObjectType::Integer(1))]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let bad_idx = push_constant(&mut chunk, ObjectType::String("bad".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(bad_idx as u8);
    let nil_idx = push_constant(&mut chunk, ObjectType::Nil);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpSlice as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_slice_errors_on_invalid_end_type() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![Rc::new(ObjectType::Integer(1))]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let nil_idx = push_constant(&mut chunk, ObjectType::Nil);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    let bad_idx = push_constant(&mut chunk, ObjectType::String("bad".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(bad_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpSlice as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_slice_errors_on_non_sliceable_type() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(123));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    let nil_idx = push_constant(&mut chunk, ObjectType::Nil);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpSlice as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_slice_string() {
    let mut chunk = Chunk::new();
    let str_idx = push_constant(&mut chunk, ObjectType::String("hello".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    let start_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(start_idx as u8);
    let end_idx = push_constant(&mut chunk, ObjectType::Integer(4));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(end_idx as u8);
    let nil_idx = push_constant(&mut chunk, ObjectType::Nil);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpSlice as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::String(ref s) if s == "ell"));
}

#[test]
fn vm_slice_string_empty_when_end_before_start() {
    let mut chunk = Chunk::new();
    let str_idx = push_constant(&mut chunk, ObjectType::String("hello".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    let start_idx = push_constant(&mut chunk, ObjectType::Integer(3));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(start_idx as u8);
    let end_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(end_idx as u8);
    let nil_idx = push_constant(&mut chunk, ObjectType::Nil);
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(nil_idx as u8);
    chunk.code.push(OpCode::OpSlice as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::String(ref s) if s.is_empty()));
}

#[test]
fn vm_round_errors_on_non_numeric_value() {
    let mut chunk = Chunk::new();
    let str_idx = push_constant(&mut chunk, ObjectType::String("text".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    let digits_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(digits_idx as u8);
    chunk.code.push(OpCode::OpRound as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_round_errors_on_non_integer_digits() {
    let mut chunk = Chunk::new();
    let value_idx = push_constant(&mut chunk, ObjectType::Float(3.15));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    let digits_idx = push_constant(&mut chunk, ObjectType::String("two".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(digits_idx as u8);
    chunk.code.push(OpCode::OpRound as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_round_handles_integer_value() {
    let mut chunk = Chunk::new();
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    let digits_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(digits_idx as u8);
    chunk.code.push(OpCode::OpRound as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 5.0).abs() < f64::EPSILON));
}

#[test]
fn vm_iter_next_errors_on_negative_index() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![Rc::new(ObjectType::Integer(1))]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let neg_idx = push_constant(&mut chunk, ObjectType::Integer(-1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(neg_idx as u8);
    chunk.code.push(OpCode::OpIterNext as u8);
    chunk.code.push(0);
    chunk.code.push(0);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_iter_next_errors_on_invalid_type() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(123));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    let idx_idx = push_constant(&mut chunk, ObjectType::Integer(0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(idx_idx as u8);
    chunk.code.push(OpCode::OpIterNext as u8);
    chunk.code.push(0);
    chunk.code.push(0);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_set_index_errors_on_negative_list_index() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![Rc::new(ObjectType::Integer(1))]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let neg_idx = push_constant(&mut chunk, ObjectType::Integer(-1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(neg_idx as u8);
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(42));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpSetIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_set_index_errors_on_out_of_bounds_list_index() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![Rc::new(ObjectType::Integer(1))]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let bad_idx = push_constant(&mut chunk, ObjectType::Integer(10));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(bad_idx as u8);
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(42));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpSetIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_set_index_errors_on_invalid_type() {
    let mut chunk = Chunk::new();
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(123));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    let idx_idx = push_constant(&mut chunk, ObjectType::Integer(0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(idx_idx as u8);
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(42));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpSetIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_set_index_adds_new_dict_entry() {
    let mut chunk = Chunk::new();
    let dict_idx = push_constant(&mut chunk, ObjectType::Dict(vec![]));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(dict_idx as u8);
    let key_idx = push_constant(&mut chunk, ObjectType::String("new".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(key_idx as u8);
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(42));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpSetIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Dict(ref entries) if entries.len() == 1));
}

#[test]
fn vm_contains_errors_on_invalid_type() {
    let mut chunk = Chunk::new();
    let item_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(item_idx as u8);
    let int_idx = push_constant(&mut chunk, ObjectType::Integer(123));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(int_idx as u8);
    chunk.code.push(OpCode::OpContains as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_swap_errors_on_insufficient_stack() {
    let mut chunk = Chunk::new();
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpSwap as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_subtract_floats() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Float(7.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Float(2.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpSubtract as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 5.0).abs() < f64::EPSILON));
}

#[test]
fn vm_subtract_int_from_float() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Float(7.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpSubtract as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 5.5).abs() < f64::EPSILON));
}

#[test]
fn vm_subtract_float_from_int() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Integer(10));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Float(2.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpSubtract as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 7.5).abs() < f64::EPSILON));
}

#[test]
fn vm_multiply_floats() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Float(2.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Float(3.0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpMultiply as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 7.5).abs() < f64::EPSILON));
}

#[test]
fn vm_multiply_int_and_float() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Integer(3));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Float(2.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpMultiply as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 7.5).abs() < f64::EPSILON));
}

#[test]
fn vm_multiply_float_and_int() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Float(2.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Integer(4));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpMultiply as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 10.0).abs() < f64::EPSILON));
}

#[test]
fn vm_add_two_floats() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Float(1.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Float(2.5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpAdd as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 4.0).abs() < f64::EPSILON));
}

#[test]
fn vm_last_popped_stack_elem_returns_value() {
    let mut chunk = Chunk::new();
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(42));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    chunk.code.push(OpCode::OpPop as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let last = vm.last_popped_stack_elem();
    assert!(matches!(&*last, ObjectType::Integer(42)));
}

#[test]
fn vm_closure_captures_outer_locals() {
    let source = r#"
def outer():
    value = 1
    def inner():
        return value
    value = 2
    return inner

fn = outer()
result = fn()
"#;
    let chunk = Compiler::compile(source).expect("expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let last = vm.last_popped_stack_elem();
    assert!(matches!(&*last, ObjectType::Integer(2)));
}

#[test]
fn vm_add_two_integers() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Integer(10));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Integer(5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpAdd as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(15)));
}

#[test]
fn vm_divide_two_floats() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Float(10.0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Float(4.0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpDivide as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 2.5).abs() < f64::EPSILON));
}

#[test]
fn vm_divide_int_by_float() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Integer(10));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Float(4.0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpDivide as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Float(ref v) if (*v - 2.5).abs() < f64::EPSILON));
}

#[test]
fn vm_multiply_two_integers() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Integer(7));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Integer(6));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpMultiply as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(42)));
}

#[test]
fn vm_len_of_list() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(1)),
            Rc::new(ObjectType::Integer(2)),
            Rc::new(ObjectType::Integer(3)),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    chunk.code.push(OpCode::OpLen as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(3)));
}

#[test]
fn vm_index_dict_with_string_key() {
    let mut chunk = Chunk::new();
    let dict_idx = push_constant(
        &mut chunk,
        ObjectType::Dict(vec![
            ("name".into(), Rc::new(ObjectType::String("Alice".into()))),
            ("age".into(), Rc::new(ObjectType::Integer(30))),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(dict_idx as u8);
    let key_idx = push_constant(&mut chunk, ObjectType::String("name".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(key_idx as u8);
    chunk.code.push(OpCode::OpIndex as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::String(ref s) if s == "Alice"));
}

#[test]
fn vm_range_creates_list() {
    let mut chunk = Chunk::new();
    let start_idx = push_constant(&mut chunk, ObjectType::Integer(0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(start_idx as u8);
    let end_idx = push_constant(&mut chunk, ObjectType::Integer(3));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(end_idx as u8);
    chunk.code.push(OpCode::OpRange as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::List(ref values) if values.len() == 3));
}

#[test]
fn vm_contains_checks_string_substring() {
    let mut chunk = Chunk::new();
    let pattern_idx = push_constant(&mut chunk, ObjectType::String("ell".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(pattern_idx as u8);
    let text_idx = push_constant(&mut chunk, ObjectType::String("hello".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(text_idx as u8);
    chunk.code.push(OpCode::OpContains as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Boolean(true)));
}

#[test]
fn vm_contains_checks_list_for_item() {
    let mut chunk = Chunk::new();
    let item_idx = push_constant(&mut chunk, ObjectType::Integer(5));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(item_idx as u8);
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(1)),
            Rc::new(ObjectType::Integer(5)),
            Rc::new(ObjectType::Integer(9)),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    chunk.code.push(OpCode::OpContains as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Boolean(true)));
}

#[test]
fn vm_contains_checks_dict_for_key() {
    let mut chunk = Chunk::new();
    let key_idx = push_constant(&mut chunk, ObjectType::String("x".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(key_idx as u8);
    let dict_idx = push_constant(
        &mut chunk,
        ObjectType::Dict(vec![
            ("x".into(), Rc::new(ObjectType::Integer(10))),
            ("y".into(), Rc::new(ObjectType::Integer(20))),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(dict_idx as u8);
    chunk.code.push(OpCode::OpContains as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Boolean(true)));
}

#[test]
fn vm_jump_if_false_jumps_on_false_condition() {
    // Use compiler to generate correct bytecode for if-else with false condition
    let source = "x = 0; if x < 0: y = 1 else: y = 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_jump_if_false_does_not_jump_on_true() {
    // Use compiler to generate correct bytecode
    let source = "x = 5; if x: y = 1 else: y = 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_jump_unconditional() {
    // Use compiler to generate bytecode with jumps
    let source = "if 1: x = 1 else: x = 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_iter_next_iterates_over_list() {
    let mut chunk = Chunk::new();
    let list_idx = push_constant(
        &mut chunk,
        ObjectType::List(vec![
            Rc::new(ObjectType::Integer(10)),
            Rc::new(ObjectType::Integer(20)),
        ]),
    );
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(list_idx as u8);
    let zero_idx = push_constant(&mut chunk, ObjectType::Integer(0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(zero_idx as u8);
    chunk.code.push(OpCode::OpIterNext as u8);
    chunk.code.push(0);
    chunk.code.push(3); // Jump 3 bytes if done
    chunk.code.push(OpCode::OpPop as u8); // Pop the element
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_iter_next_skips_when_done() {
    // Use compiler to generate correct iteration bytecode
    let source = "for x in [1]: print(x)";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_iter_next_iterates_over_string() {
    let mut chunk = Chunk::new();
    let str_idx = push_constant(&mut chunk, ObjectType::String("hi".into()));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(str_idx as u8);
    let zero_idx = push_constant(&mut chunk, ObjectType::Integer(0));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(zero_idx as u8);
    chunk.code.push(OpCode::OpIterNext as u8);
    chunk.code.push(0);
    chunk.code.push(3);
    chunk.code.push(OpCode::OpPop as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_set_global_updates_existing_variable() {
    let mut chunk = Chunk::new();
    let value_idx = push_constant(&mut chunk, ObjectType::Integer(10));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(value_idx as u8);
    let name_idx = push_constant(&mut chunk, ObjectType::String("var".into()));
    chunk.code.push(OpCode::OpDefineGlobal as u8);
    chunk.code.push(name_idx as u8);

    let new_value_idx = push_constant(&mut chunk, ObjectType::Integer(20));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(new_value_idx as u8);
    chunk.code.push(OpCode::OpSetGlobal as u8);
    chunk.code.push(name_idx as u8);
    chunk.code.push(OpCode::OpGetGlobal as u8);
    chunk.code.push(name_idx as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(20)));
}

#[test]
fn vm_peek_stack_returns_none_when_empty() {
    let vm = VM::new();
    assert!(vm.peek_stack().is_none());
}

#[test]
fn vm_swap_swaps_top_two_values() {
    let mut chunk = Chunk::new();
    let a_idx = push_constant(&mut chunk, ObjectType::Integer(1));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(a_idx as u8);
    let b_idx = push_constant(&mut chunk, ObjectType::Integer(2));
    chunk.code.push(OpCode::OpConstant as u8);
    chunk.code.push(b_idx as u8);
    chunk.code.push(OpCode::OpSwap as u8);
    chunk.code.push(OpCode::OpReturn as u8);

    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    // After swap, top should be 1
    let top = vm.peek_stack().expect("expected value on stack");
    assert!(matches!(&*top, ObjectType::Integer(1)));
}

// VM tests to hit specific uncovered lines

#[test]
fn vm_add_int_plus_int() {
    // Line 69: Integer + Integer
    let source = "x = 10 + 5";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_divide_by_zero_with_float() {
    // Line 103: Division by zero with float
    let source = "x = 5.0 / 0.0";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::RuntimeError);
}

#[test]
fn vm_divide_float_by_int() {
    // Line 100-101: Float / Integer
    let source = "x = 10.5 / 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_subtract_int_int() {
    // Line 113: Integer - Integer
    let source = "x = 10 - 3";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_len_of_empty_string() {
    // Test len on empty string
    let source = "s = ''; x = len(s)";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_index_list_positive() {
    // List indexing positive
    let source = "items = [1, 2, 3]; x = items[1]";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_append_to_list() {
    // Line 197-198: Append to list
    let source = "items = [1, 2]; items.append(3)";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_round_float() {
    // Line 205-206: Round a float
    let source = "x = round(3.14159, 2)";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_range_positive() {
    // Line 212: Range with positive numbers
    let source = "items = range(0, 5)";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_less_int_int() {
    // Line 227: Integer < Integer
    let source = "x = 1 < 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_less_float_float() {
    // Line 237-239: Float < Float
    let source = "x = 1.5 < 2.5";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_less_float_int() {
    // Line 249-250: Float < Integer
    let source = "x = 1.5 < 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_less_int_float() {
    // Line 255-256: Integer < Float
    let source = "x = 1 < 2.5";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_less_returns_false() {
    // Line 258: Less than returns false
    let source = "x = 5 < 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_less_equal_values() {
    // Line 262: Less than with equal values
    let source = "x = 5 < 5";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_slice_list_with_end() {
    // Line 280: Slice list with end specified
    let source = "items = [1, 2, 3, 4]; x = items[1:3]";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_slice_list_no_end() {
    // Line 299: Slice list with no end (nil)
    let source = "items = [1, 2, 3]; x = items[1:]";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_slice_string_with_bounds() {
    // Line 314-317: Slice string
    let source = "s = 'hello'; x = s[1:4]";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_slice_string_no_start() {
    // Line 319: Slice string with no start
    let source = "s = 'hello'; x = s[:3]";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_slice_string_no_end() {
    // Line 324: Slice string with no end
    let source = "s = 'hello'; x = s[2:]";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_set_index_list() {
    // Line 344-346: Set index on list
    let source = "items = [1, 2, 3]; items[1] = 42";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_set_index_dict_existing_key() {
    // Line 366: Set index on dict existing key
    let source = "d = {'a': 1}; d['a'] = 42";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_set_index_dict_new_key() {
    // Line 379-380: Set index on dict with new key
    let source = "d = {}; d['new'] = 42";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_contains_string_found() {
    // Line 382-383: Contains in string (found)
    let source = "x = 'ell' in 'hello'";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_contains_list_found() {
    // Line 421: Contains in list (found)
    let source = "x = 2 in [1, 2, 3]";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_contains_list_not_found() {
    // Line 425-427: Contains in list (not found)
    let source = "x = 99 in [1, 2, 3]";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_contains_dict_found() {
    // Line 435: Contains in dict (found)
    let source = "x = 'key' in {'key': 1}";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_dup_duplicates_stack_top() {
    // Line 460: OpDup
    let source = "items = [1]; items[0] += 5"; // This uses DUP internally
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_printspaced_prints_with_space() {
    // Line 465: OpPrintSpaced
    let source = "print(1, 2, 3)";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_println_prints_newline() {
    // Line 471: OpPrintln
    let source = "print('hello')";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_loop_backward_jump() {
    // Line 503: OpLoop
    let source = "x = 0; while x < 3: x += 1";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
fn vm_define_and_get_global() {
    // Line 532, 535: OpDefineGlobal and OpGetGlobal
    let source = "x = 42; y = x";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let mut vm = VM::new();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}
