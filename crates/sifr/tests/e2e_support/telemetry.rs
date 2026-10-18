//! Diagnostic observations. Only actual test outcomes authorize a gate pass.
#![expect(clippy::print_stderr, reason = "structured E2E progress and timing")]
use serde_json::{Value, json};
use std::process::{Command, Output};
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static ORIGIN: OnceLock<Instant> = OnceLock::new();
static NEXT: AtomicU64 = AtomicU64::new(0);

fn clock_us() -> u128 {
    ORIGIN.get_or_init(Instant::now).elapsed().as_micros()
}

pub(super) fn emit(value: Value) {
    eprintln!("[sifr-e2e-event] {value}");
}

pub(super) struct Span {
    pub(super) id: String,
    scope: &'static str,
    parent: Option<String>,
    start_us: u128,
    finished: bool,
}

impl Span {
    pub(super) fn start(scope: &'static str, parent: Option<&str>, details: Value) -> Self {
        let span = Self {
            id: format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ),
            scope,
            parent: parent.map(str::to_owned),
            start_us: clock_us(),
            finished: false,
        };
        emit(
            json!({"schema":1,"event":"start","id":span.id,"scope":scope,
            "parent":span.parent,"clock_us":span.start_us,
            "unix_ms":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
            "details":details}),
        );
        span
    }

    pub(super) fn finish(mut self, status: &str, details: Value) {
        self.end(status, details);
    }

    fn end(&mut self, status: &str, details: Value) {
        let end = clock_us();
        emit(
            json!({"schema":1,"event":"end","id":self.id,"scope":self.scope,
            "parent":self.parent,"clock_us":end,"elapsed_us":end-self.start_us,
            "status":status,"details":details}),
        );
        self.finished = true;
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if !self.finished {
            self.end(
                "incomplete",
                json!({"cause":"scope_left_without_terminal_result"}),
            );
        }
    }
}

pub(super) fn status(passed: bool) -> &'static str {
    if passed { "pass" } else { "fail" }
}

pub(super) fn command_details(command: &Command, deadline: Duration) -> Value {
    json!({"program":command.get_program().to_string_lossy(),
        "args":command.get_args().map(|a|a.to_string_lossy()).collect::<Vec<_>>(),
        "cwd":command.get_current_dir(),"safety_deadline_ms":deadline.as_millis(),
        "live_compiler_unit":"unknown; native Cargo output is captured"})
}

pub(super) fn output(command: &mut Command, deadline: Duration) -> std::io::Result<Output> {
    let span = Span::start("process", None, command_details(command, deadline));
    let result = sifr_driver::process_execution::output_with_deadline(command, deadline);
    let details = match &result {
        Ok(value) => json!({"cause":"exit","exit_code":value.status.code()}),
        Err(error) => failure_details(error),
    };
    span.finish(
        status(result.as_ref().is_ok_and(|v| v.status.success())),
        details,
    );
    result
}

pub(super) fn cargo_units(stdout: &str) -> Value {
    let (mut fresh, mut rebuilt) = (0, 0);
    for line in stdout.lines() {
        if let Ok(row) = serde_json::from_str::<Value>(line) {
            if row["reason"] == "compiler-artifact" {
                match row["fresh"].as_bool() {
                    Some(true) => fresh += 1,
                    Some(false) => rebuilt += 1,
                    None => {}
                }
            }
        }
    }
    json!({"fresh_units":fresh,"rebuilt_units":rebuilt,
        "unit_scope":"Cargo artifact observations in this attempt; dependencies may repeat across attempts",
        "link_us":null,"link_note":"included in native build; no independent linker duration"})
}

pub(super) fn failure_details(error: &std::io::Error) -> Value {
    let failure = error
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<sifr_driver::process_execution::ProcessFailure>());
    json!({"cause":failure.map_or("infrastructure",|v|v.cause),
        "stdout_truncated":failure.map(|v|v.stdout_truncated),
        "stderr_truncated":failure.map(|v|v.stderr_truncated)})
}
