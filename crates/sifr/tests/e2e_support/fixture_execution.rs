//! Run fixture programs outside compiler inputs, retaining unsuccessful attempts.
use std::path::Path;
use std::process::Command;
use std::time::Duration;

pub(crate) fn run_single_case(
    artifact: &Path,
    fixture: &str,
    group_root: &Path,
) -> Result<(), String> {
    run_single_case_bounded(artifact, fixture, group_root, Duration::from_secs(30 * 60))
}

pub(crate) fn run_single_case_bounded(
    artifact: &Path,
    fixture: &str,
    group_root: &Path,
    deadline: Duration,
) -> Result<(), String> {
    if deadline.is_zero() {
        return Err("diagnostic isolation deadline exhausted; fixture blocked".to_owned());
    }
    if !artifact.is_absolute() {
        return Err("fixture executable must have an absolute Cargo-selected path".to_owned());
    }
    let mut command = Command::new(artifact);
    command.args(["--case", fixture]);
    execute_fixture(command, fixture, group_root, deadline)
}

#[expect(clippy::print_stderr, reason = "per-fixture execution timing")]
pub(super) fn execute_fixture(
    command: Command,
    fixture: &str,
    group_root: &Path,
    deadline: Duration,
) -> Result<(), String> {
    let start = std::time::Instant::now();
    let result = execute_fixture_inner(command, fixture, group_root, deadline);
    eprintln!(
        "[sifr-case-timing] bucket=runtime case={} elapsed_ms={} status={}",
        fixture,
        start.elapsed().as_millis(),
        super::telemetry::status(result.is_ok())
    );
    result
}

fn execute_fixture_inner(
    mut command: Command,
    fixture: &str,
    group_root: &Path,
    deadline: Duration,
) -> Result<(), String> {
    let runs = group_root.join("runs");
    sifr_cache_storage::directory(&runs)
        .map_err(|error| format!("cannot create fixture execution directory: {error}"))?;
    let prefix = fixture
        .chars()
        .take(32)
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    let attempt = tempfile::Builder::new()
        // Exercise spaces in ordinary runs so fixtures cannot assume shell-safe paths.
        .prefix(&format!("{prefix}-run "))
        .tempdir_in(&runs)
        .map_err(|error| format!("cannot allocate fixture execution directory: {error}"))?
        // Retain even if the harness is interrupted or unwinds before capture returns.
        .keep();
    let work = attempt.join("work");
    let prepare = || -> std::io::Result<()> {
        sifr_cache_storage::directory(&work)?;
        std::fs::write(attempt.join("fixture.txt"), fixture)
    };
    if let Err(error) = prepare() {
        return Err(format!(
            "cannot prepare fixture: {error}; retained {}",
            attempt.display()
        ));
    }
    command.current_dir(&work);
    for name in ["TMPDIR", "TEMP", "TMP"] {
        command.env(name, &work);
    }
    match sifr_driver::process_execution::output_with_deadline(&mut command, deadline) {
        Ok(output) if output.status.success() => {
            std::fs::remove_dir_all(&attempt).map_err(|error| {
                format!(
                    "cannot remove completed fixture scratch {}: {error}",
                    attempt.display()
                )
            })
        }
        result => {
            let cause = match &result {
                Ok(output) => {
                    serde_json::json!({"cause":"correctness","exit_code":output.status.code()})
                }
                Err(error) => super::telemetry::failure_details(error),
            };
            super::telemetry::emit(serde_json::json!({"schema":1,"event":"fixture_failure",
                "fixture":fixture,"status":"fail","details":cause,"retained":attempt}));
            let diagnostic = match result {
                Ok(output) => format!(
                    "status: {}\nstdout:\n{}\nstderr:\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                ),
                Err(error) => error.to_string(),
            };
            let log = attempt.join("execution.log");
            let record_error = std::fs::write(&log, &diagnostic)
                .err()
                .map_or_else(String::new, |error| {
                    format!("\nfailed to save execution log: {error}")
                });
            Err(format!(
                "binary exited with error:\n{diagnostic}\nretained fixture scratch: {}{record_error}",
                attempt.display(),
            ))
        }
    }
}
