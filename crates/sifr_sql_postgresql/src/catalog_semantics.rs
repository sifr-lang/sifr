use crate::diagnostic::{PostgresDiagnostic, PostgresDiagnosticCode};
use sifr_sql_contract::{DatabaseType, ObjectId, SchemaObject, SemanticValue};

pub(crate) fn database_value(
    database_type: &DatabaseType,
) -> Result<SemanticValue, PostgresDiagnostic> {
    serde_json::to_string(database_type)
        .map(SemanticValue::Text)
        .map_err(|_| schema_error_message("cannot serialize PostgreSQL database type"))
}

pub(crate) fn database_type_property(
    object: &SchemaObject,
    key: &str,
) -> Result<DatabaseType, PostgresDiagnostic> {
    let value = text_property(object, key)?;
    serde_json::from_str(value).map_err(|_| schema_error(object, "invalid database type metadata"))
}

pub(crate) fn text_property<'a>(
    object: &'a SchemaObject,
    key: &str,
) -> Result<&'a str, PostgresDiagnostic> {
    match object.semantic.get(key) {
        Some(SemanticValue::Text(value)) => Ok(value),
        _ => Err(schema_error(
            object,
            format!("missing text property '{key}'"),
        )),
    }
}

pub(crate) fn bool_property(object: &SchemaObject, key: &str) -> Result<bool, PostgresDiagnostic> {
    optional_bool_property(object, key)
        .ok_or_else(|| schema_error(object, format!("missing boolean property '{key}'")))
}

pub(crate) fn optional_bool_property(object: &SchemaObject, key: &str) -> Option<bool> {
    match object.semantic.get(key) {
        Some(SemanticValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

pub(crate) fn object_id_list_property(
    object: &SchemaObject,
    key: &str,
) -> Result<Vec<ObjectId>, PostgresDiagnostic> {
    match object.semantic.get(key) {
        Some(SemanticValue::List(values)) => values
            .iter()
            .map(|value| match value {
                SemanticValue::Text(value) => Ok(ObjectId::new(value)),
                _ => Err(schema_error(object, "invalid object identity list")),
            })
            .collect(),
        _ => Err(schema_error(
            object,
            format!("missing list property '{key}'"),
        )),
    }
}

pub(crate) fn split_identity(identity: &ObjectId) -> Vec<String> {
    identity.as_str().split('.').map(str::to_string).collect()
}

pub(crate) fn last_segment(identity: &ObjectId) -> &str {
    identity
        .as_str()
        .rsplit('.')
        .next()
        .unwrap_or(identity.as_str())
}

pub(crate) fn string_list(values: &[String]) -> SemanticValue {
    SemanticValue::List(values.iter().cloned().map(SemanticValue::Text).collect())
}

pub(crate) fn unknown_relation(name: &str) -> PostgresDiagnostic {
    PostgresDiagnostic::at_sql(
        PostgresDiagnosticCode::UnknownRelation,
        format!("unknown PostgreSQL relation '{name}'"),
        0,
        1,
    )
}

pub(crate) fn schema_error(
    object: &SchemaObject,
    message: impl Into<String>,
) -> PostgresDiagnostic {
    let mut diagnostic = schema_error_message(format!("{}: {}", object.identity, message.into()));
    if let Some(source) = &object.source {
        diagnostic = diagnostic.with_schema_span(source.document.clone(), source.start, source.end);
    }
    diagnostic
}

pub(crate) fn schema_error_message(message: impl Into<String>) -> PostgresDiagnostic {
    PostgresDiagnostic::at_sql(PostgresDiagnosticCode::TypeMismatch, message, 0, 1)
}
