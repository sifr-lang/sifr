use ruff_text_size::TextRange;
use sifr_ir::HirImport;
use sifr_python_ast::Stmt;

use super::imported_defaults::{
    import_callable_vararg, import_callable_workload, import_python_call_shape,
};
use super::{ExternalDefs, LowerCtx, import_diagnostics, name_diagnostics};
use std::collections::HashMap;

pub(in crate::lower) fn runtime_hir_import(
    module: String,
    mut names: Vec<String>,
    mut aliases: Vec<(String, String)>,
    externals: &ExternalDefs,
) -> Option<HirImport> {
    if let Some(generic_aliases) = externals.generic_type_aliases.get(&module) {
        names.retain(|name| !generic_aliases.contains_key(name));
        aliases.retain(|(name, _)| !generic_aliases.contains_key(name));
    }
    (!names.is_empty()).then_some(HirImport {
        module,
        names,
        aliases,
    })
}

pub(in crate::lower) fn import_constant(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module: &str,
    source_name: &str,
    local_name: &str,
    class_aliases: &HashMap<String, String>,
) -> bool {
    let Some(const_ty) = externals
        .constants
        .get(module)
        .and_then(|constants| constants.get(source_name))
    else {
        return false;
    };
    ctx.scope.define(
        local_name.to_string(),
        super::imported_class_identity::type_for_import(const_ty, module, class_aliases),
    );
    if let Some(value) = externals
        .constant_integer_values
        .get(module)
        .and_then(|values| values.get(source_name))
        && let Some(binding_id) = ctx
            .scope
            .lookup(local_name)
            .map(|binding| binding.binding_id)
    {
        ctx.const_integer_values
            .record(local_name.to_string(), binding_id, value.clone());
    }
    true
}

pub(in crate::lower) fn import_generic_type_alias(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module: &str,
    source_name: &str,
    local_name: &str,
    class_aliases: &HashMap<String, String>,
) -> bool {
    let Some((type_params, alias)) = externals
        .generic_type_aliases
        .get(module)
        .and_then(|aliases| aliases.get(source_name))
    else {
        return false;
    };
    ctx.scope.define_generic_type_alias(
        local_name.to_string(),
        type_params.clone(),
        super::imported_class_identity::type_for_import(alias, module, class_aliases),
    );
    true
}

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

fn register_imported_rust_opaque_class(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module: &str,
    source_name: &str,
    local_name: &str,
) {
    if externals
        .rust_opaque_classes
        .get(module)
        .is_some_and(|classes| classes.contains(source_name))
    {
        ctx.rust_opaque_classes.insert(local_name.to_string());
    }
    if externals
        .rust_structural_classes
        .get(module)
        .is_some_and(|classes| classes.contains(source_name))
    {
        ctx.rust_structural_classes.insert(local_name.to_string());
    }
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
    range: TextRange,
) {
    if let Some(reason) = deferred_module_reason(module) {
        import_diagnostics::deferred_module(ctx, module, reason, range);
    } else {
        import_diagnostics::unknown_import_target(ctx, module, range);
    }
}

fn deferred_module_reason(module: &str) -> Option<&'static str> {
    match module {
        "sifr.contextvars" => Some("context-local state is deferred; pass task state explicitly"),
        _ => None,
    }
}

pub(in crate::lower) fn resolve_imports_early(
    stmts: &[Stmt],
    externals: &ExternalDefs,
    ctx: &mut LowerCtx,
) {
    ctx.local_structural_marker_declared = stmts
        .iter()
        .any(|stmt| matches!(stmt, Stmt::ClassDef(class) if class.name.as_str() == "Structural"));
    ctx.local_string_structural_marker_declared = stmts.iter().any(
        |stmt| matches!(stmt, Stmt::ClassDef(class) if class.name.as_str() == "StringStructural"),
    );
    ctx.local_static_program_marker_declared = stmts.iter().any(
        |stmt| matches!(stmt, Stmt::ClassDef(class) if class.name.as_str() == "StaticProgram"),
    );
    ctx.local_method_slots_marker_declared = stmts
        .iter()
        .any(|stmt| matches!(stmt, Stmt::ClassDef(class) if class.name.as_str() == "MethodSlots"));
    ctx.local_context_marker_declared = stmts
        .iter()
        .any(|stmt| matches!(stmt, Stmt::ClassDef(class) if class.name.as_str() == "Context"));
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
            if module_name == "sifr.meta"
                && import_from
                    .names
                    .iter()
                    .any(|alias| alias.name.as_str() == "Structural" && alias.asname.is_none())
            {
                ctx.canonical_structural_marker_imported = true;
            }
            if module_name == "sifr.meta"
                && import_from.names.iter().any(|alias| {
                    alias.name.as_str() == "StringStructural" && alias.asname.is_none()
                })
            {
                ctx.canonical_string_structural_marker_imported = true;
            }
            if module_name == "sifr.meta"
                && import_from
                    .names
                    .iter()
                    .any(|alias| alias.name.as_str() == "StaticProgram" && alias.asname.is_none())
            {
                ctx.canonical_static_program_marker_imported = true;
            }
            if module_name == "sifr.meta"
                && import_from
                    .names
                    .iter()
                    .any(|alias| alias.name.as_str() == "MethodSlots" && alias.asname.is_none())
            {
                ctx.canonical_method_slots_marker_imported = true;
            }
            if module_name == "sifr.meta"
                && import_from
                    .names
                    .iter()
                    .any(|alias| alias.name.as_str() == "Context" && alias.asname.is_none())
            {
                ctx.canonical_context_marker_imported = true;
            }
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
                        if !ctx.class_types.contains_key(&local)
                            || (local == "Error"
                                && ctx
                                    .class_types
                                    .get(&local)
                                    .is_some_and(sifr_type_system::Type::is_builtin_error_base))
                        {
                            let imported_class_ty = super::imported_class_identity::type_for_import(
                                class_ty,
                                &module_key,
                                &class_aliases,
                            );
                            ctx.class_types
                                .insert(local.clone(), imported_class_ty.clone());
                            if let Some(defaults) = externals
                                .class_field_defaults
                                .get(&module_key)
                                .and_then(|classes| classes.get(name))
                            {
                                ctx.class_field_defaults
                                    .insert(local.clone(), defaults.clone());
                            }
                            register_imported_structural_identity_inputs(
                                ctx,
                                externals,
                                &module_key,
                                name,
                                &local,
                            );
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
                            register_imported_rust_opaque_class(
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
                            if externals.is_error_type(&module_key, name) {
                                ctx.error_types.insert(local.clone());
                            } else if local == "Error" {
                                ctx.error_types.remove(&local);
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
            if let Some(module_aliases) = externals.generic_type_aliases.get(&module_key) {
                for name in &names {
                    let local = local_name_for(name);
                    if let Some((type_params, alias)) = module_aliases.get(name) {
                        ctx.scope.define_generic_type_alias(
                            local,
                            type_params.clone(),
                            super::imported_class_identity::type_for_import(
                                alias,
                                &module_key,
                                &class_aliases,
                            ),
                        );
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
                            && let Some(binding_id) =
                                ctx.scope.lookup(&local).map(|binding| binding.binding_id)
                        {
                            ctx.const_integer_values
                                .record(local, binding_id, value.clone());
                        }
                    }
                }
            }
        }
    }
}

fn register_imported_structural_identity_inputs(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    external_name: &str,
    local_name: &str,
) {
    let defaults_supported = externals
        .class_field_defaults
        .get(module_name)
        .and_then(|classes| classes.get(external_name))
        .into_iter()
        .flatten()
        .all(|(index, value)| {
            sifr_ir::canonical_structural_identity_value(value).is_some()
                || externals
                    .class_adapter_selections
                    .get(module_name)
                    .and_then(|classes| classes.get(external_name))
                    .and_then(|selection| selection.field_plans.get(*index))
                    .is_some_and(|field| {
                        matches!(field.default, sifr_ir::AdapterFieldDefault::Factory(_))
                    })
        });
    let metadata_supported = externals
        .declaration_metadata
        .get(module_name)
        .into_iter()
        .flatten()
        .filter(|metadata| metadata.owner == external_name)
        .all(|metadata| sifr_ir::canonical_structural_identity_value(&metadata.value).is_some());
    ctx.imported_structural_identity_inputs.insert(
        local_name.to_string(),
        defaults_supported && metadata_supported,
    );
}
