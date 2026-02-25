//! Sifr Code Generation
//!
//! Translates the typed HIR into Rust source code.

#![allow(dead_code)]

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
mod error_refs;
mod expr_emitter;
mod lower_item;
mod stmt_emitter;
pub use lower_item::*;
mod class_emitter;
mod class_method_emitter;
mod entrypoints;
mod expr_ref_emitter;
mod expr_render_helpers;
mod field_analysis_helpers;
mod function_emitter;
mod generic_bounds_helpers;
mod helpers;
mod intrinsic_method_emitters;
mod intrinsics;
mod ir_imports;
mod ir_optimize;
mod ir_validate;
mod match_emitter;
mod match_guard_helpers;
mod method_call_emitter;
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

use error_refs::collect_referenced_builtin_error_classes;
use helpers::{
    collect_mutated_vars_with_sigs, is_hashable_type_codegen, module_uses_bigint,
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

pub use entrypoints::{generate_rust, generate_rust_test, generate_rust_with_metadata};

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
    pub stmt_lowering_errors: u64,
    pub expr_total: u64,
    pub expr_structured: u64,
    pub expr_lowering_errors: u64,
    pub item_lowering_errors: u64,
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
    let mut emitter = RustEmitter::new();
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
                    stdlib_needs_file_handles |=
                        prepared.shared_needs.file_handles.needs_file_handles;
                    stdlib_provides_file_handle_struct |= prepared
                        .shared_needs
                        .file_handles
                        .provides_file_handle_struct;
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
    let needs_file_handles = emitter.runtime_needs.needs_file_handles || stdlib_needs_file_handles;
    let needs_logging = emitter.used_stdlib_modules.contains("sifr.logging")
        || emitter.used_stdlib_modules.contains("_sifr.logging")
        || emitter.runtime_needs.needs_logging_state;

    // Emit built-in error class struct definitions for referenced error types.
    let referenced_error_classes = collect_referenced_builtin_error_classes(
        module,
        &stdlib_preamble,
        &emitter.intrinsic_functions,
        needs_file_handles,
        BUILTIN_ERROR_CLASSES,
    );
    let user_defined_error_classes: HashSet<String> = module
        .classes
        .iter()
        .filter(|c| c.is_error_type)
        .map(|c| c.name.clone())
        .collect();
    let user_defined_file_handle_struct = module.classes.iter().any(|c| c.name == "FileHandle");
    let io_error_referenced = referenced_error_classes.contains("IOError")
        || IO_ERROR_SUBCLASSES
            .iter()
            .any(|subclass| referenced_error_classes.contains(*subclass))
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
        let is_referenced = referenced_error_classes.contains(error_name);
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

    let mut assembled_body_items: Vec<RustItem> = Vec::new();
    if !emitter.enum_items.is_empty() {
        assembled_body_items.extend(emitter.enum_items.clone());
    }
    if !preamble_items.is_empty() {
        assembled_body_items.extend(preamble_items.clone());
    }
    if !stdlib_preamble.is_empty() {
        assembled_body_items.push(RustItem::RawCode(stdlib_preamble.clone()));
    }
    if !emitter.output.is_empty() {
        assembled_body_items.push(RustItem::RawCode(emitter.output.clone()));
    }

    let body_import_needs = collect_import_needs_from_items(&assembled_body_items);
    let needs_hashmap = body_import_needs.collections.needs_hashmap;
    let needs_hashset = body_import_needs.collections.needs_hashset;
    let needs_vecdeque = body_import_needs.collections.needs_vecdeque;
    let needs_bigint = body_import_needs.runtime.needs_bigint;
    let needs_mutex = needs_file_handles || needs_logging || body_import_needs.runtime.needs_mutex;

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

    let mut file_items: Vec<RustItem> = Vec::new();
    file_items.extend(import_items);
    file_items.extend(assembled_body_items);
    remove_trivial_clones_in_items(&mut file_items);
    let file_issues = validate_items(&file_items);
    assert!(
        file_issues.is_empty(),
        "codegen IR validation failed (assembled file): {}",
        file_issues
            .iter()
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let rust_file = RustFile { items: file_items };
    let rust_source = Renderer::new().render_file(&rust_file);

    // Add transitive dependencies from stdlib modules
    let mut all_used_modules = emitter.used_stdlib_modules.clone();
    for module_name in &emitter.used_stdlib_modules {
        if let Some(deps) = stdlib_code.transitive_deps.get(module_name) {
            all_used_modules.extend(deps.iter().cloned());
        }
    }

    CodegenResult {
        rust_source,
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
        let mut module_import_items: Vec<RustItem> = Vec::new();

        // For non-main modules, add imports as `use` statements
        for import in &module.imports {
            for name in &import.names {
                // Check if this name has an alias
                if let Some((_, alias)) = import.aliases.iter().find(|(orig, _)| orig == name) {
                    module_import_items.push(RustItem::UseAlias {
                        path: vec!["crate".to_string(), import.module.clone(), name.clone()],
                        alias: alias.clone(),
                    });
                } else {
                    module_import_items.push(RustItem::Use(vec![
                        "crate".to_string(),
                        import.module.clone(),
                        name.clone(),
                    ]));
                }
            }
        }

        let mut assembled_items: Vec<RustItem> = Vec::new();
        assembled_items.extend(module_import_items);
        if !emitter.enum_items.is_empty() {
            assembled_items.extend(emitter.enum_items.clone());
        }
        if !emitter.output.is_empty() {
            assembled_items.push(RustItem::RawCode(emitter.output.clone()));
        }
        let import_needs = collect_import_needs_from_items(&assembled_items);

        let mut import_items: Vec<RustItem> = Vec::new();
        if import_needs.collections.needs_hashmap {
            import_items.push(RustItem::Use(vec![
                "std".to_string(),
                "collections".to_string(),
                "HashMap".to_string(),
            ]));
        }
        if import_needs.collections.needs_hashset {
            import_items.push(RustItem::Use(vec![
                "std".to_string(),
                "collections".to_string(),
                "HashSet".to_string(),
            ]));
        }
        if import_needs.collections.needs_vecdeque {
            import_items.push(RustItem::Use(vec![
                "std".to_string(),
                "collections".to_string(),
                "VecDeque".to_string(),
            ]));
        }
        if import_needs.runtime.needs_bigint {
            import_items.push(RustItem::Use(vec![
                "num_bigint".to_string(),
                "BigInt".to_string(),
            ]));
        }

        let mut file_items: Vec<RustItem> = Vec::new();
        file_items.extend(import_items);
        file_items.extend(assembled_items);
        remove_trivial_clones_in_items(&mut file_items);
        let file_issues = validate_items(&file_items);
        assert!(
            file_issues.is_empty(),
            "codegen IR validation failed (multi module file `{}`): {}",
            module_name,
            file_issues
                .iter()
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join(" | ")
        );
        let rust_file = RustFile { items: file_items };
        let result = Renderer::new().render_file(&rust_file);

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
    collection_needs: CollectionNeeds,
    runtime_needs: RuntimeNeeds,
    /// Track union enum types that need to be defined (name -> member types)
    union_enums: HashMap<String, Vec<Type>>,
    /// Accumulated union enum items to prepend
    enum_items: Vec<RustItem>,
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
    emission_ctx: EmissionContext,
    /// Whether we're inside a `Display::fmt` implementation (for __str__ methods)
    /// Return statements in this context become write!(f, "{}", val) + return Ok(())
    /// Counter for generating unique try-block error enum names
    try_enum_counter: usize,
    /// Depth of try-block closures we're currently inside (for return statement handling)
    try_closure_depth: usize,
    /// Map from variable name -> Callable parameter (type, convention) list.
    /// Populated per-function from params and locals with Callable types.
    /// Used to emit correct &arg/&mut arg/arg for Callable-typed variable calls.
    callable_var_conventions: HashMap<String, Vec<(Type, ParamConvention)>>,
    /// Fallback recursion guard for non-structured emitter paths.
    lowering_stats: LoweringStats,
}

#[derive(Default)]
struct CollectionNeeds {
    needs_hashmap: bool,
    needs_hashset: bool,
    needs_vecdeque: bool,
}

#[derive(Default)]
struct RuntimeNeeds {
    needs_file_handles: bool,
    needs_logging_state: bool,
    needs_bigint: bool,
}

#[derive(Default)]
struct EmissionContext {
    in_generator_closure: bool,
    in_display_impl: bool,
}

impl RustEmitter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            collection_needs: CollectionNeeds::default(),
            runtime_needs: RuntimeNeeds::default(),
            union_enums: HashMap::new(),
            enum_items: Vec::new(),
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
            emission_ctx: EmissionContext::default(),
            try_enum_counter: 0,
            try_closure_depth: 0,
            callable_var_conventions: HashMap::new(),
            lowering_stats: LoweringStats::default(),
        }
    }

    fn emit_module(&mut self, module: &HirModule, module_public: bool, test_mode: bool) {
        // Pre-scan: detect bigint usage
        if module_uses_bigint(module) {
            self.runtime_needs.needs_bigint = true;
        }

        self.prescan_module_metadata(module);

        self.emit_module_constants(module);
        self.emit_module_body(module, module_public, test_mode);
    }

    fn try_emit_structured_stmt(&mut self, stmt: &HirStmt) -> Result<bool, crate::CodegenError> {
        let scope_ctx = ScopeContext {
            function_return_type: self.current_return_type.clone(),
            in_generator_closure: self.emission_ctx.in_generator_closure,
            in_display_impl: self.emission_ctx.in_display_impl,
            in_loop_with_else: self.current_loop_has_else(),
            class_scope: if self.current_class_name.is_some() {
                ClassScope::Inside
            } else {
                ClassScope::Outside
            },
        };

        if let Some(lowered_stmts) = try_lower_simple_stmt_with_scope_result(
            stmt,
            &self.mutated_vars,
            &self.borrowed_params,
            &scope_ctx,
        )? {
            let rewritten_stmts = lowered_stmts
                .into_iter()
                .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                .collect::<Vec<_>>();
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            self.emit_lowered_stmts(&rewritten_stmts);
            return Ok(true);
        }

        if let HirStmt::Let {
            name, ty, value, ..
        } = stmt
        {
            if is_copyish_structured_stmt_expr_type(ty)
                && is_copyish_structured_stmt_expr_type(value.ty())
            {
                let output_len = self.output.len();
                self.write("let ");
                if self.mutated_vars.contains(name) {
                    self.write("mut ");
                }
                self.write(name);
                self.write(": ");
                self.write(&crate::render_type(&sifr_type_to_rust_type(ty)));
                self.write(" = ");
                if self.try_emit_structured_expr(value)? {
                    self.lowering_stats.stmt_structured += 1;
                    self.lowering_stats.stmt_candidate_structured += 1;
                    self.write(";\n");
                    return Ok(true);
                }
                self.output.truncate(output_len);
            }
        }

        if let HirStmt::Assign { name, value } = stmt {
            if is_copyish_structured_stmt_expr_type(value.ty()) {
                let output_len = self.output.len();
                self.write(name);
                self.write(" = ");
                if self.try_emit_structured_expr(value)? {
                    self.lowering_stats.stmt_structured += 1;
                    self.lowering_stats.stmt_candidate_structured += 1;
                    self.write(";\n");
                    return Ok(true);
                }
                self.output.truncate(output_len);
            }
        }

        if let HirStmt::Return { value: Some(value) } = stmt {
            if !self.emission_ctx.in_display_impl
                && !self.emission_ctx.in_generator_closure
                && self
                    .current_return_type
                    .as_ref()
                    .is_some_and(is_copyish_structured_stmt_expr_type)
                && is_copyish_structured_stmt_expr_type(value.ty())
            {
                let output_len = self.output.len();
                self.write("return ");
                if self.try_emit_structured_expr(value)? {
                    self.lowering_stats.stmt_structured += 1;
                    self.lowering_stats.stmt_candidate_structured += 1;
                    self.write(";\n");
                    return Ok(true);
                }
                self.output.truncate(output_len);
            }
        }

        if let HirStmt::Expr { expr } = stmt {
            let output_len = self.output.len();
            if self.try_emit_structured_expr(expr)? {
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                self.write(";\n");
                return Ok(true);
            }
            self.output.truncate(output_len);
        }

        Ok(false)
    }

    fn try_emit_structured_expr(&mut self, expr: &HirExpr) -> Result<bool, crate::CodegenError> {
        if matches!(expr, HirExpr::Compare { .. } | HirExpr::BoolOp { .. })
            && expr_uses_borrowed_param(expr, &self.borrowed_params, &self.mut_borrowed_params)
        {
            return Ok(false);
        }
        if let HirExpr::Call { func, args, .. } = expr {
            if (self.intrinsic_functions.contains(func.as_str()) || func == "builtin_open")
                && self.try_emit_intrinsic_via_registry(func, args)
            {
                self.lowering_stats.expr_structured += 1;
                return Ok(true);
            }
        }
        if let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expr
        {
            if !is_self_field_access_expr(object)
                && self.try_emit_method_via_registry(object.ty(), object, method, args)
            {
                self.lowering_stats.expr_structured += 1;
                return Ok(true);
            }
        }
        if let Some(lowered_expr) = try_lower_leaf_expr_result(expr)? {
            let rewritten_expr = self.rewrite_stdlib_constant_idents_in_expr(lowered_expr);
            self.lowering_stats.expr_structured += 1;
            self.lowering_stats.expr_candidate_structured += 1;
            self.write(&crate::render_expr(&rewritten_expr));
            return Ok(true);
        }

        Ok(false)
    }

    fn emit_stmt(&mut self, stmt: &HirStmt) {
        self.lowering_stats.stmt_total += 1;
        if is_simple_stmt_candidate(stmt) {
            self.lowering_stats.stmt_candidate_total += 1;
        }
        match self.try_emit_structured_stmt(stmt) {
            Ok(true) => return,
            Ok(false) => {}
            Err(_) => {
                self.lowering_stats.stmt_lowering_errors += 1;
                self.emit_stmt_fallback(stmt);
                return;
            }
        }
        self.emit_stmt_fallback(stmt);
    }

    fn emit_expr(&mut self, expr: &HirExpr) {
        self.lowering_stats.expr_total += 1;
        if is_leaf_expr_candidate(expr) {
            self.lowering_stats.expr_candidate_total += 1;
        }
        match self.try_emit_structured_expr(expr) {
            Ok(true) => return,
            Ok(false) => {}
            Err(_) => {
                self.lowering_stats.expr_lowering_errors += 1;
                self.emit_expr_fallback(expr);
                return;
            }
        }
        self.emit_expr_fallback(expr);
    }
}

pub fn body_contains_yield(stmts: &[HirStmt]) -> bool {
    helpers::body_contains_yield_inner(stmts)
}

fn is_self_field_access_expr(expr: &HirExpr) -> bool {
    if let HirExpr::FieldAccess { object, .. } = expr {
        return matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self");
    }
    false
}

fn is_copyish_structured_stmt_expr_type(ty: &Type) -> bool {
    match ty {
        Type::Alias(_, inner) => is_copyish_structured_stmt_expr_type(inner),
        Type::Int | Type::Float | Type::Bool | Type::Enum { .. } => true,
        _ => false,
    }
}

fn expr_uses_borrowed_param(
    expr: &HirExpr,
    borrowed_params: &HashSet<String>,
    mut_borrowed_params: &HashSet<String>,
) -> bool {
    match expr {
        HirExpr::Name { name, .. } => {
            borrowed_params.contains(name) || mut_borrowed_params.contains(name)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            expr_uses_borrowed_param(left, borrowed_params, mut_borrowed_params)
                || comparators
                    .iter()
                    .any(|c| expr_uses_borrowed_param(c, borrowed_params, mut_borrowed_params))
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .any(|v| expr_uses_borrowed_param(v, borrowed_params, mut_borrowed_params)),
        HirExpr::UnaryOp { operand, .. } => {
            expr_uses_borrowed_param(operand, borrowed_params, mut_borrowed_params)
        }
        HirExpr::BinOp { left, right, .. } => {
            expr_uses_borrowed_param(left, borrowed_params, mut_borrowed_params)
                || expr_uses_borrowed_param(right, borrowed_params, mut_borrowed_params)
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            expr_uses_borrowed_param(condition, borrowed_params, mut_borrowed_params)
                || expr_uses_borrowed_param(then_expr, borrowed_params, mut_borrowed_params)
                || expr_uses_borrowed_param(else_expr, borrowed_params, mut_borrowed_params)
        }
        _ => false,
    }
}
