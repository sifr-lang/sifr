//! Sifr Compiler Driver
//!
//! Orchestrates the full compilation pipeline:
//! parse -> type-check/HIR -> codegen -> build
//!
//! Stdlib `.sifr` files are embedded in the compiler binary via `include_str!`.
//! They are compiled before user code (two-phase compilation).

use sifr_codegen::{
    generate_project, generate_project_with_deps_and_crates, generate_rust_multi,
    generate_rust_test, generate_rust_with_metadata, generate_rust_with_stdlib, StdlibCode,
};
use sifr_hir::{
    lower_module_stdlib_with_externals, lower_module_with_externals, ExternalDefs, HirModule,
    LoweringResult,
};
use sifr_python_ast::Stmt;
use sifr_python_parser::parse_module;
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub use sifr_codegen::LoweringStats;

/// Embedded stdlib `.sifr` files.
/// Each entry is (`module_name`, `source_code`).
/// Module names use dotted notation (e.g., "sifr.test").
const STDLIB_FILES: &[(&str, &str)] = &[
    // Tier 1: Modules with no inter-stdlib dependencies
    ("sifr.test", include_str!("../../../lib/sifr/test.sifr")),
    ("sifr.env", include_str!("../../../lib/sifr/env.sifr")),
    ("sifr.bytes", include_str!("../../../lib/sifr/bytes.sifr")),
    ("sifr.base64", include_str!("../../../lib/sifr/base64.sifr")),
    ("sifr.math", include_str!("../../../lib/sifr/math.sifr")),
    (
        "sifr.hashlib",
        include_str!("../../../lib/sifr/hashlib.sifr"),
    ),
    ("sifr.io", include_str!("../../../lib/sifr/io.sifr")),
    ("sifr.os", include_str!("../../../lib/sifr/os.sifr")),
    ("sifr.json", include_str!("../../../lib/sifr/json.sifr")),
    ("sifr.time", include_str!("../../../lib/sifr/time.sifr")),
    ("sifr.random", include_str!("../../../lib/sifr/random.sifr")),
    ("sifr.re", include_str!("../../../lib/sifr/re.sifr")),
    (
        "sifr.collections",
        include_str!("../../../lib/sifr/collections.sifr"),
    ),
    ("sifr.string", include_str!("../../../lib/sifr/string.sifr")),
    ("sifr.bisect", include_str!("../../../lib/sifr/bisect.sifr")),
    (
        "sifr.functools",
        include_str!("../../../lib/sifr/functools.sifr"),
    ),
    (
        "sifr.secrets",
        include_str!("../../../lib/sifr/secrets.sifr"),
    ),
    (
        "sifr.graphlib",
        include_str!("../../../lib/sifr/graphlib.sifr"),
    ),
    ("sifr.uuid", include_str!("../../../lib/sifr/uuid.sifr")),
    (
        "sifr.platform",
        include_str!("../../../lib/sifr/platform.sifr"),
    ),
    (
        "sifr.pathlib",
        include_str!("../../../lib/sifr/pathlib.sifr"),
    ),
    (
        "sifr.logging",
        include_str!("../../../lib/sifr/logging.sifr"),
    ),
    ("sifr.heapq", include_str!("../../../lib/sifr/heapq.sifr")),
    (
        "sifr.itertools",
        include_str!("../../../lib/sifr/itertools.sifr"),
    ),
    (
        "sifr.textwrap",
        include_str!("../../../lib/sifr/textwrap.sifr"),
    ),
    ("sifr.csv", include_str!("../../../lib/sifr/csv.sifr")),
    (
        "sifr.argparse",
        include_str!("../../../lib/sifr/argparse.sifr"),
    ),
    (
        "sifr.fnmatch",
        include_str!("../../../lib/sifr/fnmatch.sifr"),
    ),
    ("sifr.shutil", include_str!("../../../lib/sifr/shutil.sifr")),
    (
        "sifr.tempfile",
        include_str!("../../../lib/sifr/tempfile.sifr"),
    ),
    (
        "sifr.difflib",
        include_str!("../../../lib/sifr/difflib.sifr"),
    ),
    (
        "sifr.ipaddress",
        include_str!("../../../lib/sifr/ipaddress.sifr"),
    ),
    ("sifr.timeit", include_str!("../../../lib/sifr/timeit.sifr")),
    (
        "sifr.tomllib",
        include_str!("../../../lib/sifr/tomllib.sifr"),
    ),
    (
        "sifr.datetime",
        include_str!("../../../lib/sifr/datetime.sifr"),
    ),
    (
        "sifr.operator",
        include_str!("../../../lib/sifr/operator.sifr"),
    ),
    (
        "sifr.calendar",
        include_str!("../../../lib/sifr/calendar.sifr"),
    ),
    ("sifr.html", include_str!("../../../lib/sifr/html.sifr")),
    ("sifr.sys", include_str!("../../../lib/sifr/sys.sifr")),
    (
        "sifr.subprocess",
        include_str!("../../../lib/sifr/subprocess.sifr"),
    ),
    ("sifr.gzip", include_str!("../../../lib/sifr/gzip.sifr")),
    (
        "sifr.zipfile",
        include_str!("../../../lib/sifr/zipfile.sifr"),
    ),
    (
        "sifr.configparser",
        include_str!("../../../lib/sifr/configparser.sifr"),
    ),
    // Tier 2: Modules that depend on other stdlib modules
    (
        "sifr.statistics",
        include_str!("../../../lib/sifr/statistics.sifr"),
    ),
    ("sifr.glob", include_str!("../../../lib/sifr/glob.sifr")),
];

/// Result of compiling all stdlib modules.
#[derive(Clone)]
struct StdlibCompiled {
    /// Type information for type checking user code
    defs: ExternalDefs,
    /// Compiled Rust code and intrinsic name tracking for codegen
    code: StdlibCode,
}

// Process-local stdlib compilation cache. This intentionally avoids cross-process
// persistence and de-duplicates repeated stdlib compilation within a single run.
static STDLIB_COMPILED_CACHE: OnceLock<Result<StdlibCompiled, Vec<CompileError>>> = OnceLock::new();

fn write_stderr_line(message: &str) {
    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(stderr, "{message}");
}

fn write_stderr(message: &str) {
    let mut stderr = std::io::stderr().lock();
    let _ = write!(stderr, "{message}");
}

fn intrinsic_constant_rust_expr(module: &str, name: &str) -> Option<&'static str> {
    match (module, name) {
        ("_sifr.math", "pi") => Some("std::f64::consts::PI"),
        ("_sifr.math", "e") => Some("std::f64::consts::E"),
        ("_sifr.math", "tau") => Some("std::f64::consts::TAU"),
        ("_sifr.math", "inf") => Some("f64::INFINITY"),
        ("_sifr.math", "nan") => Some("f64::NAN"),
        _ => None,
    }
}

/// Compile all embedded stdlib `.sifr` files and return their exports as `ExternalDefs`
/// along with compiled Rust code for pure Sifr modules.
/// Stdlib files can import from `_sifr.*` intrinsics (resolved via the intrinsic registry).
fn compile_stdlib() -> Result<StdlibCompiled, Vec<CompileError>> {
    get_or_init_stdlib_cache(&STDLIB_COMPILED_CACHE, compile_stdlib_uncached)
}

fn get_or_init_stdlib_cache(
    cache: &OnceLock<Result<StdlibCompiled, Vec<CompileError>>>,
    build: impl FnOnce() -> Result<StdlibCompiled, Vec<CompileError>>,
) -> Result<StdlibCompiled, Vec<CompileError>> {
    cache.get_or_init(build).clone()
}

fn compile_stdlib_uncached() -> Result<StdlibCompiled, Vec<CompileError>> {
    let mut stdlib_defs = ExternalDefs::default();
    let mut stdlib_code = StdlibCode::default();

    for (module_name, source) in STDLIB_FILES {
        let parsed = match parse_module(source) {
            Ok(parsed) => {
                if !parsed.is_valid() {
                    let errors: Vec<CompileError> = parsed
                        .errors()
                        .iter()
                        .map(|e| CompileError {
                            message: format!("[stdlib:{module_name}] {e}"),
                            phase: CompilePhase::Parse,
                        })
                        .collect();
                    return Err(errors);
                }
                parsed
            }
            Err(e) => {
                return Err(vec![CompileError {
                    message: format!("[stdlib:{module_name}] failed to parse: {e}"),
                    phase: CompilePhase::Parse,
                }]);
            }
        };

        // Lower the stdlib module (allows _sifr.* intrinsic imports + inter-stdlib deps)
        let result = match lower_module_stdlib_with_externals(parsed.suite(), &stdlib_defs) {
            Ok(result) => result,
            Err(errors) => {
                let compile_errors: Vec<CompileError> = errors
                    .into_iter()
                    .map(|e| CompileError {
                        message: format!("[stdlib:{}] {}", module_name, e.message),
                        phase: CompilePhase::TypeCheck,
                    })
                    .collect();
                return Err(compile_errors);
            }
        };

        // Track which names are intrinsic re-exports vs pure Sifr definitions
        let mut intrinsic_names_for_module = HashSet::new();
        // Track transitive intrinsic module dependencies
        let mut transitive_deps_for_module = HashSet::new();

        // Collect exports for this stdlib module
        let mut fn_exports = HashMap::new();
        let mut class_exports = HashMap::new();
        let mut class_type_param_exports = HashMap::new();

        // Collect functions defined in the module (pure Sifr functions)
        for func in &result.module.functions {
            if !func.name.starts_with('_') {
                let params: Vec<(String, Type, ParamConvention)> = func
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                    .collect();
                fn_exports.insert(
                    func.name.clone(),
                    FunctionType {
                        params,
                        return_type: Box::new(func.return_type.clone()),
                    },
                );
            }
        }

        // Collect re-exported functions and constants from _sifr.* intrinsic imports
        let mut const_exports = HashMap::new();
        for import in &result.module.imports {
            if import.module.starts_with("_sifr.") {
                transitive_deps_for_module.insert(import.module.clone());
                if let Some(intrinsic_mod) = sifr_hir::stdlib::get_intrinsic_module(&import.module)
                {
                    for name in &import.names {
                        if let Some(ft) = intrinsic_mod.functions.get(name) {
                            // Pure Sifr functions declared in the module should shadow
                            // imported intrinsic names (e.g. generic wrappers in sifr.test).
                            if !fn_exports.contains_key(name) {
                                fn_exports.insert(name.clone(), ft.clone());
                                intrinsic_names_for_module.insert(name.clone());
                            }
                        }
                        if let Some(const_ty) = intrinsic_mod.constants.get(name) {
                            const_exports.insert(name.clone(), const_ty.clone());
                            intrinsic_names_for_module.insert(name.clone());
                            if let Some(rust_expr) =
                                intrinsic_constant_rust_expr(&import.module, name)
                            {
                                stdlib_code
                                    .module_constants
                                    .entry(import.module.clone())
                                    .or_default()
                                    .insert(
                                        name.clone(),
                                        (const_ty.clone(), rust_expr.to_string()),
                                    );
                            }
                        }
                    }
                }
            } else if import.module.starts_with("sifr.") {
                // Inter-stdlib dependency: also include the transitive deps of the imported module
                transitive_deps_for_module.insert(import.module.clone());
                if let Some(deps) = stdlib_code.transitive_deps.get(&import.module) {
                    transitive_deps_for_module.extend(deps.iter().cloned());
                }
            }
        }

        // Collect module-level constants defined in the .sifr file
        for (name, ty, _expr) in &result.module.constants {
            if !name.starts_with('_') {
                const_exports.insert(name.clone(), ty.clone());
                // Note: NOT added to intrinsic_names — these are pure Sifr constants
            }
        }

        for class in &result.module.classes {
            if !class.name.starts_with('_') {
                let mut methods: Vec<(String, FunctionType)> = class
                    .methods
                    .iter()
                    .map(|m| {
                        let params: Vec<(String, Type, ParamConvention)> = m
                            .params
                            .iter()
                            .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                            .collect();
                        (
                            m.name.clone(),
                            FunctionType {
                                params,
                                return_type: Box::new(m.return_type.clone()),
                            },
                        )
                    })
                    .collect();
                // Include operator dunder methods so imported classes support operator overloading
                for (dunder_name, op_func) in &class.operator_impls {
                    let params: Vec<(String, Type, ParamConvention)> = op_func
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                        .collect();
                    methods.push((
                        dunder_name.clone(),
                        FunctionType {
                            params,
                            return_type: Box::new(op_func.return_type.clone()),
                        },
                    ));
                }
                let class_ty = Type::Class {
                    name: class.name.clone(),
                    fields: class.fields.clone(),
                    methods,
                    parent_class: None,
                };
                class_exports.insert(class.name.clone(), class_ty);
                if !class.type_params.is_empty() {
                    class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
                }
                // Track error types for cross-module import resolution
                if class.is_error_type {
                    stdlib_defs.error_types.insert(class.name.clone());
                }
            }
        }

        // Generate Rust code for this stdlib module (for pure Sifr functions/constants)
        // Only generate if the module has functions or constants defined in .sifr
        let has_pure_sifr_code = !result.module.functions.is_empty()
            || !result.module.constants.is_empty()
            || !result.module.classes.is_empty();
        if has_pure_sifr_code {
            // Use the existing codegen to compile the stdlib module's HIR to Rust
            // Pass a stdlib_code without module_rust_code to avoid embedding transitive deps
            // (each module's code should only contain its own functions, not its deps')
            let codegen_stdlib = StdlibCode {
                module_rust_code: HashMap::new(),
                intrinsic_names: stdlib_code.intrinsic_names.clone(),
                module_constants: stdlib_code.module_constants.clone(),
                func_signatures: stdlib_code.func_signatures.clone(),
                transitive_deps: stdlib_code.transitive_deps.clone(),
                generator_functions: stdlib_code.generator_functions.clone(),
                generic_classes: stdlib_code.generic_classes.clone(),
            };
            let codegen_result =
                sifr_codegen::generate_rust_with_stdlib(&result.module, &codegen_stdlib);
            stdlib_code
                .module_rust_code
                .insert((*module_name).to_string(), codegen_result.rust_source);
            // Store constant mappings so user code can reference them with correct Rust names
            if !codegen_result.constant_mappings.is_empty() {
                stdlib_code
                    .module_constants
                    .insert((*module_name).to_string(), codegen_result.constant_mappings);
            }
            // Store function signatures for pure Sifr functions (for borrow convention at call sites)
            let mut sig_map = HashMap::new();
            for func in &result.module.functions {
                if !func.name.starts_with('_') {
                    let param_info: Vec<(Type, ParamConvention)> = func
                        .params
                        .iter()
                        .map(|p| (p.ty.clone(), p.convention))
                        .collect();
                    sig_map.insert(func.name.clone(), (param_info, func.return_type.clone()));
                }
            }
            // Store class method signatures (ClassName::method -> params, return_type)
            for class in &result.module.classes {
                let mut has_constructor = false;
                for method in &class.methods {
                    let param_info: Vec<(Type, ParamConvention)> = method
                        .params
                        .iter()
                        .map(|p| {
                            let conv = if method.name == "new" {
                                ParamConvention::Own
                            } else {
                                p.convention
                            };
                            (p.ty.clone(), conv)
                        })
                        .collect();
                    sig_map.insert(
                        format!("{}::{}", class.name, method.name),
                        (param_info, method.return_type.clone()),
                    );
                    if method.name == "new" {
                        has_constructor = true;
                    }
                }
                if !has_constructor {
                    let ctor_params = class
                        .fields
                        .iter()
                        .map(|(_, ty)| (ty.clone(), ParamConvention::Own))
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
            if !sig_map.is_empty() {
                stdlib_code
                    .func_signatures
                    .insert((*module_name).to_string(), sig_map);
            }

            // Track generator functions (contain yield statements) for .collect() at call sites
            let mut gen_fns = HashSet::new();
            for func in &result.module.functions {
                if !func.name.starts_with('_') && sifr_codegen::body_contains_yield(&func.body) {
                    gen_fns.insert(func.name.clone());
                }
            }
            if !gen_fns.is_empty() {
                stdlib_code
                    .generator_functions
                    .insert((*module_name).to_string(), gen_fns);
            }

            // Track generic classes for correct type annotation skipping in user code
            for class in &result.module.classes {
                if !class.type_params.is_empty() {
                    stdlib_code.generic_classes.insert(class.name.clone());
                }
            }
        }

        stdlib_code
            .intrinsic_names
            .insert((*module_name).to_string(), intrinsic_names_for_module);
        if !transitive_deps_for_module.is_empty() {
            stdlib_code
                .transitive_deps
                .insert((*module_name).to_string(), transitive_deps_for_module);
        }

        stdlib_defs
            .functions
            .insert((*module_name).to_string(), fn_exports);
        stdlib_defs
            .classes
            .insert((*module_name).to_string(), class_exports);
        if !class_type_param_exports.is_empty() {
            stdlib_defs
                .class_type_params
                .insert((*module_name).to_string(), class_type_param_exports);
        }
        if !const_exports.is_empty() {
            stdlib_defs
                .constants
                .insert((*module_name).to_string(), const_exports);
        }
        if !result.module.generic_functions.is_empty() {
            stdlib_defs.generic_functions.insert(
                (*module_name).to_string(),
                result.module.generic_functions.clone(),
            );
        }
        if !result.module.type_param_bounds.is_empty() {
            stdlib_defs.type_param_bounds.insert(
                (*module_name).to_string(),
                result.module.type_param_bounds.clone(),
            );
        }
    }

    Ok(StdlibCompiled {
        defs: stdlib_defs,
        code: stdlib_code,
    })
}

/// Result of compilation.
#[derive(Debug)]
pub enum CompileResult {
    /// Compilation succeeded, contains generated Rust source.
    Success { rust_source: String },
    /// Compilation failed with errors.
    Errors { errors: Vec<CompileError> },
}

/// A compilation error with location info.
#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub phase: CompilePhase,
}

#[derive(Debug, Clone)]
pub enum CompilePhase {
    Parse,
    TypeCheck,
    Codegen,
    Build,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let phase = match self.phase {
            CompilePhase::Parse => "parse error",
            CompilePhase::TypeCheck => "type error",
            CompilePhase::Codegen => "codegen error",
            CompilePhase::Build => "build error",
        };
        write!(f, "{}: {}", phase, self.message)
    }
}

/// Compile Sifr source code to Rust source code.
pub fn compile(source: &str) -> CompileResult {
    let result = compile_with_metadata(source);
    match result {
        CompileResultFull::Success { rust_source, .. } => CompileResult::Success { rust_source },
        CompileResultFull::Errors { errors } => CompileResult::Errors { errors },
    }
}

/// Full compilation result including stdlib metadata.
pub enum CompileResultFull {
    Success {
        rust_source: String,
        used_stdlib_modules: HashSet<String>,
        required_crates: HashSet<String>,
        lowering_stats: sifr_codegen::LoweringStats,
    },
    Errors {
        errors: Vec<CompileError>,
    },
}

struct FrontendCompiled {
    stdlib: StdlibCompiled,
    lowering_result: LoweringResult,
}

#[derive(Default)]
struct FrontendModuleDiagnostics {
    reveal_types: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Clone, Copy)]
enum FrontendDiagnosticStyle {
    /// Preserve raw frontend diagnostic messages (single-file mode).
    Bare,
    /// Prefix diagnostics with `[module]` for multi-module/project contexts.
    ModulePrefixed,
}

fn compile_frontend(source: &str) -> Result<FrontendCompiled, Vec<CompileError>> {
    // Phase 0: Compile embedded stdlib .sifr files
    let stdlib_compiled = compile_stdlib()?;

    // Phase 1: Parse
    let parsed = match parse_module(source) {
        Ok(parsed) => {
            if !parsed.is_valid() {
                let errors: Vec<CompileError> = parsed
                    .errors()
                    .iter()
                    .map(|e| CompileError {
                        message: format!("{e}"),
                        phase: CompilePhase::Parse,
                    })
                    .collect();
                return Err(errors);
            }
            parsed
        }
        Err(e) => {
            return Err(vec![CompileError {
                message: format!("failed to parse: {e}"),
                phase: CompilePhase::Parse,
            }]);
        }
    };

    // Phase 2: Lower through the same module-orchestration path used by project/test modes.
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert("main".to_string(), parsed.into_suite());
    let mut project_lowering = compile_frontend_modules(
        &parsed_modules,
        stdlib_compiled.defs.clone(),
        FrontendDiagnosticStyle::Bare,
    )?;
    let main_module = project_lowering.hir_modules.remove("main").ok_or_else(|| {
        vec![CompileError {
            message: "internal error: frontend lowering missing 'main' module".to_string(),
            phase: CompilePhase::TypeCheck,
        }]
    })?;
    let main_diag = project_lowering
        .module_diagnostics
        .remove("main")
        .unwrap_or_default();
    let lowering_result = LoweringResult {
        module: main_module,
        reveal_types: main_diag.reveal_types,
        warnings: main_diag.warnings,
    };

    Ok(FrontendCompiled {
        stdlib: stdlib_compiled,
        lowering_result,
    })
}

fn emit_frontend_diagnostics(lowering_result: &LoweringResult) {
    // Print reveal_type diagnostics to stderr
    for diag in &lowering_result.reveal_types {
        write_stderr_line(diag);
    }

    // Print compiler warnings to stderr
    for warning in &lowering_result.warnings {
        write_stderr_line(warning);
    }
}

/// Compile Sifr source code to Rust source code, returning stdlib metadata.
pub fn compile_with_metadata(source: &str) -> CompileResultFull {
    let frontend = match compile_frontend(source) {
        Ok(frontend) => frontend,
        Err(errors) => return CompileResultFull::Errors { errors },
    };

    emit_frontend_diagnostics(&frontend.lowering_result);

    // Phase 3: Generate Rust code with stdlib code
    let codegen_result =
        generate_rust_with_stdlib(&frontend.lowering_result.module, &frontend.stdlib.code);

    CompileResultFull::Success {
        rust_source: codegen_result.rust_source,
        used_stdlib_modules: codegen_result.used_stdlib_modules,
        required_crates: codegen_result.required_crates,
        lowering_stats: codegen_result.lowering_stats,
    }
}

/// Type-check only (no code generation).
pub fn check(source: &str) -> Vec<CompileError> {
    match compile_frontend(source) {
        Ok(frontend) => {
            emit_frontend_diagnostics(&frontend.lowering_result);
            vec![]
        }
        Err(errors) => errors,
    }
}

fn lower_frontend_module(
    module_name: &str,
    stmts: &[sifr_python_ast::Stmt],
    external_defs: &ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<LoweringResult, Vec<CompileError>> {
    let result = match lower_module_with_externals(stmts, external_defs) {
        Ok(result) => result,
        Err(errors) => {
            let compile_errors: Vec<CompileError> = errors
                .into_iter()
                .map(|e| CompileError {
                    message: match diagnostic_style {
                        FrontendDiagnosticStyle::Bare => e.message,
                        FrontendDiagnosticStyle::ModulePrefixed => {
                            format!("[{}] {}", module_name, e.message)
                        }
                    },
                    phase: CompilePhase::TypeCheck,
                })
                .collect();
            return Err(compile_errors);
        }
    };
    Ok(result)
}

fn collect_module_exports(module_name: &str, module: &HirModule, external_defs: &mut ExternalDefs) {
    let mut fn_exports = HashMap::new();
    let mut class_exports = HashMap::new();
    let mut class_type_param_exports = HashMap::new();
    let mut const_exports = HashMap::new();

    for func in &module.functions {
        if !func.name.starts_with('_') {
            let params: Vec<(String, Type, ParamConvention)> = func
                .params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                .collect();
            fn_exports.insert(
                func.name.clone(),
                FunctionType {
                    params,
                    return_type: Box::new(func.return_type.clone()),
                },
            );
        }
    }

    for class in &module.classes {
        if !class.name.starts_with('_') {
            // Extract method types from the class
            let mut methods: Vec<(String, FunctionType)> = class
                .methods
                .iter()
                .filter(|m| m.name != "new") // Skip constructor
                .map(|m| {
                    let params: Vec<(String, Type, ParamConvention)> = m
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                        .collect();
                    (
                        m.name.clone(),
                        FunctionType {
                            params,
                            return_type: Box::new(m.return_type.clone()),
                        },
                    )
                })
                .collect();
            // Include operator dunder methods so imported classes support operator overloading
            for (dunder_name, op_func) in &class.operator_impls {
                let params: Vec<(String, Type, ParamConvention)> = op_func
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                    .collect();
                methods.push((
                    dunder_name.clone(),
                    FunctionType {
                        params,
                        return_type: Box::new(op_func.return_type.clone()),
                    },
                ));
            }
            let class_ty = Type::Class {
                name: class.name.clone(),
                fields: class.fields.clone(),
                methods,
                parent_class: None,
            };
            class_exports.insert(class.name.clone(), class_ty);
            if !class.type_params.is_empty() {
                class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
            }
        }
    }

    for (name, ty, _) in &module.constants {
        if !name.starts_with('_') {
            const_exports.insert(name.clone(), ty.clone());
        }
    }

    external_defs
        .functions
        .insert(module_name.to_string(), fn_exports);
    external_defs
        .classes
        .insert(module_name.to_string(), class_exports);
    if !class_type_param_exports.is_empty() {
        external_defs
            .class_type_params
            .insert(module_name.to_string(), class_type_param_exports);
    }
    external_defs
        .constants
        .insert(module_name.to_string(), const_exports);
}

struct ProjectLowering {
    hir_modules: HashMap<String, HirModule>,
    external_defs: ExternalDefs,
    compile_order: Vec<String>,
    module_diagnostics: HashMap<String, FrontendModuleDiagnostics>,
}

struct ModuleDependencyGraph {
    dependencies: BTreeMap<String, BTreeSet<String>>,
    reverse_dependencies: BTreeMap<String, BTreeSet<String>>,
}

fn collect_local_module_dependencies(
    stmts: &[Stmt],
    local_modules: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    for stmt in stmts {
        let Stmt::ImportFrom(import_from) = stmt else {
            continue;
        };
        if import_from.level > 1 {
            continue;
        }
        let Some(module) = &import_from.module else {
            continue;
        };
        let module_name = module.to_string();
        if module_name == "typing"
            || module_name == "enum"
            || module_name.starts_with("sifr.")
            || module_name.starts_with("_sifr.")
        {
            continue;
        }
        if local_modules.contains(&module_name) {
            deps.insert(module_name);
        }
    }
    deps
}

fn build_module_dependency_graph(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
) -> ModuleDependencyGraph {
    let local_modules: BTreeSet<String> = parsed_modules.keys().cloned().collect();
    let mut dependencies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for module_name in &local_modules {
        let module_deps = parsed_modules
            .get(module_name)
            .map(|stmts| collect_local_module_dependencies(stmts, &local_modules))
            .unwrap_or_default();
        dependencies.insert(module_name.clone(), module_deps);
    }

    let mut reverse_dependencies: BTreeMap<String, BTreeSet<String>> = local_modules
        .iter()
        .cloned()
        .map(|name| (name, BTreeSet::new()))
        .collect();
    for (module_name, deps) in &dependencies {
        for dep in deps {
            if let Some(reverse_deps) = reverse_dependencies.get_mut(dep) {
                reverse_deps.insert(module_name.clone());
            }
        }
    }

    ModuleDependencyGraph {
        dependencies,
        reverse_dependencies,
    }
}

fn find_dependency_cycle_path(
    dependencies: &BTreeMap<String, BTreeSet<String>>,
) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Unvisited,
        Visiting,
        Done,
    }

    fn dfs(
        node: &str,
        dependencies: &BTreeMap<String, BTreeSet<String>>,
        states: &mut BTreeMap<String, VisitState>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        states.insert(node.to_string(), VisitState::Visiting);
        stack.push(node.to_string());

        if let Some(neighbors) = dependencies.get(node) {
            for neighbor in neighbors {
                match states
                    .get(neighbor.as_str())
                    .copied()
                    .unwrap_or(VisitState::Unvisited)
                {
                    VisitState::Unvisited => {
                        if let Some(cycle) = dfs(neighbor, dependencies, states, stack) {
                            return Some(cycle);
                        }
                    }
                    VisitState::Visiting => {
                        if let Some(start_idx) = stack.iter().position(|entry| entry == neighbor) {
                            let mut cycle = stack[start_idx..].to_vec();
                            cycle.push(neighbor.clone());
                            return Some(cycle);
                        }
                    }
                    VisitState::Done => {}
                }
            }
        }

        let _ = stack.pop();
        states.insert(node.to_string(), VisitState::Done);
        None
    }

    let mut states: BTreeMap<String, VisitState> = dependencies
        .keys()
        .cloned()
        .map(|node| (node, VisitState::Unvisited))
        .collect();
    let mut stack = Vec::new();

    for node in dependencies.keys() {
        if states
            .get(node.as_str())
            .copied()
            .unwrap_or(VisitState::Unvisited)
            == VisitState::Unvisited
        {
            if let Some(cycle) = dfs(node, dependencies, &mut states, &mut stack) {
                return Some(cycle);
            }
        }
    }

    None
}

fn compute_module_compile_order(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
) -> Result<Vec<String>, Vec<CompileError>> {
    let graph = build_module_dependency_graph(parsed_modules);
    let mut indegree: BTreeMap<String, usize> = graph
        .dependencies
        .iter()
        .map(|(module_name, deps)| (module_name.clone(), deps.len()))
        .collect();
    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(module_name, _)| module_name.clone())
        .collect();
    let mut compile_order = Vec::with_capacity(indegree.len());

    while let Some(module_name) = ready.iter().next().cloned() {
        ready.remove(&module_name);
        compile_order.push(module_name.clone());
        if let Some(dependents) = graph.reverse_dependencies.get(&module_name) {
            for dependent in dependents {
                if let Some(degree) = indegree.get_mut(dependent) {
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        ready.insert(dependent.clone());
                    }
                }
            }
        }
    }

    if compile_order.len() == indegree.len() {
        return Ok(compile_order);
    }

    let cycle_path = find_dependency_cycle_path(&graph.dependencies)
        .unwrap_or_else(|| vec!["<cycle>".to_string()]);
    let cycle_render = cycle_path.join(" -> ");
    let edge_render = cycle_path
        .windows(2)
        .map(|edge| format!("{} imports {}", edge[0], edge[1]))
        .collect::<Vec<_>>()
        .join(", ");
    let message = format!(
        "module dependency cycle detected: {cycle_render}; import chain: {edge_render}. Break the cycle by moving shared declarations into a separate module."
    );
    Err(vec![CompileError {
        message,
        phase: CompilePhase::TypeCheck,
    }])
}

fn ordered_non_main_module_names(
    compile_order: &[String],
    rust_files: &HashMap<String, String>,
) -> Vec<String> {
    compile_order
        .iter()
        .filter(|module_name| module_name.as_str() != "main")
        .filter(|module_name| rust_files.contains_key(module_name.as_str()))
        .cloned()
        .collect()
}

fn assemble_project_main_rs(
    compile_order: &[String],
    rust_files: &HashMap<String, String>,
) -> String {
    let mut main_rs = String::new();
    let ordered_non_main = ordered_non_main_module_names(compile_order, rust_files);
    for module_name in &ordered_non_main {
        main_rs.push_str("mod ");
        main_rs.push_str(module_name);
        main_rs.push_str(";\n");
    }
    if !ordered_non_main.is_empty() && rust_files.contains_key("main") {
        main_rs.push('\n');
    }
    if let Some(main_code) = rust_files.get("main") {
        main_rs.push_str(main_code);
    }
    main_rs
}

fn compile_frontend_modules(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
    mut external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<ProjectLowering, Vec<CompileError>> {
    let mut hir_modules: HashMap<String, HirModule> = HashMap::new();
    let mut module_diagnostics: HashMap<String, FrontendModuleDiagnostics> = HashMap::new();
    let compile_order = compute_module_compile_order(parsed_modules)?;

    for module_name in &compile_order {
        let Some(stmts) = parsed_modules.get(module_name.as_str()) else {
            return Err(vec![CompileError {
                message: format!("[{module_name}] module was not parsed"),
                phase: CompilePhase::Build,
            }]);
        };
        let result = lower_frontend_module(module_name, stmts, &external_defs, diagnostic_style)?;
        let LoweringResult {
            module,
            reveal_types,
            warnings,
        } = result;
        collect_module_exports(module_name, &module, &mut external_defs);
        hir_modules.insert(module_name.clone(), module);
        module_diagnostics.insert(
            module_name.clone(),
            FrontendModuleDiagnostics {
                reveal_types,
                warnings,
            },
        );
    }

    Ok(ProjectLowering {
        hir_modules,
        external_defs,
        compile_order,
        module_diagnostics,
    })
}

fn collect_project_hir_modules(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
    external_defs: ExternalDefs,
) -> Result<ProjectLowering, Vec<CompileError>> {
    compile_frontend_modules(
        parsed_modules,
        external_defs,
        FrontendDiagnosticStyle::ModulePrefixed,
    )
}

fn discover_project_sifr_files(project_dir: &Path) -> Vec<PathBuf> {
    let mut sifr_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sifr") {
                sifr_files.push(path);
            }
        }
    }
    sifr_files.sort();
    sifr_files
}

fn read_project_sources(project_dir: &Path) -> Result<Vec<(String, String)>, Vec<CompileError>> {
    let sifr_files = discover_project_sifr_files(project_dir);
    let mut sources: Vec<(String, String)> = Vec::with_capacity(sifr_files.len());
    for file in &sifr_files {
        let module_name = file.file_stem().unwrap().to_string_lossy().to_string();
        let source = std::fs::read_to_string(file).map_err(|e| {
            vec![CompileError {
                message: format!("failed to read '{}': {}", file.display(), e),
                phase: CompilePhase::Build,
            }]
        })?;
        sources.push((module_name, source));
    }
    Ok(sources)
}

fn parse_project_sources(
    sources: &[(String, String)],
) -> Result<HashMap<String, Vec<sifr_python_ast::Stmt>>, Vec<CompileError>> {
    let mut parsed_modules: HashMap<String, Vec<sifr_python_ast::Stmt>> = HashMap::new();
    for (module_name, source) in sources {
        let parsed = match parse_module(source) {
            Ok(parsed) => {
                if !parsed.is_valid() {
                    let errors: Vec<CompileError> = parsed
                        .errors()
                        .iter()
                        .map(|e| CompileError {
                            message: format!("[{module_name}] {e}"),
                            phase: CompilePhase::Parse,
                        })
                        .collect();
                    return Err(errors);
                }
                parsed
            }
            Err(e) => {
                return Err(vec![CompileError {
                    message: format!("[{module_name}] failed to parse: {e}"),
                    phase: CompilePhase::Parse,
                }]);
            }
        };
        parsed_modules.insert(module_name.clone(), parsed.into_suite());
    }
    Ok(parsed_modules)
}

fn analyze_project_frontend(main_file: &Path) -> Result<ProjectLowering, Vec<CompileError>> {
    let project_dir = main_file.parent().unwrap_or(Path::new("."));
    let sources = read_project_sources(project_dir)?;
    let parsed_modules = parse_project_sources(&sources)?;
    let stdlib_compiled = compile_stdlib()?;
    collect_project_hir_modules(&parsed_modules, stdlib_compiled.defs)
}

fn emit_project_frontend_diagnostics(project_lowering: &ProjectLowering) {
    for module_name in &project_lowering.compile_order {
        let Some(diag) = project_lowering
            .module_diagnostics
            .get(module_name.as_str())
        else {
            continue;
        };
        for message in &diag.reveal_types {
            write_stderr_line(message);
        }
        for warning in &diag.warnings {
            write_stderr_line(warning);
        }
    }
}

/// Compile a multi-file project and build a native binary.
/// `main_file` is the path to the main .sifr file. Other .sifr files in the same
/// directory are compiled as modules.
pub fn build_project(main_file: &Path, output_dir: &Path) -> Result<PathBuf, Vec<CompileError>> {
    let project_lowering = analyze_project_frontend(main_file)?;
    emit_project_frontend_diagnostics(&project_lowering);
    let hir_modules = project_lowering.hir_modules;
    let compile_order = project_lowering.compile_order;

    // Phase 3: Generate Rust code
    let module_refs: Vec<(&str, &HirModule)> = compile_order
        .iter()
        .filter_map(|module_name| {
            hir_modules
                .get(module_name)
                .map(|module| (module_name.as_str(), module))
        })
        .collect();
    let rust_files = generate_rust_multi(&module_refs);

    // Phase 4: Build the Rust project
    let project_path = output_dir.join("sifr_output");
    let src_dir = project_path.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        vec![CompileError {
            message: format!("failed to create output directory: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write Cargo.toml
    let (cargo_toml, _) = generate_project(
        &HirModule {
            functions: vec![],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        },
        "sifr_output",
    );
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write main.rs using deterministic module declaration ordering.
    let main_rs = assemble_project_main_rs(&compile_order, &rust_files);

    std::fs::write(src_dir.join("main.rs"), &main_rs).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write main.rs: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write non-main module files in deterministic dependency-safe order.
    for module_name in ordered_non_main_module_names(&compile_order, &rust_files) {
        let Some(code) = rust_files.get(module_name.as_str()) else {
            continue;
        };
        std::fs::write(src_dir.join(format!("{module_name}.rs")), code).map_err(|e| {
            vec![CompileError {
                message: format!("failed to write {module_name}.rs: {e}"),
                phase: CompilePhase::Build,
            }]
        })?;
    }

    // Run cargo build
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&project_path)
        .output()
        .map_err(|e| {
            vec![CompileError {
                message: format!("failed to run cargo build: {e}"),
                phase: CompilePhase::Build,
            }]
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(vec![CompileError {
            message: format!("cargo build failed:\n{stderr}"),
            phase: CompilePhase::Build,
        }]);
    }

    let binary_name = if cfg!(target_os = "windows") {
        "sifr_output.exe"
    } else {
        "sifr_output"
    };
    let binary_path = project_path
        .join("target")
        .join("release")
        .join(binary_name);

    Ok(binary_path)
}

/// Type-check a multi-file project entrypoint without code generation.
pub fn check_project(main_file: &Path) -> Vec<CompileError> {
    match analyze_project_frontend(main_file) {
        Ok(project_lowering) => {
            emit_project_frontend_diagnostics(&project_lowering);
            vec![]
        }
        Err(errors) => errors,
    }
}

/// Compile and build a native binary.
pub fn build(source: &str, output_dir: &Path) -> Result<PathBuf, Vec<CompileError>> {
    let (rust_source, used_stdlib_modules, required_crates) = match compile_with_metadata(source) {
        CompileResultFull::Success {
            rust_source,
            used_stdlib_modules,
            required_crates,
            lowering_stats: _,
        } => (rust_source, used_stdlib_modules, required_crates),
        CompileResultFull::Errors { errors } => return Err(errors),
    };

    // Create a temporary Rust project
    let project_dir = output_dir.join("sifr_output");
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        vec![CompileError {
            message: format!("failed to create output directory: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write Cargo.toml with stdlib + explicit required crates from codegen metadata.
    let (cargo_toml, _) = generate_project_with_deps_and_crates(
        &sifr_hir::HirModule {
            functions: vec![],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        },
        "sifr_output",
        &used_stdlib_modules,
        &required_crates,
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write main.rs
    std::fs::write(src_dir.join("main.rs"), &rust_source).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write main.rs: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    // Run cargo build
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&project_dir)
        .output()
        .map_err(|e| {
            vec![CompileError {
                message: format!("failed to run cargo build: {e}"),
                phase: CompilePhase::Build,
            }]
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(vec![CompileError {
            message: format!("cargo build failed:\n{stderr}"),
            phase: CompilePhase::Build,
        }]);
    }

    // Return path to the built binary
    let binary_name = if cfg!(target_os = "windows") {
        "sifr_output.exe"
    } else {
        "sifr_output"
    };
    let binary_path = project_dir.join("target").join("release").join(binary_name);

    Ok(binary_path)
}

/// Discover and run tests in a directory.
/// Finds all `test_*.sifr` and `*_test.sifr` files, compiles them with
/// `#[test]` attributes, and runs `cargo test`.
pub fn run_tests(test_dir: &Path) -> Result<bool, Vec<CompileError>> {
    // Discover test files and support modules from the same directory.
    let mut test_files: Vec<PathBuf> = Vec::new();
    let mut support_modules: HashMap<String, Vec<sifr_python_ast::Stmt>> = HashMap::new();
    for path in discover_project_sifr_files(test_dir) {
        let module_name = path.file_stem().unwrap().to_string_lossy().to_string();
        let source = std::fs::read_to_string(&path).map_err(|e| {
            vec![CompileError {
                message: format!("failed to read '{}': {}", path.display(), e),
                phase: CompilePhase::Build,
            }]
        })?;
        let parsed = match parse_module(&source) {
            Ok(parsed) => {
                if !parsed.is_valid() {
                    let errors: Vec<CompileError> = parsed
                        .errors()
                        .iter()
                        .map(|e| CompileError {
                            message: format!("[{}] {}", path.display(), e),
                            phase: CompilePhase::Parse,
                        })
                        .collect();
                    return Err(errors);
                }
                parsed
            }
            Err(e) => {
                return Err(vec![CompileError {
                    message: format!("[{}] failed to parse: {}", path.display(), e),
                    phase: CompilePhase::Parse,
                }]);
            }
        };

        if module_name.starts_with("test_") || module_name.ends_with("_test") {
            test_files.push(path);
        } else {
            support_modules.insert(module_name, parsed.into_suite());
        }
    }
    test_files.sort();

    if test_files.is_empty() {
        write_stderr_line(&format!("No test files found in {}", test_dir.display()));
        return Ok(true);
    }

    write_stderr_line(&format!("Found {} test file(s)", test_files.len()));

    // Build project externals from non-test modules so test imports resolve like regular builds.
    let stdlib_compiled = compile_stdlib()?;
    let project_lowering = collect_project_hir_modules(&support_modules, stdlib_compiled.defs)?;
    let project_externals = project_lowering.external_defs.clone();
    let mut support_module_names: Vec<String> = project_lowering
        .hir_modules
        .keys()
        .filter(|name| name.as_str() != "main")
        .cloned()
        .collect();
    support_module_names.sort();
    let support_module_refs: Vec<(&str, &HirModule)> = support_module_names
        .iter()
        .filter_map(|name| {
            project_lowering
                .hir_modules
                .get(name)
                .map(|module| (name.as_str(), module))
        })
        .collect();
    let support_rust_files = generate_rust_multi(&support_module_refs);

    // Compile each test file and combine into a single Rust test binary
    let mut all_rust_code = String::new();
    let mut all_stdlib_modules = HashSet::new();
    let mut all_required_crates = HashSet::new();

    for module_name in &support_module_names {
        if let Some(module) = project_lowering.hir_modules.get(module_name) {
            let support_codegen = generate_rust_with_metadata(module);
            all_stdlib_modules.extend(support_codegen.used_stdlib_modules);
            all_required_crates.extend(support_codegen.required_crates);
        }
    }

    for test_file in &test_files {
        let module_name = test_file.file_stem().unwrap().to_string_lossy().to_string();
        let source = std::fs::read_to_string(test_file).map_err(|e| {
            vec![CompileError {
                message: format!("failed to read '{}': {}", test_file.display(), e),
                phase: CompilePhase::Build,
            }]
        })?;
        let parsed = parse_module(&source).map_err(|e| {
            vec![CompileError {
                message: format!("[{}] failed to parse: {}", test_file.display(), e),
                phase: CompilePhase::Parse,
            }]
        })?;
        if !parsed.is_valid() {
            let errors: Vec<CompileError> = parsed
                .errors()
                .iter()
                .map(|e| CompileError {
                    message: format!("[{}] {}", test_file.display(), e),
                    phase: CompilePhase::Parse,
                })
                .collect();
            return Err(errors);
        }

        // Lower to HIR
        let lowering_result = match lower_frontend_module(
            &module_name,
            parsed.suite(),
            &project_externals,
            FrontendDiagnosticStyle::Bare,
        ) {
            Ok(result) => result,
            Err(errors) => {
                let compile_errors: Vec<CompileError> = errors
                    .into_iter()
                    .map(|e| CompileError {
                        message: format!("[{}] {}", test_file.display(), e.message),
                        phase: CompilePhase::TypeCheck,
                    })
                    .collect();
                return Err(compile_errors);
            }
        };

        // Generate Rust code in test mode
        let codegen_result = generate_rust_test(&lowering_result.module);
        all_rust_code.push_str("// Tests from: ");
        all_rust_code.push_str(&test_file.file_name().unwrap().to_string_lossy());
        all_rust_code.push('\n');
        all_rust_code.push_str(&codegen_result.rust_source);
        all_rust_code.push('\n');
        all_stdlib_modules.extend(codegen_result.used_stdlib_modules);
        all_required_crates.extend(codegen_result.required_crates);
    }

    // Build and run with cargo test
    let project_dir = std::env::temp_dir().join("sifr_test_runner");
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        vec![CompileError {
            message: format!("failed to create test directory: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write Cargo.toml with stdlib + explicit required crates from codegen metadata.
    let cargo_toml = generate_test_runner_cargo_toml(&all_stdlib_modules, &all_required_crates);

    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    for module_name in &support_module_names {
        if let Some(code) = support_rust_files.get(module_name) {
            std::fs::write(src_dir.join(format!("{module_name}.rs")), code).map_err(|e| {
                vec![CompileError {
                    message: format!("failed to write {module_name}.rs: {e}"),
                    phase: CompilePhase::Build,
                }]
            })?;
        }
    }

    let test_lib = compose_test_runner_lib(&support_module_names, &all_rust_code);

    // Write the test source file as lib.rs (so cargo test finds #[test] functions)
    std::fs::write(src_dir.join("lib.rs"), &test_lib).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write lib.rs: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    // Run cargo test
    let output = Command::new("cargo")
        .args(["test"])
        .current_dir(&project_dir)
        .output()
        .map_err(|e| {
            vec![CompileError {
                message: format!("failed to run cargo test: {e}"),
                phase: CompilePhase::Build,
            }]
        })?;

    // Forward stdout and stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        write_stderr(&stdout);
    }
    if !stderr.is_empty() {
        write_stderr(&stderr);
    }

    Ok(output.status.success())
}

fn compose_test_runner_lib(support_module_names: &[String], all_rust_code: &str) -> String {
    // Keep generated helper modules and imports test-scoped so cargo test's non-test
    // library build does not emit irrelevant dead-code/unused warnings.
    let mut test_lib = String::from("#![cfg(test)]\n\n");
    for module_name in support_module_names {
        test_lib.push_str("mod ");
        test_lib.push_str(module_name);
        test_lib.push_str(";\n");
    }
    if !support_module_names.is_empty() {
        test_lib.push('\n');
    }
    test_lib.push_str(all_rust_code);
    test_lib
}

fn generate_test_runner_cargo_toml(
    stdlib_modules: &HashSet<String>,
    required_crates: &HashSet<String>,
) -> String {
    let (cargo_toml, _) = generate_project_with_deps_and_crates(
        &HirModule {
            functions: vec![],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        },
        "sifr_tests",
        stdlib_modules,
        required_crates,
    );
    cargo_toml
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn parse_suite(source: &str) -> Vec<sifr_python_ast::Stmt> {
        let parsed = parse_module(source).unwrap_or_else(|e| panic!("parse failed: {e}"));
        assert!(
            parsed.is_valid(),
            "invalid test source: {:?}",
            parsed.errors()
        );
        parsed.into_suite()
    }

    #[test]
    fn test_compile_hello_world() {
        let source = r#"
def main():
    print("Hello, World!")
"#;
        match compile(source) {
            CompileResult::Success { rust_source } => {
                assert!(rust_source.contains("fn main()"));
                assert!(rust_source.contains("println!"));
                assert!(rust_source.contains("Hello, World!"));
            }
            CompileResult::Errors { errors } => {
                panic!("compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_compile_factorial() {
        let source = r#"
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    x: int = factorial(5)
    print(x)
"#;
        match compile(source) {
            CompileResult::Success { rust_source } => {
                assert!(rust_source.contains("fn factorial(n: i64) -> i64"));
                assert!(rust_source.contains("fn main()"));
            }
            CompileResult::Errors { errors } => {
                panic!("compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_type_mismatch_error() {
        let source = r#"
def main():
    x: int = "hello"
"#;
        let errors = check(source);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("type mismatch")));
    }

    #[test]
    fn test_check_only_reports_frontend_phases() {
        let source = r#"
def main():
    x: int = "hello"
"#;
        let errors = check(source);
        assert!(!errors.is_empty());
        assert!(errors
            .iter()
            .all(|e| matches!(e.phase, CompilePhase::Parse | CompilePhase::TypeCheck)));
    }

    #[test]
    fn test_check_valid_program() {
        let source = r#"
def main():
    x: int = 42
    print(x)
"#;
        let errors = check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_reports_unsupported_multi_level_relative_import() {
        let source = r#"
from ..helper import value

def main():
    print(value())
"#;
        let errors = check(source);
        assert!(errors
            .iter()
            .any(|e| e.message.contains("unsupported relative import level 2")));
    }

    #[test]
    fn test_check_reports_unsupported_bare_relative_import() {
        let source = r#"
from . import helper

def main():
    print(helper)
"#;
        let errors = check(source);
        assert!(errors
            .iter()
            .any(|e| e.message.contains("unsupported bare relative import")));
    }

    #[test]
    fn test_check_reports_unsupported_import_statement() {
        let source = r#"
import helper

def main():
    print("ok")
"#;
        let errors = check(source);
        assert!(errors.iter().any(|e| e
            .message
            .contains("unsupported import statement 'import helper'")));
    }

    #[test]
    fn test_compile_frontend_modules_uses_explicit_diagnostic_style() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
def main():
    print(missing_name)
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let bare_errors = compile_frontend_modules(
            &parsed_modules,
            stdlib_defs.clone(),
            FrontendDiagnosticStyle::Bare,
        )
        .err()
        .expect("bare diagnostic style should still report type errors");
        let prefixed_errors = compile_frontend_modules(
            &parsed_modules,
            stdlib_defs,
            FrontendDiagnosticStyle::ModulePrefixed,
        )
        .err()
        .expect("module-prefixed diagnostic style should report type errors");

        assert!(bare_errors
            .iter()
            .any(|e| !e.message.starts_with("[main] ")));
        assert!(prefixed_errors
            .iter()
            .all(|e| e.message.starts_with("[main] ")));
    }

    #[test]
    fn test_check_and_project_lowering_share_typecheck_contract() {
        let source = r#"
def main():
    print(unknown_symbol)
"#;
        let check_errors = check(source);
        assert!(!check_errors.is_empty(), "check should report type errors");

        let mut parsed_modules = HashMap::new();
        parsed_modules.insert("main".to_string(), parse_suite(source));
        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let project_errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .err()
            .expect("project lowering should report same frontend type errors");

        let check_messages: Vec<String> = check_errors.into_iter().map(|e| e.message).collect();
        let normalized_project_messages: Vec<String> = project_errors
            .into_iter()
            .map(|e| {
                e.message
                    .strip_prefix("[main] ")
                    .unwrap_or(&e.message)
                    .to_string()
            })
            .collect();
        assert_eq!(check_messages, normalized_project_messages);
    }

    #[test]
    fn test_get_or_init_stdlib_cache_reuses_successful_compilation() {
        let cache: OnceLock<Result<StdlibCompiled, Vec<CompileError>>> = OnceLock::new();
        let build_calls = AtomicUsize::new(0);

        let first = get_or_init_stdlib_cache(&cache, || {
            build_calls.fetch_add(1, Ordering::SeqCst);
            compile_stdlib_uncached()
        })
        .expect("initial stdlib compilation should succeed");
        let second = get_or_init_stdlib_cache(&cache, || {
            build_calls.fetch_add(1, Ordering::SeqCst);
            panic!("stdlib cache should not rebuild on second lookup");
        })
        .expect("cached stdlib compilation should be reused");

        assert_eq!(build_calls.load(Ordering::SeqCst), 1);
        assert_eq!(first.defs.functions.len(), second.defs.functions.len());
        assert_eq!(
            first.code.module_rust_code.len(),
            second.code.module_rust_code.len()
        );
    }

    #[test]
    fn test_get_or_init_stdlib_cache_reuses_error_without_fallback_rebuild() {
        let cache: OnceLock<Result<StdlibCompiled, Vec<CompileError>>> = OnceLock::new();
        let build_calls = AtomicUsize::new(0);

        let first = match get_or_init_stdlib_cache(&cache, || {
            build_calls.fetch_add(1, Ordering::SeqCst);
            Err(vec![CompileError {
                message: "sentinel stdlib cache error".to_string(),
                phase: CompilePhase::Build,
            }])
        }) {
            Ok(_) => panic!("sentinel error should be cached"),
            Err(errors) => errors,
        };
        let second = match get_or_init_stdlib_cache(&cache, || {
            build_calls.fetch_add(1, Ordering::SeqCst);
            compile_stdlib_uncached()
        }) {
            Ok(_) => panic!("cached error should be reused"),
            Err(errors) => errors,
        };

        assert_eq!(build_calls.load(Ordering::SeqCst), 1);
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].message, "sentinel stdlib cache error");
    }

    #[test]
    fn test_collect_project_modules_supports_single_level_relative_import() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from .helper import value

def main():
    print(value())
"#,
            ),
        );
        parsed_modules.insert(
            "helper".to_string(),
            parse_suite(
                r#"
def value() -> int:
    return 42
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .expect("single-level relative imports should resolve in project lowering");
        assert!(result.hir_modules.contains_key("main"));
        assert!(result.hir_modules.contains_key("helper"));
    }

    #[test]
    fn test_collect_project_modules_allows_non_main_stdlib_imports() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from helper import area_like

def main():
    print(area_like(2.0))
"#,
            ),
        );
        parsed_modules.insert(
            "helper".to_string(),
            parse_suite(
                r#"
from sifr.math import pi

def area_like(r: float) -> float:
    return r * pi
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .expect("project lowering should resolve non-main stdlib imports");
        assert!(result.hir_modules.contains_key("main"));
        assert!(result.hir_modules.contains_key("helper"));
    }

    #[test]
    fn test_collect_project_modules_resolves_non_main_local_dependencies() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from a_consumer import fetch

def main():
    print(fetch())
"#,
            ),
        );
        parsed_modules.insert(
            "a_consumer".to_string(),
            parse_suite(
                r#"
from z_provider import value

def fetch() -> int:
    return value()
"#,
            ),
        );
        parsed_modules.insert(
            "z_provider".to_string(),
            parse_suite(
                r#"
def value() -> int:
    return 41
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .expect("project lowering should resolve non-main local imports");
        assert!(result.hir_modules.contains_key("main"));
        assert!(result.hir_modules.contains_key("a_consumer"));
        assert!(result.hir_modules.contains_key("z_provider"));
    }

    #[test]
    fn test_compute_module_compile_order_is_dependency_safe() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from consumer import value

def main():
    print(value())
"#,
            ),
        );
        parsed_modules.insert(
            "consumer".to_string(),
            parse_suite(
                r#"
from provider import value_provider

def value() -> int:
    return value_provider()
"#,
            ),
        );
        parsed_modules.insert(
            "provider".to_string(),
            parse_suite(
                r#"
def value_provider() -> int:
    return 42
"#,
            ),
        );

        let order = compute_module_compile_order(&parsed_modules)
            .expect("compile order should be computed for acyclic graph");
        assert_eq!(
            order,
            vec![
                "provider".to_string(),
                "consumer".to_string(),
                "main".to_string()
            ]
        );
    }

    #[test]
    fn test_assemble_project_main_rs_is_deterministic_against_hashmap_order() {
        let compile_order = vec![
            "provider".to_string(),
            "consumer".to_string(),
            "main".to_string(),
        ];

        let mut rust_files_a = HashMap::new();
        rust_files_a.insert("main".to_string(), "fn main() {}\n".to_string());
        rust_files_a.insert("consumer".to_string(), "pub fn c() {}\n".to_string());
        rust_files_a.insert("provider".to_string(), "pub fn p() {}\n".to_string());

        let mut rust_files_b = HashMap::new();
        rust_files_b.insert("provider".to_string(), "pub fn p() {}\n".to_string());
        rust_files_b.insert("main".to_string(), "fn main() {}\n".to_string());
        rust_files_b.insert("consumer".to_string(), "pub fn c() {}\n".to_string());

        let main_a = assemble_project_main_rs(&compile_order, &rust_files_a);
        let main_b = assemble_project_main_rs(&compile_order, &rust_files_b);
        assert_eq!(main_a, main_b);
        assert_eq!(main_a, "mod provider;\nmod consumer;\n\nfn main() {}\n");
    }

    #[test]
    fn test_collect_project_modules_reports_unknown_module_in_non_main() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from helper import get

def main():
    print(get())
"#,
            ),
        );
        parsed_modules.insert(
            "helper".to_string(),
            parse_suite(
                r#"
from missing_mod import value

def get() -> int:
    return value()
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .err()
            .expect("project lowering should fail when non-main imports missing module");
        assert!(errors
            .iter()
            .any(|e| e.message.contains("unknown module 'missing_mod'")));
    }

    #[test]
    fn test_collect_project_modules_cycle_reports_error() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from a import value_a

def main():
    print(value_a())
"#,
            ),
        );
        parsed_modules.insert(
            "a".to_string(),
            parse_suite(
                r#"
from b import value_b

def value_a() -> int:
    return value_b()
"#,
            ),
        );
        parsed_modules.insert(
            "b".to_string(),
            parse_suite(
                r#"
from a import value_a

def value_b() -> int:
    return value_a()
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .err()
            .expect("project lowering should fail when there is a dependency cycle");
        assert!(errors
            .iter()
            .any(|e| e.message.contains("module dependency cycle detected")));
        assert!(errors.iter().any(|e| e.message.contains("a -> b -> a")));
    }

    #[test]
    fn test_collect_project_modules_exports_local_constants() {
        let mut parsed_modules = HashMap::new();
        parsed_modules.insert(
            "main".to_string(),
            parse_suite(
                r#"
from consumer import get

def main():
    print(get())
"#,
            ),
        );
        parsed_modules.insert(
            "consumer".to_string(),
            parse_suite(
                r#"
from constants_mod import ANSWER

def get() -> int:
    return ANSWER
"#,
            ),
        );
        parsed_modules.insert(
            "constants_mod".to_string(),
            parse_suite(
                r#"
ANSWER: int = 42
"#,
            ),
        );

        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
            .expect("project lowering should resolve local constant imports");
        let constants = result
            .external_defs
            .constants
            .get("constants_mod")
            .expect("constants module exports should exist");
        assert_eq!(constants.get("ANSWER"), Some(&Type::Int));
    }

    #[test]
    fn test_run_tests_resolves_local_imports_and_constants() {
        let unique = format!(
            "sifr_test_import_parity_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let test_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&test_dir).expect("test dir should be created");

        std::fs::write(
            test_dir.join("helper.sifr"),
            r#"
BASE: int = 9

def plus_one(x: int) -> int:
    return x + 1
"#,
        )
        .expect("helper module should be written");
        std::fs::write(
            test_dir.join("test_imports.sifr"),
            r#"
from helper import BASE, plus_one

def test_import_parity():
    assert plus_one(BASE) == 10
"#,
        )
        .expect("test module should be written");

        let result = run_tests(&test_dir).expect("test runner should compile and execute");
        assert!(result, "sifr test run should succeed");

        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_run_tests_reports_deterministic_parse_error_order() {
        let unique = format!(
            "sifr_test_parse_order_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let test_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&test_dir).expect("test dir should be created");

        std::fs::write(test_dir.join("z_bad.sifr"), "def z(:\n").expect("z_bad should be written");
        std::fs::write(test_dir.join("a_bad.sifr"), "def a(:\n").expect("a_bad should be written");
        std::fs::write(
            test_dir.join("test_smoke.sifr"),
            "def test_smoke():\n    assert True\n",
        )
        .expect("test file should be written");

        let first_messages: Vec<String> = run_tests(&test_dir)
            .err()
            .expect("parse errors should be reported")
            .into_iter()
            .map(|e| e.message)
            .collect();
        let second_messages: Vec<String> = run_tests(&test_dir)
            .err()
            .expect("parse errors should be deterministic")
            .into_iter()
            .map(|e| e.message)
            .collect();

        assert_eq!(first_messages, second_messages);
        assert!(
            first_messages
                .first()
                .is_some_and(|m| m.contains("a_bad.sifr")),
            "first parse error should be from lexicographically first fixture: {first_messages:?}"
        );

        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_run_tests_frontend_type_errors_use_single_path_prefix() {
        let unique = format!(
            "sifr_test_type_error_prefix_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let test_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&test_dir).expect("test dir should be created");

        std::fs::write(
            test_dir.join("helper.sifr"),
            "def value() -> int:\n    return 1\n",
        )
        .expect("helper should be written");
        std::fs::write(
            test_dir.join("test_bad.sifr"),
            "from helper import value\n\ndef test_bad() -> int:\n    return \"bad\"\n",
        )
        .expect("bad test module should be written");

        let errors = run_tests(&test_dir)
            .err()
            .expect("type errors in test module should fail frontend");
        let messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        assert!(messages.iter().all(|m| m.contains("test_bad.sifr")));
        assert!(messages
            .iter()
            .all(|m| !m.contains("] [test_bad] return type mismatch")));

        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_check_project_resolves_valid_local_imports() {
        let unique = format!(
            "sifr_check_project_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("project dir should be created");
        std::fs::write(
            dir.join("main.sifr"),
            r#"
from helper import area

def main():
    print(area(2.0))
"#,
        )
        .expect("main module should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            r#"
from sifr.math import pi

def area(radius: float) -> float:
    return pi * radius * radius
"#,
        )
        .expect("helper module should be written");

        let errors = check_project(&dir.join("main.sifr"));
        assert!(
            errors.is_empty(),
            "check_project should succeed: {errors:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_project_error_messages_match_build_project() {
        let unique = format!(
            "sifr_check_project_error_parity_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("project dir should be created");
        std::fs::write(
            dir.join("main.sifr"),
            r#"
from helper import broken

def main():
    print(broken())
"#,
        )
        .expect("main module should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            r#"
def broken() -> int:
    return "bad"
"#,
        )
        .expect("helper module should be written");

        let check_errors = check_project(&dir.join("main.sifr"));
        let build_errors = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
            .err()
            .expect("build_project should fail with same frontend error");

        let check_messages: Vec<String> = check_errors.into_iter().map(|e| e.to_string()).collect();
        let build_messages: Vec<String> = build_errors.into_iter().map(|e| e.to_string()).collect();
        assert_eq!(check_messages, build_messages);
        assert!(build_messages
            .iter()
            .any(|m| m.contains("[helper] return type mismatch")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_generate_test_runner_cargo_toml_includes_required_crates() {
        let stdlib_modules = HashSet::new();
        let required_crates = HashSet::from([
            "regex".to_string(),
            "rand".to_string(),
            "rand_distr".to_string(),
        ]);

        let cargo_toml = generate_test_runner_cargo_toml(&stdlib_modules, &required_crates);
        assert!(cargo_toml.contains("name = \"sifr_tests\""));
        assert!(cargo_toml.contains("regex = \"1\""));
        assert!(cargo_toml.contains("rand = \"0.8\""));
        assert!(cargo_toml.contains("rand_distr = \"0.4\""));
    }

    #[test]
    fn test_generate_test_runner_cargo_toml_preserves_stdlib_deps() {
        let stdlib_modules = HashSet::from(["sifr.json".to_string()]);
        let required_crates = HashSet::new();

        let cargo_toml = generate_test_runner_cargo_toml(&stdlib_modules, &required_crates);
        assert!(cargo_toml.contains("serde_json = \"1\""));
        assert!(cargo_toml.contains("serde = { version = \"1\", features = [\"derive\"] }"));
    }

    #[test]
    fn test_compose_test_runner_lib_is_test_scoped() {
        let support_modules = vec!["helper".to_string()];
        let all_rust_code = "#[test]\nfn smoke() {}\n";
        let lib_source = compose_test_runner_lib(&support_modules, all_rust_code);
        assert!(lib_source.starts_with("#![cfg(test)]"));
        assert!(lib_source.contains("mod helper;"));
        assert!(lib_source.contains("#[test]\nfn smoke() {}"));
    }
}
