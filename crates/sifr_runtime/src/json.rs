use crate::{DEFAULT_MAX_INTEGER_DIGITS, SifrInt};
use num_traits::ToPrimitive;
use std::fmt;

pub const DEFAULT_JSON_INTEGER_DIGIT_LIMIT: usize = DEFAULT_MAX_INTEGER_DIGITS;
pub const JS_SAFE_INTEGER_MIN: i64 = -9_007_199_254_740_991;
pub const JS_SAFE_INTEGER_MAX: i64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonIntegerProfile {
    Exact,
    Web,
    StringInts,
}

impl JsonIntegerProfile {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "json.exact",
            Self::Web => "json.web",
            Self::StringInts => "json.string_ints",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "json.exact" | "exact" => Some(Self::Exact),
            "json.web" | "web" => Some(Self::Web),
            "json.string_ints" | "string_ints" => Some(Self::StringInts),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonIntegerEncoding {
    Number(String),
    DecimalString(String),
}

impl JsonIntegerEncoding {
    #[must_use]
    pub fn decimal_text(&self) -> &str {
        match self {
            Self::Number(value) | Self::DecimalString(value) => value,
        }
    }

    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    #[must_use]
    pub const fn is_decimal_string(&self) -> bool {
        matches!(self, Self::DecimalString(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonIntegerRangeError {
    message: String,
    path: String,
    profile: &'static str,
}

impl JsonIntegerRangeError {
    #[must_use]
    pub fn new(message: String, path: String, profile: &'static str) -> Self {
        Self {
            message,
            path,
            profile,
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
    }
}

impl fmt::Display for JsonIntegerRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for JsonIntegerRangeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonLimitError {
    message: String,
    limit: usize,
}

impl JsonLimitError {
    #[must_use]
    pub fn new(message: String, limit: usize) -> Self {
        Self { message, limit }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }
}

impl fmt::Display for JsonLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for JsonLimitError {}

pub fn encode_integer_for_profile(
    value: &SifrInt,
    profile: JsonIntegerProfile,
    path: &str,
) -> Result<JsonIntegerEncoding, JsonIntegerRangeError> {
    let decimal = value.to_string();
    match profile {
        JsonIntegerProfile::Exact => Ok(JsonIntegerEncoding::Number(decimal)),
        JsonIntegerProfile::StringInts => Ok(JsonIntegerEncoding::DecimalString(decimal)),
        JsonIntegerProfile::Web => encode_web_integer(value, decimal, path),
    }
}

pub fn validate_integer_token_digit_limit(token: &str, limit: usize) -> Result<(), JsonLimitError> {
    let digit_count = json_integer_token_digit_count(token).ok_or_else(|| {
        JsonLimitError::new(
            format!("json integer token is not a decimal integer: {token}"),
            limit,
        )
    })?;
    if digit_count > limit {
        return Err(JsonLimitError::new(
            format!("json integer token has {digit_count} digits, exceeding limit {limit}"),
            limit,
        ));
    }
    Ok(())
}

pub fn validate_json_integer_digit_limits(input: &str, limit: usize) -> Result<(), JsonLimitError> {
    let bytes = input.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index = skip_json_string(bytes, index + 1);
            }
            b'-' | b'0'..=b'9' if is_json_number_start_context(bytes, index) => {
                let token_start = index;
                if bytes[index] == b'-' {
                    index += 1;
                }
                let digit_start = index;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                let digit_count = index.saturating_sub(digit_start);
                if digit_count == 0 {
                    index = token_start + 1;
                    continue;
                }
                let is_integer_token = index >= bytes.len()
                    || matches!(bytes[index], b',' | b']' | b'}')
                    || bytes[index].is_ascii_whitespace();
                if is_integer_token {
                    if digit_count > limit {
                        return Err(JsonLimitError::new(
                            format!(
                                "json integer token at byte {token_start} has {digit_count} digits, exceeding limit {limit}"
                            ),
                            limit,
                        ));
                    }
                } else {
                    index = skip_json_number_suffix(bytes, index);
                }
            }
            _ => {
                index += 1;
            }
        }
    }
    Ok(())
}

fn encode_web_integer(
    value: &SifrInt,
    decimal: String,
    path: &str,
) -> Result<JsonIntegerEncoding, JsonIntegerRangeError> {
    let Some(value) = value.as_bigint().to_i64() else {
        return Err(web_range_error(&decimal, path));
    };
    if (JS_SAFE_INTEGER_MIN..=JS_SAFE_INTEGER_MAX).contains(&value) {
        Ok(JsonIntegerEncoding::Number(decimal))
    } else {
        Err(web_range_error(&decimal, path))
    }
}

fn web_range_error(value: &str, path: &str) -> JsonIntegerRangeError {
    JsonIntegerRangeError::new(
        format!(
            "integer value {value} at {path} cannot be emitted as a JSON number under json.web"
        ),
        path.to_string(),
        JsonIntegerProfile::Web.as_str(),
    )
}

fn json_integer_token_digit_count(token: &str) -> Option<usize> {
    let unsigned = token
        .strip_prefix('-')
        .or_else(|| token.strip_prefix('+'))
        .unwrap_or(token);
    if unsigned.is_empty() || !unsigned.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some(unsigned.len())
}

fn skip_json_string(bytes: &[u8], mut index: usize) -> usize {
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return index + 1;
        }
        index += 1;
    }
    index
}

fn skip_json_number_suffix(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len()
        && (bytes[index].is_ascii_digit()
            || matches!(bytes[index], b'.' | b'e' | b'E' | b'+' | b'-'))
    {
        index += 1;
    }
    index
}

fn is_json_number_start_context(bytes: &[u8], index: usize) -> bool {
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        let byte = bytes[cursor];
        if byte.is_ascii_whitespace() {
            continue;
        }
        return matches!(byte, b'[' | b',' | b':');
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(value: &str) -> SifrInt {
        SifrInt::parse_decimal(value, DEFAULT_JSON_INTEGER_DIGIT_LIMIT)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    #[test]
    fn exact_profile_emits_large_integers_as_numbers() {
        let value = parse("100000000000000000000000000000000000000");
        let encoded = encode_integer_for_profile(&value, JsonIntegerProfile::Exact, "$.id")
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            encoded,
            JsonIntegerEncoding::Number("100000000000000000000000000000000000000".to_string())
        );
    }

    #[test]
    fn string_ints_profile_emits_decimal_strings() {
        let value = SifrInt::from_i64(42);
        let encoded = encode_integer_for_profile(&value, JsonIntegerProfile::StringInts, "$.id")
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            encoded,
            JsonIntegerEncoding::DecimalString("42".to_string())
        );
    }

    #[test]
    fn web_profile_accepts_javascript_safe_boundaries() {
        for value in [JS_SAFE_INTEGER_MIN, JS_SAFE_INTEGER_MAX] {
            let encoded = encode_integer_for_profile(
                &SifrInt::from_i64(value),
                JsonIntegerProfile::Web,
                "$.value",
            )
            .unwrap_or_else(|err| panic!("{err}"));

            assert_eq!(encoded, JsonIntegerEncoding::Number(value.to_string()));
        }
    }

    #[test]
    fn web_profile_rejects_javascript_unsafe_numbers() {
        for value in [JS_SAFE_INTEGER_MIN - 1, JS_SAFE_INTEGER_MAX + 1] {
            let err = encode_integer_for_profile(
                &SifrInt::from_i64(value),
                JsonIntegerProfile::Web,
                "$.value",
            )
            .expect_err("unsafe JSON web integer should fail");

            assert_eq!(err.path(), "$.value");
            assert_eq!(err.profile(), "json.web");
        }
    }

    #[test]
    fn web_profile_rejects_arbitrary_precision_numbers() {
        let value = parse("100000000000000000000000000000000000000");
        let err = encode_integer_for_profile(&value, JsonIntegerProfile::Web, "$.value")
            .expect_err("large exact int should not be emitted as json.web number");

        assert_eq!(err.path(), "$.value");
        assert_eq!(err.profile(), "json.web");
    }

    #[test]
    fn integer_token_limit_ignores_sign_and_enforces_digits() {
        validate_integer_token_digit_limit("-1234", 4).expect("sign should not count");
        let err = validate_integer_token_digit_limit("+12345", 4)
            .expect_err("five digits should exceed the configured limit");

        assert_eq!(err.limit(), 4);
    }

    #[test]
    fn integer_token_limit_rejects_non_integer_tokens() {
        let err = validate_integer_token_digit_limit("12.0", 4)
            .expect_err("non-integer token should be rejected");

        assert_eq!(err.limit(), 4);
    }

    #[test]
    fn json_digit_limit_rejects_integer_tokens_outside_strings() {
        let payload = format!(r#"{{"id":{},"quoted":"{}"}}"#, "9".repeat(5), "8".repeat(8));
        let err = validate_json_integer_digit_limits(&payload, 4)
            .expect_err("oversized JSON integer token should fail");

        assert_eq!(err.limit(), 4);
        assert!(err.message().contains("5 digits"));
    }

    #[test]
    fn json_digit_limit_ignores_string_digits_and_fractional_numbers() {
        let payload = r#"{"quoted":"123456","fraction":12345.25,"exp":12345e2}"#;

        validate_json_integer_digit_limits(payload, 4)
            .expect("only integer number tokens are limited in this surface");
    }

    #[test]
    fn json_digit_limit_checks_nested_array_numbers() {
        let payload = r#"{"items":[1,-2345]}"#;
        let err = validate_json_integer_digit_limits(payload, 3)
            .expect_err("nested oversized JSON integer token should fail");

        assert!(err.message().contains("4 digits"));
    }
}
