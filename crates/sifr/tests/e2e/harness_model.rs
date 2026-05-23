use serde::{Deserialize, Serialize};
use sifr_diagnostics::codes::{active_registry_entries, registry_entry, DiagnosticState};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::env;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const E2E_CACHE_DIR: &str = "target/sifr_e2e_cache";
const E2E_CACHE_MANIFEST: &str = "manifest.json";
const E2E_CACHE_SCHEMA_VERSION: u32 = 1;
const E2E_CACHE_TTL_SECS: u64 = 2 * 60 * 60;
const E2E_CACHE_ENV_ALLOWLIST: [&str; 9] = [
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTC_WRAPPER",
    "SIFR_E2E_PROFILE",
    "SIFR_E2E_RUNNER_MODE",
    "SIFR_E2E_NEW_RUNNER",
    "SIFR_E2E_LEGACY_RUNNER",
    "SIFR_E2E_CACHE_DIR",
    "SIFR_E2E_FIXTURE_MANIFEST",
];
const E2E_FIXTURE_MANIFEST_ENV: &str = "SIFR_E2E_FIXTURE_MANIFEST";

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CompileFailureExpectation {
    code: String,
    column: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LocatedCompileFailureExpectation {
    line_number: usize,
    expectation: CompileFailureExpectation,
}

#[derive(Clone, Debug)]
struct CompiledFailure {
    code: String,
    message: String,
    column: Option<u32>,
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

#[derive(Deserialize, Debug)]
struct FixtureSelectionManifest {
    fixture_names: Vec<String>,
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

fn cache_groups_root(root: &Path) -> PathBuf {
    cache_dir(root).join("groups")
}

fn cache_group_path(root: &Path, group_id: &str) -> PathBuf {
    cache_groups_root(root).join(group_id)
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
    if rust_source.contains("sifr_runtime::") || rust_source.contains("use sifr_runtime") {
        crates.insert("sifr_runtime".to_string());
    }
    if rust_source.contains("regex::") {
        crates.insert("regex".to_string());
    }
    if rust_source.contains("rand::rng") {
        crates.insert("rand".to_string());
    }
    if rust_source.contains("rand_distr::") {
        crates.insert("rand_distr".to_string());
    }
    if rust_source.contains("chrono::") {
        crates.insert("chrono".to_string());
    }
    if rust_source.contains("md5::") {
        crates.insert("md5".to_string());
    }
    if rust_source.contains("uuid::") {
        crates.insert("uuid".to_string());
    }
    if rust_source.contains("toml::") {
        crates.insert("toml".to_string());
    }
    if rust_source.contains("flate2::") {
        crates.insert("flate2".to_string());
    }
    if rust_source.contains("zip::") {
        crates.insert("zip".to_string());
    }
    if rust_source.contains("base64::") {
        crates.insert("base64".to_string());
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
    if rust_source.contains("rust_decimal::") || rust_source.contains("use rust_decimal") {
        crates.insert("rust_decimal".to_string());
    }
    if rust_source.contains("bigdecimal::") || rust_source.contains("use bigdecimal") {
        crates.insert("bigdecimal".to_string());
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

fn compile_source(source: &str) -> Result<String, Vec<CompiledFailure>> {
    match sifr_driver::compile(source) {
        sifr_driver::CompileResult::Success { rust_source } => Ok(rust_source),
        sifr_driver::CompileResult::Errors { errors } => {
            let mut failures = Vec::new();
            for diagnostic in errors {
                failures.push(compiled_failure_from_rendered(diagnostic));
            }
            Err(failures)
        }
    }
}

fn compiled_failure_from_rendered(
    diagnostic: sifr_diagnostics::RenderedDiagnostic,
) -> CompiledFailure {
    CompiledFailure {
        code: diagnostic.code,
        message: diagnostic.message,
        column: diagnostic
            .spans
            .iter()
            .find(|span| span.is_primary)
            .and_then(|span| span.column),
    }
}

fn parse_expected_error(raw: &str) -> Result<CompileFailureExpectation, String> {
    parse_expected_error_parts(None, raw)
}

fn parse_expected_error_parts(
    column: Option<u32>,
    raw_code: &str,
) -> Result<CompileFailureExpectation, String> {
    let code = raw_code.trim();
    validate_expected_error_code(code)?;
    Ok(CompileFailureExpectation {
        code: code.to_string(),
        column,
    })
}

fn parse_expect_error_line(line: &str) -> Option<Result<CompileFailureExpectation, String>> {
    if let Some(raw_code) = line.strip_prefix("# expect-error:") {
        return Some(parse_expected_error_parts(None, raw_code));
    }

    let rest = line.strip_prefix("# expect-error[")?;
    let (qualifier, raw_code) = match rest.split_once("]:") {
        Some(parts) => parts,
        None => {
            return Some(Err(
                "expected expect-error qualifier syntax '# expect-error[col=<column>]: <code>'"
                    .to_string(),
            ));
        }
    };
    let Some(raw_column) = qualifier.strip_prefix("col=") else {
        return Some(Err(format!(
            "unknown expect-error qualifier '{qualifier}'; only col=<column> is supported"
        )));
    };
    let column = match raw_column.parse::<u32>() {
        Ok(value) if value > 0 => value,
        _ => {
            return Some(Err(format!(
                "invalid expect-error column '{raw_column}'; expected a positive 1-based column"
            )));
        }
    };
    Some(parse_expected_error_parts(Some(column), raw_code))
}

fn expectation_locations_overlap(
    left: &LocatedCompileFailureExpectation,
    right: &LocatedCompileFailureExpectation,
) -> bool {
    // Unqualified markers assert code existence only; they do not claim every column.
    // Contradictions require both markers to name the same explicit assertion point.
    match (left.expectation.column, right.expectation.column) {
        (Some(left_column), Some(right_column)) => left_column == right_column,
        _ => false,
    }
}

fn expectation_location_label(expectation: &LocatedCompileFailureExpectation) -> String {
    match expectation.expectation.column {
        Some(column) => format!("column {column}"),
        None => "any column".to_string(),
    }
}

fn validate_expectation_contradictions(
    expectations: &[LocatedCompileFailureExpectation],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for (left_index, left) in expectations.iter().enumerate() {
        for right in &expectations[left_index + 1..] {
            if expectation_locations_overlap(left, right)
                && left.expectation.code != right.expectation.code
            {
                let left_location = expectation_location_label(left);
                let right_location = expectation_location_label(right);
                let location_suffix = if left_location == right_location {
                    left_location
                } else {
                    format!("{left_location} overlapping {right_location}")
                };
                errors.push(format!(
                    "contradictory expect-error markers: {} at marker line {} conflicts with {} at marker line {} for {}",
                    left.expectation.code,
                    left.line_number,
                    right.expectation.code,
                    right.line_number,
                    location_suffix,
                ));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn parse_compile_failure_expectations(
    source: &str,
    fixture_path: &Path,
) -> Result<Vec<CompileFailureExpectation>, Vec<String>> {
    let mut errors = Vec::new();
    let mut located = Vec::new();
    for (line_index, line) in source.lines().enumerate() {
        if let Some(result) = parse_expect_error_line(line) {
            match result {
                Ok(expectation) => {
                    located.push(LocatedCompileFailureExpectation {
                        line_number: line_index + 1,
                        expectation,
                    });
                }
                Err(error) => {
                    errors.push(format!(
                        "{}:{} invalid expect-error marker: {}",
                        fixture_path.display(),
                        line_index + 1,
                        error
                    ));
                }
            }
        }
    }

    if let Err(contradiction_errors) = validate_expectation_contradictions(&located) {
        errors.extend(
            contradiction_errors
                .into_iter()
                .map(|error| format!("{}: {error}", fixture_path.display())),
        );
    }

    if errors.is_empty() {
        Ok(located
            .into_iter()
            .map(|located_expectation| located_expectation.expectation)
            .collect())
    } else {
        Err(errors)
    }
}

fn format_expectation_contract_errors(errors: &[String]) -> String {
    errors
        .iter()
        .map(|error| format!("FAIL {error}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_compile_failure_expectations(
    source: &str,
    fixture_path: &Path,
) -> Vec<CompileFailureExpectation> {
    parse_compile_failure_expectations(source, fixture_path).unwrap_or_else(|errors| {
        panic!("{}", format_expectation_contract_errors(&errors));
    })
}

fn validate_expected_error_code(code: &str) -> Result<(), String> {
    if code.is_empty() {
        return Err("expected a diagnostic code after expect-error".to_string());
    }

    let bare_legacy_code = code
        .strip_prefix('E')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()));
    let bracketed_legacy_code = code
        .strip_prefix("[E")
        .and_then(|digits| digits.strip_suffix(']'))
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()));
    if bare_legacy_code || bracketed_legacy_code {
        return Err(format!(
            "legacy pseudo-code '{code}' is not accepted; use canonical SIFR-<FAMILY>-dddd"
        ));
    }

    if code
        .chars()
        .any(|character| character == ':' || character.is_whitespace())
    {
        return Err(format!(
            "message substrings are not accepted in expect-error markers: '{code}'"
        ));
    }

    if !is_diagnostic_code(code) {
        return Err(format!(
            "expected canonical SIFR-<FAMILY>-dddd code, got '{code}'"
        ));
    }

    match registry_entry(code) {
        Some(entry) if entry.state == DiagnosticState::Active => Ok(()),
        Some(entry) => Err(format!(
            "diagnostic code '{}' is {}, but e2e expectations require an active code",
            entry.id,
            entry.state.as_str()
        )),
        None => {
            let hint = closest_active_diagnostic_code(code)
                .map(|candidate| format!("; did you mean {candidate}?"))
                .unwrap_or_default();
            Err(format!("unknown diagnostic code '{code}'{hint}"))
        }
    }
}

fn is_diagnostic_code(raw: &str) -> bool {
    if !raw.starts_with("SIFR-") || raw.len() <= 8 {
        return false;
    }

    let suffix = &raw[5..];
    if let Some((prefix, code)) = suffix.split_once('-') {
        prefix
            .chars()
            .all(|character| character.is_ascii_uppercase() || character.is_ascii_digit())
            && code.len() == 4
            && code.chars().all(|character| character.is_ascii_digit())
    } else {
        false
    }
}

fn closest_active_diagnostic_code(raw: &str) -> Option<&'static str> {
    active_registry_entries()
        .map(|entry| (entry.id, edit_distance(raw, entry.id)))
        .min_by_key(|(id, distance)| (*distance, *id))
        .map(|(id, _)| id)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut current = vec![0; right_chars.len() + 1];

    for (left_index, left_char) in left.chars().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_char) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_char != *right_char);
            current[right_index + 1] = (previous[right_index + 1] + 1)
                .min(current[right_index] + 1)
                .min(previous[right_index] + substitution_cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right_chars.len()]
}

fn compile_failures_to_messages(failures: &[CompiledFailure]) -> Vec<String> {
    failures
        .iter()
        .map(|failure| match failure.column {
            Some(column) => format!("{}@col{}: {}", failure.code, column, failure.message),
            None => format!("{}: {}", failure.code, failure.message),
        })
        .collect()
}

