pub fn validate_integer_digit_limit(input: &str, limit: usize) -> Result<(), String> {
    sifr_runtime::json::validate_json_integer_digit_limits(input, limit)
        .map_err(|error| error.to_string())
}

#[must_use]
pub const fn default_integer_digit_limit() -> usize {
    sifr_runtime::json::DEFAULT_JSON_INTEGER_DIGIT_LIMIT
}
