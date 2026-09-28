#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_diagnostics::{
    DiagnosticCode, DiagnosticEnvelope, DiagnosticSpan, render_compact_envelope,
    render_human_envelope, render_json_envelope,
};

const MAX_ENVELOPE_BYTES: usize = 16 * 1024;
const MAX_HIGHLIGHT_COLUMN: u32 = 4096;

fn bounded_span(span: &DiagnosticSpan) -> bool {
    span.lines.iter().all(|line| {
        line.highlight_start <= MAX_HIGHLIGHT_COLUMN && line.highlight_end <= MAX_HIGHLIGHT_COLUMN
    })
}

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_ENVELOPE_BYTES {
        return;
    }
    let Ok(envelope) = serde_json::from_slice::<DiagnosticEnvelope>(data) else {
        return;
    };
    if envelope.version != 1
        || envelope.diagnostics.is_empty()
        || envelope.diagnostics.len() > 8
        || !envelope
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::TYPE_MISMATCH.code())
        || envelope.diagnostics.iter().any(|diagnostic| {
            diagnostic.spans.iter().any(|span| !bounded_span(span))
                || diagnostic.suggestions.iter().any(|suggestion| {
                    suggestion
                        .edits
                        .iter()
                        .any(|edit| !bounded_span(&edit.span))
                })
        })
    {
        return;
    }

    // The input is the serialized rendered envelope itself. Mutations can reach
    // spans, snippet highlights, children, suggestions and typed arguments.
    let json = render_json_envelope(&envelope).expect("a serializable envelope renders as JSON");
    let reparsed: DiagnosticEnvelope =
        serde_json::from_str(&json).expect("rendered envelope JSON deserializes");
    assert_eq!(reparsed, envelope);

    let human = render_human_envelope(&envelope);
    let compact = render_compact_envelope(&envelope);
    assert!(human.contains(DiagnosticCode::TYPE_MISMATCH.code()));
    assert!(compact.contains(DiagnosticCode::TYPE_MISMATCH.code()));
});
