#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestProject {
    root: PathBuf,
    main: PathBuf,
}

impl TestProject {
    fn new(name: &str, source: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should move forward")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "sifr_build_output_{name}_{}_{}",
            std::process::id(),
            unique
        ));
        std::fs::create_dir_all(&root).expect("temp project should be created");
        let main = root.join("main.sifr");
        std::fs::write(&main, source).expect("main.sifr should be written");
        Self { root, main }
    }

    fn output_dir(&self, name: &str) -> PathBuf {
        let output = self.root.join(name);
        std::fs::create_dir_all(&output).expect("output dir should be created");
        output
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[derive(Debug)]
struct CommandCapture {
    status_code: i32,
    stdout: String,
    stderr: String,
}

fn run_sifr(args: &[&str], cwd: &Path) -> CommandCapture {
    run_sifr_with_env(args, cwd, &[])
}

fn run_sifr_with_env(args: &[&str], cwd: &Path, envs: &[(&str, &str)]) -> CommandCapture {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sifr"));
    command.args(args).current_dir(cwd);
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("sifr command should run");
    CommandCapture {
        status_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

#[test]
fn bare_invocation_shows_available_commands() {
    let project = TestProject::new("bare_invocation", "def main():\n    pass\n");

    let capture = run_sifr(&[], &project.root);

    assert_eq!(capture.status_code, 2);
    assert!(capture.stdout.is_empty());
    assert!(capture.stderr.contains("Usage:"));
    assert!(capture.stderr.contains("--diagnostic-format"));
    assert!(capture.stderr.contains("--explain"));
    assert!(capture.stderr.contains("Commands:"));
    for command in [
        "build", "run", "check", "emit", "fmt", "lint", "lsp", "test",
    ] {
        assert!(capture.stderr.contains(command), "missing {command}");
    }
    assert!(!capture.stderr.contains("SIFR-WORKSPACE-0004"));
    assert!(!capture.stderr.contains("no command provided"));
}

#[test]
fn explain_without_subcommand_still_prints_explanation() {
    let project = TestProject::new("explain_without_subcommand", "def main():\n    pass\n");

    let capture = run_sifr(&["--explain", "SIFR-PACKAGE-0105"], &project.root);

    assert_eq!(capture.status_code, 0);
    assert!(capture.stdout.contains("SIFR-PACKAGE-0105 is retired"));
    assert!(capture.stderr.is_empty());
}

#[test]
fn build_output_default_is_phase_aware_and_stderr_only() {
    let project = TestProject::new("default", "def main():\n    print(\"ok\")\n");
    let output_dir = project.output_dir("out");
    let output_dir_arg = output_dir.to_string_lossy().to_string();
    let main_arg = project.main.to_string_lossy().to_string();

    let capture = run_sifr(&["build", &main_arg, "-o", &output_dir_arg], &project.root);

    assert_eq!(capture.status_code, 0, "stderr:\n{}", capture.stderr);
    assert!(capture.stdout.is_empty());
    assert!(capture.stderr.contains("sifr "));
    assert!(capture.stderr.contains("input:  "));
    assert!(capture.stderr.contains("mode:   single-file"));
    assert!(capture.stderr.contains("target: release native"));
    assert!(capture.stderr.contains("sysroot:"));
    assert!(capture.stderr.contains("toolchain:"));
    assert!(capture.stderr.contains("digest:"));
    assert!(capture.stderr.contains("Loading Sifr standard library"));
    assert!(capture.stderr.contains("Parsing source (1 module)"));
    assert!(capture.stderr.contains("Analyzing 1 module"));
    assert!(capture.stderr.contains("Generating Rust project"));
    assert!(capture.stderr.contains("Materializing Cargo project"));
    assert!(capture.stderr.contains("Building release binary"));
    assert!(capture.stderr.contains("Finished release build in "));
    assert!(capture.stderr.contains("Binary: "));
    assert!(capture.stderr.contains("Size:   "));
    assert!(!capture.stderr.contains("compiled successfully"));
    assert!(!capture.stderr.contains("Compiling sifr_output"));
}

#[test]
fn build_output_project_mode_reports_import_closure_counts() {
    let project = TestProject::new(
        "project",
        "from helper import message\n\ndef main():\n    print(message())\n",
    );
    std::fs::write(project.root.join("sifr.toml"), "[source]\nroot = \".\"\n")
        .expect("workspace manifest should be written");
    std::fs::write(
        project.root.join("helper.sifr"),
        "def message() -> str:\n    return \"ok\"\n",
    )
    .expect("helper should be written");
    let output_dir = project.output_dir("out");
    let output_dir_arg = output_dir.to_string_lossy().to_string();
    let main_arg = project.main.to_string_lossy().to_string();

    let invocation_cwd = std::env::temp_dir();
    let capture = run_sifr(
        &["build", &main_arg, "-o", &output_dir_arg],
        &invocation_cwd,
    );

    assert_eq!(capture.status_code, 0, "stderr:\n{}", capture.stderr);
    assert!(capture.stdout.is_empty());
    assert!(capture.stderr.contains("mode:   project"));
    assert!(
        capture
            .stderr
            .contains("Parsing import closure (2 modules)")
    );
    assert!(capture.stderr.contains("Analyzing 2 modules"));
}

#[test]
fn build_output_quiet_is_two_line_success() {
    let project = TestProject::new("quiet", "def main():\n    print(\"ok\")\n");
    let output_dir = project.output_dir("out");
    let output_dir_arg = output_dir.to_string_lossy().to_string();
    let main_arg = project.main.to_string_lossy().to_string();

    let capture = run_sifr(
        &["build", "--quiet", &main_arg, "-o", &output_dir_arg],
        &project.root,
    );

    assert_eq!(capture.status_code, 0, "stderr:\n{}", capture.stderr);
    assert!(capture.stdout.is_empty());
    let lines = capture.stderr.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2, "stderr:\n{}", capture.stderr);
    assert!(lines[0].starts_with("Finished release build in "));
    assert!(lines[1].starts_with("Binary: "));
}

#[test]
fn build_output_machine_formats_emit_no_success_progress() {
    let project = TestProject::new("machine", "def main():\n    print(\"ok\")\n");
    for format in ["json", "compact"] {
        let output_dir = project.output_dir(format);
        let output_dir_arg = output_dir.to_string_lossy().to_string();
        let main_arg = project.main.to_string_lossy().to_string();

        let capture = run_sifr(
            &[
                "--diagnostic-format",
                format,
                "build",
                &main_arg,
                "-o",
                &output_dir_arg,
            ],
            &project.root,
        );

        assert_eq!(
            capture.status_code, 0,
            "{format} stderr:\n{}",
            capture.stderr
        );
        assert!(
            capture.stdout.is_empty(),
            "{format} stdout:\n{}",
            capture.stdout
        );
        assert!(
            capture.stderr.is_empty(),
            "{format} stderr:\n{}",
            capture.stderr
        );
    }
}

#[test]
fn build_output_compact_warning_success_emits_diagnostics_without_progress() {
    let project = TestProject::new(
        "compact_warning",
        "def value() -> int:\n    return 1\n    return 2\n\ndef main():\n    print(value())\n",
    );
    let output_dir = project.output_dir("out");
    let output_dir_arg = output_dir.to_string_lossy().to_string();
    let main_arg = project.main.to_string_lossy().to_string();

    let capture = run_sifr(
        &[
            "--diagnostic-format",
            "compact",
            "build",
            &main_arg,
            "-o",
            &output_dir_arg,
        ],
        &project.root,
    );

    assert_eq!(capture.status_code, 0, "stderr:\n{}", capture.stderr);
    assert!(capture.stdout.is_empty());
    assert!(capture.stderr.contains("0 errors, 1 warning"));
    assert!(capture.stderr.contains("unreachable statement ignored"));
    assert!(!capture.stderr.contains("Finished release build"));
    assert!(!capture.stderr.contains("Binary: "));
}

#[test]
fn run_output_reports_cache_miss_without_binary_and_suppresses_cache_hit() {
    let project = TestProject::new("run_cache", "def main():\n    print(\"ok\")\n");
    let main_arg = project.main.to_string_lossy().to_string();

    let first = run_sifr(&["run", &main_arg], &project.root);
    assert_eq!(first.status_code, 0, "stderr:\n{}", first.stderr);
    assert_eq!(first.stdout, "ok\n");
    assert!(first.stderr.contains("Finished release build in "));
    assert!(!first.stderr.contains("Binary: "));
    assert!(!first.stderr.contains("[sifr-artifact-cache]"));

    let second = run_sifr(&["run", &main_arg], &project.root);
    assert_eq!(second.status_code, 0, "stderr:\n{}", second.stderr);
    assert_eq!(second.stdout, "ok\n");
    assert!(second.stderr.is_empty(), "stderr:\n{}", second.stderr);
}

#[test]
fn run_output_quiet_suppresses_cache_miss_progress() {
    let project = TestProject::new("run_quiet", "def main():\n    print(\"ok\")\n");
    let main_arg = project.main.to_string_lossy().to_string();

    let capture = run_sifr(&["run", "--quiet", &main_arg], &project.root);

    assert_eq!(capture.status_code, 0, "stderr:\n{}", capture.stderr);
    assert_eq!(capture.stdout, "ok\n");
    assert!(capture.stderr.is_empty(), "stderr:\n{}", capture.stderr);
}

#[test]
fn failed_frontend_build_does_not_print_success_footer() {
    let project = TestProject::new(
        "frontend_failure",
        "def value() -> int:\n    return \"bad\"\n\ndef main():\n    print(value())\n",
    );
    let output_dir = project.output_dir("out");
    let output_dir_arg = output_dir.to_string_lossy().to_string();
    let main_arg = project.main.to_string_lossy().to_string();

    let capture = run_sifr(&["build", &main_arg, "-o", &output_dir_arg], &project.root);

    assert_ne!(capture.status_code, 0);
    assert!(capture.stdout.is_empty());
    assert!(capture.stderr.contains("return type mismatch"));
    assert!(!capture.stderr.contains("Finished release build"));
    assert!(!capture.stderr.contains("Binary: "));
}

#[test]
fn failed_materialization_does_not_print_success_footer() {
    let project = TestProject::new(
        "materialization_failure",
        "def main():\n    print(\"ok\")\n",
    );
    let output_file = project.root.join("not_a_directory");
    std::fs::write(&output_file, "occupied").expect("output file should be written");
    let output_arg = output_file.to_string_lossy().to_string();
    let main_arg = project.main.to_string_lossy().to_string();

    let capture = run_sifr(&["build", &main_arg, "-o", &output_arg], &project.root);

    assert_ne!(capture.status_code, 0);
    assert!(capture.stdout.is_empty());
    assert!(capture.stderr.contains("failed to create output directory"));
    assert!(!capture.stderr.contains("Finished release build"));
    assert!(!capture.stderr.contains("Binary: "));
}

#[test]
fn failed_cargo_invocation_does_not_print_success_footer() {
    let project = TestProject::new("cargo_failure", "def main():\n    print(\"ok\")\n");
    let output_dir = project.output_dir("out");
    let output_dir_arg = output_dir.to_string_lossy().to_string();
    let main_arg = project.main.to_string_lossy().to_string();

    let capture = run_sifr_with_env(
        &["build", &main_arg, "-o", &output_dir_arg],
        &project.root,
        &[("PATH", "")],
    );

    assert_ne!(capture.status_code, 0);
    assert!(capture.stdout.is_empty());
    assert!(
        capture.stderr.contains("SIFR-RUST-CARGO-0001")
            && capture.stderr.contains("failed to run Rust probe"),
        "stderr:\n{}",
        capture.stderr
    );
    assert!(!capture.stderr.contains("Finished release build"));
    assert!(!capture.stderr.contains("Binary: "));
}
