use crate::ast::{PostgresTypeName, SqlSpan};
use crate::raw_adapter::PostgresParseError;
use serde_json::{Map, Value};

pub(crate) fn relation_name(body: &Map<String, Value>) -> Vec<String> {
    [
        string_field(body, "catalogname"),
        string_field(body, "schemaname"),
    ]
    .into_iter()
    .flatten()
    .chain(string_field(body, "relname"))
    .map(str::to_string)
    .collect()
}

pub(crate) fn alias(body: &Map<String, Value>) -> Option<String> {
    optional_object_field(body, "alias")
        .and_then(|alias| string_field(alias, "aliasname"))
        .map(str::to_string)
}

pub(crate) fn type_name(body: &Map<String, Value>) -> PostgresTypeName {
    PostgresTypeName {
        path: name_list(body, "names"),
        modifiers: optional_array(body, "typmods")
            .iter()
            .filter_map(integer_constant)
            .collect(),
        array_dimensions: u8::try_from(optional_array(body, "arrayBounds").len())
            .unwrap_or(u8::MAX),
    }
}

pub(crate) fn integer_constant(value: &Value) -> Option<i64> {
    let object = value.as_object()?;
    let (tag, body) = tagged(object, "type modifier").ok()?;
    if tag != "A_Const" {
        return None;
    }
    if let Some(value) = body.get("ival").and_then(Value::as_object) {
        return value.get("ival").and_then(Value::as_i64);
    }
    let value = body.get("val").and_then(Value::as_object)?;
    let (tag, body) = tagged(value, "type modifier value").ok()?;
    (tag == "Integer")
        .then(|| body.get("ival").and_then(Value::as_i64))
        .flatten()
}

pub(crate) fn name_list(body: &Map<String, Value>, key: &str) -> Vec<String> {
    optional_array(body, key)
        .iter()
        .filter_map(string_node)
        .collect()
}

pub(crate) fn string_node(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    let (_, body) = tagged(object, "string node").ok()?;
    string_field(body, "sval")
        .or_else(|| string_field(body, "str"))
        .map(str::to_string)
}

pub(crate) fn tagged<'a>(
    object: &'a Map<String, Value>,
    label: &str,
) -> Result<(&'a str, &'a Map<String, Value>), PostgresParseError> {
    if object.len() != 1 {
        return Err(simple_error(format!("{label} is not a tagged parser node")));
    }
    let Some((name, value)) = object.iter().next() else {
        return Err(simple_error(format!("{label} is empty")));
    };
    let Some(body) = value.as_object() else {
        return Err(simple_error(format!("{label} body is not an object")));
    };
    Ok((name, body))
}

pub(crate) fn object<'a>(
    value: &'a Value,
    label: &str,
) -> Result<&'a Map<String, Value>, PostgresParseError> {
    value
        .as_object()
        .ok_or_else(|| simple_error(format!("{label} is not an object")))
}

pub(crate) fn object_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<&'a Map<String, Value>, PostgresParseError> {
    object
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| simple_error(format!("PostgreSQL parser node has no {key}")))
}

pub(crate) fn optional_object_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Option<&'a Map<String, Value>> {
    object.get(key).and_then(Value::as_object)
}

pub(crate) fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], PostgresParseError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| simple_error(format!("PostgreSQL parser result has no {key}")))
}

pub(crate) fn optional_array<'a>(object: &'a Map<String, Value>, key: &str) -> &'a [Value] {
    object
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

pub(crate) fn string_field<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

pub(crate) fn bool_field(object: &Map<String, Value>, key: &str) -> Option<bool> {
    object.get(key).and_then(Value::as_bool)
}

pub(crate) fn u32_field(object: &Map<String, Value>, key: &str) -> Option<u32> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

pub(crate) fn simple_error(message: impl Into<String>) -> PostgresParseError {
    PostgresParseError::unsupported(message, SqlSpan::default())
}
