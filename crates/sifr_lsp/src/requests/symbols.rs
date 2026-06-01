use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;
use sifr_analysis::{AnalysisQueryResult, SymbolQuery};

pub(crate) fn document_symbol(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, source| {
        snapshot
            .document_symbols(host, file)
            .map_err(|error| LspError::internal(error.message))?
            .into_value()
            .into_iter()
            .map(|symbol| conversion::document_symbol(symbol, source))
            .collect::<LspResult<Vec<_>>>()
            .map(Value::Array)
    })
}

pub(crate) fn workspace_symbol(session: &mut Session, params: Value) -> LspResult<Value> {
    let query = params
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let uri_map = session.store().uri_map();
    let mut symbols = Vec::new();
    for document in session.store_mut().documents_mut() {
        let mut document_symbols = document.with_host(|snapshot, host, _file, _source| {
            snapshot
                .workspace_symbols(
                    host,
                    &SymbolQuery {
                        query: query.to_string(),
                    },
                )
                .map_err(|error| LspError::internal(error.message))
                .map(AnalysisQueryResult::into_value)
        })?;
        symbols.append(&mut document_symbols);
    }
    Ok(Value::Array(
        symbols
            .into_iter()
            .filter_map(|symbol| {
                uri_map
                    .get(&symbol.file.as_u32())
                    .cloned()
                    .map(|uri| conversion::workspace_symbol(symbol, uri))
            })
            .collect(),
    ))
}
