use super::{
    typing_and_functions::ast_convention_to_param, FunctionType, HirExpr, HirParam, LowerCtx,
};
use crate::lower::{expressions::lower_expr, python_interop::is_python_omit};
use sifr_python_ast::{AstParamConvention, Expr, Parameters};
use sifr_type_system::{ParamConvention, ReceiverConvention, Type};

pub(in crate::lower) fn declared_receiver_convention(
    parameters: &Parameters,
) -> ReceiverConvention {
    let convention = parameters
        .args
        .first()
        .map_or_else(AstParamConvention::borrow, |parameter| {
            parameter.parameter.convention
        });
    if convention.is_owned() && convention.is_mutable() {
        ReceiverConvention::OwnedMutable
    } else if convention.is_owned() {
        ReceiverConvention::Owned
    } else if convention.is_mutable() {
        ReceiverConvention::MutableBorrow
    } else {
        ReceiverConvention::SharedBorrow
    }
}

pub(in crate::lower) fn fixed_trait_receiver_convention(
    method: &str,
) -> Option<ReceiverConvention> {
    match method {
        "__eq__" | "__lt__" | "__str__" | "__repr__" | "__getitem__" => {
            Some(ReceiverConvention::SharedBorrow)
        }
        "__add__" | "__sub__" | "__mul__" | "__truediv__" | "__mod__" | "__neg__" => {
            Some(ReceiverConvention::Owned)
        }
        _ => None,
    }
}

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

pub(in crate::lower) fn inherit_class_method_metadata(
    ctx: &mut LowerCtx,
    parent: &str,
    child: &str,
    method: &str,
) {
    let parent_key = format!("{parent}.{method}");
    let child_key = format!("{child}.{method}");
    if ctx.class_instance_methods.contains(&parent_key) {
        ctx.class_instance_methods.insert(child_key.clone());
    }
    let origin = ctx
        .class_method_origins
        .get(&parent_key)
        .cloned()
        .unwrap_or_else(|| parent.to_string());
    ctx.class_method_origins.insert(child_key, origin);
}

pub(in crate::lower) fn inherit_class_methods(
    methods: &mut Vec<(String, FunctionType)>,
    parent_methods: &[(String, FunctionType)],
    ctx: &mut LowerCtx,
    parent: &str,
    child: &str,
) {
    for (name, function_type) in parent_methods {
        methods.push((name.clone(), function_type.clone()));
        inherit_class_method_metadata(ctx, parent, child, name);
    }
}

pub(in crate::lower) fn record_declared_class_method_metadata(
    ctx: &mut LowerCtx,
    class_name: &str,
    method_name: &str,
    is_instance: bool,
) {
    let key = format!("{class_name}.{method_name}");
    if is_instance {
        ctx.class_instance_methods.insert(key.clone());
    } else {
        ctx.class_instance_methods.remove(&key);
    }
    ctx.class_method_origins.insert(key, class_name.to_string());
}

pub(in crate::lower) fn replace_method_signature(
    methods: &mut Vec<(String, FunctionType)>,
    method_name: String,
    function_type: FunctionType,
) {
    if let Some(entry) = methods.iter_mut().find(|(name, _)| name == &method_name) {
        *entry = (method_name, function_type);
    } else {
        methods.push((method_name, function_type));
    }
}

pub(in crate::lower) fn reject_owned_affine_operator_parameter(
    ctx: &mut LowerCtx,
    class_name: &str,
    method_name: &str,
    parameter_name: &str,
    ty: &Type,
    convention: ParamConvention,
    range: ruff_text_size::TextRange,
) {
    if (super::is_operator_dunder(method_name) || matches!(method_name, "__getitem__" | "__pow__"))
        && convention.is_owned()
        && ty.contains_affine_resource()
    {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
            format!(
                "operator '{class_name}.{method_name}' cannot consume affine parameter '{parameter_name}'; use an explicit method call"
            ),
            range,
        );
    }
}

pub(in crate::lower) fn class_declared_method_param_convention(
    syntax: sifr_python_ast::AstParamConvention,
    ty: &Type,
    ctx: &mut LowerCtx,
    owner: (&str, &str),
    parameter: (&str, ruff_text_size::TextRange),
) -> ParamConvention {
    let convention = class_method_param_convention(syntax, ty, ctx);
    reject_owned_affine_operator_parameter(
        ctx,
        owner.0,
        owner.1,
        parameter.0,
        ty,
        convention,
        parameter.1,
    );
    convention
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

pub(in crate::lower) fn prepare_method_param_ownership(
    params: &[HirParam],
    method_name: &str,
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
            && (!super::is_operator_dunder(method_name) || param.ty.contains_affine_resource())
        {
            ctx.borrowed_params.insert(param.name.clone());
        }
    }
}
