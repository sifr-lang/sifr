use sifr_frontend::{GraphRevision, SourceRevision, WorkspaceSnapshot, WorkspaceSnapshotId};

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
    pub workspace_snapshot_id: Option<WorkspaceSnapshotId>,
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

    #[must_use]
    pub(crate) fn with_workspace_snapshot_id(mut self, snapshot_id: WorkspaceSnapshotId) -> Self {
        self.metadata.workspace_snapshot_id = Some(snapshot_id);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisSnapshot {
    workspace: WorkspaceSnapshot,
    revision: AnalysisRevision,
}

impl AnalysisSnapshot {
    #[must_use]
    pub(crate) fn new(workspace: WorkspaceSnapshot, revision: AnalysisRevision) -> Self {
        Self {
            workspace,
            revision,
        }
    }

    #[must_use]
    pub fn revision(&self) -> AnalysisRevision {
        self.revision
    }

    #[must_use]
    pub fn workspace(&self) -> &WorkspaceSnapshot {
        &self.workspace
    }

    #[must_use]
    pub fn workspace_snapshot_id(&self) -> WorkspaceSnapshotId {
        self.workspace.id
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
