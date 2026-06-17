use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct ValueError(String);

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ValueError {}

#[derive(Debug)]
struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ParseError {}

fn zero_bytes(len: i64) -> Result<Vec<u8>, ValueError> {
    usize::try_from(len)
        .map(|size| vec![0; size])
        .map_err(|_| ValueError("bytes(size) requires a non-negative size".to_string()))
}

fn bytes_from_ints(values: &[i64]) -> Result<Vec<u8>, ValueError> {
    values
        .iter()
        .copied()
        .enumerate()
        .map(|(index, value)| {
            u8::try_from(value)
                .map_err(|_| ValueError(format!("byte out of range at index {index}: {value}")))
        })
        .collect()
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

fn encode_utf8(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

fn decode_utf8(bytes: Vec<u8>) -> Result<String, ParseError> {
    String::from_utf8(bytes).map_err(|err| ParseError(err.to_string()))
}

fn main() {
    let size_ok = match zero_bytes(6) {
        Ok(zeros) => zeros.len() == 6,
        Err(_) => false,
    };
    assert!(size_ok);

    let from_ints_ok = match bytes_from_ints(&[83, 105, 102, 114]) {
        Ok(from_list) => {
            from_list.first().copied() == Some(83) && from_list.get(3).copied() == Some(114)
        }
        Err(_) => false,
    };
    assert!(from_ints_ok);

    let from_hex_ok = match bytes_from_hex("53 69 66 72").and_then(decode_utf8) {
        Ok(text) => text == "Sifr",
        Err(_) => false,
    };
    assert!(from_hex_ok);

    let encode_ok = match decode_utf8(encode_utf8("bytes_constructors-demo")) {
        Ok(text) => text == "bytes_constructors-demo",
        Err(_) => false,
    };
    assert!(encode_ok);

    println!("bytes_bytes_constructors_surface_demo: ok");
}
