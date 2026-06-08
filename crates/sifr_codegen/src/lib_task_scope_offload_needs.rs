use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use sifr_ir::{HirExpr, HirModule, HirStmt};

pub(crate) fn module_uses_task_scope_offload(module: &HirModule) -> bool {
    module_uses_method(module, is_task_scope_offload_method)
}

pub(crate) fn module_uses_task_scope_spawn_cpu(module: &HirModule) -> bool {
    module_uses_method(module, is_task_scope_cpu_method)
}

pub(crate) fn module_uses_task_scope_process(module: &HirModule) -> bool {
    module_uses_method(module, |method| method == "__sifr_scope_spawn_process")
}

fn module_uses_method(module: &HirModule, predicate: fn(&str) -> bool) -> bool {
    for func in &module.functions {
        if body_uses_method(&func.body, predicate) {
            return true;
        }
    }

    for class in &module.classes {
        for method in &class.methods {
            if body_uses_method(&method.body, predicate) {
                return true;
            }
        }
    }

    false
}

fn body_uses_method(body: &[HirStmt], predicate: fn(&str) -> bool) -> bool {
    let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
    let mut on_expr = |expr: &HirExpr| {
        if matches!(expr, HirExpr::MethodCall { method, .. } if predicate(method)) {
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

fn is_task_scope_offload_method(method: &str) -> bool {
    matches!(
        method,
        "__sifr_scope_spawn_blocking_infallible"
            | "__sifr_scope_spawn_blocking_result"
            | "__sifr_scope_spawn_cpu_infallible"
            | "__sifr_scope_spawn_cpu_result"
    )
}

fn is_task_scope_cpu_method(method: &str) -> bool {
    matches!(
        method,
        "__sifr_scope_spawn_cpu_infallible" | "__sifr_scope_spawn_cpu_result"
    )
}
