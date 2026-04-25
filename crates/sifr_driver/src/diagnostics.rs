use serde::Serialize;
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
    fn workspace_diagnostic_code(&self) -> Option<&'static str> {
        if self.phase != CompilePhase::Build {
            return None;
        }
        let message = self.message.as_str();
        if message.starts_with("could not parse sifr.toml at ") {
            return Some("SIFR-WORKSPACE-0001");
        }
        if message.starts_with("[source].roots entry ") {
            if message.contains(" escapes the workspace root via '..'") {
                return Some("SIFR-WORKSPACE-0002");
            }
            if message.contains(" is not a directory under the workspace root") {
                return Some("SIFR-WORKSPACE-0003");
            }
            if message.contains(" must be a relative non-empty path under the workspace root") {
                return Some("SIFR-WORKSPACE-0004");
            }
        }
        if message.starts_with("could not resolve import ") {
            return Some("SIFR-WORKSPACE-0101");
        }
        if message.starts_with("module ") && message.contains(" is ambiguous in workspace ") {
            return Some("SIFR-WORKSPACE-0102");
        }
        if message.starts_with("module ")
            && message.contains(" resolves to file ")
            && message.contains("package directories are not supported in this phase")
        {
            return Some("SIFR-WORKSPACE-0103");
        }
        None
    }

    fn diagnostic_code(&self) -> &'static str {
        if let Some(code) = self.workspace_diagnostic_code() {
            return code;
        }
        match self.phase {
            CompilePhase::Parse => "SIFR-PARSE-0001",
            CompilePhase::TypeCheck => "SIFR-TYPE-0001",
            CompilePhase::Codegen => "SIFR-CODEGEN-0001",
            CompilePhase::Build => "SIFR-BUILD-0001",
        }
    }

    fn diagnostic_severity() -> Severity {
        Severity::Error
    }

    pub fn to_diagnostic(&self) -> CompilerDiagnostic {
        let code = self.diagnostic_code().to_string();
        CompilerDiagnostic {
            url: format!("https://sifr.dev/docs/errors/{code}"),
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
        Err(payload) => Err(CompileError {
            message: format!("{context}: {}", panic_payload_message(payload.as_ref())),
            phase: CompilePhase::Codegen,
        }),
    }
}
