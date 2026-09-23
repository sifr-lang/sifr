#![allow(clippy::expect_used)]

use std::path::Path;
use std::process::{Command, Output};

fn run_sifr(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args(args)
        .current_dir(cwd)
        .env("SIFR_CACHE_DIR", root.join("cache"))
        .output()
        .expect("Sifr CLI should start")
}

fn assert_success(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    let output = run_sifr(root, cwd, args);
    assert!(
        output.status.success(),
        "{args:?} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn legal_source_method_names_single_file_cli() {
    let workspace = tempfile::tempdir().expect("single-file workspace");
    let root = workspace.path();
    std::fs::write(
        root.join("main.sifr"),
        include_str!("fixtures/legal_failure_method_names.sifr"),
    )
    .expect("write source");

    assert_success(root, root, &["check", "main.sifr"]);
    let emitted = assert_success(root, root, &["emit", "main.sifr"]);
    let rust = String::from_utf8_lossy(&emitted.stdout);
    assert!(rust.contains(".unwrap("), "{rust}");
    assert!(rust.contains(".expect("), "{rust}");
    assert_success(root, root, &["build", "main.sifr", "-o", "out"]);
    assert_success(root, root, &["run", "main.sifr"]);
}

#[test]
fn legal_source_method_name_inherited_check_emit() {
    let workspace = tempfile::tempdir().expect("inherited-name workspace");
    let root = workspace.path();
    std::fs::write(
        root.join("main.sifr"),
        include_str!("fixtures/legal_failure_method_inherited.sifr"),
    )
    .expect("write source");

    assert_success(root, root, &["check", "main.sifr"]);
    let emitted = assert_success(root, root, &["emit", "main.sifr"]);
    assert!(String::from_utf8_lossy(&emitted.stdout).contains(".expect("));
}

#[test]
fn legal_source_method_names_project_and_test_cli() {
    let workspace = tempfile::tempdir().expect("project workspace");
    let root = workspace.path();
    std::fs::write(root.join("sifr.toml"), "[source]\nroot = \".\"\n")
        .expect("write project manifest");
    std::fs::write(
        root.join("helper.sifr"),
        r#"class Carrier:
    value: int

    def __init__(self, value: int):
        self.value = value

    def unwrap(self) -> int:
        return self.value

    def expect(self, offset: int) -> int:
        return self.value + offset

def unwrap(value: Carrier) -> int:
    return value.unwrap()

def expect(value: Carrier) -> int:
    return value.expect(2)
"#,
    )
    .expect("write imported module");
    std::fs::write(
        root.join("main.sifr"),
        "from helper import Carrier, unwrap, expect\n\ndef main():\n    carrier: Carrier = Carrier(4)\n    assert unwrap(carrier) == 4\n    assert expect(carrier) == 6\n",
    )
    .expect("write project entrypoint");
    std::fs::create_dir(root.join("tests")).expect("create test directory");
    std::fs::write(
        root.join("tests/test_names.sifr"),
        "class Carrier:\n    value: int\n    def __init__(self, value: int):\n        self.value = value\n    def unwrap(self) -> int:\n        return self.value\n    def expect(self, offset: int) -> int:\n        return self.value + offset\n\ndef test_legal_names():\n    carrier: Carrier = Carrier(4)\n    assert carrier.unwrap() == 4\n    assert carrier.expect(2) == 6\n",
    )
    .expect("write test source");

    let invocation_cwd = std::env::temp_dir();
    let main = root.join("main.sifr").to_string_lossy().into_owned();
    let tests = root.join("tests").to_string_lossy().into_owned();
    let output = root.join("out").to_string_lossy().into_owned();
    assert_success(root, &invocation_cwd, &["check", &main]);
    assert_success(root, &invocation_cwd, &["emit", &main]);
    assert_success(root, &invocation_cwd, &["build", &main, "-o", &output]);
    assert_success(root, &invocation_cwd, &["run", &main]);
    assert_success(root, &invocation_cwd, &["test", &tests]);
}
