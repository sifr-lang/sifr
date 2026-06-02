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
    let file_maps = session.file_maps_for_uri(&uri)?;
    session.with_document_analysis(&uri, |snapshot, host, file, _source| {
        let actions = snapshot
            .code_actions(host, file, range, &context)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        actions
            .into_iter()
            .map(|action| {
                conversion::code_action(
                    action,
                    &uri,
                    |file| file_maps.uri_for(file),
                    |file| file_maps.source_for(file),
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
    let data_file = data
        .get("file")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| LspError::invalid_params("codeAction/resolve missing action file"))?;
    let uri = data
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| LspError::invalid_params("codeAction/resolve missing action uri"))?
        .to_string();
    let expected_version = data.get("expectedVersion").and_then(Value::as_i64);
    let file_maps = session.file_maps_for_uri(&uri)?;
    if let (Some(expected), Some(current)) =
        (expected_version, session.store().document(&uri)?.version())
    {
        if expected != i64::from(current) {
            return Err(LspError::invalid_params(format!(
                "stale code action for version {expected}; current version is {current}"
            )));
        }
    }
    let edit = session.with_document_analysis(&uri, |snapshot, host, file, _source| {
        if file.as_u32() != data_file {
            return Err(LspError::invalid_params(format!(
                "stale code action file {data_file}; current file is {}",
                file.as_u32()
            )));
        }
        let edit = snapshot
            .safe_fix_all_action(host, file)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        conversion::workspace_edit(
            edit,
            |file| file_maps.uri_for(file),
            |file| file_maps.source_for(file),
        )
    })?;
    params["edit"] = edit;
    params["data"]["sifrResolved"] = Value::Bool(true);
    Ok(params)
}
