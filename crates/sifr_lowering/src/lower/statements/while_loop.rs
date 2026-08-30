use super::{
    FunctionType, HirStmt, LowerCtx, Ranged, StmtWhile, apply_narrowing,
    detect_narrowing_condition, detect_true_nonzero_integer_guards, detect_while_sequence_guards,
    invalidate_loop_body_const_integer_facts, lower_expr, lower_stmts, ownership_diagnostics,
    restore_const_integer_state_after_branches, validate_control_flow_condition,
};

pub(in crate::lower) fn lower_while(
    while_stmt: &StmtWhile,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    invalidate_loop_body_const_integer_facts(ctx, &while_stmt.body);
    let narrowing_cond = detect_narrowing_condition(&while_stmt.test, ctx);
    let condition = lower_expr(&while_stmt.test, ctx)?;
    validate_control_flow_condition(&condition, "while", while_stmt.test.range(), ctx);
    let saved_narrowing_state = ctx.scope.save_narrowing_state();
    let saved_const_integer_state = ctx.scope.save_const_integer_state();
    let saved_sequence_guards = ctx.save_sequence_guards();
    let saved_nonzero_integer_bindings = ctx.save_proven_nonzero_integer_bindings();
    if let Some(ref cond) = narrowing_cond {
        apply_narrowing(ctx, cond, true);
    }
    for guard in detect_while_sequence_guards(while_stmt, ctx) {
        ctx.add_sequence_guard(guard);
    }
    for name in detect_true_nonzero_integer_guards(&while_stmt.test, ctx) {
        ctx.add_proven_nonzero_integer_binding(name);
    }

    let moved_before_loop = ctx.scope.save_moved_state();

    ctx.scope.push();
    ctx.loop_depth += 1;
    let body = lower_stmts(&while_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();
    let body_const_integer_state = ctx.scope.save_const_integer_state();
    ctx.scope.restore_narrowing_state(&saved_narrowing_state);
    restore_const_integer_state_after_branches(
        ctx,
        &saved_const_integer_state,
        &[(body_const_integer_state, false)],
    );
    ctx.restore_sequence_guards(&saved_sequence_guards);
    ctx.restore_proven_nonzero_integer_bindings(&saved_nonzero_integer_bindings);

    let newly_moved = ctx.scope.moved_since(&moved_before_loop);
    for var_name in &newly_moved {
        ownership_diagnostics::moved_across_loop(ctx, var_name, while_stmt.range());
    }

    let else_body = if while_stmt.orelse.is_empty() {
        None
    } else {
        ctx.scope.push();
        let else_stmts = lower_stmts(&while_stmt.orelse, func_type, ctx);
        ctx.scope.pop();
        Some(else_stmts)
    };

    ctx.clear_sequence_pointers();

    Some(HirStmt::While {
        condition,
        body,
        else_body,
    })
}
