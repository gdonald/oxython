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

#[test]
fn run_sum_literal() {
    let output = run_example("examples/sum_literal.py");
    assert_eq!(output, "6\n");
}

#[test]
fn run_integers_example() {
    let output = run_example("examples/integers.py");
    assert_eq!(output, "c: 3\n");
}

#[test]
fn run_simple_list() {
    let output = run_example("examples/simple_list.py");
    assert_eq!(output, "[1, 2, 3]\n2\n3\n");
}

#[test]
fn run_simple_dict() {
    let output = run_example("examples/simple_dict.py");
    assert_eq!(output, "{'alice': 30, 'bob': 25}\n30\n");
}

#[test]
fn run_dict_access() {
    let output = run_example("examples/dict_access.py");
    assert_eq!(output, "Ada lives in London\n");
}

#[test]
fn run_list_append() {
    let output = run_example("examples/list_append.py");
    assert_eq!(output, "[1, 2, 3]\n");
}

#[test]
fn run_average_temperature() {
    let output = run_example("examples/average_temperature.py");
    assert_eq!(output, "69.9\n");
}

#[test]
fn run_character_counter() {
    let output = run_example("examples/character_counter.py");
    assert_eq!(output, "{'b': 1, 'a': 3, 'n': 2}\n");
}

#[test]
fn run_factorial_iterative() {
    let output = run_example("examples/factorial_iterative.py");
    assert_eq!(output, "720\n");
}

#[test]
fn run_fib_sequence() {
    let output = run_example("examples/fib_sequence.py");
    assert_eq!(output, "[0, 1, 1, 2, 3, 5, 8, 13]\n");
}

#[test]
fn run_filter_even_numbers() {
    let output = run_example("examples/filter_even_numbers.py");
    assert_eq!(output, "[2, 4, 6]\n");
}

#[test]
fn run_greeting_line() {
    let output = run_example("examples/greeting_line.py");
    assert_eq!(output, "Hello, Ada!\n");
}

#[test]
fn run_matrix_transpose() {
    let output = run_example("examples/matrix_transpose.py");
    assert_eq!(output, "[[1, 4], [2, 5], [3, 6]]\n");
}

#[test]
fn run_palindrome_check() {
    let output = run_example("examples/palindrome_check.py");
    assert_eq!(output, "True\n");
}

#[test]
fn run_primes_under_fifty() {
    let output = run_example("examples/primes_under_fifty.py");
    assert_eq!(
        output,
        "[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]\n"
    );
}

#[test]
fn run_zip_merger() {
    let output = run_example("examples/zip_merger.py");
    assert_eq!(output, "[('Ann', 88), ('Ben', 93)]\n");
}

// #[test]
// fn run_class() {
//     let output = run_example("examples/class.py");
//     assert_eq!(output, "Hello, Ada!\n");
// }

// #[test]
// fn run_function() {
//     let output = run_example("examples/function.py");
//     assert_eq!(output, "7\n");
// }
