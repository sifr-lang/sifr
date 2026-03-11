use crate::diagnostics::{run_codegen_with_boundary, CompileError, CompilePhase};
use crate::stdlib::cache::{get_or_init_stdlib_cache, STDLIB_COMPILED_CACHE};
use crate::stdlib::intrinsics::intrinsic_constant_rust_expr;
use crate::stdlib::registry::STDLIB_FILES;
use crate::stdlib::types::StdlibCompiled;
use sifr_codegen::StdlibCode;
use sifr_hir::{lower_module_stdlib_with_externals, ExternalDefs};
use sifr_python_parser::parse_module;
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::{HashMap, HashSet};

pub(crate) fn compile_stdlib() -> Result<StdlibCompiled, Vec<CompileError>> {
    get_or_init_stdlib_cache(&STDLIB_COMPILED_CACHE, compile_stdlib_uncached)
}

#[cfg(test)]
pub(crate) fn compile_stdlib_uncached() -> Result<StdlibCompiled, Vec<CompileError>> {
    compile_stdlib_uncached_impl()
}

#[cfg(not(test))]
pub(crate) fn compile_stdlib_uncached() -> Result<StdlibCompiled, Vec<CompileError>> {
    compile_stdlib_uncached_impl()
}

fn compile_stdlib_uncached_impl() -> Result<StdlibCompiled, Vec<CompileError>> {
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

        let mut intrinsic_names_for_module = HashSet::new();
        let mut transitive_deps_for_module = HashSet::new();

        let mut fn_exports = HashMap::new();
        let mut class_exports = HashMap::new();
        let mut class_type_param_exports = HashMap::new();
        let mut default_exports = HashMap::new();

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

        for (callable_name, defaults) in &result.function_defaults {
            if !callable_name.starts_with('_') {
                default_exports.insert(callable_name.clone(), defaults.clone());
            }
        }

        let mut const_exports = HashMap::new();
        for import in &result.module.imports {
            if import.module.starts_with("_sifr.") {
                transitive_deps_for_module.insert(import.module.clone());
                if let Some(intrinsic_mod) = sifr_hir::stdlib::get_intrinsic_module(&import.module)
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
            };
            let codegen_result = run_codegen_with_boundary(
                format!(
                    "internal compiler panic during stdlib code generation for '{module_name}'"
                ),
                || sifr_codegen::generate_rust_with_stdlib(&result.module, &codegen_stdlib),
            )
            .map_err(|e| {
                vec![CompileError {
                    message: format!("[stdlib:{module_name}] {}", e.message),
                    phase: e.phase,
                }]
            })?;
            stdlib_code
                .module_rust_code
                .insert((*module_name).to_string(), codegen_result.rust_source);
            if !codegen_result.constant_mappings.is_empty() {
                stdlib_code
                    .module_constants
                    .insert((*module_name).to_string(), codegen_result.constant_mappings);
            }
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
        if !default_exports.is_empty() {
            stdlib_defs
                .function_defaults
                .insert((*module_name).to_string(), default_exports);
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
