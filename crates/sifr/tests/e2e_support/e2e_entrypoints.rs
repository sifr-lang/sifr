#![expect(
    clippy::print_stderr,
    reason = "the E2E harness reports qualification progress and failures"
)]

use super::*;
#[test]
pub(crate) fn test_e2e_pass() {
    let config = runner_config();
    let report = run_pass_suite(&config);
    assert_report("pass", &report);
    eprintln!(
        "[sifr-e2e] report_signature={}",
        report_signature("pass", &report)
    );
    eprintln!(
        "  {} pass tests completed ({} passed, {} failed)",
        report.cases.len(),
        report.passed_count(),
        report.failed_count()
    );
}

#[test]
pub(crate) fn test_codegen_corpus_subset_parity() {
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

    let mut test_count = 0usize;
    let mut failures = Vec::new();

    for case in &corpus {
        let path = pass_dir.join(format!("{case}.sifr"));
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(err) => {
                failures.push(format!(
                    "FAIL [{}]: unable to read fixture {}: {}",
                    case,
                    path.display(),
                    err
                ));
                continue;
            }
        };
        let (rust_source, stdlib_modules, required_features, interop) =
            match compile_source_with_metadata(&source) {
                Ok(result) => result,
                Err(errors) => {
                    failures.push(format!(
                        "FAIL [{}]: sifr compilation failed:\n  {}",
                        case,
                        errors.join("\n  ")
                    ));
                    continue;
                }
            };

        if !rust_source.contains("fn main(") {
            failures.push(format!(
                "FAIL [{case}]: generated Rust has no main function"
            ));
            continue;
        }

        if let Err(err) = build_and_run_with_deps(
            &rust_source,
            &format!("{case}_single"),
            &stdlib_modules,
            &required_features,
            &interop,
        ) {
            failures.push(format!("FAIL [{case}]: {err}"));
            continue;
        }

        test_count += 1;
    }

    assert!(
        failures.is_empty(),
        "{} corpus parity test(s) failed:\n\n{}\n\n({} passed, {} failed)",
        failures.len(),
        failures.join("\n\n"),
        test_count,
        failures.len()
    );

    assert_eq!(
        test_count,
        corpus.len(),
        "Not all corpus subset cases executed successfully"
    );
    eprintln!("  {test_count} corpus subset parity tests completed");
}

#[test]
pub(crate) fn test_emit_pass_fixtures_do_not_include_unwrap_or_expect() {
    let pass_dir = Path::new("tests/e2e/pass");
    if !pass_dir.exists() {
        return;
    }

    let mut failures = Vec::new();
    let mut total = 0usize;

    for path in read_dir_file_paths_sorted(pass_dir) {
        if path.extension().and_then(|ext| ext.to_str()) != Some("sifr") {
            continue;
        }

        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(err) => {
                failures.push(format!(
                    "FAIL [{}]: unable to read fixture: {}",
                    path.display(),
                    err
                ));
                continue;
            }
        };

        let (rust_source, _, _, _) = match compile_source_with_metadata(&source) {
            Ok(result) => result,
            Err(errors) => {
                failures.push(format!(
                    "FAIL [{}]: compilation failed:\n  {}",
                    path.display(),
                    errors.join("\n  ")
                ));
                continue;
            }
        };

        let mut forbidden = Vec::new();
        if rust_source.contains(".unwrap(") {
            forbidden.push(".unwrap(");
        }
        if rust_source.contains(".expect(") {
            forbidden.push(".expect(");
        }
        if !forbidden.is_empty() {
            failures.push(format!(
                "FAIL [{}]: emitted forbidden runtime patterns: {}",
                path.display(),
                forbidden.join(", ")
            ));
        }
        total += 1;
    }

    assert!(total > 0, "no pass fixtures were checked");
    assert!(
        failures.is_empty(),
        "{} emitted-code safety failure(s):\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

#[test]
pub(crate) fn test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus() {
    const BASIS_POINTS: u64 = 10_000;
    const MINIMUM_BASIS_POINTS: u64 = 8_000;

    let pass_dir = Path::new("tests/e2e/pass");
    let corpus = ["statement_expression_lowering"];

    let mut total_stmt_candidate = 0_u64;
    let mut total_stmt_candidate_structured = 0_u64;
    let mut total_expr_candidate = 0_u64;
    let mut total_expr_candidate_structured = 0_u64;
    let mut failures = Vec::new();

    for case in &corpus {
        let path = pass_dir.join(format!("{case}.sifr"));
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(err) => {
                failures.push(format!(
                    "FAIL [{}]: unable to read fixture {}: {}",
                    case,
                    path.display(),
                    err
                ));
                continue;
            }
        };

        let (rust_source, _, _, _, stats) = match compile_source_with_metadata_and_stats(&source) {
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

        if !rust_source.contains("fn main(") {
            failures.push(format!(
                "FAIL [{case}]: generated Rust has no main function"
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

    assert!(
        failures.is_empty(),
        "{} structured-ratio corpus setup failure(s):\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );

    assert!(
        total_stmt_candidate > 0,
        "structured ratio gate: stmt_candidate_total must be > 0"
    );
    assert!(
        total_expr_candidate > 0,
        "structured ratio gate: expr_candidate_total must be > 0"
    );

    let stmt_basis_points =
        total_stmt_candidate_structured.saturating_mul(BASIS_POINTS) / total_stmt_candidate;
    let expr_basis_points =
        total_expr_candidate_structured.saturating_mul(BASIS_POINTS) / total_expr_candidate;

    assert!(
        stmt_basis_points >= MINIMUM_BASIS_POINTS,
        "structured ratio gate failed for statements"
    );
    assert!(
        expr_basis_points >= MINIMUM_BASIS_POINTS,
        "structured ratio gate failed for expressions"
    );

    eprintln!(
        "  structured ratio gate passed: stmt={}.{:02}% ({}/{}), expr={}.{:02}% ({}/{})",
        stmt_basis_points / 100,
        stmt_basis_points % 100,
        total_stmt_candidate_structured,
        total_stmt_candidate,
        expr_basis_points / 100,
        expr_basis_points % 100,
        total_expr_candidate_structured,
        total_expr_candidate
    );
}

#[test]
pub(crate) fn test_e2e_fail() {
    let fail_dir = Path::new("tests/e2e/fail");
    if !fail_dir.exists() {
        return;
    }

    let mut fail_cases = Vec::new();
    let mut rules_errors = Vec::new();
    for path in read_dir_file_paths_sorted(fail_dir) {
        let source = std::fs::read_to_string(&path).unwrap();
        match parse_compile_failure_expectations(&source, &path) {
            Ok(expected) => fail_cases.push((path, source, expected)),
            Err(mut errors) => rules_errors.append(&mut errors),
        }
    }
    assert!(
        rules_errors.is_empty(),
        "fail fixture expectation rules violations:\n{}",
        format_expectation_rules_errors(&rules_errors)
    );

    let mut failures = 0usize;
    for (path, source, expected) in fail_cases {
        match compile_source(&source) {
            Ok(rust_source) => {
                panic!(
                    "FAIL test {} should have failed but compiled successfully:\n{}",
                    path.display(),
                    rust_source
                );
            }
            Err(errors) => {
                match_compile_failure_expectations(&expected, &errors).unwrap_or_else(|missing| {
                    panic!(
                        "FAIL {} expected diagnostic code '{}'{} but got:\n{}",
                        path.display(),
                        missing.code,
                        missing
                            .column
                            .map_or_else(String::new, |column| format!(" at column {column}")),
                        compile_failures_to_messages(&errors).join("\n")
                    );
                });
                failures += 1;
            }
        }
    }

    assert!(failures > 0, "No fail tests found");
    eprintln!("  {failures} fail tests completed");
}

#[test]
fn test_receiver_place_representative_diagnostics_populate_declared_args() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("sifr crate should live under the workspace crates directory");
    // Deliberately receiver/place-scoped: generalizing this to every active source fixture
    // exposes many pre-existing argument gaps across unrelated families. The
    // diagnostics owner must complete that separate migration before broadening it.
    let codes = [
        sifr_diagnostics::DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
        sifr_diagnostics::DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION,
        sifr_diagnostics::DiagnosticCode::OWN_UNSUPPORTED_MUTABLE_RECEIVER_PLACE,
        sifr_diagnostics::DiagnosticCode::PROTO_RECEIVER_CONVENTION_MISMATCH,
        sifr_diagnostics::DiagnosticCode::PROTO_FIXED_RECEIVER_VIOLATION,
    ];

    for code in codes {
        let entry = sifr_diagnostics::codes::registry_entry(code.code())
            .expect("active receiver-place diagnostic should be registered");
        assert_eq!(
            entry.state,
            sifr_diagnostics::codes::DiagnosticState::Active,
            "{} should remain active",
            code.code()
        );
        let fixture = entry
            .representative_fixture_path
            .expect("active receiver-place diagnostic should declare a representative fixture");
        let source = std::fs::read_to_string(repo_root.join(fixture))
            .unwrap_or_else(|error| panic!("failed to read {fixture}: {error}"));
        let errors = match sifr_driver::compile(&source) {
            sifr_driver::CompileResult::Errors { errors } => errors,
            sifr_driver::CompileResult::Success { .. } => {
                panic!("{fixture} should emit {}", code.code())
            }
        };
        let diagnostics = errors
            .iter()
            .filter(|diagnostic| diagnostic.code == code.code())
            .collect::<Vec<_>>();
        assert!(
            !diagnostics.is_empty(),
            "{fixture} did not emit {}",
            code.code()
        );

        for diagnostic in diagnostics {
            for declaration in entry.declared_args {
                let arg = diagnostic.args.get(declaration.name).unwrap_or_else(|| {
                    panic!(
                        "{fixture} emitted {} without declared arg {}",
                        code.code(),
                        declaration.name
                    )
                });
                if let sifr_diagnostics::DiagnosticArg::String(value) = arg {
                    assert!(
                        !value.trim().is_empty(),
                        "{fixture} emitted {} with empty declared arg {}",
                        code.code(),
                        declaration.name
                    );
                }
            }
        }
    }
}

#[test]
fn test_fixed_receiver_diagnostics_survive_similar_recovery_cap() {
    let source = r#"
class Zulu:
    value: int
    def __eq__(self, other: Zulu) -> bool:
        self.value += 1
        return self.value == other.value

class Bravo:
    value: int
    def __eq__(self, other: Bravo) -> bool:
        self.value += 1
        return self.value == other.value

class Charlie:
    value: int
    def __eq__(self, other: Charlie) -> bool:
        self.value += 1
        return self.value == other.value

class Delta:
    value: int
    def __eq__(self, other: Delta) -> bool:
        self.value += 1
        return self.value == other.value

class Echo:
    value: int
    def __eq__(self, other: Echo) -> bool:
        self.value += 1
        return self.value == other.value

class Foxtrot:
    value: int
    def __eq__(self, other: Foxtrot) -> bool:
        self.value += 1
        return self.value == other.value
"#;
    let errors = match sifr_driver::compile(source) {
        sifr_driver::CompileResult::Errors { errors } => errors,
        sifr_driver::CompileResult::Success { .. } => {
            panic!("fixed receiver mutations should fail")
        }
    };
    let bounded = sifr_driver::apply_diagnostic_recovery_limits(&errors);
    let class_names = bounded
        .iter()
        .filter(|diagnostic| diagnostic.code == "SIFR-PROTO-0006")
        .map(|diagnostic| match diagnostic.args.get("class_name") {
            Some(sifr_diagnostics::DiagnosticArg::String(class_name)) => class_name.as_str(),
            other => panic!("class_name argument should be populated: {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        class_names,
        ["Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Zulu"]
    );
}

#[test]
pub(crate) fn test_decimal_fail_fixtures_do_not_emit_retired_pseudo_codes() {
    let fail_dir = Path::new("tests/e2e/fail");
    if !fail_dir.exists() {
        return;
    }

    let mut checked = 0usize;
    for path in read_dir_file_paths_sorted(fail_dir) {
        let source = std::fs::read_to_string(&path).unwrap();
        if !source.contains("SIFR-DECIMAL-") {
            continue;
        }

        let errors = compile_source(&source).expect_err("decimal fail fixture should fail");
        assert!(
            errors.iter().all(
                |failure| !failure.code.starts_with("E25") && !failure.message.contains("[E25")
            ),
            "decimal fixture {} emitted a retired pseudo-code:\n{}",
            path.display(),
            compile_failures_to_messages(&errors).join("\n")
        );
        checked += 1;
    }

    assert!(checked > 0, "No decimal fail fixtures found");
}

#[test]
pub(crate) fn test_e2e_runtime_fail() {
    let runtime_fail_dir = Path::new("tests/e2e/runtime_fail");
    if !runtime_fail_dir.exists() {
        return;
    }

    let mut failures = Vec::new();
    let mut total = 0usize;

    for path in read_dir_file_paths_sorted(runtime_fail_dir) {
        let test_name = path_to_name(&path);
        let source = std::fs::read_to_string(&path).unwrap();
        let expected_stderr = extract_expect_stderr(&source);

        let (rust_source, used_stdlib_modules, required_features, interop) =
            match compile_source_with_metadata(&source) {
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

        match build_and_run_capture_with_deps(
            &rust_source,
            &test_name,
            &used_stdlib_modules,
            &required_features,
            &interop,
        ) {
            Ok((_stdout, stderr, success)) => {
                if success {
                    failures.push(format!(
                        "FAIL [{test_name}]: expected runtime failure but binary exited successfully"
                    ));
                    continue;
                }

                for expected in &expected_stderr {
                    if !stderr.contains(expected) {
                        failures.push(format!(
                            "FAIL [{test_name}]: expected stderr containing {expected:?} but got:\n{stderr}"
                        ));
                    }
                }
            }
            Err(err) => {
                failures.push(format!("FAIL [{test_name}]: {err}"));
                continue;
            }
        }

        total += 1;
    }

    assert!(
        failures.is_empty(),
        "{} E2E runtime-fail test(s) failed:\n\n{}\n\n({} passed, {} failed)",
        failures.len(),
        failures.join("\n\n"),
        total,
        failures.len()
    );

    assert!(total > 0, "No runtime_fail tests found");
    eprintln!("  {total} runtime_fail tests completed");
}

pub(crate) fn read_dir_file_paths_sorted(directory: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(directory)
        .into_iter()
        .flat_map(|dir| dir.filter_map(Result::ok))
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "sifr"))
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    paths
}

#[test]
pub(crate) fn test_cache_root_from_env_resolution() {
    assert_eq!(
        cache_root_from_env(None),
        Path::new(E2E_CACHE_DIR).to_path_buf()
    );
    assert_eq!(
        cache_root_from_env(Some("")),
        Path::new(E2E_CACHE_DIR).to_path_buf()
    );
    assert_eq!(
        cache_root_from_env(Some("   ")),
        Path::new(E2E_CACHE_DIR).to_path_buf()
    );
    assert_eq!(
        cache_root_from_env(Some("target/custom_cache_root")),
        PathBuf::from("target/custom_cache_root")
    );
}

#[test]
pub(crate) fn test_expectation_parsing_rules() {
    let source = [
        "# expect-stderr: err-1",
        "# expect-stderr: err-2",
        "# expect-error: SIFR-PARSE-0002",
        "# expect-error: SIFR-TYPE-0002",
        "# expect-error[col=7]: SIFR-DECIMAL-0007",
    ]
    .join("\n");

    assert_eq!(
        extract_expect_stderr(&source),
        vec!["err-1".to_string(), "err-2".to_string()]
    );
    let expected = extract_compile_failure_expectations(&source, Path::new("unit.sifr"));
    assert_eq!(
        expected,
        vec![
            CompileFailureExpectation {
                code: "SIFR-PARSE-0002".to_string(),
                column: None,
            },
            CompileFailureExpectation {
                code: "SIFR-TYPE-0002".to_string(),
                column: None,
            },
            CompileFailureExpectation {
                code: "SIFR-DECIMAL-0007".to_string(),
                column: Some(7),
            },
        ]
    );
}

#[test]
pub(crate) fn test_expected_error_rules_accepts_canonical_codes_and_columns() {
    let parsed = parse_expected_error("SIFR-TYPE-0002").unwrap();
    assert_eq!(parsed.code, "SIFR-TYPE-0002");
    assert_eq!(parsed.column, None);

    let parsed = parse_expect_error_line("# expect-error[col=12]: SIFR-DECIMAL-0007")
        .unwrap()
        .unwrap();
    assert_eq!(parsed.code, "SIFR-DECIMAL-0007");
    assert_eq!(parsed.column, Some(12));
}

#[test]
pub(crate) fn test_expected_error_rules_rejects_messages_retired_and_unknown_codes() {
    let message_error = parse_expected_error("SIFR-TYPE-0002: assignment to immutability")
        .expect_err("message substrings must be rejected");
    assert!(message_error.contains("message substrings are not accepted"));

    let retired_error =
        parse_expected_error("[E2507]").expect_err("retired pseudo-codes must be rejected");
    assert!(retired_error.contains("retired pseudo-code"));

    let unknown_error =
        parse_expected_error("SIFR-TYPE-9999").expect_err("unknown codes must be rejected");
    assert!(unknown_error.contains("unknown diagnostic code 'SIFR-TYPE-9999'"));
    assert!(
        unknown_error.contains("did you mean SIFR-"),
        "unknown-code error should include closest active-code hint: {unknown_error}"
    );

    let reserved_error =
        parse_expected_error("SIFR-INTERNAL-0000").expect_err("reserved codes must be rejected");
    assert!(reserved_error.contains("Reserved"));
}

#[test]
pub(crate) fn test_expected_error_rules_rejects_malformed_grammar() {
    let empty_error = parse_expected_error("").expect_err("empty payload must be rejected");
    assert!(empty_error.contains("expected a diagnostic code"));

    let shape_error = parse_expected_error("SIFR-").expect_err("invalid shape must be rejected");
    assert!(shape_error.contains("expected canonical SIFR-<FAMILY>-dddd code"));

    let bracket_error = parse_expected_error("[SIFR-TYPE-0002]")
        .expect_err("bracketed canonical code must be rejected");
    assert!(bracket_error.contains("expected canonical SIFR-<FAMILY>-dddd code"));

    let missing_close = parse_expect_error_line("# expect-error[col=12 SIFR-TYPE-0002").unwrap();
    assert!(
        missing_close
            .expect_err("malformed qualifier must be rejected")
            .contains("expected expect-error qualifier syntax")
    );

    let unknown_qualifier =
        parse_expect_error_line("# expect-error[line=3]: SIFR-TYPE-0002").unwrap();
    assert!(
        unknown_qualifier
            .expect_err("unknown qualifier must be rejected")
            .contains("unknown expect-error qualifier")
    );

    let invalid_column = parse_expect_error_line("# expect-error[col=0]: SIFR-TYPE-0002").unwrap();
    assert!(
        invalid_column
            .expect_err("non-positive column must be rejected")
            .contains("invalid expect-error column")
    );
}

#[test]
pub(crate) fn test_expected_error_rules_rejects_contradictory_overlapping_locations() {
    let same_location = vec![
        LocatedCompileFailureExpectation {
            line_number: 12,
            expectation: CompileFailureExpectation {
                code: "SIFR-TYPE-0002".to_string(),
                column: Some(4),
            },
        },
        LocatedCompileFailureExpectation {
            line_number: 12,
            expectation: CompileFailureExpectation {
                code: "SIFR-NAME-0001".to_string(),
                column: Some(4),
            },
        },
    ];
    let errors = validate_expectation_contradictions(&same_location)
        .expect_err("same assertion column cannot claim incompatible codes");
    let err = errors.join("\n");
    assert!(err.contains("contradictory expect-error markers"));
    assert!(err.contains("marker line 12"));
    assert!(err.contains("for column 4"));
    assert!(err.contains("SIFR-TYPE-0002"));
    assert!(err.contains("SIFR-NAME-0001"));

    let extracted_errors = parse_compile_failure_expectations(
        "# expect-error[col=4]: SIFR-TYPE-0002\n# expect-error[col=4]: SIFR-NAME-0001\n",
        Path::new("unit.sifr"),
    )
    .expect_err("extractor must reject real contradictory marker lines");
    let extracted_error = extracted_errors.join("\n");
    assert!(extracted_error.contains("marker line 1"));
    assert!(extracted_error.contains("marker line 2"));
    assert!(extracted_error.contains("for column 4"));

    let multiple_errors = parse_compile_failure_expectations(
        "\
# expect-error[col=4]: SIFR-TYPE-0002
# expect-error[col=4]: SIFR-NAME-0001
# expect-error[col=9]: SIFR-TYPE-0002
# expect-error[col=9]: SIFR-NAME-0001
",
        Path::new("unit.sifr"),
    )
    .expect_err("extractor must accumulate all contradiction errors");
    assert_eq!(multiple_errors.len(), 2);

    let unqualified_marker_does_not_claim_column = vec![
        LocatedCompileFailureExpectation {
            line_number: 5,
            expectation: CompileFailureExpectation {
                code: "SIFR-TYPE-0002".to_string(),
                column: None,
            },
        },
        LocatedCompileFailureExpectation {
            line_number: 5,
            expectation: CompileFailureExpectation {
                code: "SIFR-NAME-0001".to_string(),
                column: Some(9),
            },
        },
    ];
    assert!(validate_expectation_contradictions(&unqualified_marker_does_not_claim_column).is_ok());

    let unqualified_markers_do_not_conflict = vec![
        LocatedCompileFailureExpectation {
            line_number: 5,
            expectation: CompileFailureExpectation {
                code: "SIFR-TYPE-0002".to_string(),
                column: None,
            },
        },
        LocatedCompileFailureExpectation {
            line_number: 6,
            expectation: CompileFailureExpectation {
                code: "SIFR-NAME-0001".to_string(),
                column: None,
            },
        },
    ];
    assert!(validate_expectation_contradictions(&unqualified_markers_do_not_conflict).is_ok());

    let disjoint_columns = vec![
        LocatedCompileFailureExpectation {
            line_number: 5,
            expectation: CompileFailureExpectation {
                code: "SIFR-TYPE-0002".to_string(),
                column: Some(4),
            },
        },
        LocatedCompileFailureExpectation {
            line_number: 5,
            expectation: CompileFailureExpectation {
                code: "SIFR-NAME-0001".to_string(),
                column: Some(9),
            },
        },
    ];
    assert!(validate_expectation_contradictions(&disjoint_columns).is_ok());

    let repeated_same_code = vec![
        LocatedCompileFailureExpectation {
            line_number: 8,
            expectation: CompileFailureExpectation {
                code: "SIFR-TYPE-0002".to_string(),
                column: None,
            },
        },
        LocatedCompileFailureExpectation {
            line_number: 8,
            expectation: CompileFailureExpectation {
                code: "SIFR-TYPE-0002".to_string(),
                column: Some(9),
            },
        },
    ];
    assert!(validate_expectation_contradictions(&repeated_same_code).is_ok());
}
