use sifr_runtime::interop::SifrIntBridge;

const HEADER_NAME_MAX_BYTES: usize = 1024;
const HEADER_VALUE_MAX_BYTES: usize = 64 * 1024;
const HEADER_SECTION_MAX_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderName(String);

impl HeaderName {
    pub fn new(value: &str) -> Result<Self, String> {
        http_validate_header_name(value).map(Self)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn bridge_i64(value: &SifrIntBridge, name: &str) -> Result<i64, String> {
    value
        .try_to_i64()
        .map_err(|error| format!("{name} must fit in i64: {error}"))
}

fn reject_too_large(label: &str, len: usize, max: usize) -> Result<(), String> {
    if len > max {
        return Err(format!("{label} is too large"));
    }
    Ok(())
}

fn is_cookie_name_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'a'..=b'z'
            | b'A'..=b'Z'
            | b'0'..=b'9'
            | b'!'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
    )
}

fn is_cookie_value_byte(byte: u8) -> bool {
    matches!(byte, 0x21 | 0x23..=0x2B | 0x2D..=0x3A | 0x3C..=0x5B | 0x5D..=0x7E)
}

fn validate_cookie_name(name: &str) -> Result<(), String> {
    if name.is_empty() || !name.bytes().all(is_cookie_name_byte) {
        return Err("invalid cookie name".to_string());
    }
    Ok(())
}

fn validate_cookie_value(value: &str) -> Result<(), String> {
    if !value.bytes().all(is_cookie_value_byte) {
        return Err("invalid cookie value".to_string());
    }
    Ok(())
}

fn reject_cookie_line_breaks(value: &str) -> Result<(), String> {
    if value.contains('\r') || value.contains('\n') || value.contains('\0') {
        return Err("cookie headers must not contain line breaks or NUL".to_string());
    }
    Ok(())
}

pub fn http_validate_header_name(value: &str) -> Result<String, String> {
    reject_too_large("header name", value.len(), HEADER_NAME_MAX_BYTES)?;
    let parsed = http::HeaderName::from_bytes(value.as_bytes())
        .map_err(|error| format!("invalid header name: {error}"))?;
    Ok(parsed.as_str().to_string())
}

pub fn http_validate_header_value(value: &str) -> Result<String, String> {
    reject_too_large("header value", value.len(), HEADER_VALUE_MAX_BYTES)?;
    if value.contains('\r') || value.contains('\n') {
        return Err("header values must not contain obs-fold or line breaks".to_string());
    }
    let trimmed = value.trim_matches([' ', '\t']).to_string();
    http::HeaderValue::from_str(&trimmed)
        .map_err(|error| format!("invalid header value: {error}"))?;
    Ok(trimmed)
}

pub fn http_validate_method(value: &str) -> Result<String, String> {
    let parsed = http::Method::from_bytes(value.as_bytes())
        .map_err(|error| format!("invalid HTTP method: {error}"))?;
    Ok(parsed.to_string())
}

pub fn http_validate_status(code: SifrIntBridge) -> Result<SifrIntBridge, String> {
    let code = bridge_i64(&code, "HTTP status")?;
    let status = u16::try_from(code)
        .ok()
        .and_then(|code| http::StatusCode::from_u16(code).ok())
        .ok_or_else(|| "invalid HTTP status".to_string())?;
    Ok(i64::from(status.as_u16()).into())
}

pub fn http_validate_version(value: &str) -> Result<String, String> {
    match value {
        "HTTP/1.0" | "HTTP/1.1" | "HTTP/2" | "HTTP/3" => Ok(value.to_string()),
        _ => Err("invalid HTTP version".to_string()),
    }
}

pub fn http_header_map_from_pairs(pairs: &[String]) -> Result<Vec<String>, String> {
    if !pairs.len().is_multiple_of(2) {
        return Err("HTTP header pair payload must contain name/value entries".to_string());
    }
    if pairs.len() > 1024 {
        return Err("too many HTTP headers".to_string());
    }
    let mut section_len = 0usize;
    let mut entries = Vec::with_capacity(pairs.len());
    for pair in pairs.chunks_exact(2) {
        let name = &pair[0];
        let value = &pair[1];
        section_len = section_len
            .saturating_add(name.len())
            .saturating_add(value.len());
        reject_too_large("header section", section_len, HEADER_SECTION_MAX_BYTES)?;
        entries.push(http_validate_header_name(name)?);
        entries.push(http_validate_header_value(value)?);
    }
    Ok(entries)
}

pub fn http_parse_cookie_header(value: &str) -> Result<Vec<String>, String> {
    reject_too_large("header value", value.len(), HEADER_VALUE_MAX_BYTES)?;
    reject_cookie_line_breaks(value)?;
    let mut cookies = Vec::new();
    for part in value.split(';') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((name, cookie_value)) = trimmed.split_once('=') else {
            return Err("invalid cookie header".to_string());
        };
        validate_cookie_name(name)?;
        validate_cookie_value(cookie_value)?;
        cookies.push(name.to_string());
        cookies.push(cookie_value.to_string());
    }
    Ok(cookies)
}

pub fn http_build_cookie_header(cookies: &[String]) -> Result<String, String> {
    if !cookies.len().is_multiple_of(2) {
        return Err("cookie pair payload must contain name/value entries".to_string());
    }
    let mut parts = Vec::with_capacity(cookies.len() / 2);
    for pair in cookies.chunks_exact(2) {
        let name = &pair[0];
        let value = &pair[1];
        validate_cookie_name(name)?;
        validate_cookie_value(value)?;
        parts.push(format!("{name}={value}"));
    }
    let header = parts.join("; ");
    reject_cookie_line_breaks(&header)?;
    reject_too_large("header value", header.len(), HEADER_VALUE_MAX_BYTES)?;
    Ok(header)
}
