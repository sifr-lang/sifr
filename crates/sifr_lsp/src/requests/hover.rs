use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::{document_position, text_document_uri};
use crate::session::Session;
use serde_json::Value;

pub(crate) fn hover(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = document_position(session, &uri, &params)?;
    let python = session.python_declaration_snapshot(&uri)?;
    let hover = session.with_document_analysis(&uri, |snapshot, host, file, _source| {
        Ok(snapshot
            .hover(host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value())
    })?;
    let Some(hover) = hover else {
        return Ok(Value::Null);
    };
    let symbol_name = hover.symbol_name.clone();
    let symbol_file = hover.symbol_file;
    let mut response = conversion::hover(hover);
    if let Some(symbol_file) = symbol_file {
        crate::python_declarations::enrich_hover(&mut response, &python, symbol_file, &symbol_name);
    }
    Ok(response)
}

pub(crate) fn signature_help(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = document_position(session, &uri, &params)?;
    session.with_document_analysis(&uri, |snapshot, host, file, _source| {
        let help = snapshot
            .signature_help(host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        Ok(help.map_or(Value::Null, conversion::signature_help))
    })
}
