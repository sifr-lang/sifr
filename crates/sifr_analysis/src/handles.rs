#![allow(dead_code)]

use crate::queries::{Location, WorkspaceSymbol};
use crate::snapshot::{AnalysisError, AnalysisErrorKind, AnalysisRevision, AnalysisSnapshot};
use ruff_text_size::TextRange;
use sifr_frontend::{FileId, WorkspaceSnapshotId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SnapshotHandleKind {
    Symbol,
    Type,
    Signature,
    Diagnostic,
    SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SnapshotHandleAnchor {
    pub snapshot_id: WorkspaceSnapshotId,
    pub revision: AnalysisRevision,
    pub kind: SnapshotHandleKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SymbolHandle {
    anchor: SnapshotHandleAnchor,
    symbol: WorkspaceSymbol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TypeHandle {
    anchor: SnapshotHandleAnchor,
    display: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SignatureHandle {
    anchor: SnapshotHandleAnchor,
    label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiagnosticHandle {
    anchor: SnapshotHandleAnchor,
    code: String,
    file: Option<FileId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceSpanHandle {
    anchor: SnapshotHandleAnchor,
    file: FileId,
    range: TextRange,
}

impl AnalysisSnapshot {
    #[must_use]
    pub(crate) fn symbol_handle(&self, symbol: WorkspaceSymbol) -> SymbolHandle {
        SymbolHandle {
            anchor: self.anchor(SnapshotHandleKind::Symbol),
            symbol,
        }
    }

    #[must_use]
    pub(crate) fn type_handle(&self, display: impl Into<String>) -> TypeHandle {
        TypeHandle {
            anchor: self.anchor(SnapshotHandleKind::Type),
            display: display.into(),
        }
    }

    #[must_use]
    pub(crate) fn signature_handle(&self, label: impl Into<String>) -> SignatureHandle {
        SignatureHandle {
            anchor: self.anchor(SnapshotHandleKind::Signature),
            label: label.into(),
        }
    }

    #[must_use]
    pub(crate) fn diagnostic_handle(
        &self,
        code: impl Into<String>,
        file: Option<FileId>,
    ) -> DiagnosticHandle {
        DiagnosticHandle {
            anchor: self.anchor(SnapshotHandleKind::Diagnostic),
            code: code.into(),
            file,
        }
    }

    #[must_use]
    pub(crate) fn source_span_handle(&self, file: FileId, range: TextRange) -> SourceSpanHandle {
        SourceSpanHandle {
            anchor: self.anchor(SnapshotHandleKind::SourceSpan),
            file,
            range,
        }
    }

    pub(crate) fn resolve_symbol_handle(
        &self,
        handle: &SymbolHandle,
    ) -> Result<WorkspaceSymbol, AnalysisError> {
        self.ensure_handle_current(&handle.anchor, SnapshotHandleKind::Symbol)?;
        Ok(handle.symbol.clone())
    }

    pub(crate) fn resolve_type_handle<'a>(
        &self,
        handle: &'a TypeHandle,
    ) -> Result<&'a str, AnalysisError> {
        self.ensure_handle_current(&handle.anchor, SnapshotHandleKind::Type)?;
        Ok(&handle.display)
    }

    pub(crate) fn resolve_signature_handle<'a>(
        &self,
        handle: &'a SignatureHandle,
    ) -> Result<&'a str, AnalysisError> {
        self.ensure_handle_current(&handle.anchor, SnapshotHandleKind::Signature)?;
        Ok(&handle.label)
    }

    pub(crate) fn resolve_diagnostic_handle<'a>(
        &self,
        handle: &'a DiagnosticHandle,
    ) -> Result<(&'a str, Option<FileId>), AnalysisError> {
        self.ensure_handle_current(&handle.anchor, SnapshotHandleKind::Diagnostic)?;
        Ok((&handle.code, handle.file))
    }

    pub(crate) fn resolve_source_span_handle(
        &self,
        handle: &SourceSpanHandle,
    ) -> Result<Location, AnalysisError> {
        self.ensure_handle_current(&handle.anchor, SnapshotHandleKind::SourceSpan)?;
        Ok(Location {
            file: handle.file,
            range: Some(handle.range),
        })
    }

    fn anchor(&self, kind: SnapshotHandleKind) -> SnapshotHandleAnchor {
        SnapshotHandleAnchor {
            snapshot_id: self.workspace_snapshot_id(),
            revision: self.revision(),
            kind,
        }
    }

    fn ensure_handle_current(
        &self,
        anchor: &SnapshotHandleAnchor,
        expected_kind: SnapshotHandleKind,
    ) -> Result<(), AnalysisError> {
        if anchor.kind == expected_kind
            && anchor.snapshot_id == self.workspace_snapshot_id()
            && anchor.revision == self.revision()
        {
            return Ok(());
        }
        Err(AnalysisError::new(
            AnalysisErrorKind::StaleSnapshot,
            format!(
                "snapshot handle is stale: captured workspace {} graph/source {}:{}, current workspace {} graph/source {}:{}",
                anchor.snapshot_id.as_u64(),
                anchor.revision.graph.as_u64(),
                anchor.revision.source.as_u64(),
                self.workspace_snapshot_id().as_u64(),
                self.revision().graph.as_u64(),
                self.revision().source.as_u64()
            ),
        ))
    }
}
