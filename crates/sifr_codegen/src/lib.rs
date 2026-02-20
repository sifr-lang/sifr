//! Sifr Code Generation
//!
//! Translates the typed HIR into Rust source code.

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::format_push_string)]
#![allow(clippy::type_complexity)]
#![allow(clippy::option_map_or_none)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::while_let_loop)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::ref_option)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::iter_next_loop)]
#![allow(clippy::map_clone)]
#![allow(clippy::useless_format)]
#![allow(clippy::cloned_instead_of_copied)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_semicolon)]
#![allow(dead_code)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::while_let_on_iterator)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::inefficient_to_string)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::doc_link_with_quotes)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::if_not_else)]
#![allow(clippy::unnecessary_unwrap)]

mod rust_ir;
pub use rust_ir::*;
mod render;
pub use render::*;
mod preamble;
pub use preamble::*;
mod context;
pub use context::*;
mod lower_expr;
pub use lower_expr::*;
mod lower_stmt;
pub use lower_stmt::*;
mod lower_item;
pub use lower_item::*;
mod intrinsics;

use sifr_hir::*;
use sifr_type_system::{Type, ParamConvention};
use std::collections::{HashMap, HashSet};

/// Built-in error class names that the compiler provides.
const BUILTIN_ERROR_CLASSES: &[&str] = &[
    "Error", "IOError", "ParseError", "ValueError", "DivisionError",
    "KeyError", "JSONDecodeError", "TOMLDecodeError", "RegexError",
    "FileNotFoundError", "PermissionError", "FileExistsError",
    "IsADirectoryError", "NotADirectoryError", "DirectoryNotEmptyError",
    "OverflowError", "IndexError", "AttributeError", "TypeError",
    "ZeroDivisionError", "RuntimeError", "NotImplementedError",
];

const IO_ERROR_SUBCLASSES: &[&str] = &[
    "FileNotFoundError", "PermissionError", "FileExistsError",
    "IsADirectoryError", "NotADirectoryError", "DirectoryNotEmptyError",
];

/// Check if a built-in error class name is referenced in the generated Rust code.
/// Uses word-boundary-aware matching to avoid false positives like "EmailError" matching "Error".
/// Check if a type can be auto-formatted with `{}` (implements Display).
/// Used to determine if auto-generated Display impl is safe for a class field.
fn is_auto_display_type(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Float | Type::Bool | Type::Str | Type::None => true,
        Type::LiteralInt(_) | Type::LiteralBool(_) | Type::LiteralStr(_) => true,
        Type::Class { .. } => true, // Classes get auto-Display too
        Type::Newtype { .. } => true,
        // Union types map to Option<T> or Rust enum — neither implements Display
        _ => false,
    }
}

fn is_builtin_error_referenced(code: &str, error_name: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = code[start..].find(error_name) {
        let abs_pos = start + pos;
        let before_ok = if abs_pos == 0 {
            true
        } else {
            let ch = code.as_bytes()[abs_pos - 1];
            // Must not be preceded by an alphanumeric char or underscore
            !(ch.is_ascii_alphanumeric() || ch == b'_')
        };
        let after_pos = abs_pos + error_name.len();
        let after_ok = if after_pos >= code.len() {
            true
        } else {
            let ch = code.as_bytes()[after_pos];
            // Must not be followed by an alphanumeric char or underscore
            !(ch.is_ascii_alphanumeric() || ch == b'_')
        };
        if before_ok && after_ok {
            // Skip matches inside "std::error::Error" (the Rust trait)
            let prefix_end = abs_pos;
            let is_std_error = prefix_end >= 12 && &code[prefix_end - 12..prefix_end] == "std::error::";
            if !is_std_error {
                return true;
            }
        }
        start = abs_pos + error_name.len();
    }
    false
}

/// Result of code generation, including the Rust source and metadata.
pub struct CodegenResult {
    pub rust_source: String,
    pub used_stdlib_modules: HashSet<String>,
    pub used_intrinsic_modules: HashSet<String>,
    /// Map of constant_name -> (type, rust_name) for module-level constants
    pub constant_mappings: HashMap<String, (Type, String)>,
}

/// Generate Rust source code from a HIR module.
pub fn generate_rust(module: &HirModule) -> String {
    generate_rust_with_metadata(module).rust_source
}

/// Generate Rust source code for a test module (with #[test] attributes).
pub fn generate_rust_test(module: &HirModule) -> CodegenResult {
    let mut emitter = RustEmitter::new();
    emitter.test_mode = true;

    // First pass: collect all union types used in the module
    emitter.collect_union_types(module);

    // Detect recursive (self-referential) class fields that need Box<T>
    emitter.detect_recursive_fields(module);

    // Generate enum definitions for non-Option union types
    emitter.generate_enum_definitions();

    // Second pass: emit the actual code
    emitter.emit_module(module);

    let mut result = String::new();
    if emitter.needs_hashmap {
        result.push_str("use std::collections::HashMap;\n");
    }
    if emitter.needs_hashset {
        result.push_str("use std::collections::HashSet;\n");
    }
    if emitter.needs_vecdeque {
        result.push_str("use std::collections::VecDeque;\n");
    }
    if emitter.needs_bigint {
        result.push_str("use num_bigint::BigInt;\n");
    }
    if emitter.needs_hashmap || emitter.needs_hashset || emitter.needs_vecdeque || emitter.needs_bigint {
        result.push('\n');
    }
    if !emitter.enum_defs.is_empty() {
        result.push_str(&emitter.enum_defs);
        result.push('\n');
    }
    result.push_str(&emitter.output);

    CodegenResult {
        rust_source: result,
        used_stdlib_modules: emitter.used_stdlib_modules.clone(),
        used_intrinsic_modules: emitter.used_stdlib_modules,
        constant_mappings: emitter.module_constants,
    }
}

/// Compiled stdlib information for codegen.
/// Contains per-module Rust code and intrinsic name sets.
pub struct StdlibCode {
    /// Map of module_name -> compiled Rust source code for pure Sifr functions/constants
    pub module_rust_code: HashMap<String, String>,
    /// Map of module_name -> set of names that are intrinsic re-exports (from _sifr.*)
    pub intrinsic_names: HashMap<String, HashSet<String>>,
    /// Map of module_name -> (constant_name -> (type, rust_name)) for stdlib constants
    /// This allows user code to reference stdlib constants with the correct Rust names.
    pub module_constants: HashMap<String, HashMap<String, (Type, String)>>,
    /// Map of module_name -> (func_name -> (param_types_with_conventions, return_type))
    /// for pure Sifr stdlib functions. Used to emit correct borrow prefixes at call sites.
    pub func_signatures: HashMap<String, HashMap<String, (Vec<(Type, ParamConvention)>, Type)>>,
    /// Map of module_name -> set of transitive intrinsic module dependencies.
    /// E.g., sifr.secrets depends on _sifr.crypto, so when user imports sifr.secrets,
    /// the Cargo dependencies for _sifr.crypto (rand) must be included.
    pub transitive_deps: HashMap<String, HashSet<String>>,
    /// Map of module_name -> set of function names that are generators (contain yield).
    /// Used to emit .collect() when assigning generator results to list[T] in user code.
    pub generator_functions: HashMap<String, HashSet<String>>,
    /// Set of class names that have generic type parameters across all stdlib modules.
    pub generic_classes: HashSet<String>,
}

impl Default for StdlibCode {
    fn default() -> Self {
        Self {
            module_rust_code: HashMap::new(),
            intrinsic_names: HashMap::new(),
            module_constants: HashMap::new(),
            func_signatures: HashMap::new(),
            transitive_deps: HashMap::new(),
            generator_functions: HashMap::new(),
            generic_classes: HashSet::new(),
        }
    }
}

/// Returns the default parameter convention for a type.
/// Copy types (int, float, bool) are passed by value (Own).
/// Move types (str, list, dict, class, etc.) are passed by reference (Borrow).
fn default_param_convention(ty: &Type) -> ParamConvention {
    if ty.ownership() == sifr_type_system::OwnershipKind::Copy {
        ParamConvention::Own
    } else {
        ParamConvention::Borrow
    }
}

/// Filter compiled Rust source code to only include top-level items whose names
/// are in the given set (or are transitively called by them).
fn filter_rust_code_to_needed(rust_code: &str, imported_names: &HashSet<String>) -> String {
    // Step 1: Parse the Rust code into named blocks
    let blocks = parse_rust_blocks(rust_code);

    // Step 2: Build a dependency graph (which functions call/use which)
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
    let block_names: HashSet<String> = blocks.iter().map(|(name, _)| name.clone()).collect();
    // Error types that are defined globally (not in stdlib preamble) - don't include them as deps
    let global_types: HashSet<&str> = ["IOError", "ParseError", "ValueError", "TypeError",
        "RegexError", "KeyError", "IndexError", "AttributeError", "OverflowError",
        "ZeroDivisionError", "RuntimeError", "NotImplementedError", "Error",
        "JSONDecodeError", "TOMLDecodeError",
        "FileNotFoundError", "PermissionError", "FileExistsError",
        "IsADirectoryError", "NotADirectoryError", "DirectoryNotEmptyError",
    ].iter().cloned().collect();
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
        // Accumulate dependencies: multiple blocks with the same name (e.g., impl X and impl Display for X)
        // should contribute their dependencies together
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
                if ch == '<' { depth += 1; }
                if ch == '>' { depth -= 1; }
                if depth == 0 { end = i + 1; break; }
            }
            rest[end..].trim_start()
        } else {
            rest.trim_start()
        };
        // "impl Display for Name {" or "impl std::ops::Add<&Name> for &Name {"
        if let Some(for_idx) = rest.find(" for ") {
            let after_for = &rest[for_idx + 5..];
            let after_for = after_for.trim_start_matches('&');
            let name = after_for.split(|c: char| !c.is_alphanumeric() && c != '_').next()?;
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        // "impl Name {" or "Name<T> {"
        let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next()?;
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
        if ch == '{' { depth += 1; }
        if ch == '}' { depth -= 1; }
    }
    depth
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
fn dedup_rust_items(rust_code: &str, emitted_items: &mut HashSet<String>, skip_types: &HashSet<String>) -> String {
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
                let normalized = trimmed.split('{').next().unwrap_or(trimmed).trim().to_string();
                normalized
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

/// Generate Rust source code from a HIR module, returning metadata about stdlib usage.
pub fn generate_rust_with_metadata(module: &HirModule) -> CodegenResult {
    generate_rust_with_stdlib(module, &StdlibCode::default())
}

/// Generate Rust source code from a HIR module with compiled stdlib code.
pub fn generate_rust_with_stdlib(module: &HirModule, stdlib_code: &StdlibCode) -> CodegenResult {
    let mut emitter = RustEmitter::new();
    emitter.stdlib_intrinsic_names = stdlib_code.intrinsic_names.clone();
    // Register stdlib generic classes so user code skips explicit type annotations
    emitter.generic_classes.extend(stdlib_code.generic_classes.iter().cloned());

    // Pre-register stdlib constants and function signatures so user code can reference them correctly
    for import in &module.imports {
        if let Some(const_map) = stdlib_code.module_constants.get(&import.module) {
            for name in &import.names {
                if let Some((ty, rust_name)) = const_map.get(name) {
                    emitter.module_constants.insert(name.clone(), (ty.clone(), rust_name.clone()));
                }
            }
        }
        if let Some(sig_map) = stdlib_code.func_signatures.get(&import.module) {
            for name in &import.names {
                if let Some(sig) = sig_map.get(name) {
                    emitter.func_signatures.insert(name.clone(), sig.clone());
                }
                // Also load class method signatures (ClassName::method entries)
                let prefix = format!("{}::", name);
                for (key, sig) in sig_map.iter() {
                    if key.starts_with(&prefix) {
                        emitter.func_signatures.insert(key.clone(), sig.clone());
                    }
                }
            }
            // Load class method signatures for classes returned by imported functions.
            // This handles cases like `compile_flags` returning `Pattern` - we need
            // `Pattern::search` etc. to be available for correct borrow prefix emission.
            for (key, sig) in sig_map.iter() {
                if key.contains("::") && !emitter.func_signatures.contains_key(key) {
                    emitter.func_signatures.insert(key.clone(), sig.clone());
                }
            }
        }
        // Pre-register stdlib generator functions so .collect() is emitted at call sites
        if let Some(gen_set) = stdlib_code.generator_functions.get(&import.module) {
            for name in &import.names {
                if gen_set.contains(name) {
                    emitter.generator_functions.insert(name.clone());
                }
            }
        }
    }

    // First pass: collect all union types used in the module
    emitter.collect_union_types(module);

    // Detect recursive (self-referential) class fields that need Box<T>
    emitter.detect_recursive_fields(module);

    // Generate enum definitions for non-Option union types
    emitter.generate_enum_definitions();

    // Second pass: emit the actual code
    emitter.emit_module(module);

    // Build stdlib preamble first so we can check for error type references
    let mut stdlib_preamble = String::new();
    let mut emitted_modules: HashSet<String> = HashSet::new();
    let mut emitted_items: HashSet<String> = HashSet::new();
    // Types whose definitions are always provided by the infrastructure code (error types,
    // IO helpers). All items (struct, impl, fn) for these types are stripped from stdlib output.
    let mut infra_skip_types: HashSet<String> = HashSet::new();
    for &error_name in BUILTIN_ERROR_CLASSES {
        infra_skip_types.insert(error_name.to_string());
    }
    for &error_name in IO_ERROR_SUBCLASSES {
        infra_skip_types.insert(error_name.to_string());
    }
    infra_skip_types.insert("__io_err".to_string());
    let mut all_needed: Vec<String> = Vec::new();
    let mut stdlib_needs_hashmap = false;
    let mut stdlib_needs_hashset = false;
    let mut stdlib_needs_vecdeque = false;
    let mut stdlib_needs_file_handles = false;
    for module_name in &emitter.used_stdlib_modules {
        if let Some(deps) = stdlib_code.transitive_deps.get(module_name) {
            for dep in deps {
                if dep.starts_with("sifr.") && !all_needed.contains(dep) {
                    all_needed.push(dep.clone());
                }
            }
        }
        if !all_needed.contains(module_name) {
            all_needed.push(module_name.clone());
        }
    }
    for module_name in &all_needed {
        if emitted_modules.contains(module_name) {
            continue;
        }
        if let Some(rust_code) = stdlib_code.module_rust_code.get(module_name) {
            if !rust_code.is_empty() {
                let filtered = if let Some(imported_names) = emitter.imported_stdlib_names.get(module_name) {
                    let intrinsic_set = stdlib_code.intrinsic_names.get(module_name);
                    let pure_sifr_imports: HashSet<String> = imported_names.iter()
                        .filter(|name| !intrinsic_set.map_or(false, |iset| iset.contains(*name)))
                        .cloned()
                        .collect();
                    if pure_sifr_imports.is_empty() {
                        String::new()
                    } else {
                        let mut expanded_imports = pure_sifr_imports.clone();
                        if let Some(const_map) = stdlib_code.module_constants.get(module_name) {
                            for name in &pure_sifr_imports {
                                if const_map.contains_key(name) {
                                    expanded_imports.insert(format!("__const_{}", name));
                                }
                            }
                        }
                        filter_rust_code_to_needed(rust_code, &expanded_imports)
                    }
                } else {
                    rust_code.clone()
                };
                if !filtered.trim().is_empty() {
                    // Track and strip per-module use imports; they'll be emitted once at the top
                    if filtered.contains("use std::collections::HashMap;") {
                        stdlib_needs_hashmap = true;
                    }
                    if filtered.contains("use std::collections::HashSet;") {
                        stdlib_needs_hashset = true;
                    }
                    if filtered.contains("use std::collections::VecDeque;") {
                        stdlib_needs_vecdeque = true;
                    }
                    // Track if any stdlib module needs file handle infrastructure
                    if filtered.contains("__SIFR_FILE_HANDLES") {
                        stdlib_needs_file_handles = true;
                    }
                    // Strip per-module imports and file handle infrastructure (emitted once at top)
                    let stripped: String = {
                        let mut in_file_handle_block = false;
                        let mut skip_next_blank = false;
                        let mut skip_file_handle_continuation = false;
                        let mut lines_out: Vec<&str> = Vec::new();
                        for line in filtered.lines() {
                            let t = line.trim();
                            // Skip use imports
                            if t == "use std::collections::HashMap;"
                                || t == "use std::collections::HashSet;"
                                || t == "use std::collections::VecDeque;"
                                || t == "use std::sync::Mutex;"
                            {
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
                        lines_out.join("\n")
                    };
                    if !stripped.trim().is_empty() {
                        let deduped = dedup_rust_items(&stripped, &mut emitted_items, &infra_skip_types);
                        if !deduped.trim().is_empty() {
                            stdlib_preamble.push_str(&format!("// --- stdlib: {} ---\n", module_name));
                            stdlib_preamble.push_str(&deduped);
                            stdlib_preamble.push('\n');
                        }
                    }
                }
                emitted_modules.insert(module_name.clone());
            }
        }
    }

    // Now assemble result: imports, enums, IR-backed preamble items, stdlib preamble, main output.
    let needs_file_handles = emitter.needs_file_handles
        || stdlib_needs_file_handles
        || emitter.output.contains("__SIFR_FILE_HANDLES");
    let needs_logging = emitter.used_stdlib_modules.contains("sifr.logging")
        || emitter.used_stdlib_modules.contains("_sifr.logging")
        || emitter.output.contains("__SIFR_GLOBAL_LOG_LEVEL");

    // File handle infrastructure always relies on HashMap + Mutex.
    let needs_hashmap = emitter.needs_hashmap || stdlib_needs_hashmap || needs_file_handles;
    let needs_hashset = emitter.needs_hashset || stdlib_needs_hashset;
    let needs_vecdeque = emitter.needs_vecdeque || stdlib_needs_vecdeque;
    let needs_bigint = emitter.needs_bigint;

    let mut import_items: Vec<RustItem> = Vec::new();
    if needs_hashmap {
        import_items.push(RustItem::Use(vec![
            "std".to_string(),
            "collections".to_string(),
            "HashMap".to_string(),
        ]));
    }
    if needs_hashset {
        import_items.push(RustItem::Use(vec![
            "std".to_string(),
            "collections".to_string(),
            "HashSet".to_string(),
        ]));
    }
    if needs_vecdeque {
        import_items.push(RustItem::Use(vec![
            "std".to_string(),
            "collections".to_string(),
            "VecDeque".to_string(),
        ]));
    }
    if needs_bigint {
        import_items.push(RustItem::Use(vec![
            "num_bigint".to_string(),
            "BigInt".to_string(),
        ]));
    }
    if needs_file_handles || needs_logging {
        import_items.push(RustItem::Use(vec![
            "std".to_string(),
            "sync".to_string(),
            "Mutex".to_string(),
        ]));
    }

    let mut result = String::new();
    if !import_items.is_empty() {
        result.push_str(&render_items(&import_items));
        result.push('\n');
    }
    if !emitter.enum_defs.is_empty() {
        result.push_str(&emitter.enum_defs);
        result.push('\n');
    }

    // Emit built-in error class struct definitions for any that are referenced.
    // For now this remains a compatibility shim that scans generated code.
    let combined_code = format!("{}{}", stdlib_preamble, emitter.output);
    let user_defined_error_classes: HashSet<String> = module
        .classes
        .iter()
        .filter(|c| c.is_error_type)
        .map(|c| c.name.clone())
        .collect();
    let io_error_referenced = is_builtin_error_referenced(&combined_code, "IOError")
        || IO_ERROR_SUBCLASSES
            .iter()
            .any(|s| is_builtin_error_referenced(&combined_code, s))
        || needs_file_handles;

    let mut preamble_items: Vec<RustItem> = Vec::new();
    if io_error_referenced && !user_defined_error_classes.contains("IOError") {
        preamble_items.extend(build_io_error_items());
    }

    for &error_name in BUILTIN_ERROR_CLASSES {
        // Skip IOError and its subclasses (handled separately)
        if error_name == "IOError" || IO_ERROR_SUBCLASSES.contains(&error_name) {
            continue;
        }
        let is_referenced = is_builtin_error_referenced(&combined_code, error_name);
        if is_referenced && !user_defined_error_classes.contains(error_name) {
            let (extra_fields, defaults) = if error_name == "JSONDecodeError" || error_name == "TOMLDecodeError" {
                (
                    vec![
                        ("line".to_string(), sifr_type_to_rust_type(&Type::Int)),
                        ("column".to_string(), sifr_type_to_rust_type(&Type::Int)),
                    ],
                    vec![
                        ("line".to_string(), RustExpr::Literal(RustLiteral::Int(0))),
                        ("column".to_string(), RustExpr::Literal(RustLiteral::Int(0))),
                    ],
                )
            } else if error_name == "RegexError" {
                (
                    vec![("detail".to_string(), sifr_type_to_rust_type(&Type::Str))],
                    vec![(
                        "detail".to_string(),
                        RustExpr::RawCode("String::new()".to_string()),
                    )],
                )
            } else {
                (vec![], vec![])
            };
            preamble_items.extend(build_error_type_items(error_name, &extra_fields, &defaults));
        }
    }

    // Emit file handle global state if open() built-in or any file handle intrinsic is used.
    if needs_file_handles {
        preamble_items.extend(build_file_handle_infra_items());
        if !stdlib_preamble.contains("struct FileHandle {")
            && !emitter.output.contains("struct FileHandle {")
        {
            preamble_items.extend(build_file_handle_struct_items());
        }
    }

    // Emit global log level state if logging module is used.
    if needs_logging {
        preamble_items.extend(build_logging_items());
    }

    if !preamble_items.is_empty() {
        result.push_str(&render_items(&preamble_items));
        result.push('\n');
    }

    if !stdlib_preamble.is_empty() {
        result.push_str(&stdlib_preamble);
    }

    result.push_str(&emitter.output);

    // Add transitive dependencies from stdlib modules
    let mut all_used_modules = emitter.used_stdlib_modules.clone();
    for module_name in &emitter.used_stdlib_modules {
        if let Some(deps) = stdlib_code.transitive_deps.get(module_name) {
            all_used_modules.extend(deps.iter().cloned());
        }
    }

    CodegenResult {
        rust_source: result,
        used_stdlib_modules: all_used_modules.clone(),
        used_intrinsic_modules: emitter.used_stdlib_modules,
        constant_mappings: emitter.module_constants,
    }
}

/// Generate Rust source code for a multi-module project.
/// Returns a map of filename -> Rust source code.
pub fn generate_rust_multi(modules: &[(&str, &HirModule)]) -> HashMap<String, String> {
    let mut files = HashMap::new();

    for (module_name, module) in modules {
        let mut emitter = RustEmitter::new();
        // For non-main modules, enable pub mode
        if *module_name != "main" {
            emitter.pub_mode = true;
        }
        emitter.collect_union_types(module);
        emitter.generate_enum_definitions();
        emitter.emit_module(module);

        let mut result = String::new();

        // For non-main modules, add imports as `use` statements
        for import in &module.imports {
            for name in &import.names {
                // Check if this name has an alias
                if let Some((_, alias)) = import.aliases.iter().find(|(orig, _)| orig == name) {
                    result.push_str(&format!("use crate::{}::{} as {};\n", import.module, name, alias));
                } else {
                    result.push_str(&format!("use crate::{}::{};\n", import.module, name));
                }
            }
        }

        if emitter.needs_hashmap {
            result.push_str("use std::collections::HashMap;\n");
        }
        if emitter.needs_hashset {
            result.push_str("use std::collections::HashSet;\n");
        }
        if emitter.needs_vecdeque {
            result.push_str("use std::collections::VecDeque;\n");
        }
        if emitter.needs_bigint {
            result.push_str("use num_bigint::BigInt;\n");
        }
        if !result.is_empty() {
            result.push('\n');
        }
        if !emitter.enum_defs.is_empty() {
            result.push_str(&emitter.enum_defs);
            result.push('\n');
        }

        result.push_str(&emitter.output);

        files.insert(module_name.to_string(), result);
    }

    files
}

/// Generate a complete Rust project (Cargo.toml + main.rs content).
pub fn generate_project(module: &HirModule, project_name: &str) -> (String, String) {
    generate_project_with_deps(module, project_name, &HashSet::new())
}

/// Generate a complete Rust project with stdlib dependencies.
pub fn generate_project_with_deps(module: &HirModule, project_name: &str, stdlib_modules: &HashSet<String>) -> (String, String) {
    let mut cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"
"#
    );

    // Add dependencies based on used stdlib/intrinsic modules
    let mut deps = Vec::new();
    for module_name in stdlib_modules {
        match module_name.as_str() {
            "sifr.json" | "sifr.collections" | "_sifr.json" | "_sifr.collections" => {
                if !deps.contains(&"serde_json = \"1\"".to_string()) {
                    deps.push("serde_json = \"1\"".to_string());
                    deps.push("serde = { version = \"1\", features = [\"derive\"] }".to_string());
                }
            }
            "sifr.time" | "_sifr.time" => {
                if !deps.contains(&"chrono = \"0.4\"".to_string()) {
                    deps.push("chrono = \"0.4\"".to_string());
                }
            }
            "sifr.random" | "_sifr.crypto" => {
                if !deps.contains(&"rand = \"0.8\"".to_string()) {
                    deps.push("rand = \"0.8\"".to_string());
                }
                if !deps.contains(&"rand_distr = \"0.4\"".to_string()) {
                    deps.push("rand_distr = \"0.4\"".to_string());
                }
            }
            "sifr.uuid" | "_sifr.uuid" => {
                if !deps.contains(&"rand = \"0.8\"".to_string()) {
                    deps.push("rand = \"0.8\"".to_string());
                }
            }
            "sifr.re" | "_sifr.regex" => {
                if !deps.contains(&"regex = \"1\"".to_string()) {
                    deps.push("regex = \"1\"".to_string());
                }
            }
            "sifr.hash" | "sifr.hashlib" => {
                if !deps.contains(&"sha2 = \"0.10\"".to_string()) {
                    deps.push("sha2 = \"0.10\"".to_string());
                    deps.push("md5 = \"0.7\"".to_string());
                    deps.push("sha1 = \"0.10\"".to_string());
                    deps.push("blake2 = \"0.10\"".to_string());
                }
            }
            "sifr.encoding" | "sifr.base64" => {
                if !deps.contains(&"base64 = \"0.22\"".to_string()) {
                    deps.push("base64 = \"0.22\"".to_string());
                }
            }
            "sifr.tomllib" | "_sifr.toml" => {
                if !deps.contains(&"toml = \"0.8\"".to_string()) {
                    deps.push("toml = \"0.8\"".to_string());
                }
            }
            "sifr.datetime" | "_sifr.datetime" => {
                if !deps.contains(&"chrono = \"0.4\"".to_string()) {
                    deps.push("chrono = \"0.4\"".to_string());
                }
            }
            "sifr.gzip" | "sifr.zipfile" | "_sifr.compress" => {
                if !deps.contains(&"flate2 = \"1\"".to_string()) {
                    deps.push("flate2 = \"1\"".to_string());
                }
                if !deps.contains(&"zip = \"0.6\"".to_string()) {
                    deps.push("zip = \"0.6\"".to_string());
                }
            }
            "_bigint" => {
                if !deps.contains(&"num-bigint = \"0.4\"".to_string()) {
                    deps.push("num-bigint = \"0.4\"".to_string());
                    deps.push("num-traits = \"0.2\"".to_string());
                }
            }
            // sifr.io, sifr.env, sifr.os, sifr.math, sifr.test, sifr.bytes, sifr.sys,
            // sifr.subprocess, sifr.html, sifr.calendar, sifr.operator use only std library
            _ => {}
        }
    }

    if !deps.is_empty() {
        cargo_toml.push_str("\n[dependencies]\n");
        for dep in &deps {
            cargo_toml.push_str(dep);
            cargo_toml.push('\n');
        }
    }

    let main_rs = generate_rust(module);
    (cargo_toml, main_rs)
}

struct RustEmitter {
    output: String,
    indent: usize,
    needs_hashmap: bool,
    needs_hashset: bool,
    needs_file_handles: bool,
    needs_bigint: bool,
    needs_vecdeque: bool,
    /// Track union enum types that need to be defined (name -> member types)
    union_enums: HashMap<String, Vec<Type>>,
    /// Accumulated enum definitions to prepend
    enum_defs: String,
    /// The return type of the function currently being emitted
    current_return_type: Option<Type>,
    /// Set of variable names currently narrowed via `if let Some(...)` unwrap
    option_unwrapped_vars: HashSet<String>,
    /// Function signatures: name -> (param_types_with_conventions, return_type)
    func_signatures: HashMap<String, (Vec<(Type, ParamConvention)>, Type)>,
    /// Whether we're inside a loop that has an else clause
    in_loop_with_else: bool,
    /// Whether to emit `pub` on all top-level items (for module exports)
    pub_mode: bool,
    /// Set of variable names that are mutated in the current function body
    mutated_vars: HashSet<String>,
    /// Set of class names that have Display impl (via __str__ or error type)
    display_classes: HashSet<String>,
    /// Map from child class name -> (parent class name, set of parent field names)
    parent_fields: HashMap<String, (String, HashSet<String>)>,
    /// The class currently being emitted (for field access resolution)
    current_class_name: Option<String>,
    /// Set of stdlib/intrinsic modules used (for Cargo dependency injection)
    pub used_stdlib_modules: HashSet<String>,
    /// Set of intrinsic function names (for codegen dispatch)
    intrinsic_functions: HashSet<String>,
    /// Crates requested by intrinsic registry lowering.
    intrinsic_registry_crates: HashSet<String>,
    /// Whether to emit in test mode (#[test] on test_* functions, no main)
    test_mode: bool,
    /// Set of (class_name, field_name) pairs that are self-referential and need Box<T>
    recursive_fields: HashSet<(String, String)>,
    /// Map from class name -> ordered list of field names (for constructor arg mapping)
    class_field_order: HashMap<String, Vec<String>>,
    /// Map from nested function name -> list of captured variable (name, type) pairs
    /// Used to pass extra args at call sites for recursive+capturing nested functions
    nested_fn_captures: HashMap<String, Vec<(String, Type)>>,
    /// Map from module-level constant name -> (type, rust_name)
    /// For primitives: rust_name is the UPPERCASE const name
    /// For strings/complex: rust_name is __const_name() function call
    module_constants: HashMap<String, (Type, String)>,
    /// Set of class names that have generic type parameters
    generic_classes: HashSet<String>,
    /// Map of generic class name -> list of type parameter names (e.g., "Counter" -> ["T"])
    generic_class_params: HashMap<String, Vec<String>>,
    /// Set of parameter names that are borrowed (&T) in the current function.
    /// Used to emit dereference (*name) in comparisons where &String != String.
    borrowed_params: HashSet<String>,
    /// Set of parameter names that are mutably borrowed (&mut T) in the current function.
    /// Used to avoid double-borrowing: when a &mut param is passed to another &mut param,
    /// we must NOT emit `&mut name` (it's already &mut T); just pass `name` directly.
    mut_borrowed_params: HashSet<String>,
    /// Map of module_name -> set of names that are intrinsic re-exports (from _sifr.*)
    /// Used to distinguish intrinsic function calls from pure Sifr function calls
    stdlib_intrinsic_names: HashMap<String, HashSet<String>>,
    /// Set of function names that are generators (contain yield statements)
    /// Used to emit .collect() when assigning generator results to list[T]
    generator_functions: HashSet<String>,
    /// Map of module_name -> set of imported names (for filtering preamble to only used functions)
    imported_stdlib_names: HashMap<String, HashSet<String>>,
    /// Temporarily suppress .clone() on field access (for mutating method calls on self.field)
    suppress_field_clone: bool,
    /// Whether we're inside a generator closure (yield -> return Some(val))
    in_generator_closure: bool,
    /// Whether we're inside a Display::fmt implementation (for __str__ methods)
    /// Return statements in this context become write!(f, "{}", val) + return Ok(())
    in_display_impl: bool,
    /// Counter for generating unique try-block error enum names
    try_enum_counter: usize,
    /// Depth of try-block closures we're currently inside (for return statement handling)
    try_closure_depth: usize,
    /// Map from variable name -> Callable parameter (type, convention) list.
    /// Populated per-function from params and locals with Callable types.
    /// Used to emit correct &arg/&mut arg/arg for Callable-typed variable calls.
    callable_var_conventions: HashMap<String, Vec<(Type, ParamConvention)>>,
}

impl RustEmitter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            needs_hashmap: false,
            needs_hashset: false,
            needs_file_handles: false,
            needs_bigint: false,
            needs_vecdeque: false,
            union_enums: HashMap::new(),
            enum_defs: String::new(),
            current_return_type: None,
            option_unwrapped_vars: HashSet::new(),
            func_signatures: HashMap::new(),
            in_loop_with_else: false,
            pub_mode: false,
            mutated_vars: HashSet::new(),
            display_classes: HashSet::new(),
            parent_fields: HashMap::new(),
            current_class_name: None,
            used_stdlib_modules: HashSet::new(),
            intrinsic_functions: HashSet::new(),
            intrinsic_registry_crates: HashSet::new(),
            test_mode: false,
            recursive_fields: HashSet::new(),
            class_field_order: HashMap::new(),
            nested_fn_captures: HashMap::new(),
            module_constants: HashMap::new(),
            generic_classes: HashSet::new(),
            generic_class_params: HashMap::new(),
            borrowed_params: HashSet::new(),
            mut_borrowed_params: HashSet::new(),
            stdlib_intrinsic_names: HashMap::new(),
            generator_functions: HashSet::new(),
            imported_stdlib_names: HashMap::new(),
            suppress_field_clone: false,
            in_generator_closure: false,
            in_display_impl: false,
            try_enum_counter: 0,
            try_closure_depth: 0,
            callable_var_conventions: HashMap::new(),
        }
    }

    /// Check if the object expression is `self._data` inside the `deque` class.
    fn is_deque_data_field(&self, object: &HirExpr) -> bool {
        if self.current_class_name.as_deref() != Some("deque") {
            return false;
        }
        if let HirExpr::FieldAccess { object: inner, field, .. } = object {
            if field == "_data" {
                if let HirExpr::Name { name, .. } = inner.as_ref() {
                    return name == "self";
                }
            }
        }
        false
    }

    /// Check if a generic class needs Hash + Eq bounds on its type parameters.
    /// This is true when a type parameter is used as a HashMap key (dict field with TypeVar key).
    fn class_needs_hash_eq(class: &HirClass) -> bool {
        fn type_has_typevar_dict_key(ty: &Type) -> bool {
            match ty {
                Type::Dict(key, _) => matches!(key.as_ref(), Type::TypeVar(_)),
                Type::List(inner) => type_has_typevar_dict_key(inner),
                Type::Union(members) => members.iter().any(type_has_typevar_dict_key),
                _ => false,
            }
        }
        class.fields.iter().any(|(_, ty)| type_has_typevar_dict_key(ty))
    }

    /// Check if a generic function needs Hash + Eq bounds (uses TypeVar as dict key
    /// or returns a generic class that needs Hash + Eq).
    fn func_needs_hash_eq(func: &HirFunction) -> bool {
        fn type_has_typevar_dict_key(ty: &Type) -> bool {
            match ty {
                Type::Dict(key, _) => matches!(key.as_ref(), Type::TypeVar(_)),
                Type::List(inner) => type_has_typevar_dict_key(inner),
                Type::Union(members) => members.iter().any(type_has_typevar_dict_key),
                Type::Class { fields, .. } => fields.iter().any(|(_, t)| type_has_typevar_dict_key(t)),
                _ => false,
            }
        }
        // Check params
        if func.params.iter().any(|p| type_has_typevar_dict_key(&p.ty)) {
            return true;
        }
        // Check return type
        if type_has_typevar_dict_key(&func.return_type) {
            return true;
        }
        false
    }

    fn generic_bounds_for_class(class: &HirClass) -> String {
        if Self::class_needs_hash_eq(class) {
            "Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq".to_string()
        } else {
            "Clone + std::fmt::Display + PartialOrd".to_string()
        }
    }

    /// Convert a Type to its Rust representation, appending generic type params
    /// for classes that are known to be generic (e.g., Counter → Counter<T>).
    fn rust_type_with_generics(&self, ty: &Type) -> String {
        if let Type::Class { name, .. } = ty {
            if let Some(params) = self.generic_class_params.get(name) {
                return format!("{}<{}>", name, params.join(", "));
            }
        }
        ty.rust_type()
    }

    /// Detect self-referential class fields that need Box<T> wrapping.
    /// A field is recursive if its type directly or indirectly references the class being defined.
    fn detect_recursive_fields(&mut self, module: &HirModule) {
        for class in &module.classes {
            let field_names: Vec<String> = class.fields.iter().map(|(n, _)| n.clone()).collect();
            self.class_field_order.insert(class.name.clone(), field_names);
            for (field_name, field_ty) in &class.fields {
                if type_references_class(field_ty, &class.name) {
                    self.recursive_fields.insert((class.name.clone(), field_name.clone()));
                }
            }
            if !class.type_params.is_empty() {
                self.generic_classes.insert(class.name.clone());
                self.generic_class_params.insert(class.name.clone(), class.type_params.clone());
            }
        }
    }

    /// Collect all union types from the module that need enum definitions,
    /// and build a map of function signatures for call-site wrapping.
    fn collect_union_types(&mut self, module: &HirModule) {
        for func in &module.functions {
            // Record function signature with conventions
            let param_info: Vec<(Type, ParamConvention)> = func.params.iter()
                .map(|p| (p.ty.clone(), p.convention))
                .collect();
            self.func_signatures.insert(func.name.clone(), (param_info, func.return_type.clone()));

            // Track generator functions (contain yield statements)
            if body_contains_yield(&func.body) {
                self.generator_functions.insert(func.name.clone());
            }

            // Check params
            for param in &func.params {
                self.register_union_type(&param.ty);
            }
            // Check return type
            self.register_union_type(&func.return_type);
            // Check body statements
            self.collect_union_types_in_stmts(&func.body);
        }
        // Also scan class method bodies and register their signatures
        for class in &module.classes {
            for method in &class.methods {
                // Register method signature under ClassName::method_name
                let param_info: Vec<(Type, ParamConvention)> = method.params.iter()
                    .map(|p| (p.ty.clone(), p.convention))
                    .collect();
                self.func_signatures.insert(
                    format!("{}::{}", class.name, method.name),
                    (param_info, method.return_type.clone()),
                );

                for param in &method.params {
                    self.register_union_type(&param.ty);
                }
                self.register_union_type(&method.return_type);
                self.collect_union_types_in_stmts(&method.body);
            }
        }
    }

    fn collect_union_types_in_stmts(&mut self, stmts: &[HirStmt]) {
        for stmt in stmts {
            match stmt {
                HirStmt::Let { ty, .. } => self.register_union_type(ty),
                HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                    self.collect_union_types_in_stmts(then_body);
                    for (_, body) in elif_clauses {
                        self.collect_union_types_in_stmts(body);
                    }
                    if let Some(body) = else_body {
                        self.collect_union_types_in_stmts(body);
                    }
                }
                HirStmt::While { body, else_body, .. } => {
                    self.collect_union_types_in_stmts(body);
                    if let Some(eb) = else_body {
                        self.collect_union_types_in_stmts(eb);
                    }
                }
                HirStmt::For { body, else_body, .. } => {
                    self.collect_union_types_in_stmts(body);
                    if let Some(eb) = else_body {
                        self.collect_union_types_in_stmts(eb);
                    }
                }
                _ => {}
            }
        }
    }

    fn register_union_type(&mut self, ty: &Type) {
        if let Type::Union(members) = ty {
            // Skip Option<T> pattern (T | None with exactly 2 members)
            let non_none: Vec<&Type> = members.iter().filter(|m| !matches!(m, Type::None)).collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                return; // This maps to Option<T>, no enum needed
            }
            // Register the enum name and its member types
            let enum_name = ty.union_enum_name();
            self.union_enums.entry(enum_name).or_insert_with(|| members.clone());
        }
    }

    /// Generate Rust enum definitions for all collected union types.
    fn generate_enum_definitions(&mut self) {
        // Sort enum names for deterministic output
        let mut enums: Vec<(String, Vec<Type>)> = self.union_enums.clone().into_iter().collect();
        enums.sort_by(|a, b| a.0.cmp(&b.0));

        for (enum_name, members) in &enums {
            // Generate the enum definition
            self.enum_defs.push_str(&format!("#[derive(Debug, Clone)]\n"));
            self.enum_defs.push_str(&format!("enum {} {{\n", enum_name));
            for member in members {
                let variant = member.union_variant_name();
                let rust_ty = member.rust_type();
                self.enum_defs.push_str(&format!("    {}({}),\n", variant, rust_ty));
            }
            self.enum_defs.push_str("}\n\n");

            // Generate Display impl so println!("{}", x) works
            self.enum_defs.push_str(&format!("impl std::fmt::Display for {} {{\n", enum_name));
            self.enum_defs.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            self.enum_defs.push_str("        match self {\n");
            for member in members {
                let variant = member.union_variant_name();
                // Use {:?} for class types (they derive Debug, not Display)
                let fmt_spec = if matches!(member, Type::Class { .. }) { "{:?}" } else { "{}" };
                self.enum_defs.push_str(&format!(
                    "            {}::{}(v) => write!(f, \"{}\", v),\n",
                    enum_name, variant, fmt_spec
                ));
            }
            self.enum_defs.push_str("        }\n");
            self.enum_defs.push_str("    }\n");
            self.enum_defs.push_str("}\n\n");
        }
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn emit_module(&mut self, module: &HirModule) {
        // Pre-scan: detect bigint usage
        if module_uses_bigint(module) {
            self.needs_bigint = true;
        }

        // Pre-scan: collect stdlib/intrinsic imports and register function names
        for import in &module.imports {
            if import.module.starts_with("sifr.") || import.module.starts_with("_sifr.") {
                self.used_stdlib_modules.insert(import.module.clone());
                // Track which specific names are imported from each stdlib module
                let names_set = self.imported_stdlib_names.entry(import.module.clone()).or_default();
                for name in &import.names {
                    names_set.insert(name.clone());
                }
                // Only register names as intrinsic if they are known intrinsic re-exports.
                // Pure Sifr functions/constants should go through the normal codegen path.
                let intrinsic_set = self.stdlib_intrinsic_names.get(&import.module);
                for name in &import.names {
                    if import.module.starts_with("_sifr.") {
                        // Direct _sifr.* imports are always intrinsic
                        self.intrinsic_functions.insert(name.clone());
                    } else if let Some(iset) = intrinsic_set {
                        // For sifr.* imports, only register if the name is an intrinsic re-export
                        if iset.contains(name) {
                            self.intrinsic_functions.insert(name.clone());
                        }
                    } else {
                        // No intrinsic info available (legacy path) — treat all as intrinsic
                        self.intrinsic_functions.insert(name.clone());
                    }
                }
            }
        }

        // Pre-scan: collect classes that have Display impls
        for class in &module.classes {
            let has_auto_display = !class.fields.is_empty()
                && !class.is_protocol
                && !class.operator_impls.iter().any(|(n, _)| n == "__str__" || n == "__repr__")
                && class.fields.iter().all(|(_, t)| is_auto_display_type(t));
            if class.is_error_type
                || class.newtype_inner.is_some()
                || class.operator_impls.iter().any(|(n, _)| n == "__str__" || n == "__repr__")
                || has_auto_display
            {
                self.display_classes.insert(class.name.clone());
            }
        }
        // Built-in error types all have Display impls (formatting self.message)
        for &error_name in BUILTIN_ERROR_CLASSES {
            self.display_classes.insert(error_name.to_string());
        }

        // Pre-scan: collect parent field info for inheritance
        for class in &module.classes {
            if let Some(ref parent_name) = class.parent_class {
                // Find the parent class and collect its field names
                if let Some(parent_class) = module.classes.iter().find(|c| c.name == *parent_name) {
                    let parent_field_names: HashSet<String> = parent_class.fields.iter()
                        .map(|(name, _)| name.clone())
                        .collect();
                    self.parent_fields.insert(
                        class.name.clone(),
                        (parent_name.clone(), parent_field_names),
                    );
                }
            }
        }

        // Emit module-level constants and register them for name resolution
        for (name, ty, value) in &module.constants {
            self.write_indent();
            // Use const for primitives, static for strings
            match ty {
                Type::Int | Type::Float | Type::Bool => {
                    let rust_name = name.to_uppercase();
                    self.write(&format!("const {}: {} = ", rust_name, ty.rust_type()));
                    self.emit_expr(value);
                    self.write(";\n");
                    self.module_constants.insert(name.clone(), (ty.clone(), rust_name));
                }
                Type::Str => {
                    let rust_name = format!("__const_{}", name);
                    self.write(&format!("fn {}() -> String {{ ", rust_name));
                    self.emit_expr(value);
                    self.write(".to_string() }\n");
                    self.module_constants.insert(name.clone(), (ty.clone(), format!("{}()", rust_name)));
                }
                _ => {
                    let rust_name = format!("__const_{}", name);
                    self.write(&format!("fn {}() -> {} {{ ", rust_name, ty.rust_type()));
                    self.emit_expr(value);
                    self.write(" }\n");
                    self.module_constants.insert(name.clone(), (ty.clone(), format!("{}()", rust_name)));
                }
            }
        }
        if !module.constants.is_empty() {
            self.output.push('\n');
        }

        // Emit class definitions first (structs + impls)
        for class in &module.classes {
            self.emit_class(class, module);
            self.output.push('\n');
        }

        for (i, func) in module.functions.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.emit_function(func);
        }
    }

    fn emit_class(&mut self, class: &HirClass, module: &HirModule) {
        // --- Protocol: emit trait definition ---
        if class.is_protocol {
            self.emit_protocol_trait(class);
            return;
        }

        // --- Enum: emit Rust enum with repr(i64) ---
        if class.is_enum {
            self.emit_enum_class(class);
            return;
        }

        // --- Newtype: emit tuple struct ---
        if let Some(ref inner) = class.newtype_inner {
            self.emit_newtype(class, inner);
            return;
        }

        // Check if class defines __eq__ (don't auto-derive PartialEq)
        let has_custom_eq = class.operator_impls.iter().any(|(n, _)| n == "__eq__");
        let has_custom_str = class.operator_impls.iter().any(|(n, _)| n == "__str__");

        // Check if any field is a Callable type (Box<dyn Fn> doesn't implement Debug/Clone/PartialEq)
        let has_callable_field = class.fields.iter().any(|(_, t)| matches!(t, Type::Callable(..)));

        // Derive attributes
        self.write_indent();
        if has_callable_field {
            // Callable fields (Box<dyn Fn>) don't implement Debug, Clone, or PartialEq
            self.write("#[derive()]\n");
        } else if has_custom_eq {
            // Don't derive PartialEq if custom __eq__ is defined
            self.write("#[derive(Debug, Clone)]\n");
        } else if class.is_hashable {
            self.write("#[derive(Debug, Clone, PartialEq, Eq, Hash)]\n");
        } else {
            self.write("#[derive(Debug, Clone, PartialEq)]\n");
        }

        // Struct definition
        self.write_indent();
        if self.pub_mode {
            self.write("pub struct ");
        } else {
            self.write("struct ");
        }
        self.write(&class.name);
        let class_bounds = Self::generic_bounds_for_class(class);
        if !class.type_params.is_empty() {
            self.write("<");
            for (i, tp) in class.type_params.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(&format!("{}: {}", tp, class_bounds));
            }
            self.write(">");
        }
        self.write(" {\n");
        self.indent += 1;

        // If this class has a parent, embed the parent struct as a field
        if let Some(ref parent) = class.parent_class {
            self.write_indent();
            if self.pub_mode {
                self.write("pub ");
            }
            let parent_field = parent.to_lowercase();
            self.write(&parent_field);
            self.write(": ");
            self.write(parent);
            self.write(",\n");
        }

        // Emit own fields (skip fields that come from the parent)
        for (field_name, field_ty) in &class.fields {
            // Skip parent-inherited fields (they're accessed via the embedded parent struct)
            if class.parent_class.is_some() {
                // We'll emit all fields listed in class.fields since the lowering
                // should only put the child's own fields here
            }
            self.write_indent();
            if self.pub_mode {
                self.write("pub ");
            }
            self.write(field_name);
            self.write(": ");
            let is_recursive = self.recursive_fields.contains(&(class.name.clone(), field_name.clone()));
            if is_recursive {
                self.write(&recursive_field_rust_type(field_ty, &class.name));
            } else if class.name == "deque" && field_name == "_data" {
                if let Type::List(elem) = field_ty {
                    self.needs_vecdeque = true;
                    self.write(&format!("VecDeque<{}>", elem.rust_type()));
                } else {
                    self.write(&field_ty.rust_type_for_struct_field());
                }
            } else {
                self.write(&field_ty.rust_type_for_struct_field());
            }
            self.write(",\n");
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // Impl block
        self.write_indent();
        self.write("impl");
        if !class.type_params.is_empty() {
            self.write("<");
            for (i, tp) in class.type_params.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(&format!("{}: {}", tp, class_bounds));
            }
            self.write(">");
        }
        self.write(" ");
        self.write(&class.name);
        if !class.type_params.is_empty() {
            self.write("<");
            for (i, tp) in class.type_params.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(tp);
            }
            self.write(">");
        }
        self.write(" {\n");
        self.indent += 1;

        // If no explicit constructor (no "new" method), generate a default one from fields
        let has_constructor = class.methods.iter().any(|m| m.name == "new");
        if !has_constructor && !class.fields.is_empty() {
            self.write_indent();
            if self.pub_mode {
                self.write("pub fn new(");
            } else {
                self.write("fn new(");
            }
            for (i, (field_name, field_ty)) in class.fields.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(field_name);
                self.write(": ");
                let is_recursive = self.recursive_fields.contains(&(class.name.clone(), field_name.clone()));
                if is_recursive {
                    self.write(&recursive_field_rust_type(field_ty, &class.name));
                } else {
                    self.write(&field_ty.rust_type());
                }
            }
            self.write(") -> Self {\n");
            self.indent += 1;
            self.write_indent();
            self.write("Self {\n");
            self.indent += 1;
            for (field_name, _) in &class.fields {
                self.write_indent();
                self.write(field_name);
                self.write(",\n");
            }
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n\n");
        }

        self.current_class_name = Some(class.name.clone());
        for method in &class.methods {
            self.emit_class_method(method, class);
            self.output.push('\n');
        }
        self.current_class_name = None;

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");

        // --- Emit operator trait impls ---
        self.emit_operator_impls(class);

        // For error types, implement Display and Error traits
        if class.is_error_type {
            self.output.push('\n');
            self.write_indent();
            self.write("impl std::fmt::Display for ");
            self.write(&class.name);
            self.write(" {\n");
            self.indent += 1;
            self.write_indent();
            self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            self.indent += 1;
            // If there's a 'message' field, use it for Display
            if class.fields.iter().any(|(name, _)| name == "message") {
                self.write_indent();
                self.write("write!(f, \"{}\", self.message)\n");
            } else {
                // Use Debug format as fallback
                self.write_indent();
                self.write("write!(f, \"{:?}\", self)\n");
            }
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n\n");

            self.write_indent();
            self.write("impl std::error::Error for ");
            self.write(&class.name);
            self.write(" {}\n");
        } else if has_custom_str && !class.is_error_type {
            // __str__ maps to Display (only if not error type, which already has Display)
            // The __str__ body is emitted inside the Display impl
            if let Some((_, str_func)) = class.operator_impls.iter().find(|(n, _)| n == "__str__") {
                self.output.push('\n');
                self.write_indent();
                self.write("impl std::fmt::Display for ");
                self.write(&class.name);
                self.write(" {\n");
                self.indent += 1;
                self.write_indent();
                self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
                self.indent += 1;
                // Emit the body of __str__ but wrap the return value in write!(f, "{}", ...)
                // For simplicity, if the body is a single Return, emit write!(f, "{}", return_expr)
                if str_func.body.len() == 1 {
                    if let Some(HirStmt::Return { value: Some(ref ret_expr) }) = str_func.body.first() {
                        self.write_indent();
                        self.write("write!(f, \"{}\", ");
                        self.emit_expr(ret_expr);
                        self.write(")\n");
                    } else {
                        // Single non-return statement
                        let saved = self.in_display_impl;
                        self.in_display_impl = true;
                        let saved_mutated = std::mem::take(&mut self.mutated_vars);
                        self.mutated_vars = collect_mutated_vars_with_sigs(&str_func.body, &self.func_signatures);
                        for stmt in &str_func.body {
                            self.emit_stmt(stmt);
                        }
                        self.mutated_vars = saved_mutated;
                        self.in_display_impl = saved;
                        self.write_indent();
                        self.write("Ok(())\n");
                    }
                } else {
                    // Multi-statement body: emit with in_display_impl flag
                    let saved = self.in_display_impl;
                    self.in_display_impl = true;
                    // Pre-scan for mutated variables so let mut is emitted correctly
                    let saved_mutated = std::mem::take(&mut self.mutated_vars);
                    self.mutated_vars = collect_mutated_vars_with_sigs(&str_func.body, &self.func_signatures);
                    for stmt in &str_func.body {
                        self.emit_stmt(stmt);
                    }
                    self.mutated_vars = saved_mutated;
                    self.in_display_impl = saved;
                    self.write_indent();
                    self.write("Ok(())\n");
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
        } else if !has_custom_str && !class.is_error_type && !class.fields.is_empty()
            && class.fields.iter().all(|(_, t)| is_auto_display_type(t))
        {
            // Auto-generate Display impl: ClassName(field1=value1, field2=value2)
            self.output.push('\n');
            self.write_indent();
            self.write("impl std::fmt::Display for ");
            self.write(&class.name);
            if !class.type_params.is_empty() {
                self.write("<");
                for (i, tp) in class.type_params.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.write(tp);
                }
                self.write(">");
            }
            self.write(" {\n");
            self.indent += 1;
            self.write_indent();
            self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            self.indent += 1;
            self.write_indent();
            self.write("write!(f, \"");
            self.write(&class.name);
            self.write("(");
            for (i, (field_name, _)) in class.fields.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(field_name);
                self.write("={}");
            }
            self.write(")\"");
            for (field_name, _) in &class.fields {
                self.write(", self.");
                self.write(field_name);
            }
            self.write(")\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
        }

        // Emit protocol trait impls
        self.emit_protocol_impls(class, module);
    }

    /// Emit a Rust `trait` definition for a Protocol class.
    fn emit_protocol_trait(&mut self, class: &HirClass) {
        self.write_indent();
        if self.pub_mode {
            self.write("pub trait ");
        } else {
            self.write("trait ");
        }
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;

        for method in &class.methods {
            self.write_indent();
            self.write("fn ");
            self.write(&method.name);
            self.write("(&self");
            for param in &method.params {
                self.write(", ");
                self.write(&param.name);
                self.write(": ");
                self.write(&param.ty.rust_type());
            }
            self.write(")");
            if method.return_type != Type::None {
                self.write(" -> ");
                self.write(&method.return_type.rust_type());
            }
            self.write(";\n");
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit a newtype tuple struct.
    fn emit_enum_class(&mut self, class: &HirClass) {
        // #[repr(i64)]
        // #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        // enum Color { RED = 1, GREEN = 2, BLUE = 3 }
        self.write_indent();
        self.write("#[repr(i64)]\n");
        self.write_indent();
        self.write("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
        self.write_indent();
        if self.pub_mode {
            self.write("pub enum ");
        } else {
            self.write("enum ");
        }
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;
        let mut auto_value = 1i64;
        for (variant_name, value) in &class.enum_variants {
            self.write_indent();
            self.write(variant_name);
            let v = value.unwrap_or(auto_value);
            self.write(&format!(" = {}", v));
            self.write(",\n");
            auto_value = v + 1;
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // impl Display for Color { fn fmt(...) { write!(f, "{:?}", self) } }
        self.write_indent();
        self.write("impl std::fmt::Display for ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;
        self.write_indent();
        self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
        self.indent += 1;
        self.write_indent();
        self.write("write!(f, \"{:?}\", self)\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // impl Color { fn name(&self) -> String { format!("{:?}", self) } fn value(&self) -> i64 { *self as i64 } }
        self.write_indent();
        self.write("impl ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;
        self.write_indent();
        self.write("fn name(&self) -> String { format!(\"{:?}\", self) }\n");
        self.write_indent();
        self.write("fn value(&self) -> i64 { *self as i64 }\n");
        // Emit user-defined methods
        let class_name = class.name.clone();
        let methods = class.methods.clone();
        for method in &methods {
            self.current_class_name = Some(class_name.clone());
            self.emit_class_method(method, class);
        }
        self.current_class_name = None;
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");
    }

    fn emit_newtype(&mut self, class: &HirClass, inner: &Type) {
        // Derive attributes
        self.write_indent();
        if is_hashable_type_codegen(inner) {
            self.write("#[derive(Debug, Clone, PartialEq, Eq, Hash)]\n");
        } else {
            self.write("#[derive(Debug, Clone, PartialEq)]\n");
        }

        self.write_indent();
        if self.pub_mode {
            self.write(&format!("pub struct {}({});\n\n", class.name, inner.rust_type()));
        } else {
            self.write(&format!("struct {}({});\n\n", class.name, inner.rust_type()));
        }

        // Impl block with constructor and value() accessor
        self.write_indent();
        self.write("impl ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;

        // Constructor: fn new(value: InnerType) -> Self
        self.write_indent();
        let pub_prefix = if self.pub_mode { "pub " } else { "" };
        self.write(&format!("{}fn new(value: {}) -> Self {{\n", pub_prefix, inner.rust_type()));
        self.indent += 1;
        self.write_indent();
        self.write("Self(value)\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // Accessor: fn value(&self) -> InnerType
        self.write_indent();
        self.write(&format!("{}fn value(&self) -> {} {{\n", pub_prefix, inner.rust_type()));
        self.indent += 1;
        self.write_indent();
        if inner.ownership() == sifr_type_system::OwnershipKind::Copy {
            self.write("self.0\n");
        } else {
            self.write("self.0.clone()\n");
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");

        // Emit any custom methods
        for method in &class.methods {
            self.output.push('\n');
            self.emit_class_method(method, class);
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");

        // Display impl for newtypes
        self.output.push('\n');
        self.write_indent();
        self.write("impl std::fmt::Display for ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;
        self.write_indent();
        self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
        self.indent += 1;
        self.write_indent();
        self.write("write!(f, \"{}\", self.0)\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit Rust operator trait impls for operator overloading dunders.
    fn emit_operator_impls(&mut self, class: &HirClass) {
        for (dunder, func) in &class.operator_impls {
            match dunder.as_str() {
                "__add__" => self.emit_binop_trait_impl(class, func, "Add", "add", "+"),
                "__sub__" => self.emit_binop_trait_impl(class, func, "Sub", "sub", "-"),
                "__mul__" => self.emit_binop_trait_impl(class, func, "Mul", "mul", "*"),
                "__truediv__" => self.emit_binop_trait_impl(class, func, "Div", "div", "/"),
                "__mod__" => self.emit_binop_trait_impl(class, func, "Rem", "rem", "%"),
                "__neg__" => self.emit_unaryop_trait_impl(class, func, "Neg", "neg"),
                "__eq__" => self.emit_eq_trait_impl(class, func),
                "__lt__" => self.emit_ord_trait_impl(class, func),
                "__str__" | "__repr__" => {} // Handled separately in emit_class via Display
                _ => {} // Other dunders not yet supported
            }
        }
    }

    /// Emit `impl std::ops::Trait for ClassName` for binary operators.
    /// Uses reference-based impl to avoid consuming the operands.
    fn emit_binop_trait_impl(&mut self, class: &HirClass, func: &HirFunction, trait_name: &str, method_name: &str, _op: &str) {
        let is_generic = !class.type_params.is_empty();
        let bounds = Self::generic_bounds_for_class(class);
        let generic_suffix = if is_generic {
            let params: Vec<String> = class.type_params.iter().map(|p| p.clone()).collect();
            format!("<{}>", params.join(", "))
        } else {
            String::new()
        };
        let class_with_generics = format!("{}{}", class.name, generic_suffix);

        let rhs_ty = if let Some(param) = func.params.first() {
            if param.ty.rust_type() == class.name {
                format!("&{}", class_with_generics)
            } else {
                param.ty.rust_type()
            }
        } else {
            format!("&{}", class_with_generics)
        };
        let output_ty = if func.return_type.rust_type() == class.name {
            class_with_generics.clone()
        } else {
            func.return_type.rust_type()
        };

        self.output.push('\n');
        self.write_indent();
        if is_generic {
            let bounded_params: Vec<String> = class.type_params.iter()
                .map(|p| format!("{}: {}", p, bounds))
                .collect();
            self.write(&format!("impl<{}> std::ops::{}<{}> for &{} {{\n",
                bounded_params.join(", "), trait_name, rhs_ty, class_with_generics));
        } else {
            self.write(&format!("impl std::ops::{}<{}> for &{} {{\n", trait_name, rhs_ty, class.name));
        }
        self.indent += 1;
        self.write_indent();
        self.write(&format!("type Output = {};\n\n", output_ty));
        self.write_indent();
        self.write(&format!("fn {}(self, ", method_name));
        if let Some(param) = func.params.first() {
            self.write(&param.name);
        } else {
            self.write("rhs");
        }
        self.write(": ");
        self.write(&rhs_ty);
        self.write(") -> Self::Output {\n");
        self.indent += 1;

        let saved_mutated = std::mem::take(&mut self.mutated_vars);
        self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);
        for stmt in &func.body {
            self.emit_stmt(stmt);
        }
        self.mutated_vars = saved_mutated;

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit `impl std::ops::Neg for ClassName` for unary negation.
    fn emit_unaryop_trait_impl(&mut self, class: &HirClass, func: &HirFunction, trait_name: &str, method_name: &str) {
        let output_ty = func.return_type.rust_type();

        self.output.push('\n');
        self.write_indent();
        self.write(&format!("impl std::ops::{} for {} {{\n", trait_name, class.name));
        self.indent += 1;
        self.write_indent();
        self.write(&format!("type Output = {};\n\n", output_ty));
        self.write_indent();
        self.write(&format!("fn {}(self) -> Self::Output {{\n", method_name));
        self.indent += 1;

        for stmt in &func.body {
            self.emit_stmt(stmt);
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit `impl PartialEq for ClassName` for __eq__.
    fn emit_eq_trait_impl(&mut self, class: &HirClass, func: &HirFunction) {
        self.output.push('\n');
        self.write_indent();
        self.write(&format!("impl PartialEq for {} {{\n", class.name));
        self.indent += 1;
        self.write_indent();
        self.write("fn eq(&self, ");
        if let Some(param) = func.params.first() {
            self.write(&param.name);
        } else {
            self.write("other");
        }
        self.write(&format!(": &{}) -> bool {{\n", class.name));
        self.indent += 1;

        for stmt in &func.body {
            self.emit_stmt(stmt);
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit `impl PartialOrd for ClassName` for __lt__.
    fn emit_ord_trait_impl(&mut self, class: &HirClass, func: &HirFunction) {
        self.output.push('\n');
        self.write_indent();
        self.write(&format!("impl PartialOrd for {} {{\n", class.name));
        self.indent += 1;
        self.write_indent();
        self.write("fn partial_cmp(&self, ");
        if let Some(param) = func.params.first() {
            self.write(&param.name);
        } else {
            self.write("other");
        }
        self.write(&format!(": &{}) -> Option<std::cmp::Ordering> {{\n", class.name));
        self.indent += 1;

        // For __lt__, we generate a comparison that returns Ordering
        // The user's __lt__ body returns bool, so we need to adapt
        // Simple approach: compare using the body logic
        // We'll emit: if self < other { Some(Less) } else if self == other { Some(Equal) } else { Some(Greater) }
        // But for simplicity, just use the fields for comparison
        self.write_indent();
        self.write("Some(");
        // Use the first field for comparison as a simple heuristic
        if let Some((field_name, _)) = class.fields.first() {
            self.write(&format!("self.{}.partial_cmp(&{}.{})?", field_name,
                if let Some(param) = func.params.first() { &param.name } else { "other" },
                field_name));
        } else {
            self.write("std::cmp::Ordering::Equal");
        }
        self.write(")\n");

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit `impl Protocol for ClassName` blocks for satisfied protocols.
    fn emit_protocol_impls(&mut self, class: &HirClass, module: &HirModule) {
        for proto_name in &class.implements_protocols {
            // Find the protocol definition to get its method list
            let proto_class = module.classes.iter().find(|c| c.name == *proto_name && c.is_protocol);
            let proto_method_names: Vec<String> = proto_class
                .map(|pc| pc.methods.iter().map(|m| m.name.clone()).collect())
                .unwrap_or_default();

            if proto_method_names.is_empty() { continue; }

            self.output.push('\n');
            self.write_indent();
            self.write(&format!("impl {} for {} {{\n", proto_name, class.name));
            self.indent += 1;

            // Delegate to inherent methods instead of duplicating the body
            for method in &class.methods {
                if !proto_method_names.contains(&method.name) { continue; }

                self.write_indent();
                self.write("fn ");
                self.write(&method.name);
                self.write("(&self");
                for param in &method.params {
                    self.write(", ");
                    self.write(&param.name);
                    self.write(": ");
                    self.write(&param.ty.rust_type());
                }
                self.write(")");
                if method.return_type != Type::None {
                    self.write(" -> ");
                    self.write(&method.return_type.rust_type());
                }
                self.write(" {\n");
                self.indent += 1;
                // Delegate to the inherent impl method
                self.write_indent();
                if method.return_type != Type::None {
                    self.write(&format!("{}::{}(self", class.name, method.name));
                } else {
                    self.write(&format!("{}::{}(self", class.name, method.name));
                }
                for param in &method.params {
                    self.write(", ");
                    self.write(&param.name);
                }
                self.write(")\n");
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }

            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
        }
    }

    fn emit_class_method(&mut self, method: &HirFunction, class: &HirClass) {
        self.current_return_type = Some(method.return_type.clone());

        // Pre-scan: collect mutated variables so we know which need `mut`
        self.mutated_vars = collect_mutated_vars_with_sigs(&method.body, &self.func_signatures);

        self.write_indent();
        let pub_prefix = if self.pub_mode { "pub " } else { "" };

        match method.method_kind {
            MethodKind::ClassMethod => {
                // @classmethod -> associated function (no self)
                self.write(&format!("{}fn {}(", pub_prefix, method.name));
                for (i, param) in method.params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&param.name);
                    self.write(": ");
                    self.write(&param.ty.rust_type());
                }
                self.write(")");
                if method.return_type != Type::None {
                    self.write(" -> ");
                    self.write(&method.return_type.rust_type());
                }
                self.write(" {\n");
                self.indent += 1;
                for stmt in &method.body {
                    self.emit_stmt(stmt);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
            MethodKind::StaticMethod => {
                // @staticmethod -> associated function (no self)
                self.write(&format!("{}fn {}(", pub_prefix, method.name));
                for (i, param) in method.params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&param.name);
                    self.write(": ");
                    self.write(&param.ty.rust_type());
                }
                self.write(")");
                if method.return_type != Type::None {
                    self.write(" -> ");
                    self.write(&method.return_type.rust_type());
                }
                self.write(" {\n");
                self.indent += 1;
                for stmt in &method.body {
                    self.emit_stmt(stmt);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
            MethodKind::Regular => {
                if method.name == "new" {
                    // Constructor: fn new(params) -> Self
                    self.write(&format!("{}fn new(", pub_prefix));
                    for (i, param) in method.params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.write(&param.name);
                        self.write(": ");
                        // Check if this parameter corresponds to a recursive field
                        let is_recursive = self.recursive_fields.contains(&(class.name.clone(), param.name.clone()));
                        if is_recursive {
                            self.write(&recursive_field_rust_type(&param.ty, &class.name));
                        } else if matches!(&param.ty, Type::Callable(..)) {
                            // Callable params in constructors need 'static for Box::new()
                            self.write(&format!("{} + 'static", param.ty.rust_type()));
                        } else {
                            self.write(&param.ty.rust_type());
                        }
                    }
                    self.write(") -> Self {\n");
                    self.indent += 1;

                    // Check if there's a super() call in the body
                    let has_super = method.body.iter().any(|stmt| {
                        if let HirStmt::Expr { expr } = stmt {
                            matches!(expr, HirExpr::SuperCall { .. })
                        } else {
                            false
                        }
                    });

                    if has_super && class.parent_class.is_some() {
                        // Inheritance constructor: emit super call, then Self { parent: ..., own fields }
                        let parent_name = class.parent_class.as_ref().unwrap();
                        let mut super_args: Option<&Vec<HirExpr>> = None;
                        let mut field_inits: Vec<(&str, &HirExpr)> = Vec::new();
                        let mut other_stmts: Vec<&HirStmt> = Vec::new();

                        for stmt in &method.body {
                            if let HirStmt::Expr { expr: HirExpr::SuperCall { args, .. } } = stmt {
                                super_args = Some(args);
                            } else if let HirStmt::FieldAssign { field, value, .. } = stmt {
                                field_inits.push((field, value));
                            } else {
                                other_stmts.push(stmt);
                            }
                        }

                        // Emit non-field, non-super statements first
                        for stmt in &other_stmts {
                            self.emit_stmt(stmt);
                        }

                        // Build Self { parent: ParentType::new(...), own_field: value, ... }
                        self.write_indent();
                        self.write("Self {\n");
                        self.indent += 1;

                        // Emit parent field
                        self.write_indent();
                        let parent_field = parent_name.to_lowercase();
                        self.write(&parent_field);
                        self.write(": ");
                        self.write(parent_name);
                        self.write("::new(");
                        if let Some(args) = super_args {
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.write(", ");
                                }
                                self.emit_expr(arg);
                            }
                        }
                        self.write("),\n");

                        // Emit own field inits (recursive fields already have correct Box type from params)
                        for (field_name, value) in &field_inits {
                            self.write_indent();
                            self.write(field_name);
                            self.write(": ");
                            self.emit_expr(value);
                            self.write(",\n");
                        }

                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                    } else {
                        // Regular constructor
                        let mut field_inits: Vec<(&str, &HirExpr)> = Vec::new();
                        let mut other_stmts: Vec<&HirStmt> = Vec::new();
                        for stmt in &method.body {
                            if let HirStmt::FieldAssign { field, value, .. } = stmt {
                                field_inits.push((field, value));
                            } else {
                                other_stmts.push(stmt);
                            }
                        }

                        // Emit non-field statements first
                        for stmt in &other_stmts {
                            self.emit_stmt(stmt);
                        }

                        // Emit Self { field: value, ... }
                        self.write_indent();
                        self.write("Self {\n");
                        self.indent += 1;
                        for (field_name, value) in &field_inits {
                            self.write_indent();
                            self.write(field_name);
                            self.write(": ");
                            // deque._data = [] → VecDeque::new() in constructor
                            if class.name == "deque" && *field_name == "_data" {
                                if let HirExpr::ListLiteral { elements, .. } = value {
                                    if elements.is_empty() {
                                        self.write("VecDeque::new()");
                                        self.write(",\n");
                                        continue;
                                    }
                                }
                            }
                            // Wrap Callable values in Box::new() for struct fields
                            let field_ty = class.fields.iter().find(|(n, _)| n == field_name).map(|(_, t)| t);
                            let needs_box = field_ty.map_or(false, |t| matches!(t, Type::Callable(..)));
                            if needs_box {
                                self.write("Box::new(");
                                self.emit_expr(value);
                                self.write(")");
                            } else {
                                self.emit_expr(value);
                            }
                            self.write(",\n");
                        }
                        // For any fields not explicitly assigned, check if param name matches
                        for (field_name, field_ty) in &class.fields {
                            if !field_inits.iter().any(|(f, _)| f == field_name) {
                                if method.params.iter().any(|p| &p.name == field_name) {
                                    self.write_indent();
                                    // Wrap Callable params in Box::new() for struct fields
                                    if matches!(field_ty, Type::Callable(..)) {
                                        self.write(&format!("{}: Box::new({})", field_name, field_name));
                                    } else {
                                        self.write(field_name);
                                    }
                                    self.write(",\n");
                                }
                            }
                        }
                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                    }

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                } else {
                    // Regular method: determine &self vs &mut self
                    let is_mutating = body_contains_field_assign_codegen(&method.body);
                    if is_mutating {
                        self.write(&format!("{}fn ", pub_prefix));
                        self.write(&method.name);
                        self.write("(&mut self");
                    } else {
                        self.write(&format!("{}fn ", pub_prefix));
                        self.write(&method.name);
                        self.write("(&self");
                    }
                    for param in &method.params {
                        self.write(", ");
                        self.write(&param.name);
                        self.write(": ");
                        let rust_ty = self.rust_type_with_generics(&param.ty);
                        match param.convention {
                            ParamConvention::Borrow => {
                                if param.ty.ownership() == sifr_type_system::OwnershipKind::Copy {
                                    self.write(&rust_ty);
                                } else {
                                    self.write(&format!("&{}", rust_ty));
                                }
                            }
                            ParamConvention::MutBorrow => {
                                self.write(&format!("&mut {}", rust_ty));
                            }
                            ParamConvention::Own => {
                                self.write(&rust_ty);
                            }
                        }
                    }
                    self.write(")");

                    if method.return_type != Type::None {
                        self.write(" -> ");
                        // If return type is the same generic class, include type params
                        let ret_rust_type = if let Type::Class { name: ret_name, .. } = &method.return_type {
                            if !class.type_params.is_empty() && ret_name == &class.name {
                                format!("{}<{}>", ret_name, class.type_params.join(", "))
                            } else {
                                method.return_type.rust_type()
                            }
                        } else {
                            method.return_type.rust_type()
                        };
                        self.write(&ret_rust_type);
                    }

                    self.write(" {\n");
                    self.indent += 1;

                    // Track borrowed/mut-borrowed params for correct key-ref and borrow-prefix logic
                    self.borrowed_params.clear();
                    self.mut_borrowed_params.clear();
                    for param in &method.params {
                        if param.convention == ParamConvention::Borrow
                            && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        {
                            self.borrowed_params.insert(param.name.clone());
                        }
                        if param.convention == ParamConvention::MutBorrow
                            && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        {
                            self.mut_borrowed_params.insert(param.name.clone());
                        }
                    }

                    for stmt in &method.body {
                        self.emit_stmt(stmt);
                    }

                    self.borrowed_params.clear();
                    self.mut_borrowed_params.clear();

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                }
            }
        }

        self.current_return_type = None;
        self.mutated_vars.clear();
    }

    fn extra_bounds_for_type_param(tp: &str, body: &[HirStmt]) -> String {
        let mut needs_add = false;
        let mut needs_sub = false;
        Self::scan_body_for_typevar_ops(tp, body, &mut needs_add, &mut needs_sub);
        let mut extra = String::new();
        if needs_add {
            extra.push_str(&format!(" + std::ops::Add<Output = {}>", tp));
        }
        if needs_sub {
            extra.push_str(&format!(" + std::ops::Sub<Output = {}>", tp));
        }
        extra
    }

    fn scan_body_for_typevar_ops(tp: &str, stmts: &[HirStmt], needs_add: &mut bool, needs_sub: &mut bool) {
        for stmt in stmts {
            Self::scan_stmt_for_typevar_ops(tp, stmt, needs_add, needs_sub);
        }
    }

    fn scan_stmt_for_typevar_ops(tp: &str, stmt: &HirStmt, needs_add: &mut bool, needs_sub: &mut bool) {
        match stmt {
            HirStmt::Let { value, .. } => {
                Self::scan_expr_for_typevar_ops(tp, value, needs_add, needs_sub);
            }
            HirStmt::Assign { value, .. } => {
                Self::scan_expr_for_typevar_ops(tp, value, needs_add, needs_sub);
            }
            HirStmt::Expr { expr } => {
                Self::scan_expr_for_typevar_ops(tp, expr, needs_add, needs_sub);
            }
            HirStmt::Return { value: Some(expr) } => {
                Self::scan_expr_for_typevar_ops(tp, expr, needs_add, needs_sub);
            }
            HirStmt::If { condition, then_body, elif_clauses, else_body, .. } => {
                Self::scan_expr_for_typevar_ops(tp, condition, needs_add, needs_sub);
                Self::scan_body_for_typevar_ops(tp, then_body, needs_add, needs_sub);
                for (cond, body) in elif_clauses {
                    Self::scan_expr_for_typevar_ops(tp, cond, needs_add, needs_sub);
                    Self::scan_body_for_typevar_ops(tp, body, needs_add, needs_sub);
                }
                if let Some(eb) = else_body {
                    Self::scan_body_for_typevar_ops(tp, eb, needs_add, needs_sub);
                }
            }
            HirStmt::While { condition, body, .. } => {
                Self::scan_expr_for_typevar_ops(tp, condition, needs_add, needs_sub);
                Self::scan_body_for_typevar_ops(tp, body, needs_add, needs_sub);
            }
            HirStmt::For { iter, body, .. } => {
                Self::scan_expr_for_typevar_ops(tp, iter, needs_add, needs_sub);
                Self::scan_body_for_typevar_ops(tp, body, needs_add, needs_sub);
            }
            _ => {}
        }
    }

    fn scan_expr_for_typevar_ops(tp: &str, expr: &HirExpr, needs_add: &mut bool, needs_sub: &mut bool) {
        if let HirExpr::BinOp { left, op, right, ty } = expr {
            let left_is_tp = matches!(left.ty(), Type::TypeVar(ref n) if n == tp);
            let right_is_tp = matches!(right.ty(), Type::TypeVar(ref n) if n == tp);
            let result_is_tp = matches!(ty, Type::TypeVar(ref n) if n == tp);
            if left_is_tp || right_is_tp || result_is_tp {
                match op.as_str() {
                    "+" => *needs_add = true,
                    "-" => *needs_sub = true,
                    _ => {}
                }
            }
            Self::scan_expr_for_typevar_ops(tp, left, needs_add, needs_sub);
            Self::scan_expr_for_typevar_ops(tp, right, needs_add, needs_sub);
        }
    }

    fn emit_function(&mut self, func: &HirFunction) {
        // In test mode, skip the main function
        if self.test_mode && func.name == "main" {
            return;
        }

        // Track the current function's return type for Option wrapping
        self.current_return_type = Some(func.return_type.clone());

        // Pre-scan: collect mutated variables so we know which need `mut`.
        // Use func_signatures to detect variables passed to mut params (need `let mut`).
        self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);

        // Track borrowed parameters for dereference in comparisons
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        // Track Callable-typed params/locals so we can emit correct borrow prefixes when calling them
        self.callable_var_conventions.clear();
        for param in &func.params {
            if param.convention == ParamConvention::Borrow
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
            {
                self.borrowed_params.insert(param.name.clone());
            }
            if param.convention == ParamConvention::MutBorrow
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
            {
                self.mut_borrowed_params.insert(param.name.clone());
            }
            // Register Callable-typed params for convention-aware call emission
            if let Type::Callable(ref param_types, ref conventions, _) = param.ty {
                let conv_list: Vec<(Type, ParamConvention)> = param_types.iter()
                    .zip(conventions.iter())
                    .map(|(t, c)| (t.clone(), *c))
                    .collect();
                self.callable_var_conventions.insert(param.name.clone(), conv_list);
            }
        }

        // Emit decorator comments before the function
        for decorator in &func.decorators {
            self.write_indent();
            self.write(&format!("// @{}\n", decorator));
        }

        // In test mode, add #[test] attribute for test_* functions
        if self.test_mode && func.name.starts_with("test_") {
            self.write_indent();
            self.write("#[test]\n");
        }

        // Function signature -- only emit params without defaults, or all params
        // Since Rust doesn't have default params, we emit all params and handle
        // defaults at call site
        self.write_indent();
        if self.pub_mode && func.name != "main" {
            self.write("pub fn ");
        } else {
            self.write("fn ");
        }
        self.write(&func.name);
        // Emit generic type parameters if this is a generic function
        if !func.type_params.is_empty() {
            let needs_hash_eq = Self::func_needs_hash_eq(func);
            self.write("<");
            for (i, tp) in func.type_params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                let extra = Self::extra_bounds_for_type_param(tp, &func.body);
                let base = if needs_hash_eq {
                    "Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq"
                } else {
                    "Clone + std::fmt::Display + PartialOrd"
                };
                self.write(&format!("{}: {}{}", tp, base, extra));
            }
            self.write(">");
        }
        self.write("(");

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            // Emit `mut` for parameters that are mutated in the body
            // (only for Own params; borrowed params use &mut convention instead)
            if param.convention == ParamConvention::Own && self.mutated_vars.contains(&param.name) {
                self.write("mut ");
            }
            self.write(&param.name);
            self.write(": ");
            // Emit parameter type based on convention
            let rust_ty = param.ty.rust_type();
            match param.convention {
                ParamConvention::Borrow => {
                    if param.ty.ownership() == sifr_type_system::OwnershipKind::Copy {
                        // Copy types are always passed by value
                        self.write(&rust_ty);
                    } else {
                        self.write(&format!("&{}", rust_ty));
                    }
                }
                ParamConvention::MutBorrow => {
                    self.write(&format!("&mut {}", rust_ty));
                }
                ParamConvention::Own => {
                    self.write(&rust_ty);
                }
            }
        }

        self.write(")");

        // Detect if this is a generator function (contains yield statements)
        let is_generator = body_contains_yield(&func.body);

        // Return type (omit for main and for None return)
        if func.return_type != Type::None || func.name != "main" {
            if func.return_type != Type::None {
                self.write(" -> ");
                if is_generator {
                    // Generator functions return impl Iterator<Item = T>
                    let yield_ty = if let Type::List(ref elem) = func.return_type {
                        elem.rust_type()
                    } else {
                        "i64".to_string()
                    };
                    self.write(&format!("impl Iterator<Item = {}>", yield_ty));
                } else {
                    // If return type is a generic class and this function has type params,
                    // include the type params in the return type
                    let ret_type = if let Type::Class { name: ref ret_name, .. } = func.return_type {
                        if self.generic_classes.contains(ret_name) && !func.type_params.is_empty() {
                            let type_params_in_ret: Vec<&String> = func.type_params.iter()
                                .filter(|tp| type_contains_typevar(&func.return_type, tp))
                                .collect();
                            if !type_params_in_ret.is_empty() {
                                format!("{}<{}>", ret_name, type_params_in_ret.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
                            } else {
                                func.return_type.rust_type()
                            }
                        } else {
                            func.return_type.rust_type()
                        }
                    } else {
                        func.return_type.rust_type()
                    };
                    self.write(&ret_type);
                }
            }
        }

        self.write(" {\n");
        self.indent += 1;

        if is_generator {
            // Lazy generator using std::iter::from_fn.
            // Pattern: init stmts; while cond: [pre_yield]; yield val; [post_yield]
            // Becomes: init stmts; from_fn(move || { if cond { pre_yield; let v = val; post_yield; Some(v) } else { None } })
            let yield_ty = if let Type::List(ref elem) = func.return_type {
                elem.rust_type()
            } else {
                "i64".to_string()
            };

            // Separate body into init statements and the while loop
            let mut init_stmts = Vec::new();
            let mut while_stmt = None;
            for stmt in &func.body {
                if while_stmt.is_none() {
                    if let HirStmt::While { .. } = stmt {
                        while_stmt = Some(stmt);
                    } else {
                        init_stmts.push(stmt);
                    }
                }
            }

            // Emit init statements (local variable declarations, always mutable)
            for stmt in &init_stmts {
                self.emit_generator_init_stmt(stmt);
            }

            // Emit the lazy iterator
            self.write_indent();
            self.write(&format!("std::iter::from_fn(move || -> Option<{}> {{\n", yield_ty));
            self.indent += 1;

            if let Some(HirStmt::While { condition, body, .. }) = while_stmt {
                // Check if yield is directly in the while body or nested in an if
                let has_conditional_yield = !body.iter().any(|s| matches!(s, HirStmt::Yield { .. }))
                    && body.iter().any(|s| {
                        if let HirStmt::If { then_body, .. } = s {
                            body_contains_yield(then_body)
                        } else {
                            false
                        }
                    });

                if has_conditional_yield {
                    // Conditional yield: while cond: if test: yield val; post_stmts
                    // Emit as: while cond { let mut __yielded = None; if test { __yielded = Some(val); } post_stmts; if let Some(v) = __yielded { return Some(v); } }; None
                    self.write_indent();
                    self.write("while ");
                    self.emit_expr(condition);
                    self.write(" {\n");
                    self.indent += 1;

                    // Emit __yielded variable
                    self.write_indent();
                    self.write(&format!("let mut __yielded: Option<{}> = None;\n", yield_ty));

                    // Emit body with yield replaced by __yielded = Some(val)
                    for s in body {
                        if let HirStmt::If { condition: if_cond, then_body, .. } = s {
                            if body_contains_yield(then_body) {
                                // Emit the if with yield -> __yielded = Some(val)
                                self.write_indent();
                                self.write("if ");
                                self.emit_expr(if_cond);
                                self.write(" {\n");
                                self.indent += 1;
                                for ts in then_body {
                                    if let HirStmt::Yield { value } = ts {
                                        self.write_indent();
                                        self.write("__yielded = Some(");
                                        self.emit_expr(value);
                                        self.write(");\n");
                                    } else {
                                        self.emit_stmt(ts);
                                    }
                                }
                                self.indent -= 1;
                                self.write_indent();
                                self.write("}\n");
                            } else {
                                self.emit_stmt(s);
                            }
                        } else {
                            self.emit_stmt(s);
                        }
                    }

                    // Check if a value was yielded
                    self.write_indent();
                    self.write("if let Some(__v) = __yielded {\n");
                    self.indent += 1;
                    self.write_indent();
                    self.write("return Some(__v);\n");
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                    // After while loop exits, return None
                    self.write_indent();
                    self.write("None\n");
                } else {
                    // Simple yield: while cond: pre_yield; yield val; post_yield
                    // Separate into pre-yield, yield expr, post-yield
                    let mut pre_yield = Vec::new();
                    let mut yield_expr = None;
                    let mut post_yield = Vec::new();
                    let mut found_yield = false;
                    for s in body {
                        if !found_yield {
                            if let HirStmt::Yield { value } = s {
                                yield_expr = Some(value);
                                found_yield = true;
                            } else {
                                pre_yield.push(s);
                            }
                        } else {
                            post_yield.push(s);
                        }
                    }

                    // Emit: if cond { pre_yield; let v = yield_val; post_yield; Some(v) } else { None }
                    self.write_indent();
                    self.write("if ");
                    self.emit_expr(condition);
                    self.write(" {\n");
                    self.indent += 1;

                    for s in &pre_yield {
                        self.emit_stmt(s);
                    }

                    if let Some(yexpr) = yield_expr {
                        self.write_indent();
                        self.write("let __yield_val = ");
                        self.emit_expr(yexpr);
                        self.write(";\n");
                    }

                    for s in &post_yield {
                        self.emit_stmt(s);
                    }

                    self.write_indent();
                    self.write("Some(__yield_val)\n");

                    self.indent -= 1;
                    self.write_indent();
                    self.write("} else {\n");
                    self.indent += 1;
                    self.write_indent();
                    self.write("None\n");
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                }
            } else {
                self.write_indent();
                self.write("None\n");
            }

            self.indent -= 1;
            self.write_indent();
            self.write("})\n");
        } else {
            // Non-generator: emit body normally
            for stmt in &func.body {
                self.emit_stmt(stmt);
            }
        }

        self.indent -= 1;
        self.writeln("}");

        self.current_return_type = None;
        self.mutated_vars.clear();
    }

    /// Emit a generator initialization statement (always mutable for closure capture)
    fn emit_generator_init_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let { name, ty, value, .. } => {
                self.write_indent();
                self.write("let mut ");
                self.write(name);
                self.write(": ");
                self.write(&ty.rust_type());
                self.write(" = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            _ => {
                self.emit_stmt(stmt);
            }
        }
    }

    fn emit_lowered_stmts(&mut self, lowered_stmts: &[RustStmt]) {
        for lowered_stmt in lowered_stmts {
            match lowered_stmt {
                RustStmt::Expr(lowered_expr) => {
                    self.write_indent();
                    self.write(&crate::render_expr(lowered_expr));
                    self.write(";\n");
                }
                RustStmt::RawCode(code) => {
                    self.write_indent();
                    self.write(code);
                    self.write("\n");
                }
                RustStmt::Break => {
                    self.writeln("break;");
                }
                RustStmt::Continue => {
                    self.writeln("continue;");
                }
                _ => {
                    self.write_indent();
                    let rendered = crate::render_stmts(std::slice::from_ref(lowered_stmt));
                    self.write(rendered.trim_end());
                    self.write("\n");
                }
            }
        }
    }

    fn emit_stmt(&mut self, stmt: &HirStmt) {
        if let Some(lowered_stmts) = try_lower_simple_stmt(
            stmt,
            self.in_loop_with_else,
            &self.mutated_vars,
            &self.borrowed_params,
        ) {
            self.emit_lowered_stmts(&lowered_stmts);
            return;
        }

        match stmt {
            HirStmt::Let { name, ty, value, is_mutable: _ } => {
                self.write_indent();
                // Only emit `mut` if the variable is actually mutated later
                if self.mutated_vars.contains(name) {
                    self.write("let mut ");
                } else {
                    self.write("let ");
                }
                self.write(name);
                // Skip explicit type annotation for generic class instances (let Rust infer)
                let is_generic_class = matches!(ty, Type::Class { name: ref cn, .. } if self.generic_classes.contains(cn));
                if !is_generic_class {
                    self.write(": ");
                    self.write(&ty.rust_type());
                }
                self.write(" = ");
                if matches!(ty, Type::None) && matches!(value, HirExpr::NoneLiteral) {
                    // `x: None = None` -> `let x: () = ()`
                    self.write("()");
                } else if matches!(ty, Type::BigInt) && matches!(value, HirExpr::IntLiteral(_)) {
                    // `x: bigint = 42` -> `BigInt::from(42_i64)`
                    if let HirExpr::IntLiteral(v) = value {
                        self.write(&format!("BigInt::from({}_i64)", v));
                    }
                } else if is_option_type(ty) && matches!(value, HirExpr::NoneLiteral) {
                    // `x: str | None = None` -> `let x: Option<String> = None`
                    self.write("None");
                } else if is_option_type(ty) && !is_option_type(value.ty()) && !matches!(value.ty(), Type::None) {
                    // RHS is a plain value (not already Option) -> wrap in Some()
                    // But if RHS is a function call returning Option, don't double-wrap
                    self.write("Some(");
                    self.emit_expr(value);
                    self.write(")");
                } else {
                    // Check if RHS is a call to a generator function and target is list[T]
                    let needs_collect = matches!(ty, Type::List(_)) && self.is_generator_call(value);
                    self.emit_expr(value);
                    if needs_collect {
                        self.write(".collect()");
                    }
                    // Clone borrowed TypeVar params assigned to owned TypeVar locals
                    let needs_clone_for_typevar = matches!(ty, Type::TypeVar(_))
                        && if let HirExpr::Name { name: ref vname, .. } = value {
                            self.borrowed_params.contains(vname.as_str())
                        } else {
                            false
                        };
                    if needs_clone_for_typevar {
                        self.write(".clone()");
                    }
                }
                self.write(";\n");
            }
            HirStmt::Assign { name, value } => {
                self.write_indent();
                self.write(name);
                self.write(" = ");
                self.emit_expr(value);
                // Clone borrowed TypeVar params reassigned to owned TypeVar locals
                if matches!(value.ty(), Type::TypeVar(_)) {
                    if let HirExpr::Name { name: ref vname, .. } = value {
                        if self.borrowed_params.contains(vname.as_str()) {
                            self.write(".clone()");
                        }
                    }
                }
                self.write(";\n");
            }
            HirStmt::AugAssign { name, op, value } => {
                self.write_indent();
                let var_ty = value.ty();
                match op.as_str() {
                    "+=" => {
                        // Special cases for string and list
                        match var_ty {
                            Type::Str => {
                                self.write(name);
                                self.write(".push_str(");
                                self.emit_str_ref_expr(value);
                                self.write(");\n");
                                return;
                            }
                            _ => {
                                // Check if target is a list (we need to look at the value context)
                                // For list += list, use extend
                                if let Type::List(_) = var_ty {
                                    self.write(name);
                                    self.write(".extend(");
                                    self.emit_expr(value);
                                    self.write(");\n");
                                    return;
                                }
                            }
                        }
                        self.write(name);
                        self.write(" += ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "-=" | "*=" | "%=" => {
                        self.write(name);
                        self.write(&format!(" {} ", op));
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "/=" => {
                        self.write(name);
                        self.write(" /= ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "//=" => {
                        self.write(name);
                        self.write(" /= ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "**=" => {
                        // Power assignment: x **= y
                        // If the value (exponent) is int, use i64::pow for int targets
                        if matches!(var_ty, Type::Int) {
                            self.write(name);
                            self.write(" = ");
                            self.write(&format!("{}.pow(", name));
                            self.emit_expr(value);
                            self.write(" as u32);\n");
                        } else {
                            self.write(name);
                            self.write(" = ");
                            self.write(&format!("({} as f64).powf(", name));
                            self.emit_expr(value);
                            self.write(" as f64);\n");
                        }
                    }
                    _ => {
                        self.write(name);
                        self.write(&format!(" {} ", op));
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                }
            }
            HirStmt::Return { value } => {
                // Inside Display::fmt (for __str__ methods), return statements become
                // write!(f, "{}", val); return Ok(())
                if self.in_display_impl {
                    if let Some(val) = value {
                        self.write_indent();
                        self.write("write!(f, \"{}\", ");
                        self.emit_expr(val);
                        self.write(")?;\n");
                        self.write_indent();
                        self.write("return Ok(());\n");
                    } else {
                        self.write_indent();
                        self.write("return Ok(());\n");
                    }
                    return;
                }
                let ret_is_option = self.current_return_type.as_ref().map_or(false, |t| is_option_type(t));
                let ret_is_non_option_union = self.current_return_type.as_ref().map_or(false, |t| {
                    matches!(t, Type::Union(_)) && !is_option_type(t)
                });
                self.write_indent();
                if let Some(val) = value {
                    self.write("return ");
                    if ret_is_option && matches!(val, HirExpr::NoneLiteral) {
                        // `return None` in Python -> `return None` in Rust Option
                        self.write("None");
                    } else if ret_is_option && !is_option_type(val.ty()) {
                        // Returning a non-Option value from an Option function -> wrap in Some()
                        self.write("Some(");
                        self.emit_expr(val);
                        self.write(")");
                    } else if ret_is_non_option_union {
                        // Returning a value from a non-Option union function -> wrap in enum variant
                        if let Some(ret_ty) = &self.current_return_type.clone() {
                            if let Type::Union(members) = ret_ty {
                                let arg_ty = val.ty();
                                if let Some(variant) = find_union_variant(members, arg_ty) {
                                    let enum_name = ret_ty.union_enum_name();
                                    self.write(&format!("{}::{}(", enum_name, variant));
                                    self.emit_expr(val);
                                    self.write(")");
                                } else {
                                    self.emit_expr(val);
                                }
                            } else {
                                self.emit_expr(val);
                            }
                        } else {
                            self.emit_expr(val);
                        }
                    } else if !ret_is_option && is_option_type(val.ty()) && !matches!(val.ty(), Type::None) {
                        // Returning an Option value from a non-Option function -> unwrap
                        // This happens with generic functions where T is inferred as a concrete type
                        // but the body has safe-indexing that returns Option<T>
                        self.emit_expr(val);
                        self.write(".unwrap()");
                    } else if matches!(val.ty(), Type::TypeVar(_)) {
                        // Returning a TypeVar-typed value needs .clone() to avoid move from &self
                        self.emit_expr(val);
                        self.write(".clone()");
                    } else if self.current_class_name.is_some() {
                        // Inside a class method: if returning `self` (a Name expr),
                        // we need .clone() because methods take &self in Rust
                        if let HirExpr::Name { name, .. } = val {
                            if name == "self" {
                                self.emit_expr(val);
                                self.write(".clone()");
                            } else {
                                self.emit_expr(val);
                            }
                        } else {
                            self.emit_expr(val);
                        }
                    } else {
                        self.emit_expr(val);
                    }
                    self.write(";\n");
                } else {
                    if ret_is_option {
                        self.write("return None;\n");
                    } else {
                        self.write("return;\n");
                    }
                }
            }
            HirStmt::Expr { expr } => {
                self.write_indent();
                self.emit_expr(expr);
                self.write(";\n");
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                // Detect isinstance narrowing for union enums:
                // `if isinstance(x, int):` -> `match x { IntOrStr::Int(x) => { ... }, IntOrStr::Str(x) => { ... } }`
                if let Some((var_name, variant_name, enum_name, other_variants)) = detect_isinstance_union(condition) {
                    self.write_indent();
                    self.write(&format!("match {} {{\n", var_name));
                    self.indent += 1;

                    // Then branch: the matched variant
                    let then_mutated = collect_mutated_vars(then_body);
                    let var_mut = if then_mutated.contains(&var_name) { "mut " } else { "" };
                    self.write_indent();
                    self.write(&format!("{}::{}({}{}) => {{\n", enum_name, variant_name, var_mut, var_name));
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");

                    // Emit elif isinstance branches as additional match arms
                    let mut remaining_variants = other_variants.clone();
                    for (elif_cond, elif_body) in elif_clauses {
                        if let Some((_, elif_variant, _, _)) = detect_isinstance_union(elif_cond) {
                            let elif_mutated = collect_mutated_vars(elif_body);
                            let elif_var_mut = if elif_mutated.contains(&var_name) { "mut " } else { "" };
                            self.write_indent();
                            self.write(&format!("{}::{}({}{}) => {{\n", enum_name, elif_variant, elif_var_mut, var_name));
                            self.indent += 1;
                            for s in elif_body {
                                self.emit_stmt(s);
                            }
                            self.indent -= 1;
                            self.writeln("}");
                            // Remove this variant from remaining
                            remaining_variants.retain(|(v, _)| v != &elif_variant);
                        }
                    }

                    // Else branch: remaining variant(s)
                    if let Some(else_stmts) = else_body {
                        let else_mutated = collect_mutated_vars(else_stmts);
                        let else_var_mut = if else_mutated.contains(&var_name) { "mut " } else { "" };
                        if remaining_variants.len() == 1 {
                            let (other_variant, _) = &remaining_variants[0];
                            self.write_indent();
                            self.write(&format!("{}::{}({}{}) => {{\n", enum_name, other_variant, else_var_mut, var_name));
                        } else {
                            self.write_indent();
                            self.write("_ => {\n");
                        }
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                        self.writeln("}");
                    } else {
                        // No else body: add wildcard arm so match is exhaustive
                        self.write_indent();
                        self.write("_ => {}\n");
                    }

                    self.indent -= 1;
                    self.writeln("}");
                }
                // Detect truthiness on Option: `if x:` where x is Option -> `if let Some(x) = x {`
                else if let Some(var_name) = detect_option_truthiness(condition) {
                    self.write_indent();
                    self.write(&format!("if let Some({}) = {} {{\n", var_name, var_name));
                    self.indent += 1;
                    self.option_unwrapped_vars.insert(var_name.clone());
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.option_unwrapped_vars.remove(&var_name);
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("} else {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }
                    self.writeln("}");
                }
                // Detect compound `a is not None and b is not None` -> nested if let Some
                else if let Some(vars) = detect_and_not_none_vars(condition) {
                    // Emit nested if-let-Some for each variable
                    for (i, var_name) in vars.iter().enumerate() {
                        self.write_indent();
                        self.write(&format!("if let Some({}) = {} {{\n", var_name, var_name));
                        self.indent += 1;
                        self.option_unwrapped_vars.insert(var_name.clone());
                        if i < vars.len() - 1 {
                            // More variables to unwrap, continue nesting
                        }
                    }
                    // Emit the then-body inside the innermost block
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    // Close all nested blocks
                    for var_name in vars.iter().rev() {
                        self.option_unwrapped_vars.remove(var_name);
                        self.indent -= 1;
                        if let Some(else_stmts) = else_body {
                            if var_name == vars.first().unwrap() {
                                // Only emit else on the outermost block
                                self.write_indent();
                                self.write("} else {\n");
                                self.indent += 1;
                                for s in else_stmts {
                                    self.emit_stmt(s);
                                }
                                self.indent -= 1;
                            }
                        }
                        self.writeln("}");
                    }
                }
                // Detect Option narrowing: `if x is not None:` -> `if let Some(x) = x {`
                else if let Some(var_name) = detect_is_not_none_var(condition) {
                    self.write_indent();
                    // Use `if let Some(var) = var` to unwrap and shadow the variable
                    self.write(&format!("if let Some({}) = {} {{\n", var_name, var_name));
                    self.indent += 1;
                    self.option_unwrapped_vars.insert(var_name.clone());
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.option_unwrapped_vars.remove(&var_name);
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("} else {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }
                    self.writeln("}");
                } else if let Some((var_name, enum_name, _non_none_variants)) = detect_is_none_union_var(condition) {
                    // 3+ member union `is None` check: use match with None variant
                    self.write_indent();
                    self.write(&format!("match {} {{\n", var_name));
                    self.indent += 1;

                    // None arm -> then_body
                    self.write_indent();
                    self.write(&format!("{}::None(()) => {{\n", enum_name));
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");

                    // Non-None arms -> else_body
                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("_ => {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                        self.writeln("}");
                    } else {
                        // Need a catch-all arm even without else
                        self.write_indent();
                        self.write("_ => {}\n");
                    }

                    self.indent -= 1;
                    self.writeln("}");
                } else if let Some(var_name) = detect_is_none_var(condition) {
                    self.write_indent();
                    self.write(&format!("if {}.is_none() {{\n", var_name));
                    self.indent += 1;
                    let then_exits = codegen_body_always_exits(then_body);
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        // In the else branch of `if x is None`, x is not None
                        self.write_indent();
                        self.write(&format!("}} else if let Some({}) = {} {{\n", var_name, var_name));
                        self.indent += 1;
                        self.option_unwrapped_vars.insert(var_name.clone());
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.option_unwrapped_vars.remove(&var_name);
                        self.indent -= 1;
                    }
                    self.writeln("}");

                    // Early-return narrowing: if the then-body always exits (return/break),
                    // unwrap the variable after the if block so subsequent code can use it directly
                    if then_exits && else_body.is_none() {
                        self.write_indent();
                        self.write(&format!("let {} = {}.unwrap();\n", var_name, var_name));
                        self.option_unwrapped_vars.insert(var_name.clone());
                    }
                } else {
                    // Normal if/elif/else
                    // Hoist any walrus expressions before the if
                    self.emit_walrus_hoists(condition);
                    self.write_indent();
                    self.write("if ");
                    self.emit_expr(condition);
                    self.write(" {\n");
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;

                    for (cond, body) in elif_clauses {
                        self.write_indent();
                        self.write("} else if ");
                        self.emit_expr(cond);
                        self.write(" {\n");
                        self.indent += 1;
                        for s in body {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }

                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("} else {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }

                    self.writeln("}");
                }
            }
            HirStmt::While { condition, body, else_body } => {
                let has_else = else_body.is_some();
                if has_else {
                    self.writeln("let mut _broke = false;");
                }
                let prev_loop_else = self.in_loop_with_else;
                self.in_loop_with_else = has_else;
                // Hoist any walrus expressions
                self.emit_walrus_hoists(condition);
                self.write_indent();
                self.write("while ");
                self.emit_expr(condition);
                self.write(" {\n");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
                self.in_loop_with_else = prev_loop_else;
                if let Some(else_stmts) = else_body {
                    self.writeln("if !_broke {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");
                }
            }
            HirStmt::For { target, iter, body, else_body, .. } => {
                let has_else = else_body.is_some();
                if has_else {
                    self.writeln("let mut _broke = false;");
                }
                let prev_loop_else = self.in_loop_with_else;
                self.in_loop_with_else = has_else;
                self.write_indent();
                self.write("for ");
                // Handle tuple unpacking: "i,v" -> "(i, v)"
                if target.contains(',') {
                    let names: Vec<&str> = target.split(',').collect();
                    self.write("(");
                    for (i, name) in names.iter().enumerate() {
                        if i > 0 { self.write(", "); }
                        self.write(name);
                    }
                    self.write(")");
                } else {
                    self.write(target);
                }
                self.write(" in ");
                // For lists, iterate with .iter() to borrow and clone elements
                // But not for generator expressions which are already iterators
                let is_generator_expr = matches!(iter, HirExpr::GeneratorExpr { .. });
                let is_generator_fn_call = self.is_generator_call(iter);
                let is_list = matches!(iter.ty(), Type::List(_));
                let is_dict = matches!(iter.ty(), Type::Dict(_, _));
                let is_str = matches!(iter.ty(), Type::Str);
                self.emit_expr(iter);
                if is_generator_expr || is_generator_fn_call {
                    // Generator expressions and generator function calls are already iterators
                } else if is_list {
                    self.write(".iter().cloned()");
                } else if is_dict {
                    self.write(".keys().cloned()");
                } else if is_str {
                    self.write(".chars().map(|c| c.to_string())");
                }
                self.write(" {\n");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
                self.in_loop_with_else = prev_loop_else;
                if let Some(else_stmts) = else_body {
                    self.writeln("if !_broke {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");
                }
            }
            HirStmt::Break => {
                if self.in_loop_with_else {
                    self.writeln("_broke = true;");
                }
                self.writeln("break;");
            }
            HirStmt::Continue => {
                self.writeln("continue;");
            }
            HirStmt::Pass => {
                // No-op in Rust
            }
            HirStmt::TupleUnpack { targets, value } => {
                self.write_indent();
                self.write("let (");
                for (i, (name, _ty)) in targets.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(name);
                }
                self.write(") = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::StarUnpack { before, star, after, value } => {
                // Emit: let _tmp = value.clone() to avoid moving;
                self.write_indent();
                self.write("let _star_tmp = ");
                self.emit_expr(value);
                self.write(".clone();\n");
                // Emit before vars
                for (i, (name, _ty)) in before.iter().enumerate() {
                    self.write_indent();
                    self.write(&format!("let {} = _star_tmp[{}].clone();\n", name, i));
                }
                // Emit star var
                let (star_name, _star_ty) = star;
                if after.is_empty() {
                    self.write_indent();
                    self.write(&format!("let {} = _star_tmp[{}..].to_vec();\n", star_name, before.len()));
                } else {
                    self.write_indent();
                    self.write(&format!("let {} = _star_tmp[{}.._star_tmp.len() - {}].to_vec();\n", star_name, before.len(), after.len()));
                }
                // Emit after vars
                for (i, (name, _ty)) in after.iter().enumerate() {
                    self.write_indent();
                    self.write(&format!("let {} = _star_tmp[_star_tmp.len() - {}].clone();\n", name, after.len() - i));
                }
            }
            HirStmt::TryExcept { body, handlers, body_error_types } => {
                // Helper: map IOError subclass names to their Rust kind string
                fn io_subclass_kind(name: &str) -> Option<&'static str> {
                    match name {
                        "FileNotFoundError" => Some("FileNotFound"),
                        "PermissionError" => Some("PermissionDenied"),
                        "FileExistsError" => Some("FileExists"),
                        "IsADirectoryError" => Some("IsADirectory"),
                        "NotADirectoryError" => Some("NotADirectory"),
                        "DirectoryNotEmptyError" => Some("DirectoryNotEmpty"),
                        _ => None,
                    }
                }

                // Map IOError subclass names to their parent type for Rust codegen
                fn rust_error_type(name: &str) -> &str {
                    if io_subclass_kind(name).is_some() {
                        "IOError"
                    } else {
                        name
                    }
                }

                // Collect distinct Rust error types from handlers and body
                let mut error_type_names: Vec<String> = Vec::new();
                let mut has_catch_all = false;
                for handler in handlers {
                    if let Some(ref et) = handler.error_type {
                        if et == "Error" {
                            has_catch_all = true;
                        } else {
                            let rust_ty = rust_error_type(et).to_string();
                            if !error_type_names.contains(&rust_ty) {
                                error_type_names.push(rust_ty);
                            }
                        }
                    } else {
                        has_catch_all = true;
                    }
                }
                // If catch-all only (no specific handlers), use body error types
                if error_type_names.is_empty() && has_catch_all {
                    for et in body_error_types {
                        if et != "Error" {
                            let rust_ty = rust_error_type(et).to_string();
                            if !error_type_names.contains(&rust_ty) {
                                error_type_names.push(rust_ty);
                            }
                        }
                    }
                }

                // Check if any handler catches an IOError subclass specifically
                let has_io_subclass_handler = handlers.iter().any(|h| {
                    h.error_type.as_ref().map_or(false, |et| io_subclass_kind(et).is_some())
                });

                let needs_enum = error_type_names.len() > 1;

                if needs_enum {
                    // Multi-error-type try block: generate a local error enum
                    self.try_enum_counter += 1;
                    let enum_name = format!("_TryErr{}", self.try_enum_counter);

                    // Emit enum definition
                    self.write_indent();
                    self.write("#[allow(non_camel_case_types)]\n");
                    self.write_indent();
                    self.write(&format!("enum {} {{\n", enum_name));
                    self.indent += 1;
                    for et in &error_type_names {
                        self.write_indent();
                        self.write(&format!("{}({}),\n", et, et));
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");

                    // Emit From impls for each error type
                    for et in &error_type_names {
                        self.write_indent();
                        self.write(&format!("impl From<{}> for {} {{\n", et, enum_name));
                        self.indent += 1;
                        self.write_indent();
                        self.write(&format!("fn from(e: {}) -> Self {{ {}::{}(e) }}\n", et, enum_name, et));
                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                    }

                    // Emit try body as a closure
                    // Check if the try body contains a return statement with a value.
                    let body_has_return_multi = try_body_has_value_return(body);
                    let (closure_ok_type_multi, ok_arm_multi) = if body_has_return_multi {
                        let inner_ty = self.current_return_type.as_ref()
                            .and_then(|t| if let Type::Result(ok, _) = t { Some(ok.rust_type()) } else { None })
                            .unwrap_or_else(|| "_".to_string());
                        (inner_ty, "Ok(__try_ret) => { return Ok(__try_ret); }".to_string())
                    } else {
                        ("()".to_string(), "Ok(()) => {}".to_string())
                    };

                    self.write_indent();
                    self.write(&format!("match (|| -> Result<{}, {}> {{\n", closure_ok_type_multi, enum_name));
                    self.indent += 1;
                    for stmt in body {
                        self.emit_stmt(stmt);
                    }
                    self.write_indent();
                    if body_has_return_multi {
                        self.write("unreachable!()\n");
                    } else {
                        self.write("Ok(())\n");
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("})() {\n");
                    self.indent += 1;
                    self.write_indent();
                    self.write(&format!("{}\n", ok_arm_multi));

                    // Emit match arms
                    for handler in handlers {
                        if let Some(ref et) = handler.error_type {
                            if et == "Error" {
                                // Catch-all: match on any remaining variant
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({}) => {{\n", var_name));
                                if handler.name.is_some() {
                                    self.indent += 1;
                                    self.write_indent();
                                    self.indent -= 1;
                                }
                            } else if let Some(kind) = io_subclass_kind(et) {
                                // IOError subclass: match on the parent enum variant with a guard
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!(
                                    "Err({}::IOError(ref {})) if {}.kind == \"{}\" => {{\n",
                                    enum_name, var_name, var_name, kind
                                ));
                                // Clone the variable so handler body can use it as owned
                                if handler.name.is_some() {
                                    self.indent += 1;
                                    self.write_indent();
                                    self.write(&format!("let {} = {}.clone();\n", var_name, var_name));
                                    self.indent -= 1;
                                }
                            } else if et == "IOError" && has_io_subclass_handler {
                                // IOError parent catch-all (when subclass handlers exist)
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({}::IOError({})) => {{\n", enum_name, var_name));
                            } else {
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({}::{}({})) => {{\n", enum_name, et, var_name));
                            }
                        } else {
                            // Bare except — catch-all
                            let var_name = handler.name.as_deref().unwrap_or("_e");
                            self.write_indent();
                            self.write(&format!("Err({}) => {{\n", var_name));
                        }
                        self.indent += 1;
                        for stmt in &handler.body {
                            self.emit_stmt(stmt);
                        }
                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                    }

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                } else {
                    // Single error type: use simple codegen
                    let error_rust_type = if let Some(first_body_err) = error_type_names.first() {
                        first_body_err.clone()
                    } else {
                        handlers.first()
                            .and_then(|h| h.error_resolved_type.as_ref())
                            .map(|t| {
                                let rt = t.rust_type();
                                // Map IOError subclass resolved types to IOError
                                if io_subclass_kind(&rt).is_some() {
                                    "IOError".to_string()
                                } else {
                                    rt
                                }
                            })
                            .unwrap_or_else(|| "String".to_string())
                    };

                    // Check if the try body contains a return statement with a value.
                    // If so, the closure must return Result<T, E> instead of Result<(), E>.
                    let body_has_return = try_body_has_value_return(body);
                    let (closure_ok_type, ok_arm) = if body_has_return {
                        // Use the function's return type's inner type for the closure
                        let inner_ty = self.current_return_type.as_ref()
                            .and_then(|t| if let Type::Result(ok, _) = t { Some(ok.rust_type()) } else { None })
                            .unwrap_or_else(|| "_".to_string());
                        (inner_ty.clone(), "Ok(__try_ret) => { return Ok(__try_ret); }".to_string())
                    } else {
                        ("()".to_string(), "Ok(()) => {}".to_string())
                    };

                    self.write_indent();
                    self.write(&format!("match (|| -> Result<{}, {}> {{\n", closure_ok_type, error_rust_type));
                    self.indent += 1;
                    for stmt in body {
                        self.emit_stmt(stmt);
                    }
                    self.write_indent();
                    if body_has_return {
                        self.write("unreachable!()\n");
                    } else {
                        self.write("Ok(())\n");
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("})() {\n");
                    self.indent += 1;
                    self.write_indent();
                    self.write(&format!("{}\n", ok_arm));

                    if has_io_subclass_handler && error_rust_type == "IOError" {
                        // IOError with subclass dispatch: use guard-based matching
                        for handler in handlers {
                            if let Some(ref et) = handler.error_type {
                                if et == "Error" || et == "IOError" {
                                    // Parent catch-all
                                    let var_name = handler.name.as_deref().unwrap_or("_e");
                                    self.write_indent();
                                    self.write(&format!("Err({}) => {{\n", var_name));
                                } else if let Some(kind) = io_subclass_kind(et) {
                                    // Subclass match with guard
                                    let var_name = handler.name.as_deref().unwrap_or("_e");
                                    self.write_indent();
                                    self.write(&format!(
                                        "Err(ref {}) if {}.kind == \"{}\" => {{\n",
                                        var_name, var_name, kind
                                    ));
                                    // Clone the variable so handler body can use it as owned
                                    if handler.name.is_some() {
                                        self.indent += 1;
                                        self.write_indent();
                                        self.write(&format!("let {} = {}.clone();\n", var_name, var_name));
                                        self.indent -= 1;
                                    }
                                } else {
                                    let var_name = handler.name.as_deref().unwrap_or("_e");
                                    self.write_indent();
                                    self.write(&format!("Err({}) => {{\n", var_name));
                                }
                            } else {
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({}) => {{\n", var_name));
                            }
                            self.indent += 1;
                            for stmt in &handler.body {
                                self.emit_stmt(stmt);
                            }
                            self.indent -= 1;
                            self.write_indent();
                            self.write("}\n");
                        }
                    } else {
                        // No subclass dispatch needed — simple match
                        for handler in handlers {
                            self.write_indent();
                            if let Some(ref name) = handler.name {
                                self.write(&format!("Err({}) => {{\n", name));
                            } else {
                                self.write("Err(_e) => {\n");
                            }
                            self.indent += 1;
                            for stmt in &handler.body {
                                self.emit_stmt(stmt);
                            }
                            self.indent -= 1;
                            self.write_indent();
                            self.write("}\n");
                        }
                    }

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                }
            }
            HirStmt::Raise { value } => {
                self.write_indent();
                self.write("return Err(");
                self.emit_expr(value);
                self.write(");\n");
            }
            HirStmt::Assert { test, msg } => {
                self.write_indent();
                if let Some(msg_expr) = msg {
                    self.write("assert!(");
                    self.emit_expr(test);
                    self.write(", \"{}\", ");
                    self.emit_display_expr(msg_expr);
                    self.write(");\n");
                } else {
                    self.write("assert!(");
                    self.emit_expr(test);
                    self.write(");\n");
                }
            }
            HirStmt::FieldAssign { object, field, value } => {
                self.write_indent();
                // Check if this is assigning to a parent field via inheritance
                if let Some(ref class_name) = self.current_class_name.clone() {
                    if let Some((parent_name, parent_field_names)) = self.parent_fields.get(class_name).cloned() {
                        if parent_field_names.contains(field.as_str()) {
                            self.write(object);
                            self.write(".");
                            self.write(&parent_name.to_lowercase());
                            self.write(".");
                            self.write(field);
                            self.write(" = ");
                            self.emit_expr(value);
                            self.write(";\n");
                            return;
                        }
                    }
                }
                self.write(object);
                self.write(".");
                self.write(field);
                self.write(" = ");
                // deque._data = [] → VecDeque::new()
                if self.current_class_name.as_deref() == Some("deque") && field == "_data" {
                    if let HirExpr::ListLiteral { elements, .. } = value {
                        if elements.is_empty() {
                            self.write("VecDeque::new()");
                            self.write(";\n");
                            return;
                        }
                    }
                }
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::SubscriptAssign { object, index, value, object_ty } => {
                self.write_indent();
                match object_ty {
                    Type::List(_) => {
                        // list[i] = val -> bounds-checked assignment (safe no-op if out of bounds)
                        self.write("{ let __idx = ");
                        self.emit_expr(index);
                        self.write(" as usize; if let Some(__elem) = ");
                        self.write(object);
                        self.write(".get_mut(__idx) { *__elem = ");
                        self.emit_expr(value);
                        self.write("; } }\n");
                    }
                    Type::Dict(_, _) => {
                        // dict[key] = val -> dict.insert(key, val)
                        self.write(object);
                        self.write(".insert(");
                        self.emit_expr(index);
                        self.write(", ");
                        self.emit_expr(value);
                        self.write(");\n");
                    }
                    _ => {
                        // Fallback: direct subscript
                        self.write(object);
                        self.write("[");
                        self.emit_expr(index);
                        self.write("] = ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                }
            }
            HirStmt::NestedSubscriptAssign { object, outer_index, inner_index, value, object_ty: _ } => {
                self.write_indent();
                // matrix[i][j] = val -> bounds-checked nested assignment (safe no-op if out of bounds)
                self.write("{ let __oi = ");
                self.emit_expr(outer_index);
                self.write(" as usize; let __ii = ");
                self.emit_expr(inner_index);
                self.write(" as usize; if let Some(__row) = ");
                self.write(object);
                self.write(".get_mut(__oi) { if let Some(__elem) = __row.get_mut(__ii) { *__elem = ");
                self.emit_expr(value);
                self.write("; } } }\n");
            }
            HirStmt::SubscriptAugAssign { object, index, op, value, object_ty: _ } => {
                self.write_indent();
                // list[i] += val -> bounds-checked augmented assignment (safe no-op if out of bounds)
                self.write("{ let __idx = ");
                self.emit_expr(index);
                self.write(" as usize; if let Some(__elem) = ");
                self.write(object);
                self.write(".get_mut(__idx) { ");
                // Convert **= to .pow() pattern
                if op == "**=" {
                    self.write("*__elem = __elem.pow(");
                    self.emit_expr(value);
                    self.write(" as u32);");
                } else if op == "//=" {
                    self.write("*__elem = *__elem / ");
                    self.emit_expr(value);
                    self.write(";");
                } else {
                    self.write("*__elem ");
                    self.write(op);
                    self.write(" ");
                    self.emit_expr(value);
                    self.write(";");
                }
                self.write(" } }\n");
            }
            HirStmt::AttributeAugAssign { object, field, op, value } => {
                self.write_indent();
                self.write(object);
                self.write(".");
                self.write(field);
                self.write(&format!(" {} ", op));
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::AttributeSubscriptAssign { object, field, index, value, field_ty } => {
                self.write_indent();
                let field_access = format!("{}.{}", object, field);
                match field_ty {
                    Type::List(_) => {
                        // self.field[i] = val -> bounds-checked assignment
                        self.write("{ let __idx = ");
                        self.emit_expr(index);
                        self.write(" as usize; if let Some(__elem) = ");
                        self.write(&field_access);
                        self.write(".get_mut(__idx) { *__elem = ");
                        self.emit_expr(value);
                        self.write("; } }\n");
                    }
                    Type::Dict(ref key_ty, _) => {
                        // self.field[key] = val -> self.field.insert(key_owned, val)
                        // For move-type keys: if key is a borrowed param (&T), clone for owned insert.
                        self.write(&field_access);
                        self.write(".insert(");
                        let key_needs_clone = matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_));
                        if key_needs_clone {
                            if let HirExpr::Name { name, .. } = index {
                                if self.borrowed_params.contains(name.as_str()) || self.mut_borrowed_params.contains(name.as_str()) {
                                    self.emit_expr(index);
                                    self.write(".clone()");
                                } else {
                                    self.emit_expr(index);
                                }
                            } else {
                                self.emit_expr(index);
                            }
                        } else {
                            self.emit_expr(index);
                        }
                        self.write(", ");
                        self.emit_expr(value);
                        self.write(");\n");
                    }
                    _ => {
                        // Fallback: direct subscript
                        self.write(&field_access);
                        self.write("[");
                        self.emit_expr(index);
                        self.write("] = ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                }
            }
            HirStmt::Delete { object, index } => {
                let obj_ty = object.ty();
                self.write_indent();
                match obj_ty {
                    Type::Dict(_, _) => {
                        // del d[key] -> let _ = d.remove(&key);
                        self.write("let _ = ");
                        self.emit_expr(object);
                        self.write(".remove(");
                        self.emit_key_ref_expr(index);
                        self.write(");\n");
                    }
                    Type::List(_) => {
                        // del a[i] -> let _ = a.remove(i as usize);
                        self.write("let _ = ");
                        self.emit_expr(object);
                        self.write(".remove(");
                        self.emit_expr(index);
                        self.write(" as usize);\n");
                    }
                    _ => {
                        self.write("/* unsupported del */\n");
                    }
                }
            }
            HirStmt::Yield { value } => {
                if self.in_generator_closure {
                    // Inside a generator closure: yield becomes return Some(val)
                    self.write_indent();
                    self.write("return Some(");
                    self.emit_expr(value);
                    self.write(");\n");
                } else {
                    // Eager fallback: push to yields vec
                    self.write_indent();
                    self.write("_yields.push(");
                    self.emit_expr(value);
                    self.write(");\n");
                }
            }
            HirStmt::With { items, body } => {
                self.write_indent();
                self.write("{\n");
                self.indent += 1;
                // Emit each context manager item with Drop-based cleanup
                // This ensures __exit__() is called on ALL exit paths:
                // normal completion, early return, break, continue
                for (i, (var, value, has_cm)) in items.iter().enumerate() {
                    let ctx_name = format!("__ctx_{}", i);
                    let guard_type = format!("__WithGuard{}", i);
                    let guard_var = format!("__guard_{}", i);
                    if *has_cm {
                        // Extract the class type name for the guard struct
                        let class_name = if let Type::Class { name, .. } = value.ty() {
                            name.clone()
                        } else {
                            "Unknown".to_string()
                        };
                        // Create context manager variable
                        self.write_indent();
                        self.write("let mut ");
                        self.write(&ctx_name);
                        self.write(" = ");
                        self.emit_expr(value);
                        self.write(";\n");
                        // Emit Drop guard struct that calls __exit__() on scope exit
                        self.write_indent();
                        self.write(&format!("struct {} {{ ctx: {} }}\n", guard_type, class_name));
                        self.write_indent();
                        self.write(&format!("impl Drop for {} {{\n", guard_type));
                        self.indent += 1;
                        self.write_indent();
                        self.write("fn drop(&mut self) { self.ctx.__exit__(); }\n");
                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                        // Create guard instance, moving ctx into it
                        self.write_indent();
                        self.write(&format!("let mut {} = {} {{ ctx: {} }};\n", guard_var, guard_type, ctx_name));
                        // Call __enter__() on guard's ctx and bind result to var
                        self.write_indent();
                        if stmts_reference_var(body, var) || items.iter().any(|(v, _, _)| v != var && v.contains(var)) {
                            self.write("let ");
                            self.write(var);
                        } else {
                            self.write("let _");
                            self.write(var);
                        }
                        self.write(" = ");
                        self.write(&guard_var);
                        self.write(".ctx.__enter__();\n");
                    } else {
                        // Fallback: no context manager protocol, just bind directly
                        self.write_indent();
                        if stmts_reference_var(body, var) {
                            self.write("let ");
                            self.write(var);
                        } else {
                            self.write("let _");
                            self.write(var);
                        }
                        self.write(" = ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                }
                // Emit body
                for s in body {
                    self.emit_stmt(s);
                }
                // No explicit __exit__() calls needed — Drop guards handle cleanup
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
            HirStmt::NestedFunction { func } => {
                let saved_return_type = self.current_return_type.clone();
                let saved_mutated = self.mutated_vars.clone();

                self.current_return_type = Some(func.return_type.clone());
                self.mutated_vars = collect_mutated_vars(&func.body);

                // Collect the set of parameter names
                let param_names: HashSet<String> = func.params.iter().map(|p| p.name.clone()).collect();

                // Detect captured variables: variables referenced in body that are
                // not parameters and not defined locally in the body
                let referenced_with_types = collect_referenced_vars_with_types(&func.body);
                let locally_defined = collect_locally_defined_vars(&func.body);
                let captures: Vec<(String, Type)> = referenced_with_types.into_iter()
                    .filter(|(v, _)| !param_names.contains(v) && !locally_defined.contains(v))
                    .collect();

                // Check if the nested function calls itself (recursive)
                let is_recursive = body_calls_function(&func.body, &func.name);

                if captures.is_empty() {
                    // No captures: emit as a plain inner fn (works for both recursive and non-recursive)
                    self.write_indent();
                    self.write("fn ");
                    self.write(&func.name);
                    self.write("(");

                    for (i, param) in func.params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        if self.mutated_vars.contains(&param.name) {
                            self.write("mut ");
                        }
                        self.write(&param.name);
                        self.write(": ");
                        self.write(&param.ty.rust_type());
                    }

                    self.write(")");

                    if func.return_type != Type::None {
                        self.write(" -> ");
                        self.write(&func.return_type.rust_type());
                    }

                    self.write(" {\n");
                    self.indent += 1;

                    for s in &func.body {
                        self.emit_stmt(s);
                    }

                    self.indent -= 1;
                    self.writeln("}");
                } else if !is_recursive {
                    // Has captures but not recursive: emit as a closure
                    self.write_indent();
                    self.write("let ");
                    self.write(&func.name);
                    self.write(" = |");

                    for (i, param) in func.params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        if self.mutated_vars.contains(&param.name) {
                            self.write("mut ");
                        }
                        self.write(&param.name);
                        self.write(": ");
                        self.write(&param.ty.rust_type());
                    }

                    self.write("|");

                    if func.return_type != Type::None {
                        self.write(" -> ");
                        self.write(&func.return_type.rust_type());
                    }

                    self.write(" {\n");
                    self.indent += 1;

                    for s in &func.body {
                        self.emit_stmt(s);
                    }

                    self.indent -= 1;
                    self.writeln("};");
                } else {
                    // Recursive AND captures: emit as inner fn with captured vars as extra cloned params
                    // Store the capture info so call sites can pass the extra args
                    self.nested_fn_captures.insert(func.name.clone(), captures.clone());

                    self.write_indent();
                    self.write("fn ");
                    self.write(&func.name);
                    self.write("(");

                    for (i, param) in func.params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        if self.mutated_vars.contains(&param.name) {
                            self.write("mut ");
                        }
                        self.write(&param.name);
                        self.write(": ");
                        self.write(&param.ty.rust_type());
                    }

                    // Add captured variables as extra parameters with types
                    for (cap_name, cap_ty) in &captures {
                        self.write(", ");
                        self.write(cap_name);
                        self.write(": ");
                        self.write(&cap_ty.rust_type());
                    }

                    self.write(")");

                    if func.return_type != Type::None {
                        self.write(" -> ");
                        self.write(&func.return_type.rust_type());
                    }

                    self.write(" {\n");
                    self.indent += 1;

                    for s in &func.body {
                        self.emit_stmt(s);
                    }

                    self.indent -= 1;
                    self.writeln("}");
                }

                self.current_return_type = saved_return_type;
                self.mutated_vars = saved_mutated;
            }
            HirStmt::Match { subject, subject_ty, arms } => {
                self.emit_match(subject, subject_ty, arms);
            }
        }
    }

    fn substitute_class_captures_in_guard(&self, guard_code: &str, pattern: &HirPattern, is_non_option_union: bool) -> String {
        if let HirPattern::Class { fields, .. } = pattern {
            let prefix = if is_non_option_union { "__inner" } else { "__matched" };
            let mut result = guard_code.to_string();
            for (fname, fpat) in fields {
                if let HirPattern::Capture { name, .. } = fpat {
                    let replacement = format!("{}.{}", prefix, fname);
                    result = Self::replace_identifier(&result, name, &replacement);
                }
            }
            result
        } else {
            guard_code.to_string()
        }
    }

    fn replace_identifier(code: &str, ident: &str, replacement: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = code.chars().collect();
        let ident_chars: Vec<char> = ident.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if i + ident_chars.len() <= chars.len()
                && &chars[i..i + ident_chars.len()] == ident_chars.as_slice()
            {
                let before_ok = i == 0 || !chars[i - 1].is_alphanumeric() && chars[i - 1] != '_';
                let after_ok = i + ident_chars.len() >= chars.len()
                    || !chars[i + ident_chars.len()].is_alphanumeric() && chars[i + ident_chars.len()] != '_';
                if before_ok && after_ok {
                    result.push_str(replacement);
                    i += ident_chars.len();
                    continue;
                }
            }
            result.push(chars[i]);
            i += 1;
        }
        result
    }

    fn emit_match(&mut self, subject: &HirExpr, subject_ty: &Type, arms: &[HirMatchArm]) {
        // Determine how to emit the match based on subject type
        let is_option = is_option_type(subject_ty);
        let is_non_option_union = matches!(subject_ty, Type::Union(_)) && !is_option;

        self.write_indent();

        if is_option || is_non_option_union {
            // For union types, emit as a Rust match on the enum/Option
            let subject_code = self.expr_to_string(subject);
            self.write(&format!("match {} {{\n", subject_code));
        } else {
            // For simple types (literals, etc.), emit as a Rust match
            let subject_code = self.expr_to_string(subject);
            self.write(&format!("match {} {{\n", subject_code));
        }

        self.indent += 1;

        let mut has_wildcard = false;
        for arm in arms {
            if matches!(arm.pattern, HirPattern::Wildcard) {
                has_wildcard = true;
            }
            self.emit_match_arm(&arm.pattern, subject_ty, &arm.guard, &arm.body, is_option, is_non_option_union);
        }

        // If no wildcard and not a union type, add a wildcard arm to make it exhaustive
        if !has_wildcard && !is_option && !is_non_option_union {
            // Already handled by the arms themselves
        }

        self.indent -= 1;
        self.writeln("}");
    }

    fn emit_match_arm(
        &mut self,
        pattern: &HirPattern,
        subject_ty: &Type,
        guard: &Option<HirExpr>,
        body: &[HirStmt],
        is_option: bool,
        is_non_option_union: bool,
    ) {
        self.write_indent();

        // Build the pattern part (without =>)
        let has_str_guard = matches!(pattern, HirPattern::Literal { value: HirExpr::StringLiteral(_) })
            || matches!(pattern, HirPattern::Or { patterns } if patterns.iter().any(|p| matches!(p, HirPattern::Literal { value: HirExpr::StringLiteral(_) })));

        match pattern {
            HirPattern::Wildcard => {
                self.write("_");
            }
            HirPattern::None => {
                if is_option {
                    self.write("None");
                } else {
                    self.write("_");
                }
            }
            HirPattern::Capture { name, ty } => {
                if is_option {
                    let _ = ty;
                    self.write(&format!("Some({})", name));
                } else {
                    self.write(name);
                }
            }
            HirPattern::Literal { value } => {
                let lit_code = self.expr_to_string(value);
                match value {
                    HirExpr::StringLiteral(_) => {
                        // String matching needs a guard since Rust can't match String directly
                        self.write("__s");
                    }
                    _ => {
                        self.write(&lit_code);
                    }
                }
            }
            HirPattern::Or { patterns } => {
                let has_str = patterns.iter().any(|p| matches!(p, HirPattern::Literal { value: HirExpr::StringLiteral(_) }));
                if has_str {
                    self.write("__s");
                } else {
                    let mut parts = Vec::new();
                    for p in patterns {
                        match p {
                            HirPattern::Literal { value } => {
                                let lit_code = self.expr_to_string(value);
                                parts.push(lit_code);
                            }
                            HirPattern::None => parts.push("None".to_string()),
                            HirPattern::Wildcard => parts.push("_".to_string()),
                            HirPattern::Value { path } => {
                                parts.push(path.join("::"));
                            }
                            _ => parts.push("_".to_string()),
                        }
                    }
                    self.write(&parts.join(" | "));
                }
            }
            HirPattern::Class { class_name, fields } => {
                if is_non_option_union {
                    let enum_name = subject_ty.union_enum_name();
                    let variant_name = if let Type::Union(members) = subject_ty {
                        let target_ty = match class_name.as_str() {
                            "int" => Some(Type::Int),
                            "str" => Some(Type::Str),
                            "float" => Some(Type::Float),
                            "bool" => Some(Type::Bool),
                            other => members.iter().find(|m| {
                                matches!(m, Type::Class { name, .. } if name == other)
                            }).cloned(),
                        };
                        if let Some(ty) = target_ty {
                            ty.union_variant_name()
                        } else {
                            class_name.clone()
                        }
                    } else {
                        class_name.clone()
                    };
                    if fields.is_empty() {
                        self.write(&format!("{}::{}(_)", enum_name, variant_name));
                    } else {
                        self.write(&format!("{}::{}(__inner)", enum_name, variant_name));
                    }
                } else {
                    // For direct struct patterns, use __matched with field guards
                    self.write("__matched");
                }
            }
            HirPattern::Value { path } => {
                let rust_path = path.join("::");
                self.write(&rust_path);
            }
            HirPattern::Tuple { elements } => {
                self.write("(");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    match elem {
                        HirPattern::Capture { name, .. } => self.write(name),
                        HirPattern::Wildcard => self.write("_"),
                        HirPattern::Literal { value } => {
                            let lit_code = self.expr_to_string(value);
                            self.write(&lit_code);
                        }
                        _ => self.write("_"),
                    }
                }
                self.write(")");
            }
        }

        // Build field guards for class patterns with literal field values
        let class_field_guards: Vec<String> = if let HirPattern::Class { fields, .. } = pattern {
            if !is_non_option_union {
                fields.iter().filter_map(|(fname, fpat)| {
                    match fpat {
                        HirPattern::Literal { value } => {
                            let lit_code = self.expr_to_string(value);
                            Some(format!("__matched.{} == {}", fname, lit_code))
                        }
                        HirPattern::None => {
                            Some(format!("__matched.{}.is_none()", fname))
                        }
                        _ => None,
                    }
                }).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Add guard
        if has_str_guard {
            // Build string guard condition
            let str_guard = match pattern {
                HirPattern::Literal { value: HirExpr::StringLiteral(s) } => {
                    format!("__s == {:?}", s)
                }
                HirPattern::Or { patterns } => {
                    let conditions: Vec<String> = patterns.iter().map(|p| {
                        match p {
                            HirPattern::Literal { value: HirExpr::StringLiteral(s) } => {
                                format!("__s == {:?}", s)
                            }
                            _ => "__s == _".to_string(),
                        }
                    }).collect();
                    conditions.join(" || ")
                }
                _ => String::new(),
            };
            if let Some(guard_expr) = guard {
                let guard_code = self.expr_to_string(guard_expr);
                self.write(&format!(" if ({}) && ({})", str_guard, guard_code));
            } else {
                self.write(&format!(" if {}", str_guard));
            }
        } else if !class_field_guards.is_empty() {
            let mut all_guards = class_field_guards;
            if let Some(guard_expr) = guard {
                let mut guard_code = self.expr_to_string(guard_expr);
                guard_code = self.substitute_class_captures_in_guard(&guard_code, pattern, is_non_option_union);
                all_guards.push(guard_code);
            }
            self.write(&format!(" if {}", all_guards.join(" && ")));
        } else if let Some(guard_expr) = guard {
            let mut guard_code = self.expr_to_string(guard_expr);
            guard_code = self.substitute_class_captures_in_guard(&guard_code, pattern, is_non_option_union);
            self.write(&format!(" if {}", guard_code));
        }

        self.write(" => {\n");
        self.indent += 1;

        // For class patterns with fields on union types, destructure
        if let HirPattern::Class { class_name, fields } = pattern {
            if is_non_option_union && !fields.is_empty() {
                for (fname, fpat) in fields {
                    if let HirPattern::Capture { name, .. } = fpat {
                        self.write_indent();
                        self.write(&format!("let {} = __inner.{};\n", name, fname));
                    }
                }
            } else if !is_non_option_union {
                for (fname, fpat) in fields {
                    if let HirPattern::Capture { name, .. } = fpat {
                        self.write_indent();
                        self.write(&format!("let {} = __matched.{};\n", name, fname));
                    }
                }
            }
            let _ = class_name;
        }

        for s in body {
            self.emit_stmt(s);
        }

        self.indent -= 1;
        self.writeln("}");
    }

    fn expr_to_string(&mut self, expr: &HirExpr) -> String {
        // Fast path for expressions already supported by IR lowering.
        // This gradually removes reliance on the output-buffer swapping hack.
        if let Some(lowered_expr) = try_lower_leaf_expr(expr) {
            return crate::render_expr(&lowered_expr);
        }

        let saved_output = std::mem::take(&mut self.output);
        let saved_indent = self.indent;
        self.indent = 0;
        self.emit_expr(expr);
        let result = std::mem::take(&mut self.output);
        self.output = saved_output;
        self.indent = saved_indent;
        result.trim().to_string()
    }

    /// Emit any walrus (named expression) assignments that need to be hoisted before a condition.
    fn emit_walrus_hoists(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::WalrusExpr { name, value, ty } => {
                self.write_indent();
                self.write("let ");
                self.write(name);
                self.write(": ");
                self.write(&ty.rust_type());
                self.write(" = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            HirExpr::Compare { left, comparators, .. } => {
                self.emit_walrus_hoists(left);
                for c in comparators {
                    self.emit_walrus_hoists(c);
                }
            }
            HirExpr::BoolOp { values, .. } => {
                for v in values {
                    self.emit_walrus_hoists(v);
                }
            }
            HirExpr::BinOp { left, right, .. } => {
                self.emit_walrus_hoists(left);
                self.emit_walrus_hoists(right);
            }
            _ => {}
        }
    }

    fn emit_list_slice(&mut self, object: &HirExpr, start: &Option<Box<HirExpr>>, stop: &Option<Box<HirExpr>>, step: &Option<Box<HirExpr>>) {
        if let Some(step_expr) = step {
            // Step slicing
            self.write("{ let _v = &");
            self.emit_expr(object);
            self.write("; let _len = _v.len() as i64; let _step = ");
            self.emit_expr(step_expr);
            self.write("; ");

            // Resolve start
            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _s = ");
                self.emit_expr(s);
                self.write("; if _s < 0 { ((_len + _s).max(0)) as usize } else { (_s.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { 0 } else { (_len - 1) as usize }");
            }
            self.write("; ");

            // Resolve stop
            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _e = ");
                self.emit_expr(e);
                self.write("; if _e < 0 { ((_len + _e).max(0)) as usize } else { (_e.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }");
            }
            self.write("; ");

            // Build result
            self.write("let mut _result = Vec::new(); ");
            self.write("if _step > 0 { let mut _i = _start; while _i < _stop { if let Some(_el) = _v.get(_i) { _result.push(_el.clone()); } _i += _step as usize; } }");
            self.write(" else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { if _i >= 0 { if let Some(_el) = _v.get(_i as usize) { _result.push(_el.clone()); } } _i += _step; } }");
            self.write("; _result }");
        } else {
            // Simple slice without step
            self.write("{ let _v = &");
            self.emit_expr(object);
            self.write("; let _len = _v.len() as i64; ");

            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _s = ");
                self.emit_expr(s);
                self.write("; if _s < 0 { ((_len + _s).max(0)) as usize } else { (_s.min(_len)) as usize } }");
            } else {
                self.write("0_usize");
            }
            self.write("; ");

            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _e = ");
                self.emit_expr(e);
                self.write("; if _e < 0 { ((_len + _e).max(0)) as usize } else { (_e.min(_len)) as usize } }");
            } else {
                self.write("_len as usize");
            }
            self.write("; ");

            self.write("_v[_start.._stop].to_vec() }");
        }
    }

    fn emit_string_slice(&mut self, object: &HirExpr, start: &Option<Box<HirExpr>>, stop: &Option<Box<HirExpr>>, step: &Option<Box<HirExpr>>) {
        if let Some(step_expr) = step {
            self.write("{ let _s: Vec<char> = ");
            self.emit_expr(object);
            self.write(".chars().collect(); let _len = _s.len() as i64; let _step = ");
            self.emit_expr(step_expr);
            self.write("; ");

            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _sv = ");
                self.emit_expr(s);
                self.write("; if _sv < 0 { ((_len + _sv).max(0)) as usize } else { (_sv.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { 0 } else { (_len - 1) as usize }");
            }
            self.write("; ");

            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _ev = ");
                self.emit_expr(e);
                self.write("; if _ev < 0 { ((_len + _ev).max(0)) as usize } else { (_ev.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }");
            }
            self.write("; ");

            self.write("let mut _result = String::new(); ");
            self.write("if _step > 0 { let mut _i = _start; while _i < _stop { if let Some(&_ch) = _s.get(_i) { _result.push(_ch); } _i += _step as usize; } }");
            self.write(" else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { if _i >= 0 { if let Some(&_ch) = _s.get(_i as usize) { _result.push(_ch); } } _i += _step; } }");
            self.write("; _result }");
        } else {
            self.write("{ let _s = &");
            self.emit_expr(object);
            self.write("; let _len = _s.chars().count() as i64; ");

            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _sv = ");
                self.emit_expr(s);
                self.write("; if _sv < 0 { ((_len + _sv).max(0)) as usize } else { (_sv.min(_len)) as usize } }");
            } else {
                self.write("0_usize");
            }
            self.write("; ");

            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _ev = ");
                self.emit_expr(e);
                self.write("; if _ev < 0 { ((_len + _ev).max(0)) as usize } else { (_ev.min(_len)) as usize } }");
            } else {
                self.write("_len as usize");
            }
            self.write("; ");

            self.write("_s.chars().skip(_start).take(_stop - _start).collect::<String>() }");
        }
    }

    /// Check if an expression is a call to a generator function
    fn is_generator_call(&self, expr: &HirExpr) -> bool {
        if let HirExpr::Call { func, .. } = expr {
            self.generator_functions.contains(func)
        } else {
            false
        }
    }

    fn emit_method_call(&mut self, object: &HirExpr, method: &str, args: &[HirExpr]) {
        // For mutating methods on self.field, suppress .clone() so mutations are applied
        // to the actual field, not a temporary clone.
        let is_self_field = matches!(object, HirExpr::FieldAccess { object: inner, .. }
            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"));
        if is_self_field && MUTATING_METHODS.contains(&method) {
            self.suppress_field_clone = true;
        }
        let obj_ty = object.ty();
        match (obj_ty, method) {
            // String methods
            (Type::Str, "upper") => {
                self.emit_expr(object);
                self.write(".to_uppercase()");
            }
            (Type::Str, "lower") => {
                self.emit_expr(object);
                self.write(".to_lowercase()");
            }
            (Type::Str, "strip") => {
                self.emit_expr(object);
                self.write(".trim().to_string()");
            }
            (Type::Str, "lstrip") => {
                self.emit_expr(object);
                self.write(".trim_start().to_string()");
            }
            (Type::Str, "rstrip") => {
                self.emit_expr(object);
                self.write(".trim_end().to_string()");
            }
            (Type::Str, "startswith") => {
                self.emit_expr(object);
                self.write(".starts_with(");
                if !args.is_empty() {
                    self.emit_str_ref_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Str, "endswith") => {
                self.emit_expr(object);
                self.write(".ends_with(");
                if !args.is_empty() {
                    self.emit_str_ref_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Str, "split") => {
                self.emit_expr(object);
                if args.is_empty() {
                    self.write(".split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>()");
                } else {
                    self.write(".split(");
                    self.emit_str_ref_expr(&args[0]);
                    self.write(").map(|s| s.to_string()).collect::<Vec<String>>()");
                }
            }
            (Type::Str, "replace") => {
                self.emit_expr(object);
                self.write(".replace(");
                if args.len() >= 2 {
                    self.emit_str_ref_expr(&args[0]);
                    self.write(", ");
                    self.emit_str_ref_expr(&args[1]);
                }
                self.write(")");
            }
            (Type::Str, "find") => {
                // Returns Option<i64> = int | None
                self.emit_expr(object);
                self.write(".find(");
                if !args.is_empty() {
                    self.emit_str_ref_expr(&args[0]);
                }
                self.write(").map(|i| i as i64)");
            }
            // String methods - extended
            (Type::Str, "title") => {
                // Title case: capitalize first letter of each word
                self.emit_expr(object);
                self.write(".split_whitespace().map(|w| { let mut c = w.chars(); match c.next() { None => String::new(), Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase() } }).collect::<Vec<_>>().join(\" \")");
            }
            (Type::Str, "capitalize") => {
                self.write("{ let _s = ");
                self.emit_expr(object);
                self.write("; let mut _c = _s.chars(); match _c.next() { None => String::new(), Some(f) => f.to_uppercase().to_string() + &_c.as_str().to_lowercase() } }");
            }
            (Type::Str, "swapcase") => {
                self.emit_expr(object);
                self.write(".chars().map(|c| if c.is_uppercase() { c.to_lowercase().to_string() } else { c.to_uppercase().to_string() }).collect::<String>()");
            }
            (Type::Str, "isdigit") => {
                self.write("!");
                self.emit_expr(object);
                self.write(".is_empty() && ");
                self.emit_expr(object);
                self.write(".chars().all(|c| c.is_ascii_digit())");
            }
            (Type::Str, "isalpha") => {
                self.write("!");
                self.emit_expr(object);
                self.write(".is_empty() && ");
                self.emit_expr(object);
                self.write(".chars().all(|c| c.is_alphabetic())");
            }
            (Type::Str, "isalnum") => {
                self.write("!");
                self.emit_expr(object);
                self.write(".is_empty() && ");
                self.emit_expr(object);
                self.write(".chars().all(|c| c.is_alphanumeric())");
            }
            (Type::Str, "isspace") => {
                self.write("!");
                self.emit_expr(object);
                self.write(".is_empty() && ");
                self.emit_expr(object);
                self.write(".chars().all(|c| c.is_whitespace())");
            }
            (Type::Str, "isupper") => {
                self.emit_expr(object);
                self.write(".chars().any(|c| c.is_alphabetic()) && ");
                self.emit_expr(object);
                self.write(".chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase())");
            }
            (Type::Str, "islower") => {
                self.emit_expr(object);
                self.write(".chars().any(|c| c.is_alphabetic()) && ");
                self.emit_expr(object);
                self.write(".chars().filter(|c| c.is_alphabetic()).all(|c| c.is_lowercase())");
            }
            (Type::Str, "join") => {
                // Python: "sep".join(items) -> Rust: items.join("sep")
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                    self.write(".join(");
                    self.emit_str_ref_expr(object);
                    self.write(")");
                }
            }
            (Type::Str, "count") => {
                self.emit_expr(object);
                self.write(".matches(");
                if !args.is_empty() {
                    self.emit_str_ref_expr(&args[0]);
                }
                self.write(").count() as i64");
            }
            (Type::Str, "center") => {
                self.write("{ let _s = ");
                self.emit_expr(object);
                self.write("; let _w = ");
                if !args.is_empty() { self.emit_expr(&args[0]); }
                self.write(" as usize; let _len = _s.chars().count(); if _len >= _w { _s } else { let _pad = _w - _len; let _left = _pad / 2; let _right = _pad - _left; format!(\"{}{}{}\", \" \".repeat(_left), _s, \" \".repeat(_right)) } }");
            }
            (Type::Str, "ljust") => {
                self.write("format!(\"{:<width$}\", ");
                self.emit_expr(object);
                self.write(", width = ");
                if !args.is_empty() { self.emit_expr(&args[0]); }
                self.write(" as usize)");
            }
            (Type::Str, "rjust") => {
                self.write("format!(\"{:>width$}\", ");
                self.emit_expr(object);
                self.write(", width = ");
                if !args.is_empty() { self.emit_expr(&args[0]); }
                self.write(" as usize)");
            }
            (Type::Str, "zfill") => {
                self.write("format!(\"{:0>width$}\", ");
                self.emit_expr(object);
                self.write(", width = ");
                if !args.is_empty() { self.emit_expr(&args[0]); }
                self.write(" as usize)");
            }
            // VecDeque methods (deque class _data field)
            (Type::List(_), "append") if self.is_deque_data_field(object) => {
                self.emit_expr(object);
                self.write(".push_back(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                    if matches!(args[0].ty(), Type::TypeVar(_)) {
                        self.write(".clone()");
                    }
                }
                self.write(")");
            }
            (Type::List(_), "appendleft") if self.is_deque_data_field(object) => {
                self.emit_expr(object);
                self.write(".push_front(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                    if matches!(args[0].ty(), Type::TypeVar(_)) {
                        self.write(".clone()");
                    }
                }
                self.write(")");
            }
            (Type::List(_), "pop") if self.is_deque_data_field(object) => {
                self.emit_expr(object);
                self.write(".pop_back()");
            }
            (Type::List(_), "popleft") if self.is_deque_data_field(object) => {
                self.emit_expr(object);
                self.write(".pop_front()");
            }
            // List methods
            (Type::List(_), "append") => {
                self.emit_expr(object);
                self.write(".push(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                    // Clone TypeVar arguments to avoid move issues in loops
                    if matches!(args[0].ty(), Type::TypeVar(_)) {
                        self.write(".clone()");
                    }
                }
                self.write(")");
            }
            (Type::List(_), "extend") => {
                self.emit_expr(object);
                self.write(".extend(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::List(_), "insert") => {
                self.emit_expr(object);
                self.write(".insert(");
                if args.len() >= 2 {
                    self.emit_expr(&args[0]);
                    self.write(" as usize, ");
                    // If the value arg is a borrowed/mut-borrowed param with Move ownership,
                    // we need to clone it since Vec::insert requires an owned value.
                    let needs_clone = if let HirExpr::Name { name, ty } = &args[1] {
                        (self.borrowed_params.contains(name.as_str()) || self.mut_borrowed_params.contains(name.as_str()))
                            && ty.ownership() != sifr_type_system::OwnershipKind::Copy
                    } else {
                        false
                    };
                    self.emit_expr(&args[1]);
                    if needs_clone {
                        self.write(".clone()");
                    }
                }
                self.write(")");
            }
            (Type::List(_), "clear") => {
                self.emit_expr(object);
                self.write(".clear()");
            }
            (Type::List(_), "copy") => {
                self.emit_expr(object);
                self.write(".clone()");
            }
            (Type::List(_), "reverse") => {
                self.emit_expr(object);
                self.write(".reverse()");
            }
            (Type::List(_), "sort") => {
                self.emit_expr(object);
                self.write(".sort()");
            }
            (Type::List(_), "count") => {
                self.emit_expr(object);
                self.write(".iter().filter(|x| **x == ");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(").count() as i64");
            }
            (Type::List(_), "contains") => {
                self.emit_expr(object);
                self.write(".contains(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::List(_), "pop") => {
                // Returns Option<T> = T | None
                self.emit_expr(object);
                self.write(".pop()");
            }
            (Type::List(_), "remove") => {
                // list.remove(val) -> no-op if not found (safe: no panic)
                self.write("{ if let Some(__pos) = ");
                self.emit_expr(object);
                self.write(".iter().position(|__x| *__x == ");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(") { ");
                self.emit_expr(object);
                self.write(".remove(__pos); } }");
            }
            (Type::List(_), "index") => {
                // list.index(val) -> Option[int]: Some(pos) or None
                self.emit_expr(object);
                self.write(".iter().position(|__x| *__x == ");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(").map(|__p| __p as i64)");
            }
            // Dict methods
            (Type::Dict(_, _), "keys") => {
                self.emit_expr(object);
                self.write(".keys().cloned().collect::<Vec<_>>()");
            }
            (Type::Dict(_, _), "values") => {
                self.emit_expr(object);
                self.write(".values().cloned().collect::<Vec<_>>()");
            }
            (Type::Dict(_, _), "items") => {
                self.emit_expr(object);
                self.write(".iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()");
            }
            (Type::Dict(_, _), "update") => {
                self.emit_expr(object);
                self.write(".extend(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Dict(_, _), "clear") => {
                self.emit_expr(object);
                self.write(".clear()");
            }
            (Type::Dict(_, _), "copy") => {
                self.emit_expr(object);
                self.write(".clone()");
            }
            (Type::Dict(_, _), "contains") => {
                self.emit_expr(object);
                self.write(".contains_key(");
                if !args.is_empty() {
                    self.emit_key_ref_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Dict(_, _), "get") => {
                if args.len() == 2 {
                    // dict.get(key, default) -> d.get(&key).cloned().unwrap_or(default)
                    self.emit_expr(object);
                    self.write(".get(");
                    self.emit_key_ref_expr(&args[0]);
                    self.write(").cloned().unwrap_or(");
                    self.emit_expr(&args[1]);
                    self.write(")");
                } else {
                    // dict.get(key) -> d.get(&key).cloned() (returns Option<V>)
                    self.emit_expr(object);
                    self.write(".get(");
                    if !args.is_empty() {
                        self.emit_key_ref_expr(&args[0]);
                    }
                    self.write(").cloned()");
                }
            }
            (Type::Dict(_, _), "pop") => {
                // Returns Option<V> = V | None
                self.emit_expr(object);
                self.write(".remove(");
                if !args.is_empty() {
                    self.emit_key_ref_expr(&args[0]);
                }
                self.write(")");
            }
            // Set methods
            (Type::Set(_), "add") => {
                self.emit_expr(object);
                self.write(".insert(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Set(_), "remove") => {
                self.emit_expr(object);
                self.write(".remove(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Set(_), "discard") => {
                self.emit_expr(object);
                self.write(".remove(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Set(_), "contains") => {
                self.emit_expr(object);
                self.write(".contains(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Set(_), "clear") => {
                self.emit_expr(object);
                self.write(".clear()");
            }
            (Type::Set(_), "copy") => {
                self.emit_expr(object);
                self.write(".clone()");
            }
            (Type::Set(_), "union") => {
                self.emit_expr(object);
                self.write(".union(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(").cloned().collect::<HashSet<_>>()");
                self.needs_hashset = true;
            }
            (Type::Set(_), "intersection") => {
                self.emit_expr(object);
                self.write(".intersection(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(").cloned().collect::<HashSet<_>>()");
                self.needs_hashset = true;
            }
            (Type::Set(_), "difference") => {
                self.emit_expr(object);
                self.write(".difference(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(").cloned().collect::<HashSet<_>>()");
                self.needs_hashset = true;
            }
            (Type::Set(_), "symmetric_difference") => {
                self.emit_expr(object);
                self.write(".symmetric_difference(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(").cloned().collect::<HashSet<_>>()");
                self.needs_hashset = true;
            }
            (Type::Set(_), "issubset") => {
                self.emit_expr(object);
                self.write(".is_subset(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Set(_), "issuperset") => {
                self.emit_expr(object);
                self.write(".is_superset(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Set(_), "isdisjoint") => {
                self.emit_expr(object);
                self.write(".is_disjoint(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Set(_), "pop") => {
                // set.pop() -> Option[T]: returns None on empty set (safe: no panic)
                self.write("{ let __v = ");
                self.emit_expr(object);
                self.write(".iter().next().cloned(); if let Some(ref __val) = __v { ");
                self.emit_expr(object);
                self.write(".remove(__val); } __v }");
            }
            (Type::Set(_), "len") => {
                self.write("(");
                self.emit_expr(object);
                self.write(".len() as i64)");
            }
            // Tuple count()
            (Type::Tuple(_), "count") => {
                // For tuples, count is tricky - we need to check each element
                // For now, emit a simple comparison chain
                self.write("0_i64 /* tuple.count() not fully supported */");
            }
            // Tuple len() - compile-time constant
            (Type::Tuple(elems), "len") => {
                self.write(&format!("{}_i64", elems.len()));
            }
            // String len() - character count
            (Type::Str, "len") => {
                self.write("(");
                self.emit_expr(object);
                self.write(".chars().count() as i64)");
            }
            // len() on Option types (T|None) - unwrap first
            (ty, "len") if is_option_type(ty) => {
                self.write("(");
                self.emit_expr(object);
                self.write(".as_ref().unwrap().len() as i64)");
            }
            // Generic len() for all types
            (_, "len") => {
                self.write("(");
                self.emit_expr(object);
                self.write(".len() as i64)");
            }
            (Type::Class { name: ref class_name, fields, methods, .. }, _) => {
                // Check if this is a callable field invocation (not a real method)
                let is_callable_field = !methods.iter().any(|(n, _)| n == method)
                    && fields.iter().any(|(n, t)| n == method && matches!(t, Type::Callable(..)));

                if is_callable_field {
                    // Callable field: emit (obj.field)(args) instead of obj.method(args)
                    self.write("(");
                    self.emit_expr(object);
                    self.write(&format!(".{})(", method));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.emit_expr(arg);
                    }
                    self.write(")");
                } else {
                    // Regular class instance method call -- use convention-aware argument emission
                    self.emit_expr(object);
                    self.write(&format!(".{}(", method));
                    // Look up method conventions from func_signatures
                    let method_key = format!("{}::{}", class_name, method);
                    let method_info = self.func_signatures.get(&method_key).cloned();
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        if let Some((ref params, _)) = method_info {
                            // Method params skip self, so param index i corresponds to params[i]
                            // (self is not in func_signatures params)
                            if let Some((param_ty, convention)) = params.get(i) {
                                self.emit_borrow_prefix(*convention, arg.ty(), Some(param_ty));
                                self.emit_expr(arg);
                                continue;
                            }
                        }
                        // Fallback: emit as-is
                        self.emit_expr(arg);
                    }
                    self.write(")");
                }
            }
            _ => {
                // Fallback: emit as-is
                self.emit_expr(object);
                self.write(&format!(".{}(", method));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
        }
    }

    /// Emit `&` or `&mut` prefix for a function argument based on parameter convention.
    /// Copy types never get a borrow prefix (they're passed by value),
    /// unless the parameter type is a TypeVar (generic), in which case we always borrow.
    fn emit_borrow_prefix(&mut self, convention: ParamConvention, arg_ty: &Type, param_ty: Option<&Type>) {
        self.emit_borrow_prefix_for_name(convention, arg_ty, param_ty, None);
    }

    fn emit_borrow_prefix_for_name(&mut self, convention: ParamConvention, arg_ty: &Type, param_ty: Option<&Type>, arg_name: Option<&str>) {
        // Own convention: pass by value (move), no prefix needed
        if convention == ParamConvention::Own {
            return;
        }
        // If the parameter type is a TypeVar, always emit the borrow prefix
        // because the generated Rust signature uses &T for borrowed TypeVar params
        let is_generic_param = param_ty.map_or(false, |t| matches!(t, Type::TypeVar(_)));
        // Copy types are always passed by value regardless of convention,
        // unless the parameter is generic (TypeVar)
        if !is_generic_param && arg_ty.ownership() == sifr_type_system::OwnershipKind::Copy {
            return;
        }
        // If the argument is already a borrowed parameter (&T), don't add another borrow.
        // This handles the case where a Callable call passes a borrowed param:
        //   fn apply(f: Callable[[list[int]], int], items: &Vec<i64>) { f(items) }
        // Here items is already &Vec<i64>, so we pass it as-is (no extra &).
        //
        // Similarly, if the argument is already a mutably borrowed parameter (&mut T),
        // don't add another &mut. E.g.:
        //   fn heapify(data: &mut Vec<i64>) { _sift_down(data, 0, n); }
        // Here data is already &mut Vec<i64>; passing &mut data would be &&mut Vec<i64> error.
        if let Some(name) = arg_name {
            if self.borrowed_params.contains(name) && convention == ParamConvention::Borrow {
                return; // already &T, no additional borrow needed
            }
            if self.mut_borrowed_params.contains(name) {
                if convention == ParamConvention::MutBorrow {
                    return; // already &mut T, no additional &mut needed
                }
                if convention == ParamConvention::Borrow {
                    return; // &mut T -> &T is implicit reborrow in Rust; no extra & needed
                }
            }
        }
        match convention {
            ParamConvention::Borrow => self.write("&"),
            ParamConvention::MutBorrow => self.write("&mut "),
            ParamConvention::Own => {} // no prefix -- pass by value (move)
        }
    }

    fn emit_expr(&mut self, expr: &HirExpr) {
        if let Some(lowered_expr) = try_lower_leaf_expr(expr) {
            self.write(&crate::render_expr(&lowered_expr));
            return;
        }

        match expr {
            HirExpr::IntLiteral(val) => {
                self.write(&val.to_string());
                self.write("_i64");
            }
            HirExpr::FloatLiteral(val) => {
                let s = val.to_string();
                self.write(&s);
                if !s.contains('.') {
                    self.write(".0");
                }
                self.write("_f64");
            }
            HirExpr::StringLiteral(val) => {
                self.write(&format!("{:?}.to_string()", val));
            }
            HirExpr::BoolLiteral(val) => {
                self.write(if *val { "true" } else { "false" });
            }
            HirExpr::NoneLiteral => {
                // None in sifr maps to Rust's None (for Option contexts)
                // The parent (Let/Return) handles the wrapping context
                self.write("None");
            }
            HirExpr::Name { name, .. } => {
                // Check for stdlib constants
                if self.intrinsic_functions.contains(name.as_str()) || self.is_stdlib_constant(name) {
                    self.emit_stdlib_constant(name);
                } else if let Some((_ty, rust_name)) = self.module_constants.get(name).cloned() {
                    // Module-level constant
                    self.write(&rust_name);
                } else {
                    self.write(name);
                }
            }
            HirExpr::BinOp { left, op, right, ty } => {
                // BigInt arithmetic: always clone operands to avoid move issues
                if left.ty() == &Type::BigInt && right.ty() == &Type::BigInt && op != "**" {
                    if op == "//" {
                        // BigInt floor division uses /
                        self.emit_expr_with_bigint_clone(left);
                        self.write(" / ");
                        self.emit_expr_with_bigint_clone(right);
                    } else {
                        self.emit_expr_with_bigint_clone(left);
                        self.write(&format!(" {} ", op));
                        self.emit_expr_with_bigint_clone(right);
                    }
                    return;
                }
                // Special handling for string concatenation
                if op == "+" && *ty == Type::Str {
                    // Flatten chained string concatenation into a single format! call
                    // Fold string literals directly into the format string
                    let mut parts: Vec<&HirExpr> = Vec::new();
                    collect_string_concat_parts(left, &mut parts);
                    collect_string_concat_parts(right, &mut parts);
                    let mut format_str = String::new();
                    let mut format_args: Vec<&HirExpr> = Vec::new();
                    for part in &parts {
                        if let HirExpr::StringLiteral(val) = part {
                            // Fold literal directly into format string
                            format_str.push_str(val);
                        } else {
                            format_str.push_str("{}");
                            format_args.push(part);
                        }
                    }
                    if format_args.is_empty() {
                        // All parts are literals, just emit a string literal
                        self.write(&format!("\"{}\".to_string()", format_str));
                    } else {
                        self.write(&format!("format!(\"{}\"", format_str));
                        for arg in &format_args {
                            self.write(", ");
                            self.emit_expr(arg);
                        }
                        self.write(")");
                    }
                } else if op == "+" && matches!(ty, Type::List(_)) {
                    // List concatenation: a + b -> { let mut tmp = a.clone(); tmp.extend(b.iter().cloned()); tmp }
                    self.write("{ let mut __tmp = ");
                    self.emit_expr(left);
                    self.write(".clone(); __tmp.extend(");
                    self.emit_expr(right);
                    self.write(".iter().cloned()); __tmp }");
                } else if op == "//" {
                    // Floor division (int // int -> int division in Rust)
                    // Wrap sub-expressions in parens if they are BinOps to preserve precedence
                    if matches!(left.as_ref(), HirExpr::BinOp { .. }) { self.write("("); }
                    self.emit_expr(left);
                    if matches!(left.as_ref(), HirExpr::BinOp { .. }) { self.write(")"); }
                    self.write(" / ");
                    if matches!(right.as_ref(), HirExpr::BinOp { .. }) { self.write("("); }
                    self.emit_expr(right);
                    if matches!(right.as_ref(), HirExpr::BinOp { .. }) { self.write(")"); }
                } else if op == "**" {
                    // Power: int ** int -> i64::pow, otherwise float
                    if left.ty() == &Type::BigInt {
                        // bigint ** bigint or bigint ** int -> num_bigint pow
                        self.emit_expr(left);
                        self.write(".pow(u32::try_from(");
                        self.emit_expr(right);
                        self.write(").unwrap_or(0))");
                    } else if left.ty() == &Type::Int && right.ty() == &Type::Int {
                        self.emit_expr(left);
                        self.write(".pow(");
                        self.emit_expr(right);
                        self.write(" as u32)");
                    } else if left.ty() == &Type::Float && right.ty() == &Type::Int {
                        self.emit_expr(left);
                        self.write(".powi(");
                        self.emit_expr(right);
                        self.write(" as i32)");
                    } else {
                        self.write("(");
                        self.emit_expr(left);
                        self.write(" as f64).powf(");
                        self.emit_expr(right);
                        self.write(" as f64)");
                    }
                } else if op == "*" && left.ty() == &Type::Str && right.ty() == &Type::Int {
                    // String multiplication: "abc" * 3 -> "abc".repeat(3)
                    self.emit_expr(left);
                    self.write(".repeat(");
                    self.emit_expr(right);
                    self.write(" as usize)");
                } else if op == "*" && left.ty() == &Type::Int && right.ty() == &Type::Str {
                    // Reverse string multiplication: 3 * "abc"
                    self.emit_expr(right);
                    self.write(".repeat(");
                    self.emit_expr(left);
                    self.write(" as usize)");
                } else if op == "/" && left.ty() == &Type::Int && right.ty() == &Type::Int {
                    // Python: int / int -> float (true division)
                    // Rust: i64 / i64 -> i64 (integer division)
                    // Fix: cast both to f64 for true division
                    self.write("(");
                    self.emit_expr(left);
                    self.write(" as f64) / (");
                    self.emit_expr(right);
                    self.write(" as f64)");
                } else if matches!(left.ty(), Type::Class { .. }) {
                    // Class type with operator overloading: use reference-based ops
                    self.write("&");
                    self.emit_expr(left);
                    self.write(&format!(" {} ", op));
                    self.write("&");
                    self.emit_expr(right);
                } else if is_option_type(left.ty()) || is_option_type(right.ty()) {
                    // Union/optional arithmetic: unwrap Option with .unwrap()
                    if is_option_type(left.ty()) {
                        self.emit_expr(left);
                        self.write(".unwrap()");
                    } else {
                        self.emit_expr(left);
                    }
                    self.write(&format!(" {} ", op));
                    if is_option_type(right.ty()) {
                        self.emit_expr(right);
                        self.write(".unwrap()");
                    } else {
                        self.emit_expr(right);
                    }
                } else {
                    // Handle mixed int/float arithmetic: cast int side to f64
                    let left_is_int = left.ty() == &Type::Int;
                    let right_is_int = right.ty() == &Type::Int;
                    let left_is_float = left.ty() == &Type::Float;
                    let right_is_float = right.ty() == &Type::Float;
                    let needs_left_cast = left_is_int && right_is_float;
                    let needs_right_cast = right_is_int && left_is_float;

                    // Wrap sub-expressions in parens if they are BinOps to preserve precedence
                    let needs_left_parens = matches!(left.as_ref(), HirExpr::BinOp { .. });
                    let needs_right_parens = matches!(right.as_ref(), HirExpr::BinOp { .. });
                    if needs_left_parens || needs_left_cast { self.write("("); }
                    self.emit_expr(left);
                    if needs_left_parens || needs_left_cast { self.write(")"); }
                    if needs_left_cast { self.write(" as f64"); }
                    self.write(&format!(" {} ", op));
                    if needs_right_parens || needs_right_cast { self.write("("); }
                    self.emit_expr(right);
                    if needs_right_parens || needs_right_cast { self.write(")"); }
                    if needs_right_cast { self.write(" as f64"); }
                }
            }
            HirExpr::UnaryOp { op, operand, .. } => {
                if op == "not" {
                    // Collection truthiness: `not list_var` -> `list_var.is_empty()`
                    let is_collection = matches!(
                        operand.ty(),
                        Type::List(_) | Type::Dict(_, _) | Type::Set(_) | Type::Tuple(_) | Type::Str
                    );
                    if is_collection {
                        self.emit_expr(operand);
                        self.write(".is_empty()");
                    } else if matches!(operand.ty(), Type::Union(_)) {
                        // Optional truthiness: `not x` where x is T|None -> `x.is_none()`
                        self.emit_expr(operand);
                        self.write(".is_none()");
                    } else {
                        self.write("!");
                        self.emit_expr(operand);
                    }
                } else if op == "~" {
                    // Bitwise invert maps to `!` in Rust
                    self.write("!");
                    self.emit_expr(operand);
                } else if op == "+" {
                    // Unary + is a no-op in Python/Rust, just emit the operand
                    self.emit_expr(operand);
                } else {
                    self.write(op);
                    self.emit_expr(operand);
                }
            }
            HirExpr::Compare { left, ops, comparators, .. } => {
                // For single comparison
                if ops.len() == 1 {
                    let op = &ops[0];
                    // Handle `is None` / `is not None` for Option types
                    if (op == "is" || op == "is not") && matches!(comparators[0], HirExpr::NoneLiteral) {
                        // If left is already Type::None (not T|None), it's always None
                        if matches!(left.ty(), Type::None) {
                            if op == "is" {
                                self.write("true");
                            } else {
                                self.write("false");
                            }
                        } else {
                            self.emit_expr(left);
                            if op == "is" {
                                self.write(".is_none()");
                            } else {
                                self.write(".is_some()");
                            }
                        }
                    } else if op == "is" {
                        self.emit_expr(left);
                        self.write(" == ");
                        self.emit_expr(&comparators[0]);
                    } else if op == "is not" {
                        self.emit_expr(left);
                        self.write(" != ");
                        self.emit_expr(&comparators[0]);
                    } else {
                        // Handle Option<T> vs T comparisons: wrap T in Some()
                        let left_is_option = is_option_type(left.ty());
                        let right_is_option = is_option_type(comparators[0].ty());
                        if left_is_option && !right_is_option && !matches!(comparators[0], HirExpr::NoneLiteral) {
                            self.emit_expr(left);
                            self.write(&format!(" {} Some(", op));
                            self.emit_expr(&comparators[0]);
                            self.write(")");
                        } else if !left_is_option && right_is_option && !matches!(left.as_ref(), HirExpr::NoneLiteral) {
                            self.write("Some(");
                            self.emit_expr(left);
                            self.write(")");
                            self.write(&format!(" {} ", op));
                            self.emit_expr(&comparators[0]);
                        } else {
                            // Dereference borrowed params in comparisons to avoid &String == String
                            self.emit_expr_for_compare(left);
                            self.write(&format!(" {} ", op));
                            self.emit_expr_for_compare(&comparators[0]);
                        }
                    }
                } else {
                    // Chained comparisons: a < b < c -> a < b && b < c
                    self.write("(");
                    self.emit_expr(left);
                    self.write(&format!(" {} ", ops[0]));
                    self.emit_expr(&comparators[0]);
                    for i in 1..ops.len() {
                        self.write(" && ");
                        self.emit_expr(&comparators[i - 1]);
                        self.write(&format!(" {} ", ops[i]));
                        self.emit_expr(&comparators[i]);
                    }
                    self.write(")");
                }
            }
            HirExpr::BoolOp { op, values, .. } => {
                let rust_op = if op == "and" { "&&" } else { "||" };
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        self.write(&format!(" {} ", rust_op));
                    }
                    self.emit_expr(val);
                }
            }
            HirExpr::Call { func, args, .. } => {
                if func == "print" {
                    // Map print() to println!
                    if args.is_empty() {
                        self.write("println!()");
                    } else if matches!(args[0], HirExpr::NoneLiteral) || matches!(args[0].ty(), Type::None) {
                        // print(None) -> println!("None")
                        self.write("println!(\"None\")");
                    } else if let HirExpr::StringLiteral(val) = &args[0] {
                        // Inline string literal directly: println!("hello") instead of println!("{}", "hello")
                        // Escape backslashes and double quotes for valid Rust string
                        let escaped = val.replace('\\', "\\\\").replace('"', "\\\"").replace('{', "{{").replace('}', "}}");
                        self.write(&format!("println!(\"{}\")", escaped));
                    } else if let HirExpr::FString { parts, .. } = &args[0] {
                        // Inline f-string directly into println! to avoid double-format
                        self.emit_fstring_macro("println!", parts);
                    } else if matches!(args[0].ty(), Type::Class { .. } | Type::Newtype { .. }) {
                        // Check if class has Display impl
                        let class_name = match args[0].ty() {
                            Type::Class { name, .. } | Type::Newtype { name, .. } => name.clone(),
                            _ => String::new(),
                        };
                        if self.display_classes.contains(&class_name) {
                            self.write("println!(\"{}\", ");
                        } else {
                            self.write("println!(\"{:?}\", ");
                        }
                        self.emit_expr(&args[0]);
                        self.write(")");
                    } else if matches!(args[0].ty(), Type::List(_) | Type::Dict(_, _) | Type::Tuple(_) | Type::Set(_)) {
                        // Collections use Debug format
                        self.write("println!(\"{:?}\", ");
                        self.emit_expr(&args[0]);
                        self.write(")");
                    } else {
                        // Use emit_display_expr for all other cases:
                        // - Option<T> gets map_or wrapping
                        // - String literals omit .to_string()
                        // - Everything else emits normally
                        self.write("println!(\"{}\", ");
                        self.emit_display_expr(&args[0]);
                        self.write(")");
                    };
                } else if func == "isinstance" {
                    // isinstance() is handled by narrowing at the HIR level.
                    // At codegen time, we emit `true` since the narrowing has
                    // already validated the types. In practice, isinstance checks
                    // appear in if-conditions and the narrowing determines which
                    // branch to take.
                    self.write("true");
                } else if func == "str" {
                    // str() conversion -> format!("{}", arg) or format!("{:?}", arg) for lists
                    if !args.is_empty() {
                        if matches!(args[0].ty(), Type::List(_)) {
                            self.write("format!(\"{:?}\", ");
                        } else {
                            self.write("format!(\"{}\", ");
                        }
                        self.emit_display_expr(&args[0]);
                        self.write(")");
                    } else {
                        self.write("String::new()");
                    }
                } else if func == "pow" {
                    // pow(base, exp)
                    if args.len() == 2 {
                        if args[0].ty() == &Type::Int && args[1].ty() == &Type::Int {
                            self.emit_expr(&args[0]);
                            self.write(".pow(");
                            self.emit_expr(&args[1]);
                            self.write(" as u32)");
                        } else {
                            self.write("(");
                            self.emit_expr(&args[0]);
                            self.write(" as f64).powf(");
                            self.emit_expr(&args[1]);
                            self.write(" as f64)");
                        }
                    }
                } else if func == "abs" {
                    if !args.is_empty() {
                        self.write("(");
                        self.emit_expr(&args[0]);
                        self.write(").abs()");
                    }
                } else if func == "hash" {
                    // hash(x) -> { use std::hash::{Hash, Hasher}; let mut h = std::collections::hash_map::DefaultHasher::new(); x.hash(&mut h); h.finish() as i64 }
                    if !args.is_empty() {
                        self.write("{ use std::hash::{Hash, Hasher}; let mut _h = std::collections::hash_map::DefaultHasher::new(); ");
                        self.emit_expr(&args[0]);
                        self.write(".hash(&mut _h); _h.finish() as i64 }");
                    }
                } else if func == "round" {
                    if args.len() == 1 {
                        self.emit_expr(&args[0]);
                        self.write(".round() as i64");
                    } else if args.len() == 2 {
                        // round(x, n) -> (x * 10^n).round() / 10^n
                        self.write("((");
                        self.emit_expr(&args[0]);
                        self.write(" as f64 * 10.0_f64.powi(");
                        self.emit_expr(&args[1]);
                        self.write(" as i32)).round() / 10.0_f64.powi(");
                        self.emit_expr(&args[1]);
                        self.write(" as i32))");
                    }
                } else if func == "repr" {
                    if !args.is_empty() {
                        self.write("format!(\"{:?}\", ");
                        self.emit_expr(&args[0]);
                        self.write(")");
                    }
                } else if func == "int" {
                    if !args.is_empty() {
                        match args[0].ty() {
                            Type::Float => {
                                self.write("(");
                                self.emit_expr(&args[0]);
                                self.write(") as i64");
                            }
                            Type::Str => {
                                // int(str) -> Result<i64, ParseError>
                                self.emit_expr(&args[0]);
                                self.write(".parse::<i64>().map_err(|e| ParseError { message: e.to_string() })");
                            }
                            Type::Bool => {
                                self.write("if ");
                                self.emit_expr(&args[0]);
                                self.write(" { 1_i64 } else { 0_i64 }");
                            }
                            Type::BigInt => {
                                // int(bigint) -> Result<i64, OverflowError>
                                self.write("i64::try_from(&");
                                self.emit_expr(&args[0]);
                                self.write(").map_err(|_| OverflowError { message: \"bigint value out of range for int\".to_string() })");
                            }
                            _ => {
                                self.emit_expr(&args[0]);
                            }
                        }
                    }
                } else if func == "bigint" {
                    if !args.is_empty() {
                        // bigint(n) -> BigInt::from(n)
                        self.write("BigInt::from(");
                        self.emit_expr(&args[0]);
                        self.write(")");
                    }
                } else if func == "float" {
                    if !args.is_empty() {
                        match args[0].ty() {
                            Type::Int => {
                                self.write("(");
                                self.emit_expr(&args[0]);
                                self.write(") as f64");
                            }
                            Type::Str => {
                                // float(str) -> Result<f64, ParseError>
                                self.emit_expr(&args[0]);
                                self.write(".parse::<f64>().map_err(|e| ParseError { message: e.to_string() })");
                            }
                            _ => {
                                self.emit_expr(&args[0]);
                            }
                        }
                    }
                } else if func == "bool" {
                    if !args.is_empty() {
                        match args[0].ty() {
                            Type::Int => {
                                self.emit_expr(&args[0]);
                                self.write(" != 0");
                            }
                            Type::Float => {
                                self.emit_expr(&args[0]);
                                self.write(" != 0.0");
                            }
                            Type::Str | Type::List(_) | Type::Dict(_, _) => {
                                self.write("!");
                                self.emit_expr(&args[0]);
                                self.write(".is_empty()");
                            }
                            Type::Tuple(elems) => {
                                // Non-empty tuples are always truthy, empty tuples are falsy
                                if elems.is_empty() {
                                    self.write("false");
                                } else {
                                    self.write("true");
                                }
                            }
                            Type::Bool => {
                                self.emit_expr(&args[0]);
                            }
                            Type::None => {
                                self.write("false");
                            }
                            _ => {
                                self.emit_expr(&args[0]);
                            }
                        }
                    }
                } else if func == "min" {
                    if args.len() == 2 {
                        // min(a, b) -> std::cmp::min(a, b) or a.min(b) for floats
                        if matches!(args[0].ty(), Type::Float) {
                            self.emit_expr(&args[0]);
                            self.write(".min(");
                            self.emit_expr(&args[1]);
                            self.write(")");
                        } else {
                            self.write("std::cmp::min(");
                            self.emit_expr(&args[0]);
                            self.write(", ");
                            self.emit_expr(&args[1]);
                            self.write(")");
                        }
                    } else if matches!(args[0].ty(), Type::List(ref e) if matches!(e.as_ref(), Type::Float)) {
                        // min(list[float]) -> Option[float] (safe: None on empty)
                        self.emit_expr(&args[0]);
                        self.write(".iter().cloned().reduce(f64::min)");
                    } else {
                        // min(list[T]) -> Option[T] (safe: None on empty)
                        self.emit_expr(&args[0]);
                        self.write(".iter().min().cloned()");
                    }
                } else if func == "max" {
                    if args.len() == 2 {
                        // max(a, b) -> std::cmp::max(a, b) or a.max(b) for floats
                        if matches!(args[0].ty(), Type::Float) {
                            self.emit_expr(&args[0]);
                            self.write(".max(");
                            self.emit_expr(&args[1]);
                            self.write(")");
                        } else {
                            self.write("std::cmp::max(");
                            self.emit_expr(&args[0]);
                            self.write(", ");
                            self.emit_expr(&args[1]);
                            self.write(")");
                        }
                    } else if matches!(args[0].ty(), Type::List(ref e) if matches!(e.as_ref(), Type::Float)) {
                        // max(list[float]) -> Option[float] (safe: None on empty)
                        self.emit_expr(&args[0]);
                        self.write(".iter().cloned().reduce(f64::max)");
                    } else {
                        // max(list[T]) -> Option[T] (safe: None on empty)
                        self.emit_expr(&args[0]);
                        self.write(".iter().max().cloned()");
                    }
                } else if func == "sum" {
                    // sum(list) -> list.iter().sum()
                    self.emit_expr(&args[0]);
                    self.write(".iter().sum::<");
                    if let Type::List(ref elem) = args[0].ty() {
                        self.write(&elem.rust_type());
                    } else {
                        self.write("_");
                    }
                    self.write(">()");
                } else if func == "sorted" {
                    // sorted(list) -> { let mut v = list.clone(); v.sort(); v }
                    // For f64 lists, use sort_by since f64 doesn't implement Ord
                    let is_float_list = matches!(args[0].ty(), Type::List(inner) if **inner == Type::Float);
                    self.write("{ let mut _sorted = ");
                    self.emit_expr(&args[0]);
                    if is_float_list {
                        self.write(".clone(); _sorted.sort_by(|a, b| a.total_cmp(b)); _sorted }");
                    } else {
                        self.write(".clone(); _sorted.sort(); _sorted }");
                    }
                } else if func == "reversed" {
                    // reversed(list) -> { let mut v = list.clone(); v.reverse(); v }
                    self.write("{ let mut _rev = ");
                    self.emit_expr(&args[0]);
                    self.write(".clone(); _rev.reverse(); _rev }");
                } else if func == "enumerate" {
                    // enumerate(list) -> list.iter().enumerate().map(|(i, v)| (i as i64, v.clone())).collect()
                    self.emit_expr(&args[0]);
                    self.write(".iter().enumerate().map(|(i, v)| (i as i64, v.clone())).collect::<Vec<_>>()");
                } else if func == "zip" {
                    // zip(a, b) -> a.iter().zip(b.iter()).map(|(a, b)| (a.clone(), b.clone())).collect()
                    self.emit_expr(&args[0]);
                    self.write(".iter().zip(");
                    self.emit_expr(&args[1]);
                    self.write(".iter()).map(|(a, b)| (a.clone(), b.clone())).collect::<Vec<_>>()");
                } else if func == "any" {
                    // any(list) -> list.iter().any(|x| *x)
                    self.emit_expr(&args[0]);
                    self.write(".iter().any(|x| *x)");
                } else if func == "all" {
                    // all(list) -> list.iter().all(|x| *x)
                    self.emit_expr(&args[0]);
                    self.write(".iter().all(|x| *x)");
                } else if func == "map" {
                    // map(func, list) -> list.clone().into_iter().map(func).collect()
                    self.emit_expr(&args[1]);
                    self.write(".clone().into_iter().map(");
                    self.emit_lambda_untyped(&args[0]);
                    self.write(").collect::<Vec<_>>()");
                } else if func == "filter" {
                    // filter(func, list) -> list.clone().into_iter().filter(|&x| body).collect()
                    // Inline the lambda body directly instead of closure-within-closure
                    self.emit_expr(&args[1]);
                    if let HirExpr::Lambda { params, body, .. } = &args[0] {
                        let param_name = if !params.is_empty() { &params[0].name } else { "x" };
                        // Use .clone().into_iter() for owned values, then filter with |&var| destructuring
                        self.write(&format!(".clone().into_iter().filter(|&{}| ", param_name));
                        self.emit_expr(body);
                        self.write(").collect::<Vec<_>>()");
                    } else {
                        self.write(".clone().into_iter().filter(|x| (");
                        self.emit_lambda_untyped(&args[0]);
                        self.write(")(x)).collect::<Vec<_>>()");
                    }
                } else if self.intrinsic_functions.contains(func.as_str()) || func == "builtin_open" {
                    // Intrinsic function call — emit the correct Rust code
                    self.emit_intrinsic_call(func, args);
                } else {
                    self.write(func);
                    self.write("(");
                    // Look up param types and conventions to wrap union enum arguments.
                    // First check func_signatures (regular functions), then callable_var_conventions
                    // (Callable-typed parameters/locals whose conventions are tracked per-function).
                    let param_info: Option<Vec<(Type, ParamConvention)>> = self.func_signatures.get(func)
                        .map(|(pts, _)| pts.clone())
                        .or_else(|| self.callable_var_conventions.get(func).cloned());
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        // Wrap arguments to match parameter types
                        if let Some(ref pts) = param_info {
                            if i < pts.len() {
                                let (ref param_ty, convention) = pts[i];
                                // Option param with non-Option arg -> wrap in Some()
                                if is_option_type(param_ty) && !is_option_type(arg.ty()) && !matches!(arg, HirExpr::NoneLiteral) {
                                    // Use param_ty for ownership check: the wrapped Some(...) is Option<T> (Move),
                                    // not the inner arg type which may be Copy
                                    self.emit_borrow_prefix(convention, param_ty, Some(param_ty));
                                    self.write("Some(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                // None literal passed to Option param -> emit &None for borrowed params
                                if is_option_type(param_ty) && matches!(arg, HirExpr::NoneLiteral) {
                                    self.emit_borrow_prefix(convention, param_ty, Some(param_ty));
                                    self.emit_expr(arg);
                                    continue;
                                }
                                // Result[T, Error] param with a concrete Result[T, SomeError] arg:
                                // convert the error branch so Rust types line up (Result invariance).
                                if convention == ParamConvention::Own {
                                    if let (Type::Result(_, param_err), Type::Result(_, arg_err)) =
                                        (param_ty, arg.ty())
                                    {
                                        if param_err.display_name() == "Error"
                                            && arg_err.display_name() != "Error"
                                        {
                                            self.write("(");
                                            self.emit_expr(arg);
                                            self.write(").map_err(|e| Error::new(e.to_string()))");
                                            continue;
                                        }
                                    }
                                }
                                // Non-Option union param -> wrap in enum variant
                                if let Type::Union(members) = param_ty {
                                    if !is_option_type(param_ty) {
                                        let arg_ty = arg.ty();
                                        if let Some(variant) = find_union_variant(members, arg_ty) {
                                            let enum_name = param_ty.union_enum_name();
                                            // Use param_ty for ownership check: the wrapped enum value is a Union (Move),
                                            // not the inner arg type which may be Copy (e.g., Int inside IntOrStr)
                                            self.emit_borrow_prefix(convention, param_ty, Some(param_ty));
                                            self.write(&format!("{}::{}(", enum_name, variant));
                                            self.emit_expr(arg);
                                            self.write(")");
                                            continue;
                                        }
                                    }
                                }
                                // Protocol param with concrete class arg -> wrap in Box::new()
                                if matches!(param_ty, Type::Protocol { .. }) && !matches!(arg.ty(), Type::Protocol { .. }) {
                                    self.write("Box::new(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                // Callable param with TypeVar params: wrap concrete function in
                                // adapter closure so Copy-type args get dereferenced to match the
                                // generic `impl Fn(&T) -> R` signature.
                                if let Type::Callable(callable_params, callable_convs, _callable_ret) = param_ty {
                                    let has_typevar_param = callable_params.iter().any(|p| matches!(p, Type::TypeVar(_)));
                                    if has_typevar_param {
                                        if let HirExpr::Name { name: arg_func_name, .. } = arg {
                                            if let Some((concrete_params, _)) = self.func_signatures.get(arg_func_name.as_str()).cloned() {
                                                let needs_wrapper = callable_params.iter().zip(concrete_params.iter()).any(|(cp, (ct, _))| {
                                                    matches!(cp, Type::TypeVar(_)) && ct.ownership() == sifr_type_system::OwnershipKind::Copy
                                                });
                                                if needs_wrapper {
                                                    self.write("|");
                                                    for (pi, (cp, cc)) in callable_params.iter().zip(callable_convs.iter()).enumerate() {
                                                        if pi > 0 { self.write(", "); }
                                                        let pname = format!("__a{}", pi);
                                                        if matches!(cp, Type::TypeVar(_)) || (*cc == ParamConvention::Borrow && cp.ownership() == sifr_type_system::OwnershipKind::Move) {
                                                            self.write(&format!("{}: &_", pname));
                                                        } else {
                                                            self.write(&format!("{}: _", pname));
                                                        }
                                                    }
                                                    self.write("| ");
                                                    self.write(arg_func_name);
                                                    self.write("(");
                                                    for (pi, (cp, (ct, _))) in callable_params.iter().zip(concrete_params.iter()).enumerate() {
                                                        if pi > 0 { self.write(", "); }
                                                        let pname = format!("__a{}", pi);
                                                        if matches!(cp, Type::TypeVar(_)) && ct.ownership() == sifr_type_system::OwnershipKind::Copy {
                                                            self.write(&format!("*{}", pname));
                                                        } else {
                                                            self.write(&pname);
                                                        }
                                                    }
                                                    self.write(")");
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                }
                                // Convention-aware borrow prefix for regular arguments.
                                // Pass the arg name (if it's a Name expr) so we can detect
                                // already-borrowed parameters and avoid double-borrowing.
                                let arg_name_opt = if let HirExpr::Name { name, .. } = arg {
                                    Some(name.as_str())
                                } else {
                                    None
                                };
                                // For borrowed generic params (&T), wrapping non-trivial expressions
                                // avoids Rust precedence pitfalls like `&(x) as i64`.
                                if convention == ParamConvention::Borrow
                                    && matches!(param_ty, Type::TypeVar(_))
                                    && !matches!(
                                        arg,
                                        HirExpr::Name { .. }
                                            | HirExpr::IntLiteral(_)
                                            | HirExpr::FloatLiteral(_)
                                            | HirExpr::StringLiteral(_)
                                            | HirExpr::BoolLiteral(_)
                                            | HirExpr::NoneLiteral
                                    )
                                {
                                    self.write("&(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                self.emit_borrow_prefix_for_name(convention, arg.ty(), Some(param_ty), arg_name_opt);
                                self.emit_expr(arg);
                                continue;
                            }
                        }
                        self.emit_expr(arg);
                    }
                    // For recursive nested functions with captures, pass captured vars as extra args
                    if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
                        for (idx, (cap_name, _)) in captures.iter().enumerate() {
                            if !args.is_empty() || idx > 0 {
                                self.write(", ");
                            }
                            self.write(cap_name);
                        }
                    }
                    self.write(")");
                }
            }
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.write("if ");
                self.emit_expr(condition);
                self.write(" { ");
                self.emit_expr(then_expr);
                self.write(" } else { ");
                self.emit_expr(else_expr);
                self.write(" }");
            }
            HirExpr::RangeLiteral { start, end, step, .. } => {
                if let Some(step) = step {
                    self.write("(");
                    self.emit_expr(start);
                    self.write("..");
                    self.emit_expr(end);
                    self.write(").step_by(");
                    self.emit_expr(step);
                    self.write(" as usize)");
                } else {
                    self.emit_expr(start);
                    self.write("..");
                    self.emit_expr(end);
                }
            }
            HirExpr::ListLiteral { elements, .. } => {
                self.write("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                    if let HirExpr::Name { name, ty } = elem {
                        if matches!(ty, Type::TypeVar(_))
                            && (self.borrowed_params.contains(name.as_str())
                                || self.mut_borrowed_params.contains(name.as_str()))
                        {
                            self.write(".clone()");
                        }
                    }
                }
                self.write("]");
            }
            HirExpr::SetLiteral { elements, .. } => {
                self.needs_hashset = true;
                self.write("HashSet::from([");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                }
                self.write("])");
            }
            HirExpr::DictLiteral { keys, values, .. } => {
                self.needs_hashmap = true;
                self.write("HashMap::from([");
                for (i, (key, val)) in keys.iter().zip(values.iter()).enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write("(");
                    self.emit_expr(key);
                    self.write(", ");
                    self.emit_expr(val);
                    self.write(")");
                }
                self.write("])");
            }
            HirExpr::TupleLiteral { elements, .. } => {
                self.write("(");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                }
                if elements.len() == 1 {
                    self.write(","); // Single-element tuple needs trailing comma in Rust
                }
                self.write(")");
            }
            HirExpr::Index { object, index, .. } => {
                let obj_ty = object.ty();
                match obj_ty {
                    Type::Dict(_, _) => {
                        // Safe dict indexing: d[key] -> d.get(key_ref).cloned()
                        // For self.field dict, we don't need to clone the field -- just borrow it.
                        self.suppress_field_clone = true;
                        self.emit_expr(object);
                        self.write(".get(");
                        self.emit_key_ref_expr(index);
                        self.write(").cloned()");
                    }
                    Type::Tuple(_) => {
                        // Tuple indexing: t.0, t.1, etc. (handle negative)
                        // Tuples are fixed-size, so indexing is always safe at compile time
                        if let HirExpr::IntLiteral(val) = index.as_ref() {
                            if *val < 0 {
                                if let Type::Tuple(elems) = obj_ty {
                                    let resolved = (elems.len() as i64 + val) as usize;
                                    self.emit_expr(object);
                                    self.write(&format!(".{}", resolved));
                                }
                            } else {
                                // Emit raw integer for tuple field access (e.g., .0 not .0_i64)
                                self.emit_expr(object);
                                self.write(&format!(".{}", val));
                            }
                        } else {
                            // Non-literal index: emit as raw integer (tuples require compile-time indices)
                            self.emit_expr(object);
                            self.write(".");
                            self.emit_expr(index);
                        }
                    }
                    Type::Str => {
                        // Safe string indexing: returns Option<String>
                        // Handle negative indices
                        self.write("{ let _s = &");
                        self.emit_expr(object);
                        self.write("; let _i = ");
                        self.emit_expr(index);
                        self.write("; let _idx = if _i < 0 { (_s.chars().count() as i64 + _i) as usize } else { _i as usize }; _s.chars().nth(_idx).map(|c| c.to_string()) }");
                    }
                    // Union/Optional type indexing: unwrap the Option first
                    ty if is_option_type(ty) => {
                        self.write("{ let __opt = ");
                        self.emit_expr(object);
                        self.write("; let _v = __opt.as_ref().unwrap(); let _i = ");
                        self.emit_expr(index);
                        self.write("; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }");
                    }
                    _ => {
                        // Safe list indexing: returns Option<T>
                        // Handle negative indices
                        self.write("{ let _v = &");
                        self.emit_expr(object);
                        self.write("; let _i = ");
                        self.emit_expr(index);
                        self.write("; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }");
                    }
                }
            }
            HirExpr::MethodCall { object, method, args, .. } => {
                self.emit_method_call(object, method, args);
            }
            HirExpr::ContainsOp { element, collection, .. } => {
                let coll_ty = collection.ty();
                match coll_ty {
                    Type::Dict(_, _) => {
                        self.emit_expr(collection);
                        self.write(".contains_key(");
                        self.emit_key_ref_expr(element);
                        self.write(")");
                    }
                    Type::Str => {
                        self.emit_expr(collection);
                        self.write(".contains(");
                        self.emit_str_ref_expr(element);
                        self.write(")");
                    }
                    _ => {
                        // List: collection.contains(&element)
                        self.emit_expr(collection);
                        self.write(".contains(&");
                        self.emit_expr(element);
                        self.write(")");
                    }
                }
            }
            HirExpr::Slice { object, start, stop, step, ty } => {
                let obj_ty = object.ty();
                match obj_ty {
                    Type::Str => {
                        self.emit_string_slice(object, start, stop, step);
                    }
                    Type::Tuple(_) => {
                        // Compile-time tuple slicing: direct field access
                        if let Type::Tuple(result_elems) = ty {
                            let start_idx = start.as_ref().and_then(|e| if let HirExpr::IntLiteral(v) = e.as_ref() { Some(*v as usize) } else { None }).unwrap_or(0);
                            self.write("(");
                            for (i, _) in result_elems.iter().enumerate() {
                                if i > 0 {
                                    self.write(", ");
                                }
                                self.emit_expr(object);
                                self.write(&format!(".{}", start_idx + i));
                            }
                            if result_elems.len() == 1 {
                                self.write(",");
                            }
                            self.write(")");
                        }
                    }
                    _ => {
                        // List slicing
                        self.emit_list_slice(object, start, stop, step);
                    }
                }
            }
            HirExpr::WalrusExpr { name, value: _, .. } => {
                // Walrus operator: the variable is already hoisted by emit_walrus_hoists
                // Just emit the variable name (the assignment was already emitted)
                self.write(name);
            }
            HirExpr::FieldAccess { object, field, ty } => {
                // Handle enum .name and .value as method calls
                if matches!(object.ty(), Type::Enum { .. }) {
                    self.emit_expr(object);
                    self.write(".");
                    self.write(field);
                    self.write("()");
                    return;
                }

                // Determine if we need .clone() (non-Copy field accessed on &self)
                let is_self_access = matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self");
                let needs_clone = is_self_access && needs_clone_for_type(ty) && !self.suppress_field_clone;
                self.suppress_field_clone = false;

                // Determine the class name for parent field resolution
                // Either from current_class_name (inside a method) or from the object's type
                let class_name_for_parent = if let Some(ref cn) = self.current_class_name {
                    if is_self_access { Some(cn.clone()) } else { None }
                } else {
                    None
                }.or_else(|| {
                    // For external access like obj.field, check the object's type
                    if let Type::Class { name, .. } = object.ty() {
                        Some(name.clone())
                    } else {
                        None
                    }
                });

                // Check if this is accessing a parent field via inheritance
                if let Some(ref class_name) = class_name_for_parent {
                    if let Some((parent_name, parent_field_names)) = self.parent_fields.get(class_name).cloned() {
                        if parent_field_names.contains(field.as_str()) {
                            // Access via embedded parent: obj.parent.field
                            self.emit_expr(object);
                            self.write(".");
                            self.write(&parent_name.to_lowercase());
                            self.write(".");
                            self.write(field);
                            if needs_clone {
                                self.write(".clone()");
                            }
                            return;
                        }
                    }
                }
                self.emit_expr(object);
                self.write(".");
                self.write(field);
                if needs_clone {
                    self.write(".clone()");
                }
            }
            HirExpr::ConstructorCall { class_name, args, .. } => {
                // IOError subclasses map to IOError with a specific kind field
                let io_subclass_kind = match class_name.as_str() {
                    "FileNotFoundError" => Some("FileNotFound"),
                    "PermissionError" => Some("PermissionDenied"),
                    "FileExistsError" => Some("FileExists"),
                    "IsADirectoryError" => Some("IsADirectory"),
                    "NotADirectoryError" => Some("NotADirectory"),
                    "DirectoryNotEmptyError" => Some("DirectoryNotEmpty"),
                    _ => None,
                };
                if let Some(kind) = io_subclass_kind {
                    // Emit: IOError { message: <arg>.to_string(), kind: "<kind>".to_string() }
                    self.write("IOError { message: ");
                    if !args.is_empty() {
                        self.emit_expr(&args[0]);
                        self.write(".to_string()");
                    } else {
                        self.write("String::new()");
                    }
                    self.write(&format!(", kind: \"{}\".to_string() }}", kind));
                    return;
                }
                self.write(class_name);
                self.write("::new(");
                let field_names = self.class_field_order.get(class_name).cloned();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    // Check if this argument corresponds to a recursive field
                    let is_recursive = field_names.as_ref().map_or(false, |names| {
                        names.get(i).map_or(false, |fname| {
                            self.recursive_fields.contains(&(class_name.clone(), fname.clone()))
                        })
                    });
                    if is_recursive {
                        if matches!(arg, HirExpr::NoneLiteral) {
                            // None stays as None for Option<Box<T>> fields
                            self.write("None");
                        } else {
                            // Wrap in Some(Box::new(...)) for Option<Box<T>> fields
                            // or Box::new(...) for direct recursive fields
                            self.write("Some(Box::new(");
                            self.emit_expr(arg);
                            self.write("))");
                        }
                    } else {
                        // If the argument is a borrowed parameter (non-Copy type),
                        // clone it since constructors expect owned values
                        let needs_clone = if let HirExpr::Name { name, ty } = arg {
                            self.borrowed_params.contains(name) && ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        } else {
                            false
                        };
                        self.emit_expr(arg);
                        if needs_clone {
                            self.write(".clone()");
                        }
                    }
                }
                self.write(")");
            }
            HirExpr::QuestionMark { expr, .. } => {
                self.emit_expr(expr);
                self.write("?");
            }
            HirExpr::OkWrap { value, .. } => {
                if matches!(value.as_ref(), HirExpr::NoneLiteral) {
                    self.write("Ok(())");
                } else {
                    self.write("Ok(");
                    self.emit_expr(value);
                    self.write(")");
                }
            }
            HirExpr::ErrWrap { value, .. } => {
                self.write("Err(");
                self.emit_expr(value);
                self.write(")");
            }
            HirExpr::FString { parts, .. } => {
                self.emit_fstring_macro("format!", parts);
            }
            HirExpr::SuperCall { parent_class, method, args, .. } => {
                // super().__init__(args) -> ParentType::new(args)
                self.write(parent_class);
                self.write("::");
                self.write(method);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
            HirExpr::Lambda { params, body, .. } => {
                self.write("|");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&param.name);
                    // Only emit type annotation if it's not Any
                    if param.ty != Type::Any {
                        self.write(": ");
                        // For reference types, use &Type
                        if matches!(param.ty, Type::Str | Type::Class { .. }) {
                            self.write("&");
                        }
                        self.write(&param.ty.rust_type());
                    }
                }
                self.write("| ");
                self.emit_expr(body);
            }
            HirExpr::ListComp { expr, generators, ty } => {
                if generators.len() == 1 {
                    // Single generator: use functional style
                    let (ref var, ref iter_e, ref filter) = generators[0];
                    let is_range = matches!(iter_e.ty(), Type::Range);
                    let var_pattern = if var.contains(',') {
                        let names: Vec<&str> = var.split(',').collect();
                        format!("({})", names.join(", "))
                    } else { var.clone() };
                    if is_range {
                        self.write("(");
                        self.emit_expr(iter_e);
                        self.write(")");
                    } else {
                        self.emit_expr(iter_e);
                        self.write(".clone().into_iter()");
                    }
                    if let Some(ref cond) = filter {
                        let elem_is_copy = if let Type::List(ref elem) = iter_e.ty() {
                            !needs_clone_for_type(elem)
                        } else { is_range };
                        if elem_is_copy && !var.contains(',') {
                            self.write(".filter(|&");
                        } else {
                            self.write(".filter(|");
                        }
                        self.write(&var_pattern);
                        self.write("| ");
                        self.emit_expr(cond);
                        self.write(")");
                    }
                    self.write(".map(|");
                    self.write(&var_pattern);
                    self.write("| ");
                    self.emit_expr(expr);
                    self.write(")");
                    if let Type::List(ref elem) = ty {
                        self.write(&format!(".collect::<Vec<{}>>()", elem.rust_type()));
                    } else {
                        self.write(".collect::<Vec<_>>()");
                    }
                } else {
                    // Multi-generator: use imperative style
                    self.write("{ let mut _result = Vec::new(); ");
                    for (var, iter_e, filter) in generators {
                        let var_pattern = if var.contains(',') {
                            let names: Vec<&str> = var.split(',').collect();
                            format!("({})", names.join(", "))
                        } else { var.clone() };
                        let is_range = matches!(iter_e.ty(), Type::Range);
                        self.write("for ");
                        self.write(&var_pattern);
                        self.write(" in ");
                        if is_range {
                            self.write("(");
                            self.emit_expr(&iter_e);
                            self.write(")");
                        } else {
                            self.emit_expr(&iter_e);
                            self.write(".clone().into_iter()");
                        }
                        self.write(" { ");
                        if let Some(ref cond) = filter {
                            self.write("if ");
                            self.emit_expr(cond);
                            self.write(" { ");
                        }
                    }
                    self.write("_result.push(");
                    self.emit_expr(expr);
                    self.write("); ");
                    // Close filter ifs and for loops (in reverse)
                    for (_, _, ref filter) in generators.iter().rev() {
                        if filter.is_some() { self.write("} "); }
                        self.write("} ");
                    }
                    self.write("_result }");
                }
            }
            HirExpr::SetComp { expr, generators, ty } => {
                self.needs_hashset = true;
                if generators.len() == 1 {
                    let (ref var, ref iter_e, ref filter) = generators[0];
                    let is_range = matches!(iter_e.ty(), Type::Range);
                    let var_pattern = if var.contains(',') {
                        let names: Vec<&str> = var.split(',').collect();
                        format!("({})", names.join(", "))
                    } else { var.clone() };
                    if is_range {
                        self.write("(");
                        self.emit_expr(iter_e);
                        self.write(")");
                    } else {
                        self.emit_expr(iter_e);
                        self.write(".clone().into_iter()");
                    }
                    if let Some(ref cond) = filter {
                        self.write(".filter(|");
                        self.write(&var_pattern);
                        self.write("| ");
                        self.emit_expr(cond);
                        self.write(")");
                    }
                    self.write(".map(|");
                    self.write(&var_pattern);
                    self.write("| ");
                    self.emit_expr(expr);
                    self.write(")");
                    if let Type::Set(ref elem) = ty {
                        self.write(&format!(".collect::<HashSet<{}>>()", elem.rust_type()));
                    } else {
                        self.write(".collect::<HashSet<_>>()");
                    }
                } else {
                    self.write("{ let mut _result = HashSet::new(); ");
                    for (var, iter_e, filter) in generators {
                        self.write("for ");
                        self.write(var);
                        self.write(" in ");
                        self.emit_expr(&iter_e);
                        self.write(".clone().into_iter() { ");
                        if let Some(ref cond) = filter {
                            self.write("if ");
                            self.emit_expr(cond);
                            self.write(" { ");
                        }
                    }
                    self.write("_result.insert(");
                    self.emit_expr(expr);
                    self.write("); ");
                    for (_, _, ref filter) in generators.iter().rev() {
                        if filter.is_some() { self.write("} "); }
                        self.write("} ");
                    }
                    self.write("_result }");
                }
            }
            HirExpr::DictComp { key_expr, val_expr, generators, ty } => {
                self.needs_hashmap = true;
                if generators.len() == 1 {
                    let (ref var, ref iter_e, ref filter) = generators[0];
                    let is_range = matches!(iter_e.ty(), Type::Range);
                    let var_pattern = if var.contains(',') {
                        let names: Vec<&str> = var.split(',').collect();
                        format!("({})", names.join(", "))
                    } else { var.clone() };
                    if is_range {
                        self.write("(");
                        self.emit_expr(iter_e);
                        self.write(")");
                    } else {
                        self.emit_expr(iter_e);
                        self.write(".clone().into_iter()");
                    }
                    if let Some(ref cond) = filter {
                        self.write(".filter(|");
                        self.write(&var_pattern);
                        self.write("| ");
                        self.emit_expr(cond);
                        self.write(")");
                    }
                    self.write(".map(|");
                    self.write(&var_pattern);
                    self.write("| (");
                    self.emit_expr(key_expr);
                    self.write(", ");
                    self.emit_expr(val_expr);
                    self.write("))");
                    if let Type::Dict(ref k, ref v) = ty {
                        self.write(&format!(".collect::<HashMap<{}, {}>>()", k.rust_type(), v.rust_type()));
                    } else {
                        self.write(".collect::<HashMap<_, _>>()");
                    }
                } else {
                    self.write("{ let mut _result = HashMap::new(); ");
                    for (var, iter_e, filter) in generators {
                        let var_pattern = if var.contains(',') {
                            let names: Vec<&str> = var.split(',').collect();
                            format!("({})", names.join(", "))
                        } else { var.clone() };
                        self.write("for ");
                        self.write(&var_pattern);
                        self.write(" in ");
                        self.emit_expr(&iter_e);
                        self.write(".clone().into_iter() { ");
                        if let Some(ref cond) = filter {
                            self.write("if ");
                            self.emit_expr(cond);
                            self.write(" { ");
                        }
                    }
                    self.write("_result.insert(");
                    self.emit_expr(key_expr);
                    self.write(", ");
                    self.emit_expr(val_expr);
                    self.write("); ");
                    for (_, _, ref filter) in generators.iter().rev() {
                        if filter.is_some() { self.write("} "); }
                        self.write("} ");
                    }
                    self.write("_result }");
                }
            }
            HirExpr::GeneratorExpr { expr, var, iter, filter, .. } => {
                // (expr for var in iter) -> iter.clone().into_iter().map(|var| expr)
                // Lazy iterator - no .collect()
                self.emit_expr(iter);
                if filter.is_some() {
                    self.write(".iter()");
                    if let Some(ref cond) = filter {
                        self.write(".filter(|");
                        self.write(var);
                        self.write("| { let ");
                        self.write(var);
                        self.write(" = **");
                        self.write(var);
                        self.write("; ");
                        self.emit_expr(cond);
                        self.write(" })");
                    }
                    self.write(".map(|");
                    self.write(var);
                    self.write("| { let ");
                    self.write(var);
                    self.write(" = *");
                    self.write(var);
                    self.write("; ");
                    self.emit_expr(expr);
                    self.write(" })");
                } else {
                    self.write(".clone().into_iter()");
                    self.write(".map(|");
                    self.write(var);
                    self.write("| ");
                    self.emit_expr(expr);
                    self.write(")");
                }
                // No .collect() - lazy iterator
            }
            HirExpr::EnumVariant { enum_name, variant, .. } => {
                // Color.RED -> Color::RED
                self.write(enum_name);
                self.write("::");
                self.write(variant);
            }
        }
    }

    /// Emit an f-string as a Rust format macro call (format!, println!, etc.).
    /// This avoids the double-format pattern `println!("{}", format!(...))`.
    /// Emit a lambda expression without type annotations on parameters.
    /// Used when the lambda is passed to .map()/.filter() where Rust can infer types.
    /// Check if a name is a stdlib constant.
    fn is_stdlib_constant(&self, name: &str) -> bool {
        matches!(name, "pi" | "e" | "tau" | "inf" | "nan") && self.intrinsic_functions.contains(name)
    }

    /// Emit a stdlib constant value.
    fn emit_stdlib_constant(&mut self, name: &str) {
        match name {
            "pi" => self.write("std::f64::consts::PI"),
            "e" => self.write("std::f64::consts::E"),
            "tau" => self.write("std::f64::consts::TAU"),
            "inf" => self.write("f64::INFINITY"),
            "nan" => self.write("f64::NAN"),
            _ => self.write(name),
        }
    }

    /// Emit an intrinsic function call with the correct Rust code.
    fn emit_intrinsic_call(&mut self, func: &str, args: &[HirExpr]) {
        if self.try_emit_intrinsic_via_registry(func, args) {
            return;
        }

        match func {
            // sifr.io
            "read_text" => {
                self.write("std::fs::read_to_string(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)");
            }
            "write_text" => {
                self.write("std::fs::write(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "exists" => {
                self.write("std::path::Path::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").exists()");
            }
            "read_lines" => {
                self.write("std::fs::read_to_string(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|s| s.lines().map(|l| l.to_string()).collect::<Vec<String>>()).map_err(__io_err)");
            }
            "append_text" => {
                self.write("{ use std::io::Write; (|| -> Result<(), IOError> { let mut _f = std::fs::OpenOptions::new().append(true).create(true).open(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; write!(_f, \"{}\", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map_err(__io_err)?; Ok(()) })() }");
            }
            "getcwd" => {
                self.write("std::env::current_dir().map(|p| p.to_string_lossy().to_string()).map_err(__io_err)");
            }
            "listdir" => {
                self.write("std::fs::read_dir(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).collect::<Vec<String>>()).map_err(__io_err)");
            }
            "mkdir" => {
                self.write("std::fs::create_dir_all(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "rmdir" => {
                self.write("std::fs::remove_dir(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "remove_file" => {
                self.write("std::fs::remove_file(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "rename" => {
                self.write("std::fs::rename(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "is_file" => {
                self.write("std::path::Path::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").is_file()");
            }
            "is_dir" => {
                self.write("std::path::Path::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").is_dir()");
            }
            "copy_file" => {
                self.write("std::fs::copy(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "walk_dir" => {
                self.write("{ fn __walk(p: &std::path::Path) -> Result<Vec<String>, IOError> { let mut r = Vec::new(); let entries = std::fs::read_dir(p).map_err(__io_err)?; for e in entries { let e = e.map_err(__io_err)?; let path = e.path(); r.push(path.display().to_string()); if path.is_dir() { r.extend(__walk(&path)?); } } Ok(r) } __walk(std::path::Path::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(")) }");
            }
            "rmdir_all" => {
                self.write("std::fs::remove_dir_all(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "gettempdir" => {
                self.write("std::env::temp_dir().display().to_string()");
            }
            "makedirs" => {
                self.write("std::fs::create_dir_all(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            // sifr.json
            "json_loads" => {
                self.write("serde_json::from_str::<serde_json::Value>(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|v| v.to_string()).map_err(|e| JSONDecodeError { message: e.to_string(), line: e.line() as i64, column: e.column() as i64 })");
            }
            "json_dumps" => {
                self.write("serde_json::to_string(&");
                self.emit_expr(&args[0]);
                self.write(").unwrap_or_default()");
            }
            // sifr.env
            "env_get" => {
                self.write("{ let __k = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; if __k.is_empty() || __k.contains('=') || __k.as_bytes().contains(&0) { None } else { std::env::var(__k).ok() } }");
            }
            "env_set" => {
                self.write("{ let __k = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __v = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; if !__k.is_empty() && !__k.contains('=') && !__k.as_bytes().contains(&0) && !__v.as_bytes().contains(&0) { std::env::set_var(__k, __v); } }");
            }
            "env_unset" => {
                self.write("{ let __k = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; if !__k.is_empty() && !__k.contains('=') && !__k.as_bytes().contains(&0) { std::env::remove_var(__k); } }");
            }
            "env_keys" => {
                self.write("std::env::vars_os().map(|(k, _)| k.to_string_lossy().to_string()).collect::<Vec<String>>()");
            }
            "env_values" => {
                self.write("std::env::vars_os().map(|(_, v)| v.to_string_lossy().to_string()).collect::<Vec<String>>()");
            }
            "env_items" => {
                self.write("std::env::vars_os().map(|(k, v)| format!(\"{}={}\", k.to_string_lossy(), v.to_string_lossy())).collect::<Vec<String>>()");
            }
            // sifr.os
            "run_command" => {
                self.write("(|| -> Result<String, IOError> { let output = std::process::Command::new(\"sh\").args([\"-c\", ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("]).output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) })()");
            }
            "get_args" => {
                self.write("std::env::args().collect::<Vec<String>>()");
            }
            // sifr.math
            "sqrt" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").sqrt()");
            }
            "floor" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").floor() as i64");
            }
            "ceil" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").ceil() as i64");
            }
            "abs_val" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").abs()");
            }
            "log" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").ln()");
            }
            "cbrt" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").cbrt()");
            }
            "exp2" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").exp2()");
            }
            "sin" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").sin()");
            }
            "cos" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").cos()");
            }
            "tan" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").tan()");
            }
            "pow_val" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").powf(");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "min_val" => {
                self.write("{ let __a = ");
                self.emit_expr(&args[0]);
                self.write("; let __b = ");
                self.emit_expr(&args[1]);
                self.write("; if __a < __b { __a } else { __b } }");
            }
            "max_val" => {
                self.write("{ let __a = ");
                self.emit_expr(&args[0]);
                self.write("; let __b = ");
                self.emit_expr(&args[1]);
                self.write("; if __a > __b { __a } else { __b } }");
            }
            "round_val" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").round() as i64");
            }
            "asin" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").asin()");
            }
            "acos" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").acos()");
            }
            "atan" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").atan()");
            }
            "atan2" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").atan2(");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "sinh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").sinh()");
            }
            "cosh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").cosh()");
            }
            "tanh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").tanh()");
            }
            "log10" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").log10()");
            }
            "log2" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").log2()");
            }
            "degrees" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").to_degrees()");
            }
            "radians" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").to_radians()");
            }
            "isnan" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_nan()");
            }
            "isinf" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_infinite()");
            }
            "trunc" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").trunc() as i64");
            }
            "copysign" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").copysign(");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "signbit" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_sign_negative()");
            }
            "fmod" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(") % (");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "remainder" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __y: f64 = ");
                self.emit_expr(&args[1]);
                self.write("; if __x.is_nan() || __y.is_nan() { f64::NAN } else if __y == 0.0 || __x.is_infinite() { f64::NAN } else if __y.is_infinite() { __x } else { let __q = __x / __y; let __n0 = __q.trunc(); let __frac = __q - __n0; let __abs_frac = __frac.abs(); let __n = if __abs_frac < 0.5 { __n0 } else if __abs_frac > 0.5 { __n0 + __q.signum() } else if (__n0 as i64) % 2 == 0 { __n0 } else { __n0 + __q.signum() }; let __r = __x - __n * __y; if __r == 0.0 { 0.0f64.copysign(__x) } else { __r } } }");
            }
            "hypot" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").hypot(");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "fma" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").mul_add(");
                self.emit_expr(&args[1]);
                self.write(", ");
                self.emit_expr(&args[2]);
                self.write(")");
            }
            "fmax" => {
                self.write("{ let __a: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __b: f64 = ");
                self.emit_expr(&args[1]);
                self.write("; __a.max(__b) }");
            }
            "fmin" => {
                self.write("{ let __a: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __b: f64 = ");
                self.emit_expr(&args[1]);
                self.write("; __a.min(__b) }");
            }
            "exp" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").exp()");
            }
            "expm1" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").exp_m1()");
            }
            "log1p" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").ln_1p()");
            }
            "fabs" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").abs()");
            }
            "isfinite" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_finite()");
            }
            "isnormal" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_normal()");
            }
            "issubnormal" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_subnormal()");
            }
            "acosh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").acosh()");
            }
            "asinh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").asinh()");
            }
            "atanh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").atanh()");
            }
            "isqrt" => {
                self.write("{ let __n = ");
                self.emit_expr(&args[0]);
                self.write(" as f64; __n.sqrt() as i64 }");
            }
            "dist" => {
                self.write("{ let __p = &");
                self.emit_expr(&args[0]);
                self.write("; let __q = &");
                self.emit_expr(&args[1]);
                self.write("; if __p.len() != __q.len() { f64::NAN } else if __p.is_empty() { 0.0 } else { let mut __scale = 0.0f64; let mut __ssq = 1.0f64; for __i in 0..__p.len() { let __d = (__p[__i] - __q[__i]).abs(); if __d != 0.0 { if __scale < __d { let __r = __scale / __d; __ssq = 1.0 + __ssq * __r * __r; __scale = __d; } else { let __r = __d / __scale; __ssq += __r * __r; } } } if __scale == 0.0 { 0.0 } else { __scale * __ssq.sqrt() } } }");
            }
            "fsum" => {
                self.write("{ let __data = &");
                self.emit_expr(&args[0]);
                self.write("; let mut __sum = 0.0f64; let mut __comp = 0.0f64; let mut __pos_inf = false; let mut __neg_inf = false; let mut __has_nan = false; for __x in __data.iter() { let __v = *__x; if __v.is_nan() { __has_nan = true; continue; } if __v.is_infinite() { if __v.is_sign_positive() { __pos_inf = true; } else { __neg_inf = true; } continue; } let __t = __sum + __v; if __sum.abs() >= __v.abs() { __comp += (__sum - __t) + __v; } else { __comp += (__v - __t) + __sum; } __sum = __t; } if __has_nan || (__pos_inf && __neg_inf) { f64::NAN } else if __pos_inf { f64::INFINITY } else if __neg_inf { f64::NEG_INFINITY } else { __sum + __comp } }");
            }
            "sumprod" => {
                self.write("{ let __p = &");
                self.emit_expr(&args[0]);
                self.write("; let __q = &");
                self.emit_expr(&args[1]);
                self.write("; let __len = __p.len().min(__q.len()); let mut __sum = 0.0f64; for __i in 0..__len { __sum += __p[__i] * __q[__i]; } __sum }");
            }
            // sifr.test
            "assert_eq" => {
                self.write("assert_eq!(");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "assert_ne" => {
                self.write("assert_ne!(");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "assert_true" => {
                self.write("assert!(");
                self.emit_expr(&args[0]);
                self.write(")");
            }
            "assert_false" => {
                self.write("assert!(!(");
                self.emit_expr(&args[0]);
                self.write("))");
            }
            "assert_almost_eq" => {
                self.write("assert!((");
                self.emit_expr(&args[0]);
                self.write(" - (");
                self.emit_expr(&args[1]);
                self.write(")).abs() < ");
                self.emit_expr(&args[2]);
                self.write(", \"assert_almost_eq failed: {} != {} (tolerance {})\", ");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(", ");
                self.emit_expr(&args[2]);
                self.write(")");
            }
            "assert_gt" => {
                self.write("assert!(");
                self.emit_expr(&args[0]);
                self.write(" > ");
                self.emit_expr(&args[1]);
                self.write(", \"assert_gt failed: {} is not > {}\", ");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "assert_lt" => {
                self.write("assert!(");
                self.emit_expr(&args[0]);
                self.write(" < ");
                self.emit_expr(&args[1]);
                self.write(", \"assert_lt failed: {} is not < {}\", ");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            // sifr.collections — Set operations
            "new_set" => {
                self.write("Vec::<i64>::new()");
            }
            "set_from_list" => {
                self.write("{ let mut s = ");
                self.emit_collection_expr(&args[0]);
                self.write("; s.sort(); s.dedup(); s }");
            }
            "set_add" => {
                self.write("{ let mut s = ");
                self.emit_collection_expr(&args[0]);
                self.write("; let v = ");
                self.emit_expr(&args[1]);
                self.write("; if !s.contains(&v) { s.push(v); } s }");
            }
            "set_contains" => {
                self.emit_expr(&args[0]);
                self.write(".contains(&");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "set_remove" => {
                self.write("{ let mut s = ");
                self.emit_collection_expr(&args[0]);
                self.write("; s.retain(|x| *x != ");
                self.emit_expr(&args[1]);
                self.write("); s }");
            }
            "set_len" => {
                self.emit_expr(&args[0]);
                self.write(".len() as i64");
            }
            "set_union" => {
                self.write("{ let mut s = ");
                self.emit_collection_expr(&args[0]);
                self.write("; for v in ");
                self.emit_expr(&args[1]);
                self.write(".iter() { if !s.contains(v) { s.push(*v); } } s.sort(); s }");
            }
            "set_intersection" => {
                self.write("{ let __a = ");
                self.emit_collection_expr(&args[0]);
                self.write("; let __b = ");
                self.emit_collection_expr(&args[1]);
                self.write("; __a.iter().filter(|x| __b.contains(x)).cloned().collect::<Vec<i64>>() }");
            }
            // sifr.collections — Counter
            "counter_from_list" => {
                self.write("{ let mut counts = std::collections::HashMap::<String, i64>::new(); for item in ");
                self.emit_expr(&args[0]);
                self.write(".iter() { *counts.entry(item.clone()).or_insert(0) += 1; } ");
                self.write("let pairs: Vec<String> = counts.iter().map(|(k, v)| format!(\"\\\"{}\\\":{}\", k, v)).collect(); ");
                self.write("format!(\"{{{}}}\", pairs.join(\",\")) }");
            }
            "counter_get" => {
                self.write("{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); let __key = ");
                self.emit_expr(&args[1]);
                self.write("; *data.get(__key.as_str()).unwrap_or(&0) }");
            }
            "counter_most_common" => {
                self.write("{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); let mut pairs: Vec<(String, i64)> = data.into_iter().collect(); ");
                self.write("pairs.sort_by(|a, b| b.1.cmp(&a.1)); pairs.truncate(");
                self.emit_expr(&args[1]);
                self.write(" as usize); ");
                self.write("let items: Vec<String> = pairs.iter().map(|(k, v)| format!(\"[\\\"{}\\\",{}]\", k, v)).collect(); ");
                self.write("format!(\"[{}]\", items.join(\",\")) }");
            }
            "counter_total" => {
                self.write("{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); data.values().sum::<i64>() }");
            }
            "counter_values" => {
                self.write("{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); data.values().cloned().collect::<Vec<i64>>() }");
            }
            "counter_keys" => {
                self.write("{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); data.keys().cloned().collect::<Vec<String>>() }");
            }
            "counter_items" => {
                self.write("{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); let mut pairs: Vec<(String, i64)> = data.into_iter().collect(); ");
                self.write("pairs.sort_by(|a, b| a.0.cmp(&b.0)); ");
                self.write("let items: Vec<String> = pairs.iter().map(|(k, v)| format!(\"[\\\"{}\\\",{}]\", k, v)).collect(); ");
                self.write("format!(\"[{}]\", items.join(\",\")) }");
            }
            "counter_increment" => {
                self.write("{ let mut data: std::collections::HashMap<String, i64> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); *data.entry(");
                self.emit_expr(&args[1]);
                self.write(".to_string()).or_insert(0) += 1; ");
                self.write("let pairs: Vec<String> = data.iter().map(|(k, v)| format!(\"\\\"{}\\\":{}\", k, v)).collect(); ");
                self.write("format!(\"{{{}}}\", pairs.join(\",\")) }");
            }
            // sifr.collections — DefaultDict
            "defaultdict_new" => {
                self.write("format!(\"{{\\\"__default__\\\":{}}}\", ");
                self.emit_expr(&args[0]);
                self.write(")");
            }
            "defaultdict_get" => {
                self.write("{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); let def = data.get(\"__default__\").cloned().unwrap_or(0); ");
                self.write("*data.get(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").unwrap_or(&def) }");
            }
            "defaultdict_set" => {
                self.write("{ let mut data: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); data.insert(");
                self.emit_expr(&args[1]);
                self.write(".to_string(), serde_json::json!(");
                self.emit_expr(&args[2]);
                self.write(")); serde_json::to_string(&data).unwrap_or_default() }");
            }
            // sifr.bytes
            "encode_utf8" => {
                self.emit_expr_as_bytes(&args[0]);
                self.write(".iter().map(|b| *b as i64).collect::<Vec<i64>>()");
            }
            "decode_utf8" => {
                self.write("(|| -> Result<String, ParseError> { let __vals = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __bytes: Vec<u8> = Vec::with_capacity(__vals.len()); for (__idx, __b) in __vals.iter().enumerate() { if *__b < 0 || *__b > 255 { return Err(ParseError { message: format!(\"byte out of range at index {}: {}\", __idx, *__b) }); } __bytes.push(*__b as u8); } String::from_utf8(__bytes).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            "bytes_to_hex" => {
                self.write("(|| -> Result<String, ParseError> { let __vals = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __out = String::new(); for (__idx, __b) in __vals.iter().enumerate() { if *__b < 0 || *__b > 255 { return Err(ParseError { message: format!(\"byte out of range at index {}: {}\", __idx, *__b) }); } __out.push_str(&format!(\"{:02x}\", *__b as u8)); } Ok(__out) })()");
            }
            "bytes_from_hex" => {
                self.write("(|| -> Result<Vec<i64>, ParseError> { let s = ");
                self.emit_expr(&args[0]);
                self.write("; let mut cleaned = String::new(); for ch in s.chars() { if ch.is_ascii_whitespace() { continue; } if !ch.is_ascii_hexdigit() { return Err(ParseError { message: format!(\"invalid hex character: {}\", ch) }); } cleaned.push(ch); } if cleaned.len() % 2 != 0 { return Err(ParseError { message: \"fromhex() arg must contain an even number of hexadecimal digits\".to_string() }); } let mut result = Vec::new(); for pair in cleaned.as_bytes().chunks(2) { let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError { message: e.to_string() })?; result.push(i64::from_str_radix(pair_str, 16).map_err(|e| ParseError { message: e.to_string() })?); } Ok(result) })()");
            }
            // sifr.time
            "time_now" => {
                self.write("std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64()");
            }
            "sleep" => {
                self.write("std::thread::sleep(std::time::Duration::from_secs_f64(");
                self.emit_expr(&args[0]);
                self.write("))");
            }
            "time_format" => {
                self.write("{ let secs = ");
                self.emit_expr(&args[0]);
                self.write(" as i64; let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default(); dt.format(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").to_string() }");
            }
            "perf_counter" | "monotonic" => {
                self.write("{ fn __monotonic() -> f64 { static __START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new(); let s = __START.get_or_init(std::time::Instant::now); s.elapsed().as_secs_f64() } __monotonic() }");
            }
            // sifr.random
            "random_int" => {
                self.write("{ use rand::Rng; rand::thread_rng().gen_range(");
                self.emit_expr(&args[0]);
                self.write("..=");
                self.emit_expr(&args[1]);
                self.write(") }");
            }
            "random_float" => {
                self.write("{ use rand::Rng; rand::thread_rng().gen::<f64>() }");
            }
            "random_choice" => {
                self.write("{ use rand::Rng; let items = ");
                self.emit_expr(&args[0]);
                self.write("; items[rand::thread_rng().gen_range(0..items.len())].clone() }");
            }
            "random_uniform" => {
                self.write("{ use rand::Rng; rand::thread_rng().gen_range(");
                self.emit_expr(&args[0]);
                self.write("..=");
                self.emit_expr(&args[1]);
                self.write(") }");
            }
            "random_shuffle" => {
                self.write("{ use rand::seq::SliceRandom; let mut __v = ");
                self.emit_expr(&args[0]);
                self.write(".clone(); __v.shuffle(&mut rand::thread_rng()); __v }");
            }
            "random_sample" => {
                self.write("{ use rand::seq::SliceRandom; let __items = &");
                self.emit_expr(&args[0]);
                self.write("; let __k = ");
                self.emit_expr(&args[1]);
                self.write(" as usize; if __k > __items.len() { Err(ValueError { message: format!(\"sample larger than population: {} > {}\", __k, __items.len()) }) } else { Ok(__items.choose_multiple(&mut rand::thread_rng(), __k).cloned().collect::<Vec<_>>()) } }");
            }
            "random_randrange" => {
                self.write("{ let __start = ");
                self.emit_expr(&args[0]);
                self.write("; let __stop = ");
                self.emit_expr(&args[1]);
                self.write("; let __step = ");
                self.emit_expr(&args[2]);
                self.write("; if __step == 0 { Err(ValueError { message: \"randrange: step must not be zero\".to_string() }) } else if __start >= __stop && __step > 0 { Err(ValueError { message: \"randrange: empty range\".to_string() }) } else { use rand::Rng; let __n = ((__stop - __start + __step - 1) / __step).abs(); Ok(__start + rand::thread_rng().gen_range(0..__n) * __step) } }");
            }
            "random_gauss" => {
                self.write("{ use rand_distr::{Normal, Distribution}; let __mu = ");
                self.emit_expr(&args[0]);
                self.write("; let __sigma = ");
                self.emit_expr(&args[1]);
                self.write("; Normal::new(__mu, __sigma).map(|d| d.sample(&mut rand::thread_rng())).unwrap_or(__mu) }");
            }
            // sifr.re
            "re_match" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.is_match(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(")).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_find" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.find(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|m| m.as_str().to_string())).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_replace" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.replace_all(");
                self.emit_expr_as_str_ref(&args[2]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").to_string()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_findall" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.find_iter(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|m| m.as_str().to_string()).collect::<Vec<String>>()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_split" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.split(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|s| s.to_string()).collect::<Vec<String>>()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_find_start" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.find(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map_or(-1_i64, |m| m.start() as i64)).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_find_end" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.find(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map_or(-1_i64, |m| m.end() as i64)).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            // re flags variants
            // Signatures:
            //   re_match_flags(pattern, text, flags)
            //   re_find_flags(pattern, text, flags)
            //   re_replace_flags(pattern, replacement, text, flags)
            //   re_findall_flags(pattern, text, flags)
            //   re_split_flags(pattern, text, flags)
            "re_match_flags" | "re_find_flags" | "re_findall_flags" | "re_split_flags" => {
                let flags_idx = 2usize;
                let text_idx = 1usize;
                self.write("(|| -> Result<");
                match func {
                    "re_match_flags" => self.write("bool"),
                    "re_find_flags" => self.write("Option<String>"),
                    _ => self.write("Vec<String>"),
                }
                self.write(", RegexError> { let __flags_val = ");
                self.emit_expr(&args[flags_idx]);
                self.write("; let mut __flag_str = String::new(); if __flags_val & 2 != 0 { __flag_str.push_str(\"(?i)\"); } if __flags_val & 8 != 0 { __flag_str.push_str(\"(?m)\"); } if __flags_val & 16 != 0 { __flag_str.push_str(\"(?s)\"); } if __flags_val & 64 != 0 { __flag_str.push_str(\"(?x)\"); } let __pat = __flag_str + ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __re = regex::Regex::new(&__pat).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?; ");
                match func {
                    "re_match_flags" => {
                        self.write("Ok(__re.is_match(");
                        self.emit_expr_as_str_ref(&args[text_idx]);
                        self.write("))");
                    }
                    "re_find_flags" => {
                        self.write("Ok(__re.find(");
                        self.emit_expr_as_str_ref(&args[text_idx]);
                        self.write(").map(|m| m.as_str().to_string()))");
                    }
                    "re_findall_flags" => {
                        self.write("Ok(__re.find_iter(");
                        self.emit_expr_as_str_ref(&args[text_idx]);
                        self.write(").map(|m| m.as_str().to_string()).collect())");
                    }
                    "re_split_flags" => {
                        self.write("Ok(__re.split(");
                        self.emit_expr_as_str_ref(&args[text_idx]);
                        self.write(").map(|s| s.to_string()).collect())");
                    }
                    _ => {}
                }
                self.write(" })()");
            }
            "re_replace_flags" => {
                // re_replace_flags(pattern, replacement, text, flags)
                self.write("(|| -> Result<String, RegexError> { let __flags_val = ");
                self.emit_expr(&args[3]);
                self.write("; let mut __flag_str = String::new(); if __flags_val & 2 != 0 { __flag_str.push_str(\"(?i)\"); } if __flags_val & 8 != 0 { __flag_str.push_str(\"(?m)\"); } if __flags_val & 16 != 0 { __flag_str.push_str(\"(?s)\"); } if __flags_val & 64 != 0 { __flag_str.push_str(\"(?x)\"); } let __pat = __flag_str + ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __re = regex::Regex::new(&__pat).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?; Ok(__re.replace_all(");
                self.emit_expr_as_str_ref(&args[2]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").to_string()) })()");
            }
            // sifr.hash
            "sha256" => {
                self.write("{ use sha2::Digest; format!(\"{:x}\", sha2::Sha256::digest(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(")) }");
            }
            "md5" => {
                self.write("format!(\"{:x}\", md5::compute(");
                self.emit_expr_as_bytes(&args[0]);
                self.write("))");
            }
            // sifr.encoding
            "base64_encode" => {
                self.write("{ use base64::Engine; base64::engine::general_purpose::STANDARD.encode(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(") }");
            }
            "base64_decode" => {
                self.write("(|| -> Result<String, ParseError> { use base64::Engine; let bytes = base64::engine::general_purpose::STANDARD.decode(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(").map_err(|e| ParseError { message: e.to_string() })?; String::from_utf8(bytes).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            "base64_encode_opts" => {
                self.write("(|| -> Result<String, ParseError> { use base64::Engine; let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __alt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __wrap = ");
                self.emit_expr(&args[2]);
                self.write("; if __wrap < 0 { return Err(ParseError { message: \"wrapcol must be >= 0\".to_string() }); } let mut __encoded = base64::engine::general_purpose::STANDARD.encode(__s.as_bytes()); if !__alt.is_empty() { if __alt.chars().count() != 2 { return Err(ParseError { message: format!(\"invalid altchars: {}\", __alt) }); } let mut __it = __alt.chars(); let __a = __it.next().unwrap_or('+'); let __b = __it.next().unwrap_or('/'); __encoded = __encoded.chars().map(|c| if c == '+' { __a } else if c == '/' { __b } else { c }).collect::<String>(); } if __wrap == 0 { return Ok(__encoded); } let __w = __wrap as usize; let mut __wrapped = String::new(); for (i, ch) in __encoded.chars().enumerate() { if i > 0 && i % __w == 0 { __wrapped.push('\\n'); } __wrapped.push(ch); } Ok(__wrapped) })()");
            }
            "base64_decode_opts" => {
                self.write("(|| -> Result<String, ParseError> { use base64::Engine; let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __alt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __validate = ");
                self.emit_expr(&args[2]);
                self.write("; let __ignore = ");
                self.emit_expr_as_str_ref(&args[3]);
                self.write("; let mut __has_alt = false; let mut __alt_a = '+'; let mut __alt_b = '/'; if !__alt.is_empty() { if __alt.chars().count() != 2 { return Err(ParseError { message: format!(\"invalid altchars: {}\", __alt) }); } let mut __it = __alt.chars(); __alt_a = __it.next().unwrap_or('+'); __alt_b = __it.next().unwrap_or('/'); __has_alt = true; } let mut __ignore_set = std::collections::HashSet::<char>::new(); for ch in __ignore.chars() { __ignore_set.insert(ch); } let mut __normalized = String::new(); for ch in __s.chars() { if __ignore_set.contains(&ch) { continue; } let mut mapped = ch; if __has_alt { if ch == __alt_a { mapped = '+'; } else if ch == __alt_b { mapped = '/'; } } let is_base64 = (mapped >= 'A' && mapped <= 'Z') || (mapped >= 'a' && mapped <= 'z') || (mapped >= '0' && mapped <= '9') || mapped == '+' || mapped == '/' || mapped == '='; if is_base64 { __normalized.push(mapped); } else if __validate { return Err(ParseError { message: format!(\"invalid base64 character: {}\", ch) }); } } let __bytes = base64::engine::general_purpose::STANDARD.decode(__normalized.as_bytes()).map_err(|e| ParseError { message: e.to_string() })?; String::from_utf8(__bytes).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            "sha1" => {
                self.write("{ use sha1::Digest; format!(\"{:x}\", sha1::Sha1::digest(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(")) }");
            }
            "sha512" => {
                self.write("{ use sha2::Digest; format!(\"{:x}\", sha2::Sha512::digest(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(")) }");
            }
            "urlsafe_b64encode" => {
                self.write("{ use base64::Engine; base64::engine::general_purpose::URL_SAFE.encode(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(") }");
            }
            "urlsafe_b64decode" => {
                self.write("(|| -> Result<String, ParseError> { use base64::Engine; let bytes = base64::engine::general_purpose::URL_SAFE.decode(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(").map_err(|e| ParseError { message: e.to_string() })?; String::from_utf8(bytes).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            // sifr.uuid
            "uuid4" => {
                self.write("{ use rand::Rng; let mut rng = rand::thread_rng(); let bytes: [u8; 16] = rng.gen(); format!(\"{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}\", u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]), u16::from_be_bytes([bytes[4], bytes[5]]), u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0fff, (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3fff) | 0x8000, u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]])) }");
            }
            // sifr.platform
            "platform_system" => {
                self.write("std::env::consts::OS.to_string()");
            }
            "platform_arch" => {
                self.write("std::env::consts::ARCH.to_string()");
            }
            "platform_node" => {
                self.write("std::process::Command::new(\"hostname\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()");
            }
            // sifr.toml
            "toml_parse" => {
                self.write("{ let __toml_str = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; __toml_str.parse::<toml::Value>().map(|v| format!(\"{}\", v)).map_err(|e| TOMLDecodeError { message: e.to_string(), line: 0, column: 0 }) }");
            }
            // sifr.datetime
            "datetime_now" => {
                self.write("chrono::Local::now().format(\"%Y-%m-%dT%H:%M:%S\").to_string()");
            }
            "datetime_now_struct" => {
                self.write("{ use chrono::{Datelike, Timelike}; let __dt = chrono::Local::now(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64] }");
            }
            "time_strptime" => {
                self.write("(|| -> Result<Vec<i64>, ValueError> { let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __fmt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; chrono::NaiveDateTime::parse_from_str(__s, __fmt).map(|dt| { use chrono::Datelike; use chrono::Timelike; vec![dt.year() as i64, dt.month() as i64, dt.day() as i64, dt.hour() as i64, dt.minute() as i64, dt.second() as i64, dt.weekday().num_days_from_monday() as i64, dt.ordinal() as i64] }).map_err(|e| ValueError { message: e.to_string() }) })()");
            }
            "time_gmtime" => {
                self.write("{ use chrono::{Datelike, Timelike, Utc}; let __dt = Utc::now().naive_utc(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64, __dt.weekday().num_days_from_monday() as i64, __dt.ordinal() as i64] }");
            }
            "time_localtime" => {
                self.write("{ use chrono::{Datelike, Timelike, Local}; let __dt = Local::now().naive_local(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64, __dt.weekday().num_days_from_monday() as i64, __dt.ordinal() as i64] }");
            }
            "datetime_format" => {
                self.write("{ let __dt_str = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __fmt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; __dt_str.to_string() }");
            }
            "datetime_from_timestamp" => {
                self.write("{ let __ts = ");
                self.emit_expr(&args[0]);
                self.write(" as i64; chrono::DateTime::from_timestamp(__ts, 0).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).ok_or_else(|| ValueError { message: \"invalid timestamp\".to_string() }) }");
            }
            // sifr.math new intrinsics
            "erf" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __t = 1.0 / (1.0 + 0.3275911 * __x.abs()); let __poly = __t * (0.254829592 + __t * (-0.284496736 + __t * (1.421413741 + __t * (-1.453152027 + __t * 1.061405429)))); let __r = 1.0 - __poly * (-__x * __x).exp(); if __x >= 0.0 { __r } else { -__r } }");
            }
            "erfc" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __t = 1.0 / (1.0 + 0.3275911 * __x.abs()); let __poly = __t * (0.254829592 + __t * (-0.284496736 + __t * (1.421413741 + __t * (-1.453152027 + __t * 1.061405429)))); let __r = __poly * (-__x * __x).exp(); if __x >= 0.0 { __r } else { 2.0 - __r } }");
            }
            "gamma" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x <= 0.0 && __x == __x.floor() { f64::INFINITY } else { let __g = 7usize; let __c = [0.99999999999980993f64, 676.5203681218851, -1259.1392167224028, 771.32342877765313, -176.61502916214059, 12.507343278686905, -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7]; let __z = if __x < 0.5 { let __y = std::f64::consts::PI / ((__x * std::f64::consts::PI).sin() * { let __xn = 1.0 - __x; let mut __s = __c[0]; for __i in 1..=__g+1 { __s += __c[__i] / (__xn + __i as f64 - 1.0); } let __t2 = __xn + __g as f64 - 0.5; (2.0 * std::f64::consts::PI).sqrt() * __t2.powf(__xn - 0.5) * (-__t2).exp() * __s }); __y } else { let __xm = __x - 1.0; let mut __s = __c[0]; for __i in 1..=__g+1 { __s += __c[__i] / (__xm + __i as f64); } let __t2 = __xm + __g as f64 + 0.5; (2.0 * std::f64::consts::PI).sqrt() * __t2.powf(__xm + 0.5) * (-__t2).exp() * __s }; __z } }");
            }
            "lgamma" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x <= 0.0 && __x == __x.floor() { f64::INFINITY } else { let __g = 7usize; let __c = [0.99999999999980993f64, 676.5203681218851, -1259.1392167224028, 771.32342877765313, -176.61502916214059, 12.507343278686905, -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7]; let __xm = if __x < 0.5 { 1.0 - __x } else { __x - 1.0 }; let mut __s = __c[0]; for __i in 1..=__g+1 { __s += __c[__i] / (__xm + __i as f64); } let __t2 = __xm + __g as f64 + 0.5; let __r = (2.0 * std::f64::consts::PI).sqrt().ln() + (__xm + 0.5) * __t2.ln() - __t2 + __s.ln(); if __x < 0.5 { (std::f64::consts::PI / ((__x * std::f64::consts::PI).sin() * __r.exp())).abs().ln() } else { __r } } }");
            }
            "frexp" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x == 0.0 { vec![__x, 0.0] } else if !__x.is_finite() { vec![__x, 0.0] } else { let __bits = __x.to_bits(); let __sign = __bits & 0x8000000000000000; let __exp = ((__bits >> 52) & 0x7ff) as i32; let __frac = __bits & 0x000fffffffffffff; if __exp == 0 { let __scaled = __x * (2.0f64).powi(54); let __sbits = __scaled.to_bits(); let __sexp = ((__sbits >> 52) & 0x7ff) as i32; let __sfrac = __sbits & 0x000fffffffffffff; let __mant = f64::from_bits(__sign | (0x3feu64 << 52) | __sfrac); let __e = __sexp - 1022 - 54; vec![__mant, __e as f64] } else { let __mant = f64::from_bits(__sign | (0x3feu64 << 52) | __frac); let __e = __exp - 1022; vec![__mant, __e as f64] } } }");
            }
            "ldexp" => {
                self.write("{ let __m: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __e: i64 = ");
                self.emit_expr(&args[1]);
                self.write("; __m * (2.0f64).powi(__e as i32) }");
            }
            "modf" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x.is_nan() { vec![f64::NAN, f64::NAN] } else if __x.is_infinite() { vec![0.0f64.copysign(__x), __x] } else { let __int = __x.trunc(); let mut __frac = __x - __int; if __frac == 0.0 { __frac = 0.0f64.copysign(__x); } vec![__frac, __int] } }");
            }
            "nextafter" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __y: f64 = ");
                self.emit_expr(&args[1]);
                self.write("; if __x.is_nan() || __y.is_nan() { f64::NAN } else if __x == __y { __y } else if __x == 0.0 { let __sign = if __y.is_sign_negative() { 1u64 << 63 } else { 0u64 }; f64::from_bits(__sign | 1u64) } else { let mut __bits = __x.to_bits(); if (__x < __y) == (__x > 0.0) { __bits += 1; } else { __bits -= 1; } f64::from_bits(__bits) } }");
            }
            "ulp" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x.is_nan() { f64::NAN } else if __x.is_infinite() { f64::INFINITY } else { let __a = __x.abs(); if __a == 0.0 { f64::from_bits(1u64) } else if __a == f64::MAX { __a - f64::from_bits(__a.to_bits() - 1) } else { f64::from_bits(__a.to_bits() + 1) - __a } } }");
            }
            // sifr.pathlib new intrinsics
            "touch" => {
                self.write("std::fs::OpenOptions::new().create(true).write(true).open(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "resolve_path" => {
                self.write("std::fs::canonicalize(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|p| p.to_string_lossy().to_string()).map_err(__io_err)");
            }
            "iterdir" => {
                self.write("(|| -> Result<Vec<String>, IOError> { let __entries = std::fs::read_dir(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; Ok(__entries.filter_map(|e| e.ok().map(|e| e.path().to_string_lossy().to_string())).collect()) })()");
            }
            // sifr.os new intrinsics
            "chdir" => {
                self.write("std::env::set_current_dir(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)");
            }
            "getpid" => {
                self.write("std::process::id() as i64");
            }
            "cpu_count" => {
                self.write("{ let __n = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1); __n as i64 }");
            }
            "stat_size" => {
                self.write("std::fs::metadata(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|m| m.len() as i64).map_err(__io_err)");
            }
            "which" => {
                self.write("{ let __name = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; std::env::var(\"PATH\").ok().and_then(|__path| __path.split(':').map(|d| std::path::Path::new(d).join(__name)).find(|p| p.is_file()).map(|p| p.to_string_lossy().to_string())) }");
            }
            "disk_usage" => {
                self.write("{ let __path = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __stat = std::fs::metadata(__path); match __stat { Ok(_) => { let __out = std::process::Command::new(\"df\").args([\"-k\", __path]).output(); match __out { Ok(__o) => { let __s = String::from_utf8_lossy(&__o.stdout); let __lines: Vec<&str> = __s.lines().collect(); if __lines.len() >= 2 { let __parts: Vec<&str> = __lines[1].split_whitespace().collect(); if __parts.len() >= 4 { let __total = __parts[1].parse::<i64>().unwrap_or(0) * 1024; let __used = __parts[2].parse::<i64>().unwrap_or(0) * 1024; let __free = __parts[3].parse::<i64>().unwrap_or(0) * 1024; vec![__total, __used, __free] } else { vec![0i64, 0, 0] } } else { vec![0i64, 0, 0] } }, Err(_) => vec![0i64, 0, 0] } }, Err(_) => vec![0i64, 0, 0] } }");
            }
            // open() built-in — wraps open_file and constructs FileHandle (raises IOError on failure)
            "builtin_open" => {
                self.needs_file_handles = true;
                self.used_stdlib_modules.insert("io".to_string());
                self.write("{ use std::io::{BufReader, BufWriter}; let __path = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __mode = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __handle_id: i64 = { use std::sync::atomic::{AtomicI64, Ordering}; static __NEXT_FH_ID: AtomicI64 = AtomicI64::new(1); __NEXT_FH_ID.fetch_add(1, Ordering::SeqCst) }; match __mode { \"r\" | \"rt\" => { let __f = std::fs::File::open(__path).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextRead(__reader)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"w\" | \"wt\" => { let __f = std::fs::File::create(__path).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"a\" | \"at\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"rb\" => { let __f = std::fs::File::open(__path).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryRead(__reader)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"wb\" => { let __f = std::fs::File::create(__path).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"ab\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, _ => return Err(IOError { message: format!(\"invalid mode: {}\", __mode), kind: \"Other\".to_string() }) } }");
            }
            // open() built-in file handle intrinsics
            "open_file" => {
                self.needs_file_handles = true;
                self.write("(|| -> Result<i64, IOError> { use std::io::{BufReader, BufWriter}; let __path = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __mode = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __handle_id: i64 = { use std::sync::atomic::{AtomicI64, Ordering}; static __NEXT_ID: AtomicI64 = AtomicI64::new(1); __NEXT_ID.fetch_add(1, Ordering::SeqCst) }; let __mode_s: &str = &__mode; let __path_s: &str = &__path; match __mode_s { \"r\" | \"rt\" => { let __f = std::fs::File::open(__path_s).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextRead(__reader)); Ok(__handle_id) }, \"w\" | \"wt\" => { let __f = std::fs::File::create(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); Ok(__handle_id) }, \"a\" | \"at\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); Ok(__handle_id) }, \"rb\" => { let __f = std::fs::File::open(__path_s).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryRead(__reader)); Ok(__handle_id) }, \"wb\" => { let __f = std::fs::File::create(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); Ok(__handle_id) }, \"ab\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); Ok(__handle_id) }, _ => Err(IOError { message: format!(\"invalid mode: {}\", __mode), kind: \"Other\".to_string() }) } })()");
            }
            "file_read" => {
                self.write("(|| -> Result<String, IOError> { use std::io::Read; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextRead(ref mut __r)) => { let mut __s = String::new(); __r.read_to_string(&mut __s).map_err(__io_err)?; Ok(__s) }, _ => Err(IOError { message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_write" => {
                self.write("(|| -> Result<(), IOError> { use std::io::Write; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let __data = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextWrite(ref mut __w)) => { __w.write_all(__data.as_bytes()).map_err(__io_err)?; Ok(()) }, _ => Err(IOError { message: \"file not open for writing\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_readline" => {
                self.write("(|| -> Result<Option<String>, IOError> { use std::io::BufRead; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextRead(ref mut __r)) => { let mut __line = String::new(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 { Ok(None) } else { if __line.ends_with('\\n') { __line.pop(); if __line.ends_with('\\r') { __line.pop(); } } Ok(Some(__line)) } }, _ => Err(IOError { message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_readlines" => {
                self.write("(|| -> Result<Vec<String>, IOError> { use std::io::BufRead; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextRead(ref mut __r)) => { let mut __lines: Vec<String> = Vec::new(); let mut __line = String::new(); loop { __line.clear(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 { break; } let mut __l = __line.clone(); if __l.ends_with('\\n') { __l.pop(); if __l.ends_with('\\r') { __l.pop(); } } __lines.push(__l); } Ok(__lines) }, _ => Err(IOError { message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_close" => {
                self.write("{ let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; __SIFR_FILE_HANDLES.lock().unwrap().remove(&__hid); }");
            }
            "file_read_bytes" => {
                self.write("(|| -> Result<Vec<i64>, IOError> { use std::io::Read; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::BinaryRead(ref mut __r)) => { let mut __buf = Vec::new(); __r.read_to_end(&mut __buf).map_err(__io_err)?; Ok(__buf.iter().map(|&b| b as i64).collect()) }, _ => Err(IOError { message: \"file not open for binary reading\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_write_bytes" => {
                self.write("(|| -> Result<(), IOError> { use std::io::Write; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let __data: Vec<u8> = ");
                self.emit_expr(&args[1]);
                self.write(".iter().map(|&b| b as u8).collect(); let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::BinaryWrite(ref mut __w)) => { __w.write_all(&__data).map_err(__io_err)?; Ok(()) }, _ => Err(IOError { message: \"file not open for binary writing\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            // Path.glob / Path.rglob
            "glob_pattern" => {
                self.write("(|| -> Result<Vec<String>, IOError> { let __dir = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __pat = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __full_pat = if __dir.is_empty() { __pat.to_string() } else { format!(\"{}/{}\", __dir, __pat) }; let __entries = std::fs::read_dir(__dir).map_err(__io_err)?; let mut __results: Vec<String> = Vec::new(); fn __matches_glob(name: &str, pattern: &str) -> bool { let __parts: Vec<&str> = pattern.split('*').collect(); if __parts.len() == 1 { return name == pattern; } if !name.starts_with(__parts[0]) { return false; } let mut __pos = __parts[0].len(); for __i in 1..__parts.len() { if __parts[__i].is_empty() { __pos = name.len(); continue; } match name[__pos..].find(__parts[__i]) { Some(__idx) => __pos += __idx + __parts[__i].len(), None => return false, } } true } for __entry in __entries { let __e = __entry.map_err(__io_err)?; let __name = __e.file_name().to_string_lossy().to_string(); if __matches_glob(&__name, __pat) { __results.push(__e.path().to_string_lossy().to_string()); } } __results.sort(); Ok(__results) })()");
            }
            "rglob_pattern" => {
                self.write("(|| -> Result<Vec<String>, IOError> { let __dir = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __pat = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; fn __rglob_walk(dir: &str, pattern: &str, results: &mut Vec<String>) -> Result<(), IOError> { fn __io_err_inner(e: std::io::Error) -> IOError { IOError { message: e.to_string(), kind: \"Other\".to_string() } } fn __matches_glob(name: &str, pat: &str) -> bool { let parts: Vec<&str> = pat.split('*').collect(); if parts.len() == 1 { return name == pat; } if !name.starts_with(parts[0]) { return false; } let mut pos = parts[0].len(); for i in 1..parts.len() { if parts[i].is_empty() { pos = name.len(); continue; } match name[pos..].find(parts[i]) { Some(idx) => pos += idx + parts[i].len(), None => return false, } } true } let entries = std::fs::read_dir(dir).map_err(__io_err_inner)?; for entry in entries { let e = entry.map_err(__io_err_inner)?; let path = e.path(); let name = e.file_name().to_string_lossy().to_string(); if path.is_dir() { __rglob_walk(&path.to_string_lossy(), pattern, results)?; } if __matches_glob(&name, pattern) { results.push(path.to_string_lossy().to_string()); } } Ok(()) } let mut __results: Vec<String> = Vec::new(); __rglob_walk(__dir, __pat, &mut __results).map_err(|e| e)?; __results.sort(); Ok(__results) })()");
            }
            // os constants
            "os_sep" => {
                self.write("std::path::MAIN_SEPARATOR.to_string()");
            }
            "os_linesep" => {
                #[cfg(target_os = "windows")]
                self.write("\"\\r\\n\".to_string()");
                #[cfg(not(target_os = "windows"))]
                self.write("\"\\n\".to_string()");
            }
            "os_name" => {
                self.write("{ if cfg!(target_os = \"windows\") { \"nt\".to_string() } else { \"posix\".to_string() } }");
            }
            // sifr.hashlib new intrinsics
            "sha224" => {
                self.write("{ use sha2::Digest; let mut __h = sha2::Sha224::new(); __h.update(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes()); format!(\"{:x}\", __h.finalize()) }");
            }
            "sha384" => {
                self.write("{ use sha2::Digest; let mut __h = sha2::Sha384::new(); __h.update(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes()); format!(\"{:x}\", __h.finalize()) }");
            }
            "blake2b" => {
                self.write("{ use blake2::{Blake2b512, Digest}; let mut __h = Blake2b512::new(); __h.update(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes()); format!(\"{:x}\", __h.finalize()) }");
            }
            "blake2s" => {
                self.write("{ use blake2::{Blake2s256, Digest}; let mut __h = Blake2s256::new(); __h.update(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes()); format!(\"{:x}\", __h.finalize()) }");
            }
            // sifr.base64 new intrinsics
            "b32encode" => {
                self.write("{ let __b32_alpha = b\"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567\"; let __data = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes(); let mut __out = String::new(); let mut __i = 0usize; while __i < __data.len() { let __b0 = __data[__i] as u64; let __b1 = if __i+1 < __data.len() { __data[__i+1] as u64 } else { 0 }; let __b2 = if __i+2 < __data.len() { __data[__i+2] as u64 } else { 0 }; let __b3 = if __i+3 < __data.len() { __data[__i+3] as u64 } else { 0 }; let __b4 = if __i+4 < __data.len() { __data[__i+4] as u64 } else { 0 }; let __buf = (__b0<<32)|(__b1<<24)|(__b2<<16)|(__b3<<8)|__b4; let __n = ((__data.len() - __i).min(5)) as u64; for __j in 0..8u64 { if __j < (__n*8+4)/5 { __out.push(__b32_alpha[((__buf >> (35 - __j*5)) & 0x1f) as usize] as char); } else { __out.push('='); } } __i += 5; } __out }");
            }
            "b32decode" => {
                self.write("(|| -> Result<String, ParseError> { let __b32_alpha = b\"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567\"; let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __s = __s.trim_end_matches('='); let mut __bits = 0u64; let mut __bit_count = 0u32; let mut __out: Vec<u8> = Vec::new(); for __c in __s.chars() { let __val = __b32_alpha.iter().position(|&b| b as char == __c.to_ascii_uppercase()).ok_or_else(|| ParseError { message: format!(\"invalid base32 char: {}\", __c) })? as u64; __bits = (__bits << 5) | __val; __bit_count += 5; if __bit_count >= 8 { __bit_count -= 8; __out.push(((__bits >> __bit_count) & 0xff) as u8); } } String::from_utf8(__out).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            "b32hexencode" => {
                self.write("{ let __b32_alpha = b\"0123456789ABCDEFGHIJKLMNOPQRSTUV\"; let __data = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes(); let mut __out = String::new(); let mut __i = 0usize; while __i < __data.len() { let __b0 = __data[__i] as u64; let __b1 = if __i+1 < __data.len() { __data[__i+1] as u64 } else { 0 }; let __b2 = if __i+2 < __data.len() { __data[__i+2] as u64 } else { 0 }; let __b3 = if __i+3 < __data.len() { __data[__i+3] as u64 } else { 0 }; let __b4 = if __i+4 < __data.len() { __data[__i+4] as u64 } else { 0 }; let __buf = (__b0<<32)|(__b1<<24)|(__b2<<16)|(__b3<<8)|__b4; let __n = ((__data.len() - __i).min(5)) as u64; for __j in 0..8u64 { if __j < (__n*8+4)/5 { __out.push(__b32_alpha[((__buf >> (35 - __j*5)) & 0x1f) as usize] as char); } else { __out.push('='); } } __i += 5; } __out }");
            }
            "b32hexdecode" => {
                self.write("(|| -> Result<String, ParseError> { let __b32_alpha = b\"0123456789ABCDEFGHIJKLMNOPQRSTUV\"; let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __s = __s.trim_end_matches('='); let mut __bits = 0u64; let mut __bit_count = 0u32; let mut __out: Vec<u8> = Vec::new(); for __c in __s.chars() { let __val = __b32_alpha.iter().position(|&b| b as char == __c.to_ascii_uppercase()).ok_or_else(|| ParseError { message: format!(\"invalid base32hex char: {}\", __c) })? as u64; __bits = (__bits << 5) | __val; __bit_count += 5; if __bit_count >= 8 { __bit_count -= 8; __out.push(((__bits >> __bit_count) & 0xff) as u8); } } String::from_utf8(__out).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            // sifr.platform new intrinsics
            "platform_release" => {
                self.write("{ std::process::Command::new(\"uname\").arg(\"-r\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default() }");
            }
            "platform_version" => {
                self.write("{ std::process::Command::new(\"uname\").arg(\"-v\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default() }");
            }
            "platform_processor" => {
                self.write("std::env::consts::ARCH.to_string()");
            }
            // sifr.time new intrinsics
            "strptime" => {
                self.write("(|| -> Result<String, ValueError> { use chrono::NaiveDateTime; let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __fmt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; NaiveDateTime::parse_from_str(__s, __fmt).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).map_err(|e| ValueError { message: e.to_string() }) })()");
            }
            "gmtime" => {
                self.write("{ use chrono::{DateTime, Utc}; let __ts = ");
                self.emit_expr(&args[0]);
                self.write(" as i64; DateTime::<Utc>::from_timestamp(__ts, 0).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).unwrap_or_default() }");
            }
            "localtime" => {
                self.write("{ use chrono::{DateTime, Utc, Local}; let __ts = ");
                self.emit_expr(&args[0]);
                self.write(" as i64; DateTime::<Utc>::from_timestamp(__ts, 0).map(|dt| dt.with_timezone(&Local).format(\"%Y-%m-%dT%H:%M:%S\").to_string()).unwrap_or_default() }");
            }
            // sifr.sys extras
            "sys_exit" => {
                self.write("{ std::process::exit(");
                self.emit_expr(&args[0]);
                self.write(" as i32) }");
            }
            "sys_version" => {
                self.write("\"sifr 0.1.0\".to_string()");
            }
            "sys_platform" => {
                self.write("std::env::consts::OS.to_string()");
            }
            "sys_maxsize" => {
                self.write("i64::MAX");
            }
            "subprocess_run" => {
                self.write("(|| -> Result<String, IOError> { let output = std::process::Command::new(\"sh\").args([\"-c\", ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("]).output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) })()");
            }
            "subprocess_run_with_input" => {
                self.write("(|| -> Result<String, IOError> { use std::io::Write; let mut child = std::process::Command::new(\"sh\").args([\"-c\", ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("]).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().map_err(__io_err)?; if let Some(mut stdin) = child.stdin.take() { stdin.write_all(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(".as_bytes()).map_err(__io_err)?; } let output = child.wait_with_output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) })()");
            }
            "subprocess_run_structured" => {
                self.write("(|| -> Result<Vec<String>, IOError> { let output = std::process::Command::new(\"sh\").args([\"-c\", ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("]).output().map_err(__io_err)?; let stdout = String::from_utf8_lossy(&output.stdout).to_string(); let stderr = String::from_utf8_lossy(&output.stderr).to_string(); let returncode = output.status.code().unwrap_or(-1).to_string(); Ok(vec![stdout, stderr, returncode]) })()");
            }
            // sifr.html
            "html_escape" => {
                self.write("{ let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; __s.replace('&', \"&amp;\").replace('<', \"&lt;\").replace('>', \"&gt;\").replace('\"', \"&quot;\").replace('\\'', \"&#x27;\") }");
            }
            "html_unescape" => {
                self.write("{ let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; __s.replace(\"&amp;\", \"&\").replace(\"&lt;\", \"<\").replace(\"&gt;\", \">\").replace(\"&quot;\", \"\\\"\").replace(\"&#x27;\", \"'\").replace(\"&#39;\", \"'\") }");
            }
            // sifr.calendar
            "calendar_isleap" => {
                self.write("{ let __y = ");
                self.emit_expr(&args[0]);
                self.write("; (__y % 4 == 0 && __y % 100 != 0) || (__y % 400 == 0) }");
            }
            "calendar_weekday" => {
                // Tomohiko Sakamoto's algorithm for day of week (0=Monday)
                self.write("{ let __y0 = ");
                self.emit_expr(&args[0]);
                self.write("; let __m0 = ");
                self.emit_expr(&args[1]);
                self.write("; let __d0 = ");
                self.emit_expr(&args[2]);
                self.write("; let __t = [0i64, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4]; let __y = if __m0 < 3 { __y0 - 1 } else { __y0 }; ((__y + __y/4 - __y/100 + __y/400 + __t[(__m0-1) as usize] + __d0) % 7 + 6) % 7 }");
            }
            "calendar_monthrange" => {
                self.write("{ let __y = ");
                self.emit_expr(&args[0]);
                self.write("; let __m = ");
                self.emit_expr(&args[1]);
                self.write("; let __days = match __m { 1|3|5|7|8|10|12 => 31i64, 4|6|9|11 => 30, 2 => if (__y%4==0 && __y%100!=0)||(__y%400==0) { 29 } else { 28 }, _ => 30 }; let __t = [0i64,3,2,5,0,3,5,1,4,6,2,4]; let __y2 = if __m < 3 { __y-1 } else { __y }; let __wd = ((__y2+__y2/4-__y2/100+__y2/400+__t[(__m-1) as usize]+1)%7+6)%7; vec![__wd, __days] }");
            }
            // sifr.gzip
            "gzip_compress" => {
                self.write("{ use std::io::Write; let __data = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes(); let mut __enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default()); __enc.write_all(__data).unwrap_or(()); __enc.finish().unwrap_or_default().iter().map(|b| *b as i64).collect::<Vec<i64>>() }");
            }
            "gzip_decompress" => {
                self.write("(|| -> Result<String, IOError> { use std::io::Read; let __bytes: Vec<u8> = ");
                self.emit_expr(&args[0]);
                self.write(".iter().map(|b| *b as u8).collect(); let mut __dec = flate2::read::GzDecoder::new(__bytes.as_slice()); let mut __out = String::new(); __dec.read_to_string(&mut __out).map_err(__io_err)?; Ok(__out) })()");
            }
            // sifr.zipfile
            "zip_create" => {
                self.write("(|| -> Result<(), IOError> { let __f = std::fs::File::create(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; drop(zip::ZipWriter::new(__f)); Ok(()) })()");
            }
            "zip_add_file" => {
                self.write("(|| -> Result<(), IOError> { let __path = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __name = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __content = ");
                self.emit_expr_as_str_ref(&args[2]);
                self.write("; let __f = std::fs::OpenOptions::new().read(true).write(true).open(__path).map_err(__io_err)?; let mut __zip = zip::ZipWriter::new_append(__f).map_err(|e| IOError::new(e.to_string()))?; let __opts = zip::write::FileOptions::default(); __zip.start_file(__name, __opts).map_err(|e| IOError::new(e.to_string()))?; use std::io::Write; __zip.write_all(__content.as_bytes()).map_err(__io_err)?; __zip.finish().map_err(|e| IOError::new(e.to_string()))?; Ok(()) })()");
            }
            "zip_read_file" => {
                self.write("(|| -> Result<String, IOError> { let __f = std::fs::File::open(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?; let mut __file = __zip.by_name(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map_err(|e| IOError::new(e.to_string()))?; let mut __content = String::new(); use std::io::Read; __file.read_to_string(&mut __content).map_err(__io_err)?; Ok(__content) })()");
            }
            "zip_namelist" => {
                self.write("(|| -> Result<Vec<String>, IOError> { let __f = std::fs::File::open(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?; Ok((0..__zip.len()).map(|i| __zip.by_index(i).map(|f| f.name().to_string()).unwrap_or_default()).collect()) })()");
            }
            // sifr.logging
            "set_global_level" => {
                self.write("{ *__SIFR_GLOBAL_LOG_LEVEL.lock().unwrap() = ");
                self.emit_expr(&args[0]);
                self.write("; }");
            }
            "get_global_level" => {
                self.write("*__SIFR_GLOBAL_LOG_LEVEL.lock().unwrap()");
            }
            _ => {
                // Unknown stdlib function — emit as regular call
                self.write(func);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
        }
    }

    fn try_emit_intrinsic_via_registry(&mut self, func: &str, args: &[HirExpr]) -> bool {
        let rendered_args = args.iter().map(|arg| self.expr_to_string(arg)).collect::<Vec<_>>();
        let Some(lowered) = intrinsics::lower_intrinsic(func, &rendered_args) else {
            return false;
        };

        if let Some(required_crate) = lowered.required_crate {
            self.intrinsic_registry_crates.insert(required_crate.to_string());
            self.used_stdlib_modules.insert(required_crate.to_string());
        }

        self.write(&crate::render_expr(&lowered.expr));
        true
    }

    fn emit_lambda_untyped(&mut self, expr: &HirExpr) {
        if let HirExpr::Lambda { params, body, .. } = expr {
            self.write("|");
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(&param.name);
            }
            self.write("| ");
            self.emit_expr(body);
        } else {
            // Not a lambda, emit as-is
            self.emit_expr(expr);
        }
    }

    fn emit_fstring_macro(&mut self, macro_name: &str, parts: &[HirFStringPart]) {
        let mut format_str = String::new();
        let mut exprs: Vec<&HirExpr> = Vec::new();
        for part in parts {
            match part {
                HirFStringPart::Literal(s) => {
                    // Escape braces in the literal for Rust's format!
                    for ch in s.chars() {
                        match ch {
                            '{' => format_str.push_str("{{"),
                            '}' => format_str.push_str("}}"),
                            _ => format_str.push(ch),
                        }
                    }
                }
                HirFStringPart::Expr(expr) => {
                    format_str.push_str("{}");
                    exprs.push(expr);
                }
            }
        }
        self.write(macro_name);
        self.write("(\"");
        self.write(&format_str);
        self.write("\"");
        for expr in &exprs {
            self.write(", ");
            self.emit_display_expr(expr);
        }
        self.write(")");
    }

    /// Emit an expression as a HashMap key reference.
    /// String literals are emitted directly (e.g., `"key"`) since HashMap::get accepts &str via Borrow.
    /// Other expressions are emitted with `&` prefix (e.g., `&var`).
    fn emit_key_ref_expr(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{:?}", val));
        } else if let HirExpr::Name { name, ty } = expr {
            // If the name is already a borrowed parameter (&String or &mut String),
            // emitting `&name` would produce `&&String` which fails Borrow<str> bounds.
            // For borrowed string params, emit `name.as_str()` or just `name` (deref coerces).
            if (self.borrowed_params.contains(name.as_str())
                || self.mut_borrowed_params.contains(name.as_str()))
                && matches!(ty, Type::Str)
            {
                // already &String -- deref-coerces to &str via as_str()
                self.write(name);
                self.write(".as_str()");
            } else if self.borrowed_params.contains(name.as_str())
                || self.mut_borrowed_params.contains(name.as_str())
            {
                // already a reference -- pass directly (no extra &)
                self.emit_expr(expr);
            } else {
                self.write("&");
                self.emit_expr(expr);
            }
        } else {
            self.write("&");
            self.emit_expr(expr);
        }
    }

    /// Emit an expression as a `&str` reference.
    /// String literals are emitted directly (e.g., `"hello"`).
    /// Other string expressions are emitted with `.as_str()` (e.g., `s.as_str()`).
    fn emit_str_ref_expr(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{:?}", val));
        } else {
            self.emit_expr(expr);
            self.write(".as_str()");
        }
    }

    /// Emit an expression as a `&str` for stdlib call sites.
    /// String literals are emitted as bare `"literal"` (no `.to_string()`).
    /// Borrowed parameters are emitted directly (already `&String`, deref-coerces to `&str`).
    /// Other expressions are emitted as `&expr` (borrow the String, deref-coerces to `&str`).
    /// Use this for Rust APIs that accept `&str`, `AsRef<str>`, `AsRef<Path>`, `AsRef<OsStr>`, etc.
    fn emit_expr_as_str_ref(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{:?}", val));
        } else if let HirExpr::Name { name, .. } = expr {
            if self.borrowed_params.contains(name) {
                // Already &String, no extra & needed
                self.emit_expr(expr);
            } else {
                self.write("&");
                self.emit_expr(expr);
            }
        } else {
            self.write("&");
            self.emit_expr(expr);
        }
    }

    /// Emit an expression for use in comparisons, dereferencing borrowed params.
    /// When a function parameter is `&String` (borrow-by-default), comparing it
    /// directly with a `String` fails in Rust (`&String != String`).
    /// This method emits `*name` for borrowed params so the comparison works.
    fn emit_expr_for_compare(&mut self, expr: &HirExpr) {
        if let HirExpr::Name { name, ty } = expr {
            if self.borrowed_params.contains(name) && (matches!(ty, Type::Str) || matches!(ty, Type::TypeVar(_))) {
                self.write("*");
                self.emit_expr(expr);
                return;
            }
        }
        self.emit_expr(expr);
    }

    /// Emit an expression as bytes for stdlib call sites (hash, encoding).
    /// String literals are emitted as `"literal".as_bytes()` (no `.to_string()`).
    /// Other expressions are emitted as `expr.as_bytes()` (String has `.as_bytes()`).
    fn emit_expr_as_bytes(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{:?}.as_bytes()", val));
        } else {
            self.emit_expr(expr);
            self.write(".as_bytes()");
        }
    }

    /// Check if an expression is a list literal (HirExpr::ListLiteral).
    fn is_list_literal(expr: &HirExpr) -> bool {
        matches!(expr, HirExpr::ListLiteral { .. })
    }

    /// Emit a collection expression for set operations.
    /// List literals are emitted directly (no `.clone()`).
    /// Other expressions are emitted with `.clone()`.
    fn emit_collection_expr(&mut self, expr: &HirExpr) {
        self.emit_expr(expr);
        if !Self::is_list_literal(expr) {
            self.write(".clone()");
        }
    }

    /// Emit an expression suitable for use inside format!/println! contexts.
    /// Wraps Option<T> expressions so they display as the inner value or "None".
    /// Omits `.to_string()` on string literals since format macros accept &str.
    fn emit_display_expr(&mut self, expr: &HirExpr) {
        if is_option_type(expr.ty()) {
            // Wrap: expr.map_or("None".to_string(), |_v| format!("{}", _v))
            self.write("(");
            self.emit_expr(expr);
            self.write(").map_or(\"None\".to_string(), |_v| format!(\"{}\", _v))");
        } else if let HirExpr::StringLiteral(val) = expr {
            // In display contexts, string literals don't need .to_string()
            self.write(&format!("{:?}", val));
        } else {
            self.emit_expr(expr);
        }
    }
}

/// Check if a type is an Option type (T | None with exactly 2 members).
fn is_option_type(ty: &Type) -> bool {
    if let Type::Union(members) = ty {
        let non_none: Vec<&Type> = members.iter().filter(|m| !matches!(m, Type::None)).collect();
        let has_none = members.iter().any(|m| matches!(m, Type::None));
        has_none && non_none.len() == 1
    } else {
        false
    }
}

/// Detect truthiness check on an Option variable: `if x:` where x has type T | None.
fn detect_option_truthiness(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty } = expr {
        if is_option_type(ty) {
            return Some(name.clone());
        }
    }
    None
}

/// Detect `x is not None` pattern in a Compare expression. Returns the variable name.
fn detect_is_not_none_var(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Compare { left, ops, comparators, .. } = expr {
        if ops.len() == 1 && ops[0] == "is not" && matches!(comparators[0], HirExpr::NoneLiteral) {
            if let HirExpr::Name { name, ty } = left.as_ref() {
                // Only match for Option types (2-member unions with None)
                if is_option_type(ty) {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

/// Detect compound `a is not None and b is not None` pattern.
/// Returns list of variable names that are checked for not-None.
fn detect_and_not_none_vars(expr: &HirExpr) -> Option<Vec<String>> {
    if let HirExpr::BoolOp { op, values, .. } = expr {
        if op == "and" {
            let mut vars = Vec::new();
            for val in values {
                if let Some(var_name) = detect_is_not_none_var(val) {
                    vars.push(var_name);
                }
            }
            if vars.len() >= 2 {
                return Some(vars);
            }
        }
    }
    None
}

/// Detect `isinstance(x, type)` where x is a non-Option union type.
/// Returns (var_name, variant_name, enum_name, other_variants: Vec<(variant_name, type)>).
fn detect_isinstance_union(expr: &HirExpr) -> Option<(String, String, String, Vec<(String, Type)>)> {
    if let HirExpr::Call { func, args, .. } = expr {
        if func == "isinstance" && args.len() == 2 {
            if let HirExpr::Name { name, ty } = &args[0] {
                if let Type::Union(members) = ty {
                    if !is_option_type(ty) {
                        // The second arg is a StringLiteral with the type name
                        if let HirExpr::StringLiteral(type_name) = &args[1] {
                            let target_ty = match type_name.as_str() {
                                "int" => Type::Int,
                                "str" => Type::Str,
                                "float" => Type::Float,
                                "bool" => Type::Bool,
                                other => {
                                    // Check if it's a class type in the union members
                                    if let Some(class_ty) = members.iter().find(|m| {
                                        matches!(m, Type::Class { name, .. } if name == other)
                                    }) {
                                        class_ty.clone()
                                    } else {
                                        return None;
                                    }
                                }
                            };
                            // Check that this type is a member of the union
                            if members.contains(&target_ty) {
                                let variant = target_ty.union_variant_name();
                                let enum_name = ty.union_enum_name();
                                // Collect other variants for else branch destructuring
                                let other_variants: Vec<(String, Type)> = members.iter()
                                    .filter(|m| *m != &target_ty)
                                    .map(|m| (m.union_variant_name(), m.clone()))
                                    .collect();
                                return Some((name.clone(), variant, enum_name, other_variants));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Find the matching union variant name for an argument type.
fn find_union_variant(members: &[Type], arg_ty: &Type) -> Option<String> {
    for member in members {
        if arg_ty.is_assignable_to(member) {
            return Some(member.union_variant_name());
        }
    }
    None
}

/// Detect `x is None` pattern in a Compare expression. Returns the variable name.
/// Check if a block of HIR statements always exits (return, break, continue).
/// Used for early-return narrowing in codegen.
fn codegen_body_always_exits(stmts: &[HirStmt]) -> bool {
    if let Some(last) = stmts.last() {
        matches!(last, HirStmt::Return { .. })
    } else {
        false
    }
}

/// Detect `x is None` pattern. Returns the variable name.
/// Only matches when the variable type is an Option (T | None with exactly 2 members).
fn detect_is_none_var(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Compare { left, ops, comparators, .. } = expr {
        if ops.len() == 1 && ops[0] == "is" && matches!(comparators[0], HirExpr::NoneLiteral) {
            if let HirExpr::Name { name, ty } = left.as_ref() {
                // Only match for Option types (2-member unions with None)
                if is_option_type(ty) {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

/// Detect `x is None` pattern for 3+ member unions containing None.
/// Returns (var_name, enum_name, non_none_variants).
fn detect_is_none_union_var(expr: &HirExpr) -> Option<(String, String, Vec<(String, Type)>)> {
    if let HirExpr::Compare { left, ops, comparators, .. } = expr {
        if ops.len() == 1 && ops[0] == "is" && matches!(comparators[0], HirExpr::NoneLiteral) {
            if let HirExpr::Name { name, ty } = left.as_ref() {
                if let Type::Union(members) = ty {
                    let has_none = members.iter().any(|m| matches!(m, Type::None));
                    let non_none: Vec<&Type> = members.iter().filter(|m| !matches!(m, Type::None)).collect();
                    // Only match for 3+ member unions (not simple Option)
                    if has_none && non_none.len() >= 2 {
                        let enum_name = ty.union_enum_name();
                        let non_none_variants: Vec<(String, Type)> = non_none.iter()
                            .map(|t| (t.union_variant_name(), (*t).clone()))
                            .collect();
                        return Some((name.clone(), enum_name, non_none_variants));
                    }
                }
            }
        }
    }
    None
}

/// Check if a type is hashable (for codegen derive decisions).
/// Emit a BigInt expression, cloning if it's a variable name (to avoid move).
impl RustEmitter {
    fn emit_expr_with_bigint_clone(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Name { .. } => {
                self.emit_expr(expr);
                self.write(".clone()");
            }
            HirExpr::FieldAccess { .. } => {
                self.emit_expr(expr);
                self.write(".clone()");
            }
            _ => {
                self.emit_expr(expr);
            }
        }
    }
}

fn is_hashable_type_codegen(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Bool | Type::Str | Type::None | Type::BigInt => true,
        Type::Float => false,
        _ => false,
    }
}

/// Check if a module uses the `bigint` type anywhere.
fn module_uses_bigint(module: &HirModule) -> bool {
    fn type_has_bigint(ty: &Type) -> bool {
        match ty {
            Type::BigInt => true,
            Type::List(t) | Type::Set(t) => type_has_bigint(t),
            Type::Dict(k, v) => type_has_bigint(k) || type_has_bigint(v),
            Type::Tuple(ts) | Type::Union(ts) => ts.iter().any(type_has_bigint),
            Type::Result(ok, err) => type_has_bigint(ok) || type_has_bigint(err),
            _ => false,
        }
    }
    fn expr_has_bigint(expr: &HirExpr) -> bool {
        type_has_bigint(expr.ty())
    }
    fn stmts_have_bigint(stmts: &[HirStmt]) -> bool {
        stmts.iter().any(|s| stmt_has_bigint(s))
    }
    fn stmt_has_bigint(stmt: &HirStmt) -> bool {
        match stmt {
            HirStmt::Let { ty, value, .. } => type_has_bigint(ty) || expr_has_bigint(value),
            HirStmt::Return { value } => value.as_ref().map(|e| expr_has_bigint(e)).unwrap_or(false),
            HirStmt::Expr { expr } => expr_has_bigint(expr),
            HirStmt::If { condition, then_body, else_body, elif_clauses, .. } => {
                expr_has_bigint(condition) || stmts_have_bigint(then_body)
                    || else_body.as_ref().map(|b| stmts_have_bigint(b)).unwrap_or(false)
                    || elif_clauses.iter().any(|(_, b)| stmts_have_bigint(b))
            }
            HirStmt::While { body, .. } => stmts_have_bigint(body),
            HirStmt::For { body, .. } => stmts_have_bigint(body),
            _ => false,
        }
    }
    for func in &module.functions {
        if type_has_bigint(&func.return_type) { return true; }
        if func.params.iter().any(|p| type_has_bigint(&p.ty)) { return true; }
        if stmts_have_bigint(&func.body) { return true; }
    }
    for class in &module.classes {
        if class.fields.iter().any(|(_, t)| type_has_bigint(t)) { return true; }
        for method in &class.methods {
            if type_has_bigint(&method.return_type) { return true; }
            if method.params.iter().any(|p| type_has_bigint(&p.ty)) { return true; }
            if stmts_have_bigint(&method.body) { return true; }
        }
    }
    false
}

/// Collect all parts of a chained string concatenation (`a + b + c`).
/// Recursively flattens nested BinOp::Add on strings into a flat list of expressions.
fn collect_string_concat_parts<'a>(expr: &'a HirExpr, parts: &mut Vec<&'a HirExpr>) {
    if let HirExpr::BinOp { left, op, right, ty } = expr {
        if op == "+" && *ty == Type::Str {
            collect_string_concat_parts(left, parts);
            collect_string_concat_parts(right, parts);
            return;
        }
    }
    parts.push(expr);
}

/// Check if a method body contains any field assignments or attribute augmented assignments (self.field = ... or self.field += ...).
fn body_contains_field_assign_codegen(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(|s| {
        match s {
            HirStmt::FieldAssign { .. }
            | HirStmt::AttributeAugAssign { .. }
            | HirStmt::AttributeSubscriptAssign { .. } => true,
            HirStmt::Expr { expr } => expr_contains_self_field_mutation(expr),
            HirStmt::Return { value: Some(expr) } => expr_contains_self_field_mutation(expr),
            HirStmt::Let { value, .. } => expr_contains_self_field_mutation(value),
            HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                body_contains_field_assign_codegen(then_body)
                    || elif_clauses.iter().any(|(_, body)| body_contains_field_assign_codegen(body))
                    || else_body.as_ref().map_or(false, |b| body_contains_field_assign_codegen(b))
            }
            HirStmt::While { body, .. } | HirStmt::For { body, .. } => {
                body_contains_field_assign_codegen(body)
            }
            _ => false,
        }
    })
}

/// Check if an expression contains a mutating method call on a self field (e.g., self.items.append(...)).
fn expr_contains_self_field_mutation(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::MethodCall { object, method, .. } => {
            // Check if calling a mutating method on self.field
            let is_self_field = matches!(object.as_ref(), HirExpr::FieldAccess { object: inner, .. }
                if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"));
            if is_self_field && MUTATING_METHODS.contains(&method.as_str()) {
                return true;
            }
            // Recurse into the object
            expr_contains_self_field_mutation(object)
        }
        _ => false,
    }
}

/// Check if a type contains a specific type variable name.
fn type_contains_typevar(ty: &Type, tv_name: &str) -> bool {
    match ty {
        Type::TypeVar(name) => name == tv_name,
        Type::List(inner) => type_contains_typevar(inner, tv_name),
        Type::Set(inner) => type_contains_typevar(inner, tv_name),
        Type::Dict(key, val) => type_contains_typevar(key, tv_name) || type_contains_typevar(val, tv_name),
        Type::Tuple(elems) => elems.iter().any(|e| type_contains_typevar(e, tv_name)),
        Type::Union(members) => members.iter().any(|m| type_contains_typevar(m, tv_name)),
        Type::Result(ok, err) => type_contains_typevar(ok, tv_name) || type_contains_typevar(err, tv_name),
        Type::Class { fields, methods, .. } => {
            fields.iter().any(|(_, t)| type_contains_typevar(t, tv_name))
                || methods.iter().any(|(_, ft)| {
                    ft.params.iter().any(|(_, t, _)| type_contains_typevar(t, tv_name))
                        || type_contains_typevar(&ft.return_type, tv_name)
                })
        }
        _ => false,
    }
}

/// Check if a type references a specific class name (directly or via union/option).
fn type_references_class(ty: &Type, class_name: &str) -> bool {
    match ty {
        Type::Class { name, .. } => name == class_name,
        Type::Union(members) => members.iter().any(|m| type_references_class(m, class_name)),
        Type::List(inner) => type_references_class(inner, class_name),
        Type::Dict(key, val) => type_references_class(key, class_name) || type_references_class(val, class_name),
        Type::Tuple(elems) => elems.iter().any(|e| type_references_class(e, class_name)),
        Type::Result(ok, err) => type_references_class(ok, class_name) || type_references_class(err, class_name),
        _ => false,
    }
}

/// Generate the Rust type string for a recursive field.
/// For `ClassName | None` -> `Option<Box<ClassName>>`
/// For `ClassName` directly -> `Box<ClassName>`
fn recursive_field_rust_type(ty: &Type, class_name: &str) -> String {
    match ty {
        Type::Union(members) => {
            let non_none: Vec<&Type> = members.iter().filter(|m| !matches!(m, Type::None)).collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                // T | None where T references the class -> Option<Box<T>>
                if type_references_class(non_none[0], class_name) {
                    format!("Option<Box<{}>>", non_none[0].rust_type())
                } else {
                    ty.rust_type()
                }
            } else {
                // General union with recursive member - wrap the whole thing in Box
                format!("Box<{}>", ty.rust_type())
            }
        }
        Type::Class { name, .. } if name == class_name => {
            format!("Box<{}>", name)
        }
        _ => format!("Box<{}>", ty.rust_type()),
    }
}

/// Check if a variable name is referenced anywhere in a list of statements.
fn stmts_reference_var(stmts: &[HirStmt], var_name: &str) -> bool {
    for stmt in stmts {
        match stmt {
            HirStmt::Expr { expr } => {
                if expr_references_var(expr, var_name) { return true; }
            }
            HirStmt::Return { value } => {
                if let Some(expr) = value {
                    if expr_references_var(expr, var_name) { return true; }
                }
            }
            HirStmt::Yield { value } => {
                if expr_references_var(value, var_name) { return true; }
            }
            HirStmt::Let { value, .. } => {
                if expr_references_var(value, var_name) { return true; }
            }
            HirStmt::Assign { value, .. } => {
                if expr_references_var(value, var_name) { return true; }
            }
            HirStmt::FieldAssign { value, .. } => {
                if expr_references_var(value, var_name) { return true; }
            }
            HirStmt::SubscriptAssign { index, value, .. } => {
                if expr_references_var(index, var_name) { return true; }
                if expr_references_var(value, var_name) { return true; }
            }
            HirStmt::AttributeAugAssign { value, .. } => {
                if expr_references_var(value, var_name) { return true; }
            }
            HirStmt::If { condition, then_body, elif_clauses, else_body } => {
                if expr_references_var(condition, var_name) { return true; }
                if stmts_reference_var(then_body, var_name) { return true; }
                for (cond, body) in elif_clauses {
                    if expr_references_var(cond, var_name) { return true; }
                    if stmts_reference_var(body, var_name) { return true; }
                }
                if let Some(eb) = else_body {
                    if stmts_reference_var(eb, var_name) { return true; }
                }
            }
            HirStmt::While { condition, body, .. } => {
                if expr_references_var(condition, var_name) { return true; }
                if stmts_reference_var(body, var_name) { return true; }
            }
            HirStmt::For { iter, body, .. } => {
                if expr_references_var(iter, var_name) { return true; }
                if stmts_reference_var(body, var_name) { return true; }
            }
            HirStmt::With { items, body, .. } => {
                for (_, value, _) in items {
                    if expr_references_var(value, var_name) { return true; }
                }
                if stmts_reference_var(body, var_name) { return true; }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                if stmts_reference_var(body, var_name) { return true; }
                for handler in handlers {
                    if stmts_reference_var(&handler.body, var_name) { return true; }
                }
            }
            HirStmt::Raise { value } => {
                if expr_references_var(value, var_name) { return true; }
            }
            HirStmt::AugAssign { value, .. } => {
                if expr_references_var(value, var_name) { return true; }
            }
            _ => {}
        }
    }
    false
}

/// Check if an expression references a variable name.
fn expr_references_var(expr: &HirExpr, var_name: &str) -> bool {
    match expr {
        HirExpr::Name { name, .. } => name == var_name,
        HirExpr::BinOp { left, right, .. } => {
            expr_references_var(left, var_name) || expr_references_var(right, var_name)
        }
        HirExpr::BoolOp { values, .. } => {
            values.iter().any(|v| expr_references_var(v, var_name))
        }
        HirExpr::UnaryOp { operand, .. } => expr_references_var(operand, var_name),
        HirExpr::Call { args, .. } => args.iter().any(|a| expr_references_var(a, var_name)),
        HirExpr::MethodCall { object, args, .. } => {
            expr_references_var(object, var_name) || args.iter().any(|a| expr_references_var(a, var_name))
        }
        HirExpr::FieldAccess { object, .. } => expr_references_var(object, var_name),
        HirExpr::Index { object, index, .. } => {
            expr_references_var(object, var_name) || expr_references_var(index, var_name)
        }
        HirExpr::ListLiteral { elements, .. } => elements.iter().any(|e| expr_references_var(e, var_name)),
        HirExpr::SetLiteral { elements, .. } => elements.iter().any(|e| expr_references_var(e, var_name)),
        HirExpr::TupleLiteral { elements, .. } => elements.iter().any(|e| expr_references_var(e, var_name)),
        HirExpr::Compare { left, comparators, .. } => {
            expr_references_var(left, var_name) || comparators.iter().any(|c| expr_references_var(c, var_name))
        }
        HirExpr::IfExpr { condition, then_expr, else_expr, .. } => {
            expr_references_var(condition, var_name) || expr_references_var(then_expr, var_name) || expr_references_var(else_expr, var_name)
        }
        HirExpr::Lambda { body, .. } => expr_references_var(body, var_name),
        HirExpr::ListComp { expr: e, generators, .. } => {
            expr_references_var(e, var_name) || generators.iter().any(|(_, iter, filter)| {
                expr_references_var(iter, var_name) || filter.as_ref().map_or(false, |f| expr_references_var(f, var_name))
            })
        }
        HirExpr::QuestionMark { expr, .. } => expr_references_var(expr, var_name),
        HirExpr::OkWrap { value, .. } => expr_references_var(value, var_name),
        HirExpr::ErrWrap { value, .. } => expr_references_var(value, var_name),
        HirExpr::DictLiteral { keys, values, .. } => keys.iter().chain(values.iter()).any(|e| expr_references_var(e, var_name)),
        _ => false,
    }
}

/// Check if a function body contains any yield statements (making it a generator).
/// Check if a try body contains a return statement with a non-unit value.
/// Used to determine if the try closure needs to return T instead of ().
fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    for stmt in stmts {
        match stmt {
            HirStmt::Return { value: Some(val) } => {
                // A return with a non-None value
                if !matches!(val, HirExpr::NoneLiteral) {
                    return true;
                }
            }
            HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                if try_body_has_value_return(then_body) { return true; }
                for (_, body) in elif_clauses {
                    if try_body_has_value_return(body) { return true; }
                }
                if let Some(eb) = else_body {
                    if try_body_has_value_return(eb) { return true; }
                }
            }
            HirStmt::While { body, .. } => {
                if try_body_has_value_return(body) { return true; }
            }
            HirStmt::For { body, .. } => {
                if try_body_has_value_return(body) { return true; }
            }
            HirStmt::With { body, .. } => {
                if try_body_has_value_return(body) { return true; }
            }
            _ => {}
        }
    }
    false
}

pub fn body_contains_yield(stmts: &[HirStmt]) -> bool {
    for stmt in stmts {
        match stmt {
            HirStmt::Yield { .. } => return true,
            HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                if body_contains_yield(then_body) { return true; }
                for (_, body) in elif_clauses {
                    if body_contains_yield(body) { return true; }
                }
                if let Some(eb) = else_body {
                    if body_contains_yield(eb) { return true; }
                }
            }
            HirStmt::While { body, else_body, .. } => {
                if body_contains_yield(body) { return true; }
                if let Some(eb) = else_body {
                    if body_contains_yield(eb) { return true; }
                }
            }
            HirStmt::For { body, else_body, .. } => {
                if body_contains_yield(body) { return true; }
                if let Some(eb) = else_body {
                    if body_contains_yield(eb) { return true; }
                }
            }
            HirStmt::With { body, .. } => {
                if body_contains_yield(body) { return true; }
            }
            _ => {}
        }
    }
    false
}

/// Check if a type needs .clone() when accessed from &self (non-Copy types).
fn needs_clone_for_type(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Float | Type::Bool | Type::None => false,
        Type::LiteralInt(_) | Type::LiteralBool(_) => false,
        Type::Str | Type::LiteralStr(_) => true, // String is not Copy
        Type::List(_) | Type::Dict(_, _) => true,
        Type::Tuple(_) => true, // tuples of non-Copy are non-Copy
        Type::Class { .. } => true,
        Type::Newtype { .. } => true,
        Type::TypeVar(_) => true, // Generic type params have T: Clone bound, so .clone() is safe
        Type::BigInt => true, // num_bigint::BigInt is not Copy
        _ => false,
    }
}

/// Mutating methods that require the receiver variable to be `mut`.
const MUTATING_METHODS: &[&str] = &[
    "append", "appendleft", "extend", "insert", "clear", "reverse", "sort",
    "pop", "popleft", "remove",
    "push_str", "update", "add", "discard",
];

/// Collect the set of variable names that are mutated in a function body.
/// A variable is mutated if it appears in:
/// - `HirStmt::Assign` (reassignment)
/// - `HirStmt::AugAssign` (augmented assignment like +=)
/// - `HirStmt::Expr` containing a `MethodCall` on the variable with a mutating method
/// - `HirStmt::Delete` on the variable
fn collect_mutated_vars(stmts: &[HirStmt]) -> HashSet<String> {
    let mut mutated = HashSet::new();
    collect_mutated_vars_inner(stmts, &mut mutated, None);
    mutated
}

fn collect_mutated_vars_with_sigs(stmts: &[HirStmt], func_signatures: &HashMap<String, (Vec<(Type, ParamConvention)>, Type)>) -> HashSet<String> {
    let mut mutated = HashSet::new();
    collect_mutated_vars_inner(stmts, &mut mutated, Some(func_signatures));
    mutated
}

fn collect_mutated_vars_inner(stmts: &[HirStmt], mutated: &mut HashSet<String>, func_signatures: Option<&HashMap<String, (Vec<(Type, ParamConvention)>, Type)>>) {
    for stmt in stmts {
        match stmt {
            HirStmt::Assign { name, .. } => {
                mutated.insert(name.clone());
            }
            HirStmt::AugAssign { name, .. } => {
                mutated.insert(name.clone());
            }
            HirStmt::Expr { expr } => {
                collect_mutated_vars_in_expr(expr, mutated, func_signatures);
            }
            HirStmt::Let { value, .. } => {
                // Scan the value expression for mutating method calls
                collect_mutated_vars_in_expr(value, mutated, func_signatures);
            }
            HirStmt::Return { value: Some(expr) } => {
                collect_mutated_vars_in_expr(expr, mutated, func_signatures);
            }
            HirStmt::If { condition, then_body, elif_clauses, else_body } => {
                collect_mutated_vars_in_expr(condition, mutated, func_signatures);
                collect_mutated_vars_inner(then_body, mutated, func_signatures);
                for (cond, body) in elif_clauses {
                    collect_mutated_vars_in_expr(cond, mutated, func_signatures);
                    collect_mutated_vars_inner(body, mutated, func_signatures);
                }
                if let Some(body) = else_body {
                    collect_mutated_vars_inner(body, mutated, func_signatures);
                }
            }
            HirStmt::While { condition, body, else_body } => {
                collect_mutated_vars_in_expr(condition, mutated, func_signatures);
                collect_mutated_vars_inner(body, mutated, func_signatures);
                if let Some(eb) = else_body {
                    collect_mutated_vars_inner(eb, mutated, func_signatures);
                }
            }
            HirStmt::For { body, else_body, .. } => {
                collect_mutated_vars_inner(body, mutated, func_signatures);
                if let Some(eb) = else_body {
                    collect_mutated_vars_inner(eb, mutated, func_signatures);
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                collect_mutated_vars_inner(body, mutated, func_signatures);
                for handler in handlers {
                    collect_mutated_vars_inner(&handler.body, mutated, func_signatures);
                }
            }
            HirStmt::SubscriptAssign { object, .. } => {
                mutated.insert(object.clone());
            }
            HirStmt::NestedSubscriptAssign { object, .. } => {
                mutated.insert(object.clone());
            }
            HirStmt::SubscriptAugAssign { object, .. } => {
                mutated.insert(object.clone());
            }
            HirStmt::AttributeAugAssign { object, .. } => {
                mutated.insert(object.clone());
            }
            HirStmt::Delete { object, .. } => {
                if let HirExpr::Name { name, .. } = object {
                    mutated.insert(name.clone());
                }
            }
            HirStmt::Yield { value } => {
                collect_mutated_vars_in_expr(value, mutated, func_signatures);
            }
            HirStmt::With { items, body, .. } => {
                for (_, value, _) in items {
                    collect_mutated_vars_in_expr(value, mutated, func_signatures);
                }
                collect_mutated_vars_inner(body, mutated, func_signatures);
            }
            _ => {}
        }
    }
}

fn collect_mutated_vars_in_expr(expr: &HirExpr, mutated: &mut HashSet<String>, func_signatures: Option<&HashMap<String, (Vec<(Type, ParamConvention)>, Type)>>) {
    match expr {
        HirExpr::MethodCall { object, method, args, .. } => {
            if MUTATING_METHODS.contains(&method.as_str()) {
                if let HirExpr::Name { name, .. } = object.as_ref() {
                    mutated.insert(name.clone());
                }
            }
            // Class method calls may mutate the object (conservative)
            if matches!(object.ty(), Type::Class { .. }) {
                if let HirExpr::Name { name, .. } = object.as_ref() {
                    mutated.insert(name.clone());
                }
            }
            // Recurse into sub-expressions
            collect_mutated_vars_in_expr(object, mutated, func_signatures);
            for arg in args {
                collect_mutated_vars_in_expr(arg, mutated, func_signatures);
            }
        }
        HirExpr::Call { func, args, .. } => {
            // Mark variables passed to MutBorrow params as mutated (need `let mut` in Rust)
            if let Some(sigs) = func_signatures {
                if let Some((param_convs, _)) = sigs.get(func) {
                    for (i, arg) in args.iter().enumerate() {
                        if let Some((_, ParamConvention::MutBorrow)) = param_convs.get(i) {
                            if let HirExpr::Name { name, .. } = arg {
                                mutated.insert(name.clone());
                            }
                        }
                    }
                }
            }
            for arg in args {
                collect_mutated_vars_in_expr(arg, mutated, func_signatures);
            }
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_mutated_vars_in_expr(left, mutated, func_signatures);
            collect_mutated_vars_in_expr(right, mutated, func_signatures);
        }
        HirExpr::UnaryOp { operand, .. } => {
            collect_mutated_vars_in_expr(operand, mutated, func_signatures);
        }
        HirExpr::Compare { left, comparators, .. } => {
            collect_mutated_vars_in_expr(left, mutated, func_signatures);
            for c in comparators {
                collect_mutated_vars_in_expr(c, mutated, func_signatures);
            }
        }
        HirExpr::BoolOp { values, .. } => {
            for v in values {
                collect_mutated_vars_in_expr(v, mutated, func_signatures);
            }
        }
        HirExpr::IfExpr { condition, then_expr, else_expr, .. } => {
            collect_mutated_vars_in_expr(condition, mutated, func_signatures);
            collect_mutated_vars_in_expr(then_expr, mutated, func_signatures);
            collect_mutated_vars_in_expr(else_expr, mutated, func_signatures);
        }
        HirExpr::Index { object, index, .. } => {
            collect_mutated_vars_in_expr(object, mutated, func_signatures);
            collect_mutated_vars_in_expr(index, mutated, func_signatures);
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(e) = part {
                    collect_mutated_vars_in_expr(e, mutated, func_signatures);
                }
            }
        }
        _ => {}
    }
}

/// Collect all variable names and their types referenced in a list of statements.
fn collect_referenced_vars_with_types(stmts: &[HirStmt]) -> Vec<(String, Type)> {
    let mut refs: HashMap<String, Type> = HashMap::new();
    collect_referenced_vars_with_types_inner(stmts, &mut refs);
    refs.into_iter().collect()
}

fn collect_referenced_vars_with_types_inner(stmts: &[HirStmt], refs: &mut HashMap<String, Type>) {
    for stmt in stmts {
        match stmt {
            HirStmt::Let { value, .. } => {
                collect_typed_refs_in_expr(value, refs);
            }
            HirStmt::Assign { value, .. } => {
                collect_typed_refs_in_expr(value, refs);
            }
            HirStmt::AugAssign { value, .. } => {
                collect_typed_refs_in_expr(value, refs);
            }
            HirStmt::Return { value: Some(expr) } => {
                collect_typed_refs_in_expr(expr, refs);
            }
            HirStmt::Expr { expr } => {
                collect_typed_refs_in_expr(expr, refs);
            }
            HirStmt::If { condition, then_body, elif_clauses, else_body } => {
                collect_typed_refs_in_expr(condition, refs);
                collect_referenced_vars_with_types_inner(then_body, refs);
                for (cond, body) in elif_clauses {
                    collect_typed_refs_in_expr(cond, refs);
                    collect_referenced_vars_with_types_inner(body, refs);
                }
                if let Some(body) = else_body {
                    collect_referenced_vars_with_types_inner(body, refs);
                }
            }
            HirStmt::While { condition, body, .. } => {
                collect_typed_refs_in_expr(condition, refs);
                collect_referenced_vars_with_types_inner(body, refs);
            }
            HirStmt::For { iter, body, .. } => {
                collect_typed_refs_in_expr(iter, refs);
                collect_referenced_vars_with_types_inner(body, refs);
            }
            HirStmt::FieldAssign { value, .. } => {
                collect_typed_refs_in_expr(value, refs);
            }
            HirStmt::SubscriptAssign { index, value, .. } => {
                collect_typed_refs_in_expr(index, refs);
                collect_typed_refs_in_expr(value, refs);
            }
            _ => {}
        }
    }
}

fn collect_typed_refs_in_expr(expr: &HirExpr, refs: &mut HashMap<String, Type>) {
    match expr {
        HirExpr::Name { name, ty } => {
            refs.entry(name.clone()).or_insert_with(|| ty.clone());
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_typed_refs_in_expr(left, refs);
            collect_typed_refs_in_expr(right, refs);
        }
        HirExpr::BoolOp { values, .. } => {
            for v in values {
                collect_typed_refs_in_expr(v, refs);
            }
        }
        HirExpr::UnaryOp { operand, .. } => {
            collect_typed_refs_in_expr(operand, refs);
        }
        HirExpr::Compare { left, comparators, .. } => {
            collect_typed_refs_in_expr(left, refs);
            for c in comparators {
                collect_typed_refs_in_expr(c, refs);
            }
        }
        HirExpr::Call { args, .. } => {
            for a in args {
                collect_typed_refs_in_expr(a, refs);
            }
        }
        HirExpr::MethodCall { object, args, .. } => {
            collect_typed_refs_in_expr(object, refs);
            for a in args {
                collect_typed_refs_in_expr(a, refs);
            }
        }
        HirExpr::Index { object, index, .. } => {
            collect_typed_refs_in_expr(object, refs);
            collect_typed_refs_in_expr(index, refs);
        }
        HirExpr::IfExpr { condition, then_expr, else_expr, .. } => {
            collect_typed_refs_in_expr(condition, refs);
            collect_typed_refs_in_expr(then_expr, refs);
            collect_typed_refs_in_expr(else_expr, refs);
        }
        HirExpr::ListLiteral { elements, .. } | HirExpr::TupleLiteral { elements, .. } | HirExpr::SetLiteral { elements, .. } => {
            for e in elements {
                collect_typed_refs_in_expr(e, refs);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for k in keys { collect_typed_refs_in_expr(k, refs); }
            for v in values { collect_typed_refs_in_expr(v, refs); }
        }
        HirExpr::Lambda { body, .. } => {
            collect_typed_refs_in_expr(body, refs);
        }
        _ => {}
    }
}

/// Collect all variable names defined (let-bound) in a list of statements.
/// Does NOT recurse into nested functions.
fn collect_locally_defined_vars(stmts: &[HirStmt]) -> HashSet<String> {
    let mut defined = HashSet::new();
    for stmt in stmts {
        match stmt {
            HirStmt::Let { name, .. } => {
                defined.insert(name.clone());
            }
            HirStmt::For { target, body, .. } => {
                defined.insert(target.clone());
                // Also collect from body
                defined.extend(collect_locally_defined_vars(body));
            }
            HirStmt::TupleUnpack { targets, .. } => {
                for (name, _) in targets {
                    defined.insert(name.clone());
                }
            }
            HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                defined.extend(collect_locally_defined_vars(then_body));
                for (_, body) in elif_clauses {
                    defined.extend(collect_locally_defined_vars(body));
                }
                if let Some(body) = else_body {
                    defined.extend(collect_locally_defined_vars(body));
                }
            }
            HirStmt::While { body, .. } => {
                defined.extend(collect_locally_defined_vars(body));
            }
            HirStmt::NestedFunction { func } => {
                // The nested function name itself is defined
                defined.insert(func.name.clone());
            }
            _ => {}
        }
    }
    defined
}

/// Check if a function body contains calls to a specific function name.
fn body_calls_function(stmts: &[HirStmt], func_name: &str) -> bool {
    for stmt in stmts {
        match stmt {
            HirStmt::Let { value, .. } => {
                if expr_calls_function(value, func_name) { return true; }
            }
            HirStmt::Assign { value, .. } => {
                if expr_calls_function(value, func_name) { return true; }
            }
            HirStmt::AugAssign { value, .. } => {
                if expr_calls_function(value, func_name) { return true; }
            }
            HirStmt::Return { value: Some(expr) } => {
                if expr_calls_function(expr, func_name) { return true; }
            }
            HirStmt::Expr { expr } => {
                if expr_calls_function(expr, func_name) { return true; }
            }
            HirStmt::If { condition, then_body, elif_clauses, else_body } => {
                if expr_calls_function(condition, func_name) { return true; }
                if body_calls_function(then_body, func_name) { return true; }
                for (cond, body) in elif_clauses {
                    if expr_calls_function(cond, func_name) { return true; }
                    if body_calls_function(body, func_name) { return true; }
                }
                if let Some(body) = else_body {
                    if body_calls_function(body, func_name) { return true; }
                }
            }
            HirStmt::While { condition, body, .. } => {
                if expr_calls_function(condition, func_name) { return true; }
                if body_calls_function(body, func_name) { return true; }
            }
            HirStmt::For { body, .. } => {
                if body_calls_function(body, func_name) { return true; }
            }
            _ => {}
        }
    }
    false
}

fn expr_calls_function(expr: &HirExpr, func_name: &str) -> bool {
    match expr {
        HirExpr::Call { func, args, .. } => {
            if func == func_name { return true; }
            args.iter().any(|a| expr_calls_function(a, func_name))
        }
        HirExpr::BinOp { left, right, .. } => {
            expr_calls_function(left, func_name) || expr_calls_function(right, func_name)
        }
        HirExpr::BoolOp { values, .. } => {
            values.iter().any(|v| expr_calls_function(v, func_name))
        }
        HirExpr::UnaryOp { operand, .. } => {
            expr_calls_function(operand, func_name)
        }
        HirExpr::Compare { left, comparators, .. } => {
            expr_calls_function(left, func_name) || comparators.iter().any(|c| expr_calls_function(c, func_name))
        }
        HirExpr::MethodCall { object, args, .. } => {
            expr_calls_function(object, func_name) || args.iter().any(|a| expr_calls_function(a, func_name))
        }
        HirExpr::IfExpr { condition, then_expr, else_expr, .. } => {
            expr_calls_function(condition, func_name) || expr_calls_function(then_expr, func_name) || expr_calls_function(else_expr, func_name)
        }
        HirExpr::Index { object, index, .. } => {
            expr_calls_function(object, func_name) || expr_calls_function(index, func_name)
        }
        HirExpr::ListLiteral { elements, .. } | HirExpr::TupleLiteral { elements, .. } | HirExpr::SetLiteral { elements, .. } => {
            elements.iter().any(|e| expr_calls_function(e, func_name))
        }
        HirExpr::Lambda { body, .. } => {
            expr_calls_function(body, func_name)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_hir::*;
    use sifr_type_system::{Type, ParamConvention};

    #[test]
    fn test_simple_function_codegen() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::StringLiteral("Hello, World!".to_string())],
                        ty: Type::None,
                    },
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("fn main()"));
        assert!(rust_code.contains("println!"));
        assert!(rust_code.contains("Hello, World!"));
    }

    #[test]
    fn test_arithmetic_codegen() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "add".to_string(),
                params: vec![
                    HirParam { name: "a".to_string(), ty: Type::Int, default: None, keyword_only: false, convention: ParamConvention::Own },
                    HirParam { name: "b".to_string(), ty: Type::Int, default: None, keyword_only: false, convention: ParamConvention::Own },
                ],
                return_type: Type::Int,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::BinOp {
                        left: Box::new(HirExpr::Name { name: "a".to_string(), ty: Type::Int }),
                        op: "+".to_string(),
                        right: Box::new(HirExpr::Name { name: "b".to_string(), ty: Type::Int }),
                        ty: Type::Int,
                    }),
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("fn add(a: i64, b: i64) -> i64"));
        assert!(rust_code.contains("return a + b;"));
    }

    // --- Codegen Quality Tests ---

    #[test]
    fn test_no_unnecessary_mut() {
        // Variable that is never reassigned should NOT have `mut`
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "x".to_string(),
                        ty: Type::Int,
                        value: HirExpr::IntLiteral(42),
                        is_mutable: true, // HIR says mutable, but codegen should ignore
                    },
                    HirStmt::Expr {
                        expr: HirExpr::Call {
                            func: "print".to_string(),
                            args: vec![HirExpr::Name { name: "x".to_string(), ty: Type::Int }],
                            ty: Type::None,
                        },
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("let x: i64"), "should emit `let x` without mut");
        assert!(!rust_code.contains("let mut x"), "should NOT emit `let mut x`");
    }

    #[test]
    fn test_mut_on_reassigned_variable() {
        // Variable that IS reassigned should have `mut`
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "x".to_string(),
                        ty: Type::Int,
                        value: HirExpr::IntLiteral(0),
                        is_mutable: true,
                    },
                    HirStmt::Assign {
                        name: "x".to_string(),
                        value: HirExpr::IntLiteral(1),
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("let mut x: i64"), "should emit `let mut x` for reassigned var");
    }

    #[test]
    fn test_println_fstring_inlined() {
        // print(f"hello {name}") should emit println!("hello {}", name) not println!("{}", format!(...))
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "name".to_string(),
                        ty: Type::Str,
                        value: HirExpr::StringLiteral("World".to_string()),
                        is_mutable: false,
                    },
                    HirStmt::Expr {
                        expr: HirExpr::Call {
                            func: "print".to_string(),
                            args: vec![HirExpr::FString {
                                parts: vec![
                                    HirFStringPart::Literal("Hello, ".to_string()),
                                    HirFStringPart::Expr(HirExpr::Name { name: "name".to_string(), ty: Type::Str }),
                                    HirFStringPart::Literal("!".to_string()),
                                ],
                                ty: Type::Str,
                            }],
                            ty: Type::None,
                        },
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("println!(\"Hello, {}!\", name)"), "should inline f-string into println!");
        assert!(!rust_code.contains("format!(\"Hello, {}!\""), "should NOT have standalone format! inside println!");
    }

    #[test]
    fn test_no_tostring_in_println() {
        // print("hello") should emit println!("{}", "hello") not println!("{}", "hello".to_string())
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::StringLiteral("hello".to_string())],
                        ty: Type::None,
                    },
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("println!(\"hello\")"), "should inline string literal directly into println!");
        assert!(!rust_code.contains("\"hello\".to_string()"), "should NOT have .to_string() in println context");
    }

    #[test]
    fn test_hashmap_short_name() {
        // Dict literal should use HashMap::from not std::collections::HashMap::from
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Let {
                    name: "d".to_string(),
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                    value: HirExpr::DictLiteral {
                        keys: vec![HirExpr::StringLiteral("a".to_string())],
                        values: vec![HirExpr::IntLiteral(1)],
                        ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                    },
                    is_mutable: false,
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("use std::collections::HashMap;"), "should have HashMap import");
        assert!(rust_code.contains("HashMap::from("), "should use short HashMap::from");
        assert!(!rust_code.contains("std::collections::HashMap::from("), "should NOT use fully qualified HashMap::from");
        assert!(rust_code.contains("HashMap<String, i64>"), "type annotation should use short HashMap");
    }

    #[test]
    fn test_dict_get_string_literal_key() {
        // d["key"] should emit d.get("key") not d.get(&"key".to_string())
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "d".to_string(),
                        ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                        value: HirExpr::DictLiteral {
                            keys: vec![HirExpr::StringLiteral("key".to_string())],
                            values: vec![HirExpr::IntLiteral(1)],
                            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                        },
                        is_mutable: false,
                    },
                    HirStmt::Let {
                        name: "v".to_string(),
                        ty: Type::Union(vec![Type::Int, Type::None]),
                        value: HirExpr::Index {
                            object: Box::new(HirExpr::Name {
                                name: "d".to_string(),
                                ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                            }),
                            index: Box::new(HirExpr::StringLiteral("key".to_string())),
                            ty: Type::Union(vec![Type::Int, Type::None]),
                        },
                        is_mutable: false,
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains(".get(\"key\")"), "should emit .get(\"key\") for string literal key");
        assert!(!rust_code.contains("&\"key\".to_string()"), "should NOT have &\"key\".to_string()");
    }

    #[test]
    fn test_string_concat_flattened() {
        // "a" + "b" + "c" should emit format!("{}{}{}", ...) not nested format!
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Let {
                    name: "s".to_string(),
                    ty: Type::Str,
                    value: HirExpr::BinOp {
                        left: Box::new(HirExpr::BinOp {
                            left: Box::new(HirExpr::StringLiteral("a".to_string())),
                            op: "+".to_string(),
                            right: Box::new(HirExpr::StringLiteral("b".to_string())),
                            ty: Type::Str,
                        }),
                        op: "+".to_string(),
                        right: Box::new(HirExpr::StringLiteral("c".to_string())),
                        ty: Type::Str,
                    },
                    is_mutable: false,
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        // All parts are string literals, so they should be folded into a single string
        assert!(rust_code.contains("\"abc\".to_string()"), "should fold all string literals into a single string");
        assert!(!rust_code.contains("format!"), "should NOT use format! when all parts are literals");
    }

    #[test]
    fn test_mut_on_mutating_method_call() {
        // Variable with .push() call should have `mut`
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                        value: HirExpr::ListLiteral {
                            elements: vec![HirExpr::IntLiteral(1)],
                            ty: Type::List(Box::new(Type::Int)),
                        },
                        is_mutable: true,
                    },
                    HirStmt::Expr {
                        expr: HirExpr::MethodCall {
                            object: Box::new(HirExpr::Name {
                                name: "items".to_string(),
                                ty: Type::List(Box::new(Type::Int)),
                            }),
                            method: "append".to_string(),
                            args: vec![HirExpr::IntLiteral(2)],
                            ty: Type::None,
                        },
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("let mut items"), "should emit `let mut items` for variable with .push()");
    }

    #[test]
    fn test_empty_print() {
        // print() should emit println!() not println!("{}", "")
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![],
                        ty: Type::None,
                    },
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            }],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("println!()"), "should emit println!() for empty print");
        assert!(!rust_code.contains(r#"println!("{}", "")"#), "should NOT emit println with empty string arg");
    }

    #[test]
    fn test_expr_to_string_fast_path_for_lowered_leafs() {
        let mut emitter = RustEmitter::new();
        let int_code = emitter.expr_to_string(&HirExpr::IntLiteral(7));
        assert_eq!(int_code, "7_i64");

        let bool_op = HirExpr::BoolOp {
            op: "and".to_string(),
            values: vec![HirExpr::BoolLiteral(true), HirExpr::BoolLiteral(false)],
            ty: Type::Bool,
        };
        let bool_code = emitter.expr_to_string(&bool_op);
        assert_eq!(bool_code, "true && false");
    }
}
