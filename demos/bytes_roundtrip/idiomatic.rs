use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ParseError {}

fn encode_utf8(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

fn decode_utf8(bytes: Vec<u8>) -> Result<String, ParseError> {
    String::from_utf8(bytes).map_err(|err| ParseError(err.to_string()))
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bytes_from_hex(text: &str) -> Result<Vec<u8>, ParseError> {
    let cleaned: String = text
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect();
    if cleaned.len() % 2 != 0 {
        return Err(ParseError(
            "fromhex() arg must contain an even number of hexadecimal digits".to_string(),
        ));
    }
    if !cleaned.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(ParseError("invalid hex input".to_string()));
    }

    cleaned
        .as_bytes()
        .chunks(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).map_err(|err| ParseError(err.to_string()))?;
            u8::from_str_radix(pair, 16).map_err(|err| ParseError(err.to_string()))
        })
        .collect()
}

fn main() {
    let payload = encode_utf8("binary-sample");
    assert!(payload.starts_with(b"binary"));
    assert!(payload.ends_with(b"sample"));

    let conversion_ok = bytes_from_hex(&bytes_to_hex(&payload))
        .and_then(decode_utf8)
        .map(|text| text == "binary-sample")
        .unwrap_or(false);

    assert!(conversion_ok);
}
