use crate::hir_nodes::HirImport;
use sifr_python_ast::{Expr, ExprCall};
use sifr_type_system::{FunctionType, Type};

use super::imported_defaults::{
    import_callable_defaults, import_callable_vararg, import_class_method_defaults,
    import_class_method_varargs,
};
use super::{collect_type_vars, ExternalDefs, LowerCtx};

pub(in crate::lower) fn resolve_python_compat_call_alias(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<String> {
    let Expr::Attribute(attr) = call.func.as_ref() else {
        return None;
    };
    let Expr::Name(name) = attr.value.as_ref() else {
        return None;
    };
    if name.id.as_str() == "collections" && attr.attr.as_str() == "defaultdict" {
        return Some("defaultdict".to_string());
    }
    let module_name = match name.id.as_str() {
        "collections" => Some("sifr.collections"),
        "heapq" => Some("sifr.heapq"),
        "math" => Some("sifr.math"),
        _ => None,
    }?;
    let externals = ctx.externals.clone();
    ensure_synthetic_stdlib_import(ctx, &externals, module_name, attr.attr.as_str())
}

pub(in crate::lower) fn resolve_bare_python_compat_call_alias(
    func_name: &str,
    ctx: &mut LowerCtx,
) -> Option<String> {
    if ctx.scope.lookup(func_name).is_some()
        || ctx.functions.contains_key(func_name)
        || ctx.class_types.contains_key(func_name)
    {
        return None;
    }
    let (module_name, member_name) = match func_name {
        "defaultdict" => return Some("defaultdict".to_string()),
        "deque" => ("sifr.collections", "deque"),
        "Counter" => ("sifr.collections", "Counter"),
        _ => return None,
    };
    let externals = ctx.externals.clone();
    ensure_synthetic_stdlib_import(ctx, &externals, module_name, member_name)
}

pub(in crate::lower) fn ensure_synthetic_stdlib_import(
    ctx: &mut LowerCtx,
    externals: &ExternalDefs,
    module_name: &str,
    member_name: &str,
) -> Option<String> {
    let synthetic_key = format!("{module_name}:{member_name}");
    if let Some(alias) = ctx.synthetic_import_aliases.get(&synthetic_key) {
        return Some(alias.clone());
    }

    let alias = format!("__compat_{}_{}", module_name.replace('.', "_"), member_name);
    let mut found = false;

    if let Some(module_fns) = externals.functions.get(module_name) {
        if let Some(ft) = module_fns.get(member_name) {
            ctx.functions.insert(alias.clone(), ft.clone());
            if let Some(module_defaults) = externals.function_defaults.get(module_name) {
                import_callable_defaults(ctx, module_defaults, member_name, &alias);
            }
            if let Some(module_varargs) = externals.function_varargs.get(module_name) {
                import_callable_vararg(ctx, module_varargs, member_name, &alias);
            }
            if let Some(module_gf) = externals.generic_functions.get(module_name) {
                if let Some(type_vars) = module_gf.get(member_name) {
                    ctx.generic_functions
                        .insert(alias.clone(), type_vars.clone());
                }
            }
            if !ctx.generic_functions.contains_key(&alias) {
                let mut type_vars = Vec::new();
                for (_, param_ty, _) in &ft.params {
                    collect_type_vars(param_ty, &mut type_vars);
                }
                collect_type_vars(&ft.return_type, &mut type_vars);
                type_vars.sort();
                type_vars.dedup();
                if !type_vars.is_empty() {
                    ctx.generic_functions.insert(alias.clone(), type_vars);
                }
            }
            if let Some(module_bounds) = externals.type_param_bounds.get(module_name) {
                if let Some(owner_bounds) = module_bounds.get(member_name) {
                    ctx.type_param_bounds
                        .insert(alias.clone(), owner_bounds.clone());
                }
            }
            found = true;
        }
    }

    if !found {
        if let Some(module_classes) = externals.classes.get(module_name) {
            if let Some(class_ty) = module_classes.get(member_name) {
                ctx.class_types.insert(alias.clone(), class_ty.clone());
                if let Some(module_class_type_params) = externals.class_type_params.get(module_name)
                {
                    if let Some(type_params) = module_class_type_params.get(member_name) {
                        ctx.class_declared_type_params
                            .insert(alias.clone(), type_params.clone());
                        if !type_params.is_empty() {
                            ctx.generic_functions
                                .insert(alias.clone(), type_params.clone());
                        }
                    }
                }
                if externals.error_types.contains(member_name) {
                    ctx.error_types.insert(alias.clone());
                }
                if let Type::Class {
                    fields, methods, ..
                } = class_ty
                {
                    let ft = if let Some((_, new_ft)) = methods.iter().find(|(n, _)| n == "new") {
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
                    ctx.functions.insert(alias.clone(), ft);
                    if let Some(module_defaults) = externals.function_defaults.get(module_name) {
                        import_class_method_defaults(ctx, module_defaults, member_name, &alias);
                    }
                    if let Some(module_varargs) = externals.function_varargs.get(module_name) {
                        import_class_method_varargs(ctx, module_varargs, member_name, &alias);
                    }
                }
                if let Some(module_bounds) = externals.type_param_bounds.get(module_name) {
                    if let Some(owner_bounds) = module_bounds.get(member_name) {
                        ctx.type_param_bounds
                            .insert(alias.clone(), owner_bounds.clone());
                    }
                }
                found = true;
            }
        }
    }

    if !found {
        if let Some(module_consts) = externals.constants.get(module_name) {
            if let Some(const_ty) = module_consts.get(member_name) {
                ctx.scope.define(alias.clone(), const_ty.clone());
                if let Some(value) = externals
                    .constant_integer_values
                    .get(module_name)
                    .and_then(|module_values| module_values.get(member_name))
                {
                    ctx.const_integer_values
                        .insert(alias.clone(), value.clone());
                }
                found = true;
            }
        }
    }

    if !found {
        return None;
    }

    ctx.synthetic_import_aliases
        .insert(synthetic_key, alias.clone());
    ctx.synthetic_imports.push(HirImport {
        module: module_name.to_string(),
        names: vec![member_name.to_string()],
        aliases: vec![(member_name.to_string(), alias.clone())],
    });
    Some(alias)
}
