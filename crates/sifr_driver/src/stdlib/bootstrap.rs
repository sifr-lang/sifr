use crate::diagnostics::{RenderedDiagnostic, run_codegen_with_boundary};
use crate::export_policy::should_export_callable;
use crate::stdlib::cache::{STDLIB_COMPILED_CACHE, get_or_init_stdlib_cache};
use crate::stdlib::interop::{build_stdlib_rust_interop, pending_private_interop_module};
use crate::stdlib::re_exports::{ReExportMaps, re_export_stdlib_imports};
use crate::stdlib::types::StdlibCompiled;
use sifr_codegen::{StdlibCode, StdlibRustSource};
use sifr_diagnostics::DiagnosticCode;
use sifr_lowering::{
    ExternalDefs, HirFunction, HirParam, canonicalize_user_export_type,
    canonicalize_user_export_type_in_place,
    lower_module_sysroot_private_declaration_with_externals,
    lower_module_sysroot_public_stdlib_with_externals,
};
use sifr_stdlib_manifest::{
    LoadedStdlibSource, LoadedStdlibSourceKind, load_stdlib_tooling_sources_from_sysroot,
};
use sifr_syntax::parse_module_raw;
use sifr_sysroot::ResolvedSysroot;
use sifr_sysroot::sha256_hex;
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(crate) fn compile_stdlib() -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    get_or_init_stdlib_cache(&STDLIB_COMPILED_CACHE, compile_stdlib_uncached)
}

pub fn external_defs() -> Result<ExternalDefs, Vec<RenderedDiagnostic>> {
    compile_stdlib().map(|compiled| compiled.defs)
}

pub(crate) fn compile_stdlib_uncached() -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    let sysroot = sifr_sysroot::resolve_sysroot(None).map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            error.boundary_message(),
            DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
        )]
    })?;
    let sources = load_stdlib_tooling_sources_from_sysroot(&sysroot).map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            format!("Sifr stdlib source inventory is invalid: {error}"),
            DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
        )]
    })?;
    compile_stdlib_sources_with_sysroot(&sources, sysroot)
}

fn compile_stdlib_sources_with_sysroot(
    sources: &[LoadedStdlibSource],
    sysroot: ResolvedSysroot,
) -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    let mut stdlib_defs = ExternalDefs::default();
    let mut stdlib_code = StdlibCode::default();
    let mut private_interop_modules = Vec::new();

    for stdlib_source in sources {
        let module_name = stdlib_source.module.as_str();
        let source_name = stdlib_source.path.display().to_string();
        let parsed = match parse_module_raw(stdlib_source.source.as_str(), Some(&source_name)) {
            Ok(parsed) => {
                if !parsed.has_valid_syntax() {
                    // TODO(diag_4a_parse_failure_classification): classify Ruff parse failures
                    // into the precise active parse-code buckets.
                    let errors: Vec<RenderedDiagnostic> = parsed
                        .errors()
                        .iter()
                        .map(|e| {
                            crate::diagnostics::diagnostic_with_code(
                                format!("[stdlib:{module_name}] {e}"),
                                DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
                            )
                        })
                        .collect();
                    return Err(errors);
                }
                parsed
            }
            Err(errors) => {
                // TODO(diag_4a_parse_failure_classification): classify Ruff parse failures into
                // the precise active parse-code buckets.
                return Err(errors
                    .into_iter()
                    .map(|error| {
                        crate::diagnostics::diagnostic_with_code(
                            format!("[stdlib:{module_name}] {}", error.message),
                            DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
                        )
                    })
                    .collect());
            }
        };
        let mut result = match lower_stdlib_source(stdlib_source, parsed.suite(), &stdlib_defs) {
            Ok(result) => result,
            Err(errors) => {
                let diagnostics: Vec<RenderedDiagnostic> = errors
                    .into_iter()
                    .map(|e| {
                        // Even if `e.code` is `Some(_)`, stdlib lowering
                        // failures collapse to bootstrap failures from the
                        // caller's perspective, not user-facing semantic
                        // diagnostics.
                        crate::diagnostics::diagnostic_with_code(
                            format!("[stdlib:{}] {}", module_name, e.message),
                            DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
                        )
                    })
                    .collect();
                return Err(diagnostics);
            }
        };
        let private_declaration = stdlib_source.kind == LoadedStdlibSourceKind::PrivateDeclaration;
        let local_classes = result
            .module
            .classes
            .iter()
            .map(|class| (class.name.clone(), format!("{module_name}.{}", class.name)))
            .collect::<HashMap<_, _>>();
        canonicalize_stdlib_hir_signatures(&mut result.module, module_name, &local_classes);
        if let Some(module) = pending_private_interop_module(stdlib_source, &result.module) {
            private_interop_modules.push(module);
        }
        if private_declaration
            && result.module.functions.is_empty()
            && result.module.constants.is_empty()
            && result.module.classes.is_empty()
        {
            continue;
        }

        let mut transitive_deps_for_module = HashSet::new();

        let mut fn_exports = HashMap::new();
        let mut compiler_intrinsic_exports = HashMap::new();
        let mut class_exports = HashMap::new();
        let mut error_exports = HashSet::new();
        let mut class_instance_method_exports = HashMap::new();
        let mut class_type_param_exports = HashMap::new();
        let mut default_exports = HashMap::new();
        let mut vararg_exports = HashMap::new();
        let mut workload_exports = HashMap::new();

        for func in &result.module.functions {
            if private_declaration || should_export_callable(module_name, &func.name) {
                fn_exports.insert(func.name.clone(), function_type_from_hir(func));
                if let Some(intrinsic) = func.compiler_intrinsic {
                    compiler_intrinsic_exports.insert(func.name.clone(), intrinsic);
                }
                if let Some(vararg_index) = result.function_varargs.get(&func.name) {
                    vararg_exports.insert(func.name.clone(), *vararg_index);
                }
                if let Some(label) = result.function_workloads.get(&func.name) {
                    workload_exports.insert(func.name.clone(), label.clone());
                }
            }
        }
        for (callable_name, label) in &result.function_workloads {
            let Some((owner_name, _)) = callable_name.split_once('.') else {
                continue;
            };
            if private_declaration || should_export_callable(module_name, owner_name) {
                workload_exports.insert(callable_name.clone(), label.clone());
            }
        }

        for (callable_name, defaults) in &result.function_defaults {
            if private_declaration || should_export_callable(module_name, callable_name) {
                default_exports.insert(callable_name.clone(), defaults.clone());
            }
        }

        let mut const_exports = HashMap::new();
        for import in &result.module.imports {
            if import.module.starts_with("_sifr.") {
                transitive_deps_for_module.insert(import.module.clone());
                let has_compiled_exports = stdlib_defs
                    .functions
                    .get(&import.module)
                    .is_some_and(|exports| !exports.is_empty())
                    || stdlib_defs
                        .classes
                        .get(&import.module)
                        .is_some_and(|exports| !exports.is_empty())
                    || stdlib_defs
                        .constants
                        .get(&import.module)
                        .is_some_and(|exports| !exports.is_empty());
                if has_compiled_exports {
                    let mut exports = ReExportMaps {
                        functions: &mut fn_exports,
                        compiler_intrinsics: &mut compiler_intrinsic_exports,
                        classes: &mut class_exports,
                        error_types: &mut error_exports,
                        class_type_params: &mut class_type_param_exports,
                        defaults: &mut default_exports,
                        varargs: &mut vararg_exports,
                        workloads: &mut workload_exports,
                        constants: &mut const_exports,
                    };
                    re_export_stdlib_imports(
                        &mut exports,
                        &stdlib_defs,
                        module_name,
                        &import.module,
                        &import.names,
                        &import.aliases,
                    );
                }
            } else if import.module.starts_with("sifr.") {
                transitive_deps_for_module.insert(import.module.clone());
                if module_name == "sifr.python" && import.module == "sifr.python_core" {
                    let mut exports = ReExportMaps {
                        functions: &mut fn_exports,
                        compiler_intrinsics: &mut compiler_intrinsic_exports,
                        classes: &mut class_exports,
                        error_types: &mut error_exports,
                        class_type_params: &mut class_type_param_exports,
                        defaults: &mut default_exports,
                        varargs: &mut vararg_exports,
                        workloads: &mut workload_exports,
                        constants: &mut const_exports,
                    };
                    re_export_stdlib_imports(
                        &mut exports,
                        &stdlib_defs,
                        module_name,
                        &import.module,
                        &import.names,
                        &import.aliases,
                    );
                }
                if let Some(deps) = stdlib_code.transitive_deps.get(&import.module) {
                    transitive_deps_for_module.extend(deps.iter().cloned());
                }
            }
        }

        for (name, ty, _expr) in &result.module.constants {
            if private_declaration || !name.starts_with('_') {
                const_exports.insert(name.clone(), ty.clone());
            }
        }
        if !private_declaration {
            fn_exports.retain(|name, _| should_export_callable(module_name, name));
            compiler_intrinsic_exports.retain(|name, _| should_export_callable(module_name, name));
            default_exports.retain(|name, _| should_export_callable(module_name, name));
            vararg_exports.retain(|name, _| should_export_callable(module_name, name));
            workload_exports.retain(|name, _| {
                let owner_name = name
                    .split_once('.')
                    .map_or(name.as_str(), |(owner, _)| owner);
                should_export_callable(module_name, owner_name)
            });
            class_exports.retain(|name, _| should_export_callable(module_name, name));
            error_exports.retain(|name| should_export_callable(module_name, name));
            class_type_param_exports.retain(|name, _| should_export_callable(module_name, name));
            const_exports.retain(|name, _| !name.starts_with('_'));
        }
        let const_integer_value_exports = collect_public_constant_integer_value_exports(
            result.module.constants.iter().filter_map(|(name, _, _)| {
                (private_declaration || !name.starts_with('_')).then_some(name.as_str())
            }),
            &result.constant_integer_values,
        );

        for class in &result.module.classes {
            if private_declaration || !class.name.starts_with('_') {
                class_instance_method_exports.insert(
                    class.name.clone(),
                    class
                        .methods
                        .iter()
                        .filter(|method| {
                            method.name != "new"
                                && method.method_kind == sifr_ir::MethodKind::Regular
                        })
                        .map(|method| method.name.clone())
                        .collect(),
                );
                let mut methods: Vec<(String, FunctionType)> = class
                    .methods
                    .iter()
                    .map(|method| (method.name.clone(), method_type_from_hir(method)))
                    .collect();
                for (dunder_name, op_func) in &class.operator_impls {
                    methods.push((
                        dunder_name.clone(),
                        function_type_from_params(&op_func.params, &op_func.return_type),
                    ));
                }
                let class_ty = canonical_stdlib_type(
                    &Type::Class {
                        identity: None,
                        type_args: class
                            .type_params
                            .iter()
                            .cloned()
                            .map(Type::TypeVar)
                            .collect(),
                        name: class.name.clone(),
                        fields: class.fields.clone(),
                        methods,
                        parent_class: class.semantic_parent_chain(),
                    },
                    &local_classes,
                );
                class_exports.insert(class.name.clone(), class_ty);
                if !class.type_params.is_empty() {
                    class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
                }
                if class.is_error_type {
                    error_exports.insert(class.name.clone());
                }
            }
        }

        let has_pure_sifr_code = !result.module.functions.is_empty()
            || !result.module.constants.is_empty()
            || !result.module.classes.is_empty();
        if has_pure_sifr_code {
            let codegen_stdlib = StdlibCode {
                module_rust_code: HashMap::new(),
                module_constants: stdlib_code.module_constants.clone(),
                func_signatures: stdlib_code.func_signatures.clone(),
                transitive_deps: stdlib_code.transitive_deps.clone(),
                generator_functions: stdlib_code.generator_functions.clone(),
                generic_classes: stdlib_code.generic_classes.clone(),
                generic_class_params: stdlib_code.generic_class_params.clone(),
                generic_class_templates: stdlib_code.generic_class_templates.clone(),
                module_class_fields: stdlib_code.module_class_fields.clone(),
                module_class_templates: select_imported_class_templates(
                    &result.module.imports,
                    &stdlib_code.module_class_templates,
                ),
            };
            let codegen_result = run_codegen_with_boundary(
                format!(
                    "internal compiler panic during stdlib code generation for '{module_name}'"
                ),
                || {
                    sifr_codegen::generate_rust_with_stdlib_for_module(
                        &result.module,
                        &codegen_stdlib,
                        Some(module_name),
                    )
                },
            )
            .map_err(|e| {
                let mut diagnostic = *e;
                diagnostic.message = format!("[stdlib:{module_name}] {}", diagnostic.message);
                vec![diagnostic]
            })?;
            let rust_source = stdlib_rust_source(
                module_name,
                stdlib_source,
                &sysroot,
                result
                    .module
                    .classes
                    .iter()
                    .filter(|class| {
                        !class
                            .rust_interop
                            .iter()
                            .any(|declaration| declaration.abi_requirements.opaque_handle)
                    })
                    .map(|class| class.name.clone())
                    .collect(),
                codegen_result.rust_source,
            )?;
            stdlib_code
                .module_rust_code
                .insert(module_name.to_string(), rust_source);
            if !codegen_result.constant_mappings.is_empty() {
                stdlib_code
                    .module_constants
                    .insert(module_name.to_string(), codegen_result.constant_mappings);
            }
            let mut sig_map = HashMap::new();
            for func in &result.module.functions {
                if private_declaration || should_export_callable(module_name, &func.name) {
                    let param_info = signature_params(&func.params, None);
                    sig_map.insert(func.name.clone(), (param_info, func.return_type.clone()));
                }
            }
            for class in &result.module.classes {
                let mut has_constructor = false;
                for method in &class.methods {
                    let param_info = signature_params(
                        &method.params,
                        (method.name == "new").then_some(ParamConvention::own()),
                    );
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
                        .map(|(_, ty)| (ty.clone(), ParamConvention::own()))
                        .collect::<Vec<_>>();
                    sig_map.insert(
                        format!("{}::new", class.name),
                        (
                            ctor_params,
                            Type::Class {
                                identity: None,
                                type_args: class
                                    .type_params
                                    .iter()
                                    .cloned()
                                    .map(Type::TypeVar)
                                    .collect(),
                                name: class.name.clone(),
                                fields: class.fields.clone(),
                                methods: Vec::new(),
                                parent_class: class.semantic_parent_chain(),
                            },
                        ),
                    );
                }
            }
            if !sig_map.is_empty() {
                stdlib_code
                    .func_signatures
                    .insert(module_name.to_string(), sig_map);
            }

            let mut gen_fns = HashSet::new();
            for func in &result.module.functions {
                if (private_declaration || should_export_callable(module_name, &func.name))
                    && sifr_codegen::body_contains_yield(&func.body)
                {
                    gen_fns.insert(func.name.clone());
                }
            }
            if !gen_fns.is_empty() {
                stdlib_code
                    .generator_functions
                    .insert(module_name.to_string(), gen_fns);
            }

            for class in &result.module.classes {
                if !class.type_params.is_empty() {
                    stdlib_code.generic_classes.insert(class.name.clone());
                    stdlib_code
                        .generic_class_params
                        .insert(class.name.clone(), class.type_params.clone());
                    stdlib_code
                        .generic_class_templates
                        .insert(class.name.clone(), class.clone());
                }
            }
            let class_fields = result
                .module
                .classes
                .iter()
                .map(|class| (class.name.clone(), class.fields.clone()))
                .collect();
            stdlib_code
                .module_class_fields
                .insert(module_name.to_string(), class_fields);
            let class_templates = result
                .module
                .classes
                .iter()
                .map(|class| {
                    (
                        class.name.clone(),
                        stdlib_class_template(module_name, class),
                    )
                })
                .collect();
            stdlib_code
                .module_class_templates
                .insert(module_name.to_string(), class_templates);
        }

        if !transitive_deps_for_module.is_empty() {
            stdlib_code
                .transitive_deps
                .insert(module_name.to_string(), transitive_deps_for_module);
        }

        stdlib_defs
            .functions
            .insert(module_name.to_string(), fn_exports);
        if !compiler_intrinsic_exports.is_empty() {
            stdlib_defs
                .compiler_intrinsics
                .insert(module_name.to_string(), compiler_intrinsic_exports);
        }
        stdlib_defs
            .classes
            .insert(module_name.to_string(), class_exports);
        if error_exports.is_empty() {
            stdlib_defs.error_types.remove(module_name);
        } else {
            stdlib_defs
                .error_types
                .insert(module_name.to_string(), error_exports);
        }
        stdlib_defs
            .class_instance_methods
            .insert(module_name.to_string(), class_instance_method_exports);
        if !class_type_param_exports.is_empty() {
            stdlib_defs
                .class_type_params
                .insert(module_name.to_string(), class_type_param_exports);
        }
        if !default_exports.is_empty() {
            stdlib_defs
                .function_defaults
                .insert(module_name.to_string(), default_exports);
        }
        if !vararg_exports.is_empty() {
            stdlib_defs
                .function_varargs
                .insert(module_name.to_string(), vararg_exports);
        }
        if !workload_exports.is_empty() {
            stdlib_defs
                .function_workloads
                .insert(module_name.to_string(), workload_exports);
        }
        if !const_exports.is_empty() {
            stdlib_defs
                .constants
                .insert(module_name.to_string(), const_exports);
        }
        if !const_integer_value_exports.is_empty() {
            stdlib_defs
                .constant_integer_values
                .insert(module_name.to_string(), const_integer_value_exports);
        }
        if !result.module.generic_functions.is_empty() {
            stdlib_defs.generic_functions.insert(
                module_name.to_string(),
                result.module.generic_functions.clone(),
            );
        }
        if !result.module.type_param_bounds.is_empty() {
            stdlib_defs.type_param_bounds.insert(
                module_name.to_string(),
                result.module.type_param_bounds.clone(),
            );
        }
    }

    Ok(StdlibCompiled {
        defs: stdlib_defs,
        code: stdlib_code,
        interop: build_stdlib_rust_interop(Some(sysroot), &private_interop_modules),
    })
}

fn stdlib_rust_source(
    module_name: &str,
    source: &LoadedStdlibSource,
    sysroot: &ResolvedSysroot,
    nominal_types: HashSet<String>,
    rust: String,
) -> Result<StdlibRustSource, Vec<RenderedDiagnostic>> {
    Ok(StdlibRustSource {
        module: module_name.to_string(),
        source_path: canonical_stdlib_source_path(source, sysroot)?,
        source_sha256: source_sha256(&source.source),
        nominal_types,
        rust,
    })
}

fn canonical_stdlib_source_path(
    source: &LoadedStdlibSource,
    sysroot: &ResolvedSysroot,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let relative = source
        .path
        .strip_prefix(&sysroot.paths.stdlib_root)
        .map_err(|_| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!(
                    "stdlib source path {} is outside resolved stdlib root {}",
                    source.path.display(),
                    sysroot.paths.stdlib_root.display()
                ),
                DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        });
    relative.map(|path| format!("stdlib/{}", normalized_path_string(path)))
}

fn source_sha256(source: &str) -> String {
    sha256_hex(source.as_bytes())
}

fn normalized_path_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn lower_stdlib_source(
    source: &LoadedStdlibSource,
    suite: &[sifr_python_ast::Stmt],
    stdlib_defs: &ExternalDefs,
) -> Result<sifr_ir::LoweringResult, Vec<sifr_ir::HirDiagnostic>> {
    match source.kind {
        LoadedStdlibSourceKind::Public => {
            lower_module_sysroot_public_stdlib_with_externals(suite, stdlib_defs)
        }
        LoadedStdlibSourceKind::PrivateDeclaration => {
            lower_module_sysroot_private_declaration_with_externals(suite, stdlib_defs)
        }
    }
}

fn canonical_stdlib_type(ty: &Type, local_classes: &HashMap<String, String>) -> Type {
    canonicalize_user_export_type(ty, local_classes)
}

fn canonicalize_stdlib_hir_signatures(
    module: &mut sifr_ir::HirModule,
    module_name: &str,
    local_classes: &HashMap<String, String>,
) {
    if local_classes.is_empty() {
        return;
    }
    for function in &mut module.functions {
        canonicalize_stdlib_hir_function(function, local_classes);
    }
    for class in &mut module.classes {
        class.identity = Some(format!("{module_name}.{}", class.name));
        for (_, field_type) in &mut class.fields {
            canonicalize_user_export_type_in_place(field_type, local_classes);
        }
        for method in &mut class.methods {
            canonicalize_stdlib_hir_function(method, local_classes);
        }
        for (_, operator) in &mut class.operator_impls {
            canonicalize_stdlib_hir_function(operator, local_classes);
        }
        if let Some(inner) = &mut class.newtype_inner {
            canonicalize_user_export_type_in_place(inner, local_classes);
        }
        if let Some(parent) = &mut class.parent_type {
            canonicalize_user_export_type_in_place(parent, local_classes);
        }
    }
    for (_, constant_type, _) in &mut module.constants {
        canonicalize_user_export_type_in_place(constant_type, local_classes);
    }
}

fn canonicalize_stdlib_hir_function(
    function: &mut HirFunction,
    local_classes: &HashMap<String, String>,
) {
    sifr_ir::transform_hir_function_types(function, &mut |ty| {
        canonicalize_user_export_type_in_place(ty, local_classes);
    });
}

fn function_type_from_params(params: &[HirParam], return_type: &Type) -> FunctionType {
    FunctionType {
        receiver: None,
        params: named_params(params),
        return_type: Box::new(return_type.clone()),
    }
}

fn function_type_from_hir(function: &HirFunction) -> FunctionType {
    let return_type = if function.is_async {
        coroutine_type_from_surface_return(&function.return_type)
    } else {
        function.return_type.clone()
    };
    let mut signature = function_type_from_params(&function.params, &return_type);
    signature.receiver = function.receiver;
    signature
}

fn method_type_from_hir(method: &HirFunction) -> FunctionType {
    let return_type = if method.is_async {
        coroutine_type_from_surface_return(&method.return_type)
    } else {
        method.return_type.clone()
    };
    let mut signature = function_type_from_params(&method.params, &return_type);
    signature.receiver = method.receiver;
    signature
}

fn coroutine_type_from_surface_return(surface_return_type: &Type) -> Type {
    match surface_return_type.resolve_alias() {
        Type::Result(ok, err) => Type::Coroutine(ok.clone(), err.clone()),
        other => Type::Coroutine(Box::new(other.clone()), Box::new(Type::Never)),
    }
}

fn named_params(params: &[HirParam]) -> Vec<(String, Type, ParamConvention)> {
    params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone(), param.convention))
        .collect()
}

fn signature_params(
    params: &[HirParam],
    convention_override: Option<ParamConvention>,
) -> Vec<(Type, ParamConvention)> {
    params
        .iter()
        .map(|param| {
            (
                param.ty.clone(),
                convention_override.unwrap_or(param.convention),
            )
        })
        .collect()
}

fn collect_public_constant_integer_value_exports<'a, T: Clone>(
    public_constant_names: impl Iterator<Item = &'a str>,
    constant_integer_values: &HashMap<String, T>,
) -> HashMap<String, T> {
    public_constant_names
        .filter_map(|name| {
            constant_integer_values
                .get(name)
                .map(|value| (name.to_string(), value.clone()))
        })
        .collect()
}

fn select_imported_class_templates<T: Clone>(
    imports: &[sifr_ir::HirImport],
    available: &HashMap<String, HashMap<String, T>>,
) -> HashMap<String, HashMap<String, T>> {
    let mut selected = HashMap::<String, HashMap<String, T>>::new();
    for import in imports {
        let Some(module_templates) = available.get(&import.module) else {
            continue;
        };
        if import
            .names
            .iter()
            .any(|name| module_templates.contains_key(name))
        {
            selected
                .entry(import.module.clone())
                .or_insert_with(|| module_templates.clone());
        }
    }
    selected
}

fn stdlib_class_template(module_name: &str, class: &sifr_ir::HirClass) -> sifr_ir::HirClass {
    let mut template = class.clone();
    template.identity = Some(format!("{module_name}.{}", class.name));
    for method in &mut template.methods {
        method.body.clear();
    }
    for (_, method) in &mut template.operator_impls {
        method.body.clear();
    }
    template
}

#[cfg(test)]
#[path = "bootstrap_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "bootstrap_template_tests.rs"]
mod template_tests;
