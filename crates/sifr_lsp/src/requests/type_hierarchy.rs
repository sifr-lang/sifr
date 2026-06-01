use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::{position, text_document_uri};
use crate::session::Session;
use serde_json::Value;
use sifr_analysis::{AnalysisQueryResult, TypeHierarchyItemId};

pub(crate) fn prepare(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = position(&params)?;
    let uri_map = session.store().uri_map();
    let document = session.store_mut().document_mut(&uri)?;
    document.with_host(|snapshot, host, file, source| {
        let item = snapshot
            .prepare_type_hierarchy(host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        let Some(item) = item else {
            return Ok(Value::Null);
        };
        let uri = uri_map
            .get(&item.location.file.as_u32())
            .cloned()
            .ok_or_else(|| LspError::internal("type hierarchy item references unopened file"))?;
        conversion::type_hierarchy_item(item, uri, source)
    })
}

pub(crate) fn supertypes(session: &mut Session, params: Value) -> LspResult<Value> {
    hierarchy(session, params, true)
}

pub(crate) fn subtypes(session: &mut Session, params: Value) -> LspResult<Value> {
    hierarchy(session, params, false)
}

fn hierarchy(session: &mut Session, params: Value, supertypes: bool) -> LspResult<Value> {
    let id = params
        .pointer("/item/data")
        .and_then(Value::as_str)
        .ok_or_else(|| LspError::invalid_params("typeHierarchy request requires item data"))?;
    for document in session.store_mut().documents_mut() {
        let items = document.with_host(|snapshot, host, _file, _source| {
            if supertypes {
                snapshot.type_hierarchy_supertypes(host, TypeHierarchyItemId(id.to_string()))
            } else {
                snapshot.type_hierarchy_subtypes(host, TypeHierarchyItemId(id.to_string()))
            }
            .map_err(|error| LspError::internal(error.message))
            .map(AnalysisQueryResult::into_value)
        })?;
        if items.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
    }
    Ok(Value::Array(Vec::new()))
}
