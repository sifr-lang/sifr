use crate::diagnostics::document_diagnostics;
use crate::errors::LspResult;
use crate::requests::text_document_uri;
use crate::session::Session;
use serde_json::{json, Value};

pub(crate) fn text_document_diagnostic(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let document = session.store_mut().document_mut(&uri)?;
    Ok(json!({
        "kind": "full",
        "items": document_diagnostics(document)?
    }))
}

pub(crate) fn workspace_diagnostic(session: &mut Session) -> LspResult<Value> {
    let mut items = Vec::new();
    for document in session.store_mut().documents_mut() {
        items.push(json!({
            "kind": "full",
            "uri": document.uri(),
            "version": document.version(),
            "items": document_diagnostics(document)?
        }));
    }
    Ok(json!({ "items": items }))
}
