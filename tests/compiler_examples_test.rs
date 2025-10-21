use oxython::compiler::Compiler;
use std::fs;

#[test]
fn compile_every_example_script() {
    let unsupported = [
        "filter_even_numbers.py",
        "primes_under_fifty.py",
        "palindrome_check.py",
        "matrix_transpose.py",
        "zip_merger.py",
    ];

    for entry in fs::read_dir("examples").expect("examples directory missing") {
        let entry = entry.expect("invalid dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("py") {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if unsupported.contains(&name) {
                    continue;
                }
            }
            let source = fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("failed to read {:?}: {}", path, e);
            });
            assert!(
                Compiler::compile(&source).is_some(),
                "Compilation failed for {:?}",
                path
            );
        }
    }
}
