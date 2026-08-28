#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_diagnostics::{
    DiagnosticArg, DiagnosticSpan, DiagnosticSpanLine, RenderedDiagnostic, Severity,
    render_compact_diagnostics, render_human_diagnostics, render_json_diagnostics,
};
use sifr_fuzz::bounded_text;
use std::collections::BTreeMap;

fuzz_target!(|data: &[u8]| {
    let text = bounded_text(data, 4 * 1024);
    let mut args = BTreeMap::new();
    args.insert("input".to_string(), DiagnosticArg::String(text.clone()));
    let diagnostic = RenderedDiagnostic {
        code: "SIFR-TYPE-0001".to_string(),
        severity: match data.first().copied().unwrap_or_default() % 3 {
            0 => Severity::Error,
            1 => Severity::Warning,
            _ => Severity::Note,
        },
        message: text.clone(),
        message_template: "{input}".to_string(),
        args,
        url: "https://sifr.dev/diagnostics/SIFR-TYPE-0001".to_string(),
        spans: vec![DiagnosticSpan {
            file: Some("fuzz/input.sifr".to_string()),
            byte_start: 0,
            byte_end: u32::try_from(text.len()).unwrap_or(u32::MAX),
            line: Some(1),
            column: Some(1),
            end_line: Some(1),
            end_column: u32::try_from(text.chars().count().saturating_add(1)).ok(),
            is_primary: true,
            label: Some(text.clone()),
            lines: vec![DiagnosticSpanLine {
                text,
                highlight_start: 0,
                highlight_end: 1,
            }],
        }],
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    };
    let diagnostics = [diagnostic];
    let _ = render_json_diagnostics(&diagnostics);
    let _ = render_human_diagnostics(&diagnostics);
    let _ = render_compact_diagnostics(&diagnostics);
});
