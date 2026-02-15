//! End-to-end tests for the Sifr compiler.
//!
//! Tests in `tests/e2e/pass/` must:
//!   1. Compile through sifr (parse + type-check + codegen)
//!   2. Produce valid Rust that compiles with rustc
//!   3. When run, produce the expected stdout (if `# expect-stdout:` is present)
//!
//! Tests in `tests/e2e/fail/` must fail to compile with expected error messages.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Collect all `# expect-stdout: <value>` lines into a multi-line expected output.
fn extract_expect_stdout(source: &str) -> Option<String> {
    let lines: Vec<&str> = source
        .lines()
        .filter_map(|line| line.strip_prefix("# expect-stdout:").map(|rest| rest.trim()))
        .collect();
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
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

/// Compile source and return the generated Rust code and stdlib modules, or errors.
fn compile_source(source: &str) -> Result<String, Vec<String>> {
    match sifr_driver::compile(source) {
        sifr_driver::CompileResult::Success { rust_source } => Ok(rust_source),
        sifr_driver::CompileResult::Errors { errors } => {
            Err(errors.iter().map(|e| e.message.clone()).collect())
        }
    }
}

/// Compile source and return the generated Rust code with stdlib metadata.
fn compile_source_with_metadata(source: &str) -> Result<(String, HashSet<String>), Vec<String>> {
    match sifr_driver::compile_with_metadata(source) {
        sifr_driver::CompileResultFull::Success { rust_source, used_stdlib_modules } => {
            Ok((rust_source, used_stdlib_modules))
        }
        sifr_driver::CompileResultFull::Errors { errors } => {
            Err(errors.iter().map(|e| e.message.clone()).collect())
        }
    }
}

/// Generate Cargo.toml content with optional stdlib dependencies.
fn generate_cargo_toml(stdlib_modules: &HashSet<String>) -> String {
    let mut cargo_toml = r#"[package]
name = "sifr_output"
version = "0.1.0"
edition = "2021"
"#.to_string();

    let mut deps = Vec::new();
    for module_name in stdlib_modules {
        match module_name.as_str() {
            "sifr.json" | "sifr.collections" => {
                if !deps.contains(&"serde_json = \"1\"") {
                    deps.push("serde_json = \"1\"");
                    deps.push("serde = { version = \"1\", features = [\"derive\"] }");
                }
            }
            _ => {}
        }
    }

    if !deps.is_empty() {
        cargo_toml.push_str("\n[dependencies]\n");
        for dep in &deps {
            cargo_toml.push_str(dep);
            cargo_toml.push('\n');
        }
    }

    cargo_toml
}

/// Build the generated Rust source with stdlib dependencies into a binary and run it.
fn build_and_run_with_deps(rust_source: &str, test_name: &str, stdlib_modules: &HashSet<String>) -> Result<String, String> {
    let tmp_dir = std::env::temp_dir().join("sifr_e2e_tests").join(test_name);
    let src_dir = tmp_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("failed to create dir: {}", e))?;

    // Write Cargo.toml with stdlib dependencies
    let cargo_toml = generate_cargo_toml(stdlib_modules);
    fs::write(tmp_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("failed to write Cargo.toml: {}", e))?;

    // Write main.rs
    fs::write(src_dir.join("main.rs"), rust_source)
        .map_err(|e| format!("failed to write main.rs: {}", e))?;

    // Compile with cargo build
    let build_output = Command::new("cargo")
        .args(["build", "--quiet"])
        .current_dir(&tmp_dir)
        .output()
        .map_err(|e| format!("failed to run cargo build: {}", e))?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        return Err(format!(
            "Rust compilation failed.\n\nGenerated Rust:\n{}\n\nrustc errors:\n{}",
            rust_source, stderr
        ));
    }

    // Run the binary
    let binary_name = if cfg!(target_os = "windows") {
        "sifr_output.exe"
    } else {
        "sifr_output"
    };
    let binary_path = tmp_dir.join("target").join("debug").join(binary_name);

    let run_output = Command::new(&binary_path)
        .output()
        .map_err(|e| format!("failed to run binary: {}", e))?;

    if !run_output.status.success() {
        let stderr = String::from_utf8_lossy(&run_output.stderr);
        return Err(format!("binary exited with error:\n{}", stderr));
    }

    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    Ok(stdout)
}

#[test]
fn test_e2e_pass() {
    let pass_dir = Path::new("tests/e2e/pass");
    if !pass_dir.exists() {
        return;
    }

    let mut entries: Vec<_> = fs::read_dir(pass_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sifr"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut test_count = 0;
    let mut failures: Vec<String> = Vec::new();

    for entry in &entries {
        let path = entry.path();
        let test_name = path.file_stem().unwrap().to_string_lossy().to_string();
        let source = fs::read_to_string(&path).unwrap();
        let expected_stdout = extract_expect_stdout(&source);

        // Step 1: Sifr compilation (parse + type-check + codegen)
        let (rust_source, used_stdlib_modules) = match compile_source_with_metadata(&source) {
            Ok(result) => result,
            Err(errors) => {
                failures.push(format!(
                    "FAIL [{}]: sifr compilation failed:\n  {}",
                    test_name,
                    errors.join("\n  ")
                ));
                continue;
            }
        };

        // Verify the generated Rust contains a main function
        if !rust_source.contains("fn main()") {
            failures.push(format!(
                "FAIL [{}]: generated Rust has no main function",
                test_name
            ));
            continue;
        }

        // Step 2: Compile generated Rust with rustc and run it
        match build_and_run_with_deps(&rust_source, &test_name, &used_stdlib_modules) {
            Ok(stdout) => {
                // Step 3: Verify stdout if expected
                if let Some(expected) = &expected_stdout {
                    let actual = stdout.trim_end();
                    let expected = expected.trim_end();
                    if actual != expected {
                        failures.push(format!(
                            "FAIL [{}]: stdout mismatch\n  expected: {:?}\n  actual:   {:?}",
                            test_name, expected, actual
                        ));
                        continue;
                    }
                }
            }
            Err(err) => {
                failures.push(format!("FAIL [{}]: {}", test_name, err));
                continue;
            }
        }

        test_count += 1;
    }

    if !failures.is_empty() {
        panic!(
            "\n{} E2E pass test(s) failed:\n\n{}\n\n({} passed, {} failed)",
            failures.len(),
            failures.join("\n\n"),
            test_count,
            failures.len()
        );
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
