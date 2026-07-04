use crate::{
    build_test_runner_project, compose_test_runner_lib, discover_test_root_modules,
    execute_test_runner_project, generate_test_runner_cargo_toml, run_tests,
};
use sifr_diagnostics::DiagnosticCode;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};

#[test]
fn test_run_tests_resolves_local_imports_and_constants() {
    let unique = format!(
        "sifr_test_import_parity_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let test_dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&test_dir).expect("test dir should be created");

    std::fs::write(
        test_dir.join("helper.sifr"),
        r#"
BASE: int = 9

def plus_one(x: int) -> int:
    return x + 1
"#,
    )
    .expect("helper module should be written");
    std::fs::write(
        test_dir.join("test_imports.sifr"),
        r#"
from helper import BASE, plus_one

def test_import_parity():
    assert plus_one(BASE) == 10
"#,
    )
    .expect("test module should be written");

    let result = run_tests(&test_dir).expect("test runner should compile and execute");
    assert!(result, "sifr test run should succeed");

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_run_tests_resolves_dotted_local_support_modules() {
    let unique = format!(
        "sifr_test_dotted_import_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let test_dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(test_dir.join("helpers")).expect("test helper dir should be created");

    std::fs::write(
        test_dir.join("helpers").join("nodes.sifr"),
        r#"
def value() -> int:
    return 41
"#,
    )
    .expect("dotted helper module should be written");
    std::fs::write(
        test_dir.join("test_dotted_imports.sifr"),
        r#"
from helpers.nodes import value

def test_dotted_import():
    assert value() == 41
"#,
    )
    .expect("test module should be written");

    let result = run_tests(&test_dir).expect("test runner should compile dotted support modules");
    assert!(result, "sifr test run should succeed");

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_run_tests_reuses_cached_workspace_for_unchanged_project() {
    let unique = format!(
        "sifr_test_cache_reuse_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let test_dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&test_dir).expect("test dir should be created");
    std::fs::write(
        test_dir.join("helper.sifr"),
        "def value() -> int:\n    return 7\n",
    )
    .expect("helper should be written");
    std::fs::write(
        test_dir.join("test_cache.sifr"),
        "from helper import value\n\ndef test_value():\n    assert value() == 7\n",
    )
    .expect("test module should be written");

    let discovered = discover_test_root_modules(&test_dir);
    let generated_project =
        build_test_runner_project(&test_dir, &discovered).expect("generated project should build");
    let first = execute_test_runner_project(&generated_project)
        .expect("first test execution should succeed");
    assert!(first.success);
    assert!(!first.cache_report.cache_hit());

    let second = execute_test_runner_project(&generated_project)
        .expect("second test execution should succeed");
    assert!(second.success);
    assert!(second.cache_report.cache_hit());
    assert_eq!(
        first.cache_report.workspace_root(),
        second.cache_report.workspace_root()
    );

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_run_tests_invalidates_cached_workspace_when_sources_change() {
    let unique = format!(
        "sifr_test_cache_invalidation_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let test_dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&test_dir).expect("test dir should be created");
    let helper = test_dir.join("helper.sifr");
    std::fs::write(&helper, "def value() -> int:\n    return 3\n")
        .expect("helper should be written");
    let test_file = test_dir.join("test_cache.sifr");
    std::fs::write(
        &test_file,
        "from helper import value\n\ndef test_value():\n    assert value() == 3\n",
    )
    .expect("test module should be written");

    let first_discovered = discover_test_root_modules(&test_dir);
    let first_project = build_test_runner_project(&test_dir, &first_discovered)
        .expect("generated project should build");
    let first =
        execute_test_runner_project(&first_project).expect("first test execution should succeed");
    assert!(first.success);
    let first_root = first.cache_report.workspace_root().to_path_buf();
    let first_key = first.cache_report.key().to_string();

    std::fs::write(&helper, "def value() -> int:\n    return 4\n").expect("helper should update");
    std::fs::write(
        &test_file,
        "from helper import value\n\ndef test_value():\n    assert value() == 4\n",
    )
    .expect("test module should update");

    let second_discovered = discover_test_root_modules(&test_dir);
    let second_project = build_test_runner_project(&test_dir, &second_discovered)
        .expect("updated generated project should build");
    let second =
        execute_test_runner_project(&second_project).expect("second test execution should succeed");
    assert!(second.success);
    assert!(!second.cache_report.cache_hit());
    assert_ne!(first_root, second.cache_report.workspace_root());
    assert_ne!(first_key, second.cache_report.key());

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_run_tests_parallel_invocations_are_isolated() {
    fn make_test_dir(label: &str, expected: i64) -> PathBuf {
        let unique = format!(
            "sifr_test_parallel_isolation_{label}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let test_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&test_dir).expect("test dir should be created");
        std::fs::write(
            test_dir.join("helper.sifr"),
            format!("def value() -> int:\n    return {expected}\n"),
        )
        .expect("helper should be written");
        std::fs::write(
            test_dir.join("test_parallel.sifr"),
            format!(
                "from helper import value\n\ndef test_value():\n    assert value() == {expected}\n"
            ),
        )
        .expect("test module should be written");
        test_dir
    }

    let first_dir = make_test_dir("first", 11);
    let second_dir = make_test_dir("second", 22);
    let barrier = Arc::new(Barrier::new(3));

    let first_barrier = Arc::clone(&barrier);
    let first_path = first_dir.clone();
    let first = std::thread::spawn(move || {
        first_barrier.wait();
        run_tests(&first_path)
    });

    let second_barrier = Arc::clone(&barrier);
    let second_path = second_dir.clone();
    let second = std::thread::spawn(move || {
        second_barrier.wait();
        run_tests(&second_path)
    });

    barrier.wait();
    let first_result = first.join().expect("first thread should join");
    let second_result = second.join().expect("second thread should join");
    assert!(
        matches!(first_result, Ok(true)),
        "first parallel run_tests invocation should pass: {first_result:?}"
    );
    assert!(
        matches!(second_result, Ok(true)),
        "second parallel run_tests invocation should pass: {second_result:?}"
    );

    let _ = std::fs::remove_dir_all(&first_dir);
    let _ = std::fs::remove_dir_all(&second_dir);
}

#[test]
fn test_run_tests_ignores_unrelated_non_closure_parse_errors() {
    let unique = format!(
        "sifr_test_import_closure_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let test_dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&test_dir).expect("test dir should be created");

    std::fs::write(
        test_dir.join("helper.sifr"),
        "def value() -> int:\n    return 42\n",
    )
    .expect("helper should be written");
    std::fs::write(
        test_dir.join("test_import_closure.sifr"),
        "from helper import value\n\ndef test_value():\n    assert value() == 42\n",
    )
    .expect("test module should be written");
    std::fs::write(test_dir.join("unrelated_bad.sifr"), "def unrelated(:\n")
        .expect("unrelated sibling should be written");

    let result = run_tests(&test_dir).expect("unrelated sibling parse errors should be ignored");
    assert!(result, "sifr test run should succeed");

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_run_tests_reports_deterministic_parse_error_order() {
    let unique = format!(
        "sifr_test_parse_order_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let test_dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&test_dir).expect("test dir should be created");

    std::fs::write(test_dir.join("test_z_bad.sifr"), "def z(:\n")
        .expect("test_z_bad should be written");
    std::fs::write(test_dir.join("test_a_bad.sifr"), "def a(:\n")
        .expect("test_a_bad should be written");

    let first_diagnostics: Vec<(String, Vec<String>)> = run_tests(&test_dir)
        .err()
        .expect("parse errors should be reported")
        .into_iter()
        .map(|error| {
            (
                error.message,
                error
                    .children
                    .into_iter()
                    .map(|child| child.message)
                    .collect(),
            )
        })
        .collect();
    let second_diagnostics: Vec<(String, Vec<String>)> = run_tests(&test_dir)
        .err()
        .expect("parse errors should be deterministic")
        .into_iter()
        .map(|error| {
            (
                error.message,
                error
                    .children
                    .into_iter()
                    .map(|child| child.message)
                    .collect(),
            )
        })
        .collect();

    assert_eq!(first_diagnostics, second_diagnostics);
    assert!(
        first_diagnostics
            .first()
            .is_some_and(|(_message, children)| children
                .iter()
                .any(|child| child.contains("test_a_bad.sifr"))),
        "first parse error should be from lexicographically first fixture: {first_diagnostics:?}"
    );

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_run_tests_frontend_type_errors_use_single_path_prefix() {
    let unique = format!(
        "sifr_test_type_error_prefix_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let test_dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&test_dir).expect("test dir should be created");

    std::fs::write(
        test_dir.join("helper.sifr"),
        "def value() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    std::fs::write(
        test_dir.join("test_bad.sifr"),
        "from helper import value\n\ndef test_bad() -> None:\n    if 1:\n        pass\n",
    )
    .expect("bad test module should be written");

    let errors = run_tests(&test_dir).expect_err("type errors in test module should fail frontend");
    let messages: Vec<String> = errors.iter().map(|error| error.message.clone()).collect();
    assert!(messages
        .iter()
        .all(|message| message.contains("test_bad.sifr")));
    assert!(messages
        .iter()
        .all(|message| !message.contains("] [test_bad] invalid condition type")));
    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::FLOW_INVALID_CONDITION_TYPE.code()),
        "test module frontend diagnostics should preserve semantic code identity: {errors:?}"
    );
    let primary = errors
        .iter()
        .find(|error| error.code == DiagnosticCode::FLOW_INVALID_CONDITION_TYPE.code())
        .and_then(|error| error.spans.iter().find(|span| span.is_primary))
        .expect("test module frontend diagnostic should carry a primary span");
    assert!(
        primary
            .file
            .as_deref()
            .is_some_and(|file| file.ends_with("test_bad.sifr")),
        "primary span should point at the test module source: {primary:?}"
    );
    assert_eq!(primary.line, Some(4));
    assert!(
        errors
            .iter()
            .all(|error| error.code != DiagnosticCode::INTERNAL_COMPILER_PANIC.code()),
        "test module frontend diagnostics must not be reclassified as internal compiler failures: {errors:?}"
    );

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_generate_test_runner_cargo_toml_includes_required_features() {
    let stdlib_modules = HashSet::new();
    let required_features = HashSet::from([
        sifr_stdlib_model::StdlibFeature::Regex,
        sifr_stdlib_model::StdlibFeature::Rand,
        sifr_stdlib_model::StdlibFeature::RandDistr,
        sifr_stdlib_model::StdlibFeature::SifrRuntime,
    ]);

    let cargo_toml = generate_test_runner_cargo_toml(&stdlib_modules, &required_features);
    assert!(cargo_toml.contains("name = \"sifr_tests\""));
    assert!(cargo_toml.contains("regex = \"1.12.3\""));
    assert!(cargo_toml.contains("rand = \"0.10.1\""));
    assert!(cargo_toml.contains("rand_distr = \"0.6.0\""));
    assert!(cargo_toml.contains("sifr_runtime = { path = "));
}

#[test]
fn test_generate_test_runner_cargo_toml_preserves_stdlib_deps() {
    let stdlib_modules = HashSet::from(["sifr.json".to_string()]);
    let required_features = HashSet::new();

    let cargo_toml = generate_test_runner_cargo_toml(&stdlib_modules, &required_features);
    assert!(cargo_toml
        .contains("serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }"));
    assert!(cargo_toml.contains("serde = { version = \"1.0.228\", features = [\"derive\"] }"));
}

#[test]
fn test_generate_test_runner_cargo_toml_uses_stdlib_toml_feature() {
    let stdlib_modules = HashSet::from(["sifr.tomllib".to_string()]);
    let required_features = HashSet::new();

    let cargo_toml = generate_test_runner_cargo_toml(&stdlib_modules, &required_features);
    assert!(cargo_toml.contains("sifr_stdlib = { path = "));
    assert!(cargo_toml.contains("default-features = false"));
    assert!(cargo_toml.contains("features = [\"toml\"]"));
    assert!(!cargo_toml.contains("toml = { version"));
}

#[test]
fn test_compose_test_runner_lib_is_test_scoped() {
    let support_modules = vec!["helper".to_string()];
    let all_rust_code = "#[test]\nfn smoke() {}\n";
    let lib_source = compose_test_runner_lib(&support_modules, all_rust_code);
    assert!(lib_source.starts_with("#![cfg(test)]"));
    assert!(lib_source.contains("mod helper;"));
    assert!(lib_source.contains("#[test]\nfn smoke() {}"));
}

#[test]
fn test_compose_test_runner_lib_declares_dotted_modules_by_namespace() {
    let support_modules = vec![
        "helpers.nodes".to_string(),
        "helpers.tree_node".to_string(),
        "math".to_string(),
    ];
    let all_rust_code = "#[test]\nfn smoke() {}\n";

    let lib_source = compose_test_runner_lib(&support_modules, all_rust_code);

    assert!(lib_source.starts_with("#![cfg(test)]"));
    assert!(lib_source.contains("mod helpers;\n"));
    assert!(lib_source.contains("mod math;\n"));
    assert!(!lib_source.contains("mod helpers.nodes;"));
    assert!(lib_source.contains("#[test]\nfn smoke() {}"));
}
