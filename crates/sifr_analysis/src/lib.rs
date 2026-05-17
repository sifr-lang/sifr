//! Editor-oriented Sifr analysis queries over the canonical frontend.
//!
//! This crate owns the Phase 36 editor session boundary. It deliberately routes
//! compiler facts through `sifr_frontend`, formatting through `sifr_format`, and
//! policy diagnostics through `sifr_lint`.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod completion;
mod editor;
mod host;
mod queries;
mod snapshot;
mod symbols;

pub use completion::{
    evaluate_completion_ranking, rank_completion_candidates, CompletionCandidate,
    CompletionEvaluation, CompletionRankingResult,
};
pub use host::AnalysisHost;
pub use queries::{
    CodeAction, CodeActionContext, CompletionItem, CompletionItems, Declaration,
    DiagnosticExplanation, DiagnosticId, DocumentHighlight, DocumentSymbol, FileDiagnostics,
    FoldingRange, FormatOptions, GeneratedRustPreview, HoverInfo, InlayHint, Location,
    RenameTarget, SelectionRange, SemanticToken, SignatureHelp, SymbolName, SymbolQuery,
    TestCommand, TestCommandKind, TestItem, TestItemId, TextEdit, TypeHierarchyItem,
    TypeHierarchyItemId, WorkspaceEdit, WorkspaceSymbol,
};
pub use snapshot::{
    AnalysisError, AnalysisErrorKind, AnalysisQueryKind, AnalysisQueryResult, AnalysisRevision,
    AnalysisSnapshot, QueryMetadata,
};
pub use symbols::{SymbolId, SymbolIndex, SymbolIndexEntry};

pub use sifr_frontend::{
    DocumentVersion, FileId, FrontendInput, InvalidationReport, ProjectRoot, SourcePath, SourceText,
};
pub use sifr_syntax::TextPosition;
