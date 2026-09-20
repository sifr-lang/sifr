//! Opt-in bounded projection of existing report owners. Never capture raw
//! arguments, source, environment, diagnostics, paths, or process output.
use serde_json::{Value, json};
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, Write};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const MAX_REPORTS: usize = 64;
const MAX_BYTES: usize = 32 * 1024;
static SINK: OnceLock<Mutex<Sink>> = OnceLock::new();

struct Sink {
    file: File,
    start: Instant,
    command: String,
    startup: Duration,
    reports: Vec<Value>,
    dropped: usize,
    overhead: Duration,
}

pub(crate) fn start(path: &Path, command: &str, start: Instant) -> io::Result<()> {
    let preparation = Instant::now();
    let mut file = create_trace_file(path)?;
    file.write_all(b"{\"schema_version\":1,\"outcome\":\"incomplete\"}\n")?;
    let sink = Sink {
        file,
        start,
        command: command.to_owned(),
        startup: preparation.duration_since(start),
        reports: Vec::new(),
        dropped: 0,
        overhead: preparation.elapsed(),
    };
    SINK.set(Mutex::new(sink))
        .map_err(|_| io::Error::other("trace sink already initialized"))
}

fn create_trace_file(path: &Path) -> io::Result<File> {
    // A fresh directory is required: never overwrite user files, follow a
    // destination symlink, or accumulate unbounded invocations in one sink.
    let mut directory = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(path)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path.join("trace-v1.json"))
}

fn record(make: impl FnOnce() -> Value) {
    let Some(sink) = SINK.get() else { return };
    let start = Instant::now();
    if let Ok(mut sink) = sink.lock() {
        if sink.reports.len() < MAX_REPORTS {
            sink.reports.push(make());
        } else {
            sink.dropped += 1;
        }
        sink.overhead += start.elapsed();
    }
}

pub(crate) fn build_report(report: &sifr_driver::BuildReport) {
    record(|| {
        json!({
            "owner": "BuildReport", "mode": report.mode().as_str(),
            "cache_hit": report.cache_hit(), "target": report.target(),
        "sysroot_content_sha256": safe_identity(report.sysroot().content_sha256()),
        "dependency_fingerprint": safe_identity(report.sysroot().dependency_fingerprint()),
            "total_us": report.total_elapsed().as_micros(),
            "stages": report.stages().iter().take(32).enumerate().map(|(index, stage)|
                // Labels are free strings in the owner API; indices preserve its
                // declaration order without accidentally copying user text.
                json!({"index": index, "phase": stage_phase(stage.label()), "elapsed_us": stage.elapsed().as_micros()})
            ).collect::<Vec<_>>(),
            "omitted_stages": report.stages().len().saturating_sub(32),
        })
    });
}

pub(crate) fn project_report(
    compiler: &sifr_driver::CompilerContext,
    report: &sifr_driver::project_cache::ProjectCacheReport,
) {
    record(|| {
        json!({
            "owner": "ProjectCacheReport",
            "status": match report.status.as_str() {
            value @ ("disabled" | "external-context" | "metadata-unavailable" | "restored" | "interface-restored" | "interface-restored-write-unavailable" | "interface-restored-changed-inputs" | "interface-restored-uncacheable" | "miss" | "unavailable" | "cancelled" | "published" | "write-unavailable" | "changed-inputs" | "uncacheable") => value,
            _ => "unrecognized-redacted",
        },
        "restored_checks": report.restored_checks,
            "computed_checks": report.computed_checks,
            "captured_sources": report.captured_sources,
            "validation_us": report.validation_us,
            "serialization_us": report.serialization_us,
            "payload_bytes": report.payload_bytes,
        })
    });
    record(|| {
        let stats = compiler.metadata_stats().unwrap_or(Value::Null);
        let mut selected = json!({"owner": "metadata_stats"});
        for name in [
            "physical_payload_decode_us",
            "semantic_modules",
            "decoded_semantic_records",
            "decoded_hir_modules",
            "decoded_types",
            "decoded_nominal_views",
            "projected_nominal_views",
            "decoded_hir_functions",
            "decoded_hir_classes",
            "decoded_rust_payloads",
            "decoded_templates",
            "rust_payload_reads",
            "hir_module_reads",
            "retained_decode_bound_bytes",
        ] {
            if stats[name].is_number() {
                selected[name] = stats[name].clone();
            }
        }
        for name in [
            "read_us",
            "hash_us",
            "index_us",
            "physical_decode_us",
            "input_bytes",
            "total_us",
        ] {
            if stats["load_timings"][name].is_number() {
                selected["load_timings"][name] = stats["load_timings"][name].clone();
            }
        }
        if let Some(id) = stats["metadata_id"].as_str() {
            selected["metadata_id"] = json!(safe_identity(id));
        }
        selected
    });
}

fn stage_phase(label: &str) -> &'static str {
    match label {
        "Loading Sifr standard library" => "stdlib",
        "Generating Rust project" => "generation",
        "Materializing Cargo project" => "native_preparation",
        "Building development binary" | "Building test binary" | "Building release binary" => {
            "native_compile_and_link"
        }
        "Preparing SQL schema profiles" => "sql_preparation",
        "Compiling SQL query declarations" => "sql_compilation",
        value if value.starts_with("Parsing ") => "parse",
        value if value.starts_with("Analyzing ") => "analysis",
        _ => "unclassified_redacted",
    }
}

pub(crate) fn run_program(command: &mut std::process::Command) -> io::Result<std::process::Output> {
    let start = SINK.get().map(|_| Instant::now());
    let result = sifr_driver::process_execution::run_program(command);
    if let Some(start) = start {
        let elapsed = start.elapsed();
        record(|| {
            json!({"owner": "process_execution", "phase": "runtime",
            "elapsed_us": elapsed.as_micros(), "execution_error": result.is_err(),
            "exit_code": result.as_ref().ok().and_then(|output| output.status.code())})
        });
    }
    result
}

pub(crate) fn frontend_report(report: &sifr_frontend::WorkspaceTraceLog) {
    for event in &report.events {
        record(|| {
            json!({"owner": "WorkspaceTraceLog", "sequence": event.sequence,
            "phase": event.phase.as_str()})
        });
    }
}

fn finish(code: i32) -> io::Result<()> {
    let Some(sink) = SINK.get() else {
        return Ok(());
    };
    let mut sink = sink
        .lock()
        .map_err(|_| io::Error::other("trace sink lock poisoned"))?;
    finish_sink(&mut sink, code)
}

fn finish_sink(sink: &mut Sink, code: i32) -> io::Result<()> {
    let finalization = Instant::now();
    let mut value = report_value(sink, code);
    // First account for serialization/truncation work, then freeze final timing
    // values. Reapply the bound after their widths change. The final bounding
    // serialization, like the final write itself, is outside those timestamps.
    bounded_bytes(&mut value)?;
    let bytes = final_bytes(
        &mut value,
        sink.overhead + finalization.elapsed(),
        sink.start.elapsed(),
    )?;
    sink.file.rewind()?;
    sink.file.write_all(&bytes)?;
    sink.file.set_len(bytes.len() as u64)?;
    sink.file.flush()
}

fn report_value(sink: &Sink, code: i32) -> Value {
    json!({
        "schema_version": 1,
        "compiler_identity": env!("SIFR_COMPILER_BUILD_ID"),
        "command": sink.command,
        "startup_us": sink.startup.as_micros(),
        "timing_semantics": "owner wall intervals may overlap; do not sum; unavailable stages are not inferred",
        "outcome": if code == 0 { "success" } else { "failure" },
        "exit_code": code,
        "invocation_us_before_final_write": sink.start.elapsed().as_micros(),
        "trace_overhead_us_before_final_write": sink.overhead.as_micros(),
        "final_write_included": false,
        "max_bytes": MAX_BYTES, "max_reports": MAX_REPORTS,
        "dropped_reports": sink.dropped,
        "reports": sink.reports,
    })
}

fn final_bytes(value: &mut Value, overhead: Duration, invocation: Duration) -> io::Result<Vec<u8>> {
    value["trace_overhead_us_before_final_write"] = json!(overhead.as_micros());
    value["invocation_us_before_final_write"] = json!(invocation.as_micros());
    bounded_bytes(value)
}

// A pass removes at most MAX_REPORTS optional records; required outcome,
// identity and timing fields survive even when every report must be omitted.
fn bounded_bytes(value: &mut Value) -> io::Result<Vec<u8>> {
    loop {
        let bytes = serde_json::to_vec(&value)?;
        if bytes.len() <= MAX_BYTES {
            return Ok(bytes);
        }
        let Some(reports) = value["reports"].as_array_mut() else {
            return Err(io::Error::other("invalid trace report array"));
        };
        if reports.pop().is_none() {
            return Err(io::Error::other("trace header exceeds size bound"));
        }
        let dropped = value["dropped_reports"]
            .as_u64()
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| io::Error::other("invalid trace dropped report count"))?;
        value["dropped_reports"] = json!(dropped);
    }
}

pub(crate) fn exit(code: i32) -> ! {
    let code = match finish(code) {
        Ok(()) => code,
        Err(error) => {
            // Keep transport/source/program stdout untouched. A failed command
            // retains its original exit status; a failed requested sink cannot
            // turn an otherwise successful command into false success.
            let _ = writeln!(io::stderr(), "error: cannot finalize --trace-dir: {error}");
            if code == 0 { 2 } else { code }
        }
    };
    std::process::exit(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_sink(file: File) -> Sink {
        Sink {
            file,
            start: Instant::now(),
            command: "check".into(),
            startup: Duration::from_micros(u64::MAX),
            reports: Vec::new(),
            dropped: usize::MAX - 1,
            overhead: Duration::ZERO,
        }
    }

    #[test]
    fn final_timing_width_at_byte_boundary_truncates_truthfully() {
        let widest = Duration::from_micros(u64::MAX);
        let sink = test_sink(tempfile::tempfile().unwrap());
        for delta in [-1_i32, 0, 1] {
            let mut value = report_value(&sink, 7);
            value["reports"] = json!([{"detail": ""}]);
            final_bytes(&mut value, widest, widest).unwrap();
            let header = serde_json::to_vec(&value).unwrap().len();
            value["reports"][0]["detail"] =
                json!("x".repeat((MAX_BYTES as i32 + delta) as usize - header));
            assert_eq!(
                serde_json::to_vec(&value).unwrap().len(),
                (MAX_BYTES as i32 + delta) as usize
            );
            let bytes = final_bytes(&mut value, widest, widest).unwrap();
            assert!(bytes.len() <= MAX_BYTES);
            let parsed: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(parsed["schema_version"], 1);
            assert_eq!(parsed["outcome"], "failure");
            assert_eq!(parsed["exit_code"], 7);
            assert_eq!(parsed["trace_overhead_us_before_final_write"], u64::MAX);
            assert_eq!(parsed["invocation_us_before_final_write"], u64::MAX);
            assert_eq!(
                parsed["dropped_reports"],
                json!(sink.dropped + usize::from(delta > 0))
            );
            assert_eq!(
                parsed["reports"].as_array().unwrap().len(),
                usize::from(delta <= 0)
            );
        }

        // The initial pass fits exactly; final timing digits alone cross the
        // cap. The only event is dropped and the counter gains a digit.
        let mut value = report_value(&sink, 0);
        value["dropped_reports"] = json!(9);
        value["reports"] = json!([{"detail": ""}]);
        final_bytes(&mut value, Duration::ZERO, Duration::ZERO).unwrap();
        let header = serde_json::to_vec(&value).unwrap().len();
        value["reports"][0]["detail"] = json!("x".repeat(MAX_BYTES - header));
        assert_eq!(bounded_bytes(&mut value).unwrap().len(), MAX_BYTES);
        let bytes = final_bytes(&mut value, widest, widest).unwrap();
        let parsed: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(bytes.len() < MAX_BYTES);
        assert_eq!(parsed["dropped_reports"], 10);
        assert_eq!(parsed["reports"], json!([]));
        assert_eq!(parsed["outcome"], "success");

        // A genuine final write error remains an error, independent of size.
        let temporary = tempfile::NamedTempFile::new().unwrap();
        let mut sink = test_sink(File::open(temporary.path()).unwrap());
        assert!(finish_sink(&mut sink, 0).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn trace_directory_is_private_under_permissive_umask() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        const CHILD: &str = "SIFR_TRACE_PRIVATE_DIRECTORY_CHILD";
        if std::env::var_os(CHILD).is_none() {
            // Set umask in a child shell only, never in the multithreaded test
            // process. The test executable and argument are passed separately.
            let result = std::process::Command::new("sh")
                .args(["-c", "umask 000; exec \"$1\" --exact trace_artifacts::tests::trace_directory_is_private_under_permissive_umask --nocapture", "trace-mode-test"])
                .arg(std::env::current_exe().unwrap())
                .env(CHILD, "1")
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed"));
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("trace");
        let file = create_trace_file(&path).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(file.metadata().unwrap().permissions().mode() & 0o777, 0o600);
        assert!(create_trace_file(&path).is_err());

        let existing = root.path().join("existing");
        std::fs::create_dir(&existing).unwrap();
        std::fs::set_permissions(&existing, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(create_trace_file(&existing).is_err());
        assert_eq!(
            std::fs::metadata(&existing).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert!(!existing.join("trace-v1.json").exists());
        let link = root.path().join("link");
        symlink(&existing, &link).unwrap();
        assert!(create_trace_file(&link).is_err());
        assert_eq!(
            std::fs::metadata(&existing).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert!(!existing.join("trace-v1.json").exists());
    }

    #[test]
    fn trace_dir_redaction_and_size_bound() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("trace");
        start(&path, "trace", Instant::now()).unwrap();
        let report = sifr_frontend::WorkspaceTraceLog {
            events: (0..1000)
                .map(|sequence| sifr_frontend::WorkspaceTraceEvent {
                    sequence,
                    phase: sifr_frontend::WorkspaceTracePhase::Parse,
                    snapshot_id: None,
                    detail: "secret-user-source".into(),
                })
                .collect(),
        };
        frontend_report(&report);
        finish(1).unwrap();
        let bytes = std::fs::read(path.join("trace-v1.json")).unwrap();
        assert!(bytes.len() <= MAX_BYTES);
        assert!(!String::from_utf8_lossy(&bytes).contains("secret-user-source"));
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["dropped_reports"], 936);
        assert_eq!(value["outcome"], "failure");
        // Exercise the independent serialized-byte limit with bounded records.
        {
            let mut sink = SINK.get().unwrap().lock().unwrap();
            sink.reports = (0..MAX_REPORTS).map(|_| json!({"stages":
                (0..32).map(|i| json!({"index":i,"elapsed_us":u64::MAX})).collect::<Vec<_>>()
            })).collect();
        }
        finish(0).unwrap();
        let bytes = std::fs::read(path.join("trace-v1.json")).unwrap();
        assert!(bytes.len() <= MAX_BYTES);
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(value["dropped_reports"].as_u64().unwrap() > 936);
        assert!(start(&path, "trace", Instant::now()).is_err());
        #[cfg(target_os = "linux")]
        {
            SINK.get().unwrap().lock().unwrap().file =
                OpenOptions::new().write(true).open("/dev/full").unwrap();
            assert!(finish(0).is_err());
        }
    }
}

pub(crate) fn command_name(
    command: Option<&crate::cli_model_and_entrypoint::Commands>,
) -> &'static str {
    use crate::cli_model_and_entrypoint::Commands;
    match command {
        Some(Commands::Sysroot(_)) => "sysroot",
        Some(Commands::Cache(_)) => "cache",
        Some(Commands::Build(_)) => "build",
        Some(Commands::Run(_)) => "run",
        Some(Commands::Fetch(_)) => "fetch",
        Some(Commands::Doctor(_)) => "doctor",
        Some(Commands::Init(_)) => "init",
        Some(Commands::Repair(_)) => "repair",
        Some(Commands::Bridge(_)) => "bridge",
        Some(Commands::Python(_)) => "python",
        Some(Commands::Check(_)) => "check",
        Some(Commands::Tree(_)) => "tree",
        Some(Commands::Package(_)) => "package",
        Some(Commands::Publish(_)) => "publish",
        Some(Commands::Vendor(_)) => "vendor",
        Some(Commands::Fmt(_)) => "fmt",
        Some(Commands::Lint(_)) => "lint",
        Some(Commands::Lsp(_)) => "lsp",
        Some(Commands::Trace(_)) => "trace",
        Some(Commands::Emit(_)) => "emit",
        Some(Commands::Test(_)) => "test",
        Some(Commands::Tools(_)) => "tools",
        Some(Commands::SelfCommand(_)) => "self",
        Some(Commands::HostTool(_)) => "host-tool",
        None => "metadata",
    }
}

fn safe_identity(value: &str) -> &str {
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        value
    } else {
        "unavailable-or-redacted"
    }
}
