use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;

pub(crate) fn inlay_hint(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let range = params.get("range").cloned();
    let source = session.store().document(&uri)?.text().to_string();
    let range = range
        .as_ref()
        .map(|range| conversion::lsp_range(range, &source))
        .transpose()?;
    session.with_document_analysis(&uri, |snapshot, host, file, _source| {
        let hints = snapshot
            .inlay_hints(host, file, range)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        Ok(Value::Array(
            hints.into_iter().map(conversion::inlay_hint).collect(),
        ))
    })
}
