use crate::validation_contract_support::manifest::{self, Assertion, CommandSpec, Stream, Suite};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[derive(Clone, Debug)]
struct CommandResult {
    argv: Vec<String>,
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[derive(Clone, Debug)]
enum CommandExecutionGroup {
    Serial(CommandSpec),
    Parallel(Vec<CommandSpec>),
}

pub(crate) fn run() -> Result<(), String> {
    let repo_root = repo_root();
    let suites = manifest::load(&repo_root)?;
    let started = Instant::now();
    let mut suite_timings = Vec::new();
    let mut total_rows = 0usize;

    for suite in suites {
        let suite_started = Instant::now();
        println!("validation-contract suite={}", suite.name);
        run_suite(&repo_root, &suite)?;
        let elapsed_ms = suite_started.elapsed().as_millis();
        suite_timings.push((suite.name.clone(), suite.rows.len(), elapsed_ms));
        total_rows += suite.rows.len();
    }

    println!("[validation-contract] summary:");
    for (suite_name, row_count, elapsed_ms) in suite_timings {
        println!("  - {suite_name}: {row_count} rows, {elapsed_ms}ms");
    }
    println!(
        "[validation-contract] total_rows={} total_ms={}",
        total_rows,
        started.elapsed().as_millis()
    );
    Ok(())
}

fn repo_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("crate should live under repo_root/crates/sifr")
}

fn run_suite(repo_root: &Path, suite: &Suite) -> Result<(), String> {
    for row in &suite.rows {
        let row_started = Instant::now();
        let mut row_status = "pass";
        println!("  row={}", row.id);
        let tmp_dir = temp_root(repo_root, &suite.name, &row.id)?;
        let result = (|| {
            let results = run_row_commands(repo_root, &tmp_dir, &row.commands)?;
            for assertion in &row.assertions {
                apply_assertion(assertion, &results)?;
            }
            Ok::<(), String>(())
        })();
        let _ = std::fs::remove_dir_all(&tmp_dir);
        if result.is_err() {
            row_status = "fail";
        }
        println!(
            "[sifr-case-timing] bucket=validation_contract case={}/{} elapsed_ms={} status={}",
            suite.name,
            row.id,
            row_started.elapsed().as_millis(),
            row_status
        );
        result?;
    }
    println!("{}: PASS", suite.label);
    Ok(())
}

fn temp_root(repo_root: &Path, suite_name: &str, row_id: &str) -> Result<PathBuf, String> {
    let _ = repo_root;
    let root = std::env::temp_dir().join(format!(
        "sifr-validation-contracts-{suite_name}-{row_id}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root)
        .map_err(|err| format!("failed to create {}: {err}", root.display()))?;
    Ok(root)
}

fn run_row_commands(
    repo_root: &Path,
    tmp_dir: &Path,
    commands: &[CommandSpec],
) -> Result<BTreeMap<String, CommandResult>, String> {
    let execution_groups = execution_groups(commands);
    let mut results = BTreeMap::new();
    for group in execution_groups {
        match group {
            CommandExecutionGroup::Serial(command) => {
                let result = run_command(repo_root, tmp_dir, &command)?;
                results.insert(command.id.clone(), result);
            }
            CommandExecutionGroup::Parallel(commands) => {
                let output = Arc::new(Mutex::new(Vec::with_capacity(commands.len())));
                let mut handles = Vec::with_capacity(commands.len());
                for command in commands {
                    let repo_root = repo_root.to_path_buf();
                    let tmp_dir = tmp_dir.to_path_buf();
                    let output = Arc::clone(&output);
                    handles.push(thread::spawn(move || {
                        let result = run_command(&repo_root, &tmp_dir, &command);
                        output
                            .lock()
                            .unwrap_or_else(|err| err.into_inner())
                            .push((command.id.clone(), result));
                    }));
                }
                for handle in handles {
                    handle
                        .join()
                        .map_err(|_| "parallel validation command panicked".to_string())?;
                }
                let grouped = output.lock().unwrap_or_else(|err| err.into_inner()).clone();
                for (command_id, result) in grouped {
                    results.insert(command_id, result?);
                }
            }
        }
    }
    Ok(results)
}

fn execution_groups(commands: &[CommandSpec]) -> Vec<CommandExecutionGroup> {
    let mut groups = Vec::new();
    let mut index = 0usize;
    while index < commands.len() {
        let command = commands[index].clone();
        if let Some(parallel_group) = command.parallel_group.clone() {
            let mut grouped = vec![command];
            index += 1;
            while index < commands.len() {
                let next = commands[index].clone();
                if next.parallel_group.as_deref() == Some(parallel_group.as_str()) {
                    grouped.push(next);
                    index += 1;
                } else {
                    break;
                }
            }
            groups.push(CommandExecutionGroup::Parallel(grouped));
        } else {
            groups.push(CommandExecutionGroup::Serial(command));
            index += 1;
        }
    }
    groups
}

fn run_command(
    repo_root: &Path,
    tmp_dir: &Path,
    command: &CommandSpec,
) -> Result<CommandResult, String> {
    if command.argv.is_empty() {
        return Err(format!("row command '{}' has empty argv", command.id));
    }

    let argv = command
        .argv
        .iter()
        .map(|value| value.replace("<TMP>", &tmp_dir.display().to_string()))
        .collect::<Vec<_>>();
    let mut child = Command::new(&argv[0]);
    child.args(&argv[1..]);
    child.current_dir(repo_root);
    let output = child
        .output()
        .map_err(|err| format!("failed to execute '{}': {err}", argv.join(" ")))?;
    let result = CommandResult {
        argv: argv.clone(),
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    if result.exit_code != command.expected_exit {
        return Err(format!(
            "validation contract command '{}' expected exit {}, got {}\n  command: {}\n  stderr:\n{}",
            command.id,
            command.expected_exit,
            result.exit_code,
            result.argv.join(" "),
            result.stderr
        ));
    }
    Ok(result)
}

fn apply_assertion(
    assertion: &Assertion,
    results: &BTreeMap<String, CommandResult>,
) -> Result<(), String> {
    match assertion {
        Assertion::Contains {
            command_id,
            stream,
            text,
        } => {
            let result = command_result(results, command_id)?;
            let actual = stream_text(result, *stream);
            if !actual.contains(text) {
                return Err(format!(
                    "validation contract assertion failed: command '{}' {} missing {:?}\n  command: {}\n  actual {}:\n{}",
                    command_id,
                    stream_name(*stream),
                    text,
                    result.argv.join(" "),
                    stream_name(*stream),
                    actual
                ));
            }
        }
        Assertion::EqualStreams {
            left_command_id,
            right_command_id,
            stream,
        } => {
            let left = command_result(results, left_command_id)?;
            let right = command_result(results, right_command_id)?;
            let left_text = stream_text(left, *stream);
            let right_text = stream_text(right, *stream);
            if left_text != right_text {
                return Err(format!(
                    "validation contract assertion failed: {} differed between '{}' and '{}'\n  left command: {}\n  right command: {}\n  left {}:\n{}\n  right {}:\n{}",
                    stream_name(*stream),
                    left_command_id,
                    right_command_id,
                    left.argv.join(" "),
                    right.argv.join(" "),
                    stream_name(*stream),
                    left_text,
                    stream_name(*stream),
                    right_text
                ));
            }
        }
    }
    Ok(())
}

fn command_result<'a>(
    results: &'a BTreeMap<String, CommandResult>,
    command_id: &str,
) -> Result<&'a CommandResult, String> {
    results.get(command_id).ok_or_else(|| {
        format!(
            "validation contract assertion referenced unknown command '{}'",
            command_id
        )
    })
}

fn stream_text(result: &CommandResult, stream: Stream) -> &str {
    match stream {
        Stream::Stdout => &result.stdout,
        Stream::Stderr => &result.stderr,
    }
}

fn stream_name(stream: Stream) -> &'static str {
    match stream {
        Stream::Stdout => "stdout",
        Stream::Stderr => "stderr",
    }
}
