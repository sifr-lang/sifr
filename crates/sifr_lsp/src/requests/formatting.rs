use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;
use sifr_analysis::FormatOptions;

pub(crate) fn formatting(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let options = format_options(&params);
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|host, file, source| {
        let edits = host
            .format_document(file, options)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        conversion::text_edits(edits, source)
    })
}

pub(crate) fn range_formatting(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let source = session.store().document(&uri)?.text().to_string();
    let range = params
        .get("range")
        .ok_or_else(|| LspError::invalid_params("rangeFormatting requires range"))
        .and_then(|range| conversion::lsp_range(range, &source))?;
    let options = format_options(&params);
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|host, file, source| {
        let edits = host
            .format_range(file, range, options)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        conversion::text_edits(edits, source)
    })
}

fn format_options(params: &Value) -> FormatOptions {
    let mut options = FormatOptions::default();
    options.final_newline = params
        .pointer("/options/insertFinalNewline")
        .and_then(Value::as_bool)
        .unwrap_or(options.final_newline);
    options
}
