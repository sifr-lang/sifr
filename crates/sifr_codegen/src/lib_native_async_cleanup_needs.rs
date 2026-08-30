use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use sifr_ir::{HirExpr, HirFunction, HirModule, HirStmt};

pub(crate) fn module_uses_native_async_cleanup(module: &HirModule) -> bool {
    fn stmt_uses_cleanup(stmt: &HirStmt) -> bool {
        matches!(
            stmt,
            HirStmt::AsyncWith {
                kind: sifr_ir::HirAsyncWithKind::UserDefined { .. },
                ..
            } | HirStmt::AsyncFor {
                close_error_ty: Some(_),
                ..
            }
        )
    }

    let function_uses_cleanup = |function: &HirFunction| {
        let mut on_stmt = |stmt: &HirStmt| {
            if stmt_uses_cleanup(stmt) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        let mut on_expr = |_expr: &HirExpr| TraversalControl::Continue;
        matches!(
            traversal::walk_stmts_until(
                &function.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut on_stmt,
                &mut on_expr,
            ),
            TraversalControl::Stop
        )
    };

    module.functions.iter().any(function_uses_cleanup)
        || module
            .classes
            .iter()
            .any(|class| class.methods.iter().any(function_uses_cleanup))
}
