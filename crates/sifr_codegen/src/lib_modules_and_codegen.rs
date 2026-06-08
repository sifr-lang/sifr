use super::{
    annotate_async_main_entrypoint, build_async_exit_cause_type_items,
    build_async_generator_type_items, build_cancellation_error_type_items, build_cpu_offload_items,
    build_error_into_error_impl, build_error_type_items, build_failure_type_items,
    build_file_handle_infra_items, build_file_handle_struct_items, build_io_error_items,
    build_join_set_cpu_items, build_join_set_items, build_logging_items, build_process_async_items,
    build_process_child_items, build_process_status_items, build_random_module_state_items,
    build_task_scope_cpu_offload_items, build_task_scope_items, build_task_scope_offload_items,
    build_timeout_result_type_items, build_worker_panic_hook_items,
    module_uses_async_exit_cause_type, module_uses_async_generator_type,
    module_uses_cancellation_error_type, module_uses_failure_type, module_uses_join_set,
    module_uses_join_set_spawn_cpu, module_uses_spawn_cpu, module_uses_task_scope,
    module_uses_task_scope_offload, module_uses_task_scope_spawn_cpu, module_uses_task_sleep,
    module_uses_timeout_result_type, replace_parallel_runtime_items,
    replace_sync_channel_runtime_items, sifr_type_to_rust_type, sync_channel_runtime_needed,
    Renderer, RustEmitter, RustExpr, RustFile, RustItem, RustLiteral,
};
use crate::error_refs::collect_referenced_builtin_error_classes;
use crate::ir_imports::{collect_import_needs_from_items, collect_import_needs_from_source};
use crate::ir_optimize::remove_trivial_clones_in_items;
use crate::ir_validate::validate_items;
use crate::stdlib_filter::{
    collect_and_strip_shared_prelude, dedup_rust_items, filter_stdlib_ir_to_needed,
    SharedPreludeProcessAsyncNeeds,
};
use sifr_ir::HirModule;
use sifr_stdlib::StdlibFeature;
use sifr_type_system::{ParamConvention, Type};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;

pub(crate) type FuncSignature = (Vec<(Type, ParamConvention)>, Type);
pub(crate) type ModuleFuncSignatures = HashMap<String, FuncSignature>;
type StdlibFuncSignatures = HashMap<String, ModuleFuncSignatures>;
pub(crate) type UnionVariantTypes = Vec<(String, Type)>;
pub(crate) type IsinstanceUnionMatch = (String, String, String, UnionVariantTypes);
pub(crate) type IsNoneUnionMatch = (String, String, UnionVariantTypes);

pub use crate::entrypoints::{generate_rust, generate_rust_test, generate_rust_with_metadata};

#[derive(Clone)]
pub(crate) struct NestedFnCapture {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) convention: ParamConvention,
}

/// Built-in error class names that the compiler provides.
pub(crate) const BUILTIN_ERROR_CLASSES: &[&str] = &[
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
    "WorkerRuntimeError",
    "WorkerError",
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
    /// Required stdlib/runtime features discovered during structured lowering/codegen.
    pub required_features: HashSet<StdlibFeature>,
    /// Map of `constant_name` -> (type, `rust_name`) for module-level constants
    pub constant_mappings: HashMap<String, (Type, String)>,
    /// Counters for structured lowering usage during emission.
    pub lowering_stats: LoweringStats,
}

/// Result of multi-module code generation, including aggregate dependency metadata.
pub struct MultiModuleCodegenResult {
    pub rust_files: HashMap<String, String>,
    pub used_stdlib_modules: HashSet<String>,
    pub required_features: HashSet<StdlibFeature>,
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
    pub generic_class_templates: HashMap<String, sifr_ir::HirClass>,
    /// Map of `module_name` -> (`class_name` -> ordered class fields).
    /// Multi-module project codegen also uses this for local helper modules.
    pub module_class_fields: HashMap<String, HashMap<String, Vec<(String, Type)>>>,
}

pub(super) fn module_func_signatures(module: &HirModule) -> ModuleFuncSignatures {
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

pub(super) fn module_class_fields(module: &HirModule) -> HashMap<String, Vec<(String, Type)>> {
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
    let mut transitive_dependency_modules: HashSet<String> = HashSet::new();
    let mut stdlib_needs_file_handles = false;
    let mut stdlib_needs_process_status = false;
    let mut stdlib_needs_process_async = SharedPreludeProcessAsyncNeeds::default();
    let mut stdlib_needs_process_children = false;
    let mut stdlib_provides_file_handle_struct = false;
    for module_name in emitter.used_stdlib_modules.iter().collect::<BTreeSet<_>>() {
        if let Some(deps) = stdlib_code.transitive_deps.get(module_name) {
            for dep in deps.iter().collect::<BTreeSet<_>>() {
                if dep.starts_with("sifr.") && !all_needed.contains(dep) {
                    all_needed.push(dep.clone());
                }
                if dep.starts_with("sifr.") {
                    transitive_dependency_modules.insert(dep.clone());
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
                        if transitive_dependency_modules.contains(module_name) {
                            rust_code.clone()
                        } else if pure_sifr_imports.is_empty() {
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
                if module_name == "sifr.parallel" {
                    filtered = replace_parallel_runtime_items(&filtered);
                }
                if !filtered.trim().is_empty() {
                    let prepared = collect_and_strip_shared_prelude(&filtered);
                    stdlib_needs_file_handles |=
                        prepared.shared_needs.file_handles.needs_file_handles;
                    stdlib_needs_process_status |=
                        prepared.shared_needs.process_status.needs_process_status;
                    stdlib_needs_process_async.needs_run |=
                        prepared.shared_needs.process_async.needs_run;
                    stdlib_needs_process_async.needs_run_timeout |=
                        prepared.shared_needs.process_async.needs_run_timeout;
                    stdlib_needs_process_async.needs_output |=
                        prepared.shared_needs.process_async.needs_output;
                    stdlib_needs_process_async.needs_output_timeout |=
                        prepared.shared_needs.process_async.needs_output_timeout;
                    stdlib_needs_process_async.needs_spawn |=
                        prepared.shared_needs.process_async.needs_spawn;
                    stdlib_needs_process_async.needs_wait |=
                        prepared.shared_needs.process_async.needs_wait;
                    stdlib_needs_process_children |= prepared
                        .shared_needs
                        .process_children
                        .needs_process_children;
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
    let needs_process_async = stdlib_needs_process_async.needs_run
        || stdlib_needs_process_async.needs_run_timeout
        || stdlib_needs_process_async.needs_output
        || stdlib_needs_process_async.needs_output_timeout
        || stdlib_needs_process_async.needs_spawn
        || stdlib_needs_process_async.needs_wait;
    let needs_process_status = stdlib_needs_process_status || needs_process_async;
    let needs_process_children = stdlib_needs_process_children;
    let needs_logging = emitter.used_stdlib_modules.contains("sifr.logging")
        || emitter.used_stdlib_modules.contains("_sifr.logging")
        || emitter.runtime_needs.logging_state();
    let needs_random_module_state = emitter.runtime_needs.random_module_state();

    // Emit built-in error class struct definitions for referenced error types.
    let uses_task_scope = module_uses_task_scope(module);
    let uses_join_set = module_uses_join_set(module);
    let uses_join_set_spawn_cpu = module_uses_join_set_spawn_cpu(module);
    let uses_task_scope_offload = module_uses_task_scope_offload(module);
    let uses_task_scope_spawn_cpu = module_uses_task_scope_spawn_cpu(module);
    let uses_spawn_cpu = module_uses_spawn_cpu(module);
    let uses_worker_panic_hook = uses_spawn_cpu
        || uses_join_set_spawn_cpu
        || uses_task_scope_spawn_cpu
        || stdlib_preamble.contains("__sifr_with_silent_worker_panic_hook");
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
    if uses_task_scope || uses_join_set || uses_failure_type {
        referenced_error_classes.insert("SecondaryError".to_string());
    }
    if uses_async_generator_type {
        referenced_error_classes.insert("GeneratorCloseError".to_string());
    }
    if uses_spawn_cpu || uses_join_set_spawn_cpu || uses_task_scope_spawn_cpu {
        referenced_error_classes.insert("WorkerRuntimeError".to_string());
        referenced_error_classes.insert("WorkerError".to_string());
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
    if uses_task_scope || uses_join_set || uses_failure_type {
        preamble_items.extend(build_failure_type_items());
    }
    if uses_task_scope || uses_join_set || uses_cancellation_error_type {
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
    if needs_process_status {
        preamble_items.extend(build_process_status_items());
    }
    if needs_process_async {
        preamble_items.extend(build_process_async_items(
            stdlib_needs_process_async.needs_run,
            stdlib_needs_process_async.needs_run_timeout,
            stdlib_needs_process_async.needs_output,
            stdlib_needs_process_async.needs_output_timeout,
            stdlib_needs_process_async.needs_spawn,
            stdlib_needs_process_async.needs_wait,
        ));
    }
    if needs_process_children {
        preamble_items.extend(build_process_child_items());
    }

    // Emit global log level state if logging module is used.
    if needs_logging {
        preamble_items.extend(build_logging_items());
    }
    if needs_random_module_state {
        preamble_items.extend(build_random_module_state_items());
    }
    if uses_task_scope || uses_join_set {
        preamble_items.extend(build_task_scope_items());
    }
    if uses_task_scope_offload {
        preamble_items.extend(build_task_scope_offload_items());
    }
    if uses_task_scope_spawn_cpu {
        preamble_items.extend(build_task_scope_cpu_offload_items());
    }
    if uses_join_set {
        preamble_items.extend(build_join_set_items());
    }
    if uses_worker_panic_hook {
        preamble_items.extend(build_worker_panic_hook_items());
    }
    if uses_join_set_spawn_cpu {
        preamble_items.extend(build_join_set_cpu_items());
    }
    if uses_spawn_cpu {
        preamble_items.extend(build_cpu_offload_items());
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
        || needs_process_children
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
        required_features: {
            let mut features = emitter.intrinsic_registry_features;
            if needs_bigint {
                features.insert(StdlibFeature::NumBigint);
                features.insert(StdlibFeature::NumTraits);
            }
            if needs_decimal {
                features.insert(StdlibFeature::RustDecimal);
            }
            if needs_bigdecimal {
                features.insert(StdlibFeature::BigDecimal);
            }
            if needs_sifr_int {
                features.insert(StdlibFeature::SifrRuntime);
            }
            if has_async_main_entrypoint
                || uses_task_sleep
                || module_uses_task_scope(module)
                || module_uses_join_set(module)
                || stdlib_preamble.contains("tokio::")
            {
                features.insert(StdlibFeature::Tokio);
            }
            if stdlib_preamble.contains("rayon::") {
                features.insert(StdlibFeature::Rayon);
            }
            if uses_spawn_cpu
                || module_uses_join_set_spawn_cpu(module)
                || module_uses_task_scope_spawn_cpu(module)
            {
                features.insert(StdlibFeature::Rayon);
            }
            features
        },
        constant_mappings: emitter.module_constants,
        lowering_stats: emitter.lowering_stats,
    }
}
