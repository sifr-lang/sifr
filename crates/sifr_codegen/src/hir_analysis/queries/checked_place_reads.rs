use sifr_ir::{HirExpr, HirStmt};
use sifr_type_system::Type;

use crate::hir_analysis::traversal::{self, TraversalConfig};

pub(crate) fn proven_collection_reads(stmts: &[HirStmt]) -> Vec<HirExpr> {
    let mut reads = Vec::new();
    for stmt in stmts {
        let optional_target_read = match stmt {
            HirStmt::Let { ty, value, .. } if ty.optional_member_type().is_some() => {
                matches!(value, HirExpr::Index { .. }).then_some(value)
            }
            _ => None,
        };
        traversal::walk_stmts(
            std::slice::from_ref(stmt),
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut |_| {},
            &mut |expr| {
                if optional_target_read.is_some_and(|read| std::ptr::eq(read, expr)) {
                    return;
                }
                collect_collection_read(expr, false, &mut reads);
            },
        );
    }
    reads.sort_by_key(index_depth);
    reads
}

pub(crate) fn collection_reads_in_condition(condition: &HirExpr) -> Vec<HirExpr> {
    let mut reads = Vec::new();
    traversal::walk_expr(condition, &mut |expr| {
        collect_collection_read(expr, true, &mut reads);
    });
    reads.sort_by_key(index_depth);
    reads
}

fn collect_collection_read(expr: &HirExpr, include_optional: bool, reads: &mut Vec<HirExpr>) {
    let HirExpr::Index { object, ty, .. } = expr else {
        return;
    };
    if (!include_optional && crate::helpers::is_option_type(ty))
        || !matches!(
            crate::resolve_alias_type_for_plain_call(object.ty()),
            Type::Dict(_, _) | Type::List(_) | Type::Bytes | Type::Str
        )
    {
        return;
    }
    reads.push(expr.clone());
}

fn index_depth(expr: &HirExpr) -> usize {
    let mut depth = 0;
    let mut current = expr;
    while let HirExpr::Index { object, .. } = current {
        depth += 1;
        current = object;
    }
    depth
}
