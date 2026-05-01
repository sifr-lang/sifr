use serde::Serialize;
use sifr_diagnostics::DiagnosticCode;
use std::any::Any;
use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Debug)]
pub enum CompileResult {
    Success { rust_source: String },
    Errors { errors: Vec<CompileError> },
}

pub enum CompileResultFull {
    Success {
        rust_source: String,
        used_stdlib_modules: HashSet<String>,
        required_crates: HashSet<String>,
        lowering_stats: sifr_codegen::LoweringStats,
    },
    Errors {
        errors: Vec<CompileError>,
    },
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub phase: CompilePhase,
    pub code: DiagnosticCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilePhase {
    Parse,
    TypeCheck,
    Codegen,
    Build,
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

impl CompileError {
    /// Creates a compile error with canonical diagnostic identity.
    #[must_use]
    pub fn with_code(
        message: impl Into<String>,
        phase: CompilePhase,
        code: DiagnosticCode,
    ) -> Self {
        Self {
            message: message.into(),
            phase,
            code,
        }
    }

    fn diagnostic_code(&self) -> &'static str {
        self.code.code()
    }

    fn diagnostic_severity() -> Severity {
        Severity::Error
    }

    pub fn to_diagnostic(&self) -> CompilerDiagnostic {
        let code = self.diagnostic_code().to_string();
        CompilerDiagnostic {
            url: format!("https://sifr.sh/docs/errors/{code}"),
            code,
            severity: Self::diagnostic_severity(),
            message: self.message.clone(),
            primary_span: None,
            related_spans: Vec::new(),
            children: Vec::new(),
            help: None,
            suggestions: Vec::new(),
        }
    }
}

pub fn compile_errors_to_diagnostics(errors: &[CompileError]) -> Vec<CompilerDiagnostic> {
    errors.iter().map(CompileError::to_diagnostic).collect()
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

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let phase = match self.phase {
            CompilePhase::Parse => "parse error",
            CompilePhase::TypeCheck => "type error",
            CompilePhase::Codegen => "codegen error",
            CompilePhase::Build => "build error",
        };
        write!(f, "{}: {}", phase, self.message)
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
) -> Result<T, CompileError> {
    let context = context.into();
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => Ok(value),
        Err(payload) => Err(CompileError::with_code(
            format!("{context}: {}", panic_payload_message(payload.as_ref())),
            CompilePhase::Codegen,
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        )),
    }
}
