use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterRewriteError {
    ZeroSlot,
    SlotOverflow,
}

impl fmt::Display for ParameterRewriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ZeroSlot => "PostgreSQL parameter slots start at $1",
            Self::SlotOverflow => "PostgreSQL parameter slot exceeds the supported range",
        })
    }
}

impl std::error::Error for ParameterRewriteError {}

/// Adds `slot_offset` to PostgreSQL `$n` parameters outside quoted text and comments.
///
/// Fragment composition uses this function before concatenation. Parameter metadata
/// stays zero-based; emitted PostgreSQL placeholders stay one-based.
pub fn rewrite_parameter_slots(
    source: &str,
    slot_offset: u32,
) -> Result<String, ParameterRewriteError> {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\'' => {
                let backslash_escapes = is_escape_string_prefix(bytes, cursor);
                copy_quoted(source, &mut output, &mut cursor, b'\'', backslash_escapes);
            }
            b'"' => copy_quoted(source, &mut output, &mut cursor, b'"', false),
            b'-' if bytes.get(cursor + 1) == Some(&b'-') => {
                copy_line_comment(source, &mut output, &mut cursor);
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                copy_block_comment(source, &mut output, &mut cursor);
            }
            b'$' => {
                if let Some((delimiter, end)) = dollar_quote_delimiter(source, cursor) {
                    copy_dollar_quote(source, &mut output, &mut cursor, delimiter, end);
                } else if bytes.get(cursor + 1).is_some_and(u8::is_ascii_digit) {
                    let start = cursor + 1;
                    let mut end = start;
                    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                        end += 1;
                    }
                    let slot = source[start..end]
                        .parse::<u32>()
                        .map_err(|_| ParameterRewriteError::SlotOverflow)?;
                    if slot == 0 {
                        return Err(ParameterRewriteError::ZeroSlot);
                    }
                    let shifted = slot
                        .checked_add(slot_offset)
                        .ok_or(ParameterRewriteError::SlotOverflow)?;
                    output.push('$');
                    output.push_str(&shifted.to_string());
                    cursor = end;
                } else {
                    copy_char(source, &mut output, &mut cursor);
                }
            }
            _ => copy_char(source, &mut output, &mut cursor),
        }
    }
    Ok(output)
}

fn copy_quoted(
    source: &str,
    output: &mut String,
    cursor: &mut usize,
    quote: u8,
    backslash_escapes: bool,
) {
    let bytes = source.as_bytes();
    copy_char(source, output, cursor);
    while *cursor < bytes.len() {
        let current = bytes[*cursor];
        copy_char(source, output, cursor);
        if backslash_escapes && current == b'\\' && *cursor < bytes.len() {
            copy_char(source, output, cursor);
        } else if current == quote {
            if bytes.get(*cursor) == Some(&quote) {
                copy_char(source, output, cursor);
            } else {
                break;
            }
        }
    }
}

fn is_escape_string_prefix(bytes: &[u8], quote: usize) -> bool {
    quote > 0
        && matches!(bytes[quote - 1], b'e' | b'E')
        && (quote == 1 || !bytes[quote - 2].is_ascii_alphanumeric() && bytes[quote - 2] != b'_')
}

fn copy_line_comment(source: &str, output: &mut String, cursor: &mut usize) {
    while *cursor < source.len() {
        let newline = source.as_bytes()[*cursor] == b'\n';
        copy_char(source, output, cursor);
        if newline {
            break;
        }
    }
}

fn copy_block_comment(source: &str, output: &mut String, cursor: &mut usize) {
    let bytes = source.as_bytes();
    let mut depth = 0_u32;
    while *cursor < bytes.len() {
        if bytes.get(*cursor) == Some(&b'/') && bytes.get(*cursor + 1) == Some(&b'*') {
            output.push_str("/*");
            *cursor += 2;
            depth = depth.saturating_add(1);
        } else if bytes.get(*cursor) == Some(&b'*') && bytes.get(*cursor + 1) == Some(&b'/') {
            output.push_str("*/");
            *cursor += 2;
            depth = depth.saturating_sub(1);
            if depth == 0 {
                break;
            }
        } else {
            copy_char(source, output, cursor);
        }
    }
}

fn dollar_quote_delimiter(source: &str, cursor: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    let mut end = cursor + 1;
    if bytes.get(end) == Some(&b'$') {
        return Some((&source[cursor..=end], end + 1));
    }
    let first = *bytes.get(end)?;
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return None;
    }
    end += 1;
    while bytes
        .get(end)
        .is_some_and(|value| value.is_ascii_alphanumeric() || *value == b'_')
    {
        end += 1;
    }
    (bytes.get(end) == Some(&b'$')).then(|| (&source[cursor..=end], end + 1))
}

fn copy_dollar_quote(
    source: &str,
    output: &mut String,
    cursor: &mut usize,
    delimiter: &str,
    content_start: usize,
) {
    output.push_str(delimiter);
    if let Some(relative) = source[content_start..].find(delimiter) {
        let end = content_start + relative + delimiter.len();
        output.push_str(&source[content_start..end]);
        *cursor = end;
    } else {
        output.push_str(&source[content_start..]);
        *cursor = source.len();
    }
}

fn copy_char(source: &str, output: &mut String, cursor: &mut usize) {
    let Some(value) = source[*cursor..].chars().next() else {
        return;
    };
    output.push(value);
    *cursor += value.len_utf8();
}
