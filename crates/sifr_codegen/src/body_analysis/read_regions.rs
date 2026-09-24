use super::{HirExpr, HirStmt, expr_key, traversal, walk_direct_stmt_exprs};
use std::collections::HashSet;

/// Atomic statement guards may only evaluate reads which the statement
/// evaluates unconditionally. Keep conditional and delayed reads in their
/// original expression region.
pub(super) fn conditional_reads(stmt: &HirStmt) -> HashSet<usize> {
    let mut reads = HashSet::new();
    walk_direct_stmt_exprs(stmt, &mut |root| {
        traversal::walk_expr(root, &mut |expr| {
            let mut exclude = |region: &HirExpr| {
                traversal::walk_expr(region, &mut |candidate| {
                    if matches!(candidate, HirExpr::Index { .. }) {
                        reads.insert(expr_key(candidate));
                    }
                });
            };
            match expr {
                HirExpr::IfExpr {
                    then_expr,
                    else_expr,
                    ..
                } => {
                    exclude(then_expr);
                    exclude(else_expr);
                }
                HirExpr::BoolOp { values, .. } => {
                    for value in values.iter().skip(1) {
                        exclude(value);
                    }
                }
                HirExpr::Lambda { body, .. } => exclude(body),
                HirExpr::ListComp { .. }
                | HirExpr::DictComp { .. }
                | HirExpr::SetComp { .. }
                | HirExpr::GeneratorExpr { .. } => exclude(expr),
                _ => {}
            }
        });
    });
    reads
}
