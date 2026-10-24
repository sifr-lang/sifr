//! Process isolation for CLI tests that intentionally change the working directory.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

const SELECTED_TEST: &str = "SIFR_ISOLATED_CLI_TEST";
const COMPLETED: &str = "sifr-isolated-test-completed: ";

pub(super) fn run(name: &str, body: impl FnOnce()) {
    match std::env::var(SELECTED_TEST) {
        Ok(selected) => {
            assert_eq!(selected, name, "isolated child selected a different test");
            body();
            println!("\n{COMPLETED}{name}");
        }
        Err(std::env::VarError::NotPresent) => {
            let output = launch(name, None);
            require_completion(name, &output).unwrap_or_else(|reason| panic!("{reason}"));
        }
        Err(error) => panic!("invalid isolated test selection: {error}"),
    }
}

fn launch(name: &str, control: Option<&str>) -> Output {
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        .args([
            name,
            "--exact",
            "--include-ignored",
            "--test-threads=1",
            "--nocapture",
        ])
        .env(SELECTED_TEST, name);
    if let Some(control) = control {
        command.env("SIFR_ISOLATED_CLI_TEST_CONTROL", control);
    }
    sifr_driver::process_execution::output_with_deadline(&mut command, Duration::from_mins(30))
        .expect("isolated test must complete before its deadline")
}

fn require_completion(name: &str, output: &Output) -> Result<(), String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let marker = format!("{COMPLETED}{name}");
    if output.status.success() && stdout.lines().filter(|line| *line == marker).count() == 1 {
        return Ok(());
    }
    Err(format!(
        "isolated test {name} did not complete exactly once ({status}):\n{stdout}\n{stderr}",
        status = output.status,
        stderr = String::from_utf8_lossy(&output.stderr),
    ))
}

pub(super) struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    pub(super) fn enter(path: &Path) -> Self {
        assert!(
            std::env::var_os(SELECTED_TEST).is_some(),
            "directory mutation requires an isolated child test"
        );
        let previous = std::env::current_dir().expect("test cwd should exist");
        std::env::set_current_dir(path).expect("scenario cwd should be selected");
        Self { previous }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.previous).expect("test cwd should be restored");
    }
}

#[test]
fn child_execution_preserves_cwd_and_rejects_missing_or_failed_tests() {
    const NAME: &str =
        "isolated_cli_tests::child_execution_preserves_cwd_and_rejects_missing_or_failed_tests";
    if let Ok(mode) = std::env::var("SIFR_ISOLATED_CLI_TEST_CONTROL") {
        run(NAME, || {
            let directory = crate::mode_resolution_tests::mktemp_dir("isolation_control");
            let previous = std::env::current_dir().expect("parent cwd");
            {
                let _cwd = CurrentDirGuard::enter(&directory);
                assert_eq!(std::env::current_dir().expect("child cwd"), directory);
            }
            assert_eq!(std::env::current_dir().expect("restored cwd"), previous);
            std::fs::remove_dir(directory).expect("remove control directory");
            assert_eq!(mode, "pass", "intentional child failure");
        });
        return;
    }
    let previous = std::env::current_dir().expect("parent cwd");
    let missing = "isolated_cli_tests::nonexistent_test_selection";
    let output = launch(missing, None);
    assert!(
        output.status.success(),
        "libtest accepts an empty selection"
    );
    assert!(require_completion(missing, &output).is_err());
    let output = launch(NAME, Some("fail"));
    assert!(!output.status.success());
    assert!(require_completion(NAME, &output).is_err());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("intentional child failure")
            || String::from_utf8_lossy(&output.stderr).contains("intentional child failure")
    );
    let output = launch(NAME, Some("pass"));
    require_completion(NAME, &output).expect("successful child must attest execution");
    assert_eq!(
        std::env::current_dir().expect("unchanged parent cwd"),
        previous
    );
}
