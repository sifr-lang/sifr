use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;

pub(crate) fn full(session: &mut Session, params: Value) -> LspResult<Value> {
    tokens(session, params, None)
}

pub(crate) fn range(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let source = session.store().document(&uri)?.text().to_string();
    let range = params
        .get("range")
        .ok_or_else(|| LspError::invalid_params("semanticTokens/range requires range"))
        .and_then(|range| conversion::lsp_range(range, &source))?;
    tokens(session, params, Some(range))
}

fn tokens(
    session: &mut Session,
    params: Value,
    range: Option<ruff_text_size::TextRange>,
) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    session.with_document_analysis(&uri, |snapshot, host, file, source| {
        let tokens = snapshot
            .semantic_tokens(host, file, range)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        conversion::semantic_tokens(tokens, source)
    })
}
