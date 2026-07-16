use super::Renderer;
use sifr_type_system::COMPILER_RUST_PATH_ROOTS;

impl Renderer {
    /// Make compiler-owned external paths crate-absolute without rewriting
    /// string/character data or comments embedded in structured raw fragments.
    pub(crate) fn render_compiler_path_string(value: &str) -> String {
        let bytes = value.as_bytes();
        let mut rendered = String::with_capacity(value.len() + 2);
        let mut index = 0;
        while index < bytes.len() {
            if let Some(end) = protected_region_end(value, index) {
                rendered.push_str(&value[index..end]);
                index = end;
                continue;
            }

            let matched = COMPILER_RUST_PATH_ROOTS.iter().find(|root| {
                let root_bytes = root.as_bytes();
                let end = index + root_bytes.len();
                end + 1 < bytes.len()
                    && &bytes[index..end] == root_bytes
                    && bytes[end] == b':'
                    && bytes[end + 1] == b':'
                    && (index == 0
                        || (!bytes[index - 1].is_ascii_alphanumeric()
                            && bytes[index - 1] != b'_'
                            && bytes[index - 1] != b':'))
            });
            if let Some(root) = matched {
                rendered.push_str("::");
                rendered.push_str(root);
                index += root.len();
            } else {
                let next = value[index..]
                    .chars()
                    .next()
                    .expect("index remains on a UTF-8 character boundary");
                rendered.push(next);
                index += next.len_utf8();
            }
        }
        rendered
    }

    pub(crate) fn render_path_parts(parts: &[String]) -> String {
        Self::render_compiler_path_string(&parts.join("::"))
    }
}

fn protected_region_end(value: &str, index: usize) -> Option<usize> {
    let bytes = value.as_bytes();
    if bytes[index..].starts_with(b"//") {
        return Some(
            value[index..]
                .find('\n')
                .map_or(bytes.len(), |offset| index + offset),
        );
    }
    if bytes[index..].starts_with(b"/*") {
        return Some(block_comment_end(bytes, index));
    }
    if let Some(end) = raw_string_end(bytes, index) {
        return Some(end);
    }
    if bytes[index] == b'"' {
        return Some(quoted_end(bytes, index, b'"'));
    }
    if bytes[index] == b'b' && bytes.get(index + 1) == Some(&b'"') {
        return Some(quoted_end(bytes, index + 1, b'"'));
    }
    char_literal_end(value, index)
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
