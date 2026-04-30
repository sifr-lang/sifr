use super::LowerCtx;

pub(super) fn ensure_mutable_parameter_binding(ctx: &mut LowerCtx, name: &str) -> bool {
    if ctx
        .scope
        .lookup(name)
        .is_some_and(|info| info.is_parameter_binding() && !info.is_mutable_binding())
    {
        super::ownership_diagnostics::immutable_parameter_mutation(ctx, name);
        return false;
    }
    true
}
