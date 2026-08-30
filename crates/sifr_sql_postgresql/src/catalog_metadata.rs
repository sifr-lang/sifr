use crate::catalog::{
    CatalogFunction, CatalogOperator, database_type_property, optional_bool_property, schema_error,
    text_property,
};
use crate::diagnostic::PostgresDiagnostic;
use sifr_sql_contract::{SchemaObject, SemanticValue};
use std::collections::BTreeSet;

pub(crate) fn string_set_property(object: &SchemaObject, key: &str) -> Option<BTreeSet<String>> {
    match object.semantic.get(key) {
        Some(SemanticValue::Set(values)) => values
            .iter()
            .map(|value| match value {
                SemanticValue::Text(value) => Some(value.clone()),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

pub(crate) fn nested_string_set_property(
    object: &SchemaObject,
    key: &str,
) -> Option<BTreeSet<Vec<String>>> {
    match object.semantic.get(key) {
        Some(SemanticValue::List(values)) => values
            .iter()
            .map(|value| match value {
                SemanticValue::List(values) => values
                    .iter()
                    .map(|value| match value {
                        SemanticValue::Text(value) => Some(value.clone()),
                        _ => None,
                    })
                    .collect(),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

pub(crate) fn function_from_object(
    object: &SchemaObject,
) -> Result<CatalogFunction, PostgresDiagnostic> {
    let arguments = match object.semantic.get("arguments") {
        Some(SemanticValue::List(values)) => values
            .iter()
            .map(|value| match value {
                SemanticValue::Text(value) => serde_json::from_str(value)
                    .map_err(|_| schema_error(object, "invalid function argument type")),
                _ => Err(schema_error(object, "invalid function argument metadata")),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(schema_error(object, "function has no argument list")),
    };
    Ok(CatalogFunction {
        identity: object.identity.clone(),
        arguments,
        result: database_type_property(object, "result")?,
        strict: optional_bool_property(object, "strict").unwrap_or(false),
        aggregate: optional_bool_property(object, "aggregate").unwrap_or(false),
        result_nullable: optional_bool_property(object, "result-nullable").unwrap_or(true),
    })
}

pub(crate) fn operator_from_object(
    object: &SchemaObject,
) -> Result<CatalogOperator, PostgresDiagnostic> {
    Ok(CatalogOperator {
        identity: object.identity.clone(),
        name: text_property(object, "name")?.to_string(),
        left: database_type_property(object, "left")?,
        right: database_type_property(object, "right")?,
        result: database_type_property(object, "result")?,
    })
}
