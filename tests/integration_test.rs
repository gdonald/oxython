use std::process::Command;

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_run_integers_example() {
    let output = Command::new("target/debug/oxython")
        .arg("examples/integers.py")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "c: 3\n");
}

#[test]
fn test_usage_error_exit_code() {
    let output = Command::new("target/debug/oxython")
        .args(["one", "two"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage: oxython"));
}
