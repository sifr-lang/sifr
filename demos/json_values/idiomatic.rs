#[derive(Debug, Clone)]
enum JsonValue {
    Int(i64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
    Parsed(serde_json::Value),
}

impl JsonValue {
    fn get(&self, key: &str) -> Option<JsonValue> {
        match self {
            JsonValue::Object(entries) => entries
                .iter()
                .find(|(entry_key, _)| entry_key == key)
                .map(|(_, value)| value.clone()),
            JsonValue::Parsed(value) => value.get(key).cloned().map(JsonValue::Parsed),
            _ => None,
        }
    }

    fn at(&self, index: usize) -> Option<JsonValue> {
        match self {
            JsonValue::Array(values) => values.get(index).cloned(),
            JsonValue::Parsed(value) => value.get(index).cloned().map(JsonValue::Parsed),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::Str(value) => Some(value),
            JsonValue::Parsed(value) => value.as_str(),
            _ => None,
        }
    }

    fn as_int(&self) -> Option<i64> {
        match self {
            JsonValue::Int(value) => Some(*value),
            JsonValue::Parsed(value) => value.as_i64(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct JsonDecodeError(String);

impl std::fmt::Display for JsonDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for JsonDecodeError {}

fn from_int(value: i64) -> JsonValue {
    JsonValue::Int(value)
}

fn from_str(value: &str) -> JsonValue {
    JsonValue::Str(value.to_string())
}

fn from_array(values: Vec<JsonValue>) -> JsonValue {
    JsonValue::Array(values)
}

fn from_object(values: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        values
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn json_string(text: &str) -> String {
    serde_json::Value::String(text.to_string()).to_string()
}

fn dumps(value: &JsonValue) -> String {
    match value {
        JsonValue::Int(number) => number.to_string(),
        JsonValue::Str(text) => json_string(text),
        JsonValue::Array(values) => format!(
            "[{}]",
            values.iter().map(dumps).collect::<Vec<_>>().join(",")
        ),
        JsonValue::Object(entries) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|(key, value)| format!("{}:{}", json_string(key), dumps(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        JsonValue::Parsed(value) => value.to_string(),
    }
}

fn loads(value: &str) -> Result<JsonValue, JsonDecodeError> {
    serde_json::from_str(value)
        .map(JsonValue::Parsed)
        .map_err(|error| JsonDecodeError(error.to_string()))
}

fn main() {
    let wrapper_payload = from_object(vec![
        ("module", from_str("json")),
        (
            "items",
            from_array(vec![from_int(1), from_int(2), from_int(3)]),
        ),
    ]);

    let encoded = dumps(&wrapper_payload);
    assert_eq!(encoded, "{\"module\":\"json\",\"items\":[1,2,3]}");

    let mut decoded_ok = false;
    if let Ok(decoded) = loads(&encoded) {
        let module_value = decoded.get("module");
        let items_value = decoded.get("items");
        decoded_ok = module_value.is_some() && items_value.is_some();
        if let Some(module_value) = module_value {
            assert_eq!(module_value.as_str(), Some("json"));
        }
        if let Some(items_value) = items_value {
            let second = items_value.at(1);
            assert!(second.is_some());
            if let Some(second) = second {
                assert_eq!(second.as_int(), Some(2));
            }
        }
    }
    assert!(decoded_ok);
}
