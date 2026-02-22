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

#[derive(Debug, Clone)]
struct StdlibIrItem {
    name: String,
    code: String,
    refs: HashSet<String>,
}

#[derive(Debug, Clone)]
enum StdlibIrChunk {
    Item(StdlibIrItem),
    OtherLine(String),
}

#[derive(Debug, Clone)]
struct StdlibIrFile {
    chunks: Vec<StdlibIrChunk>,
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

const GLOBAL_INFRA_TYPES: &[&str] = &[
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
];

/// Strip per-module shared imports/infrastructure and return dependency flags.
pub(crate) fn collect_and_strip_shared_prelude(filtered: &str) -> PreparedStdlibModule {
    let chunks = parse_top_level_chunks(filtered);
    let shared_needs = derive_shared_needs(filtered, &chunks);
    let mut in_file_handle_block = false;
    let mut skip_next_blank = false;
    let mut skip_file_handle_continuation = false;
    let mut lines_out: Vec<&str> = Vec::new();

    for line in filtered.lines() {
        let t = line.trim();
        if t == "use std::collections::HashMap;" {
            continue;
        }
        if t == "use std::collections::HashSet;" {
            continue;
        }
        if t == "use std::collections::VecDeque;" {
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
    let ir = parse_stdlib_ir_file(rust_code);
    let deps = deps_by_item_name(&ir);
    let needed = transitive_needed_items(imported_names, &deps);
    render_needed_ir_items(&ir, &needed)
}

/// Strip top-level items from Rust source whose names are already in `emitted_items`.
/// Items that survive are added to `emitted_items` so subsequent calls can deduplicate further.
///
/// Uses composite keys to distinguish struct/fn definitions from impl blocks:
/// - `struct X` / `fn X` → key = "X"
/// - `impl X {` → key = "impl X"
/// - `impl Trait for X {` → key = "impl Trait for X"
///
/// The `skip_types` set contains type names (e.g., "`IOError`") for which ALL items
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

fn parse_stdlib_ir_file(rust_code: &str) -> StdlibIrFile {
    let chunks = parse_top_level_chunks(rust_code);
    let item_names: HashSet<String> = chunks
        .iter()
        .filter_map(|chunk| match chunk {
            TopLevelChunk::Item(item) => Some(item.name.clone()),
            TopLevelChunk::OtherLine(_) => None,
        })
        .collect();
    let global_types: HashSet<&str> = GLOBAL_INFRA_TYPES.iter().copied().collect();

    let chunks = chunks
        .into_iter()
        .map(|chunk| match chunk {
            TopLevelChunk::Item(item) => {
                let refs = referenced_item_names(&item.code, &item_names, &item.name, &global_types);
                StdlibIrChunk::Item(StdlibIrItem {
                    name: item.name,
                    code: item.code,
                    refs,
                })
            }
            TopLevelChunk::OtherLine(line) => StdlibIrChunk::OtherLine(line),
        })
        .collect();
    StdlibIrFile { chunks }
}

fn deps_by_item_name(ir: &StdlibIrFile) -> HashMap<String, HashSet<String>> {
    let mut deps = HashMap::new();
    for chunk in &ir.chunks {
        if let StdlibIrChunk::Item(item) = chunk {
            // Multiple blocks with the same name (e.g., impl X + impl Display for X)
            // should contribute dependencies together.
            deps.entry(item.name.clone())
                .or_insert_with(HashSet::new)
                .extend(item.refs.iter().cloned());
        }
    }
    deps
}

fn transitive_needed_items(
    imported_names: &HashSet<String>,
    deps: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
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
    needed
}

fn render_needed_ir_items(ir: &StdlibIrFile, needed: &HashSet<String>) -> String {
    let mut result = String::new();
    for chunk in &ir.chunks {
        match chunk {
            StdlibIrChunk::Item(item) => {
                if needed.contains(&item.name) {
                    result.push_str(&item.code);
                    result.push('\n');
                }
            }
            StdlibIrChunk::OtherLine(line) => {
                result.push_str(line);
                result.push('\n');
            }
        }
    }
    result
}

fn derive_shared_needs(filtered: &str, chunks: &[TopLevelChunk]) -> SharedPreludeNeeds {
    let mut shared_needs = SharedPreludeNeeds::default();
    let tokens = tokenize_rust_like(filtered);
    for (idx, token) in tokens.iter().enumerate() {
        let ident = match token {
            RustToken::Ident(ident) => ident,
            RustToken::Sym(_) => continue,
        };
        match ident.as_str() {
            "__SIFR_FILE_HANDLES" => shared_needs.needs_file_handles = true,
            "HashMap" if is_reference_ident(&tokens, idx) => shared_needs.needs_hashmap = true,
            "HashSet" if is_reference_ident(&tokens, idx) => shared_needs.needs_hashset = true,
            "VecDeque" if is_reference_ident(&tokens, idx) => shared_needs.needs_vecdeque = true,
            _ => {}
        }
    }
    for chunk in chunks {
        if let TopLevelChunk::Item(item) = chunk {
            if item.name == "FileHandle" && item.header_line.trim().starts_with("struct FileHandle") {
                shared_needs.provides_file_handle_struct = true;
            }
        }
    }
    shared_needs
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

fn referenced_item_names(
    code: &str,
    item_names: &HashSet<String>,
    current_name: &str,
    global_types: &HashSet<&str>,
) -> HashSet<String> {
    let tokens = tokenize_rust_like(code);
    let mut refs = HashSet::new();
    for (idx, token) in tokens.iter().enumerate() {
        let ident = match token {
            RustToken::Ident(ident) => ident,
            RustToken::Sym(_) => continue,
        };
        if ident == current_name || global_types.contains(ident.as_str()) {
            continue;
        }
        if item_names.contains(ident) && is_reference_ident(&tokens, idx) {
            refs.insert(ident.clone());
        }
    }
    refs
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RustToken {
    Ident(String),
    Sym(String),
}

fn tokenize_rust_like(code: &str) -> Vec<RustToken> {
    let chars: Vec<char> = code.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut escape = false;

    while i < chars.len() {
        let ch = chars[i];

        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if ch == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if in_string {
            if escape {
                escape = false;
                i += 1;
                continue;
            }
            if ch == '\\' {
                escape = true;
                i += 1;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            in_block_comment = true;
            i += 2;
            continue;
        }
        if ch == '"' {
            in_string = true;
            i += 1;
            continue;
        }
        if is_char_literal_start(&chars, i) {
            i += char_literal_len(&chars, i);
            continue;
        }
        if ch == ':' && i + 1 < chars.len() && chars[i + 1] == ':' {
            out.push(RustToken::Sym("::".to_string()));
            i += 2;
            continue;
        }
        if ch == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
            out.push(RustToken::Sym("->".to_string()));
            i += 2;
            continue;
        }
        if ch == '_' || ch.is_ascii_alphabetic() {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i] == '_' || chars[i].is_ascii_alphanumeric()) {
                i += 1;
            }
            out.push(RustToken::Ident(chars[start..i].iter().collect()));
            continue;
        }
        if !ch.is_whitespace() {
            out.push(RustToken::Sym(ch.to_string()));
        }
        i += 1;
    }

    out
}

fn is_char_literal_start(chars: &[char], idx: usize) -> bool {
    if chars[idx] != '\'' {
        return false;
    }
    if idx + 2 >= chars.len() {
        return false;
    }
    if chars[idx + 1] == '\\' {
        idx + 3 < chars.len() && chars[idx + 3] == '\''
    } else {
        chars[idx + 2] == '\''
    }
}

fn char_literal_len(chars: &[char], idx: usize) -> usize {
    if chars[idx + 1] == '\\' { 4 } else { 3 }
}

fn is_reference_ident(tokens: &[RustToken], idx: usize) -> bool {
    let ident = match &tokens[idx] {
        RustToken::Ident(s) => s.as_str(),
        RustToken::Sym(_) => return false,
    };
    let prev = previous_token(tokens, idx);
    let next = next_token(tokens, idx);

    if next_is_sym(next, "(") || next_is_sym(next, "{") || next_is_sym(next, "::") {
        return true;
    }
    if prev_is_sym(prev, "->")
        || prev_is_sym(prev, ":")
        || prev_is_sym(prev, "::")
        || prev_is_ident(prev, "dyn")
        || prev_is_ident(prev, "impl")
        || prev_is_ident(prev, "for")
    {
        return true;
    }
    if prev_is_sym(prev, "=") && starts_with_uppercase(ident) {
        return true;
    }
    if is_all_caps(ident) {
        return true;
    }
    false
}

fn previous_token(tokens: &[RustToken], idx: usize) -> Option<&RustToken> {
    if idx == 0 {
        None
    } else {
        Some(&tokens[idx - 1])
    }
}

fn next_token(tokens: &[RustToken], idx: usize) -> Option<&RustToken> {
    tokens.get(idx + 1)
}

fn prev_is_ident(token: Option<&RustToken>, expected: &str) -> bool {
    matches!(token, Some(RustToken::Ident(v)) if v == expected)
}

fn prev_is_sym(token: Option<&RustToken>, expected: &str) -> bool {
    matches!(token, Some(RustToken::Sym(v)) if v == expected)
}

fn next_is_sym(token: Option<&RustToken>, expected: &str) -> bool {
    matches!(token, Some(RustToken::Sym(v)) if v == expected)
}

fn starts_with_uppercase(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

fn is_all_caps(s: &str) -> bool {
    let mut saw_alpha = false;
    for c in s.chars() {
        if c.is_ascii_alphabetic() {
            saw_alpha = true;
            if !c.is_ascii_uppercase() {
                return false;
            }
        }
    }
    saw_alpha
}

fn parse_top_level_item_name(line: &str) -> Option<String> {
    let trimmed = strip_visibility_prefix(line.trim());
    // [async|const|unsafe]* fn name( or fn name<T>(
    let fn_candidate = strip_fn_modifiers(trimmed);
    if let Some(rest) = fn_candidate.strip_prefix("fn ") {
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
    // static NAME:
    if let Some(rest) = trimmed.strip_prefix("static ") {
        let rest = rest.strip_prefix("mut ").unwrap_or(rest);
        if let Some(colon) = rest.find(':') {
            return Some(rest[..colon].trim().to_string());
        }
    }
    // struct Name
    if let Some(rest) = trimmed.strip_prefix("struct ") {
        let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next()?;
        return Some(name.to_string());
    }
    // type Alias =
    if let Some(rest) = trimmed.strip_prefix("type ") {
        if let Some(eq) = rest.find('=') {
            let name = rest[..eq].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    // enum Name
    if let Some(rest) = trimmed.strip_prefix("enum ") {
        let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next()?;
        return Some(name.to_string());
    }
    // trait Name
    if let Some(rest) = trimmed.strip_prefix("trait ") {
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

fn strip_visibility_prefix(s: &str) -> &str {
    if let Some(rest) = s.strip_prefix("pub(crate) ") {
        return rest;
    }
    if let Some(rest) = s.strip_prefix("pub ") {
        return rest;
    }
    s
}

fn strip_fn_modifiers(mut s: &str) -> &str {
    loop {
        if let Some(rest) = s.strip_prefix("async ") {
            s = rest;
            continue;
        }
        if let Some(rest) = s.strip_prefix("const ") {
            s = rest;
            continue;
        }
        if let Some(rest) = s.strip_prefix("unsafe ") {
            s = rest;
            continue;
        }
        break;
    }
    s
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
    fn filter_ignores_name_mentions_in_strings_and_comments() {
        let code = r#"
fn root() {
    let _ = "helper()";
    // helper()
    /* helper() */
}

fn helper() {}
"#;
        let imported = HashSet::from(["root".to_string()]);
        let filtered = filter_rust_code_to_needed(code, &imported);

        assert!(filtered.contains("fn root()"));
        assert!(!filtered.contains("fn helper()"));
    }

    #[test]
    fn filter_tracks_type_level_dependencies_via_identifiers() {
        let code = r#"
struct Node {}

fn root() -> Node {
    Node {}
}
"#;
        let imported = HashSet::from(["root".to_string()]);
        let filtered = filter_rust_code_to_needed(code, &imported);

        assert!(filtered.contains("fn root()"));
        assert!(filtered.contains("struct Node {}"));
    }

    #[test]
    fn filter_supports_enum_trait_static_and_pub_items() {
        let code = r#"
pub enum Mode {
    Fast,
}

pub trait Worker {
    fn run(&self) -> i64;
}

pub struct Job {}

impl Worker for Job {
    fn run(&self) -> i64 { JOB_COUNT }
}

pub static JOB_COUNT: i64 = 7;

pub fn root() -> Box<dyn Worker> {
    let _m = Mode::Fast;
    Box::new(Job {})
}
"#;
        let imported = HashSet::from(["root".to_string()]);
        let filtered = filter_rust_code_to_needed(code, &imported);

        assert!(filtered.contains("pub fn root()"));
        assert!(filtered.contains("pub enum Mode"));
        assert!(filtered.contains("pub trait Worker"));
        assert!(filtered.contains("pub struct Job"));
        assert!(filtered.contains("impl Worker for Job"));
        assert!(filtered.contains("pub static JOB_COUNT: i64 = 7;"));
    }

    #[test]
    fn filter_supports_async_const_unsafe_fn_and_static_mut() {
        let code = r#"
pub static mut COUNTER: i64 = 0;

pub const fn seed() -> i64 {
    COUNTER
}

pub unsafe fn tick() -> i64 {
    COUNTER + seed()
}

pub async fn root() -> i64 {
    tick()
}
"#;
        let imported = HashSet::from(["root".to_string()]);
        let filtered = filter_rust_code_to_needed(code, &imported);

        assert!(filtered.contains("pub async fn root()"));
        assert!(filtered.contains("pub unsafe fn tick()"));
        assert!(filtered.contains("pub const fn seed()"));
        assert!(filtered.contains("pub static mut COUNTER: i64 = 0;"));
    }

    #[test]
    fn filter_tracks_type_alias_dependencies_and_drops_unused_aliases() {
        let code = r#"
pub struct Node {}

pub type UsedAlias = Node;
pub type UnusedAlias = i64;

pub fn root() -> UsedAlias {
    Node {}
}
"#;
        let imported = HashSet::from(["root".to_string()]);
        let filtered = filter_rust_code_to_needed(code, &imported);

        assert!(filtered.contains("pub fn root() -> UsedAlias"));
        assert!(filtered.contains("pub type UsedAlias = Node;"));
        assert!(filtered.contains("pub struct Node {}"));
        assert!(!filtered.contains("pub type UnusedAlias = i64;"));
    }

    #[test]
    fn filter_avoids_false_positive_from_local_variable_name() {
        let code = r#"
pub fn root() -> i64 {
    let helper = 1;
    helper + 1
}

pub fn helper() -> i64 {
    2
}
"#;
        let imported = HashSet::from(["root".to_string()]);
        let filtered = filter_rust_code_to_needed(code, &imported);
        assert!(filtered.contains("pub fn root()"));
        assert!(!filtered.contains("pub fn helper()"));
    }

    #[test]
    fn filter_keeps_all_items_for_needed_type_name() {
        let code = r#"
pub struct Builder {}

impl Builder {
    pub fn new() -> Builder {
        Builder {}
    }
}

pub fn root() -> Builder {
    Builder::new()
}
"#;
        let imported = HashSet::from(["root".to_string()]);
        let filtered = filter_rust_code_to_needed(code, &imported);

        assert!(filtered.contains("pub fn root() -> Builder"));
        assert!(filtered.contains("pub struct Builder {}"));
        assert!(filtered.contains("impl Builder {"));
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

    #[test]
    fn shared_prelude_needs_ignore_comment_mentions() {
        let input = r#"
// HashMap HashSet VecDeque __SIFR_FILE_HANDLES
fn keep_me() {}
"#;
        let prepared = collect_and_strip_shared_prelude(input);
        assert!(!prepared.shared_needs.needs_hashmap);
        assert!(!prepared.shared_needs.needs_hashset);
        assert!(!prepared.shared_needs.needs_vecdeque);
        assert!(!prepared.shared_needs.needs_file_handles);
        assert!(prepared.stripped_code.contains("fn keep_me()"));
    }
}
