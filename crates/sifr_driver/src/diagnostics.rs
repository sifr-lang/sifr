use sifr_diagnostics::codes::registry_entry;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
pub(crate) use sifr_diagnostics::{DiagnosticSpan, RenderedDiagnostic, Severity};
use sifr_frontend::SourceOrigin;
use sifr_package::{PackageDiagnostic, PackageDiagnosticOrigin};
use sifr_stdlib_model::StdlibFeature;
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Debug)]
pub enum CompileResult {
    Success { rust_source: String },
    Errors { errors: Vec<RenderedDiagnostic> },
}

pub enum CompileResultFull {
    Success {
        rust_source: String,
        generated_source_map: Vec<GeneratedSourceMapFile>,
        used_stdlib_modules: HashSet<String>,
        required_features: HashSet<StdlibFeature>,
        lowering_stats: sifr_codegen::LoweringStats,
    },
    Errors {
        errors: Vec<RenderedDiagnostic>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedSourceMapFile {
    pub path: String,
    pub origin: SourceOrigin,
    pub source: String,
}

/// Creates a rendered error diagnostic with canonical diagnostic identity.
#[must_use]
pub(crate) fn diagnostic_with_code(
    message: impl Into<String>,
    code: DiagnosticCode,
) -> RenderedDiagnostic {
    let message = message.into();
    let mut args = BTreeMap::new();
    args.insert(
        "message".to_string(),
        DiagnosticArg::String(message.clone()),
    );
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: "{message}".to_string(),
        args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

/// Converts package diagnostics into the canonical rendered diagnostic envelope.
#[must_use]
pub fn render_package_diagnostic(diagnostic: PackageDiagnostic) -> RenderedDiagnostic {
    let PackageDiagnostic {
        code,
        message,
        origin,
        help,
    } = diagnostic;
    let mut rendered = diagnostic_with_code(message, code);
    rendered.help = help;
    add_package_origin_args(&mut rendered, &origin);
    rendered
}

fn add_package_origin_args(rendered: &mut RenderedDiagnostic, origin: &PackageDiagnosticOrigin) {
    match origin {
        PackageDiagnosticOrigin::CargoMetadata { cargo_package_id } => {
            insert_arg(rendered, "origin_kind", "cargo_metadata");
            if let Some(cargo_package_id) = cargo_package_id {
                insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
            }
        }
        PackageDiagnosticOrigin::CargoManifest {
            cargo_package_id,
            path,
            key,
        } => {
            insert_arg(rendered, "origin_kind", "cargo_manifest");
            insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
            insert_arg(rendered, "manifest_path", path.display().to_string());
            if let Some(key) = key {
                insert_arg(rendered, "manifest_key", key);
            }
        }
        PackageDiagnosticOrigin::SifrManifest {
            cargo_package_id,
            path,
            key,
        } => {
            insert_arg(rendered, "origin_kind", "sifr_manifest");
            insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
            insert_arg(rendered, "manifest_path", path.display().to_string());
            if let Some(key) = key {
                insert_arg(rendered, "manifest_key", key);
            }
        }
        PackageDiagnosticOrigin::RustMarker {
            cargo_package_id,
            path,
        } => {
            insert_arg(rendered, "origin_kind", "rust_marker");
            insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
            insert_arg(rendered, "marker_path", path.display().to_string());
        }
        PackageDiagnosticOrigin::PackageGraph { cargo_package_id } => {
            insert_arg(rendered, "origin_kind", "package_graph");
            insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
        }
        PackageDiagnosticOrigin::CargoCommand { action } => {
            insert_arg(rendered, "origin_kind", "cargo_command");
            insert_arg(rendered, "cargo_action", action);
        }
    }
}

fn insert_arg(
    rendered: &mut RenderedDiagnostic,
    name: impl Into<String>,
    value: impl Into<String>,
) {
    rendered
        .args
        .insert(name.into(), DiagnosticArg::String(value.into()));
}

const MAX_TOP_LEVEL_DIAGNOSTICS: usize = 50;
const MAX_SIMILAR_DIAGNOSTICS_PER_GROUP: usize = 5;
const SIMILAR_DIAGNOSTIC_CAP_KIND: &str = "similar-diagnostic group";
const TOP_LEVEL_CAP_KIND: &str = "top-level diagnostic stream";

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Note => 2,
    }
}

pub fn apply_diagnostic_recovery_limits(
    diagnostics: &[RenderedDiagnostic],
) -> Vec<RenderedDiagnostic> {
    let deduped = deduplicate_recovery_diagnostics(diagnostics);
    let mut grouped: BTreeMap<SimilarDiagnosticGroupKey, Vec<RenderedDiagnostic>> = BTreeMap::new();
    for diagnostic in deduped {
        let key = SimilarDiagnosticGroupKey::from_diagnostic(&diagnostic);
        grouped.entry(key).or_default().push(diagnostic.clone());
    }

    let mut bounded = Vec::new();
    for (_key, group) in grouped {
        let retained = group.len().min(MAX_SIMILAR_DIAGNOSTICS_PER_GROUP);
        for diagnostic in group.iter().take(retained) {
            bounded.push(diagnostic.clone());
        }
        if group.len() > MAX_SIMILAR_DIAGNOSTICS_PER_GROUP {
            let omitted_diagnostics = &group[MAX_SIMILAR_DIAGNOSTICS_PER_GROUP..];
            bounded.push(recovery_omission_summary(
                omitted_diagnostics.len(),
                SIMILAR_DIAGNOSTIC_CAP_KIND,
                &omitted_kind(omitted_diagnostics),
            ));
        }
    }

    if bounded.len() > MAX_TOP_LEVEL_DIAGNOSTICS {
        let omitted_diagnostics = bounded[MAX_TOP_LEVEL_DIAGNOSTICS - 1..].to_vec();
        bounded.truncate(MAX_TOP_LEVEL_DIAGNOSTICS - 1);
        bounded.push(recovery_omission_summary(
            omitted_diagnostics.len(),
            TOP_LEVEL_CAP_KIND,
            &omitted_kind(&omitted_diagnostics),
        ));
    }
    bounded
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RecoveryDedupeKey {
    code: String,
    message_template: String,
    dedupe_args: Vec<(String, String)>,
    primary_span: Option<PrimarySpanKey>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SimilarDiagnosticGroupKey {
    severity_rank: u8,
    code: String,
    message_template: String,
    dedupe_args: Vec<(String, String)>,
    primary_file: Option<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PrimarySpanKey {
    file: Option<String>,
    byte_start: u32,
    byte_end: u32,
}

impl RecoveryDedupeKey {
    fn from_diagnostic(diagnostic: &RenderedDiagnostic) -> Self {
        Self {
            code: diagnostic.code.clone(),
            message_template: diagnostic.message_template.clone(),
            dedupe_args: recovery_dedupe_args(diagnostic),
            primary_span: primary_span(diagnostic).map(PrimarySpanKey::from_span),
        }
    }
}

impl SimilarDiagnosticGroupKey {
    fn from_diagnostic(diagnostic: &RenderedDiagnostic) -> Self {
        Self {
            severity_rank: severity_rank(diagnostic.severity),
            code: diagnostic.code.clone(),
            message_template: diagnostic.message_template.clone(),
            dedupe_args: recovery_dedupe_args(diagnostic),
            primary_file: primary_span(diagnostic).and_then(|span| span.file.clone()),
        }
    }
}

impl PrimarySpanKey {
    fn from_span(span: &DiagnosticSpan) -> Self {
        Self {
            file: span.file.clone(),
            byte_start: span.byte_start,
            byte_end: span.byte_end,
        }
    }
}

fn deduplicate_recovery_diagnostics(diagnostics: &[RenderedDiagnostic]) -> Vec<RenderedDiagnostic> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for diagnostic in diagnostics {
        if seen.insert(RecoveryDedupeKey::from_diagnostic(diagnostic)) {
            deduped.push(diagnostic.clone());
        }
    }
    deduped
}

fn recovery_dedupe_args(diagnostic: &RenderedDiagnostic) -> Vec<(String, String)> {
    if let Some(entry) = registry_entry(&diagnostic.code) {
        let mut key_args = Vec::new();
        for arg in entry.dedupe_args {
            if let Some(value) = diagnostic.args.get(*arg) {
                key_args.push(((*arg).to_string(), diagnostic_arg_key(value)));
            }
        }
        return key_args;
    }

    diagnostic
        .args
        .iter()
        .map(|(key, value)| (key.clone(), diagnostic_arg_key(value)))
        .collect()
}

fn diagnostic_arg_key(arg: &DiagnosticArg) -> String {
    String::from_utf8_lossy(&arg.canonical_json_bytes()).into_owned()
}

fn omitted_kind(omitted_diagnostics: &[RenderedDiagnostic]) -> String {
    let reveal_type_count = omitted_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::TYPE_REVEAL_TYPE.code())
        .count();
    match (reveal_type_count, omitted_diagnostics.len()) {
        (0, _) => "diagnostics".to_string(),
        (reveal_type_count, total) if reveal_type_count == total => {
            "reveal_type results".to_string()
        }
        (reveal_type_count, _) => {
            format!("diagnostics (including {reveal_type_count} reveal_type results)")
        }
    }
}

fn recovery_omission_summary(
    omitted_count: usize,
    cap_kind: &'static str,
    omitted_kind: &str,
) -> RenderedDiagnostic {
    let code = DiagnosticCode::INTERNAL_RECOVERY_OMISSION_SUMMARY;
    let mut args = BTreeMap::new();
    args.insert(
        "omitted_count".to_string(),
        DiagnosticArg::Unsigned(omitted_count as u64),
    );
    args.insert(
        "cap_kind".to_string(),
        DiagnosticArg::String(cap_kind.to_string()),
    );
    args.insert(
        "omitted_kind".to_string(),
        DiagnosticArg::String(omitted_kind.to_string()),
    );
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message: format!(
            "{omitted_count} additional {omitted_kind} omitted by recovery cap ({cap_kind})"
        ),
        message_template:
            "{omitted_count} additional {omitted_kind} omitted by recovery cap ({cap_kind})"
                .to_string(),
        args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

fn primary_span(diagnostic: &RenderedDiagnostic) -> Option<&DiagnosticSpan> {
    diagnostic.spans.iter().find(|span| span.is_primary)
}

#[cfg(test)]
pub(crate) fn diagnostic_legacy_display(diagnostic: &RenderedDiagnostic) -> String {
    format!(
        "{}: {}",
        diagnostic_label_for_code_str(&diagnostic.code),
        diagnostic.message
    )
}

#[must_use]
pub fn diagnostic_label_for_code(code: DiagnosticCode) -> &'static str {
    diagnostic_label_for_code_str(code.code())
}

#[must_use]
pub fn diagnostic_label_for_code_str(code: &str) -> &'static str {
    if code == DiagnosticCode::INTERNAL_COMPILER_PANIC.code() {
        "internal compiler error"
    } else if code == DiagnosticCode::INTERNAL_RECOVERY_OMISSION_SUMMARY.code() {
        "note"
    } else if code == DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE.code()
        || code == DiagnosticCode::STDLIB_CACHE_FAILURE.code()
    {
        "build error"
    } else if code.starts_with("SIFR-PARSE-") {
        "parse error"
    } else if code.starts_with("SIFR-CODEGEN-") {
        "codegen error"
    } else if code.starts_with("SIFR-FMT-") {
        "format error"
    } else if code.starts_with("SIFR-LINT-") {
        "lint warning"
    } else if code.starts_with("SIFR-BUILD-") || code.starts_with("SIFR-WORKSPACE-") {
        "build error"
    } else {
        "type error"
    }
}

pub(crate) fn write_stderr_line(message: &str) {
    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(stderr, "{message}");
}

pub(crate) fn write_stderr(message: &str) {
    let mut stderr = std::io::stderr().lock();
    let _ = write!(stderr, "{message}");
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(msg) = payload.downcast_ref::<&str>() {
        return (*msg).to_string();
    }
    if let Some(msg) = payload.downcast_ref::<String>() {
        return msg.clone();
    }
    "non-string panic payload".to_string()
}

pub(crate) fn run_codegen_with_boundary<T>(
    context: impl Into<String>,
    f: impl FnOnce() -> T,
) -> Result<T, Box<RenderedDiagnostic>> {
    let context = context.into();
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => Ok(value),
        Err(payload) => Err(Box::new(diagnostic_with_code(
            format!("{context}: {}", panic_payload_message(payload.as_ref())),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ))),
    }
}
