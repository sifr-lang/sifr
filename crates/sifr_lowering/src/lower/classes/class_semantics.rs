use super::{HirExpr, HirPattern, HirStmt, HirTupleTargetBinding, Type};

/// Check if a type is hashable (can derive Hash + Eq).
pub(in crate::lower) fn is_hashable_type(ty: &Type) -> bool {
    ty.supports_hash_key()
}

/// Check if a method body contains any field assignments (self.field = ...).
pub(in crate::lower) fn body_contains_field_assign(stmts: &[HirStmt]) -> bool {
    fn stmt_contains_field_assign(stmt: &HirStmt) -> bool {
        match stmt {
            HirStmt::FieldAssign { .. } | HirStmt::NestedFieldAssign { .. } => true,
            HirStmt::TupleUnpack { targets, .. } => targets
                .iter()
                .any(|target| matches!(target.binding, HirTupleTargetBinding::Field { .. })),
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                body_contains_field_assign(then_body)
                    || elif_clauses
                        .iter()
                        .any(|(_, body)| body_contains_field_assign(body))
                    || else_body
                        .as_ref()
                        .is_some_and(|body| body_contains_field_assign(body))
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
                body_contains_field_assign(body)
                    || else_body
                        .as_ref()
                        .is_some_and(|body| body_contains_field_assign(body))
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                body_contains_field_assign(body)
                    || handlers
                        .iter()
                        .any(|handler| body_contains_field_assign(&handler.body))
            }
            HirStmt::TryFinally { body, finalbody } => {
                body_contains_field_assign(body) || body_contains_field_assign(finalbody)
            }
            HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
                body_contains_field_assign(body)
            }
            HirStmt::Match { arms, .. } => {
                arms.iter().any(|arm| body_contains_field_assign(&arm.body))
            }
            _ => false,
        }
    }

    stmts.iter().any(stmt_contains_field_assign)
}

pub(in crate::lower) fn collect_literal_coverage(
    pattern: &HirPattern,
    covered_literal_strs: &mut std::collections::HashSet<String>,
    covered_literal_ints: &mut std::collections::HashSet<i64>,
    covered_literal_bools: &mut std::collections::HashSet<bool>,
) {
    if let HirPattern::Literal { value } = pattern {
        match value {
            HirExpr::StringLiteral(s) => {
                covered_literal_strs.insert(s.clone());
            }
            HirExpr::IntLiteral(n) => {
                covered_literal_ints.insert(*n);
            }
            HirExpr::BoolLiteral(b) => {
                covered_literal_bools.insert(*b);
            }
            _ => {}
        }
    }
}
