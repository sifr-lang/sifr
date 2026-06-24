use base64::{engine::general_purpose, Engine as _};
use sifr_runtime::interop::SifrIntBridge;
use std::collections::HashSet;

#[must_use]
pub const fn feature_name() -> &'static str {
    "base64"
}

pub fn base64_encode(input: &str) -> String {
    general_purpose::STANDARD.encode(input.as_bytes())
}

pub fn base64_encode_bytes(data: &[u8]) -> Vec<u8> {
    general_purpose::STANDARD.encode(data).into_bytes()
}

pub fn base64_decode(input: &str) -> Result<String, String> {
    let bytes = general_purpose::STANDARD
        .decode(input.as_bytes())
        .map_err(|error| error.to_string())?;
    String::from_utf8(bytes).map_err(|error| error.to_string())
}

pub fn base64_decode_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    general_purpose::STANDARD
        .decode(data)
        .map_err(|error| error.to_string())
}

pub fn base64_encode_opts(
    input: &str,
    altchars: &str,
    wrapcol: SifrIntBridge,
) -> Result<String, String> {
    let wrapcol = wrapcol.to_i64_saturating();
    if wrapcol < 0 {
        return Err("wrapcol must be >= 0".to_string());
    }
    let mut encoded = base64_encode(input);
    if !altchars.is_empty() {
        let mut chars = altchars.chars();
        let first = chars.next();
        let second = chars.next();
        if first.is_none() || second.is_none() || chars.next().is_some() {
            return Err(format!("invalid altchars: {altchars}"));
        }
        let alt_plus = first.unwrap_or('+');
        let alt_slash = second.unwrap_or('/');
        encoded = encoded
            .chars()
            .map(|ch| {
                if ch == '+' {
                    alt_plus
                } else if ch == '/' {
                    alt_slash
                } else {
                    ch
                }
            })
            .collect();
    }
    if wrapcol == 0 {
        return Ok(encoded);
    }
    let width = usize::try_from(wrapcol).map_err(|_| "wrapcol must be >= 0".to_string())?;
    let mut wrapped = String::new();
    for (index, ch) in encoded.chars().enumerate() {
        if index > 0 && index % width == 0 {
            wrapped.push('\n');
        }
        wrapped.push(ch);
    }
    Ok(wrapped)
}

pub fn base64_decode_opts(
    input: &str,
    altchars: &str,
    validate: bool,
    ignorechars: &str,
) -> Result<String, String> {
    let (has_alt, alt_plus, alt_slash) = parse_altchars(altchars)?;
    let ignore_set = ignorechars.chars().collect::<HashSet<_>>();
    let mut normalized = String::new();
    for ch in input.chars() {
        if ignore_set.contains(&ch) {
            continue;
        }
        let mapped = if has_alt && ch == alt_plus {
            '+'
        } else if has_alt && ch == alt_slash {
            '/'
        } else {
            ch
        };
        if is_base64_char(mapped) {
            normalized.push(mapped);
        } else if validate {
            return Err(format!("invalid base64 character: {ch}"));
        }
    }
    base64_decode(&normalized)
}

pub fn urlsafe_b64encode(input: &str) -> String {
    general_purpose::URL_SAFE.encode(input.as_bytes())
}

pub fn urlsafe_b64encode_bytes(data: &[u8]) -> Vec<u8> {
    general_purpose::URL_SAFE.encode(data).into_bytes()
}

pub fn urlsafe_b64decode(input: &str) -> Result<String, String> {
    let bytes = general_purpose::URL_SAFE
        .decode(input.as_bytes())
        .map_err(|error| error.to_string())?;
    String::from_utf8(bytes).map_err(|error| error.to_string())
}

pub fn urlsafe_b64decode_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    general_purpose::URL_SAFE
        .decode(data)
        .map_err(|error| error.to_string())
}

pub fn b32encode(input: &str) -> String {
    encode_base32(input.as_bytes(), b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567")
}

pub fn b32decode(input: &str) -> Result<String, String> {
    decode_base32(
        input,
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
        "invalid base32 char: {}",
    )
}

pub fn b32hexencode(input: &str) -> String {
    encode_base32(input.as_bytes(), b"0123456789ABCDEFGHIJKLMNOPQRSTUV")
}

pub fn b32hexdecode(input: &str) -> Result<String, String> {
    decode_base32(
        input,
        b"0123456789ABCDEFGHIJKLMNOPQRSTUV",
        "invalid base32hex char: {}",
    )
}

fn parse_altchars(altchars: &str) -> Result<(bool, char, char), String> {
    if altchars.is_empty() {
        return Ok((false, '+', '/'));
    }
    let mut chars = altchars.chars();
    let Some(alt_plus) = chars.next() else {
        return Ok((false, '+', '/'));
    };
    let Some(alt_slash) = chars.next() else {
        return Err(format!("invalid altchars: {altchars}"));
    };
    if chars.next().is_some() {
        return Err(format!("invalid altchars: {altchars}"));
    }
    Ok((true, alt_plus, alt_slash))
}

fn is_base64_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '+' || ch == '/' || ch == '='
}

fn encode_base32(data: &[u8], alphabet: &[u8; 32]) -> String {
    let mut out = String::new();
    let mut index = 0;
    while index < data.len() {
        let remaining = data.len() - index;
        let chunk_len = remaining.min(5);
        let mut buffer = 0_u64;
        for offset in 0..5 {
            let byte = data.get(index + offset).copied().unwrap_or(0);
            buffer = (buffer << 8) | u64::from(byte);
        }
        let useful_chars = (chunk_len * 8).div_ceil(5);
        for shift_index in 0..8 {
            if shift_index < useful_chars {
                let alphabet_index = ((buffer >> (35 - shift_index * 5)) & 0x1f) as usize;
                out.push(char::from(alphabet[alphabet_index]));
            } else {
                out.push('=');
            }
        }
        index += 5;
    }
    out
}

fn decode_base32(
    input: &str,
    alphabet: &[u8; 32],
    invalid_message: &str,
) -> Result<String, String> {
    let trimmed = input.trim_end_matches('=');
    let mut bits = 0_u64;
    let mut bit_count = 0_u8;
    let mut out = Vec::new();
    for ch in trimmed.chars() {
        let upper = ch.to_ascii_uppercase();
        let value = alphabet
            .iter()
            .position(|byte| char::from(*byte) == upper)
            .ok_or_else(|| invalid_message.replace("{}", &ch.to_string()))?;
        bits = (bits << 5) | value as u64;
        bit_count += 5;
        if bit_count >= 8 {
            bit_count -= 8;
            out.push(((bits >> bit_count) & 0xff) as u8);
        }
    }
    String::from_utf8(out).map_err(|error| error.to_string())
}
