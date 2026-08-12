use super::{
    import_resolution, imported_defaults, name_diagnostics, ExternalDefs, LowerCtx, TextRange, Type,
};

pub(in crate::lower) fn resolve_compiled_private_imports(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    names: &[String],
    aliases: &[(String, String)],
    imported_name_range: &impl Fn(&str) -> TextRange,
) -> bool {
    if !import_resolution::external_module_exists(externals, module_name) {
        return false;
    }
    for name in names {
        let local = local_name_for(name, aliases);
        if resolve_function(ctx, externals, module_name, name, &local)
            || resolve_class(ctx, externals, module_name, name, &local)
            || resolve_constant(ctx, externals, module_name, name, &local)
        {
            continue;
        }
        name_diagnostics::missing_member(ctx, module_name, name, imported_name_range(name));
    }
    true
}

fn local_name_for(original: &str, aliases: &[(String, String)]) -> String {
    aliases
        .iter()
        .find(|(source, _)| source == original)
        .map(|(_, alias)| alias.clone())
        .unwrap_or_else(|| original.to_string())
}

fn resolve_function(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    name: &str,
    local: &str,
) -> bool {
    let Some(ft) = externals
        .functions
        .get(module_name)
        .and_then(|module_fns| module_fns.get(name))
    else {
        return false;
    };
    ctx.functions.insert(local.to_string(), ft.clone());
    import_function_metadata(ctx, externals, module_name, name, local);
    true
}

fn import_function_metadata(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    name: &str,
    local: &str,
) {
    imported_defaults::import_rust_threadsafe_callback_target(
        ctx,
        externals,
        module_name,
        name,
        local,
    );
    if let Some(module_intrinsics) = externals.compiler_intrinsics.get(module_name) {
        imported_defaults::import_callable_compiler_intrinsic(ctx, module_intrinsics, name, local);
    }
    if let Some(module_defaults) = externals.function_defaults.get(module_name) {
        imported_defaults::import_callable_defaults(ctx, module_defaults, name, local);
    }
    if let Some(module_varargs) = externals.function_varargs.get(module_name) {
        imported_defaults::import_callable_vararg(ctx, module_varargs, name, local);
    }
    if let Some(module_shapes) = externals.function_python_call_shapes.get(module_name) {
        imported_defaults::import_python_call_shape(ctx, module_shapes, name, local);
    }
    if let Some(module_workloads) = externals.function_workloads.get(module_name) {
        imported_defaults::import_callable_workload(ctx, module_workloads, name, local);
    }
    if let Some(type_vars) = externals
        .generic_functions
        .get(module_name)
        .and_then(|module_gf| module_gf.get(name))
    {
        ctx.generic_functions
            .insert(local.to_string(), type_vars.clone());
    }
    if let Some(owner_bounds) = externals
        .type_param_bounds
        .get(module_name)
        .and_then(|module_bounds| module_bounds.get(name))
    {
        ctx.type_param_bounds
            .insert(local.to_string(), owner_bounds.clone());
    }
}

fn resolve_class(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    name: &str,
    local: &str,
) -> bool {
    let Some(class_ty) = externals
        .classes
        .get(module_name)
        .and_then(|module_classes| module_classes.get(name))
    else {
        return false;
    };
    ctx.class_types.insert(local.to_string(), class_ty.clone());
    super::imports::register_imported_class_instance_methods(
        ctx,
        externals,
        module_name,
        name,
        local,
    );
    import_class_type_params(ctx, externals, module_name, name, local);
    if let Some(module_bounds) = externals.type_param_bounds.get(module_name) {
        super::generic_method_requirements::import_generic_method_requirements(
            ctx,
            module_bounds,
            name,
            local,
        );
    }
    if externals.is_error_type(module_name, name) {
        ctx.error_types.insert(local.to_string());
    }
    register_constructor(ctx, externals, module_name, name, local, class_ty);
    import_class_bounds(ctx, externals, module_name, name, local);
    true
}

fn import_class_type_params(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    name: &str,
    local: &str,
) {
    let Some(type_params) = externals
        .class_type_params
        .get(module_name)
        .and_then(|module_class_type_params| module_class_type_params.get(name))
    else {
        return;
    };
    ctx.class_declared_type_params
        .insert(local.to_string(), type_params.clone());
    ctx.class_declared_type_params
        .entry(name.to_string())
        .or_insert_with(|| type_params.clone());
    if !type_params.is_empty() {
        ctx.generic_functions
            .insert(local.to_string(), type_params.clone());
    }
}

fn register_constructor(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    name: &str,
    local: &str,
    class_ty: &Type,
) {
    let Some(ft) = super::imported_class_identity::imported_constructor_function_type(class_ty)
    else {
        return;
    };
    ctx.functions.insert(local.to_string(), ft);
    if let Some(module_defaults) = externals.function_defaults.get(module_name) {
        imported_defaults::import_class_method_defaults(ctx, module_defaults, name, local);
    }
    if let Some(module_varargs) = externals.function_varargs.get(module_name) {
        imported_defaults::import_class_method_varargs(ctx, module_varargs, name, local);
    }
    if let Some(module_workloads) = externals.function_workloads.get(module_name) {
        imported_defaults::import_class_method_workloads(ctx, module_workloads, name, local);
    }
}

fn import_class_bounds(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    name: &str,
    local: &str,
) {
    if let Some(owner_bounds) = externals
        .type_param_bounds
        .get(module_name)
        .and_then(|module_bounds| module_bounds.get(name))
    {
        ctx.type_param_bounds
            .insert(local.to_string(), owner_bounds.clone());
    }
}

fn resolve_constant(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    name: &str,
    local: &str,
) -> bool {
    let Some(const_ty) = externals
        .constants
        .get(module_name)
        .and_then(|module_consts| module_consts.get(name))
    else {
        return false;
    };
    ctx.scope.define(local.to_string(), const_ty.clone());
    if let Some(value) = externals
        .constant_integer_values
        .get(module_name)
        .and_then(|module_values| module_values.get(name))
    {
        ctx.const_integer_values
            .insert(local.to_string(), value.clone());
    }
    true
}
