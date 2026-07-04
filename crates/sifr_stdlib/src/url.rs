#[must_use]
pub const fn feature_name() -> &'static str {
    "url"
}

use sifr_runtime::interop::SifrIntBridge;

const URL_MAX_BYTES: usize = 64 * 1024;
const QUERY_MAX_BYTES: usize = 64 * 1024;

pub fn url_parse_parts(value: &str) -> Result<Vec<String>, String> {
    reject_too_large("URL", value.len(), URL_MAX_BYTES)?;
    reject_non_ascii_authority_host(value)?;
    let parsed = url::Url::parse(value).map_err(|err| format!("invalid URL: {err}"))?;
    url_parts_from_parsed(&parsed)
}

pub fn url_build_parts(
    scheme: &str,
    host: &str,
    path: &str,
    query: Option<String>,
    port: Option<SifrIntBridge>,
) -> Result<Vec<String>, String> {
    validate_scheme(scheme)?;
    validate_build_host(host)?;
    let query_len = query.as_ref().map_or(0usize, String::len);
    let input_len = scheme.len() + host.len() + path.len() + query_len;
    reject_too_large("URL", input_len, URL_MAX_BYTES)?;
    let port = port.map(|value| value.to_i64_saturating());
    if let Some(port) = port {
        if !(0..=65_535).contains(&port) {
            return Err("URL port must be in 0..65535".to_string());
        }
    }

    let authority_host = authority_host(host);
    let base = format!("{scheme}://{authority_host}");
    let mut parsed =
        url::Url::parse(&base).map_err(|err| format!("invalid URL authority: {err}"))?;
    parsed.set_path(if path.is_empty() { "/" } else { path });
    if let Some(query) = query {
        parsed.set_query(Some(&query));
    } else {
        parsed.set_query(None);
    }
    parsed.set_fragment(None);
    if let Some(port) = port {
        parsed
            .set_port(Some(
                u16::try_from(port).map_err(|_| "URL port must be in 0..65535".to_string())?,
            ))
            .map_err(|()| "URL scheme does not accept ports".to_string())?;
    }
    url_parts_from_parsed(&parsed)
}

pub fn url_percent_encode(value: &str) -> String {
    percent_encoding::utf8_percent_encode(value, percent_encoding::NON_ALPHANUMERIC).to_string()
}

pub fn url_percent_encode_bytes(value: &[u8]) -> String {
    percent_encoding::percent_encode(value, percent_encoding::NON_ALPHANUMERIC).to_string()
}

pub fn url_percent_decode(value: &str) -> Result<String, String> {
    reject_bad_percent(value)?;
    percent_encoding::percent_decode_str(value)
        .decode_utf8()
        .map(std::borrow::Cow::into_owned)
        .map_err(|err| format!("percent decoded text is not UTF-8: {err}"))
}

pub fn url_percent_decode_bytes(value: &str) -> Result<Vec<u8>, String> {
    reject_bad_percent(value)?;
    Ok(percent_encoding::percent_decode_str(value).collect())
}

pub fn url_normalize_path(path: &str) -> Result<String, String> {
    if path.contains('\0') {
        return Err("URL path contains NUL".to_string());
    }
    let absolute = path.starts_with('/');
    let trailing = path.ends_with('/') || path.ends_with("/.") || path.ends_with("/..");
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            _ => segments.push(segment),
        }
    }
    let mut normalized = String::new();
    if absolute {
        normalized.push('/');
    }
    normalized.push_str(&segments.join("/"));
    if trailing && !normalized.ends_with('/') {
        normalized.push('/');
    }
    if normalized.is_empty() {
        normalized.push(if absolute { '/' } else { '.' });
    }
    Ok(normalized)
}

pub fn url_query_parse_flat(query: &str) -> Result<Vec<String>, String> {
    reject_too_large("query string", query.len(), QUERY_MAX_BYTES)?;
    reject_bad_percent(query)?;
    let mut flat_pairs = Vec::new();
    for raw_pair in query.split('&') {
        if raw_pair.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = raw_pair.split_once('=').unwrap_or((raw_pair, ""));
        flat_pairs.push(query_component_decode(raw_key)?);
        flat_pairs.push(query_component_decode(raw_value)?);
    }
    Ok(flat_pairs)
}

pub fn url_query_build_flat(flat_pairs: &[String]) -> Result<String, String> {
    if !flat_pairs.len().is_multiple_of(2) {
        return Err("query pair payload must contain key/value entries".to_string());
    }
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for pair in flat_pairs.chunks_exact(2) {
        serializer.append_pair(&pair[0], &pair[1]);
    }
    let query = serializer.finish();
    reject_too_large("query string", query.len(), QUERY_MAX_BYTES)?;
    Ok(query)
}

fn reject_too_large(label: &str, len: usize, max: usize) -> Result<(), String> {
    if len > max {
        return Err(format!("{label} is too large"));
    }
    Ok(())
}

fn reject_non_ascii_authority_host(input: &str) -> Result<(), String> {
    let Some(scheme_end) = input.find("://") else {
        return Ok(());
    };
    let authority_start = scheme_end + 3;
    let authority_end = input[authority_start..]
        .find(['/', '?', '#'])
        .map_or(input.len(), |idx| authority_start + idx);
    let authority = &input[authority_start..authority_end];
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = if let Some(rest) = host_port.strip_prefix('[') {
        rest.find(']').map_or(host_port, |idx| &rest[..idx])
    } else {
        host_port.split(':').next().unwrap_or(host_port)
    };
    if !host.is_ascii() {
        return Err(non_ascii_host_message());
    }
    let decoded_host = percent_encoding::percent_decode_str(host).collect::<Vec<u8>>();
    if decoded_host.iter().any(|byte| !byte.is_ascii()) {
        return Err(non_ascii_host_message());
    }
    Ok(())
}

fn validate_ascii_host(host: &str) -> Result<(), String> {
    let bare_host = host
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(host);
    if bare_host.is_empty() {
        return Err("URL host is required".to_string());
    }
    if !bare_host.is_ascii() {
        return Err(non_ascii_host_message());
    }
    reject_bad_percent(bare_host)?;
    let decoded_host = percent_encoding::percent_decode_str(bare_host).collect::<Vec<u8>>();
    if decoded_host.iter().any(|byte| !byte.is_ascii()) {
        return Err(non_ascii_host_message());
    }
    Ok(())
}

fn non_ascii_host_message() -> String {
    "non-ASCII URL hosts are blocked until text/i18n IDNA alignment".to_string()
}

fn validate_scheme(scheme: &str) -> Result<(), String> {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return Err("URL scheme is required".to_string());
    };
    if !first.is_ascii_alphabetic() {
        return Err("invalid URL scheme".to_string());
    }
    if chars.any(|ch| !matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '+' | '-' | '.')) {
        return Err("invalid URL scheme".to_string());
    }
    Ok(())
}

fn is_reg_name_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'a'..=b'z'
            | b'A'..=b'Z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b'!'
            | b'$'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'='
            | b'%'
    )
}

fn validate_ipv6_literal(host: &str) -> Result<(), String> {
    if host.is_empty() {
        return Err("URL host is required".to_string());
    }
    if host
        .bytes()
        .any(|byte| !matches!(byte, b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9' | b':' | b'.'))
    {
        return Err("invalid URL host".to_string());
    }
    Ok(())
}

fn validate_build_host(host: &str) -> Result<(), String> {
    validate_ascii_host(host)?;
    if let Some(inner) = host
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        return validate_ipv6_literal(inner);
    }
    if host.starts_with('[') || host.ends_with(']') {
        return Err("invalid URL host".to_string());
    }
    if host.contains(':') {
        return validate_ipv6_literal(host);
    }
    if !host.bytes().all(is_reg_name_byte) {
        return Err("invalid URL host".to_string());
    }
    Ok(())
}

fn authority_host(host: &str) -> String {
    if host.starts_with('[') || !host.contains(':') {
        return host.to_string();
    }
    format!("[{host}]")
}

fn url_parts_from_parsed(parsed: &url::Url) -> Result<Vec<String>, String> {
    let Some(host) = parsed.host_str() else {
        return Err("URL host is required".to_string());
    };
    validate_ascii_host(host)?;
    let serialized = parsed.to_string();
    reject_too_large("URL", serialized.len(), URL_MAX_BYTES)?;
    Ok(vec![
        parsed.scheme().to_string(),
        parsed.username().to_string(),
        parsed.password().unwrap_or("").to_string(),
        bool_marker(parsed.password().is_some()),
        host.to_string(),
        parsed
            .port()
            .map_or_else(String::new, |port| port.to_string()),
        parsed.path().to_string(),
        parsed.query().unwrap_or("").to_string(),
        bool_marker(parsed.query().is_some()),
        parsed.fragment().unwrap_or("").to_string(),
        bool_marker(parsed.fragment().is_some()),
        serialized,
    ])
}

fn bool_marker(value: bool) -> String {
    if value { "1" } else { "0" }.to_string()
}

fn reject_bad_percent(value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        if bytes[idx] == b'%' {
            let valid = idx + 2 < bytes.len()
                && bytes[idx + 1].is_ascii_hexdigit()
                && bytes[idx + 2].is_ascii_hexdigit();
            if !valid {
                return Err("invalid percent escape".to_string());
            }
            idx += 3;
        } else {
            idx += 1;
        }
    }
    Ok(())
}

fn query_component_decode(value: &str) -> Result<String, String> {
    let plus_as_space = value.replace('+', " ");
    percent_encoding::percent_decode_str(&plus_as_space)
        .decode_utf8()
        .map(std::borrow::Cow::into_owned)
        .map_err(|err| format!("query component is not UTF-8: {err}"))
}
