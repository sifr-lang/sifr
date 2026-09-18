use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;

pub(crate) fn selection_range(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let source = session.store().document(&uri)?.text().to_string();
    let position_encoding = session.position_encoding();
    let positions = params
        .get("positions")
        .and_then(Value::as_array)
        .ok_or_else(|| LspError::invalid_params("selectionRange requires positions"))?
        .iter()
        .map(|position| conversion::lsp_position_to_utf8(position, &source, position_encoding))
        .collect::<LspResult<Vec<_>>>()?;
    let source = session.store().document(&uri)?.text();
    sifr_analysis::syntax_queries::selection_ranges(source, &positions)
        .map_err(|diagnostics| {
            LspError::internal(
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?
        .into_iter()
        .map(|range| conversion::selection_range(range, source, position_encoding))
        .collect::<LspResult<Vec<_>>>()
        .map(Value::Array)
}
