//! Sifr Code Generation
//!
//! Translates the typed HIR into Rust source code.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(dead_code)]
#![allow(clippy::struct_excessive_bools)]

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
mod class_emitter;
mod class_method_emitter;
mod entrypoints;
mod expr_ref_emitter;
mod expr_render_helpers;
mod function_emitter;
mod generic_bounds_helpers;
mod field_analysis_helpers;
mod helpers;
mod intrinsic_method_emitters;
mod method_call_emitter;
mod intrinsics;
mod ir_imports;
mod ir_optimize;
mod ir_validate;
mod match_emitter;
mod match_guard_helpers;
mod methods;
mod module_body;
mod module_constants;
mod module_prescan;
mod operator_protocol_emitters;
mod output_helpers;
mod slice_emitter;
mod stdlib_filter;
mod stmt_support_emitter;
mod type_emitters;
mod union_type_helpers;

#[cfg(test)]
mod lib_codegen_tests;

use helpers::{
    body_calls_function, codegen_body_always_exits, collect_locally_defined_vars,
    collect_mutated_vars, collect_mutated_vars_with_sigs, collect_referenced_vars_with_types,
    collect_string_concat_parts, detect_and_not_none_vars, detect_is_none_union_var,
    detect_is_none_var, detect_is_not_none_var, detect_isinstance_union, detect_option_truthiness,
    find_union_variant, is_builtin_error_referenced, is_hashable_type_codegen, is_option_type,
    module_uses_bigint, needs_clone_for_type, stmts_reference_var, try_body_has_value_return,
    type_contains_typevar,
};
use ir_imports::collect_import_needs_from_items;
use ir_optimize::remove_trivial_clones_in_items;
use ir_validate::validate_items;
use sifr_hir::{HirExpr, HirModule, HirStmt};
use sifr_type_system::{ParamConvention, Type};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use stdlib_filter::{
    collect_and_strip_shared_prelude, dedup_rust_items, filter_stdlib_ir_to_needed,
};

type FuncSignature = (Vec<(Type, ParamConvention)>, Type);
type ModuleFuncSignatures = HashMap<String, FuncSignature>;
type StdlibFuncSignatures = HashMap<String, ModuleFuncSignatures>;
type UnionVariantTypes = Vec<(String, Type)>;
type IsinstanceUnionMatch = (String, String, String, UnionVariantTypes);
type IsNoneUnionMatch = (String, String, UnionVariantTypes);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodegenLoweringMode {
    StructuredPreferred,
    LegacyOnly,
}

pub use entrypoints::{
    generate_rust, generate_rust_test, generate_rust_test_with_mode, generate_rust_with_metadata,
    generate_rust_with_metadata_mode,
};

/// Built-in error class names that the compiler provides.
const BUILTIN_ERROR_CLASSES: &[&str] = &[
    "Error",
    "IOError",
    "ParseError",
    "ValueError",
    "DivisionError",
    "KeyError",
    "JSONDecodeError",
    "TOMLDecodeError",
    "RegexError",
    "FileNotFoundError",
    "PermissionError",
    "FileExistsError",
    "IsADirectoryError",
    "NotADirectoryError",
    "DirectoryNotEmptyError",
    "OverflowError",
    "IndexError",
    "AttributeError",
    "TypeError",
    "ZeroDivisionError",
    "RuntimeError",
    "NotImplementedError",
];

const IO_ERROR_SUBCLASSES: &[&str] = &[
    "FileNotFoundError",
    "PermissionError",
    "FileExistsError",
    "IsADirectoryError",
    "NotADirectoryError",
    "DirectoryNotEmptyError",
];

/// Result of code generation, including the Rust source and metadata.
pub struct CodegenResult {
    pub rust_source: String,
    pub used_stdlib_modules: HashSet<String>,
    pub used_intrinsic_modules: HashSet<String>,
    /// Required external crates discovered during structured lowering/codegen.
    pub required_crates: HashSet<String>,
    /// Map of `constant_name` -> (type, `rust_name`) for module-level constants
    pub constant_mappings: HashMap<String, (Type, String)>,
    /// Counters for structured lowering usage during emission.
    pub lowering_stats: LoweringStats,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LoweringStats {
    pub stmt_total: u64,
    pub stmt_structured: u64,
    pub expr_total: u64,
    pub expr_structured: u64,
    pub stmt_candidate_total: u64,
    pub stmt_candidate_structured: u64,
    pub expr_candidate_total: u64,
    pub expr_candidate_structured: u64,
}

/// Compiled stdlib information for codegen.
/// Contains per-module Rust code and intrinsic name sets.
#[derive(Default)]
pub struct StdlibCode {
    /// Map of `module_name` -> compiled Rust source code for pure Sifr functions/constants
    pub module_rust_code: HashMap<String, String>,
    /// Map of `module_name` -> set of names that are intrinsic re-exports (from _sifr.*)
    pub intrinsic_names: HashMap<String, HashSet<String>>,
    /// Map of `module_name` -> (`constant_name` -> (type, `rust_name`)) for stdlib constants
    /// This allows user code to reference stdlib constants with the correct Rust names.
    pub module_constants: HashMap<String, HashMap<String, (Type, String)>>,
    /// Map of `module_name` -> (`func_name` -> (`param_types_with_conventions`, `return_type`))
    /// for pure Sifr stdlib functions. Used to emit correct borrow prefixes at call sites.
    pub func_signatures: StdlibFuncSignatures,
    /// Map of `module_name` -> set of transitive intrinsic module dependencies.
    /// E.g., sifr.secrets depends on _sifr.crypto, so when user imports sifr.secrets,
    /// the Cargo dependencies for _sifr.crypto (rand) must be included.
    pub transitive_deps: HashMap<String, HashSet<String>>,
    /// Map of `module_name` -> set of function names that are generators (contain yield).
    /// Used to emit .`collect()` when assigning generator results to list[T] in user code.
    pub generator_functions: HashMap<String, HashSet<String>>,
    /// Set of class names that have generic type parameters across all stdlib modules.
    pub generic_classes: HashSet<String>,
}

/// Generate Rust source code from a HIR module with compiled stdlib code.
pub fn generate_rust_with_stdlib(module: &HirModule, stdlib_code: &StdlibCode) -> CodegenResult {
    generate_rust_with_stdlib_mode(module, stdlib_code, CodegenLoweringMode::StructuredPreferred)
}

pub fn generate_rust_with_stdlib_mode(
    module: &HirModule,
    stdlib_code: &StdlibCode,
    lowering_mode: CodegenLoweringMode,
) -> CodegenResult {
    let mut emitter = RustEmitter::new_with_mode(lowering_mode);
    emitter
        .stdlib_intrinsic_names
        .clone_from(&stdlib_code.intrinsic_names);
    // Register stdlib generic classes so user code skips explicit type annotations
    emitter
        .generic_classes
        .extend(stdlib_code.generic_classes.iter().cloned());

    // Pre-register stdlib constants and function signatures so user code can reference them correctly
    for import in &module.imports {
        if let Some(const_map) = stdlib_code.module_constants.get(&import.module) {
            for name in &import.names {
                if let Some((ty, rust_name)) = const_map.get(name) {
                    emitter
                        .module_constants
                        .insert(name.clone(), (ty.clone(), rust_name.clone()));
                }
            }
        }
        if let Some(sig_map) = stdlib_code.func_signatures.get(&import.module) {
            for name in &import.names {
                if let Some(sig) = sig_map.get(name) {
                    emitter.func_signatures.insert(name.clone(), sig.clone());
                }
                // Also load class method signatures (ClassName::method entries)
                let prefix = format!("{name}::");
                for (key, sig) in sig_map {
                    if key.starts_with(&prefix) {
                        emitter.func_signatures.insert(key.clone(), sig.clone());
                    }
                }
            }
            // Load class method signatures for classes returned by imported functions.
            // This handles cases like `compile_flags` returning `Pattern` - we need
            // `Pattern::search` etc. to be available for correct borrow prefix emission.
            for (key, sig) in sig_map {
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
    emitter.emit_module(module, false, false);

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
    let mut stdlib_provides_file_handle_struct = false;
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
                let filtered =
                    if let Some(imported_names) = emitter.imported_stdlib_names.get(module_name) {
                        let intrinsic_set = stdlib_code.intrinsic_names.get(module_name);
                        let pure_sifr_imports: HashSet<String> = imported_names
                            .iter()
                            .filter(|name| !intrinsic_set.is_some_and(|iset| iset.contains(*name)))
                            .cloned()
                            .collect();
                        if pure_sifr_imports.is_empty() {
                            String::new()
                        } else {
                            let mut expanded_imports = pure_sifr_imports.clone();
                            if let Some(const_map) = stdlib_code.module_constants.get(module_name) {
                                for name in &pure_sifr_imports {
                                    if const_map.contains_key(name) {
                                        expanded_imports.insert(format!("__const_{name}"));
                                    }
                                }
                            }
                            filter_stdlib_ir_to_needed(rust_code, &expanded_imports)
                        }
                    } else {
                        rust_code.clone()
                    };
                if !filtered.trim().is_empty() {
                    let prepared = collect_and_strip_shared_prelude(&filtered);
                    stdlib_needs_hashmap |= prepared.shared_needs.needs_hashmap;
                    stdlib_needs_hashset |= prepared.shared_needs.needs_hashset;
                    stdlib_needs_vecdeque |= prepared.shared_needs.needs_vecdeque;
                    stdlib_needs_file_handles |= prepared.shared_needs.needs_file_handles;
                    stdlib_provides_file_handle_struct |=
                        prepared.shared_needs.provides_file_handle_struct;
                    let stripped = prepared.stripped_code;
                    if !stripped.trim().is_empty() {
                        let deduped =
                            dedup_rust_items(&stripped, &mut emitted_items, &infra_skip_types);
                        if !deduped.trim().is_empty() {
                            let _ = writeln!(stdlib_preamble, "// --- stdlib: {module_name} ---");
                            stdlib_preamble.push_str(&deduped);
                            stdlib_preamble.push('\n');
                        }
                    }
                }
                emitted_modules.insert(module_name.clone());
            }
        }
    }

    // Compute broad feature needs first, then refine imports structurally from preamble IR.
    let needs_file_handles = emitter.needs_file_handles || stdlib_needs_file_handles;
    let needs_logging = emitter.used_stdlib_modules.contains("sifr.logging")
        || emitter.used_stdlib_modules.contains("_sifr.logging")
        || emitter.needs_logging_state;

    // File handle infrastructure always relies on HashMap + Mutex.
    let needs_hashmap_base = emitter.needs_hashmap || stdlib_needs_hashmap || needs_file_handles;
    let needs_hashset_base = emitter.needs_hashset || stdlib_needs_hashset;
    let needs_vecdeque_base = emitter.needs_vecdeque || stdlib_needs_vecdeque;
    let needs_bigint_base = emitter.needs_bigint;

    // Emit built-in error class struct definitions for any that are referenced.
    // For now this remains a compatibility shim that scans generated code.
    let combined_code = format!("{}{}", stdlib_preamble, emitter.output);
    let user_defined_error_classes: HashSet<String> = module
        .classes
        .iter()
        .filter(|c| c.is_error_type)
        .map(|c| c.name.clone())
        .collect();
    let user_defined_file_handle_struct = module.classes.iter().any(|c| c.name == "FileHandle");
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
            let (extra_fields, defaults) =
                if error_name == "JSONDecodeError" || error_name == "TOMLDecodeError" {
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
                            RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "String".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![],
                            },
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
        if !stdlib_provides_file_handle_struct && !user_defined_file_handle_struct {
            preamble_items.extend(build_file_handle_struct_items());
        }
    }

    // Emit global log level state if logging module is used.
    if needs_logging {
        preamble_items.extend(build_logging_items());
    }

    remove_trivial_clones_in_items(&mut preamble_items);
    let ir_import_needs = collect_import_needs_from_items(&preamble_items);
    let needs_hashmap = needs_hashmap_base || ir_import_needs.needs_hashmap;
    let needs_hashset = needs_hashset_base || ir_import_needs.needs_hashset;
    let needs_vecdeque = needs_vecdeque_base || ir_import_needs.needs_vecdeque;
    let needs_bigint = needs_bigint_base || ir_import_needs.needs_bigint;
    let needs_mutex = needs_file_handles || needs_logging || ir_import_needs.needs_mutex;

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
    if needs_mutex {
        import_items.push(RustItem::Use(vec![
            "std".to_string(),
            "sync".to_string(),
            "Mutex".to_string(),
        ]));
    }

    remove_trivial_clones_in_items(&mut import_items);
    let mut result = String::new();
    let import_issues = validate_items(&import_items);
    assert!(
        import_issues.is_empty(),
        "codegen IR validation failed (imports): {}",
        import_issues
            .iter()
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let preamble_issues = validate_items(&preamble_items);
    assert!(
        preamble_issues.is_empty(),
        "codegen IR validation failed (preamble): {}",
        preamble_issues
            .iter()
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    if !import_items.is_empty() {
        result.push_str(&render_items(&import_items));
        result.push('\n');
    }
    if !emitter.enum_defs.is_empty() {
        result.push_str(&emitter.enum_defs);
        result.push('\n');
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
        required_crates: {
            let mut crates = emitter.intrinsic_registry_crates;
            if needs_bigint {
                crates.insert("num-bigint".to_string());
                crates.insert("num-traits".to_string());
            }
            crates
        },
        constant_mappings: emitter.module_constants,
        lowering_stats: emitter.lowering_stats,
    }
}

/// Generate Rust source code for a multi-module project.
/// Returns a map of filename -> Rust source code.
pub fn generate_rust_multi(modules: &[(&str, &HirModule)]) -> HashMap<String, String> {
    let mut files = HashMap::new();

    for (module_name, module) in modules {
        let mut emitter = RustEmitter::new();
        let module_public = *module_name != "main";
        emitter.collect_union_types(module);
        emitter.generate_enum_definitions();
        emitter.emit_module(module, module_public, false);

        let mut result = String::new();

        // For non-main modules, add imports as `use` statements
        for import in &module.imports {
            for name in &import.names {
                // Check if this name has an alias
                if let Some((_, alias)) = import.aliases.iter().find(|(orig, _)| orig == name) {
                    let _ = writeln!(
                        result,
                        "use crate::{}::{} as {};",
                        import.module, name, alias
                    );
                } else {
                    let _ = writeln!(result, "use crate::{}::{};", import.module, name);
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

        files.insert((*module_name).to_string(), result);
    }

    files
}

/// Generate a complete Rust project (Cargo.toml + main.rs content).
pub fn generate_project(module: &HirModule, project_name: &str) -> (String, String) {
    generate_project_with_deps(module, project_name, &HashSet::new())
}

/// Generate a complete Rust project with stdlib dependencies.
pub fn generate_project_with_deps(
    module: &HirModule,
    project_name: &str,
    stdlib_modules: &HashSet<String>,
) -> (String, String) {
    generate_project_with_deps_and_crates(module, project_name, stdlib_modules, &HashSet::new())
}

/// Generate a complete Rust project with stdlib and explicit crate dependencies.
pub fn generate_project_with_deps_and_crates(
    module: &HirModule,
    project_name: &str,
    stdlib_modules: &HashSet<String>,
    required_crates: &HashSet<String>,
) -> (String, String) {
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

    for crate_name in required_crates {
        match crate_name.as_str() {
            "serde_json" => {
                if !deps.contains(&"serde_json = \"1\"".to_string()) {
                    deps.push("serde_json = \"1\"".to_string());
                }
                if !deps
                    .contains(&"serde = { version = \"1\", features = [\"derive\"] }".to_string())
                {
                    deps.push("serde = { version = \"1\", features = [\"derive\"] }".to_string());
                }
            }
            "chrono" => {
                if !deps.contains(&"chrono = \"0.4\"".to_string()) {
                    deps.push("chrono = \"0.4\"".to_string());
                }
            }
            "rand" => {
                if !deps.contains(&"rand = \"0.8\"".to_string()) {
                    deps.push("rand = \"0.8\"".to_string());
                }
            }
            "rand_distr" => {
                if !deps.contains(&"rand_distr = \"0.4\"".to_string()) {
                    deps.push("rand_distr = \"0.4\"".to_string());
                }
            }
            "regex" => {
                if !deps.contains(&"regex = \"1\"".to_string()) {
                    deps.push("regex = \"1\"".to_string());
                }
            }
            "sha2" => {
                if !deps.contains(&"sha2 = \"0.10\"".to_string()) {
                    deps.push("sha2 = \"0.10\"".to_string());
                }
            }
            "md5" => {
                if !deps.contains(&"md5 = \"0.7\"".to_string()) {
                    deps.push("md5 = \"0.7\"".to_string());
                }
            }
            "sha1" => {
                if !deps.contains(&"sha1 = \"0.10\"".to_string()) {
                    deps.push("sha1 = \"0.10\"".to_string());
                }
            }
            "blake2" => {
                if !deps.contains(&"blake2 = \"0.10\"".to_string()) {
                    deps.push("blake2 = \"0.10\"".to_string());
                }
            }
            "base64" => {
                if !deps.contains(&"base64 = \"0.22\"".to_string()) {
                    deps.push("base64 = \"0.22\"".to_string());
                }
            }
            "toml" => {
                if !deps.contains(&"toml = \"0.8\"".to_string()) {
                    deps.push("toml = \"0.8\"".to_string());
                }
            }
            "flate2" => {
                if !deps.contains(&"flate2 = \"1\"".to_string()) {
                    deps.push("flate2 = \"1\"".to_string());
                }
            }
            "zip" => {
                if !deps.contains(&"zip = \"0.6\"".to_string()) {
                    deps.push("zip = \"0.6\"".to_string());
                }
            }
            "num-bigint" => {
                if !deps.contains(&"num-bigint = \"0.4\"".to_string()) {
                    deps.push("num-bigint = \"0.4\"".to_string());
                }
            }
            "num-traits" => {
                if !deps.contains(&"num-traits = \"0.2\"".to_string()) {
                    deps.push("num-traits = \"0.2\"".to_string());
                }
            }
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
    needs_logging_state: bool,
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
    /// Function signatures: name -> (`param_types_with_conventions`, `return_type`)
    func_signatures: HashMap<String, (Vec<(Type, ParamConvention)>, Type)>,
    /// Stack tracking whether each active loop has an else clause.
    /// The last entry is the innermost active loop context.
    loop_else_stack: Vec<bool>,
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
    /// Set of (`class_name`, `field_name`) pairs that are self-referential and need Box<T>
    recursive_fields: HashSet<(String, String)>,
    /// Map from class name -> ordered list of field names (for constructor arg mapping)
    class_field_order: HashMap<String, Vec<String>>,
    /// Map from nested function name -> list of captured variable (name, type) pairs
    /// Used to pass extra args at call sites for recursive+capturing nested functions
    nested_fn_captures: HashMap<String, Vec<(String, Type)>>,
    /// Map from module-level constant name -> (type, `rust_name`)
    /// For primitives: `rust_name` is the UPPERCASE const name
    /// For strings/complex: `rust_name` is __`const_name()` function call
    module_constants: HashMap<String, (Type, String)>,
    /// Set of class names that have generic type parameters
    generic_classes: HashSet<String>,
    /// Map of generic class name -> list of type parameter names (e.g., `Counter` -> `T`)
    generic_class_params: HashMap<String, Vec<String>>,
    /// Set of parameter names that are borrowed (&T) in the current function.
    /// Used to emit dereference (*name) in comparisons where &String != String.
    borrowed_params: HashSet<String>,
    /// Set of parameter names that are mutably borrowed (&mut T) in the current function.
    /// Used to avoid double-borrowing: when a &mut param is passed to another &mut param,
    /// we must NOT emit `&mut name` (it's already &mut T); just pass `name` directly.
    mut_borrowed_params: HashSet<String>,
    /// Map of `module_name` -> set of names that are intrinsic re-exports (from _sifr.*)
    /// Used to distinguish intrinsic function calls from pure Sifr function calls
    stdlib_intrinsic_names: HashMap<String, HashSet<String>>,
    /// Set of function names that are generators (contain yield statements)
    /// Used to emit .`collect()` when assigning generator results to list[T]
    generator_functions: HashSet<String>,
    /// Map of `module_name` -> set of imported names (for filtering preamble to only used functions)
    imported_stdlib_names: HashMap<String, HashSet<String>>,
    /// Number of upcoming `self.field` reads that should suppress auto-clone.
    /// This avoids temporal coupling from a sticky bool flag.
    pending_self_field_clone_suppression: usize,
    /// Whether we're inside a generator closure (yield -> return Some(val))
    in_generator_closure: bool,
    /// Whether we're inside a `Display::fmt` implementation (for __str__ methods)
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
    lowering_mode: CodegenLoweringMode,
    lowering_stats: LoweringStats,
}

impl RustEmitter {
    fn new() -> Self {
        Self::new_with_mode(CodegenLoweringMode::StructuredPreferred)
    }

    fn new_with_mode(lowering_mode: CodegenLoweringMode) -> Self {
        Self {
            output: String::new(),
            indent: 0,
            needs_hashmap: false,
            needs_hashset: false,
            needs_file_handles: false,
            needs_logging_state: false,
            needs_bigint: false,
            needs_vecdeque: false,
            union_enums: HashMap::new(),
            enum_defs: String::new(),
            current_return_type: None,
            option_unwrapped_vars: HashSet::new(),
            func_signatures: HashMap::new(),
            loop_else_stack: Vec::new(),
            mutated_vars: HashSet::new(),
            display_classes: HashSet::new(),
            parent_fields: HashMap::new(),
            current_class_name: None,
            used_stdlib_modules: HashSet::new(),
            intrinsic_functions: HashSet::new(),
            intrinsic_registry_crates: HashSet::new(),
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
            pending_self_field_clone_suppression: 0,
            in_generator_closure: false,
            in_display_impl: false,
            try_enum_counter: 0,
            try_closure_depth: 0,
            callable_var_conventions: HashMap::new(),
            lowering_mode,
            lowering_stats: LoweringStats::default(),
        }
    }

    fn structured_lowering_enabled(&self) -> bool {
        matches!(
            self.lowering_mode,
            CodegenLoweringMode::StructuredPreferred
        )
    }

    fn emit_module(&mut self, module: &HirModule, module_public: bool, test_mode: bool) {
        // Pre-scan: detect bigint usage
        if module_uses_bigint(module) {
            self.needs_bigint = true;
        }

        self.prescan_module_metadata(module);

        self.emit_module_constants(module);
        self.emit_module_body(module, module_public, test_mode);
    }

    fn emit_stmt(&mut self, stmt: &HirStmt) {
        self.lowering_stats.stmt_total += 1;
        if self.structured_lowering_enabled() {
            if is_simple_stmt_candidate(stmt) {
                self.lowering_stats.stmt_candidate_total += 1;
            }
            if let Some(lowered_stmts) = try_lower_simple_stmt_with_ctx(
                stmt,
                self.current_loop_has_else(),
                &self.mutated_vars,
                &self.borrowed_params,
                SimpleStmtLoweringCtx {
                    return_type: self.current_return_type.as_ref(),
                    in_display_impl: self.in_display_impl,
                    in_class_scope: self.current_class_name.is_some(),
                    in_generator_closure: self.in_generator_closure,
                },
            ) {
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                self.emit_lowered_stmts(&lowered_stmts);
                return;
            }

            if let Some(raw_bridge_stmts) = self.try_capture_legacy_stmt_as_raw(stmt) {
                self.lowering_stats.stmt_structured += 1;
                self.emit_lowered_stmts(&raw_bridge_stmts);
                return;
            }
        }

        match stmt {
            HirStmt::Let {
                name,
                ty,
                value,
                is_mutable: _,
            } => {
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
                        self.write(&format!("BigInt::from({v}_i64)"));
                    }
                } else if is_option_type(ty) && matches!(value, HirExpr::NoneLiteral) {
                    // `x: str | None = None` -> `let x: Option<String> = None`
                    self.write("None");
                } else if is_option_type(ty)
                    && !is_option_type(value.ty())
                    && !matches!(value.ty(), Type::None)
                {
                    // RHS is a plain value (not already Option) -> wrap in Some()
                    // But if RHS is a function call returning Option, don't double-wrap
                    self.write("Some(");
                    self.emit_expr(value);
                    self.write(")");
                } else {
                    // Check if RHS is a call to a generator function and target is list[T]
                    let needs_collect =
                        matches!(ty, Type::List(_)) && self.is_generator_call(value);
                    self.emit_expr(value);
                    if needs_collect {
                        self.write(".collect()");
                    }
                    // Clone borrowed TypeVar params assigned to owned TypeVar locals
                    let needs_clone_for_typevar = matches!(ty, Type::TypeVar(_))
                        && if let HirExpr::Name {
                            name: ref vname, ..
                        } = value
                        {
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
                    if let HirExpr::Name {
                        name: ref vname, ..
                    } = value
                    {
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
                        self.write(&format!(" {op} "));
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
                            self.write(&format!("{name}.pow("));
                            self.emit_expr(value);
                            self.write(" as u32);\n");
                        } else {
                            self.write(name);
                            self.write(" = ");
                            self.write(&format!("({name} as f64).powf("));
                            self.emit_expr(value);
                            self.write(" as f64);\n");
                        }
                    }
                    _ => {
                        self.write(name);
                        self.write(&format!(" {op} "));
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
                let ret_is_option = self
                    .current_return_type
                    .as_ref()
                    .is_some_and(is_option_type);
                let ret_is_non_option_union = self
                    .current_return_type
                    .as_ref()
                    .is_some_and(|t| matches!(t, Type::Union(_)) && !is_option_type(t));
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
                                    self.write(&format!("{enum_name}::{variant}("));
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
                    } else if !ret_is_option
                        && is_option_type(val.ty())
                        && !matches!(val.ty(), Type::None)
                    {
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
                if let Some((var_name, variant_name, enum_name, other_variants)) =
                    detect_isinstance_union(condition)
                {
                    self.write_indent();
                    self.write(&format!("match {var_name} {{\n"));
                    self.indent += 1;

                    // Then branch: the matched variant
                    let then_mutated = collect_mutated_vars(then_body);
                    let var_mut = if then_mutated.contains(&var_name) {
                        "mut "
                    } else {
                        ""
                    };
                    self.write_indent();
                    self.write(&format!(
                        "{enum_name}::{variant_name}({var_mut}{var_name}) => {{\n"
                    ));
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
                            let elif_var_mut = if elif_mutated.contains(&var_name) {
                                "mut "
                            } else {
                                ""
                            };
                            self.write_indent();
                            self.write(&format!(
                                "{enum_name}::{elif_variant}({elif_var_mut}{var_name}) => {{\n"
                            ));
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
                        let else_var_mut = if else_mutated.contains(&var_name) {
                            "mut "
                        } else {
                            ""
                        };
                        if remaining_variants.len() == 1 {
                            let (other_variant, _) = &remaining_variants[0];
                            self.write_indent();
                            self.write(&format!(
                                "{enum_name}::{other_variant}({else_var_mut}{var_name}) => {{\n"
                            ));
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
                    self.write(&format!("if let Some({var_name}) = {var_name} {{\n"));
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
                        self.write(&format!("if let Some({var_name}) = {var_name} {{\n"));
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
                    self.write(&format!("if let Some({var_name}) = {var_name} {{\n"));
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
                } else if let Some((var_name, enum_name, _non_none_variants)) =
                    detect_is_none_union_var(condition)
                {
                    // 3+ member union `is None` check: use match with None variant
                    self.write_indent();
                    self.write(&format!("match {var_name} {{\n"));
                    self.indent += 1;

                    // None arm -> then_body
                    self.write_indent();
                    self.write(&format!("{enum_name}::None(()) => {{\n"));
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
                    self.write(&format!("if {var_name}.is_none() {{\n"));
                    self.indent += 1;
                    let then_exits = codegen_body_always_exits(then_body);
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        // In the else branch of `if x is None`, x is not None
                        self.write_indent();
                        self.write(&format!(
                            "}} else if let Some({var_name}) = {var_name} {{\n"
                        ));
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
                        self.write(&format!("let {var_name} = {var_name}.unwrap();\n"));
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
            HirStmt::While {
                condition,
                body,
                else_body,
            } => {
                let has_else = else_body.is_some();
                if has_else {
                    self.writeln("let mut _broke = false;");
                }
                self.loop_else_stack.push(has_else);
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
                let popped = self.loop_else_stack.pop();
                debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
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
            HirStmt::For {
                target,
                iter,
                body,
                else_body,
                ..
            } => {
                let has_else = else_body.is_some();
                if has_else {
                    self.writeln("let mut _broke = false;");
                }
                self.loop_else_stack.push(has_else);
                self.write_indent();
                self.write("for ");
                // Handle tuple unpacking: "i,v" -> "(i, v)"
                if target.contains(',') {
                    let names: Vec<&str> = target.split(',').collect();
                    self.write("(");
                    for (i, name) in names.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
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
                let popped = self.loop_else_stack.pop();
                debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
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
                if self.current_loop_has_else() {
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
            HirStmt::StarUnpack {
                before,
                star,
                after,
                value,
            } => {
                // Emit: let _tmp = value.clone() to avoid moving;
                self.write_indent();
                self.write("let _star_tmp = ");
                self.emit_expr(value);
                self.write(".clone();\n");
                // Emit before vars
                for (i, (name, _ty)) in before.iter().enumerate() {
                    self.write_indent();
                    self.write(&format!("let {name} = _star_tmp[{i}].clone();\n"));
                }
                // Emit star var
                let (star_name, _star_ty) = star;
                if after.is_empty() {
                    self.write_indent();
                    self.write(&format!(
                        "let {} = _star_tmp[{}..].to_vec();\n",
                        star_name,
                        before.len()
                    ));
                } else {
                    self.write_indent();
                    self.write(&format!(
                        "let {} = _star_tmp[{}.._star_tmp.len() - {}].to_vec();\n",
                        star_name,
                        before.len(),
                        after.len()
                    ));
                }
                // Emit after vars
                for (i, (name, _ty)) in after.iter().enumerate() {
                    self.write_indent();
                    self.write(&format!(
                        "let {} = _star_tmp[_star_tmp.len() - {}].clone();\n",
                        name,
                        after.len() - i
                    ));
                }
            }
            HirStmt::TryExcept {
                body,
                handlers,
                body_error_types,
            } => {
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
                    h.error_type
                        .as_ref()
                        .is_some_and(|et| io_subclass_kind(et).is_some())
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
                    self.write(&format!("enum {enum_name} {{\n"));
                    self.indent += 1;
                    for et in &error_type_names {
                        self.write_indent();
                        self.write(&format!("{et}({et}),\n"));
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");

                    // Emit From impls for each error type
                    for et in &error_type_names {
                        self.write_indent();
                        self.write(&format!("impl From<{et}> for {enum_name} {{\n"));
                        self.indent += 1;
                        self.write_indent();
                        self.write(&format!(
                            "fn from(e: {et}) -> Self {{ {enum_name}::{et}(e) }}\n"
                        ));
                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                    }

                    // Emit try body as a closure
                    // Check if the try body contains a return statement with a value.
                    let body_has_return_multi = try_body_has_value_return(body);
                    let (closure_ok_type_multi, ok_arm_multi) = if body_has_return_multi {
                        let inner_ty = self
                            .current_return_type
                            .as_ref()
                            .and_then(|t| {
                                if let Type::Result(ok, _) = t {
                                    Some(ok.rust_type())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "_".to_string());
                        (
                            inner_ty,
                            "Ok(__try_ret) => { return Ok(__try_ret); }".to_string(),
                        )
                    } else {
                        ("()".to_string(), "Ok(()) => {}".to_string())
                    };

                    self.write_indent();
                    self.write(&format!(
                        "match (|| -> Result<{closure_ok_type_multi}, {enum_name}> {{\n"
                    ));
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
                    self.write(&format!("{ok_arm_multi}\n"));

                    // Emit match arms
                    for handler in handlers {
                        if let Some(ref et) = handler.error_type {
                            if et == "Error" {
                                // Catch-all: match on any remaining variant
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({var_name}) => {{\n"));
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
                                    "Err({enum_name}::IOError(ref {var_name})) if {var_name}.kind == \"{kind}\" => {{\n"
                                ));
                                // Clone the variable so handler body can use it as owned
                                if handler.name.is_some() {
                                    self.indent += 1;
                                    self.write_indent();
                                    self.write(&format!("let {var_name} = {var_name}.clone();\n"));
                                    self.indent -= 1;
                                }
                            } else if et == "IOError" && has_io_subclass_handler {
                                // IOError parent catch-all (when subclass handlers exist)
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!(
                                    "Err({enum_name}::IOError({var_name})) => {{\n"
                                ));
                            } else {
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({enum_name}::{et}({var_name})) => {{\n"));
                            }
                        } else {
                            // Bare except — catch-all
                            let var_name = handler.name.as_deref().unwrap_or("_e");
                            self.write_indent();
                            self.write(&format!("Err({var_name}) => {{\n"));
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
                        handlers
                            .first()
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
                        let inner_ty = self
                            .current_return_type
                            .as_ref()
                            .and_then(|t| {
                                if let Type::Result(ok, _) = t {
                                    Some(ok.rust_type())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "_".to_string());
                        (
                            inner_ty.clone(),
                            "Ok(__try_ret) => { return Ok(__try_ret); }".to_string(),
                        )
                    } else {
                        ("()".to_string(), "Ok(()) => {}".to_string())
                    };

                    self.write_indent();
                    self.write(&format!(
                        "match (|| -> Result<{closure_ok_type}, {error_rust_type}> {{\n"
                    ));
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
                    self.write(&format!("{ok_arm}\n"));

                    if has_io_subclass_handler && error_rust_type == "IOError" {
                        // IOError with subclass dispatch: use guard-based matching
                        for handler in handlers {
                            if let Some(ref et) = handler.error_type {
                                if et == "Error" || et == "IOError" {
                                    // Parent catch-all
                                    let var_name = handler.name.as_deref().unwrap_or("_e");
                                    self.write_indent();
                                    self.write(&format!("Err({var_name}) => {{\n"));
                                } else if let Some(kind) = io_subclass_kind(et) {
                                    // Subclass match with guard
                                    let var_name = handler.name.as_deref().unwrap_or("_e");
                                    self.write_indent();
                                    self.write(&format!(
                                        "Err(ref {var_name}) if {var_name}.kind == \"{kind}\" => {{\n"
                                    ));
                                    // Clone the variable so handler body can use it as owned
                                    if handler.name.is_some() {
                                        self.indent += 1;
                                        self.write_indent();
                                        self.write(&format!(
                                            "let {var_name} = {var_name}.clone();\n"
                                        ));
                                        self.indent -= 1;
                                    }
                                } else {
                                    let var_name = handler.name.as_deref().unwrap_or("_e");
                                    self.write_indent();
                                    self.write(&format!("Err({var_name}) => {{\n"));
                                }
                            } else {
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({var_name}) => {{\n"));
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
                                self.write(&format!("Err({name}) => {{\n"));
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
            HirStmt::FieldAssign {
                object,
                field,
                value,
            } => {
                self.write_indent();
                // Check if this is assigning to a parent field via inheritance
                if let Some(ref class_name) = self.current_class_name.clone() {
                    if let Some((parent_name, parent_field_names)) =
                        self.parent_fields.get(class_name).cloned()
                    {
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
            HirStmt::SubscriptAssign {
                object,
                index,
                value,
                object_ty,
            } => {
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
            HirStmt::NestedSubscriptAssign {
                object,
                outer_index,
                inner_index,
                value,
                object_ty: _,
            } => {
                self.write_indent();
                // matrix[i][j] = val -> bounds-checked nested assignment (safe no-op if out of bounds)
                self.write("{ let __oi = ");
                self.emit_expr(outer_index);
                self.write(" as usize; let __ii = ");
                self.emit_expr(inner_index);
                self.write(" as usize; if let Some(__row) = ");
                self.write(object);
                self.write(
                    ".get_mut(__oi) { if let Some(__elem) = __row.get_mut(__ii) { *__elem = ",
                );
                self.emit_expr(value);
                self.write("; } } }\n");
            }
            HirStmt::SubscriptAugAssign {
                object,
                index,
                op,
                value,
                object_ty: _,
            } => {
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
            HirStmt::AttributeAugAssign {
                object,
                field,
                op,
                value,
            } => {
                self.write_indent();
                self.write(object);
                self.write(".");
                self.write(field);
                self.write(&format!(" {op} "));
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::AttributeSubscriptAssign {
                object,
                field,
                index,
                value,
                field_ty,
            } => {
                self.write_indent();
                let field_access = format!("{object}.{field}");
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
                        let key_needs_clone =
                            matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_));
                        if key_needs_clone {
                            if let HirExpr::Name { name, .. } = index {
                                if self.borrowed_params.contains(name.as_str())
                                    || self.mut_borrowed_params.contains(name.as_str())
                                {
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
                    let ctx_name = format!("__ctx_{i}");
                    let guard_type = format!("__WithGuard{i}");
                    let guard_var = format!("__guard_{i}");
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
                        self.write(&format!("struct {guard_type} {{ ctx: {class_name} }}\n"));
                        self.write_indent();
                        self.write(&format!("impl Drop for {guard_type} {{\n"));
                        self.indent += 1;
                        self.write_indent();
                        self.write("fn drop(&mut self) { self.ctx.__exit__(); }\n");
                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                        // Create guard instance, moving ctx into it
                        self.write_indent();
                        self.write(&format!(
                            "let mut {guard_var} = {guard_type} {{ ctx: {ctx_name} }};\n"
                        ));
                        // Call __enter__() on guard's ctx and bind result to var
                        self.write_indent();
                        if stmts_reference_var(body, var)
                            || items.iter().any(|(v, _, _)| v != var && v.contains(var))
                        {
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
                let param_names: HashSet<String> =
                    func.params.iter().map(|p| p.name.clone()).collect();

                // Detect captured variables: variables referenced in body that are
                // not parameters and not defined locally in the body
                let referenced_with_types = collect_referenced_vars_with_types(&func.body);
                let locally_defined = collect_locally_defined_vars(&func.body);
                let captures: Vec<(String, Type)> = referenced_with_types
                    .into_iter()
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
                    self.nested_fn_captures
                        .insert(func.name.clone(), captures.clone());

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
            HirStmt::Match {
                subject,
                subject_ty,
                arms,
            } => {
                self.emit_match(subject, subject_ty, arms);
            }
        }
    }

    fn emit_expr(&mut self, expr: &HirExpr) {
        self.lowering_stats.expr_total += 1;
        if self.structured_lowering_enabled() {
            if is_leaf_expr_candidate(expr) {
                self.lowering_stats.expr_candidate_total += 1;
            }
            if let Some(lowered_expr) = try_lower_leaf_expr(expr) {
                self.lowering_stats.expr_structured += 1;
                self.lowering_stats.expr_candidate_structured += 1;
                self.write(&crate::render_expr(&lowered_expr));
                return;
            }
            if let Some(raw_bridge_expr) = self.try_capture_legacy_expr_as_raw(expr) {
                self.lowering_stats.expr_structured += 1;
                self.write(&crate::render_expr(&raw_bridge_expr));
                return;
            }
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
                self.write(&format!("{val:?}.to_string()"));
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
                if self.intrinsic_functions.contains(name.as_str()) || self.is_stdlib_constant(name)
                {
                    self.emit_stdlib_constant(name);
                } else if let Some((_ty, rust_name)) = self.module_constants.get(name).cloned() {
                    // Module-level constant
                    self.write(&rust_name);
                } else {
                    self.write(name);
                }
            }
            HirExpr::BinOp {
                left,
                op,
                right,
                ty,
            } => {
                // BigInt arithmetic: always clone operands to avoid move issues
                if left.ty() == &Type::BigInt && right.ty() == &Type::BigInt && op != "**" {
                    if op == "//" {
                        // BigInt floor division uses /
                        self.emit_expr_with_bigint_clone(left);
                        self.write(" / ");
                        self.emit_expr_with_bigint_clone(right);
                    } else {
                        self.emit_expr_with_bigint_clone(left);
                        self.write(&format!(" {op} "));
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
                        self.write(&format!("\"{format_str}\".to_string()"));
                    } else {
                        self.write(&format!("format!(\"{format_str}\""));
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
                    if matches!(left.as_ref(), HirExpr::BinOp { .. }) {
                        self.write("(");
                    }
                    self.emit_expr(left);
                    if matches!(left.as_ref(), HirExpr::BinOp { .. }) {
                        self.write(")");
                    }
                    self.write(" / ");
                    if matches!(right.as_ref(), HirExpr::BinOp { .. }) {
                        self.write("(");
                    }
                    self.emit_expr(right);
                    if matches!(right.as_ref(), HirExpr::BinOp { .. }) {
                        self.write(")");
                    }
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
                    self.write(&format!(" {op} "));
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
                    self.write(&format!(" {op} "));
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
                    // Also wrap if the expression is an IntLiteral (might be used with operators that need parens)
                    let needs_left_parens = matches!(left.as_ref(), HirExpr::BinOp { .. })
                        || matches!(left.as_ref(), HirExpr::IntLiteral { .. });
                    let needs_right_parens = matches!(right.as_ref(), HirExpr::BinOp { .. })
                        || matches!(right.as_ref(), HirExpr::IntLiteral { .. });
                    if needs_left_parens || needs_left_cast {
                        self.write("(");
                    }
                    self.emit_expr(left);
                    if needs_left_parens || needs_left_cast {
                        self.write(")");
                    }
                    if needs_left_cast {
                        self.write(" as f64");
                    }
                    self.write(&format!(" {op} "));
                    if needs_right_parens || needs_right_cast {
                        self.write("(");
                    }
                    self.emit_expr(right);
                    if needs_right_parens || needs_right_cast {
                        self.write(")");
                    }
                    if needs_right_cast {
                        self.write(" as f64");
                    }
                }
            }
            HirExpr::UnaryOp { op, operand, .. } => {
                if op == "not" {
                    // Collection truthiness: `not list_var` -> `list_var.is_empty()`
                    let is_collection = matches!(
                        operand.ty(),
                        Type::List(_)
                            | Type::Dict(_, _)
                            | Type::Set(_)
                            | Type::Tuple(_)
                            | Type::Str
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
            HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } => {
                // For single comparison
                if ops.len() == 1 {
                    let op = &ops[0];
                    // Handle `is None` / `is not None` for Option types
                    if (op == "is" || op == "is not")
                        && matches!(comparators[0], HirExpr::NoneLiteral)
                    {
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
                        if left_is_option
                            && !right_is_option
                            && !matches!(comparators[0], HirExpr::NoneLiteral)
                        {
                            self.emit_expr(left);
                            self.write(&format!(" {op} Some("));
                            self.emit_expr(&comparators[0]);
                            self.write(")");
                        } else if !left_is_option
                            && right_is_option
                            && !matches!(left.as_ref(), HirExpr::NoneLiteral)
                        {
                            self.write("Some(");
                            self.emit_expr(left);
                            self.write(")");
                            self.write(&format!(" {op} "));
                            self.emit_expr(&comparators[0]);
                        } else {
                            // Dereference borrowed params in comparisons to avoid &String == String
                            self.emit_expr_for_compare(left);
                            self.write(&format!(" {op} "));
                            self.emit_expr_for_compare(&comparators[0]);
                        }
                    }
                } else {
                    // Chained comparisons: a < b < c -> a < b && b < c
                    // Cast expressions need parentheses when followed by comparison operators
                    // to avoid Rust parsing `1 as i64 < x` as a generic argument
                    self.write("(");
                    self.emit_expr_with_parens_for_compare(left);
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
                        self.write(&format!(" {rust_op} "));
                    }
                    self.emit_expr(val);
                }
            }
            HirExpr::Call { func, args, .. } => {
                if func == "print" {
                    // Map print() to println!
                    if args.is_empty() {
                        self.write("println!()");
                    } else if matches!(args[0], HirExpr::NoneLiteral)
                        || matches!(args[0].ty(), Type::None)
                    {
                        // print(None) -> println!("None")
                        self.write("println!(\"None\")");
                    } else if let HirExpr::StringLiteral(val) = &args[0] {
                        // Inline string literal directly: println!("hello") instead of println!("{}", "hello")
                        // Escape backslashes and double quotes for valid Rust string
                        let escaped = val
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('{', "{{")
                            .replace('}', "}}");
                        self.write(&format!("println!(\"{escaped}\")"));
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
                    } else if matches!(
                        args[0].ty(),
                        Type::List(_) | Type::Dict(_, _) | Type::Tuple(_) | Type::Set(_)
                    ) {
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
                    }
                } else if func == "isinstance" {
                    // isinstance() is handled by narrowing at the HIR level.
                    // At codegen time, we emit `true` since the narrowing has
                    // already validated the types. In practice, isinstance checks
                    // appear in if-conditions and the narrowing determines which
                    // branch to take.
                    self.write("true");
                } else if func == "str" {
                    // str() conversion -> format!("{}", arg) or format!("{:?}", arg) for lists
                    if args.is_empty() {
                        self.write("String::new()");
                    } else {
                        if matches!(args[0].ty(), Type::List(_)) {
                            self.write("format!(\"{:?}\", ");
                        } else {
                            self.write("format!(\"{}\", ");
                        }
                        self.emit_display_expr(&args[0]);
                        self.write(")");
                    }
                } else if func == "pow" {
                    // pow(base, exp)
                    if args.len() == 2 {
                        if args[0].ty() == &Type::Int && args[1].ty() == &Type::Int {
                            // Wrap base in parens to handle cases like "(2 as i64).pow(...)"
                            self.write("(");
                            self.emit_expr(&args[0]);
                            self.write(").pow(");
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
                    } else if matches!(args[0].ty(), Type::List(ref e) if matches!(e.as_ref(), Type::Float))
                    {
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
                    } else if matches!(args[0].ty(), Type::List(ref e) if matches!(e.as_ref(), Type::Float))
                    {
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
                    let is_float_list =
                        matches!(args[0].ty(), Type::List(inner) if **inner == Type::Float);
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
                        let param_name = if params.is_empty() {
                            "x"
                        } else {
                            &params[0].name
                        };
                        // Use .clone().into_iter() for owned values, then filter with |&var| destructuring
                        self.write(&format!(".clone().into_iter().filter(|&{param_name}| "));
                        self.emit_expr(body);
                        self.write(").collect::<Vec<_>>()");
                    } else {
                        self.write(".clone().into_iter().filter(|x| (");
                        self.emit_lambda_untyped(&args[0]);
                        self.write(")(x)).collect::<Vec<_>>()");
                    }
                } else if self.intrinsic_functions.contains(func.as_str()) || func == "builtin_open"
                {
                    // Intrinsic function call — emit the correct Rust code
                    self.emit_intrinsic_call(func, args);
                } else {
                    self.write(func);
                    self.write("(");
                    // Look up param types and conventions to wrap union enum arguments.
                    // First check func_signatures (regular functions), then callable_var_conventions
                    // (Callable-typed parameters/locals whose conventions are tracked per-function).
                    let param_info: Option<Vec<(Type, ParamConvention)>> = self
                        .func_signatures
                        .get(func)
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
                                if is_option_type(param_ty)
                                    && !is_option_type(arg.ty())
                                    && !matches!(arg, HirExpr::NoneLiteral)
                                {
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
                                            self.emit_borrow_prefix(
                                                convention,
                                                param_ty,
                                                Some(param_ty),
                                            );
                                            self.write(&format!("{enum_name}::{variant}("));
                                            self.emit_expr(arg);
                                            self.write(")");
                                            continue;
                                        }
                                    }
                                }
                                // Protocol param with concrete class arg -> wrap in Box::new()
                                if matches!(param_ty, Type::Protocol { .. })
                                    && !matches!(arg.ty(), Type::Protocol { .. })
                                {
                                    self.write("Box::new(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                // Callable param with TypeVar params: wrap concrete function in
                                // adapter closure so Copy-type args get dereferenced to match the
                                // generic `impl Fn(&T) -> R` signature.
                                if let Type::Callable(
                                    callable_params,
                                    callable_convs,
                                    _callable_ret,
                                ) = param_ty
                                {
                                    let has_typevar_param = callable_params
                                        .iter()
                                        .any(|p| matches!(p, Type::TypeVar(_)));
                                    if has_typevar_param {
                                        if let HirExpr::Name {
                                            name: arg_func_name,
                                            ..
                                        } = arg
                                        {
                                            if let Some((concrete_params, _)) = self
                                                .func_signatures
                                                .get(arg_func_name.as_str())
                                                .cloned()
                                            {
                                                let needs_wrapper = callable_params.iter().zip(concrete_params.iter()).any(|(cp, (ct, _))| {
                                                    matches!(cp, Type::TypeVar(_)) && ct.ownership() == sifr_type_system::OwnershipKind::Copy
                                                });
                                                if needs_wrapper {
                                                    self.write("|");
                                                    for (pi, (cp, cc)) in callable_params
                                                        .iter()
                                                        .zip(callable_convs.iter())
                                                        .enumerate()
                                                    {
                                                        if pi > 0 {
                                                            self.write(", ");
                                                        }
                                                        let pname = format!("__a{pi}");
                                                        if matches!(cp, Type::TypeVar(_)) || (*cc == ParamConvention::Borrow && cp.ownership() == sifr_type_system::OwnershipKind::Move) {
                                                            self.write(&format!("{pname}: &_"));
                                                        } else {
                                                            self.write(&format!("{pname}: _"));
                                                        }
                                                    }
                                                    self.write("| ");
                                                    self.write(arg_func_name);
                                                    self.write("(");
                                                    for (pi, (cp, (ct, _))) in callable_params
                                                        .iter()
                                                        .zip(concrete_params.iter())
                                                        .enumerate()
                                                    {
                                                        if pi > 0 {
                                                            self.write(", ");
                                                        }
                                                        let pname = format!("__a{pi}");
                                                        if matches!(cp, Type::TypeVar(_)) && ct.ownership() == sifr_type_system::OwnershipKind::Copy {
                                                            self.write(&format!("*{pname}"));
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
                                // For borrowed generic params (&T), wrapping expressions
                                // avoids Rust precedence pitfalls like `&(x) as i64`.
                                // This includes literals which otherwise produce invalid code like `&3_i64`.
                                if convention == ParamConvention::Borrow
                                    && matches!(param_ty, Type::TypeVar(_))
                                {
                                    self.write("&(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                self.emit_borrow_prefix_for_name(
                                    convention,
                                    arg.ty(),
                                    Some(param_ty),
                                    arg_name_opt,
                                );
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
            HirExpr::RangeLiteral {
                start, end, step, ..
            } => {
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
                        let is_self_field = matches!(object.as_ref(), HirExpr::FieldAccess { object: inner, .. }
                            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"));
                        if is_self_field {
                            self.pending_self_field_clone_suppression += 1;
                        }
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
                                    self.write(&format!(".{resolved}"));
                                }
                            } else {
                                // Emit raw integer for tuple field access (e.g., .0 not .0_i64)
                                self.emit_expr(object);
                                self.write(&format!(".{val}"));
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
            HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } => {
                self.emit_method_call(object, method, args);
            }
            HirExpr::ContainsOp {
                element,
                collection,
                ..
            } => {
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
            HirExpr::Slice {
                object,
                start,
                stop,
                step,
                ty,
            } => {
                let obj_ty = object.ty();
                match obj_ty {
                    Type::Str => {
                        self.emit_string_slice(
                            object,
                            start.as_deref(),
                            stop.as_deref(),
                            step.as_deref(),
                        );
                    }
                    Type::Tuple(_) => {
                        // Compile-time tuple slicing: direct field access
                        if let Type::Tuple(result_elems) = ty {
                            let start_idx = start
                                .as_ref()
                                .and_then(|e| {
                                    if let HirExpr::IntLiteral(v) = e.as_ref() {
                                        Some(*v as usize)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0);
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
                        self.emit_list_slice(
                            object,
                            start.as_deref(),
                            stop.as_deref(),
                            step.as_deref(),
                        );
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
                let is_self_access =
                    matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self");
                let suppress_self_clone =
                    if is_self_access && self.pending_self_field_clone_suppression > 0 {
                        self.pending_self_field_clone_suppression -= 1;
                        true
                    } else {
                        false
                    };
                let needs_clone =
                    is_self_access && needs_clone_for_type(ty) && !suppress_self_clone;

                // Determine the class name for parent field resolution
                // Either from current_class_name (inside a method) or from the object's type
                let class_name_for_parent = if let Some(ref cn) = self.current_class_name {
                    if is_self_access {
                        Some(cn.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
                .or_else(|| {
                    // For external access like obj.field, check the object's type
                    if let Type::Class { name, .. } = object.ty() {
                        Some(name.clone())
                    } else {
                        None
                    }
                });

                // Check if this is accessing a parent field via inheritance
                if let Some(ref class_name) = class_name_for_parent {
                    if let Some((parent_name, parent_field_names)) =
                        self.parent_fields.get(class_name).cloned()
                    {
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
            HirExpr::ConstructorCall {
                class_name, args, ..
            } => {
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
                    if args.is_empty() {
                        self.write("String::new()");
                    } else {
                        self.emit_expr(&args[0]);
                        self.write(".to_string()");
                    }
                    self.write(&format!(", kind: \"{kind}\".to_string() }}"));
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
                    let is_recursive = field_names.as_ref().is_some_and(|names| {
                        names.get(i).is_some_and(|fname| {
                            self.recursive_fields
                                .contains(&(class_name.clone(), fname.clone()))
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
                            self.borrowed_params.contains(name)
                                && ty.ownership() != sifr_type_system::OwnershipKind::Copy
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
            HirExpr::SuperCall {
                parent_class,
                method,
                args,
                ..
            } => {
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
            HirExpr::ListComp {
                expr,
                generators,
                ty,
            } => {
                if generators.len() == 1 {
                    // Single generator: use functional style
                    let (ref var, ref iter_e, ref filter) = generators[0];
                    let is_range = matches!(iter_e.ty(), Type::Range);
                    let var_pattern = if var.contains(',') {
                        let names: Vec<&str> = var.split(',').collect();
                        format!("({})", names.join(", "))
                    } else {
                        var.clone()
                    };
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
                        } else {
                            is_range
                        };
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
                        } else {
                            var.clone()
                        };
                        let is_range = matches!(iter_e.ty(), Type::Range);
                        self.write("for ");
                        self.write(&var_pattern);
                        self.write(" in ");
                        if is_range {
                            self.write("(");
                            self.emit_expr(iter_e);
                            self.write(")");
                        } else {
                            self.emit_expr(iter_e);
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
                        if filter.is_some() {
                            self.write("} ");
                        }
                        self.write("} ");
                    }
                    self.write("_result }");
                }
            }
            HirExpr::SetComp {
                expr,
                generators,
                ty,
            } => {
                self.needs_hashset = true;
                if generators.len() == 1 {
                    let (ref var, ref iter_e, ref filter) = generators[0];
                    let is_range = matches!(iter_e.ty(), Type::Range);
                    let var_pattern = if var.contains(',') {
                        let names: Vec<&str> = var.split(',').collect();
                        format!("({})", names.join(", "))
                    } else {
                        var.clone()
                    };
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
                        self.emit_expr(iter_e);
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
                        if filter.is_some() {
                            self.write("} ");
                        }
                        self.write("} ");
                    }
                    self.write("_result }");
                }
            }
            HirExpr::DictComp {
                key_expr,
                val_expr,
                generators,
                ty,
            } => {
                self.needs_hashmap = true;
                if generators.len() == 1 {
                    let (ref var, ref iter_e, ref filter) = generators[0];
                    let is_range = matches!(iter_e.ty(), Type::Range);
                    let var_pattern = if var.contains(',') {
                        let names: Vec<&str> = var.split(',').collect();
                        format!("({})", names.join(", "))
                    } else {
                        var.clone()
                    };
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
                        self.write(&format!(
                            ".collect::<HashMap<{}, {}>>()",
                            k.rust_type(),
                            v.rust_type()
                        ));
                    } else {
                        self.write(".collect::<HashMap<_, _>>()");
                    }
                } else {
                    self.write("{ let mut _result = HashMap::new(); ");
                    for (var, iter_e, filter) in generators {
                        let var_pattern = if var.contains(',') {
                            let names: Vec<&str> = var.split(',').collect();
                            format!("({})", names.join(", "))
                        } else {
                            var.clone()
                        };
                        self.write("for ");
                        self.write(&var_pattern);
                        self.write(" in ");
                        self.emit_expr(iter_e);
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
                        if filter.is_some() {
                            self.write("} ");
                        }
                        self.write("} ");
                    }
                    self.write("_result }");
                }
            }
            HirExpr::GeneratorExpr {
                expr,
                var,
                iter,
                filter,
                ..
            } => {
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
            HirExpr::EnumVariant {
                enum_name, variant, ..
            } => {
                // Color.RED -> Color::RED
                self.write(enum_name);
                self.write("::");
                self.write(variant);
            }
        }
    }
}

pub fn body_contains_yield(stmts: &[HirStmt]) -> bool {
    helpers::body_contains_yield_inner(stmts)
}
