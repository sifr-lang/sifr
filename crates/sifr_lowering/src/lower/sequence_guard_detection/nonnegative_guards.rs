use super::literal_int;
use crate::lower::LowerCtx;
use crate::lower::sequence_guards::SequenceGuard;
use sifr_python_ast::{CmpOp, Expr, ExprCompare};

pub(super) fn detect_true_guards(
    compare: &ExprCompare,
    ctx: &LowerCtx,
) -> Option<Vec<SequenceGuard>> {
    let left = compare.left.as_ref();
    let right = compare.comparators.first()?;
    let op = compare.ops.first()?;
    let index_var = match op {
        CmpOp::Gt if literal_int(right) == Some(-1) => name(left),
        CmpOp::GtE if literal_int(right) == Some(0) => name(left),
        CmpOp::Lt if literal_int(left) == Some(-1) => name(right),
        CmpOp::LtE if literal_int(left) == Some(0) => name(right),
        _ => None,
    }?;
    let mut guards = vec![SequenceGuard::IndexVarNonNegative {
        index_var: index_var.clone(),
    }];
    if matches!(op, CmpOp::GtE) {
        if let Some(sequence) = ctx.end_pointer_sequence(&index_var) {
            guards.push(SequenceGuard::IndexVarInRange {
                sequence,
                index_var,
                max_offset: 0,
            });
        }
    }
    Some(guards)
}

pub(super) fn detect_false_exit_guard(compare: &ExprCompare) -> Option<SequenceGuard> {
    let left = compare.left.as_ref();
    let right = compare.comparators.first()?;
    let op = compare.ops.first()?;
    let index_var = match op {
        CmpOp::Lt if literal_int(right) == Some(0) => name(left),
        CmpOp::LtE if literal_int(right) == Some(-1) => name(left),
        CmpOp::Gt if literal_int(left) == Some(0) => name(right),
        CmpOp::GtE if literal_int(left) == Some(-1) => name(right),
        _ => None,
    }?;
    Some(SequenceGuard::IndexVarNonNegative { index_var })
}

fn name(expr: &Expr) -> Option<String> {
    let Expr::Name(name) = expr else {
        return None;
    };
    Some(name.id.to_string())
}
