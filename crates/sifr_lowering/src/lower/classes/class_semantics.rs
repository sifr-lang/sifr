use super::{HirExpr, HirParam, HirPattern, HirStmt, HirTupleTargetBinding, Type};
use std::collections::HashSet;

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

fn body_references_receiver(stmts: &[HirStmt]) -> bool {
    let mut body = stmts.to_vec();
    let mut references_receiver = false;
    sifr_ir::visit_hir_stmts_exprs_mut(&mut body, &mut |expr| {
        if matches!(expr, HirExpr::Name { name, .. } if name == "self") {
            references_receiver = true;
        }
    });
    sifr_ir::visit_hir_stmts_storage_roots_mut(&mut body, &mut |root| {
        if root == "self" {
            references_receiver = true;
        }
    });
    references_receiver
}

/// Return the fields that are still unavailable when constructor code first
/// needs a materialized `self` place. `__sifr_parent` denotes a missing
/// `super().__init__` result.
pub(in crate::lower) fn constructor_uninitialized_storage_at_first_self_use(
    stmts: &[HirStmt],
    own_fields: &[(String, Type)],
    params: &[HirParam],
    requires_parent: bool,
) -> Option<Vec<String>> {
    let required = own_fields
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<HashSet<_>>();
    let explicit_initializers = stmts
        .iter()
        .filter_map(|stmt| {
            let HirStmt::FieldAssign {
                object,
                field,
                value,
                ..
            } = stmt
            else {
                return None;
            };
            (object == "self"
                && required.contains(field)
                && !body_references_receiver(&[HirStmt::Expr {
                    expr: value.clone(),
                }]))
            .then_some(field.clone())
        })
        .collect::<HashSet<_>>();
    let mut initialized = params
        .iter()
        .filter(|param| {
            required.contains(&param.name) && !explicit_initializers.contains(&param.name)
        })
        .map(|param| param.name.clone())
        .collect::<HashSet<_>>();
    let mut parent_initialized = !requires_parent;

    for stmt in stmts {
        if matches!(
            stmt,
            HirStmt::Expr {
                expr: HirExpr::SuperCall { .. }
            }
        ) {
            parent_initialized = true;
            continue;
        }

        if let HirStmt::FieldAssign {
            object,
            field,
            value,
            ..
        } = stmt
        {
            if object == "self"
                && required.contains(field)
                && !initialized.contains(field)
                && !body_references_receiver(&[HirStmt::Expr {
                    expr: value.clone(),
                }])
            {
                initialized.insert(field.clone());
                continue;
            }
        }

        if body_references_receiver(std::slice::from_ref(stmt)) {
            let mut missing = required
                .difference(&initialized)
                .cloned()
                .collect::<Vec<_>>();
            missing.sort();
            if !parent_initialized {
                missing.push("__sifr_parent".to_string());
            }
            return (!missing.is_empty()).then_some(missing);
        }
    }

    None
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
