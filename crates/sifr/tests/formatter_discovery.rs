//! Exercise discovery through the real CLI, including a known drift sentinel.

use std::path::Path;
use std::process::{Command, Output};

fn check(root: &Path, target: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sifr"))
        .current_dir(root)
        .args(["fmt", "--check", "--no-cache"])
        .args(extra)
        .arg(target)
        .output()
        .unwrap()
}

#[test]
fn formatter_discovery_absolute_and_relative_paths_detect_the_same_drift() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("tmp/checkout");
    let project = root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(root.join(".gitignore"), "/tmp/\n").unwrap();
    std::fs::write(project.join("main.sifr"), "def main( ):\n    pass\n").unwrap();
    for target in [project.as_path(), Path::new("project")] {
        let result = check(&root, target, &[]);
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("SIFR-FMT-0001"));
    }
}

#[test]
fn formatter_discovery_preserves_explicit_file_and_exclusion_controls() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let project = root.join("ignored");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(root.join(".gitignore"), "/ignored/\n").unwrap();
    let file = project.join("main.sifr");
    std::fs::write(&file, "def main( ):\n    pass\n").unwrap();
    for (target, args, expected) in [
        (project.as_path(), vec![], 0),
        (project.as_path(), vec!["--no-respect-gitignore"], 1),
        (file.as_path(), vec![], 1),
        (file.as_path(), vec!["--force-exclude"], 0),
    ] {
        let result = check(root, target, &args);
        assert_eq!(result.status.code(), Some(expected), "{result:?}");
    }
}
