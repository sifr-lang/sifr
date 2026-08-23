use crate::diagnostics::document_diagnostics;
use crate::errors::LspResult;
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::{Value, json};

pub(crate) fn text_document_diagnostic(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    Ok(json!({
        "kind": "full",
        "items": document_diagnostics(session, &uri)?
    }))
}

pub(crate) fn workspace_diagnostic(session: &mut Session) -> LspResult<Value> {
    let mut items = Vec::new();
    for uri in session.document_uris() {
        let version = session.store().document(&uri)?.version();
        items.push(json!({
            "kind": "full",
            "uri": uri,
            "version": version,
            "items": document_diagnostics(session, &uri)?
        }));
    }
    Ok(json!({ "items": items }))
}
