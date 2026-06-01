use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;

pub(crate) fn folding_range(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, source| {
        snapshot
            .folding_ranges(host, file)
            .map_err(|error| LspError::internal(error.message))?
            .into_value()
            .into_iter()
            .map(|range| conversion::folding_range(range, source))
            .collect::<LspResult<Vec<_>>>()
            .map(Value::Array)
    })
}
