#[derive(Debug, Clone)]
struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
struct ValueError(String);

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ValueError {}

fn bytes_with_size(size: i64) -> Result<Vec<u8>, ValueError> {
    if size < 0 {
        return Err(ValueError(
            "bytes(size) requires a non-negative size".to_string(),
        ));
    }
    Ok(vec![0; size as usize])
}

fn bytes_from_ints(values: &[i64]) -> Result<Vec<u8>, ValueError> {
    let mut out = Vec::with_capacity(values.len());
    for (index, value) in values.iter().copied().enumerate() {
        let byte = u8::try_from(value)
            .map_err(|_| ValueError(format!("byte out of range at index {index}: {value}")))?;
        out.push(byte);
    }
    Ok(out)
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

fn encode_text(text: &str, codec: &str) -> Result<Vec<u8>, ParseError> {
    let codec = codec.to_ascii_lowercase();
    if codec != "utf-8" && codec != "utf8" {
        return Err(ParseError(format!(
            "str.encode() currently supports only UTF-8 encoding, got {codec}"
        )));
    }
    Ok(text.as_bytes().to_vec())
}

fn decode_utf8(bytes: &[u8]) -> Result<String, ParseError> {
    String::from_utf8(bytes.to_vec()).map_err(|error| ParseError(error.to_string()))
}

fn main() {
    let bad_size = bytes_with_size(-1).is_err();
    let bad_values = bytes_from_ints(&[0, 999]).is_err();
    let bad_hex = bytes_from_hex("GG").is_err();
    let bad_codec = encode_text("abc", "latin-1").is_err();
    let bad_utf8 = decode_utf8(&[0xff]).is_err();

    assert!(bad_size);
    assert!(bad_values);
    assert!(bad_hex);
    assert!(bad_codec);
    assert!(bad_utf8);

    println!("bytes_bytes_errors_boundary_demo: ok");
}
