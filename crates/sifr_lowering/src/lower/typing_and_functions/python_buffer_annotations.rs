use super::{Expr, LowerCtx, Ranged, Type, invalid_type_annotation, resolve_annotation_expr};

pub(super) fn resolve_python_buffer_annotation(slice: &Expr, ctx: &mut LowerCtx) -> Type {
    if matches!(slice, Expr::Tuple(_)) {
        invalid_type_annotation(
            ctx,
            "python.Buffer type annotation requires exactly 1 element type",
            slice.range(),
        );
        return Type::Any;
    }
    let element = resolve_annotation_expr(slice, ctx);
    if !matches!(element.resolve_alias(), Type::FixedInt(_) | Type::Float) {
        invalid_type_annotation(
            ctx,
            format!(
                "python.Buffer element type must be a fixed-width integer or float, got `{}`",
                element.display_name()
            ),
            slice.range(),
        );
        return Type::Any;
    }
    Type::PythonBuffer(Box::new(element))
}
