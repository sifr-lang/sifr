use crate::{SifrInt, DEFAULT_MAX_INTEGER_DIGITS};
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
}
