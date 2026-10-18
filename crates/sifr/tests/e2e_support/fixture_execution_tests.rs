use super::fixture_execution::execute_fixture;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

struct Probe {
    _temp: tempfile::TempDir,
    root: PathBuf,
    binary: PathBuf,
}

impl Probe {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let source = root.join("probe.rs");
        std::fs::write(&source, r#"
use std::{env, fs, process::Command, thread, time::Duration};
fn main() {
    let first = env::args().nth(1).unwrap();
    let name = if first == "--case" { env::args().nth(2).unwrap() } else { first };
    if name == "descendant" {
        fs::write("child-ready", "ready").unwrap();
        thread::sleep(Duration::from_secs(2));
        fs::write("escaped-child", "bad").unwrap();
        return;
    }
    let cwd = env::current_dir().unwrap();
    for key in ["TMPDIR", "TEMP", "TMP"] {
        assert_eq!(fs::canonicalize(env::var_os(key).unwrap()).unwrap(), cwd);
    }
    assert_eq!(fs::canonicalize(env::temp_dir()).unwrap(), cwd);
    fs::write("same-name", &name).unwrap();
    fs::write(env::temp_dir().join("same-temp-name"), &name).unwrap();
    println!("fixture stdout: {name}");
    eprintln!("fixture stderr: {name}");
    if name == "timeout" {
        let mut child = Command::new(env::current_exe().unwrap()).arg("descendant").spawn().unwrap();
        child.wait().unwrap();
        thread::sleep(Duration::from_secs(30));
    }
    thread::sleep(Duration::from_millis(100));
    assert_eq!(fs::read_to_string("same-name").unwrap(), name);
    assert_eq!(fs::read_to_string(env::temp_dir().join("same-temp-name")).unwrap(), name);
    if name != "success" { std::process::exit(7); }
}
"#).unwrap();
        let binary = root.join(format!("probe{}", std::env::consts::EXE_SUFFIX));
        let output = super::run_capture({
            let mut command = Command::new("rustc");
            command.arg(&source).arg("-o").arg(&binary);
            command
        });
        assert!(output.status_ok, "{}", output.stderr);
        Self {
            _temp: temp,
            root,
            binary,
        }
    }

    fn run(&self, name: &str, deadline: Duration) -> Result<(), String> {
        let mut command = Command::new(&self.binary);
        command.arg(name);
        execute_fixture(command, "same-fixture", &self.root, deadline)
    }

    fn attempts(&self) -> Vec<PathBuf> {
        std::fs::read_dir(self.root.join("runs"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect()
    }
}

#[test]
fn parallel_fixtures_isolate_identical_output_names_and_retain_failures() {
    let probe = Probe::new();
    std::thread::scope(|scope| {
        let first = scope.spawn(|| probe.run("first", Duration::from_secs(10)));
        let second = scope.spawn(|| probe.run("second", Duration::from_secs(10)));
        for result in [first.join().unwrap(), second.join().unwrap()] {
            assert!(result.unwrap_err().contains("retained fixture scratch:"));
        }
    });
    let attempts = probe.attempts();
    assert_eq!(attempts.len(), 2);
    let mut names = Vec::new();
    for attempt in attempts {
        assert_eq!(
            std::fs::read_to_string(attempt.join("fixture.txt")).unwrap(),
            "same-fixture"
        );
        let name = std::fs::read_to_string(attempt.join("work/same-name")).unwrap();
        let log = std::fs::read_to_string(attempt.join("execution.log")).unwrap();
        assert!(log.contains(&format!("fixture stdout: {name}")));
        assert!(log.contains(&format!("fixture stderr: {name}")));
        assert!(!log.contains("panicked"), "{log}");
        for file in ["same-name", "same-temp-name"] {
            assert_eq!(
                std::fs::read_to_string(attempt.join("work").join(file)).unwrap(),
                name
            );
        }
        names.push(name);
    }
    names.sort();
    assert_eq!(names, ["first", "second"]);
    assert!(!probe.root.join("same-name").exists());
    assert!(!probe.root.join("same-temp-name").exists());
}

#[test]
fn successful_fixture_cleans_only_its_own_completed_attempt() {
    let probe = Probe::new();
    probe.run("failure", Duration::from_secs(10)).unwrap_err();
    let failed = probe.attempts();
    super::fixture_execution::run_single_case(&probe.binary, "success", &probe.root).unwrap();
    assert_eq!(probe.attempts(), failed);
    assert_eq!(
        std::fs::read_to_string(failed[0].join("work/same-name")).unwrap(),
        "failure"
    );
}

#[test]
fn deadline_reaps_descendants_and_retains_partial_output() {
    let probe = Probe::new();
    let error = probe
        .run("timeout", Duration::from_millis(500))
        .unwrap_err();
    assert!(error.contains("deadline"), "{error}");
    let attempts = probe.attempts();
    assert_eq!(attempts.len(), 1);
    assert!(attempts[0].join("work/child-ready").exists());
    let log = std::fs::read_to_string(attempts[0].join("execution.log")).unwrap();
    assert!(log.contains("fixture stdout: timeout"), "{log}");
    assert!(log.contains("fixture stderr: timeout"), "{log}");
    std::thread::sleep(Duration::from_millis(2200));
    assert!(!attempts[0].join("work/escaped-child").exists());
}

#[test]
fn launch_failure_retains_attempt_and_fixture_names_cannot_escape_scratch() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let command = Command::new(root.join("missing-executable"));
    let error =
        execute_fixture(command, "../../fixture", &root, Duration::from_secs(1)).unwrap_err();
    assert!(error.contains("retained fixture scratch"), "{error}");
    let entries = std::fs::read_dir(root.join("runs"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 1);
    let attempt = entries[0].path();
    assert!(attempt.join("execution.log").is_file());
    assert_eq!(
        std::fs::read_to_string(attempt.join("fixture.txt")).unwrap(),
        "../../fixture"
    );
}
