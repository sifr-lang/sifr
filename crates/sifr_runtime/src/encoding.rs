//! Encoding substrate shared by generated Sifr programs.

use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Handler {
    Strict,
    Replace,
    Ignore,
    BackslashReplace,
    XmlCharRefReplace,
    NameReplace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Codec {
    Utf8,
    Utf8Sig,
    Ascii,
    Latin1,
    Utf16Le,
    Utf16Be,
    Windows(&'static encoding_rs::Encoding),
}

#[must_use]
pub fn is_supported_encoding(label: &str) -> bool {
    resolve_codec(label).is_some()
}

pub fn canonical_label(label: &str) -> Result<String, String> {
    let codec = resolve_codec(label).ok_or_else(|| unsupported_encoding(label))?;
    Ok(codec.canonical_label().to_string())
}

pub fn decode_text(data: &[u8], label: &str, handler: &str) -> Result<String, String> {
    let (text, _) = decode_with_recoveries(data, label, handler)?;
    Ok(text)
}

pub fn decode_recoveries(data: &[u8], label: &str, handler: &str) -> Result<Vec<String>, String> {
    let (_, recoveries) = decode_with_recoveries(data, label, handler)?;
    Ok(recoveries)
}

pub fn encode_bytes(text: &str, label: &str, handler: &str) -> Result<Vec<u8>, String> {
    let (bytes, _) = encode_with_recoveries(text, label, handler)?;
    Ok(bytes)
}

pub fn encode_recoveries(text: &str, label: &str, handler: &str) -> Result<Vec<String>, String> {
    let (_, recoveries) = encode_with_recoveries(text, label, handler)?;
    Ok(recoveries)
}

pub fn incremental_decode_with_recoveries(
    data: &[u8],
    pending: &[u8],
    label: &str,
    handler: &str,
    final_chunk: bool,
) -> Result<(String, Vec<String>), String> {
    let codec = resolve_codec(label).ok_or_else(|| unsupported_encoding(label))?;
    let mut combined = Vec::with_capacity(pending.len() + data.len());
    combined.extend_from_slice(pending);
    combined.extend_from_slice(data);
    let tail_len = if final_chunk {
        0
    } else {
        pending_tail_len(&combined, codec)
    };
    let ready_len = combined.len().saturating_sub(tail_len);
    decode_with_recoveries(&combined[..ready_len], label, handler)
}

pub fn incremental_decode_pending(
    data: &[u8],
    pending: &[u8],
    label: &str,
    final_chunk: bool,
) -> Result<Vec<u8>, String> {
    let codec = resolve_codec(label).ok_or_else(|| unsupported_encoding(label))?;
    if final_chunk {
        return Ok(Vec::new());
    }
    let mut combined = Vec::with_capacity(pending.len() + data.len());
    combined.extend_from_slice(pending);
    combined.extend_from_slice(data);
    let tail_len = pending_tail_len(&combined, codec);
    Ok(combined[combined.len().saturating_sub(tail_len)..].to_vec())
}

fn pending_tail_len(data: &[u8], codec: Codec) -> usize {
    match codec {
        Codec::Utf8 | Codec::Utf8Sig => utf8_pending_tail_len(data),
        Codec::Utf16Le | Codec::Utf16Be if !data.len().is_multiple_of(2) => 1,
        _ => 0,
    }
}

fn utf8_pending_tail_len(data: &[u8]) -> usize {
    let mut continuation_count = 0usize;
    for byte in data.iter().copied().rev() {
        if (0x80..=0xBF).contains(&byte) {
            continuation_count += 1;
            continue;
        }
        let expected = match byte {
            0xC2..=0xDF => 1,
            0xE0..=0xEF => 2,
            0xF0..=0xF4 => 3,
            _ => 0,
        };
        if expected == 0 {
            return 0;
        }
        if continuation_count < expected {
            return continuation_count + 1;
        }
        return 0;
    }
    0
}

pub fn decode_with_recoveries(
    data: &[u8],
    label: &str,
    handler_label: &str,
) -> Result<(String, Vec<String>), String> {
    let codec = resolve_codec(label).ok_or_else(|| unsupported_encoding(label))?;
    let handler = decode_handler(handler_label)?;
    match codec {
        Codec::Utf8 => decode_utf8(data, handler),
        Codec::Utf8Sig => {
            let without_bom = data.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(data);
            decode_utf8(without_bom, handler)
        }
        Codec::Ascii => decode_ascii(data, handler),
        Codec::Latin1 => Ok((data.iter().map(|b| char::from(*b)).collect(), Vec::new())),
        Codec::Utf16Le => decode_utf16(data, handler, Endian::Little),
        Codec::Utf16Be => decode_utf16(data, handler, Endian::Big),
        Codec::Windows(encoding) => {
            let (decoded, had_errors) = encoding.decode_without_bom_handling(data);
            if had_errors && handler == Handler::Strict {
                return Err(format!(
                    "invalid byte sequence for {}",
                    codec.canonical_label()
                ));
            }
            let recoveries = if had_errors {
                vec![format!(
                    "{} decode recovered with {}",
                    codec.canonical_label(),
                    handler_label
                )]
            } else {
                Vec::new()
            };
            Ok((decoded.into_owned(), recoveries))
        }
    }
}

pub fn encode_with_recoveries(
    text: &str,
    label: &str,
    handler_label: &str,
) -> Result<(Vec<u8>, Vec<String>), String> {
    let codec = resolve_codec(label).ok_or_else(|| unsupported_encoding(label))?;
    let handler = encode_handler(handler_label)?;
    match codec {
        Codec::Utf8 => Ok((text.as_bytes().to_vec(), Vec::new())),
        Codec::Utf8Sig => {
            let mut bytes = vec![0xEF, 0xBB, 0xBF];
            bytes.extend_from_slice(text.as_bytes());
            Ok((bytes, Vec::new()))
        }
        Codec::Ascii => encode_ascii(text, handler),
        Codec::Latin1 => encode_latin1(text, handler),
        Codec::Utf16Le => Ok((encode_utf16(text, Endian::Little), Vec::new())),
        Codec::Utf16Be => Ok((encode_utf16(text, Endian::Big), Vec::new())),
        Codec::Windows(encoding) => {
            let (encoded, _, had_errors) = encoding.encode(text);
            if had_errors && handler == Handler::Strict {
                return Err(format!(
                    "text contains characters not representable in {}",
                    codec.canonical_label()
                ));
            }
            let recoveries = if had_errors {
                vec![format!(
                    "{} encode recovered with {}",
                    codec.canonical_label(),
                    handler_label
                )]
            } else {
                Vec::new()
            };
            Ok((encoded.into_owned(), recoveries))
        }
    }
}

fn decode_utf8(data: &[u8], handler: Handler) -> Result<(String, Vec<String>), String> {
    match std::str::from_utf8(data) {
        Ok(valid) => Ok((valid.to_string(), Vec::new())),
        Err(error) if handler == Handler::Strict => Err(format!("invalid utf-8: {error}")),
        Err(_) => recover_utf8(data, handler),
    }
}

fn recover_utf8(data: &[u8], handler: Handler) -> Result<(String, Vec<String>), String> {
    let mut out = String::new();
    let mut recoveries = Vec::new();
    let mut cursor = 0;
    while cursor < data.len() {
        match std::str::from_utf8(&data[cursor..]) {
            Ok(valid) => {
                out.push_str(valid);
                break;
            }
            Err(error) => {
                let valid_up_to = error.valid_up_to();
                if valid_up_to > 0 {
                    let valid = std::str::from_utf8(&data[cursor..cursor + valid_up_to])
                        .map_err(|e| e.to_string())?;
                    out.push_str(valid);
                    cursor += valid_up_to;
                    continue;
                }
                let invalid_len = error.error_len().unwrap_or(1);
                let end = cursor.saturating_add(invalid_len).min(data.len());
                append_decode_recovery(&mut out, &data[cursor..end], handler);
                recoveries.push(format!("invalid utf-8 bytes at offset {cursor}"));
                cursor = end;
            }
        }
    }
    Ok((out, recoveries))
}

fn decode_ascii(data: &[u8], handler: Handler) -> Result<(String, Vec<String>), String> {
    let mut out = String::new();
    let mut recoveries = Vec::new();
    for (index, byte) in data.iter().copied().enumerate() {
        if byte <= 0x7F {
            out.push(char::from(byte));
            continue;
        }
        if handler == Handler::Strict {
            return Err(format!("ascii byte out of range at offset {index}: {byte}"));
        }
        append_decode_recovery(&mut out, &[byte], handler);
        recoveries.push(format!("ascii byte out of range at offset {index}: {byte}"));
    }
    Ok((out, recoveries))
}

#[derive(Clone, Copy)]
enum Endian {
    Little,
    Big,
}

fn decode_utf16(
    data: &[u8],
    handler: Handler,
    endian: Endian,
) -> Result<(String, Vec<String>), String> {
    let mut units = Vec::with_capacity(data.len() / 2);
    let mut recoveries = Vec::new();
    let mut chunks = data.chunks_exact(2);
    for chunk in &mut chunks {
        let unit = match endian {
            Endian::Little => u16::from_le_bytes([chunk[0], chunk[1]]),
            Endian::Big => u16::from_be_bytes([chunk[0], chunk[1]]),
        };
        units.push(unit);
    }
    let remainder = chunks.remainder();
    if !remainder.is_empty() {
        if handler == Handler::Strict {
            return Err("utf-16 input has trailing byte".to_string());
        }
        recoveries.push("utf-16 input has trailing byte".to_string());
    }

    let mut out = String::new();
    for decoded in char::decode_utf16(units) {
        match decoded {
            Ok(ch) => out.push(ch),
            Err(error) if handler == Handler::Strict => {
                return Err(format!(
                    "invalid utf-16 surrogate: {}",
                    error.unpaired_surrogate()
                ));
            }
            Err(error) => {
                let unit = error.unpaired_surrogate();
                append_decode_recovery(&mut out, &unit.to_le_bytes(), handler);
                recoveries.push(format!("invalid utf-16 surrogate: {unit}"));
            }
        }
    }
    if !remainder.is_empty() {
        append_decode_recovery(&mut out, remainder, handler);
    }
    Ok((out, recoveries))
}

fn append_decode_recovery(out: &mut String, bytes: &[u8], handler: Handler) {
    match handler {
        Handler::Strict => {}
        Handler::Replace => out.push(char::REPLACEMENT_CHARACTER),
        Handler::Ignore => {}
        Handler::BackslashReplace => {
            for byte in bytes {
                let _ = write!(out, "\\x{byte:02x}");
            }
        }
        Handler::XmlCharRefReplace | Handler::NameReplace => out.push(char::REPLACEMENT_CHARACTER),
    }
}

fn encode_ascii(text: &str, handler: Handler) -> Result<(Vec<u8>, Vec<String>), String> {
    encode_single_byte(
        text,
        handler,
        |ch| {
            if ch.is_ascii() {
                Some(ch as u8)
            } else {
                None
            }
        },
        "ascii",
    )
}

fn encode_latin1(text: &str, handler: Handler) -> Result<(Vec<u8>, Vec<String>), String> {
    encode_single_byte(
        text,
        handler,
        |ch| {
            let code = u32::from(ch);
            u8::try_from(code).ok()
        },
        "latin-1",
    )
}

fn encode_single_byte<F>(
    text: &str,
    handler: Handler,
    mut encode_char: F,
    label: &str,
) -> Result<(Vec<u8>, Vec<String>), String>
where
    F: FnMut(char) -> Option<u8>,
{
    let mut out = Vec::new();
    let mut recoveries = Vec::new();
    for (index, ch) in text.char_indices() {
        if let Some(byte) = encode_char(ch) {
            out.push(byte);
            continue;
        }
        if handler == Handler::Strict {
            return Err(format!(
                "character U+{:04X} not representable in {label} at byte offset {index}",
                u32::from(ch)
            ));
        }
        append_encode_recovery(&mut out, ch, handler);
        recoveries.push(format!(
            "character U+{:04X} not representable in {label} at byte offset {index}",
            u32::from(ch)
        ));
    }
    Ok((out, recoveries))
}

fn append_encode_recovery(out: &mut Vec<u8>, ch: char, handler: Handler) {
    match handler {
        Handler::Strict => {}
        Handler::Replace => out.push(b'?'),
        Handler::Ignore => {}
        Handler::BackslashReplace => {
            out.extend_from_slice(format!("\\u{:04x}", u32::from(ch)).as_bytes());
        }
        Handler::XmlCharRefReplace => {
            out.extend_from_slice(format!("&#{};", u32::from(ch)).as_bytes());
        }
        Handler::NameReplace => {
            out.extend_from_slice(format!("\\N{{U+{:04X}}}", u32::from(ch)).as_bytes());
        }
    }
}

fn encode_utf16(text: &str, endian: Endian) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() * 2);
    for unit in text.encode_utf16() {
        let bytes = match endian {
            Endian::Little => unit.to_le_bytes(),
            Endian::Big => unit.to_be_bytes(),
        };
        out.extend_from_slice(&bytes);
    }
    out
}

impl Codec {
    fn canonical_label(self) -> &'static str {
        match self {
            Self::Utf8 => "utf-8",
            Self::Utf8Sig => "utf-8-sig",
            Self::Ascii => "ascii",
            Self::Latin1 => "latin-1",
            Self::Utf16Le => "utf-16-le",
            Self::Utf16Be => "utf-16-be",
            Self::Windows(encoding) => encoding.name(),
        }
    }
}

fn resolve_codec(label: &str) -> Option<Codec> {
    let normalized = normalize_label(label);
    match normalized.as_str() {
        "utf-8" | "utf8" | "u8" => Some(Codec::Utf8),
        "utf-8-sig" | "utf8-sig" => Some(Codec::Utf8Sig),
        "ascii" | "us-ascii" => Some(Codec::Ascii),
        "latin-1" | "latin1" | "iso-8859-1" | "iso8859-1" => Some(Codec::Latin1),
        "utf-16-le" | "utf16le" | "utf-16le" => Some(Codec::Utf16Le),
        "utf-16-be" | "utf16be" | "utf-16be" => Some(Codec::Utf16Be),
        "windows-1250" | "cp1250" => Some(Codec::Windows(encoding_rs::WINDOWS_1250)),
        "windows-1251" | "cp1251" => Some(Codec::Windows(encoding_rs::WINDOWS_1251)),
        "windows-1252" | "cp1252" => Some(Codec::Windows(encoding_rs::WINDOWS_1252)),
        "windows-1253" | "cp1253" => Some(Codec::Windows(encoding_rs::WINDOWS_1253)),
        "windows-1254" | "cp1254" => Some(Codec::Windows(encoding_rs::WINDOWS_1254)),
        "windows-1255" | "cp1255" => Some(Codec::Windows(encoding_rs::WINDOWS_1255)),
        "windows-1256" | "cp1256" => Some(Codec::Windows(encoding_rs::WINDOWS_1256)),
        "windows-1257" | "cp1257" => Some(Codec::Windows(encoding_rs::WINDOWS_1257)),
        "windows-1258" | "cp1258" => Some(Codec::Windows(encoding_rs::WINDOWS_1258)),
        _ => encoding_rs_label(&normalized),
    }
}

fn encoding_rs_label(normalized: &str) -> Option<Codec> {
    let encoding = encoding_rs::Encoding::for_label(normalized.as_bytes())?;
    let name = encoding.name();
    if matches!(
        name,
        "windows-1250"
            | "windows-1251"
            | "windows-1252"
            | "windows-1253"
            | "windows-1254"
            | "windows-1255"
            | "windows-1256"
            | "windows-1257"
            | "windows-1258"
    ) {
        return Some(Codec::Windows(encoding));
    }
    None
}

fn normalize_label(label: &str) -> String {
    label.trim().to_ascii_lowercase().replace('_', "-")
}

fn unsupported_encoding(label: &str) -> String {
    format!("unsupported encoding: {label}")
}

fn decode_handler(label: &str) -> Result<Handler, String> {
    let handler = normalize_label(label);
    match handler.as_str() {
        "strict" => Ok(Handler::Strict),
        "replace" => Ok(Handler::Replace),
        "ignore" => Ok(Handler::Ignore),
        "backslashreplace" | "backslash-replace" => Ok(Handler::BackslashReplace),
        "xmlcharrefreplace" | "xml-char-ref-replace" => {
            Err("xmlcharrefreplace is encode-only".to_string())
        }
        "namereplace" | "name-replace" => Err("namereplace is encode-only".to_string()),
        _ => Err(format!("unsupported decode error handler: {label}")),
    }
}

fn encode_handler(label: &str) -> Result<Handler, String> {
    let handler = normalize_label(label);
    match handler.as_str() {
        "strict" => Ok(Handler::Strict),
        "replace" => Ok(Handler::Replace),
        "ignore" => Ok(Handler::Ignore),
        "backslashreplace" | "backslash-replace" => Ok(Handler::BackslashReplace),
        "xmlcharrefreplace" | "xml-char-ref-replace" => Ok(Handler::XmlCharRefReplace),
        "namereplace" | "name-replace" => Ok(Handler::NameReplace),
        _ => Err(format!("unsupported encode error handler: {label}")),
    }
}
