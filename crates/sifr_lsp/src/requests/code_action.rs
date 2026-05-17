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

pub(crate) fn resolve(params: Value) -> LspResult<Value> {
    if params.get("data").is_none() {
        return Err(LspError::invalid_params(
            "codeAction/resolve requires Sifr action data",
        ));
    }
    Ok(params)
}
