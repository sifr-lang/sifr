#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

pub mod codes;
pub mod model;
pub mod render;
pub mod schema;
pub mod source_map;

pub use codes::DiagnosticCode;
pub use model::{
    ChildSeverity, DiagnosticArg, DiagnosticBuilder, DiagnosticChild, DiagnosticSink,
    DiagnosticSuggestion, ErrorEmitted, InternalDiagnostic, RelatedKind, RelatedSpan, Severity,
    SifrDiagnostic, SourceDiagnostic, SuggestionApplicability, SuggestionEdit,
};
pub use render::{DiagnosticEnvelope, DiagnosticSpan, DiagnosticSpanLine, RenderedDiagnostic};
pub use source_map::{SourceId, SourceMap, SourceMapError, SourceSpan};
