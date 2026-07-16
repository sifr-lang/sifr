use std::collections::HashMap;

use crate::hir_nodes::HirExpr;
use sifr_ir::CompilerIntrinsicId;
use sifr_ir::PythonParameterKind;

use super::workload_annotations::WorkloadKind;
use super::{ExternalDefs, LowerCtx};

pub(in crate::lower) fn import_user_callable(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    external_name: &str,
    local_name: &str,
    class_aliases: &HashMap<String, String>,
) -> bool {
    let Some(function) = externals
        .functions
        .get(module_name)
        .and_then(|functions| functions.get(external_name))
    else {
        return false;
    };
    ctx.functions.insert(
        local_name.to_string(),
        super::imported_class_identity::function_type_for_import(
            function,
            module_name,
            class_aliases,
        ),
    );
    import_callable_generic_metadata(ctx, externals, module_name, external_name, local_name);
    if let Some(values) = externals.compiler_intrinsics.get(module_name) {
        import_callable_compiler_intrinsic(ctx, values, external_name, local_name);
    }
    if let Some(values) = externals.function_defaults.get(module_name) {
        import_callable_defaults(ctx, values, external_name, local_name);
    }
    if let Some(values) = externals.function_varargs.get(module_name) {
        import_callable_vararg(ctx, values, external_name, local_name);
    }
    if let Some(values) = externals.function_python_call_shapes.get(module_name) {
        import_python_call_shape(ctx, values, external_name, local_name);
    }
    if let Some(values) = externals.function_workloads.get(module_name) {
        import_callable_workload(ctx, values, external_name, local_name);
    }
    true
}

fn import_callable_generic_metadata(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    external_name: &str,
    local_name: &str,
) {
    if let Some(type_params) = externals
        .generic_functions
        .get(module_name)
        .and_then(|module_generics| module_generics.get(external_name))
    {
        ctx.generic_functions
            .insert(local_name.to_string(), type_params.clone());
    }
    if let Some(bounds) = externals
        .type_param_bounds
        .get(module_name)
        .and_then(|module_bounds| module_bounds.get(external_name))
    {
        ctx.type_param_bounds
            .insert(local_name.to_string(), bounds.clone());
    }
}

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

pub(in crate::lower) fn import_callable_compiler_intrinsic(
    ctx: &mut LowerCtx,
    module_intrinsics: &HashMap<String, CompilerIntrinsicId>,
    external_name: &str,
    local_name: &str,
) {
    if let Some(intrinsic) = module_intrinsics.get(external_name) {
        ctx.compiler_intrinsics
            .insert(local_name.to_string(), *intrinsic);
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

pub(in crate::lower) fn import_python_call_shape(
    ctx: &mut LowerCtx,
    module_shapes: &HashMap<String, Vec<PythonParameterKind>>,
    external_name: &str,
    local_name: &str,
) {
    if let Some(shapes) = module_shapes.get(external_name) {
        ctx.python_call_shapes
            .insert(local_name.to_string(), shapes.clone());
    }
}

pub(in crate::lower) fn import_callable_workload(
    ctx: &mut LowerCtx,
    module_workloads: &HashMap<String, String>,
    external_name: &str,
    local_name: &str,
) {
    let Some(label) = module_workloads.get(external_name) else {
        return;
    };
    if let Some(workload) = WorkloadKind::from_label(label) {
        ctx.function_workload_annotations
            .insert(local_name.to_string(), workload);
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

pub(in crate::lower) fn import_class_method_workloads(
    ctx: &mut LowerCtx,
    module_workloads: &HashMap<String, String>,
    external_name: &str,
    local_name: &str,
) {
    import_callable_workload(ctx, module_workloads, external_name, local_name);
    let method_prefix = format!("{external_name}.");
    for (workload_name, label) in module_workloads {
        if let Some(suffix) = workload_name.strip_prefix(&method_prefix) {
            if let Some(workload) = WorkloadKind::from_label(label) {
                ctx.function_workload_annotations
                    .insert(format!("{local_name}.{suffix}"), workload);
                ctx.function_workload_annotations
                    .insert(format!("{external_name}.{suffix}"), workload);
            }
        }
    }
}
