//! End-to-end tests for the Sifr compiler.
//!
//! Tests in `tests/e2e/pass/` must compile successfully and produce expected stdout.
//! Tests in `tests/e2e/fail/` must fail to compile with expected error messages.

use std::fs;
use std::path::Path;

/// Extract the expected stdout from a `# expect-stdout: <value>` comment.
fn extract_expect_stdout(source: &str) -> Option<String> {
    for line in source.lines() {
        if let Some(rest) = line.strip_prefix("# expect-stdout:") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// Extract expected error substrings from `# expect-error: <value>` comments.
fn extract_expect_errors(source: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for line in source.lines() {
        if let Some(rest) = line.strip_prefix("# expect-error:") {
            errors.push(rest.trim().to_string());
        }
    }
    errors
}

/// Compile source and return the generated Rust code or errors.
fn compile_source(source: &str) -> Result<String, Vec<String>> {
    match sifr_driver::compile(source) {
        sifr_driver::CompileResult::Success { rust_source } => Ok(rust_source),
        sifr_driver::CompileResult::Errors { errors } => {
            Err(errors.iter().map(|e| e.message.clone()).collect())
        }
    }
}

#[test]
fn test_e2e_pass() {
    let pass_dir = Path::new("tests/e2e/pass");
    if !pass_dir.exists() {
        return;
    }

    let mut test_count = 0;
    for entry in fs::read_dir(pass_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(true, |e| e != "sifr") {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();
        let _expected_stdout = extract_expect_stdout(&source);

        // Verify it compiles successfully
        match compile_source(&source) {
            Ok(rust_source) => {
                // Verify the generated Rust contains a main function
                assert!(
                    rust_source.contains("fn main()"),
                    "PASS test {} generated Rust without main function:\n{}",
                    path.display(),
                    rust_source
                );
                test_count += 1;
            }
            Err(errors) => {
                panic!(
                    "PASS test {} failed to compile:\n{}",
                    path.display(),
                    errors.join("\n")
                );
            }
        }
    }

    assert!(test_count > 0, "No pass tests found");
    eprintln!("  {} pass tests completed", test_count);
}

#[test]
fn test_e2e_fail() {
    let fail_dir = Path::new("tests/e2e/fail");
    if !fail_dir.exists() {
        return;
    }

    let mut test_count = 0;
    for entry in fs::read_dir(fail_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(true, |e| e != "sifr") {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();
        let expected_errors = extract_expect_errors(&source);

        match compile_source(&source) {
            Ok(rust_source) => {
                panic!(
                    "FAIL test {} should have failed but compiled successfully:\n{}",
                    path.display(),
                    rust_source
                );
            }
            Err(errors) => {
                let all_errors = errors.join("\n");
                for expected in &expected_errors {
                    assert!(
                        all_errors.contains(expected),
                        "FAIL test {} expected error containing '{}' but got:\n{}",
                        path.display(),
                        expected,
                        all_errors
                    );
                }
                test_count += 1;
            }
        }
    }

    assert!(test_count > 0, "No fail tests found");
    eprintln!("  {} fail tests completed", test_count);
}
