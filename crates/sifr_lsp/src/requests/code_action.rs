use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::{code_action_context_diagnostics, text_document_uri};
use crate::session::Session;
use serde_json::Value;
use sifr_analysis::CodeActionContext;

pub(crate) fn code_action(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let source = session.store().document(&uri)?.text().to_string();
    let range = params
        .get("range")
        .ok_or_else(|| LspError::invalid_params("codeAction requires range"))
        .and_then(|range| conversion::lsp_range(range, &source))?;
    let context = CodeActionContext {
        diagnostics: code_action_context_diagnostics(&params),
    };
    let uri_map = session.store().uri_map();
    let source_map = session.store().source_map();
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|host, file, _source| {
        let actions = host
            .code_actions(file, range, &context)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        actions
            .into_iter()
            .map(|action| {
                conversion::code_action(
                    action,
                    |file| {
                        uri_map.get(&file.as_u32()).cloned().ok_or_else(|| {
                            LspError::internal(format!("unknown action file {}", file.as_u32()))
                        })
                    },
                    |file| {
                        source_map.get(&file.as_u32()).cloned().ok_or_else(|| {
                            LspError::internal(format!("unknown action source {}", file.as_u32()))
                        })
                    },
                )
            })
            .collect::<LspResult<Vec<_>>>()
            .map(Value::Array)
    })
}

pub(crate) fn resolve(session: &mut Session, mut params: Value) -> LspResult<Value> {
    let Some(data) = params.get("data") else {
        return Err(LspError::invalid_params(
            "codeAction/resolve requires Sifr action data",
        ));
    };
    if data
        .get("sifrResolved")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(params);
    }
    let action = data
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| LspError::invalid_params("codeAction/resolve missing Sifr action"))?;
    if action != "fixAllSafePolicy" {
        return Err(LspError::invalid_params(format!(
            "unknown Sifr deferred code action {action:?}"
        )));
    }
    let file = data
        .get("file")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| LspError::invalid_params("codeAction/resolve missing action file"))?;
    let expected_version = data.get("expectedVersion").and_then(Value::as_i64);
    let uri = session
        .store()
        .uri_map()
        .get(&file)
        .cloned()
        .ok_or_else(|| LspError::invalid_params(format!("unknown action file {file}")))?;
    let uri_map = session.store().uri_map();
    let source_map = session.store().source_map();
    let document = session.store_mut().document_mut(&uri)?;
    if let (Some(expected), Some(current)) = (expected_version, document.version()) {
        if expected != i64::from(current) {
            return Err(LspError::invalid_params(format!(
                "stale code action for version {expected}; current version is {current}"
            )));
        }
    }
    let edit = document.with_host(|host, file, _source| {
        let edit = host
            .safe_fix_all_action(file)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        conversion::workspace_edit(
            edit,
            |file| {
                uri_map.get(&file.as_u32()).cloned().ok_or_else(|| {
                    LspError::internal(format!("unknown action file {}", file.as_u32()))
                })
            },
            |file| {
                source_map.get(&file.as_u32()).cloned().ok_or_else(|| {
                    LspError::internal(format!("unknown action source {}", file.as_u32()))
                })
            },
        )
    })?;
    params["edit"] = edit;
    params["data"]["sifrResolved"] = Value::Bool(true);
    Ok(params)
}
