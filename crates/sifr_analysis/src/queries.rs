use ruff_text_size::TextRange;
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::FileId;
use sifr_syntax::TextPosition;

pub type FormatOptions = sifr_format::FormatOptions;
pub type TextEdit = sifr_format::TextEdit;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompletionItems {
    pub items: Vec<CompletionItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverInfo {
    pub contents: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureHelp {
    pub label: String,
    pub active_parameter: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub file: FileId,
    pub range: Option<TextRange>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub location: Location,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolName(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceEdit {
    pub edits: Vec<FileTextEdits>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileTextEdits {
    pub file: FileId,
    pub edits: Vec<TextEdit>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: String,
    pub file: FileId,
    pub range: Option<TextRange>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolQuery {
    pub query: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceSymbol {
    pub name: String,
    pub kind: String,
    pub file: FileId,
    pub container_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticToken {
    pub range: TextRange,
    pub token_type: String,
    pub modifiers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlayHint {
    pub position: TextPosition,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentHighlight {
    pub range: TextRange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoldingRange {
    pub range: TextRange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionRange {
    pub range: TextRange,
    pub parent: Option<Box<SelectionRange>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeHierarchyItemId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeHierarchyItem {
    pub id: TypeHierarchyItemId,
    pub name: String,
    pub kind: String,
    pub location: Location,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeActionContext {
    pub diagnostics: Vec<DiagnosticId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeAction {
    pub title: String,
    pub kind: String,
    pub edit: Option<WorkspaceEdit>,
    pub data: Option<CodeActionData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeActionData {
    pub action: DeferredCodeAction,
    pub file: FileId,
    pub expected_version: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeferredCodeAction {
    FixAllSafePolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedRustPreview {
    pub file: FileId,
    pub range: Option<TextRange>,
    pub rust: Option<String>,
    pub unavailable_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticClass {
    Hard,
    Policy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticId {
    pub code: String,
    pub class: DiagnosticClass,
    pub rule_id: Option<String>,
}

impl DiagnosticId {
    pub fn hard(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            class: DiagnosticClass::Hard,
            rule_id: None,
        }
    }

    pub fn policy(code: impl Into<String>, rule_id: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            class: DiagnosticClass::Policy,
            rule_id: Some(rule_id.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticExplanation {
    pub diagnostic: Option<RenderedDiagnostic>,
    pub unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestItemId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestItem {
    pub id: TestItemId,
    pub label: String,
    pub file: Option<FileId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestCommandKind {
    Check,
    Run,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestCommand {
    pub kind: TestCommandKind,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileDiagnostics {
    pub file: Option<FileId>,
    pub diagnostics: Vec<RenderedDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenameTarget {
    pub symbol: WorkspaceSymbol,
}
