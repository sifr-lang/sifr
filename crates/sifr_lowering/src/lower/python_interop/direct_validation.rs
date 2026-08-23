use super::{
    HirParam, LowerCtx, PythonInteropDeclaration, PythonInteropEffect, PythonParameterKind, Type,
    arrow, buffer, dlpack, invalid_shape, is_direct_type, unsupported_conversion,
};
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{CompilerIntrinsicId, HirExpr};

pub(in crate::lower) fn validate_raw_conversion_intrinsic(
    intrinsic: CompilerIntrinsicId,
    args: &[HirExpr],
    result_type: &Type,
    span: TextRange,
    ctx: &mut LowerCtx,
) {
    let conversion_type = match intrinsic {
        CompilerIntrinsicId::PythonFromValue => args.first().map(HirExpr::ty),
        CompilerIntrinsicId::PythonKwarg => args.get(1).map(HirExpr::ty),
        CompilerIntrinsicId::PythonToValue => match result_type.resolve_alias() {
            Type::Result(ok_type, _error_type) => Some(ok_type.as_ref()),
            _ => None,
        },
        _ => return,
    };
    let Some(conversion_type) = conversion_type else {
        return;
    };
    if !is_direct_type(conversion_type, true, ctx) {
        ctx.error_with_code_at(
            DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE,
            format!(
                "unsupported raw Python conversion type: `{}` is not in the declaration conversion set",
                conversion_type.display_name()
            ),
            span,
        );
    }
}

pub(super) fn validate_direct_parameters(
    declaration: &PythonInteropDeclaration,
    params: &[HirParam],
    ctx: &mut LowerCtx,
) {
    let mut saw_omittable_positional = false;
    for shape in &declaration.parameters {
        let Some(param) = params.iter().find(|parameter| parameter.name == shape.name) else {
            continue;
        };
        if declaration
            .callbacks
            .iter()
            .any(|callback| callback.parameter_name == param.name)
        {
            continue;
        }
        if shape.kind == PythonParameterKind::Positional && shape.omit_when_absent {
            saw_omittable_positional = true;
        }
        if shape.kind == PythonParameterKind::PositionalVariadic && saw_omittable_positional {
            invalid_shape(
                ctx,
                "typed `*args` cannot follow an omittable positional parameter",
                shape.span,
            );
        }
        if declaration.buffer.as_ref().is_some_and(|buffer| {
            buffer.access == sifr_ir::PythonBufferAccess::Write
                && !param.convention.is_owned()
                && buffer::contains_python_identity(&param.ty, ctx)
        }) {
            buffer::invalid(
                ctx,
                &format!(
                    "writable buffer producer parameter '{}' can carry an existing Python identity and must transfer ownership with `own`",
                    param.name
                ),
                shape.span,
            );
        }
        if matches!(param.ty.resolve_alias(), Type::PythonArrow(_)) {
            if !param.convention.is_owned() {
                arrow::invalid(
                    ctx,
                    &format!(
                        "Arrow consumer parameter '{}' must transfer ownership with `own`",
                        param.name
                    ),
                    shape.span,
                );
            }
            if param.convention.is_mutable() {
                arrow::invalid(
                    ctx,
                    &format!(
                        "Arrow consumer parameter '{}' cannot use mutable ownership; transfer it with plain `own`",
                        param.name
                    ),
                    shape.span,
                );
            }
            if shape.omit_when_absent {
                arrow::invalid(
                    ctx,
                    &format!(
                        "Arrow consumer parameter '{}' cannot be omitted because affine resources require an explicit one-shot transfer",
                        param.name
                    ),
                    shape.span,
                );
            }
            if declaration.effect == PythonInteropEffect::Async {
                arrow::invalid(
                    ctx,
                    &format!(
                        "Arrow consumer parameter '{}' is not supported on async Python declarations",
                        param.name
                    ),
                    shape.span,
                );
            }
        }
        if matches!(param.ty.resolve_alias(), Type::PythonDlpackTensor(_)) {
            if !param.convention.is_owned() || param.convention.is_mutable() {
                dlpack::invalid(
                    ctx,
                    &format!(
                        "DLPack tensor consumer parameter '{}' must transfer ownership with plain `own`",
                        param.name
                    ),
                    shape.span,
                );
            }
            if shape.omit_when_absent {
                dlpack::invalid(
                    ctx,
                    &format!(
                        "DLPack tensor consumer parameter '{}' cannot be omitted because transfer is one-shot",
                        param.name
                    ),
                    shape.span,
                );
            }
            if declaration.effect == PythonInteropEffect::Async {
                dlpack::invalid(
                    ctx,
                    &format!(
                        "DLPack tensor consumer parameter '{}' is not supported on async Python declarations",
                        param.name
                    ),
                    shape.span,
                );
            }
        }
        let supported = match shape.kind {
            PythonParameterKind::Positional | PythonParameterKind::KeywordOnly => {
                matches!(
                    param.ty.resolve_alias(),
                    Type::PythonArrow(_) | Type::PythonDlpackTensor(_)
                ) || is_direct_type(&param.ty, true, ctx)
            }
            PythonParameterKind::PositionalVariadic => {
                matches!(param.ty.resolve_alias(), Type::List(element) if is_direct_type(element, false, ctx))
            }
            PythonParameterKind::KeywordVariadic => {
                matches!(param.ty.resolve_alias(), Type::Dict(key, value) if key.resolve_alias() == &Type::Str && is_direct_type(value, false, ctx))
            }
        };
        if !supported {
            unsupported_conversion(
                ctx,
                &format!(
                    "parameter '{}' has unsupported type `{}` for {:?}",
                    param.name,
                    param.ty.display_name(),
                    shape.kind
                ),
                shape.span,
            );
        }
    }
}
