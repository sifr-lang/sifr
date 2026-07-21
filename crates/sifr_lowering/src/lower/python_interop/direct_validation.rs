use super::*;

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
        if matches!(param.ty.resolve_alias(), Type::PythonArrow(_)) && !param.convention.is_owned()
        {
            arrow::invalid(
                ctx,
                &format!(
                    "Arrow consumer parameter '{}' must transfer ownership with `own`",
                    param.name
                ),
                shape.span,
            );
        }
        let supported = match shape.kind {
            PythonParameterKind::Positional | PythonParameterKind::KeywordOnly => {
                matches!(param.ty.resolve_alias(), Type::PythonArrow(_))
                    || is_direct_type(&param.ty, true, ctx)
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
