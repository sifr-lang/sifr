use super::LowerCtx;

pub(super) fn ensure_mutable_parameter_binding(
    ctx: &mut LowerCtx,
    name: &str,
    operation: &str,
) -> bool {
    if ctx
        .scope
        .lookup(name)
        .is_some_and(|info| info.is_parameter_binding() && !info.is_mutable_binding)
    {
        ctx.error(format!(
            "cannot {operation} immutable parameter `{name}`: add `mut` to the parameter declaration"
        ));
        return false;
    }
    true
}
