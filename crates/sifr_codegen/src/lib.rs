//! Sifr Code Generation: translates typed HIR into Rust source code.
#![allow(dead_code)]
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]
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
mod lower_item;
pub use lower_item::*;
mod class_emitter;
mod class_method_emitter;
mod entrypoints;
mod expr_ref_emitter;
mod expr_render_helpers;
mod field_analysis_helpers;
mod function_emitter;
mod function_like_lowering;
mod generic_bounds_helpers;
mod helpers;
mod hir_analysis;
mod intrinsic_method_emitters;
mod intrinsics;
mod ir_imports;
mod ir_optimize;
mod ir_validate;
mod lib_support;
mod match_guard_helpers;
mod method_call_emitter;
mod methods;
mod module_body;
mod module_constants;
mod module_prescan;
mod operator_protocol_emitters;
mod output_helpers;
mod stdlib_filter;
mod stmt_support_emitter;
mod type_emitters;
mod union_type_helpers;

#[cfg(test)]
mod lib_codegen_tests;

use error_refs::collect_referenced_builtin_error_classes;
use helpers::{
    collect_locally_defined_vars, collect_mutated_vars_with_sigs,
    collect_referenced_vars_with_types, default_param_convention, is_hashable_type_codegen,
    module_uses_bigint,
};
use hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use ir_imports::{collect_import_needs_from_items, collect_import_needs_from_source};
use ir_optimize::remove_trivial_clones_in_items;
use ir_validate::validate_items;
pub(crate) use lib_support::{
    resolve_alias_type_for_plain_call, try_lower_leaf_or_name_expr_result,
};
use sifr_hir::{HirExpr, HirFunction, HirModule, HirStmt};
use sifr_type_system::{ParamConvention, Type};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::env;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use stdlib_filter::{
    collect_and_strip_shared_prelude, dedup_rust_items, filter_stdlib_ir_to_needed,
    strip_rust_items_by_name,
};

type FuncSignature = (Vec<(Type, ParamConvention)>, Type);
type ModuleFuncSignatures = HashMap<String, FuncSignature>;
type StdlibFuncSignatures = HashMap<String, ModuleFuncSignatures>;
type UnionVariantTypes = Vec<(String, Type)>;
type IsinstanceUnionMatch = (String, String, String, UnionVariantTypes);
type IsNoneUnionMatch = (String, String, UnionVariantTypes);

pub use entrypoints::{generate_rust, generate_rust_test, generate_rust_with_metadata};

pub fn sifr_runtime_dependency_spec() -> String {
    let runtime_path = discover_sifr_runtime_path().unwrap_or_else(compile_time_sifr_runtime_path);
    let escaped_path = runtime_path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!("sifr_runtime = {{ path = \"{escaped_path}\" }}")
}

fn tokio_dependency_spec() -> String {
    "tokio = { version = \"1.52.3\", features = [\"macros\", \"rt\", \"sync\", \"time\"] }"
        .to_string()
}

fn discover_sifr_runtime_path() -> Option<PathBuf> {
    env::var_os("SIFR_RUNTIME_PATH")
        .map(PathBuf::from)
        .filter(|path| path.join("Cargo.toml").is_file())
        .or_else(discover_sifr_runtime_path_from_current_dir)
        .or_else(discover_sifr_runtime_path_from_current_exe)
}

fn discover_sifr_runtime_path_from_current_dir() -> Option<PathBuf> {
    env::current_dir()
        .ok()
        .and_then(|path| discover_sifr_runtime_path_from_ancestors(&path))
}

fn discover_sifr_runtime_path_from_current_exe() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|path| discover_sifr_runtime_path_from_ancestors(&path))
}

fn discover_sifr_runtime_path_from_ancestors(start: &Path) -> Option<PathBuf> {
    start.ancestors().find_map(|ancestor| {
        let candidate = ancestor.join("crates").join("sifr_runtime");
        candidate.join("Cargo.toml").is_file().then_some(candidate)
    })
}

fn compile_time_sifr_runtime_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .join("sifr_runtime")
}

#[derive(Clone)]
struct NestedFnCapture {
    name: String,
    ty: Type,
    convention: ParamConvention,
}

/// Built-in error class names that the compiler provides.
const BUILTIN_ERROR_CLASSES: &[&str] = &[
    "Error",
    "IOError",
    "ParseError",
    "ValueError",
    "DivisionError",
    "KeyError",
    "JSONDecodeError",
    "JsonIntegerRangeError",
    "JsonLimitError",
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
    "DecimalConversionError",
    "TimeoutError",
    "ScopeFailure",
    "TaskCancelled",
    "SecondaryError",
    "GeneratorCloseError",
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

/// Result of multi-module code generation, including aggregate dependency metadata.
pub struct MultiModuleCodegenResult {
    pub rust_files: HashMap<String, String>,
    pub used_stdlib_modules: HashSet<String>,
    pub required_crates: HashSet<String>,
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
#[derive(Clone, Default)]
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
    /// Map of stdlib generic class name -> declared type parameter names.
    pub generic_class_params: HashMap<String, Vec<String>>,
    /// Map of stdlib generic class name -> template HIR class for concrete type-argument inference.
    pub generic_class_templates: HashMap<String, sifr_hir::HirClass>,
    /// Map of `module_name` -> (`class_name` -> ordered class fields).
    /// Multi-module project codegen also uses this for local helper modules.
    pub module_class_fields: HashMap<String, HashMap<String, Vec<(String, Type)>>>,
}

fn module_func_signatures(module: &HirModule) -> ModuleFuncSignatures {
    let mut sig_map = HashMap::new();
    for func in &module.functions {
        let params = func
            .params
            .iter()
            .map(|param| (param.ty.clone(), param.convention))
            .collect::<Vec<_>>();
        sig_map.insert(func.name.clone(), (params, func.return_type.clone()));
    }
    for class in &module.classes {
        let mut has_constructor = false;
        for method in &class.methods {
            let params = method
                .params
                .iter()
                .map(|param| {
                    let convention = if method.name == "new" {
                        ParamConvention::own()
                    } else {
                        param.convention
                    };
                    (param.ty.clone(), convention)
                })
                .collect::<Vec<_>>();
            sig_map.insert(
                format!("{}::{}", class.name, method.name),
                (params, method.return_type.clone()),
            );
            if method.name == "new" {
                has_constructor = true;
            }
        }
        if !has_constructor {
            let ctor_params = class
                .fields
                .iter()
                .map(|(_, ty)| (ty.clone(), ParamConvention::own()))
                .collect::<Vec<_>>();
            sig_map.insert(
                format!("{}::new", class.name),
                (
                    ctor_params,
                    Type::Class {
                        name: class.name.clone(),
                        fields: class.fields.clone(),
                        methods: Vec::new(),
                        parent_class: class.parent_class.clone(),
                    },
                ),
            );
        }
    }
    sig_map
}

fn module_class_fields(module: &HirModule) -> HashMap<String, Vec<(String, Type)>> {
    module
        .classes
        .iter()
        .map(|class| (class.name.clone(), class.fields.clone()))
        .collect()
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
    emitter
        .generic_class_params
        .extend(stdlib_code.generic_class_params.clone());
    emitter
        .generic_class_templates
        .extend(stdlib_code.generic_class_templates.clone());

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
        if let Some(class_fields) = stdlib_code.module_class_fields.get(&import.module) {
            for name in &import.names {
                if let Some(fields) = class_fields.get(name) {
                    let local_name = import
                        .aliases
                        .iter()
                        .find(|(original, _)| original == name)
                        .map(|(_, alias)| alias.as_str())
                        .unwrap_or(name);
                    emitter.register_external_class_fields(local_name, name, fields);
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
    for module_name in emitter.used_stdlib_modules.iter().collect::<BTreeSet<_>>() {
        if let Some(deps) = stdlib_code.transitive_deps.get(module_name) {
            for dep in deps.iter().collect::<BTreeSet<_>>() {
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
                let mut filtered =
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
                if module_name == "sifr.sync" && sync_channel_runtime_needed(&filtered) {
                    filtered = replace_sync_channel_runtime_items(&filtered);
                }
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
    let needs_file_handles = emitter.runtime_needs.file_handles() || stdlib_needs_file_handles;
    let needs_logging = emitter.used_stdlib_modules.contains("sifr.logging")
        || emitter.used_stdlib_modules.contains("_sifr.logging")
        || emitter.runtime_needs.logging_state();
    let needs_random_module_state = emitter.runtime_needs.random_module_state();

    // Emit built-in error class struct definitions for referenced error types.
    let uses_task_scope = module_uses_task_scope(module);
    let uses_failure_type = module_uses_failure_type(module);
    let uses_cancellation_error_type = module_uses_cancellation_error_type(module);
    let uses_async_exit_cause_type = module_uses_async_exit_cause_type(module);
    let uses_timeout_result_type = module_uses_timeout_result_type(module);
    let uses_async_generator_type = module_uses_async_generator_type(module);
    let mut referenced_error_classes = collect_referenced_builtin_error_classes(
        module,
        &stdlib_preamble,
        &emitter.intrinsic_functions,
        needs_file_handles,
        BUILTIN_ERROR_CLASSES,
    );
    if uses_task_scope || uses_failure_type {
        referenced_error_classes.insert("SecondaryError".to_string());
    }
    if uses_async_generator_type {
        referenced_error_classes.insert("GeneratorCloseError".to_string());
    }
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
                } else if error_name == "JsonIntegerRangeError" {
                    (
                        vec![
                            ("path".to_string(), sifr_type_to_rust_type(&Type::Str)),
                            ("profile".to_string(), sifr_type_to_rust_type(&Type::Str)),
                        ],
                        vec![
                            (
                                "path".to_string(),
                                RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "String".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            ),
                            (
                                "profile".to_string(),
                                RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "String".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            ),
                        ],
                    )
                } else if error_name == "JsonLimitError" {
                    (
                        vec![("limit".to_string(), sifr_type_to_rust_type(&Type::Int))],
                        vec![("limit".to_string(), RustExpr::Literal(RustLiteral::Int(0)))],
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
    if referenced_error_classes.contains("Error") && !user_defined_error_classes.contains("Error") {
        for &error_name in BUILTIN_ERROR_CLASSES {
            if error_name == "Error" || IO_ERROR_SUBCLASSES.contains(&error_name) {
                continue;
            }
            if referenced_error_classes.contains(error_name)
                && !user_defined_error_classes.contains(error_name)
            {
                preamble_items.push(build_error_into_error_impl(error_name));
            }
        }
    }
    if uses_task_scope || uses_failure_type {
        preamble_items.extend(build_failure_type_items());
    }
    if uses_task_scope || uses_cancellation_error_type {
        preamble_items.extend(build_cancellation_error_type_items());
    }
    if uses_async_exit_cause_type {
        preamble_items.extend(build_async_exit_cause_type_items());
    }
    if uses_timeout_result_type && !uses_task_scope {
        preamble_items.extend(build_timeout_result_type_items());
    }
    if uses_async_generator_type {
        preamble_items.extend(build_async_generator_type_items());
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
    if needs_random_module_state {
        preamble_items.extend(build_random_module_state_items());
    }
    if uses_task_scope {
        preamble_items.extend(build_task_scope_items());
    }

    let mut assembled_body_items: Vec<RustItem> = Vec::new();
    if !emitter.enum_items.is_empty() {
        assembled_body_items.extend(emitter.enum_items.clone());
    }
    if !preamble_items.is_empty() {
        assembled_body_items.extend(preamble_items.clone());
    }
    if !emitter.body_items.is_empty() {
        assembled_body_items.extend(emitter.body_items.clone());
    }
    remove_trivial_clones_in_items(&mut assembled_body_items);
    let has_async_main_entrypoint = annotate_async_main_entrypoint(&mut assembled_body_items);
    let uses_task_sleep = module_uses_task_sleep(module);
    let body_import_needs = collect_import_needs_from_items(&assembled_body_items);
    let stdlib_import_needs = collect_import_needs_from_source(&stdlib_preamble);
    let needs_hashmap = body_import_needs.collections.needs_hashmap
        || stdlib_import_needs.collections.needs_hashmap;
    let needs_hashset = body_import_needs.collections.needs_hashset
        || stdlib_import_needs.collections.needs_hashset;
    let needs_vecdeque = body_import_needs.collections.needs_vecdeque
        || stdlib_import_needs.collections.needs_vecdeque;
    let needs_bigint = body_import_needs.runtime.numeric.needs_bigint
        || stdlib_import_needs.runtime.numeric.needs_bigint;
    let needs_decimal = body_import_needs.runtime.numeric.needs_decimal
        || stdlib_import_needs.runtime.numeric.needs_decimal;
    let needs_bigdecimal = body_import_needs.runtime.numeric.needs_bigdecimal
        || stdlib_import_needs.runtime.numeric.needs_bigdecimal;
    let needs_sifr_int =
        body_import_needs.runtime.needs_sifr_int || stdlib_import_needs.runtime.needs_sifr_int;
    let needs_mutex = needs_file_handles
        || needs_logging
        || needs_random_module_state
        || body_import_needs.runtime.needs_mutex
        || stdlib_import_needs.runtime.needs_mutex;

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
    if needs_decimal {
        import_items.push(RustItem::Use(vec![
            "rust_decimal".to_string(),
            "Decimal".to_string(),
        ]));
    }
    if needs_bigdecimal {
        import_items.push(RustItem::Use(vec![
            "bigdecimal".to_string(),
            "BigDecimal".to_string(),
        ]));
    }
    if needs_sifr_int {
        import_items.push(RustItem::Use(vec![
            "sifr_runtime".to_string(),
            "SifrInt".to_string(),
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
    file_items.extend(import_items.clone());
    file_items.extend(assembled_body_items.clone());
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
    let rust_source = if stdlib_preamble.trim().is_empty() {
        let rust_file = RustFile { items: file_items };
        Renderer::new().render_file(&rust_file)
    } else {
        assert!(
            syn::parse_file(&stdlib_preamble).is_ok(),
            "failed to parse stdlib preamble boundary as Rust source"
        );
        let mut source = String::new();
        if !import_items.is_empty() {
            let import_source = Renderer::new().render_file(&RustFile {
                items: import_items.clone(),
            });
            source.push_str(import_source.trim_end());
            source.push_str("\n\n");
        }
        source.push_str(stdlib_preamble.trim_end());
        if !assembled_body_items.is_empty() {
            let body_source = Renderer::new().render_file(&RustFile {
                items: assembled_body_items.clone(),
            });
            if !body_source.trim().is_empty() {
                source.push_str("\n\n");
                source.push_str(body_source.trim_end());
            }
        }
        source.push('\n');
        source
    };

    // Add transitive dependencies from stdlib modules
    let mut all_used_modules = emitter.used_stdlib_modules.clone();
    for module_name in emitter.used_stdlib_modules.iter().collect::<BTreeSet<_>>() {
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
            if needs_decimal {
                crates.insert("rust_decimal".to_string());
            }
            if needs_bigdecimal {
                crates.insert("bigdecimal".to_string());
            }
            if needs_sifr_int {
                crates.insert("sifr_runtime".to_string());
            }
            if has_async_main_entrypoint
                || uses_task_sleep
                || module_uses_task_scope(module)
                || stdlib_preamble.contains("tokio::")
            {
                crates.insert("tokio".to_string());
            }
            crates
        },
        constant_mappings: emitter.module_constants,
        lowering_stats: emitter.lowering_stats,
    }
}

fn sync_channel_runtime_needed(rust_code: &str) -> bool {
    rust_code.contains("struct Channel<")
        || rust_code.contains("struct ChannelSender<")
        || rust_code.contains("struct ChannelReceiver<")
        || rust_code.contains("fn channel<")
        || rust_code.contains("fn bounded_channel<")
}

fn replace_sync_channel_runtime_items(rust_code: &str) -> String {
    let strip_names = HashSet::from([
        "Channel",
        "ChannelSender",
        "ChannelReceiver",
        "channel",
        "bounded_channel",
    ]);
    let mut replaced = strip_rust_items_by_name(rust_code, &strip_names);
    if !replaced.trim().is_empty() {
        replaced.push('\n');
    }
    replaced.push_str(sync_channel_runtime_rust_code());
    replaced
}

fn sync_channel_runtime_rust_code() -> &'static str {
    r#"
#[derive(Debug)]
struct __SifrChannelState<T> {
    buffer: std::collections::VecDeque<T>,
    closed: bool,
    capacity: i64,
    sender_count: i64,
    receiver_alive: bool,
}

enum __SifrChannelPushState {
    Sent,
    Closed,
    Full,
}

enum __SifrChannelPopState<T> {
    Item(T),
    Empty,
    Closed,
}

#[derive(Debug)]
struct Channel<T: Clone> {
    _state: std::sync::Arc<std::sync::Mutex<__SifrChannelState<T>>>,
}

impl<T: Clone> Clone for Channel<T> {
    fn clone(&self) -> Self {
        return Self {
            _state: std::sync::Arc::clone(&self._state),
        };
    }
}

impl<T: Clone> Channel<T> {
    fn new(buffer: Vec<T>, capacity: i64) -> Self {
        return Self {
            _state: std::sync::Arc::new(std::sync::Mutex::new(__SifrChannelState {
                buffer: buffer.into_iter().collect(),
                closed: false,
                capacity,
                sender_count: 0,
                receiver_alive: true,
            })),
        };
    }

    fn with_state<R>(&self, f: impl FnOnce(&mut __SifrChannelState<T>) -> R) -> R {
        match self._state.lock() {
            Ok(mut state) => f(&mut state),
            Err(poisoned) => {
                let mut state = poisoned.into_inner();
                f(&mut state)
            }
        }
    }

    fn is_closed(&self) -> bool {
        return self.with_state(|state| state.closed || !state.receiver_alive);
    }

    fn close(&mut self) {
        self.with_state(|state| {
            state.closed = true;
        });
    }

    fn clone(&self) -> Channel<T> {
        return Clone::clone(self);
    }

    fn register_sender(&self) {
        self.with_state(|state| {
            state.sender_count += 1;
        });
    }

    fn release_sender(&self) {
        self.with_state(|state| {
            if state.sender_count > 0 {
                state.sender_count -= 1;
            }
            if state.sender_count == 0 {
                state.closed = true;
            }
        });
    }

    fn release_receiver(&self) {
        self.with_state(|state| {
            state.receiver_alive = false;
            state.closed = true;
        });
    }

    fn try_push_ref(&self, value: &T) -> __SifrChannelPushState {
        self.with_state(|state| {
            if state.closed || !state.receiver_alive {
                return __SifrChannelPushState::Closed;
            }
            if state.capacity >= 0 && (state.buffer.len() as i64) >= state.capacity {
                return __SifrChannelPushState::Full;
            }
            state.buffer.push_back(value.clone());
            __SifrChannelPushState::Sent
        })
    }

    fn push(&mut self, value: &T) -> Result<(), ClosedError> {
        self.with_state(|state| {
            if state.closed || !state.receiver_alive {
                return Err(ClosedError::new());
            }
            state.buffer.push_back(value.clone());
            Ok(())
        })
    }

    fn try_pop(&self) -> __SifrChannelPopState<T> {
        self.with_state(|state| {
            if let Some(value) = state.buffer.pop_front() {
                return __SifrChannelPopState::Item(value);
            }
            if state.closed || state.sender_count == 0 {
                return __SifrChannelPopState::Closed;
            }
            __SifrChannelPopState::Empty
        })
    }

    fn pop(&mut self) -> Result<T, ClosedError> {
        match self.try_pop() {
            __SifrChannelPopState::Item(value) => Ok(value),
            __SifrChannelPopState::Empty | __SifrChannelPopState::Closed => Err(ClosedError::new()),
        }
    }
}

impl<T: Clone> std::fmt::Display for Channel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", "Channel".to_string());
    }
}

#[derive(Debug)]
struct ChannelSender<T: Clone> {
    _channel: Channel<T>,
}

impl<T: Clone> ChannelSender<T> {
    fn new(channel: Channel<T>) -> Self {
        channel.register_sender();
        return Self { _channel: channel };
    }

    async fn send(&mut self, value: &T) -> Result<(), ClosedError> {
        loop {
            match self._channel.try_push_ref(value) {
                __SifrChannelPushState::Sent => return Ok(()),
                __SifrChannelPushState::Closed => return Err(ClosedError::new()),
                __SifrChannelPushState::Full => tokio::task::yield_now().await,
            }
        }
    }

    fn close(&mut self) {
        self._channel.close();
    }

    fn clone(&self) -> ChannelSender<T> {
        return ChannelSender::new(self._channel.clone());
    }
}

impl<T: Clone> Clone for ChannelSender<T> {
    fn clone(&self) -> Self {
        return ChannelSender::new(self._channel.clone());
    }
}

impl<T: Clone> Drop for ChannelSender<T> {
    fn drop(&mut self) {
        self._channel.release_sender();
    }
}

impl<T: Clone> std::fmt::Display for ChannelSender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", "ChannelSender".to_string());
    }
}

#[derive(Debug)]
struct ChannelReceiver<T: Clone> {
    _channel: Channel<T>,
}

impl<T: Clone> ChannelReceiver<T> {
    fn new(channel: Channel<T>) -> Self {
        return Self { _channel: channel };
    }

    async fn receive(&mut self) -> Result<T, ClosedError> {
        loop {
            match self._channel.try_pop() {
                __SifrChannelPopState::Item(value) => return Ok(value),
                __SifrChannelPopState::Closed => return Err(ClosedError::new()),
                __SifrChannelPopState::Empty => tokio::task::yield_now().await,
            }
        }
    }

    async fn anext(&mut self) -> Option<T> {
        match self.receive().await {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
}

impl<T: Clone> Drop for ChannelReceiver<T> {
    fn drop(&mut self) {
        self._channel.release_receiver();
    }
}

impl<T: Clone> std::fmt::Display for ChannelReceiver<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", "ChannelReceiver".to_string());
    }
}

fn channel<T: Clone + 'static>() -> (ChannelSender<T>, ChannelReceiver<T>) {
    let shared_channel = Channel::new(vec![], -(1 as i64));
    return (
        ChannelSender::new(shared_channel.clone()),
        ChannelReceiver::new(shared_channel),
    );
}

fn bounded_channel<T: Clone + 'static>(capacity: i64) -> (ChannelSender<T>, ChannelReceiver<T>) {
    let shared_channel = Channel::new(vec![], capacity);
    return (
        ChannelSender::new(shared_channel.clone()),
        ChannelReceiver::new(shared_channel),
    );
}
"#
}

fn annotate_async_main_entrypoint(items: &mut Vec<RustItem>) -> bool {
    for index in 0..items.len() {
        if let RustItem::Fn {
            name,
            is_async: true,
            ..
        } = &items[index]
        {
            if name == "main" {
                let already_annotated = index > 0
                    && matches!(
                        &items[index - 1],
                        RustItem::Attr(attr) if attr.contains("tokio::main")
                    );
                if !already_annotated {
                    items.insert(
                        index,
                        RustItem::Attr("#[tokio::main(flavor = \"current_thread\")]".to_string()),
                    );
                }
                return true;
            }
        }
    }
    false
}

fn module_uses_task_sleep(module: &HirModule) -> bool {
    fn expr_is_task_sleep(expr: &HirExpr) -> bool {
        matches!(expr, HirExpr::Call { func, .. } if func == "__sifr_task_sleep")
    }

    for (_, _, value) in &module.constants {
        let mut on_expr = |expr: &HirExpr| {
            if expr_is_task_sleep(expr) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        if matches!(
            traversal::walk_expr_until(value, &mut on_expr),
            TraversalControl::Stop
        ) {
            return true;
        }
    }

    for func in &module.functions {
        let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
        let mut on_expr = |expr: &HirExpr| {
            if expr_is_task_sleep(expr) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        if matches!(
            traversal::walk_stmts_until(
                &func.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut on_stmt,
                &mut on_expr
            ),
            TraversalControl::Stop
        ) {
            return true;
        }
    }

    for class in &module.classes {
        for method in &class.methods {
            let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
            let mut on_expr = |expr: &HirExpr| {
                if expr_is_task_sleep(expr) {
                    TraversalControl::Stop
                } else {
                    TraversalControl::Continue
                }
            };
            if matches!(
                traversal::walk_stmts_until(
                    &method.body,
                    TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                    &mut on_stmt,
                    &mut on_expr
                ),
                TraversalControl::Stop
            ) {
                return true;
            }
        }
    }

    false
}

fn type_contains_by(ty: &Type, predicate: fn(&Type) -> bool) -> bool {
    if predicate(ty) {
        return true;
    }

    match ty {
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Newtype { inner, .. }
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::Awaitable(inner) => type_contains_by(inner, predicate),
        Type::Dict(key, value)
        | Type::Result(key, value)
        | Type::Coroutine(key, value)
        | Type::Task(key, value)
        | Type::TaskResult(key, value)
        | Type::Select2(key, value)
        | Type::BlockingTask(key, value)
        | Type::AsyncIterator(key, value)
        | Type::AsyncGenerator(key, value) => {
            type_contains_by(key, predicate) || type_contains_by(value, predicate)
        }
        Type::Tuple(items) | Type::Union(items) | Type::Intersection(items) => {
            items.iter().any(|item| type_contains_by(item, predicate))
        }
        Type::Alias {
            type_args, body, ..
        } => {
            type_args.iter().any(|arg| type_contains_by(arg, predicate))
                || type_contains_by(body, predicate)
        }
        Type::Function(sig) | Type::AsyncFunction(sig) => {
            sig.params
                .iter()
                .any(|(_, param_ty, _)| type_contains_by(param_ty, predicate))
                || type_contains_by(&sig.return_type, predicate)
        }
        Type::Callable(params, _, ret) => {
            params
                .iter()
                .any(|param| type_contains_by(param, predicate))
                || type_contains_by(ret, predicate)
        }
        Type::Class {
            fields, methods, ..
        } => {
            fields
                .iter()
                .any(|(_, field_ty)| type_contains_by(field_ty, predicate))
                || methods.iter().any(|(_, method_sig)| {
                    method_sig
                        .params
                        .iter()
                        .any(|(_, param_ty, _)| type_contains_by(param_ty, predicate))
                        || type_contains_by(&method_sig.return_type, predicate)
                })
        }
        _ => false,
    }
}

fn type_contains_failure(ty: &Type) -> bool {
    type_contains_by(ty, |candidate| matches!(candidate, Type::Failure(_)))
}

fn type_contains_timeout_result(ty: &Type) -> bool {
    type_contains_by(ty, |candidate| matches!(candidate, Type::TimeoutResult(_)))
}

fn type_contains_async_generator(ty: &Type) -> bool {
    type_contains_by(ty, |candidate| {
        matches!(candidate, Type::AsyncGenerator(_, _))
    })
}

fn type_contains_cancellation_error(ty: &Type) -> bool {
    type_contains_by(
        ty,
        |candidate| matches!(candidate, Type::Class { name, .. } if name == "CancellationError"),
    )
}

fn type_contains_async_exit_cause(ty: &Type) -> bool {
    type_contains_by(
        ty,
        |candidate| matches!(candidate, Type::Class { name, .. } if name == "AsyncExitCause"),
    )
}

fn module_uses_failure_type(module: &HirModule) -> bool {
    module.functions.iter().any(function_uses_failure_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_failure(field_ty))
                || class.methods.iter().any(function_uses_failure_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_failure(ty))
}

fn module_uses_cancellation_error_type(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(function_uses_cancellation_error_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_cancellation_error(field_ty))
                || class
                    .methods
                    .iter()
                    .any(function_uses_cancellation_error_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_cancellation_error(ty))
}

fn module_uses_async_exit_cause_type(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(function_uses_async_exit_cause_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_async_exit_cause(field_ty))
                || class
                    .methods
                    .iter()
                    .any(function_uses_async_exit_cause_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_async_exit_cause(ty))
}

fn module_uses_timeout_result_type(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(function_uses_timeout_result_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_timeout_result(field_ty))
                || class.methods.iter().any(function_uses_timeout_result_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_timeout_result(ty))
}

fn module_uses_async_generator_type(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .any(function_uses_async_generator_type)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_async_generator(field_ty))
                || class.methods.iter().any(function_uses_async_generator_type)
        })
        || module
            .constants
            .iter()
            .any(|(_, ty, _)| type_contains_async_generator(ty))
}

fn function_uses_cancellation_error_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_cancellation_error(&param.ty))
        || type_contains_cancellation_error(&func.return_type)
}

fn function_uses_async_exit_cause_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_async_exit_cause(&param.ty))
        || type_contains_async_exit_cause(&func.return_type)
}

fn function_uses_failure_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_failure(&param.ty))
        || type_contains_failure(&func.return_type)
}

fn function_uses_timeout_result_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_timeout_result(&param.ty))
        || type_contains_timeout_result(&func.return_type)
}

fn function_uses_async_generator_type(func: &HirFunction) -> bool {
    func.params
        .iter()
        .any(|param| type_contains_async_generator(&param.ty))
        || type_contains_async_generator(&func.return_type)
}

fn body_contains_await(body: &[HirStmt]) -> bool {
    let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
    let mut on_expr = |expr: &HirExpr| {
        if matches!(expr, HirExpr::Await { .. }) {
            TraversalControl::Stop
        } else {
            TraversalControl::Continue
        }
    };
    matches!(
        traversal::walk_stmts_until(
            body,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr
        ),
        TraversalControl::Stop
    )
}

fn module_uses_task_scope(module: &HirModule) -> bool {
    fn stmt_uses_task_scope_runtime(stmt: &HirStmt) -> bool {
        matches!(
            stmt,
            HirStmt::AsyncWith {
                kind: sifr_hir::HirAsyncWithKind::TaskScope | sifr_hir::HirAsyncWithKind::TaskGroup,
                ..
            }
        )
    }
    fn expr_uses_task_scope_runtime(expr: &HirExpr) -> bool {
        matches!(expr, HirExpr::Call { func, .. } if func == "__sifr_task_gather" || func == "__sifr_task_race" || func == "__sifr_task_select" || func == "__sifr_spawn_blocking_infallible" || func == "__sifr_spawn_blocking_result")
    }

    for func in &module.functions {
        let mut on_stmt = |stmt: &HirStmt| {
            if stmt_uses_task_scope_runtime(stmt) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        let mut on_expr = |expr: &HirExpr| {
            if expr_uses_task_scope_runtime(expr) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        if matches!(
            traversal::walk_stmts_until(
                &func.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut on_stmt,
                &mut on_expr
            ),
            TraversalControl::Stop
        ) {
            return true;
        }
    }

    for class in &module.classes {
        for method in &class.methods {
            let mut on_stmt = |stmt: &HirStmt| {
                if stmt_uses_task_scope_runtime(stmt) {
                    TraversalControl::Stop
                } else {
                    TraversalControl::Continue
                }
            };
            let mut on_expr = |expr: &HirExpr| {
                if expr_uses_task_scope_runtime(expr) {
                    TraversalControl::Stop
                } else {
                    TraversalControl::Continue
                }
            };
            if matches!(
                traversal::walk_stmts_until(
                    &method.body,
                    TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                    &mut on_stmt,
                    &mut on_expr
                ),
                TraversalControl::Stop
            ) {
                return true;
            }
        }
    }

    false
}

fn public_visibility() -> syn::Visibility {
    syn::Visibility::Public(syn::token::Pub::default())
}

fn publicize_impl_items(items: &mut [syn::ImplItem]) {
    for item in items {
        if let syn::ImplItem::Fn(function) = item {
            function.vis = public_visibility();
        }
    }
}

fn publicize_struct_fields(fields: &mut syn::Fields) {
    match fields {
        syn::Fields::Named(fields) => {
            for field in &mut fields.named {
                field.vis = public_visibility();
            }
        }
        syn::Fields::Unnamed(fields) => {
            for field in &mut fields.unnamed {
                field.vis = public_visibility();
            }
        }
        syn::Fields::Unit => {}
    }
}

fn publicize_generated_module_source(source: &str) -> String {
    let mut file = syn::parse_file(source).unwrap_or_else(|error| {
        panic!("failed to parse generated module for publicization: {error}")
    });
    for item in &mut file.items {
        match item {
            syn::Item::Const(item) => item.vis = public_visibility(),
            syn::Item::Enum(item) => item.vis = public_visibility(),
            syn::Item::Fn(item) => item.vis = public_visibility(),
            syn::Item::Impl(item) => {
                if item.trait_.is_none() {
                    publicize_impl_items(&mut item.items);
                }
            }
            syn::Item::Static(item) => item.vis = public_visibility(),
            syn::Item::Struct(item) => {
                item.vis = public_visibility();
                publicize_struct_fields(&mut item.fields);
            }
            syn::Item::Trait(item) => item.vis = public_visibility(),
            syn::Item::Type(item) => item.vis = public_visibility(),
            syn::Item::Union(item) => {
                item.vis = public_visibility();
                for field in &mut item.fields.named {
                    field.vis = public_visibility();
                }
            }
            syn::Item::Use(item) => item.vis = public_visibility(),
            _ => {}
        }
    }
    prettyplease::unparse(&file)
}

fn render_local_module_imports(module: &HirModule) -> String {
    let mut module_import_items: Vec<RustItem> = Vec::new();
    for import in &module.imports {
        if import.module.starts_with("sifr.") || import.module.starts_with("_sifr.") {
            continue;
        }
        let mut module_path = vec!["crate".to_string()];
        module_path.extend(import.module.split('.').map(str::to_string));
        for name in &import.names {
            if let Some((_, alias)) = import.aliases.iter().find(|(orig, _)| orig == name) {
                let mut alias_path = module_path.clone();
                alias_path.push(name.clone());
                module_import_items.push(RustItem::UseAlias {
                    path: alias_path,
                    alias: alias.clone(),
                });
            } else {
                let mut import_path = module_path.clone();
                import_path.push(name.clone());
                module_import_items.push(RustItem::Use(import_path));
            }
        }
    }

    if module_import_items.is_empty() {
        String::new()
    } else {
        Renderer::new().render_file(&RustFile {
            items: module_import_items,
        })
    }
}

/// Generate Rust source code for a multi-module project, returning aggregate dependency metadata.
pub fn generate_rust_multi_with_metadata(
    modules: &[(&str, &HirModule)],
    stdlib_code: &StdlibCode,
) -> MultiModuleCodegenResult {
    let mut files = HashMap::new();
    let mut used_stdlib_modules = HashSet::new();
    let mut required_crates = HashSet::new();
    let mut project_codegen_code = stdlib_code.clone();

    for (module_name, module) in modules {
        project_codegen_code
            .func_signatures
            .insert((*module_name).to_string(), module_func_signatures(module));
        project_codegen_code
            .module_class_fields
            .insert((*module_name).to_string(), module_class_fields(module));
    }

    for (module_name, module) in modules {
        let module_public = *module_name != "main";
        let codegen_result = generate_rust_with_stdlib(module, &project_codegen_code);
        let local_imports = render_local_module_imports(module);
        let mut rust_source = codegen_result.rust_source;
        if !local_imports.trim().is_empty() {
            rust_source = format!("{}\n\n{}", local_imports.trim_end(), rust_source);
        }
        if module_public {
            rust_source = publicize_generated_module_source(&rust_source);
        }

        files.insert((*module_name).to_string(), rust_source);
        used_stdlib_modules.extend(codegen_result.used_stdlib_modules);
        required_crates.extend(codegen_result.required_crates);
    }

    MultiModuleCodegenResult {
        rust_files: files,
        used_stdlib_modules,
        required_crates,
    }
}

/// Generate Rust source code for a multi-module project.
/// Returns a map of filename -> Rust source code.
pub fn generate_rust_multi(modules: &[(&str, &HirModule)]) -> HashMap<String, String> {
    generate_rust_multi_with_metadata(modules, &StdlibCode::default())
        .rust_files
        .into_iter()
        .collect()
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

[workspace]
"#
    );

    // Add dependencies based on used stdlib/intrinsic modules
    let mut deps = Vec::new();
    for module_name in stdlib_modules
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
    {
        match module_name {
            "sifr.json" | "sifr.collections" | "_sifr.json" | "_sifr.collections" => {
                if !deps.contains(
                    &"serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }"
                        .to_string(),
                ) {
                    deps.push(
                        "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }"
                            .to_string(),
                    );
                    deps.push(
                        "serde = { version = \"1.0.228\", features = [\"derive\"] }".to_string(),
                    );
                }
            }
            "sifr.time" | "_sifr.time" => {
                if !deps.contains(&"chrono = \"0.4.44\"".to_string()) {
                    deps.push("chrono = \"0.4.44\"".to_string());
                }
            }
            "sifr.random" | "_sifr.crypto" => {
                if !deps.contains(&"rand = \"0.10.1\"".to_string()) {
                    deps.push("rand = \"0.10.1\"".to_string());
                }
                if !deps.contains(&"rand_distr = \"0.6.0\"".to_string()) {
                    deps.push("rand_distr = \"0.6.0\"".to_string());
                }
            }
            "sifr.uuid" | "_sifr.uuid" => {
                if !deps.contains(&"rand = \"0.10.1\"".to_string()) {
                    deps.push("rand = \"0.10.1\"".to_string());
                }
                let uuid_dep =
                    "uuid = { version = \"1.23.1\", features = [\"v3\", \"v5\"] }".to_string();
                if !deps.contains(&uuid_dep) {
                    deps.push(uuid_dep);
                }
            }
            "sifr.re" | "_sifr.regex" => {
                if !deps.contains(&"regex = \"1.12.3\"".to_string()) {
                    deps.push("regex = \"1.12.3\"".to_string());
                }
            }
            "sifr.pathlib" => {
                if !deps.contains(&"regex = \"1.12.3\"".to_string()) {
                    deps.push("regex = \"1.12.3\"".to_string());
                }
            }
            "sifr.hash" | "sifr.hashlib" => {
                if !deps.contains(&"sha2 = \"0.11.0\"".to_string()) {
                    deps.push("sha2 = \"0.11.0\"".to_string());
                    deps.push("md5 = \"0.8.0\"".to_string());
                    deps.push("sha1 = \"0.11.0\"".to_string());
                    deps.push("blake2 = \"0.10.6\"".to_string());
                }
            }
            "sifr.encoding" | "sifr.base64" => {
                if !deps.contains(&"base64 = \"0.22.1\"".to_string()) {
                    deps.push("base64 = \"0.22.1\"".to_string());
                }
            }
            "sifr.tomllib" | "_sifr.toml" => {
                let toml_dep =
                    "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }".to_string();
                if !deps.contains(&toml_dep) {
                    deps.push(toml_dep);
                }
            }
            "sifr.datetime" | "_sifr.datetime" => {
                if !deps.contains(&"chrono = \"0.4.44\"".to_string()) {
                    deps.push("chrono = \"0.4.44\"".to_string());
                }
            }
            "sifr.gzip" | "sifr.zipfile" | "_sifr.compress" => {
                if !deps.contains(&"flate2 = \"1.1.9\"".to_string()) {
                    deps.push("flate2 = \"1.1.9\"".to_string());
                }
                if !deps.contains(&"zip = \"8.6.0\"".to_string()) {
                    deps.push("zip = \"8.6.0\"".to_string());
                }
            }
            "_bigint" => {
                if !deps.contains(&"num-bigint = \"0.4.6\"".to_string()) {
                    deps.push("num-bigint = \"0.4.6\"".to_string());
                    deps.push("num-traits = \"0.2.19\"".to_string());
                }
            }
            // sifr.io, sifr.env, sifr.os, sifr.math, sifr.test, sifr.bytes, sifr.sys,
            // sifr.subprocess, sifr.html, sifr.calendar, sifr.operator use only std library
            _ => {}
        }
    }

    for crate_name in required_crates
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
    {
        match crate_name {
            "serde_json" => {
                if !deps.contains(
                    &"serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }"
                        .to_string(),
                ) {
                    deps.push(
                        "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }"
                            .to_string(),
                    );
                }
                if !deps
                    .contains(&"serde = { version = \"1.0.228\", features = [\"derive\"] }".to_string())
                {
                    deps.push("serde = { version = \"1.0.228\", features = [\"derive\"] }".to_string());
                }
            }
            "chrono" => {
                if !deps.contains(&"chrono = \"0.4.44\"".to_string()) {
                    deps.push("chrono = \"0.4.44\"".to_string());
                }
            }
            "rand" => {
                if !deps.contains(&"rand = \"0.10.1\"".to_string()) {
                    deps.push("rand = \"0.10.1\"".to_string());
                }
            }
            "rand_distr" => {
                if !deps.contains(&"rand_distr = \"0.6.0\"".to_string()) {
                    deps.push("rand_distr = \"0.6.0\"".to_string());
                }
            }
            "regex" => {
                if !deps.contains(&"regex = \"1.12.3\"".to_string()) {
                    deps.push("regex = \"1.12.3\"".to_string());
                }
            }
            "sha2" => {
                if !deps.contains(&"sha2 = \"0.11.0\"".to_string()) {
                    deps.push("sha2 = \"0.11.0\"".to_string());
                }
            }
            "md5" => {
                if !deps.contains(&"md5 = \"0.8.0\"".to_string()) {
                    deps.push("md5 = \"0.8.0\"".to_string());
                }
            }
            "sha1" => {
                if !deps.contains(&"sha1 = \"0.11.0\"".to_string()) {
                    deps.push("sha1 = \"0.11.0\"".to_string());
                }
            }
            "uuid" => {
                let uuid_dep =
                    "uuid = { version = \"1.23.1\", features = [\"v3\", \"v5\"] }".to_string();
                if !deps.contains(&uuid_dep) {
                    deps.push(uuid_dep);
                }
            }
            "blake2" => {
                if !deps.contains(&"blake2 = \"0.10.6\"".to_string()) {
                    deps.push("blake2 = \"0.10.6\"".to_string());
                }
            }
            "base64" => {
                if !deps.contains(&"base64 = \"0.22.1\"".to_string()) {
                    deps.push("base64 = \"0.22.1\"".to_string());
                }
            }
            "toml" => {
                let toml_dep =
                    "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }".to_string();
                if !deps.contains(&toml_dep) {
                    deps.push(toml_dep);
                }
            }
            "flate2" => {
                if !deps.contains(&"flate2 = \"1.1.9\"".to_string()) {
                    deps.push("flate2 = \"1.1.9\"".to_string());
                }
            }
            "zip" => {
                if !deps.contains(&"zip = \"8.6.0\"".to_string()) {
                    deps.push("zip = \"8.6.0\"".to_string());
                }
            }
            "num-bigint" => {
                if !deps.contains(&"num-bigint = \"0.4.6\"".to_string()) {
                    deps.push("num-bigint = \"0.4.6\"".to_string());
                }
            }
            "num-traits" => {
                if !deps.contains(&"num-traits = \"0.2.19\"".to_string()) {
                    deps.push("num-traits = \"0.2.19\"".to_string());
                }
            }
            "rust_decimal" => {
                if !deps.contains(
                    &"rust_decimal = { version = \"1.41.0\", features = [\"maths\", \"serde-with-str\"] }".to_string(),
                ) {
                    deps.push(
                        "rust_decimal = { version = \"1.41.0\", features = [\"maths\", \"serde-with-str\"] }".to_string(),
                    );
                }
            }
            "bigdecimal" => {
                if !deps.contains(
                    &"bigdecimal = { version = \"0.4.10\", features = [\"serde\"] }".to_string(),
                ) {
                    deps.push(
                        "bigdecimal = { version = \"0.4.10\", features = [\"serde\"] }".to_string(),
                    );
                }
            }
            "sifr_runtime" | "sifr-runtime" => {
                let dep = sifr_runtime_dependency_spec();
                if !deps.contains(&dep) {
                    deps.push(dep);
                }
            }
            "tokio" => {
                let dep = tokio_dependency_spec();
                if !deps.contains(&dep) {
                    deps.push(dep);
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
    collection_needs: CollectionNeeds,
    runtime_needs: RuntimeNeeds,
    /// Track union enum types that need to be defined (name -> member types)
    union_enums: HashMap<String, Vec<Type>>,
    /// Accumulated union enum items to prepend
    enum_items: Vec<RustItem>,
    /// Accumulated non-enum body items to assemble before raw output rendering.
    body_items: Vec<RustItem>,
    /// The return type of the function currently being emitted
    current_return_type: Option<Type>,
    /// Active `async with task.timeout(...)` duration expressions for await lowering.
    active_timeout_durations: Vec<RustExpr>,
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
    /// Map of (`class_name`, `field_name`) -> concrete Rust type used for recursive field storage.
    recursive_field_rust_types: HashMap<(String, String), String>,
    /// Map from class name -> ordered list of field names (for constructor arg mapping)
    class_field_order: HashMap<String, Vec<String>>,
    /// Map of (`class_name`, `field_name`) -> field type for method receiver recovery.
    class_field_types: HashMap<(String, String), Type>,
    /// Map from nested function name -> list of captured variable (name, type) pairs
    /// Used to pass extra args at call sites for recursive+capturing nested functions
    nested_fn_captures: HashMap<String, Vec<NestedFnCapture>>,
    /// Map from module-level constant name -> (type, `rust_name`)
    /// For primitives: `rust_name` is the UPPERCASE const name
    /// For strings/complex: `rust_name` is __`const_name()` function call
    module_constants: HashMap<String, (Type, String)>,
    /// Set of class names that have generic type parameters
    generic_classes: HashSet<String>,
    /// Map of generic class name -> list of type parameter names (e.g., `Counter` -> `T`)
    generic_class_params: HashMap<String, Vec<String>>,
    /// Map of generic class name -> original HIR class template.
    generic_class_templates: HashMap<String, sifr_hir::HirClass>,
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
    /// Depth of try-block closures that capture return statements.
    try_closure_depth: usize,
    /// Per-try closure return wrapping mode (true => wrap return payload in Some(...)).
    try_closure_option_wrap: Vec<bool>,
    /// Per-try closure target error type for `?` adaptation.
    try_closure_error_type: Vec<String>,
    /// Map from variable name -> Callable parameter (type, convention) list.
    /// Populated per-function from params and locals with Callable types.
    /// Used to emit correct &arg/&mut arg/arg for Callable-typed variable calls.
    callable_var_conventions: HashMap<String, Vec<(Type, ParamConvention)>>,
    /// Map from local binding name -> declared type for the active function-like scope.
    /// Used to preserve assignment coercions that depend on the target local type.
    local_binding_types: HashMap<String, Type>,
    /// Local names widened to `T | None` due `name = None` reassignment in current scope.
    none_widened_local_bindings: HashSet<String>,
    /// Local names whose generated Rust binding has been promoted from legacy `i64` to `SifrInt`.
    sifr_int_local_bindings: RefCell<HashSet<String>>,
    /// Local names pre-promoted to `SifrInt` because a later assignment needs exact-int storage.
    sifr_int_forced_local_bindings: RefCell<HashSet<String>>,
    /// Local names whose generated Rust `Result[int, E]` binding payload is `SifrInt`.
    sifr_int_result_local_bindings: RefCell<HashSet<String>>,
    /// Function names whose generated Rust return type has been promoted from legacy `i64` to `SifrInt`.
    sifr_int_function_returns: RefCell<HashSet<String>>,
    /// Function names whose `Result[int, E]` generated Rust return payload is `SifrInt`.
    sifr_int_result_function_returns: RefCell<HashSet<String>>,
    /// Class method keys whose `Result[int, E]` generated Rust return payload is `SifrInt`.
    sifr_int_result_method_returns: RefCell<HashSet<String>>,
    /// Module-level function `int` parameters promoted from legacy `i64` to `SifrInt`.
    sifr_int_function_params: RefCell<HashMap<String, HashSet<usize>>>,
    /// Module-level function `Result[int, E]` parameters promoted from legacy `i64` payloads to `SifrInt`.
    sifr_int_result_function_params: RefCell<HashMap<String, HashSet<usize>>>,
    /// Class method `Result[int, E]` parameters promoted from legacy `i64` payloads to `SifrInt`.
    sifr_int_result_method_params: RefCell<HashMap<String, HashSet<usize>>>,
    /// Whether the active function-like body returns `SifrInt` for source-level `int`.
    current_sifr_int_return: Cell<bool>,
    /// Whether the active function-like body returns `Result<SifrInt, E>` for source-level `Result[int, E]`.
    current_sifr_int_result_return: Cell<bool>,
    /// Stack used to capture structured statement emission as IR nodes.
    stmt_capture_stack: Vec<Vec<RustStmt>>,
    /// Recursion guard for non-structured emitter paths.
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
    flags: HashSet<RuntimeNeed>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RuntimeNeed {
    FileHandles,
    LoggingState,
    RandomModuleState,
    BigInt,
}

impl RuntimeNeeds {
    fn require(&mut self, need: RuntimeNeed) {
        self.flags.insert(need);
    }

    fn contains(&self, need: RuntimeNeed) -> bool {
        self.flags.contains(&need)
    }

    fn file_handles(&self) -> bool {
        self.contains(RuntimeNeed::FileHandles)
    }

    fn logging_state(&self) -> bool {
        self.contains(RuntimeNeed::LoggingState)
    }

    fn random_module_state(&self) -> bool {
        self.contains(RuntimeNeed::RandomModuleState)
    }

    fn bigint(&self) -> bool {
        self.contains(RuntimeNeed::BigInt)
    }
}

#[derive(Default)]
struct EmissionContext {
    in_generator_closure: bool,
    in_display_impl: bool,
}

impl RustEmitter {
    fn new() -> Self {
        Self {
            collection_needs: CollectionNeeds::default(),
            runtime_needs: RuntimeNeeds::default(),
            union_enums: HashMap::new(),
            enum_items: Vec::new(),
            body_items: Vec::new(),
            current_return_type: None,
            active_timeout_durations: Vec::new(),
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
            recursive_field_rust_types: HashMap::new(),
            class_field_order: HashMap::new(),
            class_field_types: HashMap::new(),
            nested_fn_captures: HashMap::new(),
            module_constants: HashMap::new(),
            generic_classes: HashSet::new(),
            generic_class_params: HashMap::new(),
            generic_class_templates: HashMap::new(),
            borrowed_params: HashSet::new(),
            mut_borrowed_params: HashSet::new(),
            stdlib_intrinsic_names: HashMap::new(),
            generator_functions: HashSet::new(),
            imported_stdlib_names: HashMap::new(),
            pending_self_field_clone_suppression: 0,
            emission_ctx: EmissionContext::default(),
            try_enum_counter: 0,
            try_closure_depth: 0,
            try_closure_option_wrap: Vec::new(),
            try_closure_error_type: Vec::new(),
            callable_var_conventions: HashMap::new(),
            local_binding_types: HashMap::new(),
            none_widened_local_bindings: HashSet::new(),
            sifr_int_local_bindings: RefCell::new(HashSet::new()),
            sifr_int_forced_local_bindings: RefCell::new(HashSet::new()),
            sifr_int_result_local_bindings: RefCell::new(HashSet::new()),
            sifr_int_function_returns: RefCell::new(HashSet::new()),
            sifr_int_result_function_returns: RefCell::new(HashSet::new()),
            sifr_int_result_method_returns: RefCell::new(HashSet::new()),
            sifr_int_function_params: RefCell::new(HashMap::new()),
            sifr_int_result_function_params: RefCell::new(HashMap::new()),
            sifr_int_result_method_params: RefCell::new(HashMap::new()),
            current_sifr_int_return: Cell::new(false),
            current_sifr_int_result_return: Cell::new(false),
            stmt_capture_stack: Vec::new(),
            lowering_stats: LoweringStats::default(),
        }
    }

    pub(crate) fn capture_structured_stmts<F>(&mut self, emit: F) -> Vec<RustStmt>
    where
        F: FnOnce(&mut Self),
    {
        self.stmt_capture_stack.push(Vec::new());
        emit(self);
        self.stmt_capture_stack.pop().unwrap_or_default()
    }

    fn collect_recursive_nested_fn_captures(&self, func: &HirFunction) -> Vec<NestedFnCapture> {
        if !crate::hir_analysis::queries::body_calls_function(&func.body, &func.name) {
            return Vec::new();
        }

        let param_names = func
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<HashSet<_>>();
        let referenced_with_types = collect_referenced_vars_with_types(&func.body);
        let locally_defined = collect_locally_defined_vars(&func.body);
        let mutated_captures = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);

        let mut captures = referenced_with_types
            .into_iter()
            .filter(|(name, _)| !param_names.contains(name) && !locally_defined.contains(name))
            .map(|(name, ty)| {
                let convention = if ty.ownership() == sifr_type_system::OwnershipKind::Copy {
                    ParamConvention::own()
                } else if mutated_captures.contains(&name) {
                    ParamConvention::mut_borrow()
                } else {
                    default_param_convention(&ty)
                };
                NestedFnCapture {
                    name,
                    ty,
                    convention,
                }
            })
            .collect::<Vec<_>>();
        captures.sort_by(|left, right| left.name.cmp(&right.name));
        captures
    }

    fn emit_module(&mut self, module: &HirModule, module_public: bool, test_mode: bool) {
        // Pre-scan: detect bigint usage
        if module_uses_bigint(module) {
            self.runtime_needs.require(RuntimeNeed::BigInt);
        }

        self.prescan_module_metadata(module);

        self.emit_module_constants(module, module_public);
        self.register_sifr_int_function_returns(module);
        self.emit_module_body(module, module_public, test_mode);
    }

    pub(crate) fn try_lower_structured_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
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

        if let HirStmt::NestedFunction { func } = stmt {
            let captures = self.collect_recursive_nested_fn_captures(func);
            if captures.is_empty() {
                self.nested_fn_captures.remove(&func.name);
            } else {
                self.nested_fn_captures.insert(func.name.clone(), captures);
            }
        }

        let should_bypass_simple_lowering = matches!(
            stmt,
            HirStmt::NestedFunction { .. } | HirStmt::Assign { .. }
        ) || matches!(
            stmt,
            HirStmt::AsyncWith {
                kind: sifr_hir::HirAsyncWithKind::TaskTimeout { .. }
                    | sifr_hir::HirAsyncWithKind::UserDefined { .. },
                body,
                ..
            } if body_contains_await(body)
        ) || matches!(
            stmt,
            HirStmt::AsyncWith {
                kind: sifr_hir::HirAsyncWithKind::UserDefined { .. },
                ..
            }
        ) || matches!(stmt, HirStmt::Let { ty, .. } if self.type_contains_generic_class(ty));
        if !should_bypass_simple_lowering {
            if let Some(lowered_stmts) = try_lower_simple_stmt_with_scope_result_and_bindings(
                stmt,
                &self.mutated_vars,
                &self.borrowed_params,
                &self.local_binding_types,
                &scope_ctx,
            )? {
                self.lowering_stats.expr_candidate_total += 1;
                self.lowering_stats.expr_candidate_structured += 1;
                let rewritten_stmts = lowered_stmts
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect::<Vec<_>>();
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                self.emit_lowered_stmts(&rewritten_stmts);
                return Ok(true);
            }
        }

        if self.try_lower_structured_nested_function_stmt(stmt) {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }

        if let HirStmt::AsyncWith { kind, target, body } = stmt {
            if let Some(lowered_stmt) =
                self.try_lower_async_with_stmt_for_ir(kind, target.as_deref(), body)?
            {
                self.push_captured_stmt(&self.rewrite_stdlib_constant_idents_in_stmt(lowered_stmt));
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
        }

        if let HirStmt::AsyncFor {
            target,
            iter,
            iter_error_ty,
            close_error_ty,
            body,
            ..
        } = stmt
        {
            if let Some(lowered_stmt) = self.try_lower_async_for_stmt_for_ir(
                target,
                iter,
                iter_error_ty,
                close_error_ty.as_ref(),
                body,
            )? {
                self.push_captured_stmt(&self.rewrite_stdlib_constant_idents_in_stmt(lowered_stmt));
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
        }

        if let HirStmt::TupleUnpack { targets, value } = stmt {
            if let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? {
                let lowered_stmts = crate::lower_tuple_unpack_targets(
                    targets,
                    self.rewrite_stdlib_constant_idents_in_expr(lowered_value),
                    &self.mutated_vars,
                );
                for lowered_stmt in lowered_stmts {
                    self.push_captured_stmt(&lowered_stmt);
                }
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
        }

        if let HirStmt::Let {
            name, ty, value, ..
        } = stmt
        {
            let effective_ty = self
                .local_binding_types
                .get(name)
                .cloned()
                .unwrap_or_else(|| ty.clone());
            let is_generic_class = matches!(
                &effective_ty,
                Type::Class {
                    name: class_name,
                    ..
                } if self.generic_classes.contains(class_name)
            );
            let lowered_value = if effective_ty.ownership() == sifr_type_system::OwnershipKind::Move
            {
                if let HirExpr::Name {
                    name: value_name, ..
                } = value
                {
                    if self.borrowed_params.contains(value_name)
                        || self.mut_borrowed_params.contains(value_name)
                    {
                        Some(crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(
                            value_name.clone(),
                        ))))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            let lowered_value = if let Some(clone_expr) = lowered_value {
                clone_expr
            } else {
                if let Some(lowered) = self.lower_rendered_expr_for_ir(value)? {
                    self.coerce_local_value_for_target_type_for_ir(&effective_ty, value, lowered)?
                } else if let Some(lowered) = self.lower_stmt_expr_for_ir(value)? {
                    self.coerce_local_value_for_target_type_for_ir(&effective_ty, value, lowered)?
                } else {
                    return Ok(false);
                }
            };

            let lowered_stmt = RustStmt::Let {
                mutable: self.mutated_vars.contains(name)
                    || matches!(
                        &effective_ty,
                        Type::Alias { name: alias_name, .. }
                            if alias_name.starts_with("__compat_defaultdict_")
                    )
                    || matches!(effective_ty.resolve_alias(), Type::Iterator(_)),
                name: name.clone(),
                ty: if name == "_"
                    || is_generic_class
                    || match (&effective_ty, value) {
                        (resolved_ty, HirExpr::Call { func, args, .. })
                            if matches!(
                                resolve_alias_type_for_plain_call(resolved_ty),
                                Type::Set(_)
                            ) && func == "set"
                                && args.is_empty() =>
                        {
                            true
                        }
                        (
                            Type::Alias {
                                name: alias_name,
                                body,
                                ..
                            },
                            HirExpr::Call { func, args, .. },
                        ) if func == alias_name
                            && args.is_empty()
                            && alias_name.starts_with("__compat_defaultdict_") =>
                        {
                            if let Type::Dict(key_ty, value_ty) = body.resolve_alias() {
                                matches!(key_ty.as_ref(), Type::Any | Type::Unknown)
                                    || matches!(value_ty.as_ref(), Type::List(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
                                    || matches!(value_ty.as_ref(), Type::Set(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
                            } else {
                                false
                            }
                        }
                        _ => false,
                    } {
                    None
                } else {
                    Some(self.rust_ir_type_with_generics(&effective_ty))
                },
                value: lowered_value,
            };
            let lowered_stmt = self.rewrite_stdlib_constant_idents_in_stmt(lowered_stmt);
            self.push_captured_stmt(&lowered_stmt);
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }

        if let HirStmt::Assign { name, value } = stmt {
            let lowered_value = if let Some(lowered) = self.lower_rendered_expr_for_ir(value)? {
                if let Some(target_ty) = self.local_binding_types.get(name).cloned() {
                    let mut lowered =
                        self.coerce_local_value_for_target_type_for_ir(&target_ty, value, lowered)?;
                    if !crate::helpers::is_option_type(&target_ty)
                        && crate::helpers::is_option_type(value.ty())
                    {
                        let fallback = if crate::helpers::is_copy_type_for_codegen(&target_ty) {
                            RustExpr::Ident(name.clone())
                        } else {
                            RustExpr::Clone(Box::new(RustExpr::Ident(name.clone())))
                        };
                        lowered = RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                            method: "unwrap_or".to_string(),
                            args: vec![fallback],
                        };
                    }
                    lowered
                } else {
                    lowered
                }
            } else if let Some(lowered) = self.lower_stmt_expr_for_ir(value)? {
                if let Some(target_ty) = self.local_binding_types.get(name).cloned() {
                    let mut lowered =
                        self.coerce_local_value_for_target_type_for_ir(&target_ty, value, lowered)?;
                    if !crate::helpers::is_option_type(&target_ty)
                        && crate::helpers::is_option_type(value.ty())
                    {
                        let fallback = if crate::helpers::is_copy_type_for_codegen(&target_ty) {
                            RustExpr::Ident(name.clone())
                        } else {
                            RustExpr::Clone(Box::new(RustExpr::Ident(name.clone())))
                        };
                        lowered = RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                            method: "unwrap_or".to_string(),
                            args: vec![fallback],
                        };
                    }
                    lowered
                } else {
                    lowered
                }
            } else {
                return Ok(false);
            };
            let lowered_stmt = RustStmt::Assign {
                target: RustExpr::Ident(name.clone()),
                value: lowered_value,
            };
            let lowered_stmt = self.rewrite_stdlib_constant_idents_in_stmt(lowered_stmt);
            self.push_captured_stmt(&lowered_stmt);
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if let HirStmt::Yield { value } = stmt {
            let lowered_value = if let Some(lowered) = self.lower_rendered_expr_for_ir(value)? {
                lowered
            } else if let Some(lowered) = self.lower_stmt_expr_for_ir(value)? {
                lowered
            } else {
                return Ok(false);
            };
            self.push_captured_stmt(&RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("_yields".to_string())),
                method: "push".to_string(),
                args: vec![lowered_value],
            }));
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_field_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_nested_field_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_subscript_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_nested_subscript_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_attribute_nested_subscript_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_subscript_augassign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_delete_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_attribute_subscript_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_return_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_raise_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_if_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_while_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_for_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_with_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_try_except_stmt(stmt) {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if let HirStmt::TryFinally { body, finalbody } = stmt {
            let Some(lowered_stmts) = self.try_lower_try_finally_stmt_for_ir(body, finalbody)?
            else {
                return Ok(false);
            };
            for lowered_stmt in lowered_stmts {
                self.push_captured_stmt(&lowered_stmt);
            }
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_assert_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if self.try_lower_structured_aug_assign_stmt(stmt)? {
            self.lowering_stats.stmt_structured += 1;
            self.lowering_stats.stmt_candidate_structured += 1;
            return Ok(true);
        }
        if let HirStmt::Expr { expr } = stmt {
            if let Some(lowered_expr) = self.try_lower_stmt_expr_statement_only(expr)? {
                self.lowering_stats.expr_total += 1;
                self.lowering_stats.expr_candidate_total += 1;
                self.lowering_stats.expr_structured += 1;
                self.lowering_stats.expr_candidate_structured += 1;
                let rewritten = self.rewrite_stdlib_constant_idents_in_expr(lowered_expr);
                self.push_captured_stmt(&RustStmt::Expr(rewritten));
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
            if let Some(lowered_expr) = self.lower_stmt_expr_for_ir(expr)? {
                self.lowering_stats.expr_total += 1;
                self.lowering_stats.expr_candidate_total += 1;
                self.lowering_stats.expr_structured += 1;
                self.lowering_stats.expr_candidate_structured += 1;
                let rewritten = self.rewrite_stdlib_constant_idents_in_expr(lowered_expr);
                self.push_captured_stmt(&RustStmt::Expr(rewritten));
                self.lowering_stats.stmt_structured += 1;
                self.lowering_stats.stmt_candidate_structured += 1;
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn emit_stmt(&mut self, stmt: &HirStmt) {
        self.lowering_stats.stmt_total += 1;
        if is_simple_stmt_candidate(stmt) {
            self.lowering_stats.stmt_candidate_total += 1;
        }
        match self.try_lower_structured_stmt(stmt) {
            Ok(true) => {}
            Ok(false) => {
                self.lowering_stats.stmt_lowering_errors += 1;
                self.push_captured_stmt(&RustStmt::Expr(RustExpr::MacroCall {
                    name: "compile_error".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Str(format!(
                        "structured statement emission missing for production path: {stmt:?}"
                    )))],
                }));
            }
            Err(err) => {
                self.lowering_stats.stmt_lowering_errors += 1;
                self.push_captured_stmt(&RustStmt::Expr(RustExpr::MacroCall {
                    name: "compile_error".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Str(format!(
                        "structured statement lowering failed for production path: {stmt:?}; error={err}"
                    )))],
                }));
            }
        }
    }
}

pub fn body_contains_yield(stmts: &[HirStmt]) -> bool {
    hir_analysis::queries::body_contains_yield(stmts)
}
