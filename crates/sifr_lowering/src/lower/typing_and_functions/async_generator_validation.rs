use super::{
    first_yield_range_in_stmts, function_body_contains_yield, DiagnosticCode, HirParam, LowerCtx,
    Ranged, StmtFunctionDef, TextRange, Type,
};

pub(in crate::lower) fn reject_declared_async_generator_boundary(
    function_name: &str,
    params: &[HirParam],
    return_type: &Type,
    yield_range: TextRange,
    ctx: &mut LowerCtx,
) {
    let nested_captures = ctx.nested_function_captures.get(function_name).cloned();
    reject_affine_async_generator_boundary(
        params,
        return_type,
        nested_captures.as_deref(),
        yield_range,
        ctx,
    );
}

pub(in crate::lower) fn reject_unsupported_nested_async_generator(
    func: &StmtFunctionDef,
    return_type: &Type,
    ctx: &mut LowerCtx,
) {
    if !func.is_async || !function_body_contains_yield(&func.body) {
        return;
    }
    let captures = ctx
        .nested_function_captures
        .get(func.name.as_str())
        .cloned()
        .unwrap_or_default();
    let yield_range = first_yield_range_in_stmts(&func.body).unwrap_or_else(|| func.name.range());
    reject_affine_async_generator_boundary(&[], return_type, Some(&captures), yield_range, ctx);
}

pub(in crate::lower) fn reject_affine_async_generator_boundary(
    params: &[HirParam],
    return_type: &Type,
    nested_captures: Option<&[(String, Type)]>,
    yield_range: TextRange,
    ctx: &mut LowerCtx,
) {
    if let Some(captures) = nested_captures {
        if let Some((name, ty)) = captures
            .iter()
            .find(|(_, ty)| ty.contains_affine_resource())
        {
            ctx.error_with_code_at(
                DiagnosticCode::PYZC_INVALID_DECLARATION,
                format!(
                    "nested async generator capture '{name}' of type '{}' contains an affine Python resource and cannot enter the lazy Send factory",
                    ty.display_name()
                ),
                yield_range,
            );
        } else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                "nested async generators require dedicated lazy materialization code generation and are not supported yet"
                    .to_string(),
                yield_range,
            );
        }
        return;
    }
    if params
        .iter()
        .any(|param| param.ty.contains_affine_resource())
        || return_type.contains_affine_resource()
    {
        ctx.error_with_code_at(
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            "async generators cannot capture or yield affine Python resources because their lazy factory must be Send"
                .to_string(),
            yield_range,
        );
    }
}
