use crate::diagnostics::{run_codegen_with_boundary, RenderedDiagnostic};
use crate::export_policy::should_export_callable;
use crate::stdlib::cache::{get_or_init_stdlib_cache, STDLIB_COMPILED_CACHE};
use crate::stdlib::interop::{build_stdlib_rust_interop, pending_private_interop_module};
use crate::stdlib::intrinsics::intrinsic_constant_rust_expr;
use crate::stdlib::re_exports::{re_export_stdlib_imports, ReExportMaps};
use crate::stdlib::types::StdlibCompiled;
use sifr_codegen::StdlibCode;
use sifr_diagnostics::DiagnosticCode;
use sifr_lowering::{lower_module_stdlib_with_externals, ExternalDefs, HirFunction, HirParam};
use sifr_stdlib_model::{
    load_stdlib_tooling_sources_from_sysroot, LoadedStdlibSource, LoadedStdlibSourceKind,
};
use sifr_syntax::parse_module_raw;
use sifr_sysroot::ResolvedSysroot;
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::{HashMap, HashSet};

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
    compile_stdlib_sources_with_sysroot(&sources, Some(sysroot))
}

fn compile_stdlib_sources_with_sysroot(
    sources: &[LoadedStdlibSource],
    sysroot: Option<ResolvedSysroot>,
) -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    let mut stdlib_defs = ExternalDefs::default();
    let mut stdlib_code = StdlibCode::default();
    let mut private_interop_modules = Vec::new();
    seed_http_transport_harness_aliases(&mut stdlib_defs, &mut stdlib_code);

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
        let result = match lower_module_stdlib_with_externals(parsed.suite(), &stdlib_defs) {
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
        if let Some(module) = pending_private_interop_module(stdlib_source, &result.module) {
            private_interop_modules.push(module);
        }
        if stdlib_source.kind == LoadedStdlibSourceKind::PrivateDeclaration {
            continue;
        }

        let mut intrinsic_names_for_module = HashSet::new();
        let mut transitive_deps_for_module = HashSet::new();

        let mut fn_exports = HashMap::new();
        let mut class_exports = HashMap::new();
        let mut class_type_param_exports = HashMap::new();
        let mut default_exports = HashMap::new();
        let mut vararg_exports = HashMap::new();
        let mut workload_exports = HashMap::new();

        for func in &result.module.functions {
            if should_export_callable(module_name, &func.name) {
                fn_exports.insert(
                    func.name.clone(),
                    function_type_from_params(&func.params, &func.return_type),
                );
                if module_name == "sifr.python"
                    && matches!(func.name.as_str(), "local_callback" | "threadsafe_callback")
                {
                    intrinsic_names_for_module.insert(func.name.clone());
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
            if should_export_callable(module_name, owner_name) {
                workload_exports.insert(callable_name.clone(), label.clone());
            }
        }

        for (callable_name, defaults) in &result.function_defaults {
            if should_export_callable(module_name, callable_name) {
                default_exports.insert(callable_name.clone(), defaults.clone());
            }
        }

        let mut const_exports = HashMap::new();
        for import in &result.module.imports {
            if import.module.starts_with("_sifr.") {
                transitive_deps_for_module.insert(import.module.clone());
                if let Some(intrinsic_mod) = sifr_stdlib_model::get_intrinsic_module(&import.module)
                {
                    for name in &import.names {
                        if let Some(ft) = intrinsic_mod.functions.get(name) {
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
                transitive_deps_for_module.insert(import.module.clone());
                if module_name == "sifr.python" && import.module == "sifr.python_core" {
                    let mut exports = ReExportMaps {
                        functions: &mut fn_exports,
                        classes: &mut class_exports,
                        class_type_params: &mut class_type_param_exports,
                        defaults: &mut default_exports,
                        varargs: &mut vararg_exports,
                        workloads: &mut workload_exports,
                        constants: &mut const_exports,
                    };
                    re_export_stdlib_imports(
                        &mut exports,
                        &stdlib_defs,
                        &import.module,
                        &import.names,
                    );
                }
                if let Some(deps) = stdlib_code.transitive_deps.get(&import.module) {
                    transitive_deps_for_module.extend(deps.iter().cloned());
                }
            }
        }

        for (name, ty, _expr) in &result.module.constants {
            if !name.starts_with('_') {
                const_exports.insert(name.clone(), ty.clone());
            }
        }
        let const_integer_value_exports = collect_public_constant_integer_value_exports(
            result
                .module
                .constants
                .iter()
                .filter_map(|(name, _, _)| (!name.starts_with('_')).then_some(name.as_str())),
            &result.constant_integer_values,
        );

        for class in &result.module.classes {
            if !class.name.starts_with('_') {
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
                let class_ty = Type::Class {
                    name: class.name.clone(),
                    fields: class.fields.clone(),
                    methods,
                    parent_class: class.parent_class.clone(),
                };
                class_exports.insert(class.name.clone(), class_ty);
                if !class.type_params.is_empty() {
                    class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
                }
                if class.is_error_type {
                    stdlib_defs.error_types.insert(class.name.clone());
                }
            }
        }

        let has_pure_sifr_code = !result.module.functions.is_empty()
            || !result.module.constants.is_empty()
            || !result.module.classes.is_empty();
        if has_pure_sifr_code {
            let codegen_stdlib = StdlibCode {
                module_rust_code: HashMap::new(),
                intrinsic_names: stdlib_code.intrinsic_names.clone(),
                module_constants: stdlib_code.module_constants.clone(),
                func_signatures: stdlib_code.func_signatures.clone(),
                transitive_deps: stdlib_code.transitive_deps.clone(),
                generator_functions: stdlib_code.generator_functions.clone(),
                generic_classes: stdlib_code.generic_classes.clone(),
                generic_class_params: stdlib_code.generic_class_params.clone(),
                generic_class_templates: stdlib_code.generic_class_templates.clone(),
                module_class_fields: stdlib_code.module_class_fields.clone(),
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
            stdlib_code
                .module_rust_code
                .insert(module_name.to_string(), codegen_result.rust_source);
            if !codegen_result.constant_mappings.is_empty() {
                stdlib_code
                    .module_constants
                    .insert(module_name.to_string(), codegen_result.constant_mappings);
            }
            let mut sig_map = HashMap::new();
            for func in &result.module.functions {
                if should_export_callable(module_name, &func.name) {
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
                    .insert(module_name.to_string(), sig_map);
            }

            let mut gen_fns = HashSet::new();
            for func in &result.module.functions {
                if should_export_callable(module_name, &func.name)
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
        }

        stdlib_code
            .intrinsic_names
            .insert(module_name.to_string(), intrinsic_names_for_module);
        if !transitive_deps_for_module.is_empty() {
            stdlib_code
                .transitive_deps
                .insert(module_name.to_string(), transitive_deps_for_module);
        }

        stdlib_defs
            .functions
            .insert(module_name.to_string(), fn_exports);
        stdlib_defs
            .classes
            .insert(module_name.to_string(), class_exports);
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
        interop: build_stdlib_rust_interop(sysroot, private_interop_modules),
    })
}

fn function_type_from_params(params: &[HirParam], return_type: &Type) -> FunctionType {
    FunctionType {
        params: named_params(params),
        return_type: Box::new(return_type.clone()),
    }
}

fn method_type_from_hir(method: &HirFunction) -> FunctionType {
    let return_type = if method.is_async {
        coroutine_type_from_surface_return(&method.return_type)
    } else {
        method.return_type.clone()
    };
    function_type_from_params(&method.params, &return_type)
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

const HTTP_TRANSPORT_HARNESS_ALIASES: &[(&str, &str)] = &[
    (
        "transport_http1_client_roundtrip_tcp",
        "http1_client_roundtrip_tcp",
    ),
    (
        "transport_http2_client_roundtrip_tcp",
        "http2_client_roundtrip_tcp",
    ),
    (
        "transport_http1_client_roundtrip_tls",
        "http1_client_roundtrip_tls",
    ),
    (
        "transport_http2_client_roundtrip_tls",
        "http2_client_roundtrip_tls",
    ),
    (
        "transport_http1_server_respond_tcp",
        "http1_server_respond_tcp",
    ),
    (
        "transport_http2_server_respond_tcp",
        "http2_server_respond_tcp",
    ),
    (
        "transport_http1_server_respond_tls",
        "http1_server_respond_tls",
    ),
    (
        "transport_http2_server_respond_tls",
        "http2_server_respond_tls",
    ),
];

fn seed_http_transport_harness_aliases(
    stdlib_defs: &mut ExternalDefs,
    stdlib_code: &mut StdlibCode,
) {
    let Some(intrinsic_http) = sifr_stdlib_model::get_intrinsic_module("_sifr.http") else {
        return;
    };

    let mut functions = HashMap::new();
    let mut sig_map = HashMap::new();
    for (alias, intrinsic_name) in HTTP_TRANSPORT_HARNESS_ALIASES {
        let Some(function_type) = intrinsic_http.functions.get(*intrinsic_name) else {
            continue;
        };
        let param_info = function_type
            .params
            .iter()
            .map(|(_, ty, _)| (ty.clone(), ParamConvention::own()))
            .collect();
        sig_map.insert(
            (*alias).to_string(),
            (param_info, (*function_type.return_type).clone()),
        );
        functions.insert((*alias).to_string(), function_type.clone());
    }

    if functions.is_empty() {
        return;
    }

    stdlib_defs
        .functions
        .insert("sifr.http_transport".to_string(), functions);
    stdlib_code
        .intrinsic_names
        .insert("sifr.http_transport".to_string(), HashSet::new());
    stdlib_code
        .func_signatures
        .insert("sifr.http_transport".to_string(), sig_map);
    stdlib_code.module_rust_code.insert(
        "sifr.http_transport".to_string(),
        HTTP_TRANSPORT_HARNESS_RUST.to_string(),
    );
    stdlib_code.transitive_deps.insert(
        "sifr.http_transport".to_string(),
        HashSet::from(["sifr.http".to_string()]),
    );
}

const HTTP_TRANSPORT_HARNESS_RUST: &str = r#"
async fn transport_http1_client_roundtrip_tcp(
    handle: i64,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
) -> Result<(i64, String, Vec<(String, String)>, Vec<u8>), HttpError> {
    sifr_runtime::http::http1_request_tcp(
        handle,
        method,
        path,
        headers,
        body,
        max_request_bytes,
        max_response_bytes,
        0.0,
        false,
    )
    .await
    .map(|response| (response.status, response.version, response.headers, response.body))
    .map_err(HttpError::new)
}

async fn transport_http2_client_roundtrip_tcp(
    handle: i64,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
) -> Result<(i64, String, Vec<(String, String)>, Vec<u8>), HttpError> {
    sifr_runtime::http::http2_request_tcp(
        handle,
        method,
        path,
        headers,
        body,
        max_request_bytes,
        max_response_bytes,
        0.0,
        false,
    )
    .await
    .map(|response| (response.status, response.version, response.headers, response.body))
    .map_err(HttpError::new)
}

async fn transport_http1_client_roundtrip_tls(
    handle: i64,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
) -> Result<(i64, String, Vec<(String, String)>, Vec<u8>), HttpError> {
    sifr_runtime::http::http1_request_tls(
        handle,
        method,
        path,
        headers,
        body,
        max_request_bytes,
        max_response_bytes,
        0.0,
        false,
    )
    .await
    .map(|response| (response.status, response.version, response.headers, response.body))
    .map_err(HttpError::new)
}

async fn transport_http2_client_roundtrip_tls(
    handle: i64,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
) -> Result<(i64, String, Vec<(String, String)>, Vec<u8>), HttpError> {
    sifr_runtime::http::http2_request_tls(
        handle,
        method,
        path,
        headers,
        body,
        max_request_bytes,
        max_response_bytes,
        0.0,
        false,
    )
    .await
    .map(|response| (response.status, response.version, response.headers, response.body))
    .map_err(HttpError::new)
}

async fn transport_http1_server_respond_tcp(
    handle: i64,
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
) -> Result<(String, String, String, Vec<(String, String)>, Vec<u8>), HttpError> {
    sifr_runtime::http::http1_respond_tcp(
        handle,
        status,
        headers,
        body,
        max_request_bytes,
        max_response_bytes,
        0.0,
        false,
    )
    .await
    .map(|request| (request.method, request.target, request.version, request.headers, request.body))
    .map_err(HttpError::new)
}

async fn transport_http2_server_respond_tcp(
    handle: i64,
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
) -> Result<(String, String, String, Vec<(String, String)>, Vec<u8>), HttpError> {
    sifr_runtime::http::http2_respond_tcp(
        handle,
        status,
        headers,
        body,
        max_request_bytes,
        max_response_bytes,
        0.0,
        false,
    )
    .await
    .map(|request| (request.method, request.target, request.version, request.headers, request.body))
    .map_err(HttpError::new)
}

async fn transport_http1_server_respond_tls(
    handle: i64,
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
) -> Result<(String, String, String, Vec<(String, String)>, Vec<u8>), HttpError> {
    sifr_runtime::http::http1_respond_tls(
        handle,
        status,
        headers,
        body,
        max_request_bytes,
        max_response_bytes,
        0.0,
        false,
    )
    .await
    .map(|request| (request.method, request.target, request.version, request.headers, request.body))
    .map_err(HttpError::new)
}

async fn transport_http2_server_respond_tls(
    handle: i64,
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
) -> Result<(String, String, String, Vec<(String, String)>, Vec<u8>), HttpError> {
    sifr_runtime::http::http2_respond_tls(
        handle,
        status,
        headers,
        body,
        max_request_bytes,
        max_response_bytes,
        0.0,
        false,
    )
    .await
    .map(|request| (request.method, request.target, request.version, request.headers, request.body))
    .map_err(HttpError::new)
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_param(name: &str, ty: Type, convention: ParamConvention) -> HirParam {
        HirParam {
            name: name.to_string(),
            ty,
            default: None,
            keyword_only: false,
            convention,
        }
    }

    #[test]
    fn function_type_from_params_preserves_named_conventions() {
        let params = vec![
            sample_param("value", Type::Int, ParamConvention::borrow()),
            sample_param("count", Type::Int, ParamConvention::own()),
        ];

        let function_type = function_type_from_params(&params, &Type::Bool);

        assert_eq!(
            function_type,
            FunctionType {
                params: vec![
                    ("value".to_string(), Type::Int, ParamConvention::borrow()),
                    ("count".to_string(), Type::Int, ParamConvention::own()),
                ],
                return_type: Box::new(Type::Bool),
            }
        );
    }

    #[test]
    fn signature_params_can_override_constructor_conventions() {
        let params = vec![sample_param("self", Type::Str, ParamConvention::borrow())];

        assert_eq!(
            signature_params(&params, Some(ParamConvention::own())),
            vec![(Type::Str, ParamConvention::own())]
        );
    }

    #[test]
    fn public_constant_integer_value_exports_filter_to_public_recorded_values() {
        let mut values = HashMap::new();
        values.insert("ANSWER".to_string(), 42);
        values.insert("_PRIVATE".to_string(), 99);
        values.insert("STALE".to_string(), 100);

        let exports = collect_public_constant_integer_value_exports(
            ["ANSWER", "MISSING"].into_iter(),
            &values,
        );

        assert_eq!(exports.len(), 1);
        assert_eq!(exports.get("ANSWER"), Some(&42));
        assert!(!exports.contains_key("_PRIVATE"));
        assert!(!exports.contains_key("STALE"));
    }

    #[test]
    fn stdlib_class_exports_preserve_parent_markers() {
        let compiled = compile_stdlib_uncached().expect("stdlib should compile");
        let object_ty = compiled
            .defs
            .classes
            .get("sifr.python")
            .and_then(|classes| classes.get("Object"))
            .expect("sifr.python.Object should be exported");

        assert!(matches!(
            object_ty,
            Type::Class {
                parent_class: Some(parent),
                ..
            } if parent.split('|').any(|name| name == "NonSend")
        ));
    }

    #[test]
    fn python_core_re_exports_preserve_callable_metadata() {
        let compiled = compile_stdlib_uncached().expect("stdlib should compile");
        let workloads = compiled
            .defs
            .function_workloads
            .get("sifr.python")
            .expect("sifr.python should export workload metadata");

        assert_eq!(
            workloads.get("threadsafe_callback_echo"),
            Some(&"blocking_io".to_string())
        );
        assert_eq!(
            workloads.get("close_local_callback"),
            Some(&"blocking_io".to_string())
        );

        let defaults = compiled
            .defs
            .function_defaults
            .get("sifr.python")
            .and_then(|module_defaults| module_defaults.get("PythonError"))
            .expect("PythonError constructor defaults should be re-exported");
        assert_eq!(defaults.len(), 4);
    }
}
