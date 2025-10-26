use oxython::cli::{
    handle_args, handle_args_with_prompt, run_file, run_main_with_args, run_prompt,
    run_prompt_with_io, run_prompt_with_streams,
};
use std::cell::Cell;
use std::env;
use std::fs;
use std::io::{Cursor, Read};

const BANNER: &str = include_str!("../src/banner.txt");

#[test]
fn handle_args_runs_prompt_for_no_args() {
    let args: Vec<String> = vec![];
    let called = Cell::new(false);
    handle_args_with_prompt(&args, || called.set(true)).unwrap();
    assert!(called.get());
}

#[test]
fn handle_args_reports_usage_error_for_extra_args() {
    let args = vec![String::from("script.py"), String::from("extra")];
    let result = handle_args_with_prompt(&args, || {});
    assert_eq!(result.unwrap_err(), 64);
}

#[test]
fn run_file_executes_valid_script() {
    let mut path = env::temp_dir();
    path.push(format!("oxython_test_{}_ok.py", std::process::id()));
    fs::write(&path, "print('ok')").unwrap();

    assert!(run_file(path.to_str().unwrap()).is_ok());

    let _ = fs::remove_file(&path);
}

#[test]
fn run_file_reports_missing_file() {
    let mut path = env::temp_dir();
    path.push(format!("oxython_test_{}_missing.py", std::process::id()));

    let result = run_file(path.to_str().unwrap());
    assert_eq!(result.unwrap_err(), 74);
}

#[test]
fn run_prompt_with_io_evaluates_expression() {
    let input = b"1 + 1\n";
    let mut reader = Cursor::new(&input[..]);
    let mut buffer = Vec::new();

    run_prompt_with_io(&mut reader, &mut buffer).unwrap();

    let mut output = String::new();
    Cursor::new(buffer).read_to_string(&mut output).unwrap();

    assert!(output.contains(BANNER.trim_end()));
    assert!(output.contains("\nWelcome to the oxython REPL!"));
    assert!(output.contains("> "));
    assert!(output.contains("2"));
}

#[test]
fn run_prompt_with_streams_returns_captured_output() {
    let writer = run_prompt_with_streams(Cursor::new(&b""[..]), Vec::<u8>::new()).unwrap();
    let output = String::from_utf8(writer).unwrap();
    assert!(output.contains(BANNER.trim_end()));
    assert!(output.contains("\nWelcome to the oxython REPL!"));
    assert!(output.contains("> "));
}

#[test]
fn handle_args_runs_file_for_single_arg() {
    let mut path = env::temp_dir();
    path.push(format!(
        "oxythonlang_test_arg_run_{}.py",
        std::process::id()
    ));
    fs::write(&path, "print('ok')").unwrap();
    let result = handle_args(&[path.to_str().unwrap().to_string()]);
    assert!(result.is_ok());
    let _ = fs::remove_file(&path);
}

#[test]
fn handle_args_reports_usage_for_many_args() {
    let result = handle_args(&[
        String::from("one"),
        String::from("two"),
        String::from("three"),
    ]);
    assert_eq!(result.unwrap_err(), 64);
}

#[test]
fn run_main_with_args_executes_script() {
    let mut path = env::temp_dir();
    path.push(format!(
        "oxythonlang_test_{}_cli_run.py",
        std::process::id()
    ));
    fs::write(&path, "print('ok')").unwrap();

    let args = vec![path.to_str().unwrap().to_string()];
    assert!(run_main_with_args(&args).is_ok());

    let _ = fs::remove_file(&path);
}

#[test]
fn run_main_with_args_reports_usage_error() {
    let args = vec![String::from("one"), String::from("two")];
    assert_eq!(run_main_with_args(&args).unwrap_err(), 64);
}

#[test]
fn run_prompt_reports_compile_error() {
    let input = b"@\n";
    let mut reader = Cursor::new(&input[..]);
    let mut buffer = Vec::new();

    run_prompt_with_io(&mut reader, &mut buffer).unwrap();

    let mut output = String::new();
    Cursor::new(buffer).read_to_string(&mut output).unwrap();

    assert!(output.contains("Compilation failed."));
}

#[test]
fn run_prompt_reports_runtime_error() {
    let input = b"a\n";
    let mut reader = Cursor::new(&input[..]);
    let mut buffer = Vec::new();

    run_prompt_with_io(&mut reader, &mut buffer).unwrap();

    let mut output = String::new();
    Cursor::new(buffer).read_to_string(&mut output).unwrap();

    assert!(output.contains("Runtime error."));
}

#[test]
fn run_file_reports_compile_error() {
    let mut path = env::temp_dir();
    path.push(format!(
        "oxythonlang_test_{}_compile_err.py",
        std::process::id()
    ));
    fs::write(&path, "@").unwrap();

    let result = run_file(path.to_str().unwrap());
    assert_eq!(result.unwrap_err(), 65);

    let _ = fs::remove_file(&path);
}

#[test]
fn run_main_with_args_propagates_compile_error() {
    let mut path = env::temp_dir();
    path.push(format!(
        "oxythonlang_test_{}_main_compile_err.py",
        std::process::id()
    ));
    fs::write(&path, "@").unwrap();

    let args = vec![path.to_str().unwrap().to_string()];
    let result = run_main_with_args(&args);
    assert_eq!(result.unwrap_err(), 65);

    let _ = fs::remove_file(&path);
}

#[test]
fn run_prompt_skips_blank_lines_and_reports_compile_failure() {
    let input = b"\n@\n";
    let mut reader = Cursor::new(&input[..]);
    let mut buffer = Vec::new();

    run_prompt_with_io(&mut reader, &mut buffer).unwrap();

    let mut output = String::new();
    Cursor::new(buffer).read_to_string(&mut output).unwrap();

    assert!(output.contains("> > "));
    assert!(output.contains("Compilation failed."));
}

#[test]
fn run_prompt_displays_expression_without_pop() {
    let input = b"x = 5\nx\n";
    let mut reader = Cursor::new(&input[..]);
    let mut buffer = Vec::new();

    run_prompt_with_io(&mut reader, &mut buffer).unwrap();

    let mut output = String::new();
    Cursor::new(buffer).read_to_string(&mut output).unwrap();

    assert!(output.contains("5"));
}

#[test]
fn run_prompt_suppresses_nil_values() {
    let input = b"x = nil\nx\n";
    let mut reader = Cursor::new(&input[..]);
    let mut buffer = Vec::new();

    run_prompt_with_io(&mut reader, &mut buffer).unwrap();

    let mut output = String::new();
    Cursor::new(buffer).read_to_string(&mut output).unwrap();

    // Should not display "nil" on a separate line
    let lines: Vec<&str> = output.lines().collect();
    let after_prompts: Vec<&str> = lines
        .iter()
        .filter(|l| {
            !l.starts_with('>')
                && !l.trim().is_empty()
                && !l.contains("Welcome")
                && !l.contains("REPL")
                && !l.contains("oxython")
        })
        .copied()
        .collect();

    // Nil values should not be printed
    assert!(!after_prompts
        .iter()
        .any(|l| l.contains("nil") || l.contains("Nil")));
}

#[test]
fn run_file_executes_and_interprets() {
    let mut path = env::temp_dir();
    path.push(format!("oxythonlang_test_{}_exec.py", std::process::id()));
    fs::write(&path, "x = 10\ny = 20\nz = x + y").unwrap();

    // This should execute the VM interpret path (lines 47-49)
    let result = run_file(path.to_str().unwrap());
    assert!(result.is_ok());

    let _ = fs::remove_file(&path);
}

#[cfg(unix)]
#[test]
fn run_prompt_uses_stdin_stdout() {
    let output = with_piped_stdio("1 + 2\n", run_prompt);
    assert!(output.contains("Welcome to the oxython REPL!"));
    assert!(
        output.contains("3"),
        "Captured output missing 3: {:?}",
        output
    );
}

#[cfg(unix)]
#[test]
fn handle_args_no_args_invokes_prompt() {
    let output = with_piped_stdio("40 + 2\n", || handle_args(&[]).unwrap());
    assert!(
        output.contains("42"),
        "Captured output missing 42: {:?}",
        output
    );
}

#[cfg(unix)]
fn with_piped_stdio<F>(input: &str, f: F) -> String
where
    F: FnOnce(),
{
    use std::fs::File;
    use std::io::Read;
    use std::os::unix::io::{FromRawFd, RawFd};

    unsafe fn make_pipe() -> (RawFd, RawFd) {
        let mut fds = [0; 2];
        if libc::pipe(fds.as_mut_ptr()) != 0 {
            panic!("pipe failed");
        }
        (fds[0], fds[1])
    }

    unsafe {
        let (stdin_read, stdin_write) = make_pipe();
        let (stdout_read, stdout_write) = make_pipe();

        let old_stdin = libc::dup(libc::STDIN_FILENO);
        let old_stdout = libc::dup(libc::STDOUT_FILENO);

        libc::write(
            stdin_write,
            input.as_bytes().as_ptr() as *const _,
            input.len(),
        );
        libc::close(stdin_write);

        libc::dup2(stdin_read, libc::STDIN_FILENO);
        libc::close(stdin_read);

        libc::dup2(stdout_write, libc::STDOUT_FILENO);
        libc::close(stdout_write);

        f();

        libc::dup2(old_stdin, libc::STDIN_FILENO);
        libc::dup2(old_stdout, libc::STDOUT_FILENO);
        libc::close(old_stdin);
        libc::close(old_stdout);

        let mut output = String::new();
        let mut reader = File::from_raw_fd(stdout_read);
        reader.read_to_string(&mut output).unwrap();
        output
    }
}
