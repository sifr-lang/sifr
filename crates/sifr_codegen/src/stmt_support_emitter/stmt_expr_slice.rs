macro_rules! stmt_expr_slice {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } = $expr
        {
            let Some(lowered_object) = $emitter.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            if let Some(step_expr) = step {
                stmt_expr_stepped_slice!($emitter, object, start, stop, step_expr, lowered_object);
            }
            stmt_expr_unit_slice!($emitter, object, start, stop, lowered_object);
        }
    }};
}
