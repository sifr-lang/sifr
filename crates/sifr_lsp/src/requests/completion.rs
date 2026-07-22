use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::{document_position, text_document_uri};
use crate::session::Session;
use serde_json::{json, Value};

pub(crate) fn completion(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = document_position(session, &uri, &params)?;
    let python = session.python_declaration_snapshot(&uri)?;
    let mut response = session.with_document_analysis(&uri, |snapshot, host, file, _source| {
        let result = snapshot
            .completion(host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        Ok(json!({
            "isIncomplete": false,
            "items": result.items.into_iter().map(conversion::completion_item).collect::<Vec<_>>()
        }))
    })?;
    if let Some(items) = response.get_mut("items").and_then(Value::as_array_mut) {
        for item in items {
            crate::python_declarations::enrich_completion_item(item, &python);
        }
    }
    Ok(response)
}

pub(crate) fn resolve(mut params: Value) -> LspResult<Value> {
    let label = params
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("completion item")
        .to_string();
    let kind = params
        .pointer("/data/sifrKind")
        .and_then(Value::as_str)
        .unwrap_or("symbol")
        .to_string();
    if params.get("detail").is_none() {
        params["detail"] = Value::String(format!("Sifr {kind}"));
    }
    if params.get("documentation").is_none() {
        params["documentation"] = serde_json::json!({
            "kind": "markdown",
            "value": format!("Resolved Sifr completion for `{label}`.")
        });
    }
    Ok(params)
}
