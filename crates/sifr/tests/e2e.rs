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

/// Collect all `# expect-stderr: <value>` lines.
fn extract_expect_stderr(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("# expect-stderr:").map(|rest| rest.trim().to_string()))
        .collect()
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
        sifr_driver::CompileResultFull::Success {
            rust_source,
            used_stdlib_modules,
            required_crates: _,
            lowering_stats: _,
        } => {
            Ok((rust_source, used_stdlib_modules))
        }
        sifr_driver::CompileResultFull::Errors { errors } => {
            Err(errors.iter().map(|e| e.message.clone()).collect())
        }
    }
}

/// Compile source and return generated Rust/stdlib metadata using an explicit codegen lowering mode.
fn compile_source_with_metadata_mode(
    source: &str,
    lowering_mode: sifr_driver::CodegenLoweringMode,
) -> Result<(String, HashSet<String>), Vec<String>> {
    match sifr_driver::compile_with_metadata_mode(source, lowering_mode) {
        sifr_driver::CompileResultFull::Success {
            rust_source,
            used_stdlib_modules,
            required_crates: _,
            lowering_stats: _,
        } => {
            Ok((rust_source, used_stdlib_modules))
        }
        sifr_driver::CompileResultFull::Errors { errors } => {
            Err(errors.iter().map(|e| e.message.clone()).collect())
        }
    }
}

/// Compile source and return Rust/stdlib metadata and lowering stats for gate checks.
fn compile_source_with_metadata_mode_and_stats(
    source: &str,
    lowering_mode: sifr_driver::CodegenLoweringMode,
) -> Result<(String, HashSet<String>, sifr_driver::LoweringStats), Vec<String>> {
    match sifr_driver::compile_with_metadata_mode(source, lowering_mode) {
        sifr_driver::CompileResultFull::Success {
            rust_source,
            used_stdlib_modules,
            required_crates: _,
            lowering_stats,
        } => Ok((rust_source, used_stdlib_modules, lowering_stats)),
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
            "sifr.json" | "sifr.collections" | "_sifr.json" | "_sifr.collections" => {
                if !deps.contains(&"serde_json = \"1\"") {
                    deps.push("serde_json = \"1\"");
                    deps.push("serde = { version = \"1\", features = [\"derive\"] }");
                }
            }
            "sifr.time" | "_sifr.time" => {
                if !deps.contains(&"chrono = \"0.4\"") {
                    deps.push("chrono = \"0.4\"");
                }
            }
            "sifr.random" | "_sifr.crypto" => {
                if !deps.contains(&"rand = \"0.8\"") {
                    deps.push("rand = \"0.8\"");
                }
                if !deps.contains(&"rand_distr = \"0.4\"") {
                    deps.push("rand_distr = \"0.4\"");
                }
            }
            "sifr.uuid" | "_sifr.uuid" => {
                if !deps.contains(&"rand = \"0.8\"") {
                    deps.push("rand = \"0.8\"");
                }
            }
            "sifr.re" | "_sifr.regex" => {
                if !deps.contains(&"regex = \"1\"") {
                    deps.push("regex = \"1\"");
                }
            }
            "sifr.hash" | "sifr.hashlib" => {
                if !deps.contains(&"sha2 = \"0.10\"") {
                    deps.push("sha2 = \"0.10\"");
                    deps.push("md5 = \"0.7\"");
                    deps.push("blake2 = \"0.10\"");
                }
            }
            "sifr.encoding" | "sifr.base64" => {
                if !deps.contains(&"base64 = \"0.22\"") {
                    deps.push("base64 = \"0.22\"");
                }
            }
            "sifr.tomllib" | "_sifr.toml" => {
                if !deps.contains(&"toml = \"0.8\"") {
                    deps.push("toml = \"0.8\"");
                }
            }
            "sifr.datetime" | "_sifr.datetime" => {
                if !deps.contains(&"chrono = \"0.4\"") {
                    deps.push("chrono = \"0.4\"");
                }
            }
            "sifr.gzip" | "sifr.zipfile" | "_sifr.compress" => {
                if !deps.contains(&"flate2 = \"1\"") {
                    deps.push("flate2 = \"1\"");
                }
                if !deps.contains(&"zip = \"0.6\"") {
                    deps.push("zip = \"0.6\"");
                }
            }
            "_bigint" => {
                if !deps.contains(&"num-bigint = \"0.4\"") {
                    deps.push("num-bigint = \"0.4\"");
                    deps.push("num-traits = \"0.2\"");
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
/// Returns (stdout, stderr, success_status).
fn build_and_run_capture_with_deps(
    rust_source: &str,
    test_name: &str,
    stdlib_modules: &HashSet<String>,
) -> Result<(String, String, bool), String> {
    let tmp_dir = std::env::temp_dir().join("sifr_e2e_tests").join(test_name);
    let src_dir = tmp_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("failed to create dir: {}", e))?;

    // Write Cargo.toml with stdlib dependencies (also detect bigint usage)
    let mut effective_modules = stdlib_modules.clone();
    if rust_source.contains("num_bigint::BigInt") || rust_source.contains("use num_bigint") {
        effective_modules.insert("_bigint".to_string());
    }
    let cargo_toml = generate_cargo_toml(&effective_modules);
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

    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();
    Ok((stdout, stderr, run_output.status.success()))
}

/// Build and run, requiring successful process exit.
fn build_and_run_with_deps(
    rust_source: &str,
    test_name: &str,
    stdlib_modules: &HashSet<String>,
) -> Result<String, String> {
    match build_and_run_capture_with_deps(rust_source, test_name, stdlib_modules) {
        Ok((stdout, stderr, success)) => {
            if success {
                Ok(stdout)
            } else {
                Err(format!("binary exited with error:\n{}", stderr))
            }
        }
        Err(e) => Err(e),
    }
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
fn test_codegen_differential_old_vs_new_corpus_parity() {
    let pass_dir = Path::new("tests/e2e/pass");
    let corpus = [
        "if_else",
        "narrowing_elif_equality",
        "loop_else",
        "subscript_aug_assign",
        "subscript_nested_assign",
        "for_tuple_unpack",
        "del_statement",
        "match_guard",
    ];

    let mut test_count = 0;
    let mut failures: Vec<String> = Vec::new();

    for case in &corpus {
        let path = pass_dir.join(format!("{case}.sifr"));
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(err) => {
                failures.push(format!("FAIL [{}]: unable to read fixture {}: {}", case, path.display(), err));
                continue;
            }
        };
        let expected_stdout = extract_expect_stdout(&source);

        let (new_rust, new_modules) = match compile_source_with_metadata_mode(
            &source,
            sifr_driver::CodegenLoweringMode::StructuredPreferred,
        ) {
            Ok(result) => result,
            Err(errors) => {
                failures.push(format!(
                    "FAIL [{}][StructuredPreferred]: sifr compilation failed:\n  {}",
                    case,
                    errors.join("\n  ")
                ));
                continue;
            }
        };

        let (legacy_rust, legacy_modules) = match compile_source_with_metadata_mode(
            &source,
            sifr_driver::CodegenLoweringMode::LegacyOnly,
        ) {
            Ok(result) => result,
            Err(errors) => {
                failures.push(format!(
                    "FAIL [{}][LegacyOnly]: sifr compilation failed:\n  {}",
                    case,
                    errors.join("\n  ")
                ));
                continue;
            }
        };

        if !new_rust.contains("fn main()") {
            failures.push(format!(
                "FAIL [{}][StructuredPreferred]: generated Rust has no main function",
                case
            ));
            continue;
        }
        if !legacy_rust.contains("fn main()") {
            failures.push(format!(
                "FAIL [{}][LegacyOnly]: generated Rust has no main function",
                case
            ));
            continue;
        }

        let new_stdout = match build_and_run_with_deps(
            &new_rust,
            &format!("{case}_structured"),
            &new_modules,
        ) {
            Ok(stdout) => stdout,
            Err(err) => {
                failures.push(format!("FAIL [{}][StructuredPreferred]: {}", case, err));
                continue;
            }
        };

        let legacy_stdout = match build_and_run_with_deps(
            &legacy_rust,
            &format!("{case}_legacy"),
            &legacy_modules,
        ) {
            Ok(stdout) => stdout,
            Err(err) => {
                failures.push(format!("FAIL [{}][LegacyOnly]: {}", case, err));
                continue;
            }
        };

        let new_trimmed = new_stdout.trim_end();
        let legacy_trimmed = legacy_stdout.trim_end();

        if let Some(expected) = &expected_stdout {
            let expected_trimmed = expected.trim_end();
            if new_trimmed != expected_trimmed {
                failures.push(format!(
                    "FAIL [{}][StructuredPreferred]: stdout mismatch\n  expected: {:?}\n  actual:   {:?}",
                    case, expected_trimmed, new_trimmed
                ));
                continue;
            }
            if legacy_trimmed != expected_trimmed {
                failures.push(format!(
                    "FAIL [{}][LegacyOnly]: stdout mismatch\n  expected: {:?}\n  actual:   {:?}",
                    case, expected_trimmed, legacy_trimmed
                ));
                continue;
            }
        }

        if new_trimmed != legacy_trimmed {
            failures.push(format!(
                "FAIL [{}]: differential mismatch\n  structured: {:?}\n  legacy:     {:?}",
                case, new_trimmed, legacy_trimmed
            ));
            continue;
        }

        test_count += 1;
    }

    if !failures.is_empty() {
        panic!(
            "\n{} differential parity test(s) failed:\n\n{}\n\n({} passed, {} failed)",
            failures.len(),
            failures.join("\n\n"),
            test_count,
            failures.len()
        );
    }

    assert_eq!(
        test_count,
        corpus.len(),
        "Not all differential corpus cases executed successfully"
    );
    eprintln!("  {} differential parity tests completed", test_count);
}

#[test]
fn test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus() {
    let pass_dir = Path::new("tests/e2e/pass");
    let corpus = ["codegen_structured_ratio_gate"];

    let mut total_stmt_candidate = 0_u64;
    let mut total_stmt_candidate_structured = 0_u64;
    let mut total_expr_candidate = 0_u64;
    let mut total_expr_candidate_structured = 0_u64;
    let mut failures: Vec<String> = Vec::new();

    for case in &corpus {
        let path = pass_dir.join(format!("{case}.sifr"));
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(err) => {
                failures.push(format!("FAIL [{}]: unable to read fixture {}: {}", case, path.display(), err));
                continue;
            }
        };

        let (rust_source, _stdlib_modules, stats) = match compile_source_with_metadata_mode_and_stats(
            &source,
            sifr_driver::CodegenLoweringMode::StructuredPreferred,
        ) {
            Ok(result) => result,
            Err(errors) => {
                failures.push(format!(
                    "FAIL [{}]: structured compile failed:\n  {}",
                    case,
                    errors.join("\n  ")
                ));
                continue;
            }
        };

        if !rust_source.contains("fn main()") {
            failures.push(format!(
                "FAIL [{}]: generated Rust has no main function",
                case
            ));
            continue;
        }

        eprintln!(
            "  [{}] stmt={}/{} expr={}/{}",
            case,
            stats.stmt_candidate_structured,
            stats.stmt_candidate_total,
            stats.expr_candidate_structured,
            stats.expr_candidate_total
        );

        total_stmt_candidate += stats.stmt_candidate_total;
        total_stmt_candidate_structured += stats.stmt_candidate_structured;
        total_expr_candidate += stats.expr_candidate_total;
        total_expr_candidate_structured += stats.expr_candidate_structured;
    }

    if !failures.is_empty() {
        panic!(
            "\n{} structured-ratio corpus setup failure(s):\n\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }

    assert!(
        total_stmt_candidate > 0,
        "structured ratio gate: stmt_candidate_total must be > 0"
    );
    assert!(
        total_expr_candidate > 0,
        "structured ratio gate: expr_candidate_total must be > 0"
    );

    let stmt_ratio = total_stmt_candidate_structured as f64 / total_stmt_candidate as f64;
    let expr_ratio = total_expr_candidate_structured as f64 / total_expr_candidate as f64;

    assert!(
        stmt_ratio >= 0.80,
        "structured ratio gate failed for statements: {:.3} < 0.80 ({} / {})",
        stmt_ratio,
        total_stmt_candidate_structured,
        total_stmt_candidate
    );
    assert!(
        expr_ratio >= 0.80,
        "structured ratio gate failed for expressions: {:.3} < 0.80 ({} / {})",
        expr_ratio,
        total_expr_candidate_structured,
        total_expr_candidate
    );

    eprintln!(
        "  structured ratio gate passed: stmt={:.3} ({}/{}), expr={:.3} ({}/{})",
        stmt_ratio,
        total_stmt_candidate_structured,
        total_stmt_candidate,
        expr_ratio,
        total_expr_candidate_structured,
        total_expr_candidate
    );
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

#[test]
fn test_e2e_runtime_fail() {
    let runtime_fail_dir = Path::new("tests/e2e/runtime_fail");
    if !runtime_fail_dir.exists() {
        return;
    }

    let mut entries: Vec<_> = fs::read_dir(runtime_fail_dir)
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
        let expected_stderr = extract_expect_stderr(&source);

        let (rust_source, used_stdlib_modules) = match compile_source_with_metadata(&source) {
            Ok(result) => result,
            Err(errors) => {
                failures.push(format!(
                    "FAIL [{}]: sifr compilation failed (runtime-fail tests must compile):\n  {}",
                    test_name,
                    errors.join("\n  ")
                ));
                continue;
            }
        };

        match build_and_run_capture_with_deps(&rust_source, &test_name, &used_stdlib_modules) {
            Ok((_stdout, stderr, success)) => {
                if success {
                    failures.push(format!(
                        "FAIL [{}]: expected runtime failure but binary exited successfully",
                        test_name
                    ));
                    continue;
                }

                for expected in &expected_stderr {
                    if !stderr.contains(expected) {
                        failures.push(format!(
                            "FAIL [{}]: expected stderr containing {:?} but got:\n{}",
                            test_name, expected, stderr
                        ));
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
            "\n{} E2E runtime-fail test(s) failed:\n\n{}\n\n({} passed, {} failed)",
            failures.len(),
            failures.join("\n\n"),
            test_count,
            failures.len()
        );
    }

    assert!(test_count > 0, "No runtime_fail tests found");
    eprintln!("  {} runtime_fail tests completed", test_count);
}
