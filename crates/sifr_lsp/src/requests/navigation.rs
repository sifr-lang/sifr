use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::{position, text_document_uri};
use crate::session::Session;
use serde_json::{json, Value};
use sifr_analysis::SymbolName;
use std::collections::BTreeMap;

pub(crate) fn definition(session: &mut Session, params: Value) -> LspResult<Value> {
    locations(session, params, |snapshot, host, file, position| {
        snapshot.definition(host, file, position)
    })
}

pub(crate) fn declaration(session: &mut Session, params: Value) -> LspResult<Value> {
    locations(session, params, |snapshot, host, file, position| {
        snapshot.declaration(host, file, position)
    })
}

pub(crate) fn type_definition(session: &mut Session, params: Value) -> LspResult<Value> {
    locations(session, params, |snapshot, host, file, position| {
        snapshot.type_definition(host, file, position)
    })
}

pub(crate) fn references(session: &mut Session, params: Value) -> LspResult<Value> {
    locations(session, params, |snapshot, host, file, position| {
        snapshot.references(host, file, position)
    })
}

pub(crate) fn document_highlight(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = position(&params)?;
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, source| {
        let highlights = snapshot
            .document_highlights(host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        highlights
            .into_iter()
            .map(|highlight| conversion::document_highlight(highlight, source))
            .collect::<LspResult<Vec<_>>>()
            .map(Value::Array)
    })
}

pub(crate) fn prepare_rename(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = position(&params)?;
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, _source| {
        let target = snapshot
            .prepare_rename(host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        let Some(target) = target else {
            return Ok(Value::Null);
        };
        Ok(json!({
            "range": {
                "start": { "line": position.line, "character": position.character },
                "end": {
                    "line": position.line,
                    "character": position
                        .character
                        .saturating_add(u32::try_from(target.symbol.name.len()).unwrap_or(0))
                }
            },
            "placeholder": target.symbol.name
        }))
    })
}

pub(crate) fn rename(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = position(&params)?;
    let new_name = params
        .get("newName")
        .and_then(Value::as_str)
        .ok_or_else(|| LspError::invalid_params("textDocument/rename requires newName"))?;
    let uri_map = session.store().uri_map();
    let source_map = session.store().source_map();
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, _source| {
        let edit = snapshot
            .rename(host, file, &position, &SymbolName(new_name.to_string()))
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        conversion::workspace_edit(
            edit,
            |file| uri_map.uri_for(file),
            |file| source_map.source_for(file),
        )
    })
}

fn locations(
    session: &mut Session,
    params: Value,
    operation: impl FnOnce(
        &sifr_analysis::AnalysisSnapshot,
        &mut sifr_analysis::AnalysisHost,
        sifr_analysis::FileId,
        &sifr_analysis::TextPosition,
    ) -> Result<
        sifr_analysis::AnalysisQueryResult<Vec<sifr_analysis::Location>>,
        sifr_analysis::AnalysisError,
    >,
) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = position(&params)?;
    let source_lookup = session.store().source_map();
    let uri_lookup = session.store().uri_map();
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, source| {
        operation(snapshot, host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value()
            .into_iter()
            .map(|location| {
                let source = source_lookup
                    .get(&location.file.as_u32())
                    .map(String::as_str)
                    .unwrap_or(source);
                conversion::location(&location, |file| uri_lookup.uri_for(file), source)
            })
            .collect::<LspResult<Vec<_>>>()
            .map(Value::Array)
    })
}

trait UriLookup {
    fn uri_for(&self, file: sifr_analysis::FileId) -> LspResult<String>;
}

impl UriLookup for BTreeMap<u32, String> {
    fn uri_for(&self, file: sifr_analysis::FileId) -> LspResult<String> {
        self.get(&file.as_u32())
            .cloned()
            .ok_or_else(|| LspError::internal(format!("unknown file {}", file.as_u32())))
    }
}

trait SourceLookup {
    fn source_for(&self, file: sifr_analysis::FileId) -> LspResult<String>;
}

impl SourceLookup for BTreeMap<u32, String> {
    fn source_for(&self, file: sifr_analysis::FileId) -> LspResult<String> {
        self.get(&file.as_u32())
            .cloned()
            .ok_or_else(|| LspError::internal(format!("unknown source {}", file.as_u32())))
    }
}
