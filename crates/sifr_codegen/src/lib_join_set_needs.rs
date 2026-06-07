use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use sifr_ir::{HirExpr, HirModule, HirStmt};

pub(crate) fn module_uses_join_set_spawn_cpu(module: &HirModule) -> bool {
    for func in &module.functions {
        if body_uses_join_set_spawn_cpu(&func.body) {
            return true;
        }
    }

    for class in &module.classes {
        for method in &class.methods {
            if body_uses_join_set_spawn_cpu(&method.body) {
                return true;
            }
        }
    }

    false
}

fn body_uses_join_set_spawn_cpu(body: &[HirStmt]) -> bool {
    let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
    let mut on_expr = |expr: &HirExpr| {
        if matches!(expr, HirExpr::MethodCall { method, .. } if method == "__sifr_spawn_cpu") {
            TraversalControl::Stop
        } else {
            TraversalControl::Continue
        }
    };

    matches!(
        traversal::walk_stmts_until(
            body,
            TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
            &mut on_stmt,
            &mut on_expr
        ),
        TraversalControl::Stop
    )
}
