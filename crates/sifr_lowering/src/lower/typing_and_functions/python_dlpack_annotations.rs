use super::{invalid_type_annotation, resolve_annotation_expr, Expr, LowerCtx, Ranged, Type};

pub(super) fn resolve_python_dlpack_tensor_annotation(slice: &Expr, ctx: &mut LowerCtx) -> Type {
    if matches!(slice, Expr::Tuple(_)) {
        invalid_type_annotation(
            ctx,
            "python.DlpackTensor type annotation requires exactly 1 element type",
            slice.range(),
        );
        return Type::Any;
    }
    let element = resolve_annotation_expr(slice, ctx);
    if !matches!(
        element.resolve_alias(),
        Type::FixedInt(_) | Type::Float | Type::Bool
    ) {
        invalid_type_annotation(
            ctx,
            format!(
                "python.DlpackTensor element type must be a fixed-width integer, float, or bool, got `{}`",
                element.display_name()
            ),
            slice.range(),
        );
        return Type::Any;
    }
    Type::PythonDlpackTensor(Box::new(element))
}
