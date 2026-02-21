use std::collections::{HashMap, HashSet};

/// Filter compiled Rust source code to only include top-level items whose names
/// are in the given set (or are transitively called by them).
pub(crate) fn filter_rust_code_to_needed(
    rust_code: &str,
    imported_names: &HashSet<String>,
) -> String {
    // Step 1: Parse the Rust code into named blocks
    let blocks = parse_rust_blocks(rust_code);

    // Step 2: Build a dependency graph (which functions call/use which)
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
    let block_names: HashSet<String> = blocks.iter().map(|(name, _)| name.clone()).collect();
    // Error types that are defined globally (not in stdlib preamble) - don't include them as deps
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
    for (name, code) in &blocks {
        let mut called = HashSet::new();
        for other_name in &block_names {
            if other_name != name && !global_types.contains(other_name.as_str()) {
                // Check if this block's code contains a call to or type reference of other_name
                // Use word-boundary check to avoid substring false positives
                // Check for function calls: other_name(
                // Check for type references: -> TypeName, TypeName {, TypeName::
                let patterns = [
                    format!("{}(", other_name),
                    format!("-> {}", other_name),
                    format!("{} {{", other_name),
                    format!("{}::", other_name),
                ];
                for pattern in &patterns {
                    for (idx, _) in code.match_indices(pattern.as_str()) {
                        // Check that the character before the match is not alphanumeric or underscore
                        let is_word_boundary = if idx == 0 {
                            true
                        } else {
                            let prev_char = code.as_bytes()[idx - 1] as char;
                            !prev_char.is_alphanumeric() && prev_char != '_'
                        };
                        if is_word_boundary {
                            called.insert(other_name.clone());
                            break;
                        }
                    }
                }
            }
        }
        // Accumulate dependencies: multiple blocks with the same name
        // (e.g., impl X and impl Display for X) should contribute together.
        deps.entry(name.clone()).or_default().extend(called);
    }

    // Step 3: Compute transitive closure of needed functions
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

    // Step 4: Emit only needed blocks (preserving order) + non-block lines
    let mut result = String::new();
    let mut lines = rust_code.lines().peekable();
    // Buffer for attribute lines (#[...]) that precede an item
    let mut pending_attrs: Vec<String> = Vec::new();

    while let Some(line) = lines.next() {
        // Collect attribute lines to attach to the next item
        if line.trim().starts_with("#[") {
            pending_attrs.push(line.to_string());
            continue;
        }

        let item_name = extract_top_level_item_name(line);

        if let Some(ref name) = item_name {
            if needed.contains(name) {
                // Emit pending attributes
                for attr in &pending_attrs {
                    result.push_str(attr);
                    result.push('\n');
                }
                // Emit this entire item
                result.push_str(line);
                result.push('\n');
                let mut depth: i32 = count_braces(line);
                if depth > 0 {
                    while let Some(next_line) = lines.next() {
                        result.push_str(next_line);
                        result.push('\n');
                        depth += count_braces(next_line);
                        if depth <= 0 {
                            break;
                        }
                    }
                }
                result.push('\n');
            } else {
                // Skip this entire item (and its pending attributes)
                let mut depth: i32 = count_braces(line);
                if depth > 0 {
                    while let Some(next_line) = lines.next() {
                        depth += count_braces(next_line);
                        if depth <= 0 {
                            break;
                        }
                    }
                }
            }
            pending_attrs.clear();
        } else if line.trim().is_empty() {
            // Skip blank lines
            pending_attrs.clear();
        } else {
            // Non-item lines (use statements, comments) — always include
            // Also flush any pending attributes
            for attr in &pending_attrs {
                result.push_str(attr);
                result.push('\n');
            }
            pending_attrs.clear();
            result.push_str(line);
            result.push('\n');
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
    let mut result = String::new();
    let mut lines = rust_code.lines().peekable();
    let mut pending_attrs: Vec<String> = Vec::new();

    while let Some(line) = lines.next() {
        if line.trim().starts_with("#[") {
            pending_attrs.push(line.to_string());
            continue;
        }

        let item_name = extract_top_level_item_name(line);

        if let Some(ref name) = item_name {
            let mut item_lines = Vec::new();
            for attr in &pending_attrs {
                item_lines.push(attr.clone());
            }
            item_lines.push(line.to_string());
            let mut depth: i32 = count_braces(line);
            if depth > 0 {
                while let Some(next_line) = lines.next() {
                    item_lines.push(next_line.to_string());
                    depth += count_braces(next_line);
                    if depth <= 0 {
                        break;
                    }
                }
            }
            pending_attrs.clear();

            // If the type name is in skip_types, unconditionally strip everything related to it
            if skip_types.contains(name) {
                continue;
            }

            // Build a dedup key that distinguishes struct/fn from impl blocks
            let trimmed = line.trim();
            let dedup_key = if trimmed.starts_with("impl") {
                // Use a normalized impl signature as the key
                // Strip generic params for matching: "impl<T> X<T> {" → "impl X {"
                trimmed.split('{').next().unwrap_or(trimmed).trim().to_string()
            } else {
                name.clone()
            };

            if !emitted_items.contains(&dedup_key) {
                emitted_items.insert(dedup_key);
                for il in &item_lines {
                    result.push_str(il);
                    result.push('\n');
                }
                result.push('\n');
            }
        } else if line.trim().is_empty() {
            pending_attrs.clear();
        } else {
            for attr in &pending_attrs {
                result.push_str(attr);
                result.push('\n');
            }
            pending_attrs.clear();
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Parse Rust source into a list of (name, full_code) blocks for top-level items.
fn parse_rust_blocks(rust_code: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut lines = rust_code.lines().peekable();
    let mut pending_attrs = String::new();

    while let Some(line) = lines.next() {
        // Collect attribute lines
        if line.trim().starts_with("#[") {
            pending_attrs.push_str(line);
            pending_attrs.push('\n');
            continue;
        }
        if let Some(name) = extract_top_level_item_name(line) {
            let mut block_code = pending_attrs.clone();
            pending_attrs.clear();
            block_code.push_str(line);
            block_code.push('\n');
            let mut depth: i32 = count_braces(line);
            if depth > 0 {
                while let Some(next_line) = lines.next() {
                    block_code.push_str(next_line);
                    block_code.push('\n');
                    let delta = count_braces(next_line);
                    depth += delta;
                    if depth <= 0 {
                        break;
                    }
                }
            }
            blocks.push((name, block_code));
        } else {
            pending_attrs.clear();
        }
    }

    blocks
}

/// Extract the name of a top-level Rust item from a line, if it starts one.
fn extract_top_level_item_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    // fn name( or fn name<T>(
    if let Some(rest) = trimmed.strip_prefix("fn ") {
        // Check for generic params first: fn name<T: ...>(...)
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
        // Skip generic params: impl<T: Bound> ... → skip past the closing '>'
        let rest = if rest.starts_with('<') {
            let mut depth = 0i32;
            let mut end = 0;
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
        // "impl Display for Name {" or "impl std::ops::Add<&Name> for &Name {"
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
        // "impl Name {" or "Name<T> {"
        let name = rest
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()?;
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// Count net brace depth change in a line (opening braces minus closing braces).
fn count_braces(line: &str) -> i32 {
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
