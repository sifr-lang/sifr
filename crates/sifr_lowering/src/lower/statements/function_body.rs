use super::{lower_stmts, FunctionType, HirStmt, LowerCtx};
use sifr_python_ast::Stmt;

pub(in crate::lower) fn lower_function_stmts(
    stmts: &[Stmt],
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Vec<HirStmt> {
    let previous_handlers = std::mem::take(&mut ctx.rust_threadsafe_callback_move_handlers);
    let result = lower_stmts(stmts, func_type, ctx);
    ctx.rust_threadsafe_callback_move_handlers = previous_handlers;
    result
}

pub(super) fn mark_threadsafe_callback_move_handlers(stmts: &mut [HirStmt], ctx: &LowerCtx) {
    for stmt in stmts {
        if let HirStmt::NestedFunction {
            func,
            move_captures,
            capture_clones,
        } = stmt
        {
            if let Some(captures) = ctx.rust_threadsafe_callback_move_handlers.get(&func.name) {
                *move_captures = true;
                capture_clones.clone_from(captures);
            } else {
                *move_captures = false;
                capture_clones.clear();
            }
        }
    }
}
