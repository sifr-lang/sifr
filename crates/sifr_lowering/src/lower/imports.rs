use ruff_text_size::TextRange;
use sifr_python_ast::Stmt;

use super::imported_defaults::{
    import_callable_vararg, import_callable_workload, import_python_call_shape,
};
use super::{import_diagnostics, name_diagnostics, ExternalDefs, LowerCtx};
use std::collections::HashMap;

pub(super) fn register_imported_class_instance_methods(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module: &str,
    source_name: &str,
    local_name: &str,
) {
    super::imported_defaults::import_rust_threadsafe_callback_class(
        ctx,
        externals,
        module,
        source_name,
        local_name,
    );
    let Some(methods) = externals
        .class_instance_methods
        .get(module)
        .and_then(|classes| classes.get(source_name))
    else {
        return;
    };
    ctx.class_instance_methods.extend(
        methods
            .iter()
            .map(|method| format!("{local_name}.{method}")),
    );
}

fn register_imported_rust_consuming_methods(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module: &str,
    source_name: &str,
    local_name: &str,
) {
    let Some(methods) = externals
        .rust_consuming_methods
        .get(module)
        .and_then(|classes| classes.get(source_name))
    else {
        return;
    };
    ctx.rust_consuming_methods.extend(
        methods
            .iter()
            .map(|method| format!("{local_name}.{method}")),
    );
}

pub(in crate::lower) fn class_aliases_by_module(
    stmts: &[Stmt],
    externals: &ExternalDefs,
    ctx: &LowerCtx,
) -> HashMap<String, HashMap<String, String>> {
    let mut aliases_by_module: HashMap<String, HashMap<String, String>> = HashMap::new();
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
        let module_name =
            ctx.effective_import_module_name(module.as_ref(), import_from.level, externals);
        let names = import_from
            .names
            .iter()
            .map(|alias| alias.name.to_string())
            .collect::<Vec<_>>();
        let aliases = import_from
            .names
            .iter()
            .filter_map(|alias| {
                alias
                    .asname
                    .as_ref()
                    .map(|local| (alias.name.to_string(), local.to_string()))
            })
            .collect::<Vec<_>>();
        let imported = super::imported_class_identity::class_aliases_for_import(
            &module_name,
            externals.classes.get(&module_name),
            &names,
            &aliases,
        );
        aliases_by_module
            .entry(module_name)
            .or_default()
            .extend(imported);
    }
    aliases_by_module
}

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
    let aliases_by_module = class_aliases_by_module(stmts, externals, ctx);
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
            let class_aliases = aliases_by_module
                .get(&module_key)
                .cloned()
                .unwrap_or_default();
            if let Some(module_classes) = externals.classes.get(&module_key) {
                for name in &names {
                    let local = local_name_for(name);
                    if let Some(class_ty) = module_classes.get(name) {
                        if !ctx.class_types.contains_key(&local) {
                            let imported_class_ty = super::imported_class_identity::type_for_import(
                                class_ty,
                                &module_key,
                                &class_aliases,
                            );
                            ctx.class_types
                                .insert(local.clone(), imported_class_ty.clone());
                            register_imported_class_instance_methods(
                                ctx,
                                externals,
                                &module_key,
                                name,
                                &local,
                            );
                            register_imported_rust_consuming_methods(
                                ctx,
                                externals,
                                &module_key,
                                name,
                                &local,
                            );
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
                            if let Some(ft) =
                                super::imported_class_identity::imported_constructor_function_type(
                                    &imported_class_ty,
                                )
                            {
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
                        let imported = super::imported_class_identity::function_type_for_import(
                            ft,
                            &module_key,
                            &class_aliases,
                        );
                        ctx.functions.entry(local.clone()).or_insert(imported);
                        super::imported_defaults::import_rust_threadsafe_callback_target(
                            ctx,
                            externals,
                            &module_key,
                            name,
                            &local,
                        );
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
                        ctx.scope.define(
                            local.clone(),
                            super::imported_class_identity::type_for_import(
                                const_ty,
                                &module_key,
                                &class_aliases,
                            ),
                        );
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
