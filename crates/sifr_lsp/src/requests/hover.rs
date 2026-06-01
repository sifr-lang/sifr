use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::{position, text_document_uri};
use crate::session::Session;
use serde_json::Value;

pub(crate) fn hover(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = position(&params)?;
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, _source| {
        let hover = snapshot
            .hover(host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        Ok(hover.map_or(Value::Null, conversion::hover))
    })
}

pub(crate) fn signature_help(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = position(&params)?;
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, _source| {
        let help = snapshot
            .signature_help(host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        Ok(help.map_or(Value::Null, conversion::signature_help))
    })
}
