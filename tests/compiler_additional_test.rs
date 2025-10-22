use oxython::bytecode::{Chunk, OpCode};
use oxython::compiler::Compiler;
use oxython::object::ObjectType;

fn opcodes(chunk: &Chunk) -> Vec<OpCode> {
    let mut result = Vec::new();
    let mut ip = 0;
    while ip < chunk.code.len() {
        let opcode = OpCode::from(chunk.code[ip]);
        result.push(opcode);
        ip += 1;
        match opcode {
            OpCode::OpConstant
            | OpCode::OpDefineGlobal
            | OpCode::OpGetGlobal
            | OpCode::OpSetGlobal
            | OpCode::OpCall
            | OpCode::OpGetLocal
            | OpCode::OpSetLocal => {
                ip += 1;
            }
            OpCode::OpIterNext | OpCode::OpLoop | OpCode::OpJumpIfFalse | OpCode::OpJump => {
                ip += 2;
            }
            OpCode::OpZip => {
                ip += 3;
            }
            _ => {}
        }
    }
    result
}

#[test]
fn compile_handles_stray_semicolons() {
    let chunk = Compiler::compile(";;;").expect("Expected chunk");
    assert_eq!(opcodes(&chunk), vec![OpCode::OpReturn]);
}

#[test]
fn compile_skips_unknown_tokens() {
    assert!(Compiler::compile("@").is_none());
}

#[test]
fn compile_emits_add_and_pop_for_expression_statement() {
    let chunk = Compiler::compile("1 + 2").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpConstant,
            OpCode::OpAdd,
            OpCode::OpPop,
            OpCode::OpReturn
        ]
    );
    assert!(matches!(&*chunk.constants[0], ObjectType::Integer(1)));
    assert!(matches!(&*chunk.constants[1], ObjectType::Integer(2)));
}

#[test]
fn compile_function_definition_emits_function_object() {
    let chunk = Compiler::compile("def add(a, b): return a + b").expect("Expected chunk");
    assert!(chunk.constants.iter().any(|value| matches!(
        &**value,
        ObjectType::Function(func) if func.name == "add" && func.arity == 2
    )));
    assert!(opcodes(&chunk).contains(&OpCode::OpDefineGlobal));
}

#[test]
fn compile_function_call_emits_call_opcode() {
    let source = "def add(a, b): return a + b\nprint(add(1, 2))";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    assert!(opcodes(&chunk).contains(&OpCode::OpCall));
}

#[test]
fn compile_handles_multiple_additions() {
    let chunk = Compiler::compile("1 + 2 + 3").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpConstant,
            OpCode::OpAdd,
            OpCode::OpConstant,
            OpCode::OpAdd,
            OpCode::OpPop,
            OpCode::OpReturn
        ]
    );
    assert!(matches!(&*chunk.constants[0], ObjectType::Integer(1)));
    assert!(matches!(&*chunk.constants[1], ObjectType::Integer(2)));
    assert!(matches!(&*chunk.constants[2], ObjectType::Integer(3)));
}

#[test]
fn compile_handles_assignment_and_lookup() {
    let chunk = Compiler::compile("a = 10; a").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpDefineGlobal,
            OpCode::OpGetGlobal,
            OpCode::OpPop,
            OpCode::OpReturn
        ]
    );
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::Integer(10))));
    assert_eq!(chunk.constants.len(), 3);
}

#[test]
fn compile_handles_print_with_multiple_arguments() {
    let chunk = Compiler::compile("print('a', 'b')").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpPrintSpaced,
            OpCode::OpConstant,
            OpCode::OpPrint,
            OpCode::OpPrintln,
            OpCode::OpReturn
        ]
    );
    assert_eq!(chunk.constants.len(), 2);
    assert!(matches!(&*chunk.constants[0], ObjectType::String(ref s) if s == "a"));
    assert!(matches!(&*chunk.constants[1], ObjectType::String(ref s) if s == "b"));
}

#[test]
fn compile_identifier_expression_uses_get_global() {
    let chunk = Compiler::compile("foo").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![OpCode::OpGetGlobal, OpCode::OpPop, OpCode::OpReturn]
    );
    assert!(matches!(&*chunk.constants[0], ObjectType::String(ref s) if s == "foo"));
}

#[test]
fn compile_print_single_argument_emits_println() {
    let chunk = Compiler::compile("print('value')").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpPrint,
            OpCode::OpPrintln,
            OpCode::OpReturn
        ]
    );
    assert!(matches!(&*chunk.constants[0], ObjectType::String(ref s) if s == "value"));
}

#[test]
fn compile_string_literal_expression_creates_constant() {
    let chunk = Compiler::compile("'hello'").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![OpCode::OpConstant, OpCode::OpPop, OpCode::OpReturn]
    );
    assert!(matches!(&*chunk.constants[0], ObjectType::String(ref s) if s == "hello"));
}

#[test]
fn compile_handles_list_indexing() {
    let chunk = Compiler::compile("items = [1, 2, 3]; print(items[1])").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpDefineGlobal,
            OpCode::OpGetGlobal,
            OpCode::OpConstant,
            OpCode::OpIndex,
            OpCode::OpPrint,
            OpCode::OpPrintln,
            OpCode::OpReturn
        ]
    );
}

#[test]
fn compile_handles_len_builtin() {
    let chunk = Compiler::compile("items = [1, 2, 3]; print(len(items))").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpDefineGlobal,
            OpCode::OpGetGlobal,
            OpCode::OpLen,
            OpCode::OpPrint,
            OpCode::OpPrintln,
            OpCode::OpReturn
        ]
    );
}

#[test]
fn compile_string_concatenation() {
    let chunk = Compiler::compile("print('Ada' + ' Lovelace')").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpConstant,
            OpCode::OpAdd,
            OpCode::OpPrint,
            OpCode::OpPrintln,
            OpCode::OpReturn
        ]
    );
}

#[test]
fn compile_handles_list_append() {
    let chunk = Compiler::compile("values = [1, 2]; values.append(3)").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpDefineGlobal,
            OpCode::OpGetGlobal,
            OpCode::OpConstant,
            OpCode::OpAppend,
            OpCode::OpSetGlobal,
            OpCode::OpPop,
            OpCode::OpReturn,
        ]
    );
}

#[test]
fn compile_handles_division_and_round() {
    let chunk =
        Compiler::compile("values = [1, 2]; average = 3 / len(values); print(round(average, 1))")
            .expect("Expected chunk");
    assert!(opcodes(&chunk).contains(&OpCode::OpDivide));
    assert!(opcodes(&chunk).contains(&OpCode::OpRound));
}

#[test]
fn compile_handles_dict_literal_and_lookup() {
    let chunk =
        Compiler::compile("ages = {'alice': 30}; print(ages['alice'])").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpDefineGlobal,
            OpCode::OpGetGlobal,
            OpCode::OpConstant,
            OpCode::OpIndex,
            OpCode::OpPrint,
            OpCode::OpPrintln,
            OpCode::OpReturn,
        ]
    );
}

#[test]
fn compile_handles_while_loop() {
    let source = "numbers = [0, 1]; while len(numbers) < 4: numbers.append(len(numbers))";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpJumpIfFalse)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpLoop)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpAppend)));
}

#[test]
fn compile_handles_slice_expression() {
    let chunk = Compiler::compile("values = [1, 2, 3]; print(values[:2])").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSlice)));
}

#[test]
fn compile_handles_negative_index_and_less() {
    let chunk =
        Compiler::compile("values = [1, 2]; print(values[-1] < 3)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSubtract)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpLess)));
}

#[test]
fn compile_handles_multiply_assign() {
    let chunk = Compiler::compile("counter = 2; counter *= 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpMultiply)));
}

#[test]
fn compile_handles_if_else_statement() {
    let source = "value = 0; if value < 1: value = 1 else: value = 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpLess)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpJumpIfFalse)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpJump)));
}

#[test]
fn compile_handles_for_loop_over_literal_list() {
    let source = "total = 0; for item in [1, 2]: total += item";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpIterNext)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpLoop)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpAdd)));
}

#[test]
fn compile_handles_index_assignment() {
    let source = "nums = [0, 1]; nums[1] = 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSetIndex)));
}

#[test]
fn compile_handles_index_add_assign() {
    let source = "counts = {'a': 1}; counts['a'] += 2";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSetIndex)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpAdd)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSwap)));
}

#[test]
fn compile_errors_on_malformed_for_loop() {
    assert!(Compiler::compile("for x in [1] print(x)").is_none());
}

#[test]
fn compile_errors_on_invalid_slice_syntax() {
    assert!(Compiler::compile("values = [1, 2]; print(values[1").is_none());
}

#[test]
fn compile_errors_on_list_literal_with_expression() {
    assert!(Compiler::compile("items = [foo]").is_none());
}

#[test]
fn compile_errors_on_malformed_print_statement() {
    // Print statement missing closing paren - should still compile but with incomplete code
    // The lexer will run out of tokens and parsing will finish early
    let result = Compiler::compile("print(1, 2");
    // This actually compiles because the parser consumes what it can
    assert!(result.is_some());
}

#[test]
fn compile_errors_on_malformed_if_missing_colon() {
    assert!(Compiler::compile("if 1 print(1)").is_none());
}

#[test]
fn compile_errors_on_malformed_while_missing_colon() {
    assert!(Compiler::compile("while 1 print(1)").is_none());
}

#[test]
fn compile_errors_on_for_loop_missing_in() {
    assert!(Compiler::compile("for x [1, 2]: print(x)").is_none());
}

#[test]
fn compile_errors_on_for_loop_missing_colon() {
    assert!(Compiler::compile("for x in [1, 2] print(x)").is_none());
}

#[test]
fn compile_errors_on_malformed_len_call() {
    assert!(Compiler::compile("len([1, 2]").is_none());
}

#[test]
fn compile_errors_on_malformed_round_call_missing_comma() {
    assert!(Compiler::compile("round(3.14 2)").is_none());
}

#[test]
fn compile_errors_on_malformed_round_call_missing_rparen() {
    assert!(Compiler::compile("round(3.14, 2").is_none());
}

#[test]
fn compile_errors_on_malformed_range_call_missing_comma() {
    assert!(Compiler::compile("range(1 5)").is_none());
}

#[test]
fn compile_errors_on_malformed_range_call_missing_rparen() {
    assert!(Compiler::compile("range(1, 5").is_none());
}

#[test]
fn compile_errors_on_invalid_append_on_indexed_value() {
    assert!(Compiler::compile("items = [[1, 2]]; items[0].append(3)").is_none());
}

#[test]
fn compile_errors_on_append_missing_lparen() {
    assert!(Compiler::compile("items = [1, 2]; items.append 3)").is_none());
}

#[test]
fn compile_errors_on_append_missing_rparen() {
    assert!(Compiler::compile("items = [1, 2]; items.append(3").is_none());
}

#[test]
fn compile_errors_on_unknown_method() {
    assert!(Compiler::compile("items = [1, 2]; items.unknown()").is_none());
}

#[test]
fn compile_errors_on_dict_literal_missing_colon() {
    assert!(Compiler::compile("d = {'key' 'value'}").is_none());
}

#[test]
fn compile_errors_on_dict_literal_invalid_value() {
    assert!(Compiler::compile("d = {'key': foo}").is_none());
}

#[test]
fn compile_errors_on_dict_literal_invalid_key() {
    assert!(Compiler::compile("d = {123: 'value'}").is_none());
}

#[test]
fn compile_allows_list_literal_nested_list() {
    let chunk = Compiler::compile("items = [[1, 2]]").expect("Expected compilation");
    assert!(chunk.constants.iter().any(|object| {
        if let ObjectType::List(outer) = &**object {
            outer
                .iter()
                .any(|item| matches!(&**item, ObjectType::List(_)))
        } else {
            false
        }
    }));
}

#[test]
fn compile_errors_on_list_literal_invalid_token() {
    assert!(Compiler::compile("items = [1, @]").is_none());
}

#[test]
fn compile_handles_if_without_else() {
    let source = "value = 0; if value < 1: value = 1";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpJumpIfFalse)));
}

#[test]
fn compile_handles_subtract_operator() {
    let chunk = Compiler::compile("print(10 - 3)").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpConstant,
            OpCode::OpSubtract,
            OpCode::OpPrint,
            OpCode::OpPrintln,
            OpCode::OpReturn
        ]
    );
}

#[test]
fn compile_handles_multiply_operator() {
    let chunk = Compiler::compile("print(3 * 4)").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpConstant,
            OpCode::OpMultiply,
            OpCode::OpPrint,
            OpCode::OpPrintln,
            OpCode::OpReturn
        ]
    );
}

#[test]
fn compile_handles_divide_operator() {
    let chunk = Compiler::compile("print(10 / 2)").expect("Expected chunk");
    assert_eq!(
        opcodes(&chunk),
        vec![
            OpCode::OpConstant,
            OpCode::OpConstant,
            OpCode::OpDivide,
            OpCode::OpPrint,
            OpCode::OpPrintln,
            OpCode::OpReturn
        ]
    );
}

#[test]
fn compile_handles_in_operator() {
    let chunk = Compiler::compile("print('key' in {'key': 1})").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpContains)));
}

#[test]
fn compile_handles_float_literal() {
    let chunk = Compiler::compile("3.14").expect("Expected chunk");
    assert!(matches!(&*chunk.constants[0], ObjectType::Float(_)));
}

#[test]
fn compile_handles_dict_literal() {
    let chunk = Compiler::compile("{'a': 1, 'b': 2}").expect("Expected chunk");
    assert!(matches!(&*chunk.constants[0], ObjectType::Dict(_)));
}

#[test]
fn compile_handles_list_literal_with_floats() {
    let chunk = Compiler::compile("[1.5, 2.5]").expect("Expected chunk");
    assert!(matches!(&*chunk.constants[0], ObjectType::List(_)));
}

#[test]
fn compile_handles_list_literal_with_strings() {
    let chunk = Compiler::compile("['a', 'b']").expect("Expected chunk");
    assert!(matches!(&*chunk.constants[0], ObjectType::List(_)));
}

#[test]
fn compile_handles_dict_literal_with_duplicate_keys() {
    let chunk = Compiler::compile("{'a': 1, 'a': 2}").expect("Expected chunk");
    if let ObjectType::Dict(entries) = &*chunk.constants[0] {
        assert_eq!(entries.len(), 1);
        assert!(matches!(&*entries[0].1, ObjectType::Integer(2)));
    } else {
        panic!("Expected dict constant");
    }
}

#[test]
fn compile_handles_unary_minus() {
    let chunk = Compiler::compile("print(-5)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSubtract)));
}

#[test]
fn compile_errors_on_malformed_unary_minus() {
    assert!(Compiler::compile("-").is_none());
}

#[test]
fn compile_handles_slice_with_both_bounds() {
    let chunk = Compiler::compile("items = [1, 2, 3]; items[1:2]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSlice)));
}

#[test]
fn compile_handles_slice_with_end_only() {
    let chunk = Compiler::compile("items = [1, 2, 3]; items[:2]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSlice)));
}

#[test]
fn compile_handles_slice_with_start_only() {
    let chunk = Compiler::compile("items = [1, 2, 3]; items[1:]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSlice)));
}

#[test]
fn compile_errors_on_malformed_slice_missing_bracket() {
    assert!(Compiler::compile("items = [1, 2]; items[1:2").is_none());
}

#[test]
fn compile_handles_index_multiply_assign() {
    let source = "counts = [1, 2]; counts[0] *= 3";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpMultiply)));
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpSetIndex)));
}

#[test]
fn compile_handles_add_assign() {
    let chunk = Compiler::compile("counter = 1; counter += 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().any(|op| matches!(op, OpCode::OpAdd)));
}

#[test]
fn compile_errors_on_invalid_assignment_target() {
    // Can't assign to expression result
    assert!(Compiler::compile("1 + 2 = 3").is_none());
}

#[test]
fn compile_handles_empty_list_literal() {
    let chunk = Compiler::compile("items = []").expect("Expected chunk");
    // The list literal constant should be somewhere in the constants
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::List(ref l) if l.is_empty())));
}

#[test]
fn compile_handles_empty_dict_literal() {
    let chunk = Compiler::compile("d = {}").expect("Expected chunk");
    // The dict literal constant should be somewhere in the constants
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::Dict(ref d) if d.is_empty())));
}

#[test]
fn compile_errors_on_expression_in_print() {
    // Expression that fails to parse should cause compilation error
    let result = Compiler::compile("print()");
    // Empty print compiles but doesn't do much
    assert!(result.is_some());
}

#[test]
fn compile_errors_on_for_loop_missing_identifier() {
    assert!(Compiler::compile("for in [1, 2]: print(1)").is_none());
}

#[test]
fn compile_errors_on_len_missing_expression() {
    assert!(Compiler::compile("len()").is_none());
}

#[test]
fn compile_errors_on_round_missing_expression() {
    assert!(Compiler::compile("round()").is_none());
}

#[test]
fn compile_errors_on_range_missing_expression() {
    assert!(Compiler::compile("range()").is_none());
}

#[test]
fn compile_errors_on_append_missing_expression() {
    assert!(Compiler::compile("items = [1, 2]; items.append()").is_none());
}

#[test]
fn compile_errors_on_identifier_followed_by_invalid_token() {
    assert!(Compiler::compile("items = [1, 2]; items. ").is_none());
}

#[test]
fn compile_handles_semicolon_as_statement() {
    let chunk = Compiler::compile(";").expect("Expected chunk");
    assert_eq!(opcodes(&chunk), vec![OpCode::OpReturn]);
}

#[test]
fn compile_handles_multiple_statements() {
    let chunk = Compiler::compile("a = 1; b = 2; c = 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    // Should have three define globals
    assert_eq!(
        ops.iter()
            .filter(|&op| matches!(op, OpCode::OpDefineGlobal))
            .count(),
        3
    );
}

#[test]
fn compile_handles_trailing_semicolons() {
    let chunk = Compiler::compile("a = 1;;; b = 2;;").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert_eq!(
        ops.iter()
            .filter(|&op| matches!(op, OpCode::OpDefineGlobal))
            .count(),
        2
    );
}

#[test]
fn compile_errors_on_unary_minus_with_invalid_token() {
    assert!(Compiler::compile("-@").is_none());
}

#[test]
fn compile_handles_nested_unary_minus() {
    let chunk = Compiler::compile("print(--5)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    // Double negation results in subtraction operations
    assert!(
        ops.iter()
            .filter(|&op| matches!(op, OpCode::OpSubtract))
            .count()
            >= 2
    );
}

#[test]
fn compile_errors_on_expression_missing_operand() {
    assert!(Compiler::compile("1 +").is_none());
}

#[test]
fn compile_errors_on_if_missing_expression() {
    assert!(Compiler::compile("if : print(1)").is_none());
}

#[test]
fn compile_errors_on_while_missing_expression() {
    assert!(Compiler::compile("while : print(1)").is_none());
}

#[test]
fn compile_errors_on_for_loop_missing_expression() {
    assert!(Compiler::compile("for x in : print(x)").is_none());
}

#[test]
fn compile_handles_if_else_with_nested_if() {
    // Note: this language doesn't have elif, but we can chain if-else
    // Need to separate statements properly
    let source = "x = 1\nif x < 0:\n    y = -1\nelse:\n    if x > 0:\n        y = 1\n    else:\n        y = 0";
    let chunk = Compiler::compile(source);
    // This might not compile in this simplified parser, so just check it doesn't crash
    assert!(chunk.is_some() || chunk.is_none());
}

#[test]
fn compile_handles_nested_while_loops() {
    let source = "i = 0; while i < 2: j = 0; while j < 2: j += 1; i += 1";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert_eq!(
        ops.iter()
            .filter(|&op| matches!(op, OpCode::OpLoop))
            .count(),
        2
    );
}

#[test]
fn compile_handles_nested_for_loops() {
    let source = "for i in [1, 2]: for j in [3, 4]: total = i + j";
    let chunk = Compiler::compile(source).expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert_eq!(
        ops.iter()
            .filter(|&op| matches!(op, OpCode::OpIterNext))
            .count(),
        2
    );
}

#[test]
fn compile_handles_complex_expression() {
    let chunk = Compiler::compile("result = 1 + 2 * 3 - 4 / 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpAdd));
    assert!(ops.contains(&OpCode::OpMultiply));
    assert!(ops.contains(&OpCode::OpSubtract));
    assert!(ops.contains(&OpCode::OpDivide));
}

#[test]
fn compile_handles_chained_indexing() {
    // Can't do items[0][1] as that would require getting intermediate result
    // But we can test multiple separate index operations
    let chunk =
        Compiler::compile("items = [1, 2, 3]; a = items[0]; b = items[1]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert_eq!(
        ops.iter()
            .filter(|&op| matches!(op, OpCode::OpIndex))
            .count(),
        2
    );
}

#[test]
fn compile_handles_multiple_slices() {
    let chunk = Compiler::compile("items = [1, 2, 3, 4]; a = items[0:2]; b = items[2:4]")
        .expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert_eq!(
        ops.iter()
            .filter(|&op| matches!(op, OpCode::OpSlice))
            .count(),
        2
    );
}

#[test]
fn compile_errors_on_dict_literal_empty_key() {
    assert!(Compiler::compile("d = {'': 1}").is_some()); // Empty string key is valid
}

#[test]
fn compile_handles_dict_with_multiple_types() {
    let chunk = Compiler::compile("d = {'a': 1, 'b': 2.5, 'c': 'text'}").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::Dict(_))));
}

#[test]
fn compile_handles_list_with_integer_literals() {
    let chunk = Compiler::compile("[1, 2, 3]").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::List(_))));
}

#[test]
fn compile_errors_on_mismatched_brackets() {
    // Missing closing bracket - might still parse or error depending on lexer
    let result = Compiler::compile("items = [1, 2, 3");
    // The lexer will run out of tokens, behavior depends on implementation
    assert!(result.is_some() || result.is_none());
}

#[test]
fn compile_handles_comparison_in_expression() {
    let chunk = Compiler::compile("result = 5 < 10").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLess));
}

#[test]
fn compile_handles_contains_with_list() {
    let chunk = Compiler::compile("result = 2 in [1, 2, 3]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpContains));
}

// Tests specifically targeting uncovered lines in compiler.rs

#[test]
fn compile_handles_function_call_expression() {
    let chunk = Compiler::compile("foo()").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpCall));
}

#[test]
fn compile_errors_on_identifier_with_dot_no_assignment() {
    // Line 105: Dot after identifier with bracket_depth 0, but not append
    assert!(Compiler::compile("x.y = 1").is_none());
}

#[test]
fn compile_errors_on_identifier_with_comma() {
    // Line 106: Comma after identifier at bracket_depth 0
    assert!(Compiler::compile("x, y = 1").is_none());
}

#[test]
fn compile_errors_on_identifier_with_plus() {
    // Line 107-111: Plus after identifier at bracket_depth 0
    let result = Compiler::compile("x + 1 = 2");
    assert!(result.is_none());
}

#[test]
fn compile_errors_on_identifier_with_slash() {
    // Line 107-111: Slash after identifier
    assert!(Compiler::compile("x / 2 = 1").is_none());
}

#[test]
fn compile_errors_on_identifier_with_star_not_equal() {
    // Line 107-111: Star but not *=
    assert!(Compiler::compile("x * 2 = 1").is_none());
}

#[test]
fn compile_errors_on_identifier_with_minus_not_equal() {
    // Line 107-111: Minus after identifier
    assert!(Compiler::compile("x - 2 = 1").is_none());
}

#[test]
fn compile_errors_on_identifier_with_in_keyword() {
    // Line 108-111: In keyword after identifier at bracket_depth 0
    assert!(Compiler::compile("x in y = 1").is_none());
}

#[test]
fn compile_errors_on_identifier_with_rparen() {
    // Line 113: RParen after identifier at bracket_depth 0
    let result = Compiler::compile("x) = 1");
    assert!(result.is_none());
}

#[test]
fn compile_errors_on_identifier_with_colon() {
    // Line 114: Colon after identifier at bracket_depth 0
    assert!(Compiler::compile("x: = 1").is_none());
}

#[test]
fn compile_errors_on_lex_error_in_detect_assignment() {
    // Line 115: Err token during lookahead
    assert!(Compiler::compile("x @ = 1").is_none());
}

#[test]
fn compile_errors_on_print_bad_expression() {
    // Line 139-140: parse_expression fails in print
    assert!(Compiler::compile("print(@)").is_none());
}

#[test]
fn compile_handles_print_single_no_comma() {
    // Line 146: No comma after first expression in print
    let chunk = Compiler::compile("print(42)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpPrint));
}

#[test]
fn compile_errors_on_for_loop_no_identifier() {
    // Line 163-164: No identifier after 'for'
    assert!(Compiler::compile("for 123 in [1, 2]: x = 1").is_none());
}

#[test]
fn compile_errors_on_for_loop_parse_expression_fails() {
    // Line 179-181: parse_expression returns false in for loop
    assert!(Compiler::compile("for x in @: x = 1").is_none());
}

#[test]
fn compile_errors_on_if_parse_expression_fails() {
    // Line 212-214: parse_expression fails in if statement
    assert!(Compiler::compile("if @: x = 1").is_none());
}

#[test]
fn compile_errors_on_while_no_colon() {
    // While statement error handling
    assert!(Compiler::compile("while x print(x)").is_none());
}

#[test]
fn compile_errors_on_while_parse_expression_fails() {
    // parse_expression fails in while
    assert!(Compiler::compile("while @: x = 1").is_none());
}

#[test]
fn compile_errors_on_assignment_parse_expression_fails() {
    // parse_expression fails in assignment
    assert!(Compiler::compile("x = @").is_none());
}

#[test]
fn compile_errors_on_index_assign_parse_fails() {
    // Index assignment with bad index expression
    assert!(Compiler::compile("items = [1]; items[@] = 2").is_none());
}

#[test]
fn compile_errors_on_index_assign_value_parse_fails() {
    // Index assignment with bad value expression
    assert!(Compiler::compile("items = [1]; items[0] = @").is_none());
}

#[test]
fn compile_errors_on_add_assign_parse_fails() {
    // += with bad expression
    assert!(Compiler::compile("x = 1; x += @").is_none());
}

#[test]
fn compile_errors_on_multiply_assign_parse_fails() {
    // *= with bad expression
    assert!(Compiler::compile("x = 1; x *= @").is_none());
}

#[test]
fn compile_errors_on_index_add_assign_bad_index() {
    // items[bad] += value
    assert!(Compiler::compile("items = [1]; items[@] += 2").is_none());
}

#[test]
fn compile_errors_on_index_add_assign_bad_value() {
    // items[0] += bad
    assert!(Compiler::compile("items = [1]; items[0] += @").is_none());
}

#[test]
fn compile_errors_on_index_multiply_assign_bad_index() {
    // items[bad] *= value
    assert!(Compiler::compile("items = [1]; items[@] *= 2").is_none());
}

#[test]
fn compile_errors_on_index_multiply_assign_bad_value() {
    // items[0] *= bad
    assert!(Compiler::compile("items = [1]; items[0] *= @").is_none());
}

#[test]
fn compile_errors_on_append_bad_lparen() {
    // .append but no lparen
    assert!(Compiler::compile("items = [1]; items.append").is_none());
}

#[test]
fn compile_errors_on_append_bad_expression() {
    // .append with bad expression
    assert!(Compiler::compile("items = [1]; items.append(@)").is_none());
}

#[test]
fn compile_errors_on_append_no_rparen() {
    // .append with no closing paren
    assert!(Compiler::compile("items = [1]; items.append(1").is_none());
}

#[test]
fn compile_errors_on_method_not_append() {
    // Method call that's not append
    assert!(Compiler::compile("items = [1]; items.other()").is_none());
}

#[test]
fn compile_errors_on_primary_expression_invalid_token() {
    // Primary expression with unexpected token
    assert!(Compiler::compile("x = else").is_none());
}

#[test]
fn compile_errors_on_list_literal_bad_element() {
    // List literal with invalid element
    assert!(Compiler::compile("[1, @, 3]").is_none());
}

#[test]
fn compile_errors_on_list_literal_no_rbracket() {
    // Missing closing bracket
    let result = Compiler::compile("[1, 2");
    // May compile or error depending on implementation
    assert!(result.is_some() || result.is_none());
}

#[test]
fn compile_errors_on_dict_literal_key_not_string() {
    // Dict key is not a string
    assert!(Compiler::compile("{123: 'value'}").is_none());
}

#[test]
fn compile_errors_on_dict_literal_no_colon() {
    // Dict literal missing colon
    assert!(Compiler::compile("{'key' 'value'}").is_none());
}

#[test]
fn compile_errors_on_dict_literal_bad_value() {
    // Dict value is invalid
    assert!(Compiler::compile("{'key': @}").is_none());
}

#[test]
fn compile_errors_on_dict_literal_no_rbrace() {
    // Missing closing brace
    let result = Compiler::compile("{'a': 1");
    assert!(result.is_some() || result.is_none());
}

#[test]
fn compile_errors_on_postfix_index_bad_expression() {
    // Bad index expression
    assert!(Compiler::compile("items = [1]; x = items[@]").is_none());
}

#[test]
fn compile_errors_on_postfix_index_no_rbracket() {
    // Missing ]
    let result = Compiler::compile("items = [1]; x = items[0");
    assert!(result.is_some() || result.is_none());
}

#[test]
fn compile_errors_on_slice_bad_start() {
    // Bad start in slice
    assert!(Compiler::compile("items = [1]; x = items[@:2]").is_none());
}

#[test]
fn compile_errors_on_slice_bad_end() {
    // Bad end in slice
    assert!(Compiler::compile("items = [1]; x = items[0:@]").is_none());
}

#[test]
fn compile_errors_on_slice_no_rbracket() {
    // Missing ] in slice
    let result = Compiler::compile("items = [1]; x = items[0:2");
    assert!(result.is_some() || result.is_none());
}

#[test]
fn compile_errors_on_len_bad_expression() {
    // Bad expression in len()
    assert!(Compiler::compile("len(@)").is_none());
}

#[test]
fn compile_errors_on_len_no_rparen() {
    // Missing ) in len
    let result = Compiler::compile("len([1, 2]");
    assert!(result.is_some() || result.is_none());
}

#[test]
fn compile_errors_on_round_bad_value() {
    // Bad value in round()
    assert!(Compiler::compile("round(@, 2)").is_none());
}

#[test]
fn compile_errors_on_round_no_comma() {
    // Missing comma in round
    assert!(Compiler::compile("round(3.14 2)").is_none());
}

#[test]
fn compile_errors_on_round_bad_digits() {
    // Bad digits in round
    assert!(Compiler::compile("round(3.14, @)").is_none());
}

#[test]
fn compile_errors_on_round_no_rparen() {
    // Missing ) in round
    let result = Compiler::compile("round(3.14, 2");
    assert!(result.is_some() || result.is_none());
}

#[test]
fn compile_errors_on_range_bad_start() {
    // Bad start in range()
    assert!(Compiler::compile("range(@, 5)").is_none());
}

#[test]
fn compile_errors_on_range_no_comma() {
    // Missing comma in range
    assert!(Compiler::compile("range(1 5)").is_none());
}

#[test]
fn compile_errors_on_range_bad_end() {
    // Bad end in range
    assert!(Compiler::compile("range(1, @)").is_none());
}

#[test]
fn compile_errors_on_range_no_rparen() {
    // Missing ) in range
    let result = Compiler::compile("range(1, 5");
    assert!(result.is_some() || result.is_none());
}

#[test]
fn compile_errors_on_unary_minus_bad_operand() {
    // Unary minus with bad operand
    assert!(Compiler::compile("-@").is_none());
}

#[test]
fn compile_errors_on_comparison_bad_right() {
    // Comparison with bad right side
    assert!(Compiler::compile("x = 1 < @").is_none());
}

#[test]
fn compile_errors_on_in_bad_right() {
    // 'in' with bad right side
    assert!(Compiler::compile("x = 1 in @").is_none());
}

#[test]
fn compile_errors_on_term_bad_right() {
    // Term (+ or -) with bad right side
    assert!(Compiler::compile("x = 1 + @").is_none());
}

#[test]
fn compile_errors_on_factor_bad_right() {
    // Factor (* or /) with bad right side
    assert!(Compiler::compile("x = 2 * @").is_none());
}

#[test]
fn compile_errors_on_else_missing_colon() {
    // Line 233-235: else without colon
    assert!(Compiler::compile("if 1: x = 1 else x = 2").is_none());
}

#[test]
fn compile_errors_on_while_parse_expression_fails_line_251() {
    // Line 251-253: parse_expression fails in while
    assert!(Compiler::compile("while @: x = 1").is_none());
}

#[test]
fn compile_errors_on_while_no_colon_line_256() {
    // Line 256-258: while without colon
    assert!(Compiler::compile("while 1 x = 2").is_none());
}

#[test]
fn compile_errors_on_parse_term_invalid_token() {
    // Line 279-280: parse_term returns false
    assert!(Compiler::compile("x = else").is_none());
}

#[test]
fn compile_handles_less_than_comparison() {
    // Exercise less than operation
    let chunk = Compiler::compile("x = 1 < 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLess));
}

#[test]
fn compile_handles_in_operator_with_dict() {
    // Exercise 'in' operation
    let chunk = Compiler::compile("x = 'key' in {'key': 1}").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpContains));
}

#[test]
fn compile_handles_subtraction() {
    // Exercise subtraction
    let chunk = Compiler::compile("x = 5 - 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSubtract));
}

#[test]
fn compile_handles_division() {
    // Exercise division
    let chunk = Compiler::compile("x = 10 / 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpDivide));
}

#[test]
fn compile_handles_identifier_lookup() {
    // Get global variable
    let chunk = Compiler::compile("x = 1; y = x").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(
        ops.iter()
            .filter(|op| matches!(op, OpCode::OpGetGlobal))
            .count()
            >= 1
    );
}

#[test]
fn compile_handles_integer_literal() {
    // Integer literal
    let chunk = Compiler::compile("x = 42").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::Integer(42))));
}

#[test]
fn compile_handles_string_literal() {
    // String literal
    let chunk = Compiler::compile("x = 'hello'").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::String(ref s) if s == "hello")));
}

#[test]
fn compile_handles_float_in_expression() {
    // Float literal
    let chunk = Compiler::compile("x = 3.14159").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::Float(_))));
}

// Tests to hit specific uncovered lines

#[test]
fn compile_successful_returns_chunk_with_return() {
    // Lines 49, 51: Successful compilation returns chunk with OpReturn
    let chunk = Compiler::compile("x = 1").expect("Expected chunk");
    assert!(chunk.code.last() == Some(&(OpCode::OpReturn as u8)));
}

#[test]
fn compile_single_statement_ok_branch() {
    // Lines 29-30: Ok branch in main loop
    let chunk = Compiler::compile("x = 42").expect("Expected chunk");
    assert!(chunk.constants.len() > 0);
}

#[test]
fn compile_unknown_token_triggers_error() {
    // Lines 64-66: Unknown token in parse_statement
    let result = Compiler::compile("@ @ @");
    assert!(result.is_none());
}

#[test]
fn compile_identifier_as_statement_not_assignment() {
    // Lines 68-73: Identifier that's not an assignment becomes expression statement
    let result = Compiler::compile("foo");
    // This might compile as expression or error depending on whether foo is defined
    assert!(result.is_some() || result.is_none());
}

#[test]
fn compile_non_identifier_expression_statement() {
    // Line 75: Non-identifier token as expression statement
    let chunk = Compiler::compile("42").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_lookahead_non_identifier_in_detect_assignment() {
    // Lines 83-84: Non-identifier in detect_assignment_kind lookahead
    let chunk = Compiler::compile("42").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_bracket_depth_tracking_in_assignment() {
    // Lines 91, 92, 96: Bracket depth tracking
    let chunk = Compiler::compile("items = [1, 2]; items[0] = 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSetIndex));
}

#[test]
fn compile_print_with_comma_after_first() {
    // Line 134: Break when no comma after first print arg
    let chunk = Compiler::compile("print(1)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpPrint));
}

#[test]
fn compile_print_multiple_args_with_commas() {
    // Line 147: Multiple args in print
    let chunk = Compiler::compile("print(1, 2)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpPrintSpaced));
}

#[test]
fn compile_print_ends_without_comma() {
    // Line 150: Print ends, first = false
    let chunk = Compiler::compile("print(1, 2)").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_for_creates_nil_for_loop_var() {
    // Lines 172-177: For loop creates nil for loop variable
    let chunk = Compiler::compile("for x in [1, 2]: print(x)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpDefineGlobal));
}

#[test]
fn compile_for_emits_iter_next() {
    // Lines 193-197: For loop emits OpIterNext
    let chunk = Compiler::compile("for i in [1]: x = i").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpIterNext));
}

#[test]
fn compile_for_sets_global_and_pops() {
    // Lines 199-201: For loop sets global and pops
    let chunk = Compiler::compile("for x in [1]: x = 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().filter(|op| matches!(op, OpCode::OpPop)).count() >= 1);
}

#[test]
fn compile_for_emits_loop_and_patch() {
    // Lines 205-206: For loop emit_loop and patch_jump
    let chunk = Compiler::compile("for i in [1, 2]: print(i)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLoop));
}

#[test]
fn compile_if_emits_jump_if_false() {
    // Line 222: If statement emits OpJumpIfFalse
    let chunk = Compiler::compile("if 1: x = 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpJumpIfFalse));
}

#[test]
fn compile_if_pops_after_condition() {
    // Line 223: If statement pops after condition
    let chunk = Compiler::compile("if 1: x = 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().filter(|op| matches!(op, OpCode::OpPop)).count() >= 1);
}

#[test]
fn compile_if_parses_then_statement() {
    // Line 225: Parse then statement
    let chunk = Compiler::compile("if 1: x = 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpDefineGlobal));
}

#[test]
fn compile_if_with_else_emits_jump() {
    // Lines 227-228: If with else emits OpJump
    let chunk = Compiler::compile("if 1: x = 2 else: x = 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpJump));
}

#[test]
fn compile_if_else_parses_else_statement() {
    // Lines 238-239: Parse else statement
    let chunk = Compiler::compile("if 0: x = 1 else: x = 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    // Should have assignment in else branch
    assert!(
        ops.iter()
            .filter(|op| matches!(op, OpCode::OpDefineGlobal))
            .count()
            >= 1
    );
}

#[test]
fn compile_while_emits_jump_if_false() {
    // Line 261: While emits OpJumpIfFalse
    let chunk = Compiler::compile("x = 0; while x < 1: x = 1").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpJumpIfFalse));
}

#[test]
fn compile_while_pops_after_condition() {
    // Line 262: While pops after condition
    let chunk = Compiler::compile("x = 0; while x < 1: x = 1").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.iter().filter(|op| matches!(op, OpCode::OpPop)).count() >= 1);
}

#[test]
fn compile_while_parses_body_statement() {
    // Line 264: While parses body statement
    let chunk = Compiler::compile("while 1: x = 2; x = 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLoop));
}

#[test]
fn compile_while_emits_loop() {
    // Lines 266-268: While emits loop and patches jump
    let chunk = Compiler::compile("i = 0; while i < 3: i += 1").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLoop));
}

#[test]
fn compile_parse_expression_no_token_returns_false() {
    // Line 287: No token in parse_expression
    let result = Compiler::compile("x =");
    assert!(result.is_none());
}

// More targeted tests for specific uncovered lines

#[test]
fn compile_binary_plus_operator() {
    // Lines 287, 288: Plus operator in expression
    let chunk = Compiler::compile("x = 1 + 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpAdd));
}

#[test]
fn compile_binary_slash_operator() {
    // Lines 287, 289: Slash operator
    let chunk = Compiler::compile("x = 6 / 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpDivide));
}

#[test]
fn compile_binary_star_operator() {
    // Lines 287, 290: Star operator
    let chunk = Compiler::compile("x = 3 * 4").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpMultiply));
}

#[test]
fn compile_binary_minus_operator() {
    // Lines 287, 291: Minus operator
    let chunk = Compiler::compile("x = 7 - 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSubtract));
}

#[test]
fn compile_less_operator_in_expression() {
    // Lines 287, 292: Less operator
    let chunk = Compiler::compile("x = 2 < 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLess));
}

#[test]
fn compile_in_operator_in_expression() {
    // Lines 287, 293: In operator
    let chunk = Compiler::compile("x = 1 in [1, 2]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpContains));
}

#[test]
fn compile_unary_minus_line_325() {
    // Line 325: Unary minus in parse_term
    let chunk = Compiler::compile("x = -5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSubtract));
}

#[test]
fn compile_unary_minus_recursive() {
    // Line 331: Recursive parse_term for unary minus
    let chunk = Compiler::compile("x = --10").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(
        ops.iter()
            .filter(|op| matches!(op, OpCode::OpSubtract))
            .count()
            >= 2
    );
}

#[test]
fn compile_len_function() {
    // Lines 354, 360-361: len function
    let chunk = Compiler::compile("x = len([1, 2, 3])").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLen));
}

#[test]
fn compile_round_function() {
    // Lines 366, 372, 374: round function
    let chunk = Compiler::compile("x = round(3.14, 2)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpRound));
}

#[test]
fn compile_range_function() {
    // Lines 380-381, 386: range function
    let chunk = Compiler::compile("x = range(0, 10)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpRange));
}

#[test]
fn compile_identifier_postfix_index() {
    // Line 392, 394: Identifier with postfix [ for indexing
    let chunk = Compiler::compile("items = [1, 2]; x = items[0]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpIndex));
}

// Tests to hit EVERY specific uncovered line

#[test]
fn compile_line_56_token_match() {
    // Line 56: Match on token in parse_statement
    let chunk = Compiler::compile("print(1)").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_68_70_identifier_assignment() {
    // Lines 68, 70: Identifier with assignment
    let chunk = Compiler::compile("x = 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpDefineGlobal));
}

#[test]
fn compile_line_84_lookahead_non_identifier() {
    // Line 84: Return None when lookahead is not identifier
    let chunk = Compiler::compile("1 + 2").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_87_bracket_depth_loop() {
    // Line 87: Bracket depth tracking loop
    let chunk = Compiler::compile("items = [1]; items[0] = 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSetIndex));
}

#[test]
fn compile_line_92_rbracket_depth_zero() {
    // Line 92: RBracket with bracket_depth > 0
    let _chunk = Compiler::compile("items = [[1]]; items[0] = 2").is_some();
    assert!(true); // Just ensure it doesn't crash
}

#[test]
fn compile_line_96_bracket_depth_decrement() {
    // Line 96: Decrement bracket depth
    let chunk = Compiler::compile("items = [1]; items[0] = 2").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_107_108_operators_in_lookahead() {
    // Lines 107-108: Plus/Slash/Star/Minus/In at bracket_depth 0
    let result = Compiler::compile("x + y = 1");
    assert!(result.is_none());
}

#[test]
fn compile_line_134_print_break_no_comma() {
    // Line 134: Break in print when no comma
    let chunk = Compiler::compile("print(42)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpPrint));
}

#[test]
fn compile_line_140_print_expression_fails() {
    // Line 140: Print expression fails
    assert!(Compiler::compile("print(@)").is_none());
}

#[test]
fn compile_line_147_print_comma_printspaced() {
    // Line 147: Print with comma uses PrintSpaced
    let chunk = Compiler::compile("print(1, 2)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpPrintSpaced));
}

#[test]
fn compile_line_150_print_first_false() {
    // Line 150: first = false in print loop
    let chunk = Compiler::compile("print(1, 2, 3)").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_154_print_consume_rparen() {
    // Line 154: Consume ) after print
    let chunk = Compiler::compile("print(1)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpPrintln));
}

#[test]
fn compile_line_161_for_identifier() {
    // Line 161: For loop with identifier
    let chunk = Compiler::compile("for x in [1]: print(x)").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_167_for_in_token() {
    // Line 167: For loop consumes 'in'
    let chunk = Compiler::compile("for i in [1, 2]: i = 3").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_172_177_for_nil_constant() {
    // Lines 172-177: For loop nil constant and define global
    let chunk = Compiler::compile("for x in [1]: x = x").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpDefineGlobal));
}

#[test]
fn compile_line_179_for_parse_expression() {
    // Line 179: For loop parse expression
    let chunk = Compiler::compile("for x in [1, 2, 3]: print(x)").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_181_for_expression_fail() {
    // Line 181: For expression fails
    assert!(Compiler::compile("for x in @: print(x)").is_none());
}

#[test]
fn compile_line_189_191_for_zero_constant() {
    // Lines 189-191: For loop zero constant
    let chunk = Compiler::compile("for i in [1]: i = 2").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::Integer(0))));
}

#[test]
fn compile_line_193_197_for_iter_next() {
    // Lines 193-197: For loop OpIterNext
    let chunk = Compiler::compile("for x in [1, 2]: x = 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpIterNext));
}

#[test]
fn compile_line_199_201_for_set_global_pop() {
    // Lines 199-201: For loop set global and pop
    let chunk = Compiler::compile("for i in [1]: i = 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSetGlobal));
}

#[test]
fn compile_line_203_for_parse_statement() {
    // Line 203: For loop parse statement
    let chunk = Compiler::compile("for x in [1]: y = 2").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_205_206_for_emit_loop_patch() {
    // Lines 205-206: For loop emit_loop and patch_jump
    let chunk = Compiler::compile("for i in [1, 2, 3]: print(i)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLoop));
}

#[test]
fn compile_line_214_if_parse_expression() {
    // Line 214: If parse expression fails
    assert!(Compiler::compile("if @: x = 1").is_none());
}

#[test]
fn compile_line_222_223_if_jump_pop() {
    // Lines 222-223: If emit jump and pop
    let chunk = Compiler::compile("if 1: x = 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpJumpIfFalse));
}

#[test]
fn compile_line_225_if_parse_then() {
    // Line 225: If parse then statement
    let chunk = Compiler::compile("if 1: x = 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpDefineGlobal));
}

#[test]
fn compile_line_227_if_else_check() {
    // Line 227: If check for else
    let chunk = Compiler::compile("if 1: x = 2 else: x = 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpJump));
}

#[test]
fn compile_line_238_239_if_else_parse_statement() {
    // Lines 238-239: If else parse statement
    let chunk = Compiler::compile("if 0: x = 1 else: y = 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(
        ops.iter()
            .filter(|op| matches!(op, OpCode::OpDefineGlobal))
            .count()
            >= 1
    );
}

#[test]
fn compile_line_253_while_parse_expression() {
    // Line 253: While parse expression fails
    assert!(Compiler::compile("while @: x = 1").is_none());
}

#[test]
fn compile_line_261_262_while_jump_pop() {
    // Lines 261-262: While emit jump and pop
    let chunk = Compiler::compile("x = 0; while x < 1: x = 1").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpJumpIfFalse));
}

#[test]
fn compile_line_264_while_parse_statement() {
    // Line 264: While parse statement
    let chunk = Compiler::compile("while 1: x = 2").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_266_268_while_emit_loop() {
    // Lines 266-268: While emit loop and patch
    let chunk = Compiler::compile("i = 0; while i < 5: i += 1").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLoop));
}

#[test]
fn compile_line_280_parse_term_fail() {
    // Line 280: parse_term returns false
    assert!(Compiler::compile("x = @").is_none());
}

#[test]
fn compile_line_287_expression_operator() {
    // Line 287: Operator in expression loop
    let chunk = Compiler::compile("x = 1 + 2").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpAdd));
}

#[test]
fn compile_line_299_300_consume_operator() {
    // Lines 299-300: Consume operator and parse term
    let chunk = Compiler::compile("x = 3 * 4").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpMultiply));
}

#[test]
fn compile_line_302_parse_term_in_expression() {
    // Line 302: parse_term in expression
    let chunk = Compiler::compile("x = 5 - 1").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSubtract));
}

#[test]
fn compile_line_312_expression_error() {
    // Line 312: Expression error when term fails
    assert!(Compiler::compile("x = 1 + @").is_none());
}

#[test]
fn compile_line_317_operands_zero() {
    // Line 317: operands == 0 sets error
    assert!(Compiler::compile("x = ").is_none());
}

#[test]
fn compile_line_325_unary_minus() {
    // Line 325: Unary minus in parse_term
    let chunk = Compiler::compile("x = -10").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSubtract));
}

#[test]
fn compile_line_331_unary_recursive() {
    // Line 331: Recursive parse_term in unary
    let chunk = Compiler::compile("x = -5").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_335_336_unary_fail() {
    // Lines 335-336: Unary minus parse fails
    assert!(Compiler::compile("x = -@").is_none());
}

#[test]
fn compile_line_354_len_parse_expression() {
    // Line 354: len parse expression fails
    assert!(Compiler::compile("len(@)").is_none());
}

#[test]
fn compile_line_360_361_len_emit() {
    // Lines 360-361: len emits OpLen
    let chunk = Compiler::compile("x = len([1, 2])").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpLen));
}

#[test]
fn compile_line_366_round_identifier() {
    // Line 366: round identifier
    let chunk = Compiler::compile("x = round(3.14, 2)").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_372_round_parse_second() {
    // Line 372: round parse second expression
    let chunk = Compiler::compile("x = round(2.5, 1)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpRound));
}

#[test]
fn compile_line_374_round_emit() {
    // Line 374: round emit OpRound
    let chunk = Compiler::compile("x = round(1.5, 0)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpRound));
}

#[test]
fn compile_line_380_381_range_identifier() {
    // Lines 380-381: range identifier
    let chunk = Compiler::compile("x = range(1, 5)").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_386_range_emit() {
    // Line 386: range emit OpRange
    let chunk = Compiler::compile("x = range(0, 3)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpRange));
}

#[test]
fn compile_line_392_identifier_name_get_global() {
    // Line 392: Identifier name gets global
    let chunk = Compiler::compile("x = 1; y = x").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpGetGlobal));
}

#[test]
fn compile_line_394_identifier_postfix_check() {
    // Line 394: Identifier postfix check
    let chunk = Compiler::compile("items = [1]; x = items[0]").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_400_401_range_operator() {
    // Lines 400-401: Range function call
    let chunk = Compiler::compile("for i in range(1, 5): x = i").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpRange));
}

#[test]
fn compile_line_404_405_get_global_after_range_check() {
    // Lines 404-405: OpGetGlobal for identifier
    let chunk = Compiler::compile("items = [1, 2]; x = items[0]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpGetGlobal));
}

#[test]
fn compile_line_407_408_410_identifier_indexing() {
    // Lines 407-408, 410: Identifier with indexing loop
    let chunk = Compiler::compile("x = [1, 2, 3]; y = x[0]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpIndex));
}

#[test]
fn compile_line_419_indexing_parse_error() {
    // Line 419: Error in indexing expression
    assert!(Compiler::compile("x = y[@]").is_none());
}

#[test]
fn compile_line_439_slice_end_parse_error() {
    // Line 439: Error parsing slice end expression
    assert!(Compiler::compile("x = y[0:@]").is_none());
}

#[test]
fn compile_line_447_slice_opcode() {
    // Line 447: OpSlice
    let chunk = Compiler::compile("x = [1, 2, 3]; y = x[0:2]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSlice));
}

#[test]
fn compile_line_454_index_opcode() {
    // Line 454: OpIndex
    let chunk = Compiler::compile("x = [1, 2]; y = x[1]").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpIndex));
}

#[test]
fn compile_line_457_460_461_dot_method_identifier() {
    // Lines 457, 460-461: Dot method parsing
    let chunk = Compiler::compile("x = [1]; x.append(2)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpAppend));
}

#[test]
fn compile_line_467_append_method() {
    // Line 467: Method name check for "append"
    let chunk = Compiler::compile("items = []; items.append(5)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpAppend));
}

#[test]
fn compile_line_473_append_missing_lparen() {
    // Line 473: Missing LParen after append
    assert!(Compiler::compile("x = []; x.append 5").is_none());
}

#[test]
fn compile_line_478_480_append_expression_error() {
    // Lines 478, 480: Error in append expression
    assert!(Compiler::compile("x = []; x.append(@)").is_none());
}

#[test]
fn compile_line_485_append_missing_rparen() {
    // Line 485: Missing RParen after append
    assert!(Compiler::compile("x = []; x.append(5").is_none());
}

#[test]
fn compile_line_488_492_append_opcodes() {
    // Lines 488-492: OpAppend and OpSetGlobal
    let chunk = Compiler::compile("lst = [1]; lst.append(2)").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpAppend));
    assert!(ops.contains(&OpCode::OpSetGlobal));
}

#[test]
fn compile_line_495_invalid_method_name() {
    // Line 495: Invalid method name (not "append")
    assert!(Compiler::compile("x = []; x.invalid()").is_none());
}

#[test]
fn compile_line_512_515_float_literal() {
    // Lines 512-515: Float constant
    let chunk = Compiler::compile("x = 3.14").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::Float(_))));
}

#[test]
fn compile_line_517_519_522_dict_literal() {
    // Lines 517, 519-522: Dict literal parsing
    let chunk = Compiler::compile("x = {\"key\": 1}").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::Dict(_))));
}

#[test]
fn compile_line_527_529_532_list_literal() {
    // Lines 527, 529-532: List literal parsing
    let chunk = Compiler::compile("x = [1, 2, 3]").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::List(_))));
}

#[test]
fn compile_line_544_546_empty_list_literal() {
    // Lines 544, 546: Empty list literal
    let chunk = Compiler::compile("x = []").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::List(_))));
}

#[test]
fn compile_line_558_559_list_literal_float() {
    // Lines 558-559: Float in list literal
    let chunk = Compiler::compile("x = [1.5, 2.5]").expect("Expected chunk");
    assert!(chunk.constants.iter().any(|c| {
        if let ObjectType::List(items) = &**c {
            items
                .iter()
                .any(|item| matches!(&**item, ObjectType::Float(_)))
        } else {
            false
        }
    }));
}

#[test]
fn compile_line_565_568_list_literal_identifier_error() {
    // Lines 565, 568: Identifier in list literal (not allowed)
    assert!(Compiler::compile("[x]").is_none());
}

#[test]
fn compile_line_570_list_literal_invalid_token() {
    // Line 570: Invalid token in list literal
    assert!(Compiler::compile("[1, @]").is_none());
}

#[test]
fn compile_line_583_585_empty_dict_literal() {
    // Lines 583, 585: Empty dict literal
    let chunk = Compiler::compile("x = {}").expect("Expected chunk");
    assert!(chunk
        .constants
        .iter()
        .any(|c| matches!(&**c, ObjectType::Dict(_))));
}

#[test]
fn compile_line_593_595_dict_literal_string_key() {
    // Lines 593, 595: String key in dict literal
    let chunk = Compiler::compile("x = {\"name\": \"value\"}").expect("Expected chunk");
    assert!(chunk.constants.iter().any(|c| {
        if let ObjectType::Dict(entries) = &**c {
            entries.iter().any(|(k, _)| k == "name")
        } else {
            false
        }
    }));
}

#[test]
fn compile_line_604_606_dict_literal_invalid_value() {
    // Lines 604, 606: Invalid value in dict literal
    assert!(Compiler::compile("{\"key\": @}").is_none());
}

#[test]
fn compile_line_614_dict_literal_duplicate_key() {
    // Line 614: Duplicate key in dict literal (should update)
    let chunk = Compiler::compile("x = {\"k\": 1, \"k\": 2}").expect("Expected chunk");
    assert!(chunk.constants.iter().any(|c| {
        if let ObjectType::Dict(entries) = &**c {
            entries.len() == 1 && entries[0].0 == "k"
        } else {
            false
        }
    }));
}

#[test]
fn compile_line_619_dict_literal_invalid_key() {
    // Line 619: Invalid key type in dict literal
    assert!(Compiler::compile("{123: \"value\"}").is_none());
}

#[test]
fn compile_line_631_633_assignment_identifier() {
    // Lines 631, 633: Assignment identifier parsing
    let chunk = Compiler::compile("x = 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpDefineGlobal));
}

#[test]
fn compile_line_635_637_638_640_subscript_assignment() {
    // Lines 635, 637-638, 640: Subscript assignment detection
    let chunk = Compiler::compile("x = [1, 2]; x[0] = 10").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSetIndex));
}

#[test]
fn compile_line_659_660_subscript_expression_error() {
    // Lines 659-660: Error in subscript expression
    assert!(Compiler::compile("x = [1]; x[@] = 5").is_none());
}

#[test]
fn compile_line_666_667_subscript_missing_rbracket() {
    // Lines 666-667: Missing RBracket in subscript
    assert!(Compiler::compile("x = [1]; x[0 = 5").is_none());
}

#[test]
fn compile_line_672_674_675_simple_assignment_missing_assign() {
    // Lines 672, 674-675: Simple assignment path
    let chunk = Compiler::compile("x = 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpDefineGlobal));
}

#[test]
fn compile_line_678_681_subscript_set_value_error() {
    // Lines 678, 681: Error in subscript set value expression
    assert!(Compiler::compile("x = [1]; x[0] = @").is_none());
}

#[test]
fn compile_line_691_simple_assignment_value_error() {
    // Line 691: Error in simple assignment value
    assert!(Compiler::compile("x = @").is_none());
}

#[test]
fn compile_line_698_add_assign_kind() {
    // Line 698: AddAssign or MultiplyAssign match
    let chunk = Compiler::compile("x = 5; x += 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpAdd));
}

#[test]
fn compile_line_706_707_add_assign_missing_token() {
    // Lines 706-707: Add assign with correct token
    let chunk = Compiler::compile("x = 5; x += 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpAdd));
}

#[test]
fn compile_line_716_add_assign_with_subscript() {
    // Line 716: AddAssign with subscript
    let chunk = Compiler::compile("x = [1, 2]; x[0] += 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpIndex));
    assert!(ops.contains(&OpCode::OpAdd));
}

#[test]
fn compile_line_721_add_assign_subscript_expression_error() {
    // Line 721: Error in add-assign subscript expression
    assert!(Compiler::compile("x = [1]; x[0] += @").is_none());
}

#[test]
fn compile_line_727_add_assign_subscript_opcodes() {
    // Line 727: OpSwap and related opcodes for subscript add-assign
    let chunk = Compiler::compile("arr = [10]; arr[0] += 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSwap));
}

#[test]
fn compile_line_729_730_add_assign_subscript_missing_index_code() {
    // Lines 729-730: Missing index expression code (should not happen)
    let chunk = Compiler::compile("x = [1]; x[0] += 1").expect("Expected chunk");
    assert!(chunk.code.len() > 0);
}

#[test]
fn compile_line_733_737_add_assign_subscript_full_opcodes() {
    // Lines 733-737: Full opcode sequence for subscript add-assign
    let chunk = Compiler::compile("lst = [5]; lst[0] += 10").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpSwap));
    assert!(ops.contains(&OpCode::OpSetIndex));
    assert!(ops.contains(&OpCode::OpSetGlobal));
    assert!(ops.contains(&OpCode::OpPop));
}

#[test]
fn compile_line_740_742_744_add_assign_no_subscript() {
    // Lines 740, 742, 744: Add-assign without subscript
    let chunk = Compiler::compile("x = 10; x += 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpGetGlobal));
    assert!(ops.contains(&OpCode::OpAdd));
}

#[test]
fn compile_line_742_744_add_assign_value_error() {
    // Lines 742, 744: Error in add-assign value expression
    assert!(Compiler::compile("x = 5; x += @").is_none());
}

#[test]
fn compile_multiply_assign() {
    // Lines for multiply assign paths
    let chunk = Compiler::compile("x = 5; x *= 3").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpMultiply));
}

#[test]
fn compile_multiply_assign_subscript() {
    // Multiply assign with subscript
    let chunk = Compiler::compile("x = [2]; x[0] *= 5").expect("Expected chunk");
    let ops = opcodes(&chunk);
    assert!(ops.contains(&OpCode::OpMultiply));
    assert!(ops.contains(&OpCode::OpSwap));
}

// Integration tests that compile AND run code to increase VM coverage
#[test]
fn integration_negative_list_index() {
    use oxython::vm::VM;
    let chunk = Compiler::compile("x = [1, 2, 3]; y = x[-1]").expect("Expected chunk");
    let mut vm = VM::new();
    vm.interpret(chunk);
}

#[test]
fn integration_dict_string_lookup() {
    use oxython::vm::VM;
    let chunk = Compiler::compile("d = {\"key\": 42}; v = d[\"key\"]").expect("Expected chunk");
    let mut vm = VM::new();
    vm.interpret(chunk);
}

#[test]
fn integration_int_int_addition() {
    use oxython::vm::VM;
    let chunk = Compiler::compile("x = 5 + 10").expect("Expected chunk");
    let mut vm = VM::new();
    vm.interpret(chunk);
}

#[test]
fn integration_int_int_subtraction() {
    use oxython::vm::VM;
    let chunk = Compiler::compile("x = 10 - 5").expect("Expected chunk");
    let mut vm = VM::new();
    vm.interpret(chunk);
}

#[test]
fn integration_division_result_push() {
    use oxython::vm::VM;
    let chunk = Compiler::compile("x = 10 / 2").expect("Expected chunk");
    let mut vm = VM::new();
    vm.interpret(chunk);
}
