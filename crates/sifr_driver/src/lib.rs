//! Sifr Compiler Driver
//!
//! Orchestrates the full compilation pipeline:
//! parse -> type-check/HIR -> codegen -> build
//!
//! Stdlib `.sifr` files are embedded in the compiler binary via `include_str!`.
//! They are compiled before user code (two-phase compilation).

use sifr_python_parser::parse_module;
use sifr_hir::{lower_module, lower_module_with_externals, lower_module_stdlib_with_externals, ExternalDefs, HirModule};
use sifr_codegen::{generate_rust_with_stdlib, generate_rust_test, generate_rust_multi, generate_project, generate_project_with_deps, StdlibCode};
use sifr_type_system::{Type, FunctionType, ParamConvention};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Embedded stdlib `.sifr` files.
/// Each entry is (module_name, source_code).
/// Module names use dotted notation (e.g., "sifr.test").
const STDLIB_FILES: &[(&str, &str)] = &[
    // Tier 1: Modules with no inter-stdlib dependencies
    ("sifr.test", include_str!("../../../lib/sifr/test.sifr")),
    ("sifr.env", include_str!("../../../lib/sifr/env.sifr")),
    ("sifr.bytes", include_str!("../../../lib/sifr/bytes.sifr")),
    ("sifr.base64", include_str!("../../../lib/sifr/base64.sifr")),
    ("sifr.math", include_str!("../../../lib/sifr/math.sifr")),
    ("sifr.hashlib", include_str!("../../../lib/sifr/hashlib.sifr")),
    ("sifr.io", include_str!("../../../lib/sifr/io.sifr")),
    ("sifr.os", include_str!("../../../lib/sifr/os.sifr")),
    ("sifr.json", include_str!("../../../lib/sifr/json.sifr")),
    ("sifr.time", include_str!("../../../lib/sifr/time.sifr")),
    ("sifr.random", include_str!("../../../lib/sifr/random.sifr")),
    ("sifr.re", include_str!("../../../lib/sifr/re.sifr")),
    ("sifr.collections", include_str!("../../../lib/sifr/collections.sifr")),
    ("sifr.string", include_str!("../../../lib/sifr/string.sifr")),
    ("sifr.bisect", include_str!("../../../lib/sifr/bisect.sifr")),
    ("sifr.functools", include_str!("../../../lib/sifr/functools.sifr")),
    ("sifr.secrets", include_str!("../../../lib/sifr/secrets.sifr")),
    ("sifr.graphlib", include_str!("../../../lib/sifr/graphlib.sifr")),
    ("sifr.uuid", include_str!("../../../lib/sifr/uuid.sifr")),
    ("sifr.platform", include_str!("../../../lib/sifr/platform.sifr")),
    ("sifr.pathlib", include_str!("../../../lib/sifr/pathlib.sifr")),
    ("sifr.logging", include_str!("../../../lib/sifr/logging.sifr")),
    ("sifr.heapq", include_str!("../../../lib/sifr/heapq.sifr")),
    ("sifr.itertools", include_str!("../../../lib/sifr/itertools.sifr")),
    ("sifr.textwrap", include_str!("../../../lib/sifr/textwrap.sifr")),
    ("sifr.csv", include_str!("../../../lib/sifr/csv.sifr")),
    ("sifr.argparse", include_str!("../../../lib/sifr/argparse.sifr")),
    ("sifr.fnmatch", include_str!("../../../lib/sifr/fnmatch.sifr")),
    ("sifr.shutil", include_str!("../../../lib/sifr/shutil.sifr")),
    ("sifr.tempfile", include_str!("../../../lib/sifr/tempfile.sifr")),
    // Tier 2: Modules that depend on other stdlib modules
    ("sifr.statistics", include_str!("../../../lib/sifr/statistics.sifr")),
    ("sifr.glob", include_str!("../../../lib/sifr/glob.sifr")),
];

/// Result of compiling all stdlib modules.
struct StdlibCompiled {
    /// Type information for type checking user code
    defs: ExternalDefs,
    /// Compiled Rust code and intrinsic name tracking for codegen
    code: StdlibCode,
}

/// Compile all embedded stdlib `.sifr` files and return their exports as ExternalDefs
/// along with compiled Rust code for pure Sifr modules.
/// Stdlib files can import from `_sifr.*` intrinsics (resolved via the intrinsic registry).
fn compile_stdlib() -> Result<StdlibCompiled, Vec<CompileError>> {
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
                            message: format!("[stdlib:{}] {}", module_name, e),
                            phase: CompilePhase::Parse,
                        })
                        .collect();
                    return Err(errors);
                }
                parsed
            }
            Err(e) => {
                return Err(vec![CompileError {
                    message: format!("[stdlib:{}] failed to parse: {}", module_name, e),
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

        // Collect functions defined in the module (pure Sifr functions)
        for func in &result.module.functions {
            if !func.name.starts_with('_') {
                let params: Vec<(String, Type, ParamConvention)> = func.params.iter()
                    .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                    .collect();
                fn_exports.insert(func.name.clone(), FunctionType {
                    params,
                    return_type: Box::new(func.return_type.clone()),
                });
            }
        }

        // Collect re-exported functions and constants from _sifr.* intrinsic imports
        let mut const_exports = HashMap::new();
        for import in &result.module.imports {
            if import.module.starts_with("_sifr.") {
                transitive_deps_for_module.insert(import.module.clone());
                if let Some(intrinsic_mod) = sifr_hir::stdlib::get_intrinsic_module(&import.module) {
                    for name in &import.names {
                        if let Some(ft) = intrinsic_mod.functions.get(name) {
                            fn_exports.insert(name.clone(), ft.clone());
                            intrinsic_names_for_module.insert(name.clone());
                        }
                        if let Some(const_ty) = intrinsic_mod.constants.get(name) {
                            const_exports.insert(name.clone(), const_ty.clone());
                            intrinsic_names_for_module.insert(name.clone());
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
                let methods: Vec<(String, FunctionType)> = class.methods.iter()
                    .filter(|m| m.name != "new")
                    .map(|m| {
                        let params: Vec<(String, Type, ParamConvention)> = m.params.iter()
                            .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                            .collect();
                        (m.name.clone(), FunctionType {
                            params,
                            return_type: Box::new(m.return_type.clone()),
                        })
                    })
                    .collect();
                let class_ty = Type::Class {
                    name: class.name.clone(),
                    fields: class.fields.clone(),
                    methods,
                };
                class_exports.insert(class.name.clone(), class_ty);
            }
        }

        // Generate Rust code for this stdlib module (for pure Sifr functions/constants)
        // Only generate if the module has functions or constants defined in .sifr
        let has_pure_sifr_code = !result.module.functions.is_empty() || !result.module.constants.is_empty();
        if has_pure_sifr_code {
            // Use the existing codegen to compile the stdlib module's HIR to Rust
            // Pass the current stdlib_code so that inter-stdlib intrinsic dispatch works correctly
            let codegen_result = sifr_codegen::generate_rust_with_stdlib(&result.module, &stdlib_code);
            stdlib_code.module_rust_code.insert(module_name.to_string(), codegen_result.rust_source);
            // Store constant mappings so user code can reference them with correct Rust names
            if !codegen_result.constant_mappings.is_empty() {
                stdlib_code.module_constants.insert(module_name.to_string(), codegen_result.constant_mappings);
            }
            // Store function signatures for pure Sifr functions (for borrow convention at call sites)
            let mut sig_map = HashMap::new();
            for func in &result.module.functions {
                if !func.name.starts_with('_') {
                    let param_info: Vec<(Type, ParamConvention)> = func.params.iter()
                        .map(|p| (p.ty.clone(), p.convention))
                        .collect();
                    sig_map.insert(func.name.clone(), (param_info, func.return_type.clone()));
                }
            }
            if !sig_map.is_empty() {
                stdlib_code.func_signatures.insert(module_name.to_string(), sig_map);
            }
        }

        stdlib_code.intrinsic_names.insert(module_name.to_string(), intrinsic_names_for_module);
        if !transitive_deps_for_module.is_empty() {
            stdlib_code.transitive_deps.insert(module_name.to_string(), transitive_deps_for_module);
        }

        stdlib_defs.functions.insert(module_name.to_string(), fn_exports);
        stdlib_defs.classes.insert(module_name.to_string(), class_exports);
        if !const_exports.is_empty() {
            stdlib_defs.constants.insert(module_name.to_string(), const_exports);
        }
    }

    Ok(StdlibCompiled { defs: stdlib_defs, code: stdlib_code })
}

/// Result of compilation.
#[derive(Debug)]
pub enum CompileResult {
    /// Compilation succeeded, contains generated Rust source.
    Success {
        rust_source: String,
    },
    /// Compilation failed with errors.
    Errors {
        errors: Vec<CompileError>,
    },
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
    },
    Errors {
        errors: Vec<CompileError>,
    },
}

/// Compile Sifr source code to Rust source code, returning stdlib metadata.
pub fn compile_with_metadata(source: &str) -> CompileResultFull {
    // Phase 0: Compile embedded stdlib .sifr files
    let stdlib_compiled = match compile_stdlib() {
        Ok(compiled) => compiled,
        Err(errors) => return CompileResultFull::Errors { errors },
    };

    // Phase 1: Parse
    let parsed = match parse_module(source) {
        Ok(parsed) => {
            if !parsed.is_valid() {
                let errors: Vec<CompileError> = parsed
                    .errors()
                    .iter()
                    .map(|e| CompileError {
                        message: format!("{}", e),
                        phase: CompilePhase::Parse,
                    })
                    .collect();
                return CompileResultFull::Errors { errors };
            }
            parsed
        }
        Err(e) => {
            return CompileResultFull::Errors {
                errors: vec![CompileError {
                    message: format!("failed to parse: {}", e),
                    phase: CompilePhase::Parse,
                }],
            };
        }
    };

    // Phase 2: Lower to HIR with stdlib externals (type checking + name resolution)
    let lowering_result = match lower_module_with_externals(parsed.suite(), &stdlib_compiled.defs) {
        Ok(result) => result,
        Err(errors) => {
            let compile_errors: Vec<CompileError> = errors
                .into_iter()
                .map(|e| CompileError {
                    message: e.message,
                    phase: CompilePhase::TypeCheck,
                })
                .collect();
            return CompileResultFull::Errors {
                errors: compile_errors,
            };
        }
    };

    // Print reveal_type diagnostics to stderr
    for diag in &lowering_result.reveal_types {
        eprintln!("{}", diag);
    }

    // Phase 3: Generate Rust code with stdlib code
    let codegen_result = generate_rust_with_stdlib(&lowering_result.module, &stdlib_compiled.code);

    CompileResultFull::Success {
        rust_source: codegen_result.rust_source,
        used_stdlib_modules: codegen_result.used_stdlib_modules,
    }
}

/// Type-check only (no code generation).
pub fn check(source: &str) -> Vec<CompileError> {
    match compile(source) {
        CompileResult::Success { .. } => vec![],
        CompileResult::Errors { errors } => errors,
    }
}

/// Compile a multi-file project and build a native binary.
/// `main_file` is the path to the main .sifr file. Other .sifr files in the same
/// directory are compiled as modules.
pub fn build_project(main_file: &Path, output_dir: &Path) -> Result<PathBuf, Vec<CompileError>> {
    let project_dir = main_file.parent().unwrap_or(Path::new("."));

    // Discover all .sifr files in the project directory
    let mut sifr_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "sifr") {
                sifr_files.push(path);
            }
        }
    }
    sifr_files.sort();

    // Read all source files
    let mut sources: HashMap<String, String> = HashMap::new();
    for file in &sifr_files {
        let module_name = file.file_stem().unwrap().to_string_lossy().to_string();
        let source = std::fs::read_to_string(file).map_err(|e| {
            vec![CompileError {
                message: format!("failed to read '{}': {}", file.display(), e),
                phase: CompilePhase::Build,
            }]
        })?;
        sources.insert(module_name, source);
    }

    // Phase 1: Parse all modules
    let mut parsed_modules: HashMap<String, Vec<sifr_python_ast::Stmt>> = HashMap::new();
    for (module_name, source) in &sources {
        let parsed = match parse_module(source) {
            Ok(parsed) => {
                if !parsed.is_valid() {
                    let errors: Vec<CompileError> = parsed
                        .errors()
                        .iter()
                        .map(|e| CompileError {
                            message: format!("[{}] {}", module_name, e),
                            phase: CompilePhase::Parse,
                        })
                        .collect();
                    return Err(errors);
                }
                parsed
            }
            Err(e) => {
                return Err(vec![CompileError {
                    message: format!("[{}] failed to parse: {}", module_name, e),
                    phase: CompilePhase::Parse,
                }]);
            }
        };
        parsed_modules.insert(module_name.clone(), parsed.into_suite());
    }

    // Phase 1.5: Compile embedded stdlib .sifr files
    let stdlib_compiled = compile_stdlib()?;

    // Phase 2: Lower non-main modules first to collect their exports
    let mut external_defs = stdlib_compiled.defs;
    let mut hir_modules: HashMap<String, HirModule> = HashMap::new();

    // First, lower all non-main modules
    for (module_name, stmts) in &parsed_modules {
        if module_name == "main" {
            continue;
        }
        let result = match lower_module(stmts) {
            Ok(result) => result,
            Err(errors) => {
                let compile_errors: Vec<CompileError> = errors
                    .into_iter()
                    .map(|e| CompileError {
                        message: format!("[{}] {}", module_name, e.message),
                        phase: CompilePhase::TypeCheck,
                    })
                    .collect();
                return Err(compile_errors);
            }
        };

        // Collect exports for this module
        let mut fn_exports = HashMap::new();
        let mut class_exports = HashMap::new();

        for func in &result.module.functions {
            if !func.name.starts_with('_') {
                let params: Vec<(String, Type, ParamConvention)> = func.params.iter()
                    .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                    .collect();
                fn_exports.insert(func.name.clone(), FunctionType {
                    params,
                    return_type: Box::new(func.return_type.clone()),
                });
            }
        }

        for class in &result.module.classes {
            if !class.name.starts_with('_') {
                // Extract method types from the class
                let methods: Vec<(String, FunctionType)> = class.methods.iter()
                    .filter(|m| m.name != "new") // Skip constructor
                    .map(|m| {
                        let params: Vec<(String, Type, ParamConvention)> = m.params.iter()
                            .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                            .collect();
                        (m.name.clone(), FunctionType {
                            params,
                            return_type: Box::new(m.return_type.clone()),
                        })
                    })
                    .collect();
                let class_ty = Type::Class {
                    name: class.name.clone(),
                    fields: class.fields.clone(),
                    methods,
                };
                class_exports.insert(class.name.clone(), class_ty);
            }
        }

        external_defs.functions.insert(module_name.clone(), fn_exports);
        external_defs.classes.insert(module_name.clone(), class_exports);
        hir_modules.insert(module_name.clone(), result.module);
    }

    // Then, lower main module with external definitions
    if let Some(main_stmts) = parsed_modules.get("main") {
        let result = match lower_module_with_externals(main_stmts, &external_defs) {
            Ok(result) => result,
            Err(errors) => {
                let compile_errors: Vec<CompileError> = errors
                    .into_iter()
                    .map(|e| CompileError {
                        message: format!("[main] {}", e.message),
                        phase: CompilePhase::TypeCheck,
                    })
                    .collect();
                return Err(compile_errors);
            }
        };
        hir_modules.insert("main".to_string(), result.module);
    }

    // Phase 3: Generate Rust code
    let module_refs: Vec<(&str, &HirModule)> = hir_modules.iter()
        .map(|(name, module)| (name.as_str(), module))
        .collect();
    let rust_files = generate_rust_multi(&module_refs);

    // Phase 4: Build the Rust project
    let project_path = output_dir.join("sifr_output");
    let src_dir = project_path.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        vec![CompileError {
            message: format!("failed to create output directory: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write Cargo.toml
    let (cargo_toml, _) = generate_project(
        &HirModule { functions: vec![], classes: vec![], imports: vec![], constants: vec![] },
        "sifr_output",
    );
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write main.rs with mod declarations and main module code
    let mut main_rs = String::new();

    // Add mod declarations for non-main modules
    for (module_name, _) in &rust_files {
        if module_name != "main" {
            main_rs.push_str(&format!("mod {};\n", module_name));
        }
    }
    if rust_files.len() > 1 {
        main_rs.push('\n');
    }

    // Add main module code
    if let Some(main_code) = rust_files.get("main") {
        main_rs.push_str(main_code);
    }

    std::fs::write(src_dir.join("main.rs"), &main_rs).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write main.rs: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write non-main module files
    for (module_name, code) in &rust_files {
        if module_name != "main" {
            std::fs::write(src_dir.join(format!("{}.rs", module_name)), code).map_err(|e| {
                vec![CompileError {
                    message: format!("failed to write {}.rs: {}", module_name, e),
                    phase: CompilePhase::Build,
                }]
            })?;
        }
    }

    // Run cargo build
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&project_path)
        .output()
        .map_err(|e| {
            vec![CompileError {
                message: format!("failed to run cargo build: {}", e),
                phase: CompilePhase::Build,
            }]
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(vec![CompileError {
            message: format!("cargo build failed:\n{}", stderr),
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

/// Compile and build a native binary.
pub fn build(source: &str, output_dir: &Path) -> Result<PathBuf, Vec<CompileError>> {
    let (rust_source, used_stdlib_modules) = match compile_with_metadata(source) {
        CompileResultFull::Success { rust_source, used_stdlib_modules } => (rust_source, used_stdlib_modules),
        CompileResultFull::Errors { errors } => return Err(errors),
    };

    // Create a temporary Rust project
    let project_dir = output_dir.join("sifr_output");
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        vec![CompileError {
            message: format!("failed to create output directory: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write Cargo.toml with stdlib dependencies
    let (cargo_toml, _) = generate_project_with_deps(
        &sifr_hir::HirModule { functions: vec![], classes: vec![], imports: vec![], constants: vec![] },
        "sifr_output",
        &used_stdlib_modules,
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write main.rs
    std::fs::write(src_dir.join("main.rs"), &rust_source).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write main.rs: {}", e),
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
                message: format!("failed to run cargo build: {}", e),
                phase: CompilePhase::Build,
            }]
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(vec![CompileError {
            message: format!("cargo build failed:\n{}", stderr),
            phase: CompilePhase::Build,
        }]);
    }

    // Return path to the built binary
    let binary_name = if cfg!(target_os = "windows") {
        "sifr_output.exe"
    } else {
        "sifr_output"
    };
    let binary_path = project_dir
        .join("target")
        .join("release")
        .join(binary_name);

    Ok(binary_path)
}

/// Discover and run tests in a directory.
/// Finds all `test_*.sifr` and `*_test.sifr` files, compiles them with
/// `#[test]` attributes, and runs `cargo test`.
pub fn run_tests(test_dir: &Path) -> Result<bool, Vec<CompileError>> {
    // Discover test files
    let mut test_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(test_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "sifr") {
                let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                if stem.starts_with("test_") || stem.ends_with("_test") {
                    test_files.push(path);
                }
            }
        }
    }
    test_files.sort();

    if test_files.is_empty() {
        eprintln!("No test files found in {}", test_dir.display());
        return Ok(true);
    }

    eprintln!("Found {} test file(s)", test_files.len());

    // Compile each test file and combine into a single Rust test binary
    let mut all_rust_code = String::new();
    let mut all_stdlib_modules = HashSet::new();

    for test_file in &test_files {
        let source = std::fs::read_to_string(test_file).map_err(|e| {
            vec![CompileError {
                message: format!("failed to read '{}': {}", test_file.display(), e),
                phase: CompilePhase::Build,
            }]
        })?;

        // Parse
        let parsed = match parse_module(&source) {
            Ok(parsed) => {
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
                parsed
            }
            Err(e) => {
                return Err(vec![CompileError {
                    message: format!("[{}] failed to parse: {}", test_file.display(), e),
                    phase: CompilePhase::Parse,
                }]);
            }
        };

        // Lower to HIR
        let lowering_result = match lower_module(parsed.suite()) {
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
        all_rust_code.push_str(&format!("// Tests from: {}\n", test_file.file_name().unwrap().to_string_lossy()));
        all_rust_code.push_str(&codegen_result.rust_source);
        all_rust_code.push('\n');
        all_stdlib_modules.extend(codegen_result.used_stdlib_modules);
    }

    // Build and run with cargo test
    let project_dir = std::env::temp_dir().join("sifr_test_runner");
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        vec![CompileError {
            message: format!("failed to create test directory: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write Cargo.toml with dependencies
    let mut cargo_toml = format!(
        r#"[package]
name = "sifr_tests"
version = "0.1.0"
edition = "2021"
"#
    );

    let mut deps = Vec::new();
    for module_name in &all_stdlib_modules {
        match module_name.as_str() {
            "sifr.json" => deps.push("serde_json = \"1\"".to_string()),
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

    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write the test source file as lib.rs (so cargo test finds #[test] functions)
    std::fs::write(src_dir.join("lib.rs"), &all_rust_code).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write lib.rs: {}", e),
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
                message: format!("failed to run cargo test: {}", e),
                phase: CompilePhase::Build,
            }]
        })?;

    // Forward stdout and stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        eprint!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    Ok(output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_check_valid_program() {
        let source = r#"
def main():
    x: int = 42
    print(x)
"#;
        let errors = check(source);
        assert!(errors.is_empty());
    }
}
