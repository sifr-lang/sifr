use super::LowerCtx;
use super::sequence_guard_detection::detect_range_sequence_guards;
use super::sequence_guards::SequenceGuard;
use super::sequence_shapes::SequenceShapeFact;
use sifr_python_ast::{Expr, Stmt, StmtFor};

pub(in crate::lower) fn record_append_growth_sequence_shape_fact(
    for_stmt: &StmtFor,
    target_name: &str,
    ctx: &mut LowerCtx,
) {
    let Some(anchor_sequence) = range_anchor_sequence(for_stmt, target_name, ctx) else {
        return;
    };
    let Some(sequence_var) = append_target_name(&for_stmt.body) else {
        return;
    };
    ctx.record_sequence_shape_fact(SequenceShapeFact::SizedByAnchor {
        sequence_var,
        anchor_sequence,
        extra_len: 0,
    });
}

fn range_anchor_sequence(for_stmt: &StmtFor, target_name: &str, ctx: &LowerCtx) -> Option<String> {
    detect_range_sequence_guards(for_stmt, target_name, ctx)
        .into_iter()
        .find_map(|guard| match guard {
            SequenceGuard::IndexVarInRange {
                sequence,
                index_var,
                max_offset,
            } if index_var == target_name && max_offset == 0 => Some(sequence),
            SequenceGuard::MinLength { .. }
            | SequenceGuard::DictContains { .. }
            | SequenceGuard::SubscriptPresent { .. }
            | SequenceGuard::IndexVarInRange { .. } => None,
        })
}

fn append_target_name(stmts: &[Stmt]) -> Option<String> {
    let [Stmt::Expr(expr_stmt)] = stmts else {
        return None;
    };
    let Expr::Call(call) = expr_stmt.value.as_ref() else {
        return None;
    };
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        return None;
    }
    let Expr::Attribute(attr) = call.func.as_ref() else {
        return None;
    };
    if attr.attr.as_str() != "append" {
        return None;
    }
    let Expr::Name(list_name) = attr.value.as_ref() else {
        return None;
    };
    Some(list_name.id.to_string())
}
