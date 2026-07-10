#![allow(clippy::expect_used)]

use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &str = include_str!("e2e/pass/runtime_diagnostics_tracing.sifr");

struct TestProject {
    root: PathBuf,
    main: PathBuf,
}

impl TestProject {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should move forward")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "sifr_runtime_observability_boundary_{}_{}",
            std::process::id(),
            unique
        ));
        std::fs::create_dir_all(&root).expect("temp project should be created");
        let main = root.join("main.sifr");
        std::fs::write(&main, FIXTURE).expect("fixture should be copied");
        Self { root, main }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sifr"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("sifr command should run")
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn assert_success(output: &Output, command: &str) {
    assert!(
        output.status.success(),
        "sifr {command} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn standalone_runtime_diagnostics_use_only_the_stdlib_boundary() {
    let project = TestProject::new();
    let main = project.main.to_string_lossy().into_owned();

    let checked = project.run(&["check", &main]);
    assert_success(&checked, "check");

    let emitted = project.run(&["emit", &main]);
    assert_success(&emitted, "emit");
    let rust = String::from_utf8_lossy(&emitted.stdout);
    assert!(rust.contains("sifr_stdlib::runtime_observability::emit_diagnostic"));
    assert!(!rust.contains("metrics::"));
    assert!(!rust.contains("tracing::"));

    let output_dir = project.root.join("build-output");
    let output_dir_arg = output_dir.to_string_lossy().into_owned();
    let built = project.run(&["build", "--quiet", &main, "-o", &output_dir_arg]);
    assert_success(&built, "build");

    let ran = project.run(&["run", "--quiet", &main]);
    assert_success(&ran, "run");
}
