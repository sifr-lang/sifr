//! Generated URL, header, and cookie helper functions.

use crate::RustItem;

const URL_RUNTIME: &str = r#"
const __SIFR_URL_MAX_BYTES: usize = 64 * 1024;
const __SIFR_QUERY_MAX_BYTES: usize = 64 * 1024;

fn __sifr_url_error(message: String) -> UrlError {
    UrlError { message }
}

fn __sifr_url_reject_too_large(label: &str, len: usize, max: usize) -> Result<(), UrlError> {
    if len > max {
        return Err(__sifr_url_error(format!("{label} is too large")));
    }
    Ok(())
}

fn __sifr_url_reject_non_ascii_authority_host(input: &str) -> Result<(), UrlError> {
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
    if host.chars().any(|ch| !ch.is_ascii()) {
        return Err(__sifr_url_error(
            "non-ASCII URL hosts are blocked until text/i18n IDNA alignment".to_string(),
        ));
    }
    let decoded_host = percent_encoding::percent_decode_str(host).collect::<Vec<u8>>();
    if decoded_host.iter().any(|byte| !byte.is_ascii()) {
        return Err(__sifr_url_error(
            "non-ASCII URL hosts are blocked until text/i18n IDNA alignment".to_string(),
        ));
    }
    Ok(())
}

fn __sifr_url_validate_ascii_host(host: &str) -> Result<(), UrlError> {
    let bare_host = host
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(host);
    if bare_host.is_empty() {
        return Err(__sifr_url_error("URL host is required".to_string()));
    }
    if bare_host.chars().any(|ch| !ch.is_ascii()) {
        return Err(__sifr_url_error(
            "non-ASCII URL hosts are blocked until text/i18n IDNA alignment".to_string(),
        ));
    }
    __sifr_url_reject_bad_percent(bare_host)?;
    let decoded_host = percent_encoding::percent_decode_str(bare_host).collect::<Vec<u8>>();
    if decoded_host.iter().any(|byte| !byte.is_ascii()) {
        return Err(__sifr_url_error(
            "non-ASCII URL hosts are blocked until text/i18n IDNA alignment".to_string(),
        ));
    }
    Ok(())
}

fn __sifr_url_validate_scheme(scheme: &str) -> Result<(), UrlError> {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return Err(__sifr_url_error("URL scheme is required".to_string()));
    };
    if !first.is_ascii_alphabetic() {
        return Err(__sifr_url_error("invalid URL scheme".to_string()));
    }
    if chars.any(|ch| !matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '+' | '-' | '.')) {
        return Err(__sifr_url_error("invalid URL scheme".to_string()));
    }
    Ok(())
}

fn __sifr_url_is_reg_name_byte(byte: u8) -> bool {
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

fn __sifr_url_validate_ipv6_literal(host: &str) -> Result<(), UrlError> {
    if host.is_empty() {
        return Err(__sifr_url_error("URL host is required".to_string()));
    }
    if host
        .bytes()
        .any(|byte| !matches!(byte, b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9' | b':' | b'.'))
    {
        return Err(__sifr_url_error("invalid URL host".to_string()));
    }
    Ok(())
}

fn __sifr_url_validate_build_host(host: &str) -> Result<(), UrlError> {
    __sifr_url_validate_ascii_host(host)?;
    if let Some(inner) = host.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) {
        return __sifr_url_validate_ipv6_literal(inner);
    }
    if host.starts_with('[') || host.ends_with(']') {
        return Err(__sifr_url_error("invalid URL host".to_string()));
    }
    if host.contains(':') {
        return __sifr_url_validate_ipv6_literal(host);
    }
    if !host.bytes().all(__sifr_url_is_reg_name_byte) {
        return Err(__sifr_url_error("invalid URL host".to_string()));
    }
    Ok(())
}

fn __sifr_url_authority_host(host: &str) -> String {
    if host.starts_with('[') || !host.contains(':') {
        return host.to_string();
    }
    format!("[{host}]")
}

fn __sifr_url_from_parsed(parsed: url::Url) -> Result<Url, UrlError> {
    let Some(host) = parsed.host_str() else {
        return Err(__sifr_url_error("URL host is required".to_string()));
    };
    __sifr_url_validate_ascii_host(host)?;
    let serialized = parsed.to_string();
    __sifr_url_reject_too_large("URL", serialized.len(), __SIFR_URL_MAX_BYTES)?;
    Ok(Url {
        scheme: parsed.scheme().to_string(),
        username: parsed.username().to_string(),
        password: parsed.password().map(str::to_string),
        host: host.to_string(),
        port: parsed.port().map(i64::from),
        path: parsed.path().to_string(),
        query: parsed.query().map(str::to_string),
        fragment: parsed.fragment().map(str::to_string),
        serialized,
    })
}

fn __sifr_url_parse(value: String) -> Result<Url, UrlError> {
    __sifr_url_reject_too_large("URL", value.len(), __SIFR_URL_MAX_BYTES)?;
    __sifr_url_reject_non_ascii_authority_host(&value)?;
    let parsed = url::Url::parse(&value)
        .map_err(|err| __sifr_url_error(format!("invalid URL: {err}")))?;
    __sifr_url_from_parsed(parsed)
}

fn __sifr_url_build(
    scheme: String,
    host: String,
    path: String,
    query: Option<String>,
    port: Option<i64>,
) -> Result<Url, UrlError> {
    __sifr_url_validate_scheme(&scheme)?;
    __sifr_url_validate_build_host(&host)?;
    let query_len = query.as_ref().map_or(0usize, String::len);
    let input_len = scheme.len() + host.len() + path.len() + query_len;
    __sifr_url_reject_too_large("URL", input_len, __SIFR_URL_MAX_BYTES)?;
    if let Some(port) = port {
        if !(0..=65_535).contains(&port) {
            return Err(__sifr_url_error("URL port must be in 0..65535".to_string()));
        }
    }
    let authority_host = __sifr_url_authority_host(&host);
    let base = format!("{scheme}://{authority_host}");
    let mut parsed = url::Url::parse(&base)
        .map_err(|err| __sifr_url_error(format!("invalid URL authority: {err}")))?;
    parsed.set_path(if path.is_empty() { "/" } else { &path });
    parsed.set_query(query.as_deref());
    parsed.set_fragment(None);
    if let Some(port) = port {
        parsed
            .set_port(Some(u16::try_from(port).map_err(|_| {
                __sifr_url_error("URL port must be in 0..65535".to_string())
            })?))
            .map_err(|()| __sifr_url_error("URL scheme does not accept ports".to_string()))?;
    }
    __sifr_url_from_parsed(parsed)
}

fn __sifr_url_percent_encode(value: String) -> String {
    percent_encoding::utf8_percent_encode(&value, percent_encoding::NON_ALPHANUMERIC).to_string()
}

fn __sifr_url_percent_encode_bytes(value: Vec<u8>) -> String {
    percent_encoding::percent_encode(&value, percent_encoding::NON_ALPHANUMERIC).to_string()
}

fn __sifr_url_reject_bad_percent(value: &str) -> Result<(), UrlError> {
    let bytes = value.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        if bytes[idx] == b'%' {
            let valid = idx + 2 < bytes.len()
                && bytes[idx + 1].is_ascii_hexdigit()
                && bytes[idx + 2].is_ascii_hexdigit();
            if !valid {
                return Err(__sifr_url_error("invalid percent escape".to_string()));
            }
            idx += 3;
        } else {
            idx += 1;
        }
    }
    Ok(())
}

fn __sifr_url_percent_decode(value: String) -> Result<String, UrlError> {
    __sifr_url_reject_bad_percent(&value)?;
    percent_encoding::percent_decode_str(&value)
        .decode_utf8()
        .map(|decoded| decoded.into_owned())
        .map_err(|err| __sifr_url_error(format!("percent decoded text is not UTF-8: {err}")))
}

fn __sifr_url_percent_decode_bytes(value: String) -> Result<Vec<u8>, UrlError> {
    __sifr_url_reject_bad_percent(&value)?;
    Ok(percent_encoding::percent_decode_str(&value).collect())
}

fn __sifr_url_normalize_path(path: String) -> Result<String, UrlError> {
    if path.contains('\0') {
        return Err(__sifr_url_error("URL path contains NUL".to_string()));
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

fn __sifr_url_query_parse(query: String) -> Result<Vec<(String, String)>, UrlError> {
    __sifr_url_reject_too_large("query string", query.len(), __SIFR_QUERY_MAX_BYTES)?;
    __sifr_url_reject_bad_percent(&query)?;
    let mut pairs = Vec::new();
    for raw_pair in query.split('&') {
        if raw_pair.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = raw_pair.split_once('=').unwrap_or((raw_pair, ""));
        pairs.push((
            __sifr_url_query_component_decode(raw_key)?,
            __sifr_url_query_component_decode(raw_value)?,
        ));
    }
    Ok(pairs)
}

fn __sifr_url_query_component_decode(value: &str) -> Result<String, UrlError> {
    let plus_as_space = value.replace('+', " ");
    percent_encoding::percent_decode_str(&plus_as_space)
        .decode_utf8()
        .map(|decoded| decoded.into_owned())
        .map_err(|err| __sifr_url_error(format!("query component is not UTF-8: {err}")))
}

fn __sifr_url_query_build(pairs: Vec<(String, String)>) -> Result<String, UrlError> {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(&key, &value);
    }
    let query = serializer.finish();
    __sifr_url_reject_too_large("query string", query.len(), __SIFR_QUERY_MAX_BYTES)?;
    Ok(query)
}
"#;

const HTTP_RUNTIME: &str = r#"
const __SIFR_HEADER_NAME_MAX_BYTES: usize = 1024;
const __SIFR_HEADER_VALUE_MAX_BYTES: usize = 64 * 1024;
const __SIFR_HEADER_SECTION_MAX_BYTES: usize = 1024 * 1024;

fn __sifr_header_error(message: String) -> HeaderError {
    HeaderError { message }
}

fn __sifr_http_reject_too_large(label: &str, len: usize, max: usize) -> Result<(), HeaderError> {
    if len > max {
        return Err(__sifr_header_error(format!("{label} is too large")));
    }
    Ok(())
}

fn __sifr_http_validate_header_name(value: String) -> Result<HeaderName, HeaderError> {
    __sifr_http_reject_too_large("header name", value.len(), __SIFR_HEADER_NAME_MAX_BYTES)?;
    let parsed = http::HeaderName::from_bytes(value.as_bytes())
        .map_err(|err| __sifr_header_error(format!("invalid header name: {err}")))?;
    Ok(HeaderName {
        value: parsed.as_str().to_string(),
    })
}

fn __sifr_http_validate_header_value(value: String) -> Result<HeaderValue, HeaderError> {
    __sifr_http_reject_too_large("header value", value.len(), __SIFR_HEADER_VALUE_MAX_BYTES)?;
    if value.contains('\r') || value.contains('\n') {
        return Err(__sifr_header_error(
            "header values must not contain obs-fold or line breaks".to_string(),
        ));
    }
    let trimmed = value.trim_matches([' ', '\t']).to_string();
    http::HeaderValue::from_str(&trimmed)
        .map_err(|err| __sifr_header_error(format!("invalid header value: {err}")))?;
    Ok(HeaderValue { value: trimmed })
}

fn __sifr_http_is_cookie_name_byte(byte: u8) -> bool {
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

fn __sifr_http_is_cookie_value_byte(byte: u8) -> bool {
    matches!(byte, 0x21 | 0x23..=0x2B | 0x2D..=0x3A | 0x3C..=0x5B | 0x5D..=0x7E)
}

fn __sifr_http_validate_cookie_name(name: &str) -> Result<(), HeaderError> {
    if name.is_empty() {
        return Err(__sifr_header_error("invalid cookie name".to_string()));
    }
    if !name.bytes().all(__sifr_http_is_cookie_name_byte) {
        return Err(__sifr_header_error("invalid cookie name".to_string()));
    }
    Ok(())
}

fn __sifr_http_validate_cookie_value(value: &str) -> Result<(), HeaderError> {
    if !value.bytes().all(__sifr_http_is_cookie_value_byte) {
        return Err(__sifr_header_error("invalid cookie value".to_string()));
    }
    Ok(())
}

fn __sifr_http_reject_cookie_line_breaks(value: &str) -> Result<(), HeaderError> {
    if value.contains('\r') || value.contains('\n') || value.contains('\0') {
        return Err(__sifr_header_error(
            "cookie headers must not contain line breaks or NUL".to_string(),
        ));
    }
    Ok(())
}

fn __sifr_http_header_map_from_pairs(
    pairs: Vec<(String, String)>,
) -> Result<HeaderMap, HeaderError> {
    if pairs.len() > 1024 {
        return Err(__sifr_header_error("too many HTTP headers".to_string()));
    }
    let mut section_len = 0usize;
    let mut entries = Vec::with_capacity(pairs.len());
    for (name, value) in pairs {
        section_len = section_len.saturating_add(name.len()).saturating_add(value.len());
        __sifr_http_reject_too_large(
            "header section",
            section_len,
            __SIFR_HEADER_SECTION_MAX_BYTES,
        )?;
        entries.push((
            __sifr_http_validate_header_name(name)?,
            __sifr_http_validate_header_value(value)?,
        ));
    }
    Ok(HeaderMap { entries })
}

fn __sifr_http_parse_cookie_header(value: String) -> Result<Vec<(String, String)>, HeaderError> {
    __sifr_http_reject_too_large("header value", value.len(), __SIFR_HEADER_VALUE_MAX_BYTES)?;
    __sifr_http_reject_cookie_line_breaks(&value)?;
    let mut cookies = Vec::new();
    for part in value.split(';') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((name, cookie_value)) = trimmed.split_once('=') else {
            return Err(__sifr_header_error("invalid cookie header".to_string()));
        };
        __sifr_http_validate_cookie_name(name)?;
        __sifr_http_validate_cookie_value(cookie_value)?;
        cookies.push((name.to_string(), cookie_value.to_string()));
    }
    Ok(cookies)
}

fn __sifr_http_build_cookie_header(cookies: Vec<(String, String)>) -> Result<String, HeaderError> {
    let mut parts = Vec::with_capacity(cookies.len());
    for (name, value) in cookies {
        __sifr_http_validate_cookie_name(&name)?;
        __sifr_http_validate_cookie_value(&value)?;
        parts.push(format!("{name}={value}"));
    }
    let header = parts.join("; ");
    __sifr_http_reject_cookie_line_breaks(&header)?;
    __sifr_http_reject_too_large("header value", header.len(), __SIFR_HEADER_VALUE_MAX_BYTES)?;
    Ok(header)
}
"#;

pub(crate) fn build_url_runtime_items() -> Vec<RustItem> {
    vec![RustItem::Attr(URL_RUNTIME.to_string())]
}

pub(crate) fn build_http_runtime_items() -> Vec<RustItem> {
    vec![RustItem::Attr(HTTP_RUNTIME.to_string())]
}
