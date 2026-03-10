use crate::{compose_test_runner_lib, generate_test_runner_cargo_toml, run_tests};
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

    let first_messages: Vec<String> = run_tests(&test_dir)
        .err()
        .expect("parse errors should be reported")
        .into_iter()
        .map(|error| error.message)
        .collect();
    let second_messages: Vec<String> = run_tests(&test_dir)
        .err()
        .expect("parse errors should be deterministic")
        .into_iter()
        .map(|error| error.message)
        .collect();

    assert_eq!(first_messages, second_messages);
    assert!(
        first_messages
            .first()
            .is_some_and(|message| message.contains("test_a_bad.sifr")),
        "first parse error should be from lexicographically first fixture: {first_messages:?}"
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
        "from helper import value\n\ndef test_bad() -> int:\n    return \"bad\"\n",
    )
    .expect("bad test module should be written");

    let errors = run_tests(&test_dir)
        .err()
        .expect("type errors in test module should fail frontend");
    let messages: Vec<String> = errors.iter().map(|error| error.message.clone()).collect();
    assert!(messages
        .iter()
        .all(|message| message.contains("test_bad.sifr")));
    assert!(messages
        .iter()
        .all(|message| !message.contains("] [test_bad] return type mismatch")));

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_generate_test_runner_cargo_toml_includes_required_crates() {
    let stdlib_modules = HashSet::new();
    let required_crates = HashSet::from([
        "regex".to_string(),
        "rand".to_string(),
        "rand_distr".to_string(),
    ]);

    let cargo_toml = generate_test_runner_cargo_toml(&stdlib_modules, &required_crates);
    assert!(cargo_toml.contains("name = \"sifr_tests\""));
    assert!(cargo_toml.contains("regex = \"1\""));
    assert!(cargo_toml.contains("rand = \"0.8\""));
    assert!(cargo_toml.contains("rand_distr = \"0.4\""));
}

#[test]
fn test_generate_test_runner_cargo_toml_preserves_stdlib_deps() {
    let stdlib_modules = HashSet::from(["sifr.json".to_string()]);
    let required_crates = HashSet::new();

    let cargo_toml = generate_test_runner_cargo_toml(&stdlib_modules, &required_crates);
    assert!(cargo_toml.contains("serde_json = \"1\""));
    assert!(cargo_toml.contains("serde = { version = \"1\", features = [\"derive\"] }"));
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
