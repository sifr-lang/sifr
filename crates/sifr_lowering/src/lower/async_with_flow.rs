use crate::hir_nodes::HirStmt;

pub(super) fn stmt_contains_scope_early_exit(stmt: &HirStmt) -> bool {
    stmt_contains_scope_exit(stmt, true)
}

fn stmt_contains_scope_exit(stmt: &HirStmt, include_return: bool) -> bool {
    match stmt {
        HirStmt::Return { .. } => include_return,
        HirStmt::Raise { .. } | HirStmt::Yield { .. } => true,
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            then_body
                .iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                || elif_clauses.iter().any(|(_, body)| {
                    body.iter()
                        .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                })
                || else_body.as_ref().is_some_and(|body| {
                    body.iter()
                        .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                })
        }
        HirStmt::While {
            body, else_body, ..
        }
        | HirStmt::For {
            body, else_body, ..
        }
        | HirStmt::AsyncFor {
            body, else_body, ..
        } => {
            body.iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                || else_body.as_ref().is_some_and(|body| {
                    body.iter()
                        .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                })
        }
        HirStmt::AsyncWith { body, .. } | HirStmt::With { body, .. } => body
            .iter()
            .any(|stmt| stmt_contains_scope_exit(stmt, include_return)),
        HirStmt::TryExcept { body, handlers, .. } => {
            body.iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                || handlers.iter().any(|handler| {
                    handler
                        .body
                        .iter()
                        .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                })
        }
        HirStmt::TryFinally { body, finalbody } => {
            body.iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
                || finalbody
                    .iter()
                    .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
        }
        HirStmt::Match { arms, .. } => arms.iter().any(|arm| {
            arm.body
                .iter()
                .any(|stmt| stmt_contains_scope_exit(stmt, include_return))
        }),
        _ => false,
    }
}
