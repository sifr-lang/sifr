//! Exercise discovery through the real CLI, including a known drift sentinel.

use std::path::Path;
use std::process::{Command, Output};

#[test]
fn formatter_validation_rechecks_no_cache_inputs_and_preserves_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let file = root.join("main.sifr");
    let cache = root.join("format-cache");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_sifr"))
            .current_dir(root)
            .args(["fmt", "--no-cache", "--cache-dir"])
            .arg(&cache)
            .arg(&file)
            .output()
            .unwrap()
    };
    for source in ["def first( ):\n    pass\n", "def second( ):\n    pass\n"] {
        std::fs::write(&file, source).unwrap();
        let output = run();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            source.replace("( )", "()")
        );
        assert!(!cache.exists());
    }
    let invalid = "def broken(:\n";
    std::fs::write(&file, invalid).unwrap();
    assert_eq!(run().status.code(), Some(1));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), invalid);
    assert!(!cache.exists());
}

#[test]
fn formatter_validation_rejects_invalid_source_before_cache_publication() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let source = "value = \"unterminated\n";
    std::fs::write(root.join("main.sifr"), source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .current_dir(root)
        .args(["fmt", "--cache-dir", "format-cache", "main.sifr"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("SIFR-FMT-0001"));
    assert_eq!(
        std::fs::read_to_string(root.join("main.sifr")).unwrap(),
        source
    );
    assert!(!root.join("format-cache").exists());
}

#[test]
fn formatter_discovery_reuses_rules_without_cross_path_decisions() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::create_dir(root.join("project")).unwrap();
    std::fs::write(root.join(".gitignore"), "*.sifr\n![km]eep.sifr\n").unwrap();
    for name in ["keep.sifr", "drop.sifr"] {
        std::fs::write(root.join("project").join(name), "def main( ):\n    pass\n").unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .current_dir(root)
        .args(["fmt", "--no-cache", "project"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        std::fs::read_to_string(root.join("project/keep.sifr")).unwrap(),
        "def main():\n    pass\n"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("project/drop.sifr")).unwrap(),
        "def main( ):\n    pass\n"
    );
    std::fs::write(root.join(".gitignore"), "# ordered\n*.sifr\n{a,b\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .current_dir(root)
        .args(["fmt", "--no-cache", "project/drop.sifr", "project"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains(".gitignore:3:"));
    assert_eq!(
        std::fs::read_to_string(root.join("project/drop.sifr")).unwrap(),
        "def main():\n    pass\n",
        "earlier explicit target is processed before invalid-directory discovery"
    );
}

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

#[test]
fn formatter_discovery_explicit_files_bypass_malformed_ignore_rules() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("main.sifr"), "def main():\n    pass\n").unwrap();
    std::fs::write(root.join("drift.sifr"), "def main( ):\n    pass\n").unwrap();
    for rule in ["{a,b", "[z-a]", "\\"] {
        std::fs::write(root.join(".gitignore"), format!("*.sifr\n{rule}\n")).unwrap();
        for (name, expected) in [("main.sifr", 0), ("drift.sifr", 1)] {
            for target in [root.join(name), name.into()] {
                let result = check(root, &target, &[]);
                assert_eq!(result.status.code(), Some(expected), "{rule:?}: {result:?}");
                let stderr = String::from_utf8_lossy(&result.stderr);
                assert!(!stderr.contains("gitignore"), "{stderr}");
                if expected == 1 {
                    assert!(stderr.contains("SIFR-FMT-0001"), "{stderr}");
                }
            }
        }
    }
}

#[test]
fn formatter_discovery_directory_and_forced_files_report_malformed_ignore_rules() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::create_dir(root.join("empty")).unwrap();
    std::fs::write(root.join("main.sifr"), "def main():\n    pass\n").unwrap();
    for rule in ["{a,b", "[z-a]", "\\"] {
        std::fs::write(root.join(".gitignore"), format!("# rules\n\n{rule}\n")).unwrap();
        for (target, args) in [
            (Path::new("."), vec![]),
            (Path::new("empty"), vec![]),
            (Path::new("main.sifr"), vec!["--force-exclude"]),
        ] {
            let result = check(root, target, &args);
            assert_eq!(result.status.code(), Some(1), "{rule:?}: {result:?}");
            let stderr = String::from_utf8_lossy(&result.stderr);
            assert!(stderr.contains("SIFR-FMT-0001"), "{stderr}");
            assert!(stderr.contains("invalid formatter gitignore"), "{stderr}");
            assert!(stderr.contains(".gitignore:3:"), "{stderr}");
        }
    }
}

#[test]
fn formatter_discovery_no_respect_gitignore_bypasses_malformed_rules() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("main.sifr"), "def main( ):\n    pass\n").unwrap();
    for rule in ["{a,b", "[z-a]", "\\"] {
        std::fs::write(root.join(".gitignore"), format!("*.sifr\n{rule}\n")).unwrap();
        for target in [Path::new("."), Path::new("main.sifr")] {
            let result = check(root, target, &["--force-exclude", "--no-respect-gitignore"]);
            assert_eq!(result.status.code(), Some(1), "{rule:?}: {result:?}");
            let stderr = String::from_utf8_lossy(&result.stderr);
            assert!(stderr.contains("SIFR-FMT-0001"), "{stderr}");
            assert!(!stderr.contains("gitignore"), "{stderr}");
        }
    }
}

#[test]
fn formatter_discovery_mixed_targets_load_rules_at_directory_selection() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::create_dir(root.join("empty")).unwrap();
    std::fs::write(root.join(".gitignore"), "{a,b\n").unwrap();
    for name in ["first.sifr", "second.sifr"] {
        std::fs::write(root.join(name), "def main( ):\n    pass\n").unwrap();
    }
    let result = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .current_dir(root)
        .args(["fmt", "--no-cache", "first.sifr", "second.sifr", "empty"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("invalid formatter gitignore"));
    for name in ["first.sifr", "second.sifr"] {
        assert_eq!(
            std::fs::read_to_string(root.join(name)).unwrap(),
            "def main():\n    pass\n"
        );
    }
}
