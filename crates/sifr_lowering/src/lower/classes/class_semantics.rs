use super::{HirExpr, HirPattern, HirStmt, HirTupleTargetBinding, Type};

/// Check if a type is hashable (can derive Hash + Eq).
pub(in crate::lower) fn is_hashable_type(ty: &Type) -> bool {
    ty.supports_hash_key()
}

/// Check if a method body directly mutates receiver-owned storage.
pub(in crate::lower) fn body_contains_receiver_mutation(stmts: &[HirStmt]) -> bool {
    fn stmt_contains_receiver_mutation(stmt: &HirStmt) -> bool {
        match stmt {
            HirStmt::FieldAssign { object, .. }
            | HirStmt::NestedFieldAssign { object, .. }
            | HirStmt::SubscriptAssign { object, .. }
            | HirStmt::NestedSubscriptAssign { object, .. }
            | HirStmt::AttributeNestedSubscriptAssign { object, .. }
            | HirStmt::SubscriptAugAssign { object, .. }
            | HirStmt::AttributeAugAssign { object, .. }
            | HirStmt::AttributeSubscriptAssign { object, .. } => object == "self",
            HirStmt::TupleUnpack { targets, .. } => targets.iter().any(|target| {
                matches!(
                    &target.binding,
                    HirTupleTargetBinding::Field { object, .. } if object == "self"
                )
            }),
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                body_contains_receiver_mutation(then_body)
                    || elif_clauses
                        .iter()
                        .any(|(_, body)| body_contains_receiver_mutation(body))
                    || else_body
                        .as_ref()
                        .is_some_and(|body| body_contains_receiver_mutation(body))
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
                body_contains_receiver_mutation(body)
                    || else_body
                        .as_ref()
                        .is_some_and(|body| body_contains_receiver_mutation(body))
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                body_contains_receiver_mutation(body)
                    || handlers
                        .iter()
                        .any(|handler| body_contains_receiver_mutation(&handler.body))
            }
            HirStmt::TryFinally { body, finalbody } => {
                body_contains_receiver_mutation(body) || body_contains_receiver_mutation(finalbody)
            }
            HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
                body_contains_receiver_mutation(body)
            }
            HirStmt::Match { arms, .. } => arms
                .iter()
                .any(|arm| body_contains_receiver_mutation(&arm.body)),
            _ => false,
        }
    }

    stmts.iter().any(stmt_contains_receiver_mutation)
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
