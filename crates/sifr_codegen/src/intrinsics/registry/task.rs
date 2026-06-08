use crate::RustExpr;

pub(crate) fn lower_task_current_context(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident("__sifr_task_current_context()".to_string()))
}
