use std::collections::HashMap;

use crate::hir_nodes::HirExpr;

use super::LowerCtx;

pub(in crate::lower) fn import_callable_defaults(
    ctx: &mut LowerCtx,
    module_defaults: &HashMap<String, Vec<(usize, HirExpr)>>,
    external_name: &str,
    local_name: &str,
) {
    if let Some(defaults) = module_defaults.get(external_name) {
        ctx.function_defaults
            .insert(local_name.to_string(), defaults.clone());
    }
}

pub(in crate::lower) fn import_callable_vararg(
    ctx: &mut LowerCtx,
    module_varargs: &HashMap<String, usize>,
    external_name: &str,
    local_name: &str,
) {
    if let Some(vararg_index) = module_varargs.get(external_name) {
        ctx.vararg_functions
            .insert(local_name.to_string(), *vararg_index);
    }
}

pub(in crate::lower) fn import_class_method_defaults(
    ctx: &mut LowerCtx,
    module_defaults: &HashMap<String, Vec<(usize, HirExpr)>>,
    external_name: &str,
    local_name: &str,
) {
    import_callable_defaults(ctx, module_defaults, external_name, local_name);
    let method_prefix = format!("{external_name}.");
    for (default_name, defaults) in module_defaults {
        if let Some(suffix) = default_name.strip_prefix(&method_prefix) {
            ctx.function_defaults
                .insert(format!("{local_name}.{suffix}"), defaults.clone());
        }
    }
}

pub(in crate::lower) fn import_class_method_varargs(
    ctx: &mut LowerCtx,
    module_varargs: &HashMap<String, usize>,
    external_name: &str,
    local_name: &str,
) {
    import_callable_vararg(ctx, module_varargs, external_name, local_name);
    let method_prefix = format!("{external_name}.");
    for (vararg_name, vararg_index) in module_varargs {
        if let Some(suffix) = vararg_name.strip_prefix(&method_prefix) {
            ctx.vararg_functions
                .insert(format!("{local_name}.{suffix}"), *vararg_index);
        }
    }
}
