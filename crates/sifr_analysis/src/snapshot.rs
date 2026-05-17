use sifr_frontend::{GraphRevision, SourceRevision};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnalysisRevision {
    pub graph: GraphRevision,
    pub source: SourceRevision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisQueryKind {
    Diagnostics,
    WorkspaceDiagnostics,
    Completion,
    Hover,
    SignatureHelp,
    Definition,
    Declaration,
    TypeDefinition,
    References,
    PrepareRename,
    Rename,
    DocumentSymbols,
    WorkspaceSymbols,
    SemanticTokens,
    InlayHints,
    DocumentHighlights,
    FoldingRanges,
    SelectionRanges,
    PrepareTypeHierarchy,
    TypeHierarchySupertypes,
    TypeHierarchySubtypes,
    CodeActions,
    FormatDocument,
    FormatRange,
    GeneratedRustPreview,
    ExplainDiagnostic,
    DiscoverTests,
    TestCommand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueryMetadata {
    pub query: AnalysisQueryKind,
    pub revision: AnalysisRevision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisQueryResult<T> {
    value: T,
    metadata: QueryMetadata,
}

impl<T> AnalysisQueryResult<T> {
    #[must_use]
    pub fn new(value: T, metadata: QueryMetadata) -> Self {
        Self { value, metadata }
    }

    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[must_use]
    pub fn metadata(&self) -> QueryMetadata {
        self.metadata
    }

    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisSnapshot {
    revision: AnalysisRevision,
}

impl AnalysisSnapshot {
    #[must_use]
    pub(crate) fn new(revision: AnalysisRevision) -> Self {
        Self { revision }
    }

    #[must_use]
    pub fn revision(&self) -> AnalysisRevision {
        self.revision
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnalysisErrorKind {
    UnknownFile,
    UnknownSymbol,
    StaleDocumentVersion,
    StaleSnapshot,
    InvalidFormatRange,
    FrontendDiagnostic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisError {
    pub kind: AnalysisErrorKind,
    pub message: String,
}

impl AnalysisError {
    #[must_use]
    pub fn new(kind: AnalysisErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}
