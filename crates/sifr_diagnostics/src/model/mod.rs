use crate::codes::DiagnosticCode;
use crate::source_map::SourceSpan;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum ChildSeverity {
    Note,
    Help,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DiagnosticArg {
    String(String),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
}

impl DiagnosticArg {
    #[must_use]
    pub fn canonical_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

static UNEMITTED_DIAGNOSTIC_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

#[must_use]
pub fn take_unemitted_diagnostic_drop_count() -> usize {
    UNEMITTED_DIAGNOSTIC_DROP_COUNT.swap(0, Ordering::SeqCst)
}

impl From<&str> for DiagnosticArg {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for DiagnosticArg {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<i64> for DiagnosticArg {
    fn from(value: i64) -> Self {
        Self::Signed(value)
    }
}

impl From<i32> for DiagnosticArg {
    fn from(value: i32) -> Self {
        Self::Signed(i64::from(value))
    }
}

impl From<u64> for DiagnosticArg {
    fn from(value: u64) -> Self {
        Self::Unsigned(value)
    }
}

impl From<u32> for DiagnosticArg {
    fn from(value: u32) -> Self {
        Self::Unsigned(u64::from(value))
    }
}

impl From<bool> for DiagnosticArg {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for DiagnosticArg {
    fn from(value: f64) -> Self {
        assert!(value.is_finite(), "diagnostic float args must be finite");
        Self::Float(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticChild {
    pub severity: ChildSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedSpan {
    pub span: SourceSpan,
    pub label: Option<String>,
    pub kind: RelatedKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelatedKind {
    Label,
    Note,
    Origin,
    ReplacementTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticSuggestion {
    pub message: String,
    pub applicability: SuggestionApplicability,
    pub edits: Vec<SuggestionEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestionEdit {
    pub span: SourceSpan,
    pub replacement: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SuggestionApplicability {
    MachineApplicable,
    MaybeIncorrect,
    HasPlaceholders,
    Unspecified,
}

#[must_use]
#[derive(Debug)]
pub enum SifrDiagnostic {
    Source(SourceDiagnostic),
    Internal(InternalDiagnostic),
}

#[derive(Debug)]
pub struct SourceDiagnostic {
    pub(crate) code: DiagnosticCode,
    pub(crate) severity: Severity,
    pub(crate) message: String,
    pub(crate) message_template: &'static str,
    pub(crate) args: BTreeMap<String, DiagnosticArg>,
    pub(crate) primary_span: SourceSpan,
    pub(crate) related_spans: Vec<RelatedSpan>,
    pub(crate) children: Vec<DiagnosticChild>,
    pub(crate) help: Option<String>,
    pub(crate) suggestions: Vec<DiagnosticSuggestion>,
    consumed: Cell<bool>,
}

#[derive(Debug)]
pub struct InternalDiagnostic {
    pub(crate) code: DiagnosticCode,
    pub(crate) severity: Severity,
    pub(crate) message: String,
    pub(crate) message_template: &'static str,
    pub(crate) args: BTreeMap<String, DiagnosticArg>,
    pub(crate) children: Vec<DiagnosticChild>,
    pub(crate) help: Option<String>,
    consumed: Cell<bool>,
}

impl SifrDiagnostic {
    #[must_use]
    pub fn code(&self) -> DiagnosticCode {
        match self {
            Self::Source(diag) => diag.code,
            Self::Internal(diag) => diag.code,
        }
    }

    #[must_use]
    pub fn severity(&self) -> Severity {
        match self {
            Self::Source(diag) => diag.severity,
            Self::Internal(diag) => diag.severity,
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::Source(diag) => &diag.message,
            Self::Internal(diag) => &diag.message,
        }
    }

    #[must_use]
    pub fn message_template(&self) -> &'static str {
        match self {
            Self::Source(diag) => diag.message_template,
            Self::Internal(diag) => diag.message_template,
        }
    }

    #[must_use]
    pub fn args(&self) -> &BTreeMap<String, DiagnosticArg> {
        match self {
            Self::Source(diag) => &diag.args,
            Self::Internal(diag) => &diag.args,
        }
    }

    #[must_use]
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Source(diag) => Some(diag.primary_span.clone()),
            Self::Internal(_) => None,
        }
    }

    #[must_use]
    pub fn related_spans(&self) -> &[RelatedSpan] {
        match self {
            Self::Source(diag) => &diag.related_spans,
            Self::Internal(_) => &[],
        }
    }

    #[must_use]
    pub fn children(&self) -> &[DiagnosticChild] {
        match self {
            Self::Source(diag) => &diag.children,
            Self::Internal(diag) => &diag.children,
        }
    }

    #[must_use]
    pub fn help(&self) -> Option<&str> {
        match self {
            Self::Source(diag) => diag.help.as_deref(),
            Self::Internal(diag) => diag.help.as_deref(),
        }
    }

    #[must_use]
    pub fn suggestions(&self) -> &[DiagnosticSuggestion] {
        match self {
            Self::Source(diag) => &diag.suggestions,
            Self::Internal(_) => &[],
        }
    }

    pub fn cancel(self) {
        self.mark_consumed();
    }

    pub(crate) fn mark_consumed(&self) {
        match self {
            Self::Source(diag) => diag.consumed.set(true),
            Self::Internal(diag) => diag.consumed.set(true),
        }
    }
}

impl Drop for SourceDiagnostic {
    fn drop(&mut self) {
        if !self.consumed.get() && !std::thread::panicking() {
            UNEMITTED_DIAGNOSTIC_DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            debug_assert!(
                false,
                "source diagnostic was dropped without emission, return, or cancel"
            );
        }
    }
}

impl Drop for InternalDiagnostic {
    fn drop(&mut self) {
        if !self.consumed.get() && !std::thread::panicking() {
            UNEMITTED_DIAGNOSTIC_DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            debug_assert!(
                false,
                "internal diagnostic was dropped without emission, return, or cancel"
            );
        }
    }
}

#[must_use]
pub struct DiagnosticBuilder {
    kind: DiagnosticBuilderKind,
    code: DiagnosticCode,
    severity: Severity,
    message_template: Option<&'static str>,
    args: BTreeMap<String, DiagnosticArg>,
    related_spans: Vec<RelatedSpan>,
    children: Vec<DiagnosticChild>,
    help: Option<String>,
    suggestions: Vec<DiagnosticSuggestion>,
    consumed: bool,
}

enum DiagnosticBuilderKind {
    Source { primary_span: SourceSpan },
    Internal,
}

impl DiagnosticBuilder {
    pub fn source(code: DiagnosticCode, severity: Severity, primary_span: SourceSpan) -> Self {
        assert_eq!(
            severity,
            code.declared_severity(),
            "diagnostic severity must match the registry-declared severity"
        );
        Self::new(
            DiagnosticBuilderKind::Source { primary_span },
            code,
            severity,
        )
    }

    pub fn internal(code: DiagnosticCode, severity: Severity) -> Self {
        assert_eq!(
            severity,
            code.declared_severity(),
            "diagnostic severity must match the registry-declared severity"
        );
        Self::new(DiagnosticBuilderKind::Internal, code, severity)
    }

    fn new(kind: DiagnosticBuilderKind, code: DiagnosticCode, severity: Severity) -> Self {
        Self {
            kind,
            code,
            severity,
            message_template: None,
            args: BTreeMap::new(),
            related_spans: Vec::new(),
            children: Vec::new(),
            help: None,
            suggestions: Vec::new(),
            consumed: false,
        }
    }

    pub fn message_template(mut self, template: &'static str) -> Self {
        self.message_template = Some(template);
        self
    }

    pub fn arg(mut self, name: &'static str, value: impl Into<DiagnosticArg>) -> Self {
        assert_valid_placeholder(name);
        assert!(
            self.args.insert(name.to_string(), value.into()).is_none(),
            "duplicate diagnostic arg `{name}`"
        );
        self
    }

    pub fn arg_owned(mut self, name: &str, value: impl Into<DiagnosticArg>) -> Self {
        assert_valid_placeholder(name);
        assert!(
            self.args.insert(name.to_string(), value.into()).is_none(),
            "duplicate diagnostic arg `{name}`"
        );
        self
    }

    pub fn related(mut self, span: SourceSpan, kind: RelatedKind, label: Option<String>) -> Self {
        self.related_spans.push(RelatedSpan { span, label, kind });
        self
    }

    pub fn child(mut self, severity: ChildSeverity, message: impl Into<String>) -> Self {
        self.children.push(DiagnosticChild {
            severity,
            message: message.into(),
        });
        self
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn suggestion(mut self, suggestion: DiagnosticSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn build(mut self) -> SifrDiagnostic {
        self.consumed = true;
        let Some(template) = self.message_template else {
            panic!("diagnostic message_template is required");
        };
        validate_template_args(template, &self.args);
        let message = render_message_template(template, &self.args);
        let args = std::mem::take(&mut self.args);
        let related_spans = std::mem::take(&mut self.related_spans);
        let children = std::mem::take(&mut self.children);
        let help = self.help.take();
        let suggestions = std::mem::take(&mut self.suggestions);
        match &self.kind {
            DiagnosticBuilderKind::Source { primary_span } => {
                SifrDiagnostic::Source(SourceDiagnostic {
                    code: self.code,
                    severity: self.severity,
                    message,
                    message_template: template,
                    args,
                    primary_span: primary_span.clone(),
                    related_spans,
                    children,
                    help,
                    suggestions,
                    consumed: Cell::new(false),
                })
            }
            DiagnosticBuilderKind::Internal => SifrDiagnostic::Internal(InternalDiagnostic {
                code: self.code,
                severity: self.severity,
                message,
                message_template: template,
                args,
                children,
                help,
                consumed: Cell::new(false),
            }),
        }
    }

    pub fn cancel(mut self) {
        self.consumed = true;
    }
}

impl Drop for DiagnosticBuilder {
    fn drop(&mut self) {
        if !self.consumed && !std::thread::panicking() {
            UNEMITTED_DIAGNOSTIC_DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            debug_assert!(
                false,
                "diagnostic builder was dropped without build or cancel"
            );
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ErrorEmitted(());

#[derive(Debug)]
pub struct AdmittedDiagnostic {
    diagnostic: SifrDiagnostic,
    insertion_order: u64,
}

impl AdmittedDiagnostic {
    pub fn diagnostic(&self) -> &SifrDiagnostic {
        &self.diagnostic
    }

    #[must_use]
    pub const fn insertion_order(&self) -> u64 {
        self.insertion_order
    }
}

#[derive(Default, Debug)]
pub struct DiagnosticSink {
    diagnostics: Vec<AdmittedDiagnostic>,
    next_insertion_order: u64,
}

impl DiagnosticSink {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&mut self, diag: SifrDiagnostic) {
        assert!(
            matches!(diag.severity(), Severity::Warning | Severity::Note),
            "DiagnosticSink::emit accepts only warning or note diagnostics"
        );
        self.push(diag);
    }

    pub fn emit_error(&mut self, diag: SifrDiagnostic) -> ErrorEmitted {
        assert_eq!(
            diag.severity(),
            Severity::Error,
            "DiagnosticSink::emit_error accepts only error diagnostics"
        );
        self.push(diag);
        ErrorEmitted(())
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[AdmittedDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|entry| entry.diagnostic.severity() == Severity::Error)
    }

    fn push(&mut self, diag: SifrDiagnostic) {
        diag.mark_consumed();
        let insertion_order = self.next_insertion_order;
        self.next_insertion_order = self
            .next_insertion_order
            .checked_add(1)
            .unwrap_or_else(|| panic!("diagnostic insertion order overflowed"));
        self.diagnostics.push(AdmittedDiagnostic {
            diagnostic: diag,
            insertion_order,
        });
    }
}

fn validate_template_args(template: &str, args: &BTreeMap<String, DiagnosticArg>) {
    for placeholder in extract_placeholders(template) {
        assert!(
            args.contains_key(placeholder),
            "diagnostic template placeholder {{{placeholder}}} has no matching arg"
        );
    }
}

fn render_message_template(template: &str, args: &BTreeMap<String, DiagnosticArg>) -> String {
    let mut output = String::new();
    let mut chars = template.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch == '{' {
            if matches!(chars.peek(), Some((_, '{'))) {
                chars.next();
                output.push('{');
                continue;
            }
            let mut name = String::new();
            let mut closed = false;
            for (_, next) in chars.by_ref() {
                if next == '}' {
                    closed = true;
                    break;
                }
                name.push(next);
            }
            assert!(closed, "diagnostic template has an unclosed placeholder");
            assert_valid_placeholder(&name);
            let value = args
                .get(&name)
                .unwrap_or_else(|| panic!("missing diagnostic arg `{name}`"));
            output.push_str(&format_arg(value));
        } else if ch == '}' {
            if matches!(chars.peek(), Some((_, '}'))) {
                chars.next();
                output.push('}');
            } else {
                panic!("diagnostic template has an unmatched closing brace");
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn extract_placeholders(template: &str) -> Vec<&str> {
    let mut placeholders = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        rest = &rest[start + 1..];
        if let Some(stripped) = rest.strip_prefix('{') {
            rest = stripped;
            continue;
        }
        let Some(end) = rest.find('}') else {
            panic!("diagnostic template has an unclosed placeholder");
        };
        let name = &rest[..end];
        assert_valid_placeholder(name);
        placeholders.push(name);
        rest = &rest[end + 1..];
    }
    placeholders
}

fn assert_valid_placeholder(name: &str) {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        panic!("diagnostic template has an empty placeholder");
    };
    assert!(
        first.is_ascii_lowercase()
            && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'),
        "diagnostic placeholder `{name}` must match [a-z][a-z0-9_]*"
    );
}

fn format_arg(value: &DiagnosticArg) -> String {
    match value {
        DiagnosticArg::String(value) => value.clone(),
        DiagnosticArg::Signed(value) => value.to_string(),
        DiagnosticArg::Unsigned(value) => value.to_string(),
        DiagnosticArg::Float(value) => serde_json::to_string(value)
            .unwrap_or_else(|err| panic!("failed to render finite diagnostic float arg: {err}")),
        DiagnosticArg::Bool(value) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DiagnosticBuilder, DiagnosticSink, Severity, SifrDiagnostic,
        take_unemitted_diagnostic_drop_count,
    };
    use crate::codes::DiagnosticCode;
    use crate::source_map::{SourceMap, SourceSpan};
    use ruff_text_size::{TextRange, TextSize};

    fn span() -> SourceSpan {
        let mut source_map = SourceMap::new();
        let source_id = source_map.register_source("test.sifr", "x = y\n");
        SourceSpan::new(
            source_id,
            TextRange::new(TextSize::new(4), TextSize::new(5)),
        )
    }

    #[test]
    fn builder_renders_message_from_template_and_args() {
        let diag =
            DiagnosticBuilder::source(DiagnosticCode::TEST_SOURCE_ERROR, Severity::Error, span())
                .message_template("undefined variable: {name}")
                .arg("name", "answer")
                .build();
        assert_eq!(diag.message(), "undefined variable: answer");
        assert_eq!(diag.message_template(), "undefined variable: {name}");
        diag.cancel();
    }

    #[test]
    fn builder_cancel_consumes_without_emission() {
        DiagnosticBuilder::internal(DiagnosticCode::TEST_INTERNAL_ERROR, Severity::Error)
            .message_template("internal compiler error")
            .cancel();
    }

    #[test]
    fn diagnostic_cancel_consumes_built_diagnostic() {
        let diag =
            DiagnosticBuilder::internal(DiagnosticCode::TEST_INTERNAL_ERROR, Severity::Error)
                .message_template("internal compiler error")
                .build();
        diag.cancel();
    }

    #[test]
    fn sink_records_errors_with_insertion_order() {
        let mut sink = DiagnosticSink::new();
        let diag =
            DiagnosticBuilder::source(DiagnosticCode::TEST_SOURCE_ERROR, Severity::Error, span())
                .message_template("undefined variable: {name}")
                .arg("name", "x")
                .build();
        let _proof = sink.emit_error(diag);
        assert!(sink.has_errors());
        assert_eq!(sink.diagnostics()[0].insertion_order(), 0);
    }

    #[test]
    fn warning_and_note_use_non_error_emit_path() {
        let mut sink = DiagnosticSink::new();
        let diag = DiagnosticBuilder::internal(DiagnosticCode::TEST_NOTE, Severity::Note)
            .message_template("{count} additional diagnostics omitted by recovery cap")
            .arg("count", 3_u32)
            .build();
        sink.emit(diag);
        assert_eq!(
            sink.diagnostics()[0].diagnostic().severity(),
            Severity::Note
        );
    }

    #[test]
    #[should_panic(expected = "accepts only warning or note")]
    fn emit_rejects_errors() {
        let mut sink = DiagnosticSink::new();
        let diag =
            DiagnosticBuilder::internal(DiagnosticCode::TEST_INTERNAL_ERROR, Severity::Error)
                .message_template("internal compiler error")
                .build();
        sink.emit(diag);
    }

    #[test]
    #[should_panic(expected = "accepts only error")]
    fn emit_error_rejects_notes() {
        let mut sink = DiagnosticSink::new();
        let diag = DiagnosticBuilder::internal(DiagnosticCode::TEST_NOTE, Severity::Note)
            .message_template("{count} additional diagnostics omitted by recovery cap")
            .arg("count", 3_u32)
            .build();
        let _proof = sink.emit_error(diag);
    }

    #[test]
    fn diagnostic_is_not_clone() {
        static_assertions::assert_not_impl_any!(SifrDiagnostic: Clone);
        static_assertions::assert_not_impl_any!(DiagnosticBuilder: Clone);
        static_assertions::assert_eq_size!(super::ErrorEmitted, ());
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "diagnostic builder was dropped without build or cancel")]
    fn dropping_builder_without_consumption_panics_in_debug() {
        let _builder =
            DiagnosticBuilder::internal(DiagnosticCode::TEST_INTERNAL_ERROR, Severity::Error)
                .message_template("internal compiler error");
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(
        expected = "internal diagnostic was dropped without emission, return, or cancel"
    )]
    fn dropping_diagnostic_without_consumption_panics_in_debug() {
        let _diag =
            DiagnosticBuilder::internal(DiagnosticCode::TEST_INTERNAL_ERROR, Severity::Error)
                .message_template("internal compiler error")
                .build();
    }

    #[test]
    fn release_drop_violation_hook_is_queryable() {
        let _ = take_unemitted_diagnostic_drop_count();
        assert_eq!(take_unemitted_diagnostic_drop_count(), 0);
    }

    #[test]
    #[should_panic(expected = "duplicate diagnostic arg")]
    fn duplicate_args_are_rejected() {
        DiagnosticBuilder::internal(DiagnosticCode::TEST_INTERNAL_ERROR, Severity::Error)
            .message_template("{name}")
            .arg("name", "x")
            .arg("name", "y")
            .cancel();
    }
}
