use super::{DiagnosticCode, LowerCtx, Ranged, StmtFunctionDef};
use crate::lower::must_use_obligations::MustUseObligationKind;

pub(super) fn reject_live_join_sets_at_function_exit(func: &StmtFunctionDef, ctx: &mut LowerCtx) {
    let mut live_sets = ctx
        .live_join_set_bindings
        .iter()
        .filter(|name| !ctx.scope.is_moved(name))
        .cloned()
        .collect::<Vec<_>>();
    live_sets.sort();
    for name in live_sets {
        ctx.error_with_code_at(
            DiagnosticCode::OWN_USE_AFTER_MOVE,
            format!(
                "JoinSet binding '{name}' accepted task handles and must be consumed with await {name}.join_all() or await {name}.cancel_all() before function exit"
            ),
            func.name.range(),
        );
    }
    ctx.live_join_set_bindings.clear();
    ctx.join_set_terminal_awaitables.clear();
}

pub(super) fn reject_live_must_use_bindings_at_function_exit(
    func: &StmtFunctionDef,
    ctx: &mut LowerCtx,
) {
    let mut live = ctx
        .live_must_use_bindings
        .iter()
        .filter(|(name, _)| ctx.scope.lookup(name.as_str()).is_some() && !ctx.scope.is_moved(name))
        .map(|(name, obligation)| (name.clone(), obligation.clone()))
        .collect::<Vec<_>>();
    live.sort();
    for (name, obligation) in live {
        let requirement = match obligation.kind {
            MustUseObligationKind::ContextOnly => "must be consumed by `with` before function exit",
            MustUseObligationKind::AsyncContextOnly => {
                "must be consumed by `async with` before function exit"
            }
            MustUseObligationKind::CloseLike => {
                "must be closed or transferred before function exit"
            }
        };
        ctx.error_with_code_at(
            DiagnosticCode::OWN_USE_AFTER_MOVE,
            format!("must-use binding '{name}' owns {obligation} and {requirement}"),
            func.name.range(),
        );
    }
    ctx.live_must_use_bindings.clear();
}
