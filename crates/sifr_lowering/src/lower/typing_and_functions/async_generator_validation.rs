use super::{DiagnosticCode, HirParam, LowerCtx, TextRange, Type};

pub(super) fn reject_affine_async_generator_boundary(
    params: &[HirParam],
    return_type: &Type,
    yield_range: TextRange,
    ctx: &mut LowerCtx,
) {
    if params
        .iter()
        .any(|param| param.ty.contains_affine_resource())
        || return_type.contains_affine_resource()
    {
        ctx.error_with_code_at(
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            "async generators cannot capture or yield affine Python buffers because their lazy factory must be Send"
                .to_string(),
            yield_range,
        );
    }
}
