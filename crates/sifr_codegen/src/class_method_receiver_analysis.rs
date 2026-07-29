use crate::{
    hir_analysis::traversal::{self, TraversalConfig},
    RustEmitter,
};
use sifr_ir::{HirExpr, HirStmt};
use sifr_type_system::Type;
use std::collections::HashSet;

impl RustEmitter {
    pub(crate) fn collect_direct_class_method_calls(
        stmts: &[HirStmt],
        class_name: &str,
    ) -> HashSet<String> {
        let mut calls = HashSet::new();
        let mut on_stmt = |_stmt: &HirStmt| {};
        let mut on_expr = |expr: &HirExpr| {
            if let HirExpr::MethodCall { object, method, .. } = expr {
                if matches!(object.ty().resolve_alias(), Type::Class { name, .. } if name == class_name)
                {
                    calls.insert(method.clone());
                }
            }
        };
        traversal::walk_stmts(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        calls
    }
}
