//! Editor-oriented Sifr analysis queries over the canonical frontend.
//!
//! This crate owns the editor tooling editor session boundary. It deliberately routes
//! compiler facts through `sifr_frontend`, formatting through `sifr_format`, and
//! policy diagnostics through `sifr_lint`.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod completion;
mod editor;
mod handles;
mod host;
mod queries;
mod snapshot;
mod symbols;
mod worker_lanes;

pub use completion::{
    evaluate_completion_ranking, rank_completion_candidates, CompletionCandidate,
    CompletionEvaluation, CompletionRankingResult,
};
pub use host::AnalysisHost;
pub use queries::{
    CodeAction, CodeActionContext, CodeActionData, CompletionItem, CompletionItems, Declaration,
    DeferredCodeAction, DiagnosticClass, DiagnosticExplanation, DiagnosticId, DocumentHighlight,
    DocumentSymbol, FileDiagnostics, FileTextEdits, FoldingRange, FormatOptions,
    GeneratedRustPreview, HoverInfo, InlayHint, Location, RenameTarget, SelectionRange,
    SemanticToken, SignatureHelp, SymbolName, SymbolQuery, TestCommand, TestCommandKind, TestItem,
    TestItemId, TextEdit, TypeHierarchyItem, TypeHierarchyItemId, WorkspaceEdit, WorkspaceSymbol,
};
pub use snapshot::{
    AnalysisError, AnalysisErrorKind, AnalysisQueryKind, AnalysisQueryResult, AnalysisRevision,
    AnalysisSnapshot, QueryMetadata,
};
pub use symbols::{
    SymbolBucketId, SymbolBucketKind, SymbolBucketReadiness, SymbolBucketReadinessState, SymbolId,
    SymbolIndex, SymbolIndexEntry,
};
pub use worker_lanes::{
    ApprovedWorkerLane, SingleOwnerCompilerPhase, APPROVED_WORKER_LANES, SINGLE_OWNER_PHASES,
};

pub use sifr_frontend::{
    DocumentVersion, FileId, FrontendInput, FrontendMode, InvalidationReport, ProjectRoot,
    SourcePath, SourceText, WorkspaceTraceEvent, WorkspaceTraceLog, WorkspaceTracePhase,
};
pub use sifr_syntax::TextPosition;

pub use sifr_driver::ToolingSysrootStatus;

pub fn tooling_sysroot_status(
) -> Result<ToolingSysrootStatus, Vec<sifr_diagnostics::RenderedDiagnostic>> {
    sifr_driver::stdlib_tooling_sysroot_status()
}

pub fn format_options_for_path(
    path: &std::path::Path,
) -> Result<FormatOptions, Vec<sifr_diagnostics::RenderedDiagnostic>> {
    sifr_format::config::effective_format_options_for_file(path)
}
