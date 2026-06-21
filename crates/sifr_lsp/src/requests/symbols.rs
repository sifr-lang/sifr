use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::Value;
use sifr_analysis::SymbolQuery;
use std::collections::BTreeSet;

pub(crate) fn document_symbol(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position_encoding = session.position_encoding();
    session.with_document_analysis(&uri, |snapshot, host, file, source| {
        snapshot
            .document_symbols(host, file)
            .map_err(|error| LspError::internal(error.message))?
            .into_value()
            .into_iter()
            .map(|symbol| conversion::document_symbol(symbol, source, position_encoding))
            .collect::<LspResult<Vec<_>>>()
            .map(Value::Array)
    })
}

pub(crate) fn workspace_symbol(session: &mut Session, params: Value) -> LspResult<Value> {
    let query = params
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut seen = BTreeSet::new();
    let symbols = session.workspace_symbols(&SymbolQuery {
        query: query.to_string(),
    })?;
    Ok(Value::Array(
        symbols
            .into_iter()
            .filter(|mapped| {
                seen.insert((
                    mapped.symbol.file.as_u32(),
                    mapped.uri.clone(),
                    mapped.symbol.name.clone(),
                    mapped.symbol.kind.clone(),
                    mapped.symbol.container_name.clone(),
                ))
            })
            .map(|mapped| conversion::workspace_symbol(mapped.symbol, mapped.uri))
            .collect(),
    ))
}
