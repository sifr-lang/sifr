use sifr_diagnostics::codes::registry_entry;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
pub(crate) use sifr_diagnostics::{DiagnosticSpan, RenderedDiagnostic, Severity};
use std::any::Any;
use std::collections::{BTreeMap, HashSet};
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
        used_stdlib_modules: HashSet<String>,
        required_crates: HashSet<String>,
        lowering_stats: sifr_codegen::LoweringStats,
    },
    Errors {
        errors: Vec<RenderedDiagnostic>,
    },
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
    let mut grouped: BTreeMap<RecoveryGroupKey, Vec<RenderedDiagnostic>> = BTreeMap::new();
    for diagnostic in diagnostics {
        let key = RecoveryGroupKey::from_diagnostic(diagnostic);
        grouped.entry(key).or_default().push(diagnostic.clone());
    }

    let mut bounded = Vec::new();
    for (_key, group) in grouped {
        let retained = group.len().min(MAX_SIMILAR_DIAGNOSTICS_PER_GROUP);
        for diagnostic in group.iter().take(retained) {
            bounded.push(diagnostic.clone());
        }
        if group.len() > MAX_SIMILAR_DIAGNOSTICS_PER_GROUP {
            bounded.push(recovery_omission_summary(
                group.len() - MAX_SIMILAR_DIAGNOSTICS_PER_GROUP,
                SIMILAR_DIAGNOSTIC_CAP_KIND,
            ));
        }
    }

    if bounded.len() > MAX_TOP_LEVEL_DIAGNOSTICS {
        let omitted = bounded.len() - (MAX_TOP_LEVEL_DIAGNOSTICS - 1);
        bounded.truncate(MAX_TOP_LEVEL_DIAGNOSTICS - 1);
        bounded.push(recovery_omission_summary(omitted, TOP_LEVEL_CAP_KIND));
    }
    bounded
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RecoveryGroupKey {
    severity_rank: u8,
    code: String,
    message_template: String,
    dedupe_args: Vec<(String, String)>,
    primary_file: Option<String>,
}

impl RecoveryGroupKey {
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

fn recovery_dedupe_args(diagnostic: &RenderedDiagnostic) -> Vec<(String, String)> {
    if let Some(entry) = registry_entry(&diagnostic.code) {
        let mut key_args = Vec::new();
        for arg in entry.dedupe_args {
            if let Some(value) = diagnostic.args.get(*arg) {
                key_args.push(((*arg).to_string(), diagnostic_arg_key(value)));
            }
        }
        for (arg, value) in &diagnostic.args {
            if !entry.dedupe_args.contains(&arg.as_str()) {
                key_args.push((arg.clone(), diagnostic_arg_key(value)));
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
    match arg {
        DiagnosticArg::String(value) => value.clone(),
        DiagnosticArg::Signed(value) => value.to_string(),
        DiagnosticArg::Unsigned(value) => value.to_string(),
        DiagnosticArg::Float(value) => value.to_string(),
        DiagnosticArg::Bool(value) => value.to_string(),
    }
}

fn recovery_omission_summary(omitted_count: usize, cap_kind: &'static str) -> RenderedDiagnostic {
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
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message: format!(
            "{omitted_count} additional diagnostics omitted by recovery cap ({cap_kind})"
        ),
        message_template:
            "{omitted_count} additional diagnostics omitted by recovery cap ({cap_kind})"
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
