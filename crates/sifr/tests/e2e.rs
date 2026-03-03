//! End-to-end tests for the Sifr compiler.
//!
//! Legacy contract
//! 1. Discovery is lexicographic by fixture path.
//! 2. Expectation annotations preserve declaration order.
//! 3. Failure aggregation reports all failures and passed/failed counts.
//! 4. Pass/fail exit semantics panic on any failure in `test_e2e_pass`.
//!
//! Throughput runner contract
//! - Contract is controlled via `SIFR_E2E_RUNNER_MODE` and legacy booleans.
//! - `SIFR_E2E_SIFR_JOBS`: bounded parallel compile workers.
//! - `SIFR_E2E_RUST_JOBS`: bounded parallel build workers.
//! - `SIFR_E2E_RUN_JOBS`: bounded parallel run workers.
//! - `SIFR_E2E_CARGO_BUILD_JOBS`: cargo jobs per generated group build.
//! - `SIFR_E2E_DISABLE_CACHE=1` disables cache reuse.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const E2E_CACHE_DIR: &str = "target/sifr_e2e_cache";
const E2E_CACHE_MANIFEST: &str = "manifest.json";
const E2E_CACHE_SCHEMA_VERSION: u32 = 1;
const E2E_CACHE_ENV_ALLOWLIST: [&str; 8] = [
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTC_WRAPPER",
    "SIFR_E2E_PROFILE",
    "SIFR_E2E_RUNNER_MODE",
    "SIFR_E2E_NEW_RUNNER",
    "SIFR_E2E_LEGACY_RUNNER",
    "SIFR_E2E_CACHE_DIR",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunnerMode {
    Legacy,
    New,
    Compare,
}

#[derive(Clone, Debug)]
struct RunnerConfig {
    mode: RunnerMode,
    sifr_jobs: usize,
    rust_jobs: usize,
    run_jobs: usize,
    cargo_build_jobs: usize,
    cache: CacheConfig,
}

#[derive(Clone, Debug)]
struct CacheConfig {
    enabled: bool,
    root: PathBuf,
}

#[derive(Clone, Debug)]
struct FixtureCase {
    name: String,
    path: PathBuf,
    source: String,
    source_hash: String,
    expected_stdout: Option<String>,
    _expected_stderr: Vec<String>,
    _expected_errors: Vec<String>,
}

#[derive(Clone, Debug)]
struct CompiledCase {
    fixture: FixtureCase,
    rust_source: String,
    stdlib_modules: BTreeSet<String>,
    required_crates: BTreeSet<String>,
    _compile_duration_ms: u128,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct DependencyFingerprint {
    stdlib_modules: BTreeSet<String>,
    required_crates: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct BatchGroup {
    id: String,
    fingerprint: DependencyFingerprint,
    cases: Vec<CompiledCase>,
    generated_main: String,
    generated_rust_hash: String,
    package_name: String,
}

#[derive(Clone, Debug)]
struct FixtureExecution {
    name: String,
    status: Result<(), String>,
}

#[derive(Clone, Debug)]
struct PassReport {
    cases: Vec<FixtureExecution>,
}

impl PassReport {
    fn passed_count(&self) -> usize {
        self.cases.iter().filter(|case| case.status.is_ok()).count()
    }

    fn failed_count(&self) -> usize {
        self.cases.len().saturating_sub(self.passed_count())
    }
}

#[derive(Clone, Debug)]
struct ToolchainInfo {
    rustc_v: String,
    rustc_vv: String,
    cargo_v: String,
    target: String,
    os: String,
    arch: String,
}

impl ToolchainInfo {
    fn signature(&self) -> String {
        format!(
            "sifr={}|rustc={}|cargo={}|target={}|os={}|arch={}",
            env!("CARGO_PKG_VERSION"),
            self.rustc_vv,
            self.cargo_v,
            self.target,
            self.os,
            self.arch,
        )
    }
}

#[derive(Clone, Debug)]
struct GroupBuildOutcome {
    group: BatchGroup,
    artifact_path: Option<PathBuf>,
    build_log_path: Option<PathBuf>,
    build_error: Option<String>,
    build_ms: u128,
    cache_hit: bool,
}

#[derive(Clone, Debug)]
struct GroupRunOutcome {
    group_id: String,
    fixture_count: usize,
    cache_hit: bool,
    elapsed_ms: u128,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct CacheManifest {
    schema_version: u32,
    entries: BTreeMap<String, CacheEntry>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct CacheEntry {
    schema_version: u32,
    cache_key: String,
    group_id: String,
    group_fingerprint: String,
    group_rust_hash: String,
    fixture_sources: Vec<FixtureSourceHash>,
    compiler_signature: String,
    rustc_v: String,
    rustc_vv: String,
    cargo_v: String,
    target: String,
    os: String,
    arch: String,
    env_signature: String,
    artifact_path: String,
    build_log_path: Option<String>,
    built_at_unix_secs: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct FixtureSourceHash {
    fixture: String,
    hash: String,
}

struct CommandCapture {
    status_ok: bool,
    stdout: String,
    stderr: String,
}

fn run_capture(mut command: Command) -> CommandCapture {
    match command.output() {
        Ok(output) => CommandCapture {
            status_ok: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(error) => CommandCapture {
            status_ok: false,
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

fn command_with_capture(program: &str, args: &[&str], cwd: Option<&Path>) -> CommandCapture {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(path) = cwd {
        command.current_dir(path);
    }
    run_capture(command)
}

fn parse_bool_env(raw: &str) -> Result<bool, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "on" | "yes" | "y" => Ok(true),
        "0" | "false" | "off" | "no" | "n" => Ok(false),
        other => Err(format!(
            "invalid boolean env value '{other}', expected 1/0, true/false, on/off, yes/no"
        )),
    }
}

fn parse_runner_mode(raw: &str) -> Result<RunnerMode, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "legacy" => Ok(RunnerMode::Legacy),
        "new" => Ok(RunnerMode::New),
        "compare" => Ok(RunnerMode::Compare),
        _ => Err(format!(
            "unsupported SIFR_E2E_RUNNER_MODE value '{raw}', expected legacy|new|compare"
        )),
    }
}

fn parse_runner_mode_from_env(
    mode: Option<&str>,
    new_runner: Option<&str>,
    legacy_runner: Option<&str>,
) -> Result<RunnerMode, String> {
    let explicit = mode.map(parse_runner_mode).transpose()?;
    let new_runner = parse_optional_bool_env_value(new_runner)?;
    let legacy_runner = parse_optional_bool_env_value(legacy_runner)?;

    if matches!(new_runner, Some(true)) && matches!(legacy_runner, Some(true)) {
        return Err("conflicting SIFR_E2E_NEW_RUNNER and SIFR_E2E_LEGACY_RUNNER".to_string());
    }

    if let Some(value) = explicit {
        return Ok(value);
    }
    if matches!(new_runner, Some(true)) {
        return Ok(RunnerMode::New);
    }
    if matches!(legacy_runner, Some(true)) {
        return Ok(RunnerMode::Legacy);
    }
    Ok(RunnerMode::Legacy)
}

fn parse_optional_bool_env_value(value: Option<&str>) -> Result<Option<bool>, String> {
    match value {
        Some(raw) => Ok(Some(parse_bool_env(raw)?)),
        None => Ok(None),
    }
}

fn parse_positive_usize(value: Option<&str>, default: usize) -> usize {
    match value.and_then(|raw| raw.parse::<usize>().ok()) {
        Some(value) if value > 0 => value,
        _ => default,
    }
}

fn cache_root_from_env(raw: Option<&str>) -> PathBuf {
    match raw.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => Path::new(E2E_CACHE_DIR).to_path_buf(),
    }
}

fn runner_config() -> Result<RunnerConfig, String> {
    let mode = parse_runner_mode_from_env(
        env::var("SIFR_E2E_RUNNER_MODE").ok().as_deref(),
        env::var("SIFR_E2E_NEW_RUNNER").ok().as_deref(),
        env::var("SIFR_E2E_LEGACY_RUNNER").ok().as_deref(),
    )?;

    let available_workers = std::thread::available_parallelism()
        .map(|v| v.get())
        .unwrap_or(1);

    let sifr_jobs = parse_positive_usize(
        env::var("SIFR_E2E_SIFR_JOBS").ok().as_deref(),
        available_workers,
    );
    let rust_jobs = parse_positive_usize(
        env::var("SIFR_E2E_RUST_JOBS").ok().as_deref(),
        available_workers,
    );
    let run_jobs = parse_positive_usize(
        env::var("SIFR_E2E_RUN_JOBS").ok().as_deref(),
        available_workers,
    );
    let cargo_build_jobs =
        parse_positive_usize(env::var("SIFR_E2E_CARGO_BUILD_JOBS").ok().as_deref(), 1);

    let cache_enabled = !matches!(
        env::var("SIFR_E2E_DISABLE_CACHE")
            .ok()
            .as_deref()
            .and_then(|raw| parse_bool_env(raw).ok()),
        Some(true)
    );
    let cache_root = cache_root_from_env(env::var("SIFR_E2E_CACHE_DIR").ok().as_deref());

    Ok(RunnerConfig {
        mode,
        sifr_jobs,
        rust_jobs,
        run_jobs,
        cargo_build_jobs,
        cache: CacheConfig {
            enabled: cache_enabled,
            root: cache_root,
        },
    })
}

fn cache_dir(root: &Path) -> PathBuf {
    root.to_path_buf()
}

fn cache_manifest_path(root: &Path) -> PathBuf {
    cache_dir(root).join(E2E_CACHE_MANIFEST)
}

fn deterministic_hash(value: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn join_sorted(values: &BTreeSet<String>) -> String {
    values.iter().cloned().collect::<Vec<_>>().join("|")
}

/// Collect all `# expect-stdout: <value>` lines into a multi-line expected output.
fn extract_expect_stdout(source: &str) -> Option<String> {
    let lines: Vec<&str> = source
        .lines()
        .filter_map(|line| {
            line.strip_prefix("# expect-stdout:")
                .map(|rest| rest.trim())
        })
        .collect();

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Collect all `# expect-stderr: <value>` lines.
fn extract_expect_stderr(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            line.strip_prefix("# expect-stderr:")
                .map(|rest| rest.trim().to_string())
        })
        .collect()
}

/// Extract expected error substrings from `# expect-error: <value>` comments.
fn extract_expect_errors(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            line.strip_prefix("# expect-error:")
                .map(|rest| rest.trim().to_string())
        })
        .collect()
}

fn path_to_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

fn normalize_dependency_set<I, T>(values: I) -> BTreeSet<String>
where
    I: IntoIterator<Item = T>,
    T: Into<String>,
{
    values.into_iter().map(Into::into).collect()
}

fn sanitize_identifier(value: &str) -> String {
    let mut value = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();

    if value.is_empty() || value.chars().next().unwrap_or('_').is_ascii_digit() {
        value.insert(0, '_');
    }

    value
}

fn package_name_from_id(id: &str) -> String {
    let mut name = format!("sifr_batch_{id}");
    name = sanitize_identifier(&name).to_ascii_lowercase();
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        name.insert(0, 's');
    }
    name
}

fn fixture_module_name(fixture_name: &str, index: usize) -> String {
    let safe = sanitize_identifier(fixture_name);
    let suffix = deterministic_hash(&format!("{fixture_name}-{index}"));
    format!("sifr_case_{safe}_{}_{}", index, &suffix[..8])
}

fn fixture_entry_name(module_name: &str) -> String {
    format!("{module_name}_main")
}

fn infer_dependencies(
    rust_source: &str,
    stdlib_modules: &BTreeSet<String>,
    required_crates: &BTreeSet<String>,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut modules = stdlib_modules.clone();
    let mut crates = required_crates.clone();

    if rust_source.contains("num_bigint::BigInt") || rust_source.contains("use num_bigint") {
        modules.insert("_bigint".to_string());
    }
    if rust_source.contains("regex::") {
        crates.insert("regex".to_string());
    }
    if rust_source.contains("rand::thread_rng") {
        crates.insert("rand".to_string());
    }
    if rust_source.contains("rand_distr::") {
        crates.insert("rand_distr".to_string());
    }
    if rust_source.contains("sha1::") {
        crates.insert("sha1".to_string());
    }
    if rust_source.contains("sha2::") {
        crates.insert("sha2".to_string());
    }
    if rust_source.contains("blake2::") {
        crates.insert("blake2".to_string());
    }

    (modules, crates)
}

impl DependencyFingerprint {
    fn signature(&self) -> String {
        format!(
            "stdlib={}|crates={}",
            join_sorted(&self.stdlib_modules),
            join_sorted(&self.required_crates),
        )
    }

    fn hash(&self) -> String {
        deterministic_hash(&self.signature())
    }
}

impl FixtureCase {
    fn module_name(&self, ordinal: usize) -> String {
        fixture_module_name(&self.name, ordinal)
    }
}

impl CompiledCase {
    fn dependency_fingerprint(&self) -> DependencyFingerprint {
        DependencyFingerprint {
            stdlib_modules: self.stdlib_modules.clone(),
            required_crates: self.required_crates.clone(),
        }
    }
}

fn compile_source(source: &str) -> Result<String, Vec<String>> {
    match sifr_driver::compile(source) {
        sifr_driver::CompileResult::Success { rust_source } => Ok(rust_source),
        sifr_driver::CompileResult::Errors { errors } => {
            Err(errors.iter().map(|error| error.message.clone()).collect())
        }
    }
}

fn compile_source_with_metadata(
    source: &str,
) -> Result<(String, HashSet<String>, HashSet<String>), Vec<String>> {
    match sifr_driver::compile_with_metadata(source) {
        sifr_driver::CompileResultFull::Success {
            rust_source,
            used_stdlib_modules,
            required_crates,
            ..
        } => Ok((rust_source, used_stdlib_modules, required_crates)),
        sifr_driver::CompileResultFull::Errors { errors } => {
            Err(errors.iter().map(|error| error.message.clone()).collect())
        }
    }
}

fn compile_source_with_metadata_and_stats(
    source: &str,
) -> Result<
    (
        String,
        HashSet<String>,
        HashSet<String>,
        sifr_driver::LoweringStats,
    ),
    Vec<String>,
> {
    match sifr_driver::compile_with_metadata(source) {
        sifr_driver::CompileResultFull::Success {
            rust_source,
            used_stdlib_modules,
            required_crates,
            lowering_stats,
        } => Ok((
            rust_source,
            used_stdlib_modules,
            required_crates,
            lowering_stats,
        )),
        sifr_driver::CompileResultFull::Errors { errors } => {
            Err(errors.iter().map(|error| error.message.clone()).collect())
        }
    }
}

fn discover_fixtures(base_dir: &Path) -> Vec<FixtureCase> {
    if !base_dir.exists() {
        return Vec::new();
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(base_dir)
        .into_iter()
        .flat_map(|read| read.filter_map(Result::ok))
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "sifr") {
            match std::fs::read_to_string(&path) {
                Ok(source) => {
                    let name = path_to_name(&path);
                    let source_hash = deterministic_hash(&source);
                    entries.push(FixtureCase {
                        name,
                        path,
                        source: source.clone(),
                        source_hash,
                        expected_stdout: extract_expect_stdout(&source),
                        _expected_stderr: extract_expect_stderr(&source),
                        _expected_errors: extract_expect_errors(&source),
                    });
                }
                Err(err) => {
                    eprintln!("[sifr-e2e] skipping fixture read error: {}", err);
                }
            }
        }
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));
    entries
}

fn build_rust_source_from_module(raw: &str, entry_fn: &str) -> Result<String, String> {
    let marker = "fn main(";
    let index = raw
        .find(marker)
        .ok_or_else(|| "generated fixture Rust source does not define fn main".to_string())?;
    let after = &raw[index + marker.len()..];
    let prefix = &raw[..index];
    Ok(format!("{prefix}fn {entry_fn}({after}"))
}

fn compile_fixture(fixture: &FixtureCase) -> Result<CompiledCase, String> {
    let started = Instant::now();
    let (rust_source, stdlib_modules, required_crates) =
        compile_source_with_metadata(&fixture.source)
            .map_err(|errors| format!("sifr compilation failed:\n  {}", errors.join("\n  ")))?;

    let stdlib_modules = infer_dependencies(
        &rust_source,
        &normalize_dependency_set(stdlib_modules),
        &BTreeSet::new(),
    )
    .0;

    let required_crates = infer_dependencies(
        &rust_source,
        &BTreeSet::new(),
        &normalize_dependency_set(required_crates),
    )
    .1;

    if !rust_source.contains("fn main(") {
        return Err("generated Rust has no main function".to_string());
    }

    Ok(CompiledCase {
        fixture: fixture.clone(),
        rust_source,
        stdlib_modules,
        required_crates,
        _compile_duration_ms: started.elapsed().as_millis(),
    })
}

fn compile_suite_parallel(
    fixtures: &[FixtureCase],
    workers: usize,
) -> Vec<(FixtureCase, Result<CompiledCase, String>)> {
    run_in_parallel(fixtures, workers, |fixture| {
        let result = compile_fixture(fixture);
        (fixture.clone(), result)
    })
}

fn generate_cargo_toml(
    stdlib_modules: &BTreeSet<String>,
    required_crates: &BTreeSet<String>,
    package_name: &str,
) -> String {
    let mut contents = format!(
        "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n"
    );

    let mut deps = BTreeSet::new();
    for module_name in stdlib_modules {
        match module_name.as_str() {
            "sifr.json" | "sifr.collections" | "_sifr.json" | "_sifr.collections" => {
                deps.insert("serde_json = \"1\"".to_string());
                deps.insert("serde = { version = \"1\", features = [\"derive\"] }".to_string());
            }
            "sifr.time" | "_sifr.time" | "sifr.datetime" | "_sifr.datetime" => {
                deps.insert("chrono = \"0.4\"".to_string());
            }
            "sifr.random" | "_sifr.crypto" | "sifr.uuid" | "_sifr.uuid" => {
                deps.insert("rand = \"0.8\"".to_string());
                deps.insert("rand_distr = \"0.4\"".to_string());
            }
            "sifr.re" | "_sifr.regex" => {
                deps.insert("regex = \"1\"".to_string());
            }
            "sifr.hash" | "sifr.hashlib" => {
                deps.insert("sha2 = \"0.10\"".to_string());
                deps.insert("md5 = \"0.7\"".to_string());
                deps.insert("sha1 = \"0.10\"".to_string());
                deps.insert("blake2 = \"0.10\"".to_string());
            }
            "sifr.encoding" | "sifr.base64" => {
                deps.insert("base64 = \"0.22\"".to_string());
            }
            "sifr.tomllib" | "_sifr.toml" => {
                deps.insert("toml = \"0.8\"".to_string());
            }
            "sifr.gzip" | "sifr.zipfile" | "_sifr.compress" => {
                deps.insert("flate2 = \"1\"".to_string());
                deps.insert("zip = \"0.6\"".to_string());
            }
            "_bigint" => {
                deps.insert("num-bigint = \"0.4\"".to_string());
                deps.insert("num-traits = \"0.2\"".to_string());
            }
            _ => {}
        }
    }

    for crate_name in required_crates {
        match crate_name.as_str() {
            "serde_json" => {
                deps.insert("serde_json = \"1\"".to_string());
                deps.insert("serde = { version = \"1\", features = [\"derive\"] }".to_string());
            }
            "chrono" => {
                deps.insert("chrono = \"0.4\"".to_string());
            }
            "rand" => {
                deps.insert("rand = \"0.8\"".to_string());
            }
            "rand_distr" => {
                deps.insert("rand_distr = \"0.4\"".to_string());
            }
            "regex" => {
                deps.insert("regex = \"1\"".to_string());
            }
            "sha2" => {
                deps.insert("sha2 = \"0.10\"".to_string());
            }
            "md5" => {
                deps.insert("md5 = \"0.7\"".to_string());
            }
            "sha1" => {
                deps.insert("sha1 = \"0.10\"".to_string());
            }
            "blake2" => {
                deps.insert("blake2 = \"0.10\"".to_string());
            }
            "base64" => {
                deps.insert("base64 = \"0.22\"".to_string());
            }
            "toml" => {
                deps.insert("toml = \"0.8\"".to_string());
            }
            "flate2" => {
                deps.insert("flate2 = \"1\"".to_string());
            }
            "zip" => {
                deps.insert("zip = \"0.6\"".to_string());
            }
            "num-bigint" => {
                deps.insert("num-bigint = \"0.4\"".to_string());
            }
            "num-traits" => {
                deps.insert("num-traits = \"0.2\"".to_string());
            }
            _ => {}
        }
    }

    if !deps.is_empty() {
        contents.push_str("[dependencies]\n");
        for dep in deps {
            contents.push_str(&dep);
            contents.push('\n');
        }
    }

    // Keep generated grouped crates outside of the parent workspace to avoid Cargo
    // interpreting them as non-members of the existing workspace.
    contents.push_str("\n[workspace]\n");

    contents
}

fn make_entry_function_public(source: &str, entry_fn: &str) -> String {
    let marker = format!("fn {}(", entry_fn);
    let mut output = String::new();
    let mut changed = false;

    for line in source.lines() {
        let mut indent_len = 0;
        for (index, ch) in line.char_indices() {
            if ch == ' ' || ch == '\t' {
                indent_len = index + ch.len_utf8();
                continue;
            }
            break;
        }

        let indent = &line[..indent_len];
        let body = &line[indent_len..];
        if body.starts_with(&marker) {
            output.push_str(indent);
            output.push_str("pub ");
            output.push_str(body);
            output.push('\n');
            changed = true;
            continue;
        }
        output.push_str(line);
        output.push('\n');
    }

    if changed {
        output
    } else {
        source.to_string()
    }
}

fn build_group_sources(group_cases: Vec<CompiledCase>) -> Result<BatchGroup, String> {
    let mut cases = group_cases;
    cases.sort_by(|left, right| left.fixture.name.cmp(&right.fixture.name));

    let fingerprint = cases
        .first()
        .map(|case| case.dependency_fingerprint())
        .unwrap_or_else(|| DependencyFingerprint {
            stdlib_modules: BTreeSet::new(),
            required_crates: BTreeSet::new(),
        });

    let mut case_modules = Vec::with_capacity(cases.len());
    let mut generated_modules = String::new();
    let mut case_signature = String::new();

    for (ordinal, case) in cases.iter().enumerate() {
        let module_name = case.fixture.module_name(ordinal);
        let entry_fn = fixture_entry_name(&module_name);
        let rust_source = build_rust_source_from_module(&case.rust_source, &entry_fn)
            .map_err(|err| format!("fixture {}: {}", case.fixture.name, err))?;

        generated_modules.push_str(&format!("pub mod {module_name} {{\n"));
        generated_modules.push_str(&make_entry_function_public(&rust_source, &entry_fn));
        generated_modules.push('\n');
        generated_modules.push_str("}\n\n");

        case_signature.push_str(&format!(
            "{}:{}",
            case.fixture.name, case.fixture.source_hash
        ));
        case_signature.push('|');

        case_modules.push((case.fixture.name.clone(), module_name, entry_fn));
    }

    let _union_stdlib = cases
        .iter()
        .flat_map(|case| case.stdlib_modules.iter().cloned())
        .collect::<BTreeSet<_>>();
    let _union_crates = cases
        .iter()
        .flat_map(|case| case.required_crates.iter().cloned())
        .collect::<BTreeSet<_>>();

    let mut generated_main = String::new();
    generated_main.push_str("fn usage() -> ! {\n");
    generated_main.push_str("    eprintln!(\"usage: --case <fixture_name>\");\n");
    generated_main.push_str("    std::process::exit(2);\n");
    generated_main.push_str("}\n\n");
    generated_main.push_str("fn main() {\n");
    generated_main.push_str("    let mut args = std::env::args().skip(1);\n");
    generated_main.push_str("    let flag = args.next();\n");
    generated_main.push_str("    let case = args.next();\n");
    generated_main.push_str(
        "    if flag.as_deref() != Some(\"--case\") || case.is_none() || args.next().is_some() {\n",
    );
    generated_main.push_str("        usage();\n");
    generated_main.push_str("    }\n");
    generated_main.push_str("    match case.as_deref().expect(\"case\") {\n");
    for module in &case_modules {
        generated_main.push_str(&format!(
            "        \"{}\" => {}::{}(),\n",
            module.0, module.1, module.2
        ));
    }
    generated_main.push_str("        other => {\n");
    generated_main.push_str("            eprintln!(\"--case must be one of: \");\n");
    generated_main.push_str("            eprintln!(\"  {}\", other);\n");
    generated_main.push_str("            std::process::exit(2);\n");
    generated_main.push_str("        }\n");
    generated_main.push_str("    }\n");
    generated_main.push_str("}\n");

    let generated_rust = format!("{}{}", generated_modules, generated_main);
    let generated_rust_hash = deterministic_hash(&generated_rust);
    let raw_group_signature = format!(
        "{}|{}|{}",
        fingerprint.signature(),
        case_signature,
        generated_rust_hash,
    );
    let id = deterministic_hash(&raw_group_signature);
    let package_name = package_name_from_id(&id);

    Ok(BatchGroup {
        id,
        fingerprint,
        cases,
        generated_main: generated_rust,
        generated_rust_hash,
        package_name,
    })
}

fn plan_batches(compiled_cases: Vec<CompiledCase>) -> (Vec<BatchGroup>, Vec<FixtureExecution>) {
    let mut buckets: BTreeMap<DependencyFingerprint, Vec<CompiledCase>> = BTreeMap::new();
    for case in compiled_cases {
        let fp = case.dependency_fingerprint();
        buckets.entry(fp).or_default().push(case);
    }

    let mut groups = Vec::with_capacity(buckets.len());
    let mut planning_failures = Vec::new();
    for (_, cases) in buckets {
        let fixture_names = cases
            .iter()
            .map(|case| case.fixture.name.clone())
            .collect::<Vec<_>>();
        match build_group_sources(cases) {
            Ok(group) => groups.push(group),
            Err(err) => {
                for fixture_name in fixture_names {
                    planning_failures.push(FixtureExecution {
                        name: fixture_name.clone(),
                        status: Err(format!(
                            "FAIL [{}]: failed to generate grouped crate source: {}",
                            fixture_name, err
                        )),
                    });
                }
            }
        }
    }

    groups.sort_by(|left, right| left.id.cmp(&right.id));
    planning_failures.sort_by(|left, right| left.name.cmp(&right.name));
    (groups, planning_failures)
}

fn cache_env_signature() -> String {
    let mut values = Vec::with_capacity(E2E_CACHE_ENV_ALLOWLIST.len());
    for key in &E2E_CACHE_ENV_ALLOWLIST {
        values.push(format!(
            "{key}={}",
            env::var(key).unwrap_or_else(|_| "<unset>".to_string())
        ));
    }
    deterministic_hash(&values.join("\0"))
}

fn read_cache_manifest(root: &Path) -> CacheManifest {
    let manifest_path = cache_manifest_path(root);
    let raw = std::fs::read_to_string(&manifest_path).ok();
    match raw {
        Some(raw) => serde_json::from_str::<CacheManifest>(&raw).unwrap_or_else(|err| {
            eprintln!("[sifr-e2e-cache] manifest parse failed: {err}; rebuilding cache manifest");
            CacheManifest {
                schema_version: E2E_CACHE_SCHEMA_VERSION,
                entries: BTreeMap::new(),
            }
        }),
        None => CacheManifest {
            schema_version: E2E_CACHE_SCHEMA_VERSION,
            entries: BTreeMap::new(),
        },
    }
}

fn write_cache_manifest(root: &Path, manifest: &CacheManifest) {
    if let Err(err) = std::fs::create_dir_all(root) {
        eprintln!("[sifr-e2e-cache] cannot create cache dir: {err}");
        return;
    }
    let manifest_path = cache_manifest_path(root);
    let content = match serde_json::to_string_pretty(manifest) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("[sifr-e2e-cache] cannot serialize manifest: {err}");
            return;
        }
    };

    if let Err(err) = std::fs::write(manifest_path, content) {
        eprintln!("[sifr-e2e-cache] cannot persist manifest: {err}");
    }
}

fn toolchain_info() -> ToolchainInfo {
    let rustc_version = command_with_capture("rustc", &["-V"], None).stdout;
    let rustc_vv = command_with_capture("rustc", &["-Vv"], None).stdout;
    let cargo_v = command_with_capture("cargo", &["-V"], None).stdout;

    let mut target = "unknown-target".to_string();
    for line in rustc_vv.lines() {
        if let Some((label, value)) = line.split_once(':') {
            if label == "host" {
                target = value.trim().to_string();
            }
        }
    }

    ToolchainInfo {
        rustc_v: rustc_version.trim().to_string(),
        rustc_vv: rustc_vv.trim().to_string(),
        cargo_v: cargo_v.trim().to_string(),
        target,
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
    }
}

fn cache_key_for_group(
    group: &BatchGroup,
    toolchain: &ToolchainInfo,
    env_signature: &str,
) -> String {
    let fixture_signature = group
        .cases
        .iter()
        .map(|case| format!("{}:{}", case.fixture.name, case.fixture.source_hash))
        .collect::<Vec<_>>()
        .join("|");
    deterministic_hash(&format!(
        "schema={}\nfp={}\ngen={}\nsig={}\nfixture={}\n{}",
        E2E_CACHE_SCHEMA_VERSION,
        group.fingerprint.signature(),
        group.generated_rust_hash,
        toolchain.signature(),
        fixture_signature,
        env_signature,
    ))
}

fn cache_entry_valid(
    entry: &CacheEntry,
    group: &BatchGroup,
    cache_key: &str,
    toolchain: &ToolchainInfo,
    env_signature: &str,
) -> bool {
    if entry.schema_version != E2E_CACHE_SCHEMA_VERSION {
        return false;
    }
    if entry.cache_key != cache_key || entry.group_id != group.id {
        return false;
    }
    if entry.group_fingerprint != group.fingerprint.signature()
        || entry.group_rust_hash != group.generated_rust_hash
    {
        return false;
    }
    if entry.compiler_signature != toolchain.signature()
        || entry.rustc_v != toolchain.rustc_v
        || entry.rustc_vv != toolchain.rustc_vv
        || entry.cargo_v != toolchain.cargo_v
        || entry.target != toolchain.target
        || entry.os != toolchain.os
        || entry.arch != toolchain.arch
        || entry.env_signature != env_signature
    {
        return false;
    }

    if group.cases.len() != entry.fixture_sources.len() {
        return false;
    }

    for (case, saved) in group.cases.iter().zip(entry.fixture_sources.iter()) {
        if case.fixture.name != saved.fixture || case.fixture.source_hash != saved.hash {
            return false;
        }
    }

    Path::new(&entry.artifact_path).exists()
}

fn build_group_binary_path(group_root: &Path, package_name: &str) -> PathBuf {
    let debug_dir = group_root.join("target").join("debug");
    if cfg!(target_os = "windows") {
        debug_dir.join(format!("{package_name}.exe"))
    } else {
        debug_dir.join(package_name)
    }
}

fn build_batch_group(
    group: BatchGroup,
    config: &RunnerConfig,
    toolchain: &ToolchainInfo,
    env_signature: &str,
    manifest: &Arc<Mutex<CacheManifest>>,
) -> GroupBuildOutcome {
    let started = Instant::now();
    let group_root = config.cache.root.join("groups").join(&group.id);
    let mut build_error = None;
    let mut build_log = None;
    let mut artifact = None;
    let mut cache_hit = false;

    if let Err(err) = std::fs::create_dir_all(group_root.join("src")) {
        build_error = Some(format!("failed to create batch crate dir: {err}"));
    }

    let cache_key = cache_key_for_group(&group, toolchain, env_signature);
    let cached_entry = if config.cache.enabled {
        manifest
            .lock()
            .ok()
            .and_then(|stored| stored.entries.get(&cache_key).cloned())
            .filter(|entry| cache_entry_valid(entry, &group, &cache_key, toolchain, env_signature))
    } else {
        None
    };

    if let Some(entry) = cached_entry {
        artifact = Some(PathBuf::from(entry.artifact_path));
        build_log = entry.build_log_path.map(PathBuf::from);
        cache_hit = true;
        return GroupBuildOutcome {
            group,
            artifact_path: artifact,
            build_log_path: build_log,
            build_error: None,
            build_ms: started.elapsed().as_millis(),
            cache_hit,
        };
    }

    if build_error.is_none() {
        let stdlib_union = group
            .cases
            .iter()
            .flat_map(|case| case.stdlib_modules.iter().cloned())
            .collect::<BTreeSet<_>>();
        let crate_union = group
            .cases
            .iter()
            .flat_map(|case| case.required_crates.iter().cloned())
            .collect::<BTreeSet<_>>();
        let cargo_toml = generate_cargo_toml(&stdlib_union, &crate_union, &group.package_name);
        let source_path = group_root.join("src").join("main.rs");
        let cargo_toml_path = group_root.join("Cargo.toml");

        if let Err(err) = std::fs::write(&cargo_toml_path, cargo_toml) {
            build_error = Some(format!("failed to write Cargo.toml: {err}"));
        } else if let Err(err) = std::fs::write(&source_path, &group.generated_main) {
            build_error = Some(format!("failed to write main.rs: {err}"));
        } else {
            let mut build_command = Command::new("cargo");
            build_command
                .args(["build", "--quiet", "-j"])
                .arg(config.cargo_build_jobs.to_string())
                .current_dir(&group_root);
            let build_capture = run_capture(build_command);
            if !build_capture.status_ok {
                let log_path = group_root.join("build.log");
                let mut diagnostic = String::new();
                let _ = std::fmt::Write::write_str(
                    &mut diagnostic,
                    &format!(
                        "Rust build failed for {} ({})\n\nSTDOUT:\n{}\n\nSTDERR:\n{}\n",
                        group.id,
                        group.fingerprint.hash(),
                        build_capture.stdout,
                        build_capture.stderr
                    ),
                );
                let _ = std::fmt::Write::write_str(&mut diagnostic, "Generated Rust:\n");
                let _ = std::fmt::Write::write_str(&mut diagnostic, &group.generated_main);
                if let Err(err) = std::fs::write(&log_path, diagnostic) {
                    eprintln!("[sifr-e2e-cache] failed to write build log: {err}");
                }
                build_error = Some(format!(
                    "Rust compilation failed. Check build log: {}",
                    log_path.display()
                ));
                build_log = Some(log_path);
            } else {
                artifact = Some(build_group_binary_path(&group_root, &group.package_name));
                if let Some(path) = &artifact {
                    if config.cache.enabled {
                        let built_at_unix_secs = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|dur| dur.as_secs())
                            .unwrap_or(0);
                        let entry = CacheEntry {
                            schema_version: E2E_CACHE_SCHEMA_VERSION,
                            cache_key: cache_key.clone(),
                            group_id: group.id.clone(),
                            group_fingerprint: group.fingerprint.signature(),
                            group_rust_hash: group.generated_rust_hash.clone(),
                            fixture_sources: group
                                .cases
                                .iter()
                                .map(|case| FixtureSourceHash {
                                    fixture: case.fixture.name.clone(),
                                    hash: case.fixture.source_hash.clone(),
                                })
                                .collect(),
                            compiler_signature: toolchain.signature(),
                            rustc_v: toolchain.rustc_v.clone(),
                            rustc_vv: toolchain.rustc_vv.clone(),
                            cargo_v: toolchain.cargo_v.clone(),
                            target: toolchain.target.clone(),
                            os: toolchain.os.clone(),
                            arch: toolchain.arch.clone(),
                            env_signature: env_signature.to_string(),
                            artifact_path: path.to_string_lossy().to_string(),
                            build_log_path: build_log
                                .as_ref()
                                .map(|path| path.to_string_lossy().to_string()),
                            built_at_unix_secs,
                        };
                        if let Ok(mut manifest_lock) = manifest.lock() {
                            manifest_lock.entries.insert(cache_key.clone(), entry);
                        }
                    }
                }
            }
        }
    }

    GroupBuildOutcome {
        group,
        artifact_path: artifact,
        build_log_path: build_log,
        build_error,
        build_ms: started.elapsed().as_millis(),
        cache_hit,
    }
}

fn build_batch_suite(
    groups: Vec<BatchGroup>,
    config: &RunnerConfig,
    toolchain: &ToolchainInfo,
    env_signature: &str,
    manifest: &CacheManifest,
) -> (Vec<GroupBuildOutcome>, CacheManifest) {
    let cache_root = manifest.entries.len();
    let _ = cache_root;
    let shared_manifest = Arc::new(Mutex::new(manifest.clone()));
    let outcomes = run_in_parallel(&groups, config.rust_jobs, |group| {
        build_batch_group(
            group.clone(),
            config,
            toolchain,
            env_signature,
            &shared_manifest,
        )
    });

    let next_manifest = shared_manifest
        .lock()
        .map_or_else(|_| manifest.clone(), |lock| lock.clone());
    if config.cache.enabled {
        write_cache_manifest(&config.cache.root, &next_manifest);
    }

    (outcomes, next_manifest)
}

fn run_single_case(
    artifact_path: &Path,
    fixture_name: &str,
    expected_stdout: Option<&String>,
) -> Result<String, String> {
    let args = ["--case", fixture_name];
    let run_capture = command_with_capture(
        artifact_path
            .to_str()
            .unwrap_or_else(|| "sifr_batch_binary"),
        &args,
        None,
    );
    if !run_capture.status_ok {
        return Err(format!("binary exited with error:\n{}", run_capture.stderr));
    }

    let actual = run_capture.stdout;
    if let Some(expected) = expected_stdout {
        let expected = expected.trim_end();
        let actual = actual.trim_end();
        if expected != actual {
            return Err(format!(
                "stdout mismatch\n  expected: {:?}\n  actual:   {:?}",
                expected, actual
            ));
        }
    }

    Ok(actual)
}

fn run_batch_outcomes(group_outcome: &GroupBuildOutcome) -> Vec<FixtureExecution> {
    let group = &group_outcome.group;
    let fixture_names = group
        .cases
        .iter()
        .map(|case| case.fixture.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    if let Some(error) = &group_outcome.build_error {
        return group
            .cases
            .iter()
            .map(|case| FixtureExecution {
                name: case.fixture.name.clone(),
                status: Err(format!(
                    "FAIL [{}]: {}\n  group: {}\n  group fixture list: [{}]\n  group fingerprint: {}\n  crate: {}\n  build log: {}",
                    case.fixture.name,
                    error,
                    group.id,
                    fixture_names,
                    group.fingerprint.hash(),
                    config_cache_root().join("groups").join(&group.id).display(),
                    group_outcome
                        .build_log_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "<none>".to_string())
                )),
            })
            .collect();
    }

    let artifact = match &group_outcome.artifact_path {
        Some(artifact) => artifact.clone(),
        None => {
            let message = group_outcome
                .build_error
                .as_deref()
                .unwrap_or("missing batch artifact");
            return group
                .cases
                .iter()
                .map(|case| FixtureExecution {
                    name: case.fixture.name.clone(),
                    status: Err(format!(
                        "FAIL [{}]: {}\n  group: {}\n  group fingerprint: {}\n  crate: {}",
                        case.fixture.name,
                        message,
                        group.id,
                        group.fingerprint.hash(),
                        config_cache_root().join("groups").join(&group.id).display(),
                    )),
                })
                .collect();
        }
    };

    group
        .cases
        .iter()
        .map(|case| {
            let status =
                match run_single_case(&artifact, &case.fixture.name, case.fixture.expected_stdout.as_ref()) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(format!(
                        "FAIL [{}]: {}\n  group: {}\n  group fingerprint: {}\n  crate: {}\n  artifact: {}",
                        case.fixture.name,
                        err,
                        group.id,
                        group.fingerprint.hash(),
                        config_cache_root().join("groups").join(&group.id).display(),
                        artifact.display(),
                    )),
                };
            FixtureExecution {
                name: case.fixture.name.clone(),
                status: status,
            }
        })
        .collect()
}

fn run_batch_suite(
    build_outcomes: &[GroupBuildOutcome],
    config: &RunnerConfig,
) -> (Vec<FixtureExecution>, Vec<GroupRunOutcome>) {
    let mut outputs = Vec::new();
    let mut run_outcomes = Vec::with_capacity(build_outcomes.len());

    let per_group = run_in_parallel(build_outcomes, config.run_jobs, |group| {
        let started = Instant::now();
        let results = run_batch_outcomes(group);
        (
            GroupRunOutcome {
                group_id: group.group.id.clone(),
                fixture_count: group.group.cases.len(),
                cache_hit: group.cache_hit,
                elapsed_ms: started.elapsed().as_millis(),
            },
            results,
        )
    });

    for (outcome, results) in per_group {
        outputs.extend(results);
        run_outcomes.push(outcome);
    }

    outputs.sort_by(|left, right| left.name.cmp(&right.name));
    (outputs, run_outcomes)
}

fn build_and_run_capture_with_deps(
    rust_source: &str,
    test_name: &str,
    stdlib_modules: &HashSet<String>,
    required_crates: &HashSet<String>,
) -> Result<(String, String, bool), String> {
    let tmp_dir = env::temp_dir().join("sifr_e2e_tests").join(test_name);
    let src_dir = tmp_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|err| format!("failed to create dir: {err}"))?;

    let mut modules = normalize_dependency_set(stdlib_modules);
    let mut crates = normalize_dependency_set(required_crates);
    let (inferred_modules, inferred_crates) = infer_dependencies(rust_source, &modules, &crates);
    modules = inferred_modules;
    crates = inferred_crates;

    let cargo_toml = generate_cargo_toml(&modules, &crates, "sifr_output");
    std::fs::write(tmp_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|err| format!("failed to write Cargo.toml: {err}"))?;
    std::fs::write(src_dir.join("main.rs"), rust_source)
        .map_err(|err| format!("failed to write main.rs: {err}"))?;

    let build_capture = command_with_capture("cargo", &["build", "--quiet"], Some(&tmp_dir));
    if !build_capture.status_ok {
        return Err(format!(
            "Rust compilation failed.\n\nGenerated Rust:\n{}\n\nrustc errors:\n{}",
            rust_source, build_capture.stderr
        ));
    }

    let binary_name = if cfg!(target_os = "windows") {
        "sifr_output.exe"
    } else {
        "sifr_output"
    };
    let binary_path = tmp_dir.join("target").join("debug").join(binary_name);
    let run_capture = command_with_capture(
        binary_path.to_str().unwrap_or("sifr_output"),
        &[],
        Some(&tmp_dir),
    );
    Ok((
        run_capture.stdout,
        run_capture.stderr,
        run_capture.status_ok,
    ))
}

fn build_and_run_with_deps(
    rust_source: &str,
    test_name: &str,
    stdlib_modules: &HashSet<String>,
    required_crates: &HashSet<String>,
) -> Result<String, String> {
    match build_and_run_capture_with_deps(rust_source, test_name, stdlib_modules, required_crates) {
        Ok((stdout, stderr, status_ok)) => {
            if status_ok {
                Ok(stdout)
            } else {
                Err(format!("binary exited with error:\n{stderr}"))
            }
        }
        Err(err) => Err(err),
    }
}

fn run_in_parallel<T, R, F>(items: &[T], workers: usize, worker: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Send + Sync,
{
    if items.is_empty() {
        return Vec::new();
    }
    let workers = workers.max(1).min(items.len());
    let results: Arc<Mutex<Vec<Option<R>>>> = Arc::new(Mutex::new(
        (0..items.len()).map(|_| None).collect::<Vec<_>>(),
    ));
    let index = Arc::new(Mutex::new(0usize));
    let worker = Arc::new(worker);

    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let worker = Arc::clone(&worker);
            let index = Arc::clone(&index);
            let results = Arc::clone(&results);

            let handle = scope.spawn(move || loop {
                let item_index = {
                    let mut cursor = index.lock().unwrap_or_else(|err| err.into_inner());
                    let next = *cursor;
                    *cursor += 1;
                    next
                };

                if item_index >= items.len() {
                    break;
                }

                let result = worker(&items[item_index]);
                if let Ok(mut output) = results.lock() {
                    output[item_index] = Some(result);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }
    });

    let mut ordered = Vec::with_capacity(items.len());
    let mut output = results.lock().unwrap_or_else(|err| err.into_inner());
    for slot in output.iter_mut() {
        if let Some(value) = slot.take() {
            ordered.push(value);
        }
    }
    ordered
}

fn run_legacy_pass_suite() -> PassReport {
    let fixtures = discover_fixtures(Path::new("tests/e2e/pass"));
    assert!(!fixtures.is_empty(), "No pass tests found");

    let mut cases = Vec::with_capacity(fixtures.len());
    for fixture in fixtures {
        let status = match compile_source_with_metadata(&fixture.source) {
            Ok((rust_source, used_modules, required_crates)) => {
                if !rust_source.contains("fn main(") {
                    Err("generated Rust has no main function".to_string())
                } else {
                    let stdlib_modules = used_modules;
                    let required_crates = required_crates;
                    match build_and_run_with_deps(
                        &rust_source,
                        &fixture.name,
                        &stdlib_modules,
                        &required_crates,
                    ) {
                        Ok(stdout) => {
                            if let Some(expected) = &fixture.expected_stdout {
                                let expected = expected.trim_end();
                                let actual = stdout.trim_end();
                                if expected != actual {
                                    Err(format!(
                                        "FAIL [{}]: stdout mismatch\n  expected: {:?}\n  actual:   {:?}",
                                        fixture.name, expected, actual
                                    ))
                                } else {
                                    Ok(())
                                }
                            } else {
                                Ok(())
                            }
                        }
                        Err(err) => Err(format!("FAIL [{}]: {err}", fixture.name)),
                    }
                }
            }
            Err(errors) => Err(format!(
                "FAIL [{}]: sifr compilation failed:\n  {}",
                fixture.name,
                errors.join("\n  ")
            )),
        };

        cases.push(FixtureExecution {
            name: fixture.name,
            status,
        });
    }

    PassReport { cases }
}

fn run_new_pass_suite(config: &RunnerConfig) -> PassReport {
    let fixtures = discover_fixtures(Path::new("tests/e2e/pass"));
    assert!(!fixtures.is_empty(), "No pass tests found");

    let compile_started = Instant::now();
    let compiled_results = compile_suite_parallel(&fixtures, config.sifr_jobs);
    let compile_ms = compile_started.elapsed().as_millis();

    let mut compiled_failures = Vec::new();
    let mut compiled_cases = Vec::new();
    for (fixture, result) in compiled_results {
        match result {
            Ok(compiled) => compiled_cases.push(compiled),
            Err(message) => compiled_failures.push(FixtureExecution {
                name: fixture.name.clone(),
                status: Err(format!("FAIL [{}]: {}", fixture.name, message)),
            }),
        }
    }

    let plan_started = Instant::now();
    let (groups, planning_failures) = plan_batches(compiled_cases);
    let plan_ms = plan_started.elapsed().as_millis();

    let toolchain = toolchain_info();
    let env_signature = cache_env_signature();
    let initial_manifest = if config.cache.enabled {
        if let Err(err) = std::fs::create_dir_all(&config.cache.root) {
            eprintln!("[sifr-e2e-cache] cannot create cache root: {err}");
        }
        read_cache_manifest(&config.cache.root)
    } else {
        CacheManifest {
            schema_version: E2E_CACHE_SCHEMA_VERSION,
            entries: BTreeMap::new(),
        }
    };

    let build_started = Instant::now();
    let (build_outcomes, _updated_manifest) = build_batch_suite(
        groups,
        config,
        &toolchain,
        &env_signature,
        &initial_manifest,
    );
    let build_ms = build_started.elapsed().as_millis();
    let observed_build_ms: u128 = build_outcomes.iter().map(|outcome| outcome.build_ms).sum();
    let cache_hits = build_outcomes
        .iter()
        .filter(|outcome| outcome.cache_hit)
        .count();

    let mut all_cases = Vec::new();
    let run_started = Instant::now();
    let (run_cases, run_outcomes) = run_batch_suite(&build_outcomes, config);
    all_cases.extend(run_cases);
    all_cases.extend(compiled_failures);
    all_cases.extend(planning_failures);
    let run_ms = run_started.elapsed().as_millis();

    let mut build_timing = build_outcomes
        .iter()
        .map(|outcome| {
            (
                outcome.group.id.clone(),
                outcome.build_ms,
                outcome.group.cases.len(),
                outcome.cache_hit,
            )
        })
        .collect::<Vec<_>>();
    build_timing.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let top_build = build_timing.iter().take(3).cloned().collect::<Vec<_>>();

    let mut run_timing = run_outcomes
        .into_iter()
        .map(|outcome| {
            (
                outcome.group_id,
                outcome.elapsed_ms,
                outcome.fixture_count,
                outcome.cache_hit,
            )
        })
        .collect::<Vec<_>>();
    run_timing.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let top_run = run_timing.iter().take(3).cloned().collect::<Vec<_>>();

    let summarize_groups = |groups: &[(String, u128, usize, bool)]| {
        if groups.is_empty() {
            return String::new();
        }

        groups
            .into_iter()
            .map(|(id, ms, count, cache_hit)| {
                format!(
                    "  - {} ({} fixtures, {}ms, cache_hit={})",
                    id, count, ms, cache_hit
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    eprintln!(
        "[sifr-e2e] timing: compile={}ms plan={}ms build={}ms build-sum={}ms run={}ms cache_hits={}",
        compile_ms,
        plan_ms,
        build_ms,
        observed_build_ms,
        run_ms,
        cache_hits
    );
    eprintln!(
        "[sifr-e2e] slowest build groups:\n{}",
        summarize_groups(&top_build)
    );
    eprintln!(
        "[sifr-e2e] slowest run groups:\n{}",
        summarize_groups(&top_run)
    );

    all_cases.sort_by(|left, right| left.name.cmp(&right.name));
    PassReport { cases: all_cases }
}

fn config_cache_root() -> PathBuf {
    Path::new(E2E_CACHE_DIR).to_path_buf()
}

fn compare_pass_reports(legacy: &PassReport, fresh: &PassReport) -> Result<(), String> {
    let mut legacy_by_name = BTreeMap::new();
    let mut new_by_name = BTreeMap::new();
    for exec in &legacy.cases {
        legacy_by_name.insert(exec.name.clone(), exec.status.is_ok());
    }
    for exec in &fresh.cases {
        new_by_name.insert(exec.name.clone(), exec.status.is_ok());
    }

    if legacy_by_name.len() != new_by_name.len() || legacy_by_name.keys().ne(new_by_name.keys()) {
        return Err("runner comparison requires identical fixture set".to_string());
    }

    let mut diffs = Vec::new();
    for (name, legacy_ok) in legacy_by_name {
        if *new_by_name.get(&name).unwrap_or(&false) != legacy_ok {
            diffs.push(format!(
                "fixture {name}: legacy={}, new={}",
                if legacy_ok { "pass" } else { "fail" },
                if new_by_name[&name] { "pass" } else { "fail" }
            ));
        }
    }

    if diffs.is_empty() {
        return Ok(());
    }

    Err(format!(
        "Pass/fail outcome differs for {} fixture(s): {}",
        diffs.len(),
        diffs.join("\n")
    ))
}

fn failure_group(reason: &str) -> &'static str {
    if reason.contains("sifr compilation failed") {
        "compile"
    } else if reason.contains("failed to generate grouped crate source") {
        "planning"
    } else if reason.contains("Rust compilation failed")
        || reason.contains("build log:")
        || reason.contains("missing batch artifact")
    {
        "build"
    } else if reason.contains("stdout mismatch") || reason.contains("binary exited with error") {
        "run"
    } else {
        "other"
    }
}

fn indent_multiline(text: &str, indent: &str) -> String {
    text.lines()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_failures(kind: &str, cases: &[FixtureExecution]) -> String {
    let mut failures = cases
        .iter()
        .filter_map(|case| {
            case.status.as_ref().err().map(|reason| {
                (
                    case.name.clone(),
                    failure_group(reason).to_string(),
                    reason.clone(),
                )
            })
        })
        .collect::<Vec<_>>();

    if failures.is_empty() {
        return String::new();
    }

    failures.sort_by(|left, right| {
        left.1
            .cmp(&right.1)
            .then_with(|| left.0.cmp(&right.0))
            .then_with(|| left.2.cmp(&right.2))
    });

    let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (name, group, reason) in failures {
        grouped.entry(group).or_default().push((name, reason));
    }

    let passed = cases.iter().filter(|case| case.status.is_ok()).count();
    let failed = cases.len().saturating_sub(passed);
    let mut sections = Vec::new();
    for (group, entries) in grouped {
        let mut rows = Vec::new();
        for (name, reason) in entries {
            rows.push(format!("- [{}]\n{}", name, indent_multiline(&reason, "  ")));
        }
        sections.push(format!(
            "[{group}] {} failure(s)\n{}",
            rows.len(),
            rows.join("\n")
        ));
    }

    format!(
        "{kind} E2E pass failures ({passed} passed, {failed} failed)\n\n{}",
        sections.join("\n\n")
    )
}

fn report_signature(kind: &str, report: &PassReport) -> String {
    let summary = format_failures(kind, &report.cases);
    deterministic_hash(&format!(
        "{kind}|{}|{}|{}",
        report.cases.len(),
        report.passed_count(),
        summary
    ))
}

fn assert_report(label: &str, report: &PassReport) {
    let summary = format_failures(label, &report.cases);
    assert!(summary.is_empty(), "{}", summary);
}

#[test]
fn test_e2e_pass() {
    let config = runner_config().expect("runner config");
    match config.mode {
        RunnerMode::Legacy => {
            let report = run_legacy_pass_suite();
            assert_report("legacy", &report);
            eprintln!(
                "[sifr-e2e] report_signature={}",
                report_signature("legacy", &report)
            );
            eprintln!(
                "  {} pass tests completed ({} passed, {} failed)",
                report.cases.len(),
                report.passed_count(),
                report.failed_count()
            );
        }
        RunnerMode::New => {
            let report = run_new_pass_suite(&config);
            assert_report("new", &report);
            eprintln!(
                "[sifr-e2e] report_signature={}",
                report_signature("new", &report)
            );
            eprintln!(
                "  {} pass tests completed ({} passed, {} failed)",
                report.cases.len(),
                report.passed_count(),
                report.failed_count()
            );
        }
        RunnerMode::Compare => {
            let legacy = run_legacy_pass_suite();
            let fresh = run_new_pass_suite(&config);
            compare_pass_reports(&legacy, &fresh).expect("new runner mismatch");
            assert_report("legacy", &legacy);
            assert_report("new", &fresh);
            eprintln!(
                "[sifr-e2e] report_signature_legacy={}",
                report_signature("legacy", &legacy)
            );
            eprintln!(
                "[sifr-e2e] report_signature_new={}",
                report_signature("new", &fresh)
            );
            eprintln!(
                "  compare mode pass tests completed ({} pass in legacy/new)\n    legacy: {} pass, {} fail\n    new: {} pass, {} fail",
                legacy.cases.len(),
                legacy.passed_count(),
                legacy.failed_count(),
                fresh.passed_count(),
                fresh.failed_count()
            );
        }
    }
}

#[test]
fn test_codegen_corpus_subset_parity() {
    let pass_dir = Path::new("tests/e2e/pass");
    let corpus = [
        "if_else",
        "narrowing_elif_equality",
        "loop_else",
        "subscript_aug_assign",
        "subscript_nested_assign",
        "for_tuple_unpack",
        "del_statement",
        "match_guard",
    ];

    let mut test_count = 0usize;
    let mut failures = Vec::new();

    for case in &corpus {
        let path = pass_dir.join(format!("{case}.sifr"));
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(err) => {
                failures.push(format!(
                    "FAIL [{}]: unable to read fixture {}: {}",
                    case,
                    path.display(),
                    err
                ));
                continue;
            }
        };
        let expected_stdout = extract_expect_stdout(&source);

        let (rust_source, stdlib_modules, required_crates) =
            match compile_source_with_metadata(&source) {
                Ok(result) => result,
                Err(errors) => {
                    failures.push(format!(
                        "FAIL [{}]: sifr compilation failed:\n  {}",
                        case,
                        errors.join("\n  ")
                    ));
                    continue;
                }
            };

        if !rust_source.contains("fn main(") {
            failures.push(format!(
                "FAIL [{}]: generated Rust has no main function",
                case
            ));
            continue;
        }

        let stdout = match build_and_run_with_deps(
            &rust_source,
            &format!("{case}_single"),
            &stdlib_modules,
            &required_crates,
        ) {
            Ok(stdout) => stdout,
            Err(err) => {
                failures.push(format!("FAIL [{}]: {}", case, err));
                continue;
            }
        };

        let actual = stdout.trim_end();
        if let Some(expected) = expected_stdout {
            let expected = expected.trim_end();
            if actual != expected {
                failures.push(format!(
                    "FAIL [{}]: stdout mismatch\n  expected: {:?}\n  actual:   {:?}",
                    case, expected, actual
                ));
                continue;
            }
        }

        test_count += 1;
    }

    if !failures.is_empty() {
        panic!(
            "{} corpus parity test(s) failed:\n\n{}\n\n({} passed, {} failed)",
            failures.len(),
            failures.join("\n\n"),
            test_count,
            failures.len()
        );
    }

    assert_eq!(
        test_count,
        corpus.len(),
        "Not all corpus subset cases executed successfully"
    );
    eprintln!("  {} corpus subset parity tests completed", test_count);
}

#[test]
fn test_codegen_structured_lowering_ratio_gate_stmt_expr_corpus() {
    let pass_dir = Path::new("tests/e2e/pass");
    let corpus = ["codegen_structured_ratio_gate"];

    let mut total_stmt_candidate = 0_u64;
    let mut total_stmt_candidate_structured = 0_u64;
    let mut total_expr_candidate = 0_u64;
    let mut total_expr_candidate_structured = 0_u64;
    let mut failures = Vec::new();

    for case in &corpus {
        let path = pass_dir.join(format!("{case}.sifr"));
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(err) => {
                failures.push(format!(
                    "FAIL [{}]: unable to read fixture {}: {}",
                    case,
                    path.display(),
                    err
                ));
                continue;
            }
        };

        let (rust_source, _, _, stats) = match compile_source_with_metadata_and_stats(&source) {
            Ok(result) => result,
            Err(errors) => {
                failures.push(format!(
                    "FAIL [{}]: structured compile failed:\n  {}",
                    case,
                    errors.join("\n  ")
                ));
                continue;
            }
        };

        if !rust_source.contains("fn main(") {
            failures.push(format!(
                "FAIL [{}]: generated Rust has no main function",
                case
            ));
            continue;
        }

        eprintln!(
            "  [{}] stmt={}/{} expr={}/{}",
            case,
            stats.stmt_candidate_structured,
            stats.stmt_candidate_total,
            stats.expr_candidate_structured,
            stats.expr_candidate_total
        );

        total_stmt_candidate += stats.stmt_candidate_total;
        total_stmt_candidate_structured += stats.stmt_candidate_structured;
        total_expr_candidate += stats.expr_candidate_total;
        total_expr_candidate_structured += stats.expr_candidate_structured;
    }

    if !failures.is_empty() {
        panic!(
            "{} structured-ratio corpus setup failure(s):\n\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }

    assert!(
        total_stmt_candidate > 0,
        "structured ratio gate: stmt_candidate_total must be > 0"
    );
    assert!(
        total_expr_candidate > 0,
        "structured ratio gate: expr_candidate_total must be > 0"
    );

    let stmt_ratio = total_stmt_candidate_structured as f64 / total_stmt_candidate as f64;
    let expr_ratio = total_expr_candidate_structured as f64 / total_expr_candidate as f64;

    assert!(
        stmt_ratio >= 0.80,
        "structured ratio gate failed for statements"
    );
    assert!(
        expr_ratio >= 0.80,
        "structured ratio gate failed for expressions"
    );

    eprintln!(
        "  structured ratio gate passed: stmt={:.3} ({}/{}), expr={:.3} ({}/{})",
        stmt_ratio,
        total_stmt_candidate_structured,
        total_stmt_candidate,
        expr_ratio,
        total_expr_candidate_structured,
        total_expr_candidate
    );
}

#[test]
fn test_e2e_fail() {
    let fail_dir = Path::new("tests/e2e/fail");
    if !fail_dir.exists() {
        return;
    }

    let mut failures = 0usize;
    for path in read_dir_file_paths_sorted(fail_dir) {
        let source = std::fs::read_to_string(&path).unwrap();
        let expected = extract_expect_errors(&source);

        match compile_source(&source) {
            Ok(rust_source) => {
                panic!(
                    "FAIL test {} should have failed but compiled successfully:\n{}",
                    path.display(),
                    rust_source
                );
            }
            Err(errors) => {
                let all = errors.join("\n");
                for expected in &expected {
                    assert!(
                        all.contains(expected),
                        "FAIL {} expected error containing '{}' but got:\n{}",
                        path.display(),
                        expected,
                        all
                    );
                }
                failures += 1;
            }
        }
    }

    assert!(failures > 0, "No fail tests found");
    eprintln!("  {} fail tests completed", failures);
}

#[test]
fn test_e2e_runtime_fail() {
    let runtime_fail_dir = Path::new("tests/e2e/runtime_fail");
    if !runtime_fail_dir.exists() {
        return;
    }

    let mut failures = Vec::new();
    let mut total = 0usize;

    for path in read_dir_file_paths_sorted(runtime_fail_dir) {
        let test_name = path_to_name(&path);
        let source = std::fs::read_to_string(&path).unwrap();
        let expected_stderr = extract_expect_stderr(&source);

        let (rust_source, used_stdlib_modules, required_crates) =
            match compile_source_with_metadata(&source) {
                Ok(result) => result,
                Err(errors) => {
                    failures.push(format!(
                    "FAIL [{}]: sifr compilation failed (runtime-fail tests must compile):\n  {}",
                    test_name,
                    errors.join("\n  ")
                ));
                    continue;
                }
            };

        match build_and_run_capture_with_deps(
            &rust_source,
            &test_name,
            &used_stdlib_modules,
            &required_crates,
        ) {
            Ok((_stdout, stderr, success)) => {
                if success {
                    failures.push(format!(
                        "FAIL [{}]: expected runtime failure but binary exited successfully",
                        test_name
                    ));
                    continue;
                }

                for expected in &expected_stderr {
                    if !stderr.contains(expected) {
                        failures.push(format!(
                            "FAIL [{}]: expected stderr containing {:?} but got:\n{}",
                            test_name, expected, stderr
                        ));
                    }
                }
            }
            Err(err) => {
                failures.push(format!("FAIL [{}]: {}", test_name, err));
                continue;
            }
        }

        total += 1;
    }

    if !failures.is_empty() {
        panic!(
            "{} E2E runtime-fail test(s) failed:\n\n{}\n\n({} passed, {} failed)",
            failures.len(),
            failures.join("\n\n"),
            total,
            failures.len()
        );
    }

    assert!(total > 0, "No runtime_fail tests found");
    eprintln!("  {} runtime_fail tests completed", total);
}

fn read_dir_file_paths_sorted(directory: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(directory)
        .into_iter()
        .flat_map(|dir| dir.filter_map(Result::ok))
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "sifr"))
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    paths
}

#[test]
fn test_runner_mode_resolution() {
    assert!(matches!(
        parse_runner_mode_from_env(Some("legacy"), None, None).unwrap(),
        RunnerMode::Legacy
    ));
    assert!(matches!(
        parse_runner_mode_from_env(Some("new"), None, None).unwrap(),
        RunnerMode::New
    ));
    assert!(matches!(
        parse_runner_mode_from_env(Some("compare"), None, None).unwrap(),
        RunnerMode::Compare
    ));
    assert!(matches!(
        parse_runner_mode_from_env(None, Some("1"), Some("0")).unwrap(),
        RunnerMode::New
    ));
    assert!(matches!(
        parse_runner_mode_from_env(None, Some("0"), Some("1")).unwrap(),
        RunnerMode::Legacy
    ));
    assert!(parse_runner_mode_from_env(Some("legacy"), Some("1"), Some("1")).is_err());
}

#[test]
fn test_cache_root_from_env_resolution() {
    assert_eq!(
        cache_root_from_env(None),
        Path::new(E2E_CACHE_DIR).to_path_buf()
    );
    assert_eq!(
        cache_root_from_env(Some("")),
        Path::new(E2E_CACHE_DIR).to_path_buf()
    );
    assert_eq!(
        cache_root_from_env(Some("   ")),
        Path::new(E2E_CACHE_DIR).to_path_buf()
    );
    assert_eq!(
        cache_root_from_env(Some("target/custom_cache_root")),
        PathBuf::from("target/custom_cache_root")
    );
}

#[test]
fn test_expectation_parsing_contract() {
    let source = [
        "# expect-stdout: a",
        "# expect-stdout: b",
        "# expect-stderr: err-1",
        "# expect-stderr: err-2",
        "# expect-error: issue-1",
        "# expect-error: issue-2",
    ]
    .join("\n");

    assert_eq!(extract_expect_stdout(&source), Some("a\nb".to_string()));
    assert_eq!(
        extract_expect_stderr(&source),
        vec!["err-1".to_string(), "err-2".to_string()]
    );
    assert_eq!(
        extract_expect_errors(&source),
        vec!["issue-1".to_string(), "issue-2".to_string()]
    );
}

#[test]
fn test_failure_summary_is_grouped_and_order_stable() {
    let cases = vec![
        FixtureExecution {
            name: "z_run".to_string(),
            status: Err("FAIL [z_run]: stdout mismatch\n  expected: \"a\"\n  actual:   \"b\"".to_string()),
        },
        FixtureExecution {
            name: "a_compile".to_string(),
            status: Err("FAIL [a_compile]: sifr compilation failed:\n  unknown symbol".to_string()),
        },
        FixtureExecution {
            name: "b_build".to_string(),
            status: Err("FAIL [b_build]: Rust compilation failed. Check build log: /tmp/log".to_string()),
        },
        FixtureExecution {
            name: "ok_case".to_string(),
            status: Ok(()),
        },
    ];

    let mut reversed = cases.clone();
    reversed.reverse();

    let first = format_failures("new", &cases);
    let second = format_failures("new", &reversed);
    assert_eq!(first, second);
    assert!(first.contains("[compile] 1 failure(s)"));
    assert!(first.contains("[build] 1 failure(s)"));
    assert!(first.contains("[run] 1 failure(s)"));
}

#[test]
fn test_report_signature_is_order_invariant() {
    let cases = vec![
        FixtureExecution {
            name: "case_b".to_string(),
            status: Err("FAIL [case_b]: binary exited with error:\nboom".to_string()),
        },
        FixtureExecution {
            name: "case_a".to_string(),
            status: Err("FAIL [case_a]: sifr compilation failed:\n  error".to_string()),
        },
        FixtureExecution {
            name: "case_ok".to_string(),
            status: Ok(()),
        },
    ];
    let report = PassReport {
        cases: cases.clone(),
    };

    let mut shuffled = cases;
    shuffled.swap(0, 1);
    let report_shuffled = PassReport { cases: shuffled };

    assert_eq!(
        report_signature("new", &report),
        report_signature("new", &report_shuffled)
    );
}

#[test]
fn test_report_signature_changes_on_failure_delta() {
    let base = PassReport {
        cases: vec![
            FixtureExecution {
                name: "case_a".to_string(),
                status: Err("FAIL [case_a]: sifr compilation failed:\n  err-a".to_string()),
            },
            FixtureExecution {
                name: "case_ok".to_string(),
                status: Ok(()),
            },
        ],
    };
    let changed = PassReport {
        cases: vec![
            FixtureExecution {
                name: "case_a".to_string(),
                status: Err("FAIL [case_a]: sifr compilation failed:\n  err-b".to_string()),
            },
            FixtureExecution {
                name: "case_ok".to_string(),
                status: Ok(()),
            },
        ],
    };

    assert_ne!(
        report_signature("new", &base),
        report_signature("new", &changed)
    );
}

#[test]
fn test_fixture_discovery_is_deterministic() {
    let root = env::temp_dir().join(format!(
        "sifr-e2e-discovery-{}",
        deterministic_hash(
            &SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .to_string()
        )
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("zeta.sifr"), "# expect-stdout: ok\n").unwrap();
    std::fs::write(root.join("alpha.sifr"), "# expect-stdout: ok\n").unwrap();
    std::fs::write(root.join("beta.sifr"), "# expect-stdout: ok\n").unwrap();

    let names: Vec<_> = discover_fixtures(&root)
        .into_iter()
        .map(|fixture| fixture.name)
        .collect();
    let _ = std::fs::remove_dir_all(root);
    assert_eq!(
        names,
        vec!["alpha".to_string(), "beta".to_string(), "zeta".to_string()]
    );
}

#[test]
fn test_dependency_fingerprint_and_cache_key_determinism() {
    let fixture = DependencyFingerprint {
        stdlib_modules: normalize_dependency_set(
            vec!["a", "b", "a"].into_iter().map(str::to_string),
        ),
        required_crates: normalize_dependency_set(
            vec!["x", "y", "x"].into_iter().map(str::to_string),
        ),
    };
    let same = DependencyFingerprint {
        stdlib_modules: normalize_dependency_set(vec!["b", "a"].into_iter().map(str::to_string)),
        required_crates: normalize_dependency_set(vec!["y", "x"].into_iter().map(str::to_string)),
    };
    assert_eq!(fixture.signature(), same.signature());
    assert_eq!(fixture.hash(), same.hash());

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let case = FixtureCase {
        name: "fixture-a".to_string(),
        path: PathBuf::from("tests/e2e/pass/fixture-a.sifr"),
        source: "print('x')".to_string(),
        source_hash: deterministic_hash(&format!("hash-{}", now.as_nanos())),
        expected_stdout: None,
        _expected_stderr: Vec::new(),
        _expected_errors: Vec::new(),
    };
    let compiled = CompiledCase {
        fixture: case.clone(),
        rust_source: "fn main() {}".to_string(),
        stdlib_modules: BTreeSet::new(),
        required_crates: BTreeSet::new(),
        _compile_duration_ms: 0,
    };
    let group = build_group_sources(vec![compiled]).unwrap();
    let toolchain = toolchain_info();
    let env_signature = cache_env_signature();
    let key_a = cache_key_for_group(&group, &toolchain, &env_signature);
    let key_b = cache_key_for_group(&group, &toolchain, "different");
    assert_ne!(key_a, key_b);
    assert!(group.id.len() > 0);
}

fn sample_cache_entry(
    group: &BatchGroup,
    toolchain: &ToolchainInfo,
    env_signature: &str,
) -> CacheEntry {
    let key = cache_key_for_group(group, toolchain, env_signature);
    let artifact_path = env::temp_dir().join("sifr-e2e-cache-key-sample").join(&key);

    std::fs::create_dir_all(&artifact_path).expect("cache sample dir");
    let binary_path = artifact_path.join(format!("{}.bin", group.package_name));
    std::fs::write(&binary_path, b"cache-test").expect("cache sample binary");

    CacheEntry {
        schema_version: E2E_CACHE_SCHEMA_VERSION,
        cache_key: key.clone(),
        group_id: group.id.clone(),
        group_fingerprint: group.fingerprint.signature(),
        group_rust_hash: group.generated_rust_hash.clone(),
        fixture_sources: group
            .cases
            .iter()
            .map(|case| FixtureSourceHash {
                fixture: case.fixture.name.clone(),
                hash: case.fixture.source_hash.clone(),
            })
            .collect(),
        compiler_signature: toolchain.signature(),
        rustc_v: toolchain.rustc_v.clone(),
        rustc_vv: toolchain.rustc_vv.clone(),
        cargo_v: toolchain.cargo_v.clone(),
        target: toolchain.target.clone(),
        os: toolchain.os.clone(),
        arch: toolchain.arch.clone(),
        env_signature: env_signature.to_string(),
        artifact_path: binary_path.to_string_lossy().to_string(),
        build_log_path: None,
        built_at_unix_secs: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
    }
}

#[test]
fn test_cache_entry_invalidation_rules() {
    let case = FixtureCase {
        name: "fixture-cache-a".to_string(),
        path: PathBuf::from("tests/e2e/pass/fixture-cache-a.sifr"),
        source: "print('x')".to_string(),
        source_hash: deterministic_hash("cache-fixture-a"),
        expected_stdout: None,
        _expected_stderr: Vec::new(),
        _expected_errors: Vec::new(),
    };
    let compiled = CompiledCase {
        fixture: case.clone(),
        rust_source: "fn main() {}".to_string(),
        stdlib_modules: BTreeSet::new(),
        required_crates: BTreeSet::new(),
        _compile_duration_ms: 0,
    };
    let group = build_group_sources(vec![compiled]).expect("cache sample group");
    let toolchain = toolchain_info();
    let env_signature = cache_env_signature();

    let valid_entry = sample_cache_entry(&group, &toolchain, &env_signature);
    let cache_key = cache_key_for_group(&group, &toolchain, &env_signature);
    assert!(cache_entry_valid(
        &valid_entry,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));

    let mut invalid_schema = valid_entry.clone();
    invalid_schema.schema_version = E2E_CACHE_SCHEMA_VERSION + 1;
    assert!(!cache_entry_valid(
        &invalid_schema,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));

    let mut invalid_fixture = valid_entry.clone();
    invalid_fixture
        .fixture_sources
        .iter_mut()
        .for_each(|item| item.hash.push_str("-changed"));
    assert!(!cache_entry_valid(
        &invalid_fixture,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));

    let mut invalid_env = valid_entry.clone();
    invalid_env.env_signature = "different-env".to_string();
    assert!(!cache_entry_valid(
        &invalid_env,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));

    let invalid_path = env::temp_dir()
        .join("sifr-e2e-missing-artifact")
        .join("no-file");
    let mut invalid_artifact = valid_entry.clone();
    invalid_artifact.artifact_path = invalid_path.to_string_lossy().to_string();
    assert!(!cache_entry_valid(
        &invalid_artifact,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));
}
