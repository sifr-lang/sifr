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
pub use render::{
    DiagnosticEnvelope, DiagnosticSpan, DiagnosticSpanLine, PresentationRenderError,
    RenderedDiagnostic, render_compact_diagnostics, render_compact_envelope,
    render_human_diagnostics, render_human_envelope, render_json_diagnostics, render_json_envelope,
    render_sink_compact, render_sink_human, render_sink_json,
};
pub use source_map::{SourceId, SourceMap, SourceMapError, SourceSpan};
