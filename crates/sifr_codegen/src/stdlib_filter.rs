use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct TopLevelItem {
    name: String,
    header_line: String,
    code: String,
}

#[derive(Debug, Clone)]
enum TopLevelChunk {
    Item(TopLevelItem),
    OtherLine(String),
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SharedPreludeNeeds {
    pub(crate) needs_hashmap: bool,
    pub(crate) needs_hashset: bool,
    pub(crate) needs_vecdeque: bool,
    pub(crate) needs_file_handles: bool,
    pub(crate) provides_file_handle_struct: bool,
}

pub(crate) struct PreparedStdlibModule {
    pub(crate) stripped_code: String,
    pub(crate) shared_needs: SharedPreludeNeeds,
}

/// Strip per-module shared imports/infrastructure and return dependency flags.
pub(crate) fn collect_and_strip_shared_prelude(filtered: &str) -> PreparedStdlibModule {
    let mut shared_needs = SharedPreludeNeeds::default();
    let mut in_file_handle_block = false;
    let mut skip_next_blank = false;
    let mut skip_file_handle_continuation = false;
    let mut lines_out: Vec<&str> = Vec::new();

    for line in filtered.lines() {
        let t = line.trim();

        if line.contains("__SIFR_FILE_HANDLES") {
            shared_needs.needs_file_handles = true;
        }
        if t.starts_with("struct FileHandle {") {
            shared_needs.provides_file_handle_struct = true;
        }
        if t == "use std::collections::HashMap;" {
            shared_needs.needs_hashmap = true;
            continue;
        }
        if t == "use std::collections::HashSet;" {
            shared_needs.needs_hashset = true;
            continue;
        }
        if t == "use std::collections::VecDeque;" {
            shared_needs.needs_vecdeque = true;
            continue;
        }
        if t == "use std::sync::Mutex;" {
            continue;
        }

        // Skip file handle infrastructure block (SifrFileHandle enum)
        if t.starts_with("enum SifrFileHandle {") {
            in_file_handle_block = true;
            continue;
        }
        if in_file_handle_block {
            if t == "}" {
                in_file_handle_block = false;
                skip_next_blank = true;
            }
            continue;
        }
        // Skip __SIFR_FILE_HANDLES static declaration (multi-line)
        if t.starts_with("static __SIFR_FILE_HANDLES:") {
            skip_file_handle_continuation = true;
            skip_next_blank = true;
            continue;
        }
        // Skip __SIFR_GLOBAL_LOG_LEVEL static declaration (multi-line)
        if t.starts_with("static __SIFR_GLOBAL_LOG_LEVEL:") {
            skip_file_handle_continuation = true;
            skip_next_blank = true;
            continue;
        }
        if skip_file_handle_continuation {
            skip_file_handle_continuation = false;
            continue;
        }
        if skip_next_blank && t.is_empty() {
            skip_next_blank = false;
            continue;
        }
        skip_next_blank = false;
        lines_out.push(line);
    }

    PreparedStdlibModule {
        stripped_code: lines_out.join("\n"),
        shared_needs,
    }
}

/// Filter compiled Rust source code to only include top-level items whose names
/// are in the given set (or are transitively called by them).
pub(crate) fn filter_rust_code_to_needed(
    rust_code: &str,
    imported_names: &HashSet<String>,
) -> String {
    let chunks = parse_top_level_chunks(rust_code);
    let items: Vec<&TopLevelItem> = chunks
        .iter()
        .filter_map(|chunk| match chunk {
            TopLevelChunk::Item(item) => Some(item),
            TopLevelChunk::OtherLine(_) => None,
        })
        .collect();

    // Step 1: Build a dependency graph (which top-level items refer to which others).
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
    let item_names: HashSet<String> = items.iter().map(|item| item.name.clone()).collect();
    // Error types that are defined globally (not in stdlib preamble) - don't include them as deps.
    let global_types: HashSet<&str> = [
        "IOError",
        "ParseError",
        "ValueError",
        "TypeError",
        "RegexError",
        "KeyError",
        "IndexError",
        "AttributeError",
        "OverflowError",
        "ZeroDivisionError",
        "RuntimeError",
        "NotImplementedError",
        "Error",
        "JSONDecodeError",
        "TOMLDecodeError",
        "FileNotFoundError",
        "PermissionError",
        "FileExistsError",
        "IsADirectoryError",
        "NotADirectoryError",
        "DirectoryNotEmptyError",
    ]
    .iter()
    .cloned()
    .collect();
    for item in &items {
        let mut called = HashSet::new();
        for other_name in &item_names {
            if other_name == &item.name || global_types.contains(other_name.as_str()) {
                continue;
            }
            if item_references_name(&item.code, other_name) {
                called.insert(other_name.clone());
            }
        }
        // Multiple blocks with the same name (e.g., impl X + impl Display for X)
        // should contribute dependencies together.
        deps.entry(item.name.clone()).or_default().extend(called);
    }

    // Step 2: Compute transitive closure of required item names.
    let mut needed: HashSet<String> = imported_names.clone();
    let mut worklist: Vec<String> = imported_names.iter().cloned().collect();
    while let Some(name) = worklist.pop() {
        if let Some(called) = deps.get(&name) {
            for dep in called {
                if needed.insert(dep.clone()) {
                    worklist.push(dep.clone());
                }
            }
        }
    }

    // Step 3: Emit required items in original order, keeping non-item top-level lines.
    let mut result = String::new();
    for chunk in chunks {
        match chunk {
            TopLevelChunk::Item(item) => {
                if needed.contains(&item.name) {
                    result.push_str(&item.code);
                    result.push('\n');
                }
            }
            TopLevelChunk::OtherLine(line) => {
                result.push_str(&line);
                result.push('\n');
            }
        }
    }

    result
}

/// Strip top-level items from Rust source whose names are already in `emitted_items`.
/// Items that survive are added to `emitted_items` so subsequent calls can deduplicate further.
///
/// Uses composite keys to distinguish struct/fn definitions from impl blocks:
/// - `struct X` / `fn X` → key = "X"
/// - `impl X {` → key = "impl X"
/// - `impl Trait for X {` → key = "impl Trait for X"
///
/// The `skip_types` set contains type names (e.g., "IOError") for which ALL items
/// (struct, impl, trait impls) should be unconditionally stripped.
pub(crate) fn dedup_rust_items(
    rust_code: &str,
    emitted_items: &mut HashSet<String>,
    skip_types: &HashSet<String>,
) -> String {
    let chunks = parse_top_level_chunks(rust_code);
    let mut result = String::new();

    for chunk in chunks {
        match chunk {
            TopLevelChunk::Item(item) => {
                if skip_types.contains(&item.name) {
                    continue;
                }

                let trimmed = item.header_line.trim();
                let dedup_key = if trimmed.starts_with("impl") {
                    trimmed
                        .split('{')
                        .next()
                        .unwrap_or(trimmed)
                        .trim()
                        .to_string()
                } else {
                    item.name.clone()
                };

                if emitted_items.insert(dedup_key) {
                    result.push_str(&item.code);
                    result.push('\n');
                }
            }
            TopLevelChunk::OtherLine(line) => {
                result.push_str(&line);
                result.push('\n');
            }
        }
    }

    result
}

fn parse_top_level_chunks(rust_code: &str) -> Vec<TopLevelChunk> {
    let lines: Vec<&str> = rust_code.lines().collect();
    let mut chunks = Vec::new();
    let mut pending_attrs: Vec<String> = Vec::new();
    let mut idx = 0usize;

    while idx < lines.len() {
        let line = lines[idx];
        if line.trim().starts_with("#[") {
            pending_attrs.push(line.to_string());
            idx += 1;
            continue;
        }

        if let Some(name) = parse_top_level_item_name(line) {
            let mut item_lines: Vec<String> = std::mem::take(&mut pending_attrs);
            item_lines.push(line.to_string());
            let mut depth = brace_delta(line);
            let header_line = line.to_string();
            idx += 1;

            if depth > 0 {
                while idx < lines.len() {
                    let next_line = lines[idx];
                    item_lines.push(next_line.to_string());
                    depth += brace_delta(next_line);
                    idx += 1;
                    if depth <= 0 {
                        break;
                    }
                }
            }

            let mut code = item_lines.join("\n");
            code.push('\n');
            chunks.push(TopLevelChunk::Item(TopLevelItem {
                name,
                header_line,
                code,
            }));
            continue;
        }

        for attr in pending_attrs.drain(..) {
            if !attr.trim().is_empty() {
                chunks.push(TopLevelChunk::OtherLine(attr));
            }
        }
        if !line.trim().is_empty() {
            chunks.push(TopLevelChunk::OtherLine(line.to_string()));
        }
        idx += 1;
    }

    for attr in pending_attrs {
        if !attr.trim().is_empty() {
            chunks.push(TopLevelChunk::OtherLine(attr));
        }
    }

    chunks
}

fn item_references_name(code: &str, other_name: &str) -> bool {
    let patterns = [
        format!("{other_name}("),
        format!("-> {other_name}"),
        format!("{other_name} {{"),
        format!("{other_name}::"),
    ];

    patterns.iter().any(|pattern| {
        code.match_indices(pattern.as_str()).any(|(idx, _)| {
            if idx == 0 {
                return true;
            }
            let prev_char = code.as_bytes()[idx - 1] as char;
            !prev_char.is_alphanumeric() && prev_char != '_'
        })
    })
}

fn parse_top_level_item_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    // fn name( or fn name<T>(
    if let Some(rest) = trimmed.strip_prefix("fn ") {
        if let Some(lt) = rest.find('<') {
            let paren = rest.find('(');
            if paren.is_none() || lt < paren.unwrap() {
                return Some(rest[..lt].trim().to_string());
            }
        }
        if let Some(paren) = rest.find('(') {
            return Some(rest[..paren].trim().to_string());
        }
    }
    // const NAME:
    if let Some(rest) = trimmed.strip_prefix("const ") {
        if let Some(colon) = rest.find(':') {
            return Some(rest[..colon].trim().to_string());
        }
    }
    // struct Name
    if let Some(rest) = trimmed.strip_prefix("struct ") {
        let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next()?;
        return Some(name.to_string());
    }
    // impl Name or impl Display for Name
    if let Some(rest) = trimmed.strip_prefix("impl") {
        let rest = if rest.starts_with('<') {
            let mut depth = 0i32;
            let mut end = 0usize;
            for (i, ch) in rest.char_indices() {
                if ch == '<' {
                    depth += 1;
                }
                if ch == '>' {
                    depth -= 1;
                }
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            rest[end..].trim_start()
        } else {
            rest.trim_start()
        };
        if let Some(for_idx) = rest.find(" for ") {
            let after_for = &rest[for_idx + 5..];
            let after_for = after_for.trim_start_matches('&');
            let name = after_for
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .next()?;
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        let name = rest
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()?;
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

fn brace_delta(line: &str) -> i32 {
    let mut depth = 0i32;
    for ch in line.chars() {
        if ch == '{' {
            depth += 1;
        }
        if ch == '}' {
            depth -= 1;
        }
    }
    depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_keeps_transitive_dependencies_in_item_order() {
        let code = r#"
use std::collections::HashMap;

fn root() {
    helper();
}

fn helper() {
    leaf();
}

fn leaf() {}

fn unused() {}
"#;
        let imported = HashSet::from(["root".to_string()]);
        let filtered = filter_rust_code_to_needed(code, &imported);

        assert!(filtered.contains("use std::collections::HashMap;"));
        assert!(filtered.contains("fn root()"));
        assert!(filtered.contains("fn helper()"));
        assert!(filtered.contains("fn leaf()"));
        assert!(!filtered.contains("fn unused()"));
    }

    #[test]
    fn dedup_uses_impl_signature_keys() {
        let code = r#"
struct Item {}

impl Item {
    fn a(&self) {}
}

impl Item {
    fn b(&self) {}
}

impl std::fmt::Display for Item {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }
}
"#;
        let mut emitted = HashSet::new();
        let skip_types = HashSet::new();
        let once = dedup_rust_items(code, &mut emitted, &skip_types);
        let twice = dedup_rust_items(code, &mut emitted, &skip_types);

        assert!(once.contains("struct Item {}"));
        assert!(once.contains("impl Item {"));
        assert!(once.contains("impl std::fmt::Display for Item {"));
        assert!(twice.trim().is_empty());
    }

    #[test]
    fn collects_and_strips_shared_prelude_bits() {
        let input = r#"
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Mutex;

enum SifrFileHandle {
    Reader(std::fs::File),
}

static __SIFR_FILE_HANDLES: std::sync::OnceLock<
    Mutex<HashMap<i64, SifrFileHandle>>
> = std::sync::OnceLock::new();

struct FileHandle {
    _handle: i64,
}

fn keep_me() {
    let _ = __SIFR_FILE_HANDLES.get();
}
"#;
        let prepared = collect_and_strip_shared_prelude(input);
        assert!(prepared.shared_needs.needs_hashmap);
        assert!(prepared.shared_needs.needs_hashset);
        assert!(prepared.shared_needs.needs_vecdeque);
        assert!(prepared.shared_needs.needs_file_handles);
        assert!(prepared.shared_needs.provides_file_handle_struct);
        assert!(!prepared.stripped_code.contains("use std::collections::HashMap;"));
        assert!(!prepared.stripped_code.contains("enum SifrFileHandle {"));
        assert!(!prepared.stripped_code.contains("static __SIFR_FILE_HANDLES:"));
        assert!(prepared.stripped_code.contains("fn keep_me()"));
    }
}
