use super::{typing_and_functions::ast_convention_to_param, LowerCtx};
use sifr_python_ast::AstParamConvention;
use sifr_type_system::{ParamConvention, Type};

pub(in crate::lower) fn class_method_param_convention(
    syntax: AstParamConvention,
    ty: &Type,
    ctx: &LowerCtx,
) -> ParamConvention {
    let declared = ast_convention_to_param(syntax, ty);
    if ctx.must_use_obligation_for_type(ty).is_some()
        || (matches!(
            ty.resolve_alias(),
            Type::Callable(..) | Type::AsyncCallable(..)
        ) && declared.is_owned())
        || declared.is_mutable()
    {
        declared
    } else {
        ParamConvention::default()
    }
}
