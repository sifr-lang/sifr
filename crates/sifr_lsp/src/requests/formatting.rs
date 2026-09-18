use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;
use sifr_analysis::FormatOptions;
use std::path::Path;

pub(crate) fn formatting(session: &mut Session, params: Value) -> LspResult<Value> {
    ensure_formatting_enabled(session)?;
    let uri = text_document_uri(&params)?;
    let path = session.store().document(&uri)?.path().to_path_buf();
    let options = format_options(&params, &path)?;
    let position_encoding = session.position_encoding();
    let source = session.store().document(&uri)?.text();
    let formatted = sifr_analysis::syntax_queries::format_source(source, Some(&path), options)
        .map_err(|diagnostics| LspError::internal(formatter_diagnostic_message(&diagnostics)))?;
    if formatted.formatted == source {
        return Ok(serde_json::json!([]));
    }
    let length =
        u32::try_from(source.len()).map_err(|_| LspError::invalid_params("source too large"))?;
    conversion::text_edits(
        vec![sifr_analysis::TextEdit {
            range: ruff_text_size::TextRange::up_to(ruff_text_size::TextSize::new(length)),
            replacement: formatted.formatted,
        }],
        source,
        position_encoding,
    )
}

pub(crate) fn range_formatting(session: &mut Session, params: Value) -> LspResult<Value> {
    ensure_formatting_enabled(session)?;
    let uri = text_document_uri(&params)?;
    let document = session.store().document(&uri)?;
    let source = document.text().to_string();
    let path = document.path().to_path_buf();
    let position_encoding = session.position_encoding();
    let range = params
        .get("range")
        .ok_or_else(|| LspError::invalid_params("rangeFormatting requires range"))
        .and_then(|range| conversion::lsp_range(range, &source, position_encoding))?;
    let options = format_options(&params, &path)?;
    let edits = sifr_analysis::syntax_queries::format_range(&source, range, Some(&path), options)
        .map_err(|diagnostics| {
        LspError::internal(formatter_diagnostic_message(&diagnostics))
    })?;
    conversion::text_edits(edits, &source, position_encoding)
}

fn ensure_formatting_enabled(session: &Session) -> LspResult<()> {
    if session.store().settings().format_enable {
        return Ok(());
    }
    Err(LspError::method_not_found(
        "Sifr formatting is disabled by sifr.format.enable",
    ))
}

fn format_options(params: &Value, path: &Path) -> LspResult<FormatOptions> {
    let mut provider = sifr_analysis::DiskSourceProvider::new();
    let mut options = sifr_analysis::format_options_for_path(path, &mut provider)
        .map_err(|diagnostics| LspError::internal(formatter_diagnostic_message(&diagnostics)))?;
    options.final_newline = params
        .pointer("/options/insertFinalNewline")
        .and_then(Value::as_bool)
        .unwrap_or(options.final_newline);
    if let Some(line_length) = params
        .pointer("/options/lineLength")
        .or_else(|| params.pointer("/options/sifr/lineLength"))
        .and_then(Value::as_u64)
    {
        options.line_length = u16::try_from(line_length)
            .map_err(|_| LspError::invalid_params("formatting lineLength is out of range"))?;
    }
    options.preview = params
        .pointer("/options/preview")
        .or_else(|| params.pointer("/options/sifr/preview"))
        .and_then(Value::as_bool)
        .unwrap_or(options.preview);
    Ok(options)
}

fn formatter_diagnostic_message(diagnostics: &[sifr_diagnostics::RenderedDiagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}
