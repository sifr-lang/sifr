use crate::check_and_package_commands::{check_entrypoint, compile_entrypoint, emit_entrypoint};
use crate::cli_model_and_entrypoint::{
    diagnostic_exit_code, diagnostic_with_code, legacy_diagnostic_display, run_with_panic_boundary,
    Cli, CompilationMode, DiagnosticFormat, EXIT_INTERNAL_COMPILER_FAILURE, EXIT_SUCCESS,
    EXIT_USAGE_OR_CONFIG, EXIT_USER_DIAGNOSTIC,
};
use crate::diagnostic_rendering_and_run::{canonical_diagnostic_stream, render_diagnostic_output};
use crate::mode_resolution_tests::{mktemp_dir, resolved_mode, TestProject};
use crate::mode_resolution_tests::{primary_test_span, test_diagnostic};
use clap::Parser;
use sifr_diagnostics::{
    render_compact_diagnostics, ChildSeverity, DiagnosticArg, DiagnosticCode, RenderedDiagnostic,
    Severity,
};
use sifr_driver::CompileResult;
use std::fmt::Write as _;
#[test]
pub(super) fn test_resolve_compilation_mode_single_file_for_multi_level_relative_import() {
    let project = TestProject::new("relative_import_multi_level");
    let main = project.write(
        "main.sifr",
        "from ..helper import value\n\ndef main():\n    print(value())\n",
        "main file should be written",
    );
    project.write(
        "helper.sifr",
        "def value() -> int:\n    return 1\n",
        "helper file should be written",
    );

    assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
}

#[test]
pub(super) fn test_resolve_compilation_mode_single_file_for_bare_relative_import() {
    let project = TestProject::new("relative_import_bare");
    let main = project.write(
        "main.sifr",
        "from . import value\n\ndef main():\n    print(value)\n",
        "main file should be written",
    );

    assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
}

#[test]
pub(super) fn test_compile_entrypoint_error_consistency_for_project_mode() {
    let dir = mktemp_dir("entrypoint_consistency");
    let main = dir.join("main.sifr");
    let helper = dir.join("helper.sifr");
    std::fs::write(
        &main,
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main file should be written");
    std::fs::write(&helper, "def value(:\n").expect("helper file should be written");

    let run_out = mktemp_dir("run_path");
    let build_out = mktemp_dir("build_path");
    let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
    let build_err = compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
    let run_messages: Vec<String> = run_err.iter().map(legacy_diagnostic_display).collect();
    let build_messages: Vec<String> = build_err.iter().map(legacy_diagnostic_display).collect();
    assert_eq!(run_messages, build_messages);

    let _ = std::fs::remove_dir_all(run_out);
    let _ = std::fs::remove_dir_all(build_out);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_compile_entrypoint_error_consistency_for_import_statement() {
    let dir = mktemp_dir("entrypoint_import_statement");
    let main = dir.join("main.sifr");
    let helper = dir.join("helper.sifr");
    std::fs::write(&main, "import helper\n\ndef main():\n    print(\"ok\")\n")
        .expect("main file should be written");
    std::fs::write(&helper, "def value() -> int:\n    return 1\n")
        .expect("helper file should be written");

    let run_out = mktemp_dir("run_path_import_statement");
    let build_out = mktemp_dir("build_path_import_statement");
    let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
    let build_err = compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
    let run_messages: Vec<String> = run_err.iter().map(legacy_diagnostic_display).collect();
    let build_messages: Vec<String> = build_err.iter().map(legacy_diagnostic_display).collect();
    assert_eq!(run_messages, build_messages);
    assert!(run_messages
        .iter()
        .any(|m| m.contains("unsupported import form: import helper")));

    let _ = std::fs::remove_dir_all(run_out);
    let _ = std::fs::remove_dir_all(build_out);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_compile_entrypoint_error_consistency_for_bare_relative_import() {
    let dir = mktemp_dir("entrypoint_bare_relative");
    let main = dir.join("main.sifr");
    std::fs::write(
        &main,
        "from . import helper\n\ndef main():\n    print(helper)\n",
    )
    .expect("main file should be written");

    let run_out = mktemp_dir("run_path_bare_relative");
    let build_out = mktemp_dir("build_path_bare_relative");
    let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
    let build_err = compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
    let run_messages: Vec<String> = run_err.iter().map(legacy_diagnostic_display).collect();
    let build_messages: Vec<String> = build_err.iter().map(legacy_diagnostic_display).collect();
    assert_eq!(run_messages, build_messages);
    assert!(run_messages
        .iter()
        .any(|m| m.contains("unsupported import form: bare relative import")));

    let _ = std::fs::remove_dir_all(run_out);
    let _ = std::fs::remove_dir_all(build_out);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_compile_entrypoint_error_consistency_for_multi_level_relative_import() {
    let dir = mktemp_dir("entrypoint_multi_level_relative");
    let main = dir.join("main.sifr");
    std::fs::write(
        &main,
        "from ..helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main file should be written");

    let run_out = mktemp_dir("run_path_multi_level_relative");
    let build_out = mktemp_dir("build_path_multi_level_relative");
    let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
    let build_err = compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
    let run_messages: Vec<String> = run_err.iter().map(legacy_diagnostic_display).collect();
    let build_messages: Vec<String> = build_err.iter().map(legacy_diagnostic_display).collect();
    assert_eq!(run_messages, build_messages);
    assert!(run_messages
        .iter()
        .any(|m| m.contains("unsupported import form: relative import level 2")));

    let _ = std::fs::remove_dir_all(run_out);
    let _ = std::fs::remove_dir_all(build_out);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_check_entrypoint_project_mode_resolves_local_imports() {
    let dir = mktemp_dir("check_entrypoint_project_imports");
    let main = dir.join("main.sifr");
    let helper = dir.join("helper.sifr");
    std::fs::write(
        &main,
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main file should be written");
    std::fs::write(&helper, "def value() -> int:\n    return 42\n")
        .expect("helper file should be written");

    let errors = check_entrypoint(&main);
    assert!(
        errors.is_empty(),
        "project-aware check should succeed for valid local imports: {errors:?}"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_check_entrypoint_single_file_reveal_type_is_structured_spanned_note() {
    let dir = mktemp_dir("check_entrypoint_single_reveal_type");
    let main = dir.join("main.sifr");
    std::fs::write(&main, "def main():\n    reveal_type(1)\n")
        .expect("main file should be written");

    let diagnostics = check_entrypoint(&main);
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.code, DiagnosticCode::TYPE_REVEAL_TYPE.code());
    assert_eq!(diagnostic.severity, Severity::Note);
    assert_eq!(
        diagnostic.message_template,
        "revealed type is {revealed_type}"
    );
    assert_eq!(
        diagnostic.args.get("revealed_type"),
        Some(&DiagnosticArg::String("int".to_string()))
    );

    let primary_span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("reveal_type diagnostic should carry a primary span");
    assert_eq!(
        primary_span.file.as_deref(),
        Some(main.to_string_lossy().as_ref())
    );
    assert_eq!(primary_span.line, Some(2));
    assert!(
        primary_span.byte_end > primary_span.byte_start,
        "reveal_type primary span should cover source bytes"
    );
    assert_eq!(diagnostic_exit_code(&diagnostics), EXIT_SUCCESS);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_check_entrypoint_single_file_arithmetic_warning_is_structured_spanned_warning() {
    let dir = mktemp_dir("check_entrypoint_single_arithmetic_warning");
    let main = dir.join("main.sifr");
    std::fs::write(
            &main,
            "def multiply(a: int, b: int) -> int:\n    return a * b\n\ndef main():\n    print(multiply(2, 3))\n",
        )
        .expect("main file should be written");

    let diagnostics = check_entrypoint(&main);
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::TYPE_ARITHMETIC_OVERFLOW_RISK.code()
    );
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(
        diagnostic.message_template,
        "integer {operation} may overflow at runtime"
    );
    assert_eq!(
        diagnostic.args.get("operation"),
        Some(&DiagnosticArg::String("multiplication".to_string()))
    );

    let primary_span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("arithmetic warning should carry a primary span");
    assert_eq!(
        primary_span.file.as_deref(),
        Some(main.to_string_lossy().as_ref())
    );
    assert_eq!(primary_span.line, Some(2));
    assert_eq!(diagnostic_exit_code(&diagnostics), EXIT_SUCCESS);

    let human = render_diagnostic_output(&diagnostics, DiagnosticFormat::Human)
        .expect("human warning diagnostics should render");
    assert!(
        human.contains("warning[SIFR-TYPE-0901]: integer multiplication may overflow at runtime")
    );
    assert!(human.contains(&format!("  --> {}:2:", main.display())));
    assert!(human.contains("^"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_check_entrypoint_single_file_unreachable_statement_warning_is_structured() {
    let dir = mktemp_dir("check_entrypoint_single_unreachable_warning");
    let main = dir.join("main.sifr");
    std::fs::write(
        &main,
        "def value() -> int:\n    return 1\n    return 2\n\ndef main():\n    print(value())\n",
    )
    .expect("main file should be written");

    let diagnostics = check_entrypoint(&main);
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::FLOW_UNREACHABLE_STATEMENT.code()
    );
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(diagnostic.message_template, "unreachable statement ignored");
    assert!(diagnostic.args.is_empty());
    let primary_span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("unreachable warning should carry a primary span");
    assert_eq!(
        primary_span.file.as_deref(),
        Some(main.to_string_lossy().as_ref())
    );
    assert_eq!(primary_span.line, Some(3));
    assert_eq!(diagnostic_exit_code(&diagnostics), EXIT_SUCCESS);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_check_entrypoint_reveal_type_notes_obey_recovery_cap() {
    let dir = mktemp_dir("check_entrypoint_reveal_type_cap");
    let main = dir.join("main.sifr");
    let mut source = String::new();
    for index in 0..60 {
        let _ = writeln!(source, "class T{index}:");
        let _ = writeln!(source, "    pass");
        let _ = writeln!(source);
    }
    let _ = writeln!(source, "def main():");
    for index in 0..60 {
        let _ = writeln!(source, "    reveal_type(T{index}())");
    }
    std::fs::write(&main, source).expect("main file should be written");

    let diagnostics = check_entrypoint(&main);
    assert_eq!(diagnostics.len(), 60);
    assert_eq!(diagnostic_exit_code(&diagnostics), EXIT_SUCCESS);

    let canonical = canonical_diagnostic_stream(&diagnostics);
    assert_eq!(canonical.len(), 50);
    assert_eq!(
        canonical
            .iter()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::TYPE_REVEAL_TYPE.code())
            .count(),
        49
    );
    let summary = canonical
        .last()
        .expect("recovery cap should append an omission summary");
    assert_eq!(
        summary.code,
        DiagnosticCode::INTERNAL_RECOVERY_OMISSION_SUMMARY.code()
    );
    // The summary occupies the final display slot, so 60 raw notes become
    // 49 explicit notes plus one summary for the 11 omitted notes.
    assert_eq!(
        summary.message,
        "11 additional reveal_type results omitted by recovery cap (top-level diagnostic stream)"
    );
    assert_eq!(
        summary.args.get("omitted_count"),
        Some(&DiagnosticArg::Unsigned(11))
    );
    assert_eq!(
        summary.args.get("omitted_kind"),
        Some(&DiagnosticArg::String("reveal_type results".to_string()))
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_check_entrypoint_project_mode_error_parity_with_compile_entrypoint() {
    let dir = mktemp_dir("check_entrypoint_error_parity");
    let main = dir.join("main.sifr");
    let helper = dir.join("helper.sifr");
    std::fs::write(
        &main,
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main file should be written");
    std::fs::write(&helper, "def value() -> int:\n    return \"bad\"\n")
        .expect("helper file should be written");

    let check_errors = check_entrypoint(&main);
    let build_out = mktemp_dir("check_entrypoint_build_out");
    let build_errors = compile_entrypoint(&main, &build_out)
        .err()
        .expect("build path should fail for helper type mismatch");

    let check_messages: Vec<String> = check_errors.iter().map(legacy_diagnostic_display).collect();
    let build_messages: Vec<String> = build_errors.iter().map(legacy_diagnostic_display).collect();
    assert_eq!(check_messages, build_messages);
    assert!(check_messages
        .iter()
        .any(|m| m.contains("[helper] return type mismatch")));

    let _ = std::fs::remove_dir_all(build_out);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_compile_entrypoint_single_file_ignores_unrelated_sibling_parse_errors() {
    let dir = mktemp_dir("single_file_sibling_isolation");
    let main = dir.join("main.sifr");
    let output = mktemp_dir("single_file_sibling_isolation_out");
    std::fs::write(&main, "def main():\n    print(\"solo\")\n")
        .expect("main file should be written");
    std::fs::write(dir.join("scratch.sifr"), "def broken(:\n")
        .expect("unrelated sibling should be written");

    let binary = compile_entrypoint(&main, &output)
        .expect("single-file build should ignore unrelated sibling parse errors");
    assert!(binary.exists());

    let _ = std::fs::remove_dir_all(output);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_compile_entrypoint_non_main_input_stays_single_file() {
    let dir = mktemp_dir("non_main_single_file_boundary");
    let app = dir.join("app.sifr");
    let output = mktemp_dir("non_main_single_file_boundary_out");
    std::fs::write(&app, "def main():\n    print(\"app\")\n").expect("app should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("project-like main should be written");
    std::fs::write(dir.join("helper.sifr"), "def value(:\n").expect("helper should be written");

    let binary = compile_entrypoint(&app, &output).expect("non-main entry should stay single-file");
    assert!(binary.exists());

    let _ = std::fs::remove_dir_all(output);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_emit_entrypoint_uses_project_mode_for_project_like_main() {
    let dir = mktemp_dir("emit_project_boundary");
    let main = dir.join("main.sifr");
    let helper = dir.join("helper.sifr");
    std::fs::write(
        &main,
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main should be written");
    std::fs::write(&helper, "def value() -> int:\n    return 42\n")
        .expect("helper should be written");

    let check_errors = check_entrypoint(&main);
    assert!(
        check_errors.is_empty(),
        "check should preserve project-mode behavior: {check_errors:?}"
    );

    let emit_result = emit_entrypoint(&main);
    let rust_source = match emit_result {
        CompileResult::Success { rust_source } => rust_source,
        CompileResult::Errors { errors } => {
            panic!("emit should use project mode successfully: {errors:?}")
        }
    };
    assert!(rust_source.contains("// src/main.rs"));
    assert!(rust_source.contains("// src/helper.rs"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_frontend_error_messages_match_across_check_build_and_run_paths() {
    let dir = mktemp_dir("frontend_error_mode_parity");
    let main = dir.join("main.sifr");
    let helper = dir.join("helper.sifr");
    std::fs::write(
        &main,
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main file should be written");
    std::fs::write(&helper, "def value() -> int:\n    return \"bad\"\n")
        .expect("helper file should be written");

    let check_errors = check_entrypoint(&main);
    let run_out = mktemp_dir("frontend_parity_run_out");
    let build_out = mktemp_dir("frontend_parity_build_out");
    let run_errors = compile_entrypoint(&main, &run_out)
        .err()
        .expect("run path should fail on helper type error");
    let build_errors = compile_entrypoint(&main, &build_out)
        .err()
        .expect("build path should fail on helper type error");

    let check_messages: Vec<String> = check_errors.iter().map(legacy_diagnostic_display).collect();
    let run_messages: Vec<String> = run_errors.iter().map(legacy_diagnostic_display).collect();
    let build_messages: Vec<String> = build_errors.iter().map(legacy_diagnostic_display).collect();
    assert_eq!(check_messages, run_messages);
    assert_eq!(run_messages, build_messages);

    let _ = std::fs::remove_dir_all(run_out);
    let _ = std::fs::remove_dir_all(build_out);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
pub(super) fn test_diagnostic_exit_code_rules_user_vs_internal() {
    let user_error = diagnostic_with_code("type mismatch", DiagnosticCode::TYPE_MISMATCH);
    assert_eq!(diagnostic_exit_code(&[user_error]), EXIT_USER_DIAGNOSTIC);

    let reveal_note = test_diagnostic(
        "SIFR-TYPE-0902",
        Severity::Note,
        "revealed type is int",
        None,
        None,
    );
    assert_eq!(diagnostic_exit_code(&[reveal_note]), EXIT_SUCCESS);

    let overflow_warning = test_diagnostic(
        "SIFR-TYPE-0901",
        Severity::Warning,
        "integer addition may overflow at runtime",
        None,
        None,
    );
    assert_eq!(diagnostic_exit_code(&[overflow_warning]), EXIT_SUCCESS);

    let internal_error = diagnostic_with_code(
        "internal compiler panic during single-file code generation: boom",
        DiagnosticCode::INTERNAL_COMPILER_PANIC,
    );
    assert_eq!(
        diagnostic_exit_code(&[internal_error]),
        EXIT_INTERNAL_COMPILER_FAILURE
    );
}

#[test]
pub(super) fn test_diagnostic_format_cli_rejects_unknown_value_with_usage_exit_code() {
    let parse_result = Cli::try_parse_from([
        "sifr",
        "--diagnostic-format",
        "not-a-format",
        "check",
        "main.sifr",
    ]);
    match parse_result {
        Ok(_) => panic!("unknown diagnostic format should fail"),
        Err(error) => assert_eq!(error.exit_code(), EXIT_USAGE_OR_CONFIG),
    }
}

#[test]
pub(super) fn test_diagnostic_format_cli_accepts_compact_value() {
    let parse_result = Cli::try_parse_from([
        "sifr",
        "--diagnostic-format",
        "compact",
        "check",
        "main.sifr",
    ]);
    assert!(parse_result.is_ok(), "compact format should parse");
}

#[test]
pub(super) fn test_format_selection_regression_check_build_run_emit_commands() {
    let cases: &[(&str, &[&str])] = &[
        ("format_selection_regression_check", &["check", "main.sifr"]),
        ("format_selection_regression_build", &["build", "main.sifr"]),
        ("format_selection_regression_run", &["run", "main.sifr"]),
        ("format_selection_regression_emit", &["emit", "main.sifr"]),
    ];
    for (marker, command_args) in cases {
        let mut args = vec!["sifr", "--diagnostic-format", "compact"];
        args.extend_from_slice(command_args);
        let cli = Cli::try_parse_from(args).unwrap_or_else(|error| {
            panic!("{marker}: diagnostic format should route through parser: {error}")
        });
        assert_eq!(cli.diagnostic_format, DiagnosticFormat::Compact, "{marker}");
    }
}

#[test]
pub(super) fn test_run_with_panic_boundary_converts_panic_to_internal_diagnostic() {
    let error = run_with_panic_boundary(
        "internal compiler panic during test boundary",
        || -> usize { panic!("boom") },
    )
    .expect_err("panic should convert to an internal compiler diagnostic");
    assert!(error
        .message
        .contains("internal compiler panic during test boundary: boom"));
    let error = *error;
    assert_eq!(
        diagnostic_exit_code(&[error]),
        EXIT_INTERNAL_COMPILER_FAILURE
    );
}

#[test]
pub(super) fn test_compact_renderer_invariants_one_line_per_diagnostic() {
    let mut diagnostics = Vec::new();
    for idx in 0..8 {
        diagnostics.push(test_diagnostic(
            "SIFR-TYPE-0002",
            Severity::Error,
            "type mismatch: expected 'int', got 'str'",
            Some(primary_test_span("main.sifr", idx + 1, 1)),
            Some("fix assignment type"),
        ));
    }
    let compact = render_compact_diagnostics(&diagnostics);
    let mut lines = compact.lines();
    let first_line = lines.next().expect("compact output should have first line");
    assert_eq!(first_line, "8 errors, 0 warnings, 0 notes");
    assert_eq!(lines.count(), diagnostics.len());
    assert_eq!(compact.matches("E SIFR-TYPE-0002 main.sifr:").count(), 8);
    assert!(!compact.contains("help: "));
    assert!(!compact.contains("url: "));
    assert!(!compact.contains(" (x"));
    assert!(!compact.contains("  at "));
}

#[test]
pub(super) fn test_compact_renderer_never_drops_or_invents_relative_to_json_count() {
    let diagnostics = vec![
        test_diagnostic(
            "SIFR-TYPE-0002",
            Severity::Error,
            "mismatch one",
            None,
            None,
        ),
        test_diagnostic(
            "SIFR-TYPE-0002",
            Severity::Error,
            "mismatch one",
            None,
            None,
        ),
        test_diagnostic("SIFR-PARSE-0002", Severity::Error, "parse fail", None, None),
    ];
    let compact = render_compact_diagnostics(&diagnostics);
    assert_eq!(compact.lines().skip(1).count(), diagnostics.len());
}

#[test]
pub(super) fn test_diagnostic_formats_share_canonical_sorted_capped_stream() {
    let mut diagnostics = Vec::new();
    for idx in (0..49).rev() {
        diagnostics.push(test_diagnostic(
            "SIFR-TYPE-0002",
            Severity::Error,
            &format!("distinct diagnostic {idx:02}"),
            Some(primary_test_span(
                &format!("zzz_distinct_{idx:02}.sifr"),
                1,
                1,
            )),
            None,
        ));
    }
    for idx in (0..8).rev() {
        diagnostics.push(test_diagnostic(
            "SIFR-TYPE-0002",
            Severity::Error,
            "aaa repeated mismatch",
            Some(primary_test_span("aaa_repeated.sifr", idx + 1, 1)),
            None,
        ));
    }

    let canonical = canonical_diagnostic_stream(&diagnostics);
    assert_eq!(canonical.len(), 50);
    assert!(canonical
        .iter()
        .take(5)
        .all(|diagnostic| diagnostic.code == "SIFR-TYPE-0002"
            && diagnostic.message == "aaa repeated mismatch"));
    assert_eq!(canonical[5].code, "SIFR-INTERNAL-0002");
    assert_eq!(
        canonical[5].message,
        "3 additional diagnostics omitted by recovery cap (similar-diagnostic group)"
    );
    assert!(canonical
        .iter()
        .any(|diagnostic| diagnostic.message == "distinct diagnostic 42"));
    assert!(!canonical
        .iter()
        .any(|diagnostic| diagnostic.message == "distinct diagnostic 43"));
    assert_eq!(canonical[49].code, "SIFR-INTERNAL-0002");
    assert_eq!(
        canonical[49].message,
        "6 additional diagnostics omitted by recovery cap (top-level diagnostic stream)"
    );

    let json_output = render_diagnostic_output(&diagnostics, DiagnosticFormat::Json)
        .expect("JSON diagnostics should render");
    let json_diagnostics: Vec<RenderedDiagnostic> =
        serde_json::from_str(&json_output).expect("JSON output should be diagnostic stream");
    assert_eq!(json_diagnostics, canonical);

    let human_output = render_diagnostic_output(&diagnostics, DiagnosticFormat::Human)
        .expect("human diagnostics should render");
    assert!(human_output.contains("error[SIFR-TYPE-0002]: aaa repeated mismatch"));
    assert!(human_output.contains("  --> aaa_repeated.sifr:"));
    assert!(human_output.contains("note[SIFR-INTERNAL-0002]: 6 additional diagnostics omitted"));
    assert!(human_output.contains("  = location: <unavailable>"));

    let compact_output = render_diagnostic_output(&diagnostics, DiagnosticFormat::Compact)
        .expect("compact diagnostics should render");
    let summary = compact_output
        .lines()
        .next()
        .expect("compact output should start with a summary");
    assert_eq!(summary, "48 errors, 0 warnings, 2 notes");
    assert_eq!(compact_output.lines().skip(1).count(), canonical.len());
    assert!(
        compact_output.contains("E SIFR-TYPE-0002 zzz_distinct_42.sifr:1:1 distinct diagnostic 42")
    );
    assert!(!compact_output.contains("distinct diagnostic 43"));
}

#[test]
pub(super) fn test_human_diagnostic_format_renders_child_notes() {
    let mut diagnostic = test_diagnostic(
        "SIFR-PARSE-0002",
        Severity::Error,
        "syntax error: expected expression",
        None,
        None,
    );
    diagnostic
        .children
        .push(sifr_diagnostics::render::RenderedDiagnosticChild {
            severity: ChildSeverity::Note,
            message: "while parsing helper".to_string(),
        });

    let human_output = render_diagnostic_output(&[diagnostic], DiagnosticFormat::Human)
        .expect("human diagnostics should render");
    assert_eq!(
        human_output,
        concat!(
            "error[SIFR-PARSE-0002]: syntax error: expected expression\n",
            "  = location: <unavailable>\n",
            "  = note: while parsing helper\n",
            "  = docs: https://docs.sifr.sh/errors/SIFR-PARSE-0002\n",
        )
    );
}

#[test]
pub(super) fn test_compact_renderer_snapshot_repeated_diagnostics_preserves_order() {
    let mut diagnostics = Vec::new();
    for _ in 0..5 {
        diagnostics.push(test_diagnostic(
            "SIFR-TYPE-0002",
            Severity::Error,
            "type mismatch: expected 'int', got 'str'",
            None,
            None,
        ));
    }
    diagnostics.push(test_diagnostic(
        "SIFR-TYPE-0002",
        Severity::Error,
        "... +3 more similar diagnostics",
        None,
        None,
    ));

    let expected = concat!(
        "6 errors, 0 warnings, 0 notes\n",
        "E SIFR-TYPE-0002 <unknown> type mismatch: expected 'int', got 'str'\n",
        "E SIFR-TYPE-0002 <unknown> type mismatch: expected 'int', got 'str'\n",
        "E SIFR-TYPE-0002 <unknown> type mismatch: expected 'int', got 'str'\n",
        "E SIFR-TYPE-0002 <unknown> type mismatch: expected 'int', got 'str'\n",
        "E SIFR-TYPE-0002 <unknown> type mismatch: expected 'int', got 'str'\n",
        "E SIFR-TYPE-0002 <unknown> ... +3 more similar diagnostics\n",
    );
    assert_eq!(render_compact_diagnostics(&diagnostics), expected);
}

#[test]
pub(super) fn test_compact_renderer_snapshot_multi_severity_group_order() {
    let diagnostics = vec![
        test_diagnostic(
            "SIFR-TYPE-0002",
            Severity::Warning,
            "unused value",
            None,
            Some("remove the assignment"),
        ),
        test_diagnostic(
            "SIFR-PARSE-0002",
            Severity::Error,
            "parse failure",
            None,
            None,
        ),
        test_diagnostic(
            "SIFR-INTERNAL-0002",
            Severity::Note,
            "consider adding a type annotation",
            None,
            None,
        ),
    ];

    let expected = concat!(
        "1 error, 1 warning, 1 note\n",
        "W SIFR-TYPE-0002 <unknown> unused value\n",
        "E SIFR-PARSE-0002 <unknown> parse failure\n",
        "N SIFR-INTERNAL-0002 <unknown> consider adding a type annotation\n",
    );
    assert_eq!(render_compact_diagnostics(&diagnostics), expected);
}
