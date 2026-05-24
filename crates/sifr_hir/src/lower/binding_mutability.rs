use super::LowerCtx;
use ruff_text_size::TextRange;

pub(in crate::lower) fn ensure_mutable_parameter_binding(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) -> bool {
    if ctx
        .scope
        .lookup(name)
        .is_some_and(|info| info.is_parameter_binding() && !info.is_mutable_binding())
    {
        super::ownership_diagnostics::immutable_parameter_mutation(ctx, name, range);
        return false;
    }
    true
}
