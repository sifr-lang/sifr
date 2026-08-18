use serde_json::Value as JsonValue;
use std::fmt;

#[derive(Debug, Clone)]
struct JsonDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl fmt::Display for JsonDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for JsonDecodeError {}

fn loads(text: &str) -> Result<JsonValue, JsonDecodeError> {
    serde_json::from_str(text).map_err(|error| JsonDecodeError {
        message: error.to_string(),
        line: error.line() as i64,
        column: error.column() as i64,
    })
}

fn dumps(value: impl Into<JsonValue>) -> String {
    value.into().to_string()
}

fn collect_positive_actual() -> Vec<bool> {
    let parsed_obj = loads("{\"name\":\"sifr\"}");
    let parsed_arr = loads("[1,2,3]");
    let parsed_roundtrip = loads(&dumps(7));

    vec![
        parsed_obj
            .as_ref()
            .is_ok_and(|value| value.to_string() == "{\"name\":\"sifr\"}"),
        parsed_arr
            .as_ref()
            .is_ok_and(|value| value.to_string() == "[1,2,3]"),
        parsed_roundtrip
            .as_ref()
            .is_ok_and(|value| value.to_string() == "7"),
        dumps("hello") == "\"hello\"",
        dumps(false) == "false",
    ]
}

fn collect_negative_actual() -> Vec<bool> {
    vec![loads("{")
        .err()
        .is_some_and(|error| !error.message.is_empty() && error.line >= 1 && error.column >= 1)]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_positive_actual());
    actual.extend(collect_negative_actual());

    let expected = vec![true, true, true, true, true, true];
    assert_eq!(actual, expected);
    println!("json json parity demo: pass");
}
