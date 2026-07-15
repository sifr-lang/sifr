use ruff_text_size::TextRange;
use sifr_python_ast::Stmt;
use sifr_type_system::{FunctionType, Type};

use super::imported_defaults::{
    import_callable_vararg, import_callable_workload, import_python_call_shape,
};
use super::{import_diagnostics, name_diagnostics, ExternalDefs, LowerCtx};

pub(in crate::lower) fn report_missing_stdlib_member(
    ctx: &mut LowerCtx,
    module: &str,
    member: &str,
    range: TextRange,
) {
    name_diagnostics::missing_member(ctx, module, member, range);
}

pub(in crate::lower) fn report_unknown_stdlib_module(
    ctx: &mut LowerCtx,
    module: &str,
    imported_names: &str,
    range: TextRange,
) {
    if let Some(legacy_module) = sifr_stdlib_imports::unsupported_legacy_stdlib_module(module) {
        import_diagnostics::unsupported_legacy_stdlib_module(
            ctx,
            &legacy_module,
            imported_names,
            range,
        );
    } else if let Some(reason) = deferred_module_reason(module) {
        import_diagnostics::deferred_compat_module(ctx, module, reason, range);
    } else {
        import_diagnostics::unknown_import_target(ctx, module, range);
    }
}

fn deferred_module_reason(module: &str) -> Option<&'static str> {
    match module {
        "sifr.contextlib" => Some(
            "contextlib compatibility is rejected; use deterministic `sifr.resource` scopes instead",
        ),
        "sifr.warnings" => Some(
            "Python global warning filters are rejected; use typed Sifr diagnostics and runtime observability instead",
        ),
        "sifr.selectors" => {
            Some("public selectors APIs are deferred; compose tasks and channels instead")
        }
        "sifr.contextvars" => Some("context-local state is deferred; pass task state explicitly"),
        _ => None,
    }
}

pub(in crate::lower) fn resolve_imports_early(
    stmts: &[Stmt],
    externals: &ExternalDefs,
    ctx: &mut LowerCtx,
) {
    for stmt in stmts {
        if let Stmt::ImportFrom(import_from) = stmt {
            if import_from.level > 1 {
                continue;
            }
            let Some(ref module) = import_from.module else {
                continue;
            };
            let module_name =
                ctx.effective_import_module_name(module.as_ref(), import_from.level, externals);
            let is_absolute_import = import_from.level == 0;
            if is_absolute_import && (module_name == "typing" || module_name == "enum") {
                continue;
            }
            let names: Vec<String> = import_from
                .names
                .iter()
                .map(|alias| alias.name.to_string())
                .collect();
            let aliases: Vec<(String, String)> = import_from
                .names
                .iter()
                .filter_map(|alias| {
                    alias
                        .asname
                        .as_ref()
                        .map(|asname| (alias.name.to_string(), asname.to_string()))
                })
                .collect();
            let local_name_for = |original: &str| -> String {
                aliases
                    .iter()
                    .find(|(orig, _)| orig == original)
                    .map(|(_, alias)| alias.clone())
                    .unwrap_or_else(|| original.to_string())
            };

            // Only resolve from externals (stdlib and local modules)
            let module_key = module_name.clone();
            if let Some(module_classes) = externals.classes.get(&module_key) {
                for name in &names {
                    let local = local_name_for(name);
                    if let Some(class_ty) = module_classes.get(name) {
                        if !ctx.class_types.contains_key(&local) {
                            ctx.class_types.insert(local.clone(), class_ty.clone());
                            if let Some(module_class_type_params) =
                                externals.class_type_params.get(&module_key)
                            {
                                if let Some(type_params) = module_class_type_params.get(name) {
                                    ctx.class_declared_type_params
                                        .insert(local.clone(), type_params.clone());
                                    ctx.class_declared_type_params
                                        .entry(name.clone())
                                        .or_insert_with(|| type_params.clone());
                                }
                            }
                            if let Some(module_bounds) =
                                externals.type_param_bounds.get(&module_key)
                            {
                                super::generic_method_requirements::import_generic_method_requirements(
                                    ctx,
                                    module_bounds,
                                    name,
                                    &local,
                                );
                            }
                            // Register as error type if flagged
                            if externals.error_types.contains(name) {
                                ctx.error_types.insert(local.clone());
                            }
                            if let Some(module_workloads) =
                                externals.function_workloads.get(&module_key)
                            {
                                super::imported_defaults::import_class_method_workloads(
                                    ctx,
                                    module_workloads,
                                    name,
                                    &local,
                                );
                            }
                            // Register constructor
                            if let Type::Class {
                                fields, methods, ..
                            } = class_ty
                            {
                                let ft = if let Some((_, new_ft)) =
                                    methods.iter().find(|(n, _)| n == "new")
                                {
                                    let params: Vec<(String, Type)> = new_ft
                                        .params
                                        .iter()
                                        .map(|(n, t, _)| (n.clone(), t.clone()))
                                        .collect();
                                    FunctionType::new(params, class_ty.clone())
                                } else {
                                    let params: Vec<(String, Type)> = fields.clone();
                                    FunctionType::new(params, class_ty.clone())
                                };
                                ctx.functions.insert(local, ft);
                            }
                        }
                    }
                }
            }
            if let Some(module_fns) = externals.functions.get(&module_key) {
                for name in &names {
                    let local = local_name_for(name);
                    if let Some(ft) = module_fns.get(name) {
                        ctx.functions
                            .entry(local.clone())
                            .or_insert_with(|| ft.clone());
                        if let Some(intrinsic) = externals
                            .compiler_intrinsics
                            .get(&module_key)
                            .and_then(|module_intrinsics| module_intrinsics.get(name))
                        {
                            ctx.compiler_intrinsics.insert(local.clone(), *intrinsic);
                        }
                        if let Some(module_varargs) = externals.function_varargs.get(&module_key) {
                            import_callable_vararg(
                                ctx,
                                module_varargs,
                                name,
                                &local_name_for(name),
                            );
                        }
                        if let Some(module_shapes) =
                            externals.function_python_call_shapes.get(&module_key)
                        {
                            import_python_call_shape(
                                ctx,
                                module_shapes,
                                name,
                                &local_name_for(name),
                            );
                        }
                        if let Some(module_workloads) =
                            externals.function_workloads.get(&module_key)
                        {
                            import_callable_workload(
                                ctx,
                                module_workloads,
                                name,
                                &local_name_for(name),
                            );
                        }
                    }
                }
            }
            if let Some(module_consts) = externals.constants.get(&module_key) {
                for name in &names {
                    let local = local_name_for(name);
                    if let Some(const_ty) = module_consts.get(name) {
                        ctx.scope.define(local.clone(), const_ty.clone());
                        if let Some(value) = externals
                            .constant_integer_values
                            .get(&module_key)
                            .and_then(|module_values| module_values.get(name))
                        {
                            ctx.const_integer_values.insert(local, value.clone());
                        }
                    }
                }
            }
        }
    }
}
