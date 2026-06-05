use super::{collect_type_vars, infer_type_var_bindings, substitute_type_vars};
use crate::hir_nodes::HirExpr;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

pub(in crate::lower) fn refine_constructor_return_type_from_args(
    ft: &FunctionType,
    args: &[HirExpr],
    return_ty: &Type,
) -> Type {
    let mut type_vars = Vec::new();
    collect_type_vars(return_ty, &mut type_vars);
    if type_vars.is_empty() {
        return return_ty.clone();
    }

    let mut bindings = HashMap::new();
    for (arg, (_, param_ty, _)) in args.iter().zip(ft.params.iter()) {
        infer_type_var_bindings(param_ty, arg.ty(), &mut bindings);
    }
    bindings.retain(|_, bound_ty| {
        !matches!(
            bound_ty.resolve_alias(),
            Type::Any | Type::Unknown | Type::TypeVar(_)
        )
    });
    if bindings.is_empty() {
        return return_ty.clone();
    }

    substitute_type_vars(return_ty, &bindings)
}
