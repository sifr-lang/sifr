use serde::Serialize;
use sifr_sql_contract::{SchemaContractError, SchemaContractErrorKind};

/// Serialize provider AST as semantic JSON. Source spans are diagnostic data and
/// never participate in schema identity or live/declarative parity.
pub fn canonical_postgres_ast_json<T: Serialize>(value: &T) -> Result<String, SchemaContractError> {
    let mut value = serde_json::to_value(value).map_err(|_| invalid_ast())?;
    remove_spans(&mut value);
    serde_json::to_string(&value).map_err(|_| invalid_ast())
}

fn remove_spans(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                remove_spans(value);
            }
        }
        serde_json::Value::Object(values) => {
            values.remove("span");
            for value in values.values_mut() {
                remove_spans(value);
            }
        }
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {}
    }
}

fn invalid_ast() -> SchemaContractError {
    SchemaContractError::new(
        SchemaContractErrorKind::InvalidProvider,
        "PostgreSQL semantic AST cannot be serialized",
    )
}
