use super::{LowerCtx, PythonArrowKind, TextRange, Type, invalid_type_annotation};

pub(super) fn resolve_python_arrow_annotation(
    name: &str,
    span: TextRange,
    ctx: &mut LowerCtx,
) -> Type {
    let kind = match name {
        "ArrowArray" => PythonArrowKind::Array,
        "ArrowSchema" => PythonArrowKind::Schema,
        "ArrowStream" => PythonArrowKind::Stream,
        "ArrowDeviceArray" => PythonArrowKind::DeviceArray,
        "ArrowDeviceStream" => PythonArrowKind::DeviceStream,
        _ => {
            invalid_type_annotation(
                ctx,
                format!("unknown Python affine resource type `python.{name}`"),
                span,
            );
            return Type::Any;
        }
    };
    Type::PythonArrow(kind)
}
