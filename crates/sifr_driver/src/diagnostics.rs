use serde::Serialize;
use sifr_diagnostics::DiagnosticCode;
use std::any::Any;
use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Debug)]
pub enum CompileResult {
    Success { rust_source: String },
    Errors { errors: Vec<CompilerDiagnostic> },
}

pub enum CompileResultFull {
    Success {
        rust_source: String,
        used_stdlib_modules: HashSet<String>,
        required_crates: HashSet<String>,
        lowering_stats: sifr_codegen::LoweringStats,
    },
    Errors {
        errors: Vec<CompilerDiagnostic>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SuggestionKind {
    DidYouMean,
    ReplaceText,
    InsertText,
    DeleteText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticSpan {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelatedSpan {
    pub label: String,
    pub span: DiagnosticSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticChild {
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticSuggestion {
    pub kind: SuggestionKind,
    pub message: String,
    pub replacement: Option<String>,
    pub span: Option<DiagnosticSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompilerDiagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub url: String,
    pub primary_span: Option<DiagnosticSpan>,
    pub related_spans: Vec<RelatedSpan>,
    pub children: Vec<DiagnosticChild>,
    pub help: Option<String>,
    pub suggestions: Vec<DiagnosticSuggestion>,
}

impl CompilerDiagnostic {
    /// Creates an error diagnostic with canonical diagnostic identity.
    #[must_use]
    pub fn with_code(message: impl Into<String>, code: DiagnosticCode) -> Self {
        let code = code.code().to_string();
        Self {
            url: format!("https://sifr.sh/docs/errors/{code}"),
            code,
            severity: Severity::Error,
            message: message.into(),
            primary_span: None,
            related_spans: Vec::new(),
            children: Vec::new(),
            help: None,
            suggestions: Vec::new(),
        }
    }
}

const MAX_TOP_LEVEL_DIAGNOSTICS: usize = 50;
const MAX_SIMILAR_DIAGNOSTICS_PER_GROUP: usize = 5;

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Note => 2,
        Severity::Help => 3,
    }
}

pub fn apply_diagnostic_recovery_limits(
    diagnostics: &[CompilerDiagnostic],
) -> Vec<CompilerDiagnostic> {
    let mut grouped: BTreeMap<(u8, String, String, Option<String>), Vec<CompilerDiagnostic>> =
        BTreeMap::new();
    for diagnostic in diagnostics {
        let key = (
            severity_rank(diagnostic.severity),
            diagnostic.code.clone(),
            diagnostic.message.clone(),
            diagnostic
                .primary_span
                .as_ref()
                .and_then(|span| span.file.clone()),
        );
        grouped.entry(key).or_default().push(diagnostic.clone());
    }

    let mut bounded = Vec::new();
    for ((_severity_rank, _code, _message, _file), group) in grouped {
        let retained = group.len().min(MAX_SIMILAR_DIAGNOSTICS_PER_GROUP);
        for diagnostic in group.iter().take(retained) {
            bounded.push(diagnostic.clone());
        }
        if group.len() > MAX_SIMILAR_DIAGNOSTICS_PER_GROUP {
            let mut summary = group[0].clone();
            summary.message = format!(
                "... +{} more similar diagnostics",
                group.len() - MAX_SIMILAR_DIAGNOSTICS_PER_GROUP
            );
            summary.primary_span = None;
            summary.related_spans.clear();
            summary.children.clear();
            summary.help = None;
            summary.suggestions.clear();
            bounded.push(summary);
        }
    }

    if bounded.len() > MAX_TOP_LEVEL_DIAGNOSTICS {
        bounded.truncate(MAX_TOP_LEVEL_DIAGNOSTICS);
    }
    bounded
}

impl std::fmt::Display for CompilerDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            diagnostic_label_for_code_str(&self.code),
            self.message
        )
    }
}

#[must_use]
pub fn diagnostic_label_for_code(code: DiagnosticCode) -> &'static str {
    diagnostic_label_for_code_str(code.code())
}

#[must_use]
pub fn diagnostic_label_for_code_str(code: &str) -> &'static str {
    if code == DiagnosticCode::INTERNAL_COMPILER_PANIC.code() {
        "internal compiler error"
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
) -> Result<T, Box<CompilerDiagnostic>> {
    let context = context.into();
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => Ok(value),
        Err(payload) => Err(Box::new(CompilerDiagnostic::with_code(
            format!("{context}: {}", panic_payload_message(payload.as_ref())),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ))),
    }
}
