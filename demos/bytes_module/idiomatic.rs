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

fn encode_utf8(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

fn decode_utf8(bytes: &[u8]) -> Result<String, ParseError> {
    String::from_utf8(bytes.to_vec()).map_err(|error| ParseError::new(error.to_string()))
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bytes_from_hex(text: &str) -> Result<Vec<u8>, ParseError> {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    if compact.len() % 2 != 0 {
        return Err(ParseError::new(
            "fromhex() arg must contain an even number of hexadecimal digits",
        ));
    }

    (0..compact.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&compact[index..index + 2], 16)
                .map_err(|_| ParseError::new("invalid hex digit"))
        })
        .collect()
}

fn count_byte(payload: &[u8], needle: u8) -> usize {
    payload.iter().filter(|&&byte| byte == needle).count()
}

fn find_byte(payload: &[u8], needle: u8) -> Option<usize> {
    payload.iter().position(|&byte| byte == needle)
}

fn starts_with(payload: &[u8], prefix: &[u8]) -> bool {
    payload.starts_with(prefix)
}

fn ends_with(payload: &[u8], suffix: &[u8]) -> bool {
    payload.ends_with(suffix)
}

fn render_opt_int(value: Option<usize>) -> String {
    value
        .map(|number| number.to_string())
        .unwrap_or_else(|| "None".to_string())
}

fn collect_primary_actual(payload: &[u8]) -> Vec<String> {
    vec![
        count_byte(payload, 115).to_string(),
        render_opt_int(find_byte(payload, 45)),
        starts_with(payload, &encode_utf8("bytes")).to_string(),
        ends_with(payload, &encode_utf8("e30")).to_string(),
    ]
}

fn bytes_to_hex_or_empty(payload: &[u8]) -> String {
    bytes_to_hex(payload)
}

fn bytes_from_hex_to_text_or_empty(payload: &str) -> String {
    bytes_from_hex(payload)
        .and_then(|parsed| decode_utf8(&parsed))
        .unwrap_or_default()
}

fn collect_invalid_actual_ok() -> Vec<bool> {
    vec![bytes_from_hex("abc").is_ok(), decode_utf8(b"\xff").is_ok()]
}

fn main() {
    let payload = encode_utf8("bytes-bytes_module");
    let actual = collect_primary_actual(&payload);
    assert_vector_eq(
        &actual,
        &[
            "2".to_string(),
            "5".to_string(),
            "true".to_string(),
            "true".to_string(),
        ],
    );

    let hex_text = bytes_to_hex_or_empty(&encode_utf8("Hi"));
    assert!(!hex_text.is_empty());
    assert_eq!(hex_text, "4869");

    let roundtrip_text = bytes_from_hex_to_text_or_empty("48 69");
    assert!(!roundtrip_text.is_empty());
    assert_eq!(roundtrip_text, "Hi");

    assert_bool_vector_eq(&collect_invalid_actual_ok(), &[false, false]);

    println!("bytes_module bytes parity demo: pass");
}
