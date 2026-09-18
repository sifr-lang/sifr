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
    PythonInterop,
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
    compiler: SnapshotCompiler,
}

impl AnalysisSnapshot {
    #[must_use]
    pub(crate) fn new(
        workspace: WorkspaceSnapshot,
        revision: AnalysisRevision,
        compiler: sifr_driver::CompilerContext,
        owner: std::sync::Arc<()>,
    ) -> Self {
        Self {
            workspace,
            revision,
            compiler: SnapshotCompiler(compiler, owner),
        }
    }

    #[must_use]
    pub(crate) fn belongs_to(&self, owner: &std::sync::Arc<()>) -> bool {
        std::sync::Arc::ptr_eq(&self.compiler.1, owner)
    }

    pub(crate) fn matches_compiler(&self, compiler: &sifr_driver::CompilerContext) -> bool {
        self.compiler.0.shares_metadata_generation(compiler)
    }

    /// This is a captured source-authority description, not disk-cache eligibility:
    /// a future writer must revalidate every saved input before publication.
    pub fn has_unsaved_overlays(&self) -> bool {
        self.workspace
            .overlays
            .iter()
            .any(|overlay| !overlay.matches_disk)
    }

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

// Retaining a snapshot pins the exact toolchain owner even after host closure or
// explicit re-resolution. Equality is generation identity, never live disk state.
#[derive(Clone)]
struct SnapshotCompiler(sifr_driver::CompilerContext, std::sync::Arc<()>);
impl std::fmt::Debug for SnapshotCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SnapshotCompiler")
            .field(self.0.identity())
            .finish()
    }
}
impl PartialEq for SnapshotCompiler {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.1, &other.1) && self.0.shares_metadata_generation(&other.0)
    }
}
impl Eq for SnapshotCompiler {}
