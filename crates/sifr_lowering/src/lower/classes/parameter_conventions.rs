use super::{typing_and_functions::ast_convention_to_param, HirExpr, HirParam, LowerCtx};
use crate::lower::{expressions::lower_expr, python_interop::is_python_omit};
use sifr_python_ast::{AstParamConvention, Expr};
use sifr_type_system::{ParamConvention, Type};

pub(in crate::lower) fn class_method_param_convention(
    syntax: AstParamConvention,
    ty: &Type,
    ctx: &LowerCtx,
) -> ParamConvention {
    let declared = ast_convention_to_param(syntax, ty);
    if syntax.is_owned() || ctx.must_use_obligation_for_type(ty).is_some() || declared.is_mutable()
    {
        declared
    } else {
        ParamConvention::default()
    }
}

pub(in crate::lower) fn class_method_param_default(
    default: Option<&Expr>,
    has_python_interop: bool,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    default.and_then(|value| {
        (!has_python_interop || !is_python_omit(value))
            .then(|| lower_expr(value, ctx))
            .flatten()
    })
}

pub(in crate::lower) fn prepare_class_method_param_ownership(
    params: &[HirParam],
    skips_normal_body_lowering: bool,
    ctx: &mut LowerCtx,
) {
    for param in params {
        if !skips_normal_body_lowering && param.convention.is_owned() {
            ctx.record_must_use_binding(&param.name, &param.ty);
        }
        if param.convention.is_borrowed()
            && param.ty.ownership() == sifr_type_system::OwnershipKind::Move
            && !matches!(param.ty, Type::TypeVar(_))
        {
            ctx.borrowed_params.insert(param.name.clone());
        }
    }
}
