#[derive(Debug, Clone)]
struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ParseError {}

fn encode_utf8(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

fn decode_utf8(bytes: &[u8]) -> Result<String, ParseError> {
    String::from_utf8(bytes.to_vec()).map_err(|error| ParseError(error.to_string()))
}

fn bytes_from_hex(text: &str) -> Result<Vec<u8>, ParseError> {
    let cleaned: String = text
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect();
    if !cleaned.chars().all(|ch| ch.is_ascii_hexdigit()) {
        let invalid = cleaned
            .chars()
            .find(|ch| !ch.is_ascii_hexdigit())
            .unwrap_or('?');
        return Err(ParseError(format!("invalid hex character: {invalid}")));
    }
    if cleaned.len() % 2 != 0 {
        return Err(ParseError(
            "fromhex() arg must contain an even number of hexadecimal digits".to_string(),
        ));
    }
    cleaned
        .as_bytes()
        .chunks(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).map_err(|error| ParseError(error.to_string()))?;
            u8::from_str_radix(pair, 16).map_err(|error| ParseError(error.to_string()))
        })
        .collect()
}

fn main() {
    let encoded = encode_utf8("sifr-bytes");
    let decode_ok = decode_utf8(&encoded).is_ok_and(|decoded| decoded == "sifr-bytes");
    assert!(decode_ok);

    let hex_ok = bytes_from_hex("73696672")
        .and_then(|parsed| decode_utf8(&parsed))
        .is_ok_and(|decoded| decoded == "sifr");
    assert!(hex_ok);
}
