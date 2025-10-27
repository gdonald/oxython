use std::process::Command;

fn run_example(path: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_oxython");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut cmd = Command::new(binary);
    cmd.current_dir(manifest_dir).arg(path);

    let output = cmd.output().expect("failed to execute example");
    assert!(
        output.status.success(),
        "example {} exited with status {:?}",
        path,
        output.status
    );

    String::from_utf8(output.stdout).expect("stdout was not utf8")
}

// ============================================================================
// BASICS
// ============================================================================

#[test]
fn run_sum_literal() {
    let output = run_example("examples/basics/sum_literal.py");
    assert_eq!(output, "6\n");
}

#[test]
fn run_integers_example() {
    let output = run_example("examples/basics/integers.py");
    assert_eq!(output, "c: 3\n");
}

#[test]
fn run_greeting_line() {
    let output = run_example("examples/basics/greeting_line.py");
    assert_eq!(output, "Hello, Ada!\n");
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[test]
fn run_simple_list() {
    let output = run_example("examples/data-structures/simple_list.py");
    assert_eq!(output, "[1, 2, 3]\n2\n3\n");
}

#[test]
fn run_simple_dict() {
    let output = run_example("examples/data-structures/simple_dict.py");
    assert_eq!(output, "{'alice': 30, 'bob': 25}\n30\n");
}

#[test]
fn run_dict_access() {
    let output = run_example("examples/data-structures/dict_access.py");
    assert_eq!(output, "Ada lives in London\n");
}

#[test]
fn run_list_append() {
    let output = run_example("examples/data-structures/list_append.py");
    assert_eq!(output, "[1, 2, 3]\n");
}

// ============================================================================
// ALGORITHMS
// ============================================================================

#[test]
fn run_average_temperature() {
    let output = run_example("examples/algorithms/average_temperature.py");
    assert_eq!(output, "69.9\n");
}

#[test]
fn run_character_counter() {
    let output = run_example("examples/algorithms/character_counter.py");
    assert_eq!(output, "{'b': 1, 'a': 3, 'n': 2}\n");
}

#[test]
fn run_factorial_iterative() {
    let output = run_example("examples/algorithms/factorial_iterative.py");
    assert_eq!(output, "720\n");
}

#[test]
fn run_fib_sequence() {
    let output = run_example("examples/algorithms/fib_sequence.py");
    assert_eq!(output, "[0, 1, 1, 2, 3, 5, 8, 13]\n");
}

#[test]
fn run_filter_even_numbers() {
    let output = run_example("examples/algorithms/filter_even_numbers.py");
    assert_eq!(output, "[2, 4, 6]\n");
}

#[test]
fn run_matrix_transpose() {
    let output = run_example("examples/algorithms/matrix_transpose.py");
    assert_eq!(output, "[[1, 4], [2, 5], [3, 6]]\n");
}

#[test]
fn run_palindrome_check() {
    let output = run_example("examples/algorithms/palindrome_check.py");
    assert_eq!(output, "True\n");
}

#[test]
fn run_primes_under_fifty() {
    let output = run_example("examples/algorithms/primes_under_fifty.py");
    assert_eq!(
        output,
        "[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]\n"
    );
}

#[test]
fn run_zip_merger() {
    let output = run_example("examples/algorithms/zip_merger.py");
    assert_eq!(output, "[('Ann', 88), ('Ben', 93)]\n");
}

// ============================================================================
// FUNCTIONS
// ============================================================================

#[test]
fn run_function() {
    let output = run_example("examples/functions/function.py");
    assert_eq!(output, "7\n");
}

#[test]
fn run_locals_scope() {
    let output = run_example("examples/functions/locals_scope.py");
    assert_eq!(output, "20\n[0, 2, 4, 6, 8]\n");
}

#[test]
fn run_closures_nested() {
    let output = run_example("examples/functions/closures_nested.py");
    assert_eq!(output, "10\n12\n");
}

#[test]
fn run_nonlocal_counter() {
    let output = run_example("examples/functions/nonlocal_counter.py");
    assert_eq!(output, "1\n2\n3\n3\n0\n1\n");
}

// ============================================================================
// OBJECT-ORIENTED PROGRAMMING
// ============================================================================

#[test]
fn run_class() {
    let output = run_example("examples/oop/class.py");
    assert_eq!(output, "Hello, Ada!\n");
}

#[test]
fn run_inheritance() {
    let output = run_example("examples/oop/inheritance.py");
    assert_eq!(output, "Buddy says Woof!\nWhiskers says Meow!\n");
}

#[test]
fn run_super() {
    let output = run_example("examples/oop/super.py");
    assert_eq!(
        output,
        "Buddy\nGolden Retriever\nSome sound -> Woof!\nI am an animal named Buddy\n"
    );
}

#[test]
fn run_test_str() {
    let output = run_example("examples/oop/test_str.py");
    assert_eq!(output, "Person: Alice\n");
}

#[test]
fn run_test_repr() {
    let output = run_example("examples/oop/test_repr.py");
    assert_eq!(output, "Point(0, 0)\n");
}

#[test]
fn run_test_iter() {
    let output = run_example("examples/oop/test_iter.py");
    assert_eq!(output, "next\n");
}

#[test]
fn run_special_methods() {
    let output = run_example("examples/oop/special_methods.py");
    assert_eq!(
        output,
        "Book: Python Guide\nMagazine: Tech Monthly\nnext_value\n"
    );
}
