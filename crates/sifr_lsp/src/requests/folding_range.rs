use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;

pub(crate) fn folding_range(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position_encoding = session.position_encoding();
    let source = session.store().document(&uri)?.text();
    sifr_analysis::syntax_queries::folding_ranges(source)
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
        .map(|range| conversion::folding_range(range, source, position_encoding))
        .collect::<LspResult<Vec<_>>>()
        .map(Value::Array)
}
