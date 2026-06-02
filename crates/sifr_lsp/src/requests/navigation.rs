use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::{position, text_document_uri};
use crate::session::Session;
use serde_json::{json, Value};
use sifr_analysis::SymbolName;

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
    session.with_document_analysis(&uri, |snapshot, host, file, source| {
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
    session.with_document_analysis(&uri, |snapshot, host, file, _source| {
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
    let file_maps = session.file_maps_for_uri(&uri)?;
    session.with_document_analysis(&uri, |snapshot, host, file, _source| {
        let edit = snapshot
            .rename(host, file, &position, &SymbolName(new_name.to_string()))
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        conversion::workspace_edit(
            edit,
            |file| file_maps.uri_for(file),
            |file| file_maps.source_for(file),
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
    let file_maps = session.file_maps_for_uri(&uri)?;
    session.with_document_analysis(&uri, |snapshot, host, file, source| {
        operation(snapshot, host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value()
            .into_iter()
            .map(|location| {
                let location_source = file_maps
                    .source_for(location.file)
                    .unwrap_or_else(|_| source.to_string());
                conversion::location(&location, |file| file_maps.uri_for(file), &location_source)
            })
            .collect::<LspResult<Vec<_>>>()
            .map(Value::Array)
    })
}
