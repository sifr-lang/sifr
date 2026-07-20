use super::Renderer;
use sifr_type_system::COMPILER_RUST_PATH_ROOTS;

impl Renderer {
    /// Make compiler-owned external paths crate-absolute without rewriting
    /// string/character data or comments embedded in structured raw fragments.
    pub(crate) fn render_compiler_path_string(value: &str) -> String {
        if !value.contains("::") {
            return value.to_string();
        }

        let bytes = value.as_bytes();
        let mut rendered = None;
        let mut copied = 0;
        let mut index = 0;
        while index < bytes.len() {
            if let Some(end) = protected_region_end(value, index) {
                index = end;
                continue;
            }

            if is_identifier_start(bytes, index) {
                let end = identifier_end(bytes, index);
                if bytes.get(end..end + 2) == Some(b"::")
                    && COMPILER_RUST_PATH_ROOTS.contains(&&value[index..end])
                {
                    let output =
                        rendered.get_or_insert_with(|| String::with_capacity(value.len() + 2));
                    output.push_str(&value[copied..index]);
                    output.push_str("::");
                    copied = index;
                }
                index = end;
                continue;
            }
            index += 1;
        }

        rendered.map_or_else(
            || value.to_string(),
            |mut output| {
                output.push_str(&value[copied..]);
                output
            },
        )
    }

    pub(crate) fn render_path_parts(parts: &[String]) -> String {
        let joined = parts.join("::");
        if parts
            .first()
            .is_some_and(|root| COMPILER_RUST_PATH_ROOTS.contains(&root.as_str()))
        {
            format!("::{joined}")
        } else {
            Self::render_compiler_path_string(&joined)
        }
    }

    pub(crate) fn render_identifier_or_compiler_path(value: &str) -> String {
        if value.contains("::") {
            Self::render_compiler_path_string(&Self::render_identifier(value))
        } else {
            Self::render_identifier(value)
        }
    }
}

fn is_identifier_start(bytes: &[u8], start: usize) -> bool {
    bytes
        .get(start)
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && (start == 0
            || (!bytes[start - 1].is_ascii_alphanumeric()
                && bytes[start - 1] != b'_'
                && bytes[start - 1] != b':'))
}

fn identifier_end(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while bytes
        .get(end)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        end += 1;
    }
    end
}

fn protected_region_end(value: &str, index: usize) -> Option<usize> {
    let bytes = value.as_bytes();
    match bytes[index] {
        b'/' if bytes.get(index + 1) == Some(&b'/') => Some(
            value[index..]
                .find('\n')
                .map_or(bytes.len(), |offset| index + offset),
        ),
        b'/' if bytes.get(index + 1) == Some(&b'*') => Some(block_comment_end(bytes, index)),
        b'r' => raw_string_end(bytes, index),
        b'b' => raw_string_end(bytes, index).or_else(|| {
            (bytes.get(index + 1) == Some(&b'"')).then(|| quoted_end(bytes, index + 1, b'"'))
        }),
        b'"' => Some(quoted_end(bytes, index, b'"')),
        b'\'' => char_literal_end(value, index),
        _ => None,
    }
}

fn block_comment_end(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1usize;
    let mut index = start + 2;
    while index + 1 < bytes.len() {
        match &bytes[index..index + 2] {
            b"/*" => {
                depth += 1;
                index += 2;
            }
            b"*/" => {
                depth -= 1;
                index += 2;
                if depth == 0 {
                    return index;
                }
            }
            _ => index += 1,
        }
    }
    bytes.len()
}

fn raw_string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut quote = start;
    if bytes.get(quote) == Some(&b'b') {
        quote += 1;
    }
    if bytes.get(quote) != Some(&b'r') {
        return None;
    }
    quote += 1;
    let hashes_start = quote;
    while bytes.get(quote) == Some(&b'#') {
        quote += 1;
    }
    if bytes.get(quote) != Some(&b'"') {
        return None;
    }
    let hash_count = quote - hashes_start;
    let mut index = quote + 1;
    while index < bytes.len() {
        if bytes[index] == b'"'
            && bytes
                .get(index + 1..index + 1 + hash_count)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Some(index + 1 + hash_count);
        }
        index += 1;
    }
    Some(bytes.len())
}

fn quoted_end(bytes: &[u8], quote: usize, delimiter: u8) -> usize {
    let mut escaped = false;
    let mut index = quote + 1;
    while index < bytes.len() {
        if escaped {
            escaped = false;
        } else if bytes[index] == b'\\' {
            escaped = true;
        } else if bytes[index] == delimiter {
            return index + 1;
        }
        index += 1;
    }
    bytes.len()
}

fn char_literal_end(value: &str, start: usize) -> Option<usize> {
    let bytes = value.as_bytes();
    let quote = if bytes[start] == b'\'' {
        start
    } else if bytes[start] == b'b' && bytes.get(start + 1) == Some(&b'\'') {
        start + 1
    } else {
        return None;
    };
    let content = quote + 1;
    if bytes.get(content) == Some(&b'\\') {
        let end = quoted_end(bytes, quote, b'\'');
        return (end < bytes.len() || bytes.last() == Some(&b'\'')).then_some(end);
    }
    let character = value.get(content..)?.chars().next()?;
    let closing = content + character.len_utf8();
    (bytes.get(closing) == Some(&b'\'')).then_some(closing + 1)
}
