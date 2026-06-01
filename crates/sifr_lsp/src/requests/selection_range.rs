use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;

pub(crate) fn selection_range(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let positions = params
        .get("positions")
        .and_then(Value::as_array)
        .ok_or_else(|| LspError::invalid_params("selectionRange requires positions"))?
        .iter()
        .map(conversion::lsp_position)
        .collect::<LspResult<Vec<_>>>()?;
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, source| {
        snapshot
            .selection_ranges(host, file, &positions)
            .map_err(|error| LspError::internal(error.message))?
            .into_value()
            .into_iter()
            .map(|range| conversion::selection_range(range, source))
            .collect::<LspResult<Vec<_>>>()
            .map(Value::Array)
    })
}
