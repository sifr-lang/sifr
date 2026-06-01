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
    session.with_document_analysis(&uri, |snapshot, host, file, source| {
        let edits = snapshot
            .format_document(host, file, options)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        conversion::text_edits(edits, source)
    })
}

pub(crate) fn range_formatting(session: &mut Session, params: Value) -> LspResult<Value> {
    ensure_formatting_enabled(session)?;
    let uri = text_document_uri(&params)?;
    let document = session.store().document(&uri)?;
    let source = document.text().to_string();
    let path = document.path().to_path_buf();
    let range = params
        .get("range")
        .ok_or_else(|| LspError::invalid_params("rangeFormatting requires range"))
        .and_then(|range| conversion::lsp_range(range, &source))?;
    let options = format_options(&params, &path)?;
    session.with_document_analysis(&uri, |snapshot, host, file, source| {
        let edits = snapshot
            .format_range(host, file, range, options)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        conversion::text_edits(edits, source)
    })
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
    let mut options = sifr_analysis::format_options_for_path(path)
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
