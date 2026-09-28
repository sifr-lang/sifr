use crate::conversion;
use crate::errors::{LspError, LspResult};
use crate::requests::{document_position, text_document_uri};
use crate::session::Session;
use serde_json::Value;
use sifr_analysis::FileId;
use sifr_analysis::{AnalysisQueryResult, TypeHierarchyItemId};

pub(crate) fn prepare(session: &mut Session, params: Value) -> LspResult<Value> {
    let uri = text_document_uri(&params)?;
    let position = document_position(session, &uri, &params)?;
    let file_maps = session.file_maps_for_uri(&uri)?;
    let position_encoding = session.position_encoding();
    session.with_document_analysis(&uri, |snapshot, host, file, _source| {
        let item = snapshot
            .prepare_type_hierarchy(host, file, &position)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        let Some(item) = item else {
            return Ok(Value::Null);
        };
        let uri = file_maps.uri_for(item.location.file)?;
        let source = file_maps.source_for(item.location.file)?;
        conversion::type_hierarchy_item(item, uri, &source, position_encoding)
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
    let item_uri = params
        .pointer("/item/uri")
        .and_then(Value::as_str)
        .ok_or_else(|| LspError::invalid_params("typeHierarchy request requires item URI"))?;
    let file_number = id
        .split_once(':')
        .and_then(|(file, _)| file.parse::<u32>().ok())
        .ok_or_else(|| LspError::invalid_params("invalid typeHierarchy item data"))?;
    let position_encoding = session.position_encoding();
    for uri in session.document_uris() {
        let file_maps = session.file_maps_for_uri(&uri)?;
        if file_maps.uri_for(FileId::new(file_number)).ok().as_deref() != Some(item_uri) {
            continue;
        }
        let items = session.with_document_analysis(&uri, |snapshot, host, _file, _source| {
            if supertypes {
                snapshot.type_hierarchy_supertypes(host, TypeHierarchyItemId(id.to_string()))
            } else {
                snapshot.type_hierarchy_subtypes(host, TypeHierarchyItemId(id.to_string()))
            }
            .map_err(|error| LspError::internal(error.message))
            .map(AnalysisQueryResult::into_value)
        })?;
        return items
            .into_iter()
            .map(|item| {
                let uri = file_maps.uri_for(item.location.file)?;
                let source = file_maps.source_for(item.location.file)?;
                conversion::type_hierarchy_item(item, uri, &source, position_encoding)
            })
            .collect::<LspResult<Vec<_>>>()
            .map(Value::Array);
    }
    Ok(Value::Array(Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn relation_requests_return_items_and_follow_edits() {
        let temp = tempfile::tempdir().expect("temp directory");
        let path = temp.path().join("main.sifr");
        let source = "class lower:\n    pass\nclass Child(lower):\n    pass\nUPPER: int = 1\n";
        std::fs::write(&path, source).expect("source file");
        let uri = url::Url::from_file_path(&path).expect("URI").to_string();
        let mut session = Session::new();
        session
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                source.to_string(),
            )
            .expect("open document");
        let child = prepare(
            &mut session,
            json!({
                "textDocument": {"uri": uri}, "position": {"line": 2, "character": 8}
            }),
        )
        .expect("prepare child");
        assert_eq!(child["name"], "Child");
        let parents = supertypes(&mut session, json!({"item": child})).expect("supertype result");
        assert_eq!(parents.as_array().expect("array").len(), 1);
        assert_eq!(parents[0]["name"], "lower");
        let children = subtypes(&mut session, json!({"item": parents[0]})).expect("subtype result");
        assert_eq!(children[0]["name"], "Child");
        let value = prepare(
            &mut session,
            json!({
                "textDocument": {"uri": uri}, "position": {"line": 4, "character": 2}
            }),
        )
        .expect("prepare value");
        assert!(value.is_null());
        session
            .change_compacted(
                &uri,
                Some(2),
                &[json!({
                    "text": "class lower:\n    pass\nclass Child:\n    pass\nUPPER: int = 1\n"
                })],
            )
            .expect("remove inheritance");
        let former_children =
            subtypes(&mut session, json!({"item": parents[0]})).expect("updated subtype result");
        assert!(former_children.as_array().expect("array").is_empty());
    }
}
