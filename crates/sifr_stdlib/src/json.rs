use serde_json::{Map, Number, Value};
use sifr_runtime::SifrInt;
use sifr_runtime::json::{
    DEFAULT_JSON_INTEGER_DIGIT_LIMIT, JsonIntegerProfile, JsonIntegerRangeError, JsonLimitError,
    encode_integer_for_profile, validate_json_integer_digit_limits,
};
use std::fmt;

const ROOT_PATH: &str = "$";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonDecodeBridgeError {
    message: String,
    line: usize,
    column: usize,
}

impl JsonDecodeBridgeError {
    #[must_use]
    pub fn new(message: String, line: usize, column: usize) -> Self {
        Self {
            message,
            line,
            column,
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    #[must_use]
    pub const fn column(&self) -> usize {
        self.column
    }
}

impl fmt::Display for JsonDecodeBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for JsonDecodeBridgeError {}

pub fn json_load_tokens(input: &str) -> Result<Vec<String>, JsonDecodeBridgeError> {
    validate_json_integer_digit_limits(input, DEFAULT_JSON_INTEGER_DIGIT_LIMIT)
        .map_err(json_limit_as_decode_error)?;
    let parsed = serde_json::from_str::<Value>(input).map_err(json_decode_error)?;
    value_to_tokens(&parsed)
}

pub fn json_validate_integer_digit_limits(input: &str) -> Result<(), JsonLimitError> {
    validate_json_integer_digit_limits(input, DEFAULT_JSON_INTEGER_DIGIT_LIMIT)
}

pub fn json_dump_tokens(tokens: &[String]) -> String {
    dump_tokens_with_profile(tokens, JsonIntegerProfile::Exact).unwrap_or_else(|_err| "null".into())
}

pub fn json_dump_tokens_exact(tokens: &[String]) -> String {
    json_dump_tokens(tokens)
}

pub fn json_dump_tokens_string_ints(tokens: &[String]) -> String {
    dump_tokens_with_profile(tokens, JsonIntegerProfile::StringInts)
        .unwrap_or_else(|_err| "null".into())
}

pub fn json_dump_tokens_web(tokens: &[String]) -> Result<String, JsonIntegerRangeError> {
    dump_tokens_with_profile(tokens, JsonIntegerProfile::Web)
}

fn json_decode_error(error: serde_json::Error) -> JsonDecodeBridgeError {
    JsonDecodeBridgeError::new(error.to_string(), error.line(), error.column())
}

fn json_limit_as_decode_error(error: JsonLimitError) -> JsonDecodeBridgeError {
    JsonDecodeBridgeError::new(error.message().to_string(), 0, 0)
}

fn value_to_tokens(value: &Value) -> Result<Vec<String>, JsonDecodeBridgeError> {
    let mut tokens = Vec::new();
    append_value_tokens(value, &mut tokens)?;
    Ok(tokens)
}

fn append_value_tokens(
    value: &Value,
    tokens: &mut Vec<String>,
) -> Result<(), JsonDecodeBridgeError> {
    match value {
        Value::Null => tokens.push("null".into()),
        Value::Bool(value) => {
            tokens.push("bool".into());
            tokens.push(value.to_string());
        }
        Value::Number(number) => append_number_tokens(number, tokens)?,
        Value::String(value) => {
            tokens.push("str".into());
            tokens.push(value.clone());
        }
        Value::Array(items) => {
            tokens.push("array".into());
            tokens.push(items.len().to_string());
            for item in items {
                append_value_tokens(item, tokens)?;
            }
        }
        Value::Object(entries) => {
            tokens.push("object".into());
            tokens.push(entries.len().to_string());
            for (key, item) in entries {
                tokens.push(key.clone());
                append_value_tokens(item, tokens)?;
            }
        }
    }
    Ok(())
}

fn append_number_tokens(
    number: &Number,
    tokens: &mut Vec<String>,
) -> Result<(), JsonDecodeBridgeError> {
    if let Some(value) = number.as_i64() {
        tokens.push("int".into());
        tokens.push(value.to_string());
        return Ok(());
    }
    if number.is_u64() {
        return Err(JsonDecodeBridgeError::new(
            "json integer out of range for sifr int".into(),
            0,
            0,
        ));
    }
    if let Some(value) = number.as_f64() {
        tokens.push("float".into());
        tokens.push(value.to_string());
        return Ok(());
    }
    Err(JsonDecodeBridgeError::new(
        "unsupported json number representation".into(),
        0,
        0,
    ))
}

fn dump_tokens_with_profile(
    tokens: &[String],
    profile: JsonIntegerProfile,
) -> Result<String, JsonIntegerRangeError> {
    let mut cursor = TokenCursor::new(tokens);
    let value = cursor.value(ROOT_PATH, profile)?;
    if cursor.is_finished() {
        return Ok(match serde_json::to_string(&value) {
            Ok(serialized) => serialized,
            Err(_error) => "null".into(),
        });
    }
    Ok("null".into())
}

struct TokenCursor {
    tokens: Vec<String>,
    index: usize,
}

impl TokenCursor {
    fn new(tokens: &[String]) -> Self {
        Self {
            tokens: tokens.to_vec(),
            index: 0,
        }
    }

    fn is_finished(&self) -> bool {
        self.index == self.tokens.len()
    }

    fn value(
        &mut self,
        path: &str,
        profile: JsonIntegerProfile,
    ) -> Result<Value, JsonIntegerRangeError> {
        let Some(tag) = self.next() else {
            return Ok(Value::Null);
        };
        match tag.as_str() {
            "null" => Ok(Value::Null),
            "bool" => Ok(Value::Bool(
                self.next().is_some_and(|value| value == "true"),
            )),
            "int" => self.integer(path, profile),
            "float" => Ok(self.float()),
            "str" => Ok(Value::String(self.next().unwrap_or_default())),
            "array" => self.array(path, profile),
            "object" => self.object(path, profile),
            _ => Ok(Value::Null),
        }
    }

    fn integer(
        &mut self,
        path: &str,
        profile: JsonIntegerProfile,
    ) -> Result<Value, JsonIntegerRangeError> {
        let token = self.next().unwrap_or_else(|| "0".into());
        let int_value =
            SifrInt::parse_decimal(&token, DEFAULT_JSON_INTEGER_DIGIT_LIMIT).map_err(|error| {
                JsonIntegerRangeError::new(error.to_string(), path.into(), profile.as_str())
            })?;
        let encoded = encode_integer_for_profile(&int_value, profile, path)?;
        if encoded.is_decimal_string() {
            return Ok(Value::String(encoded.decimal_text().into()));
        }
        Ok(match token.parse::<i64>() {
            Ok(value) => Value::Number(Number::from(value)),
            Err(_error) => Value::Null,
        })
    }

    fn float(&mut self) -> Value {
        let token = self.next().unwrap_or_else(|| "0.0".into());
        let Ok(value) = token.parse::<f64>() else {
            return Value::Null;
        };
        match Number::from_f64(value) {
            Some(number) => Value::Number(number),
            None => Value::Null,
        }
    }

    fn array(
        &mut self,
        path: &str,
        profile: JsonIntegerProfile,
    ) -> Result<Value, JsonIntegerRangeError> {
        let len = self.next_usize();
        let mut items = Vec::with_capacity(len);
        for index in 0..len {
            let child_path = format!("{path}[{index}]");
            items.push(self.value(&child_path, profile)?);
        }
        Ok(Value::Array(items))
    }

    fn object(
        &mut self,
        path: &str,
        profile: JsonIntegerProfile,
    ) -> Result<Value, JsonIntegerRangeError> {
        let len = self.next_usize();
        let mut entries = Map::new();
        for _ in 0..len {
            let key = self.next().unwrap_or_default();
            let child_path = format!("{path}.{key}");
            entries.insert(key, self.value(&child_path, profile)?);
        }
        Ok(Value::Object(entries))
    }

    fn next_usize(&mut self) -> usize {
        self.next()
            .and_then(|token| token.parse::<usize>().ok())
            .unwrap_or(0)
    }

    fn next(&mut self) -> Option<String> {
        let value = self.tokens.get(self.index)?.clone();
        self.index += 1;
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_tokens_preserves_decode_line_and_column() {
        let err = json_load_tokens("{invalid json")
            .expect_err("invalid JSON should preserve serde location");

        assert_eq!(err.line(), 1);
        assert_eq!(err.column(), 2);
        assert!(!err.message().is_empty());
    }

    #[test]
    fn load_tokens_shapes_structured_values() {
        let tokens = json_load_tokens(r#"{"name":"sifr","items":[1,true,null]}"#)
            .expect("valid JSON should tokenize");

        assert_eq!(
            tokens,
            vec![
                "object", "2", "name", "str", "sifr", "items", "array", "3", "int", "1", "bool",
                "true", "null",
            ]
        );
    }

    #[test]
    fn dump_tokens_escapes_strings_with_serde_json() {
        let payload = json_dump_tokens(&["str".into(), "a\"b".into()]);

        assert_eq!(payload, r#""a\"b""#);
    }

    #[test]
    fn dump_tokens_web_rejects_unsafe_integer_with_path() {
        let err = json_dump_tokens_web(&[
            "object".into(),
            "1".into(),
            "items".into(),
            "array".into(),
            "2".into(),
            "int".into(),
            "1".into(),
            "int".into(),
            "9007199254740992".into(),
        ])
        .expect_err("unsafe web integer should fail");

        assert_eq!(err.path(), "$.items[1]");
        assert_eq!(err.profile(), "json.web");
    }

    #[test]
    fn dump_tokens_string_ints_emits_integer_strings() {
        let payload = json_dump_tokens_string_ints(&[
            "array".into(),
            "2".into(),
            "int".into(),
            "1".into(),
            "int".into(),
            "2".into(),
        ]);

        assert_eq!(payload, r#"["1","2"]"#);
    }
}
