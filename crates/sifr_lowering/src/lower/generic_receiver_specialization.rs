use super::{LowerCtx, infer_type_var_bindings, substitute_type_vars};
use crate::hir_nodes::HirExpr;
use sifr_type_system::Type;
use std::collections::HashMap;

pub(in crate::lower) fn refine_generic_class_binding_expr(
    expr: HirExpr,
    method_name: &str,
    args: &[HirExpr],
    ctx: &mut LowerCtx,
) -> HirExpr {
    let HirExpr::Name {
        name,
        binding_id,
        ty,
    } = &expr
    else {
        return expr;
    };
    let Type::Class { methods, .. } = ty.resolve_alias() else {
        return expr;
    };
    let Some((_, method_ft)) = methods
        .iter()
        .find(|(candidate, _)| candidate == method_name)
    else {
        return expr;
    };

    let mut bindings = HashMap::new();
    for (arg, (_, param_ty, _)) in args.iter().zip(method_ft.params.iter()) {
        infer_type_var_bindings(param_ty, arg.ty(), &mut bindings);
    }
    bindings.retain(|_, bound_ty| {
        !matches!(
            bound_ty.resolve_alias(),
            Type::Any | Type::Unknown | Type::TypeVar(_)
        )
    });
    if bindings.is_empty() {
        return expr;
    }

    let refined_ty = substitute_type_vars(ty, &bindings);
    if &refined_ty == ty {
        return expr;
    }

    let _ = ctx.scope.set_type(name, refined_ty.clone());
    ctx.narrow_var_with_flow(
        name,
        refined_ty.clone(),
        "generic-receiver-specialization".to_string(),
        true,
    );
    HirExpr::Name {
        name: name.clone(),
        binding_id: *binding_id,
        ty: refined_ty,
    }
}
