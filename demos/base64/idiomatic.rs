use base64::engine::general_purpose::{STANDARD, URL_SAFE};
use base64::Engine;

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

fn assert_vector_eq(actual: &[String], expected: &[String]) {
    assert_eq!(actual, expected);
}

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn decode_to_utf8(bytes: Vec<u8>) -> Result<String, ParseError> {
    String::from_utf8(bytes).map_err(|error| ParseError::new(error.to_string()))
}

fn b64encode(payload: &str) -> String {
    STANDARD.encode(payload)
}

fn b64decode(payload: &str) -> Result<String, ParseError> {
    STANDARD
        .decode(payload)
        .map_err(|error| ParseError::new(error.to_string()))
        .and_then(decode_to_utf8)
}

fn standard_b64encode(payload: &str) -> String {
    b64encode(payload)
}

fn standard_b64decode(payload: &str) -> Result<String, ParseError> {
    b64decode(payload)
}

fn urlsafe_b64encode(payload: &str) -> String {
    URL_SAFE.encode(payload)
}

fn urlsafe_b64decode(payload: &str) -> Result<String, ParseError> {
    URL_SAFE
        .decode(payload)
        .map_err(|error| ParseError::new(error.to_string()))
        .and_then(decode_to_utf8)
}

fn b16encode(payload: &str) -> String {
    payload
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect()
}

fn b16decode(payload: &str) -> Result<String, ParseError> {
    if payload.len() % 2 != 0 {
        return Err(ParseError::new(
            "b16decode requires an even number of hexadecimal digits",
        ));
    }

    let bytes = (0..payload.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&payload[index..index + 2], 16)
                .map_err(|_| ParseError::new("invalid base16 digit"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    decode_to_utf8(bytes)
}

fn collect_positive_actual() -> Vec<String> {
    let urlsafe_encoded = urlsafe_b64encode("hello");
    let b16_encoded = b16encode("Hi");

    vec![
        b64encode("foo"),
        b64decode("Zm9v").unwrap_or_default(),
        standard_b64encode("foo"),
        standard_b64decode("Zm9v").unwrap_or_default(),
        urlsafe_encoded.clone(),
        urlsafe_b64decode(&urlsafe_encoded).unwrap_or_default(),
        b16_encoded.clone(),
        b16decode(&b16_encoded).unwrap_or_default(),
    ]
}

fn collect_decode_actual_ok(inputs: &[&str]) -> Vec<bool> {
    inputs
        .iter()
        .map(|payload| b64decode(payload).is_ok())
        .collect()
}

fn main() {
    let actual = collect_positive_actual();
    assert_vector_eq(
        &actual,
        &[
            "Zm9v".to_string(),
            "foo".to_string(),
            "Zm9v".to_string(),
            "foo".to_string(),
            "aGVsbG8=".to_string(),
            "hello".to_string(),
            "4869".to_string(),
            "Hi".to_string(),
        ],
    );

    assert_bool_vector_eq(
        &collect_decode_actual_ok(&["not base64!!!", "Zm9v"]),
        &[false, true],
    );

    println!("base64 base64 parity demo: pass");
}
