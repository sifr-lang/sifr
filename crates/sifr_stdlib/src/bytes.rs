#[must_use]
pub const fn feature_name() -> &'static str {
    "bytes"
}

#[must_use]
pub fn encode_utf8(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

pub fn bytes_to_hex(bytes: &[u8]) -> Result<String, String> {
    // Preserve the existing Sifr Result API; hexadecimal formatting is infallible.
    Ok(bytes_to_hex_strict(bytes))
}

#[must_use]
fn bytes_to_hex_strict(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{bytes_to_hex, encode_utf8};

    #[test]
    fn encode_utf8_returns_utf8_bytes() {
        assert_eq!(encode_utf8("sifr"), vec![115, 105, 102, 114]);
        assert_eq!(encode_utf8("é"), vec![195, 169]);
    }

    #[test]
    fn bytes_to_hex_formats_lowercase_pairs() {
        assert_eq!(bytes_to_hex(&[0, 1, 15, 16, 255]).unwrap(), "00010f10ff");
    }
}
