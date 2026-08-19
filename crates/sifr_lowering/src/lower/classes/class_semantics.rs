use super::{HirExpr, HirParam, HirPattern, HirStmt, HirTupleTargetBinding, Type};
use std::collections::HashSet;

/// Check if a type is hashable (can derive Hash + Eq).
pub(in crate::lower) fn is_hashable_type(ty: &Type) -> bool {
    ty.supports_hash_key()
}

/// Check whether Rust can derive conditional `Eq` and `Hash` implementations.
///
/// A type variable is valid here because Rust adds the required bounds to the
/// generated trait implementation. Containers that are never hashable remain
/// excluded even when they contain a type variable.
pub(in crate::lower) fn is_derivably_hashable_type(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::TypeVar(_) => true,
        Type::Tuple(values) | Type::Union(values) => values.iter().all(is_derivably_hashable_type),
        Type::Result(left, right) => {
            is_derivably_hashable_type(left) && is_derivably_hashable_type(right)
        }
        Type::Newtype { inner, .. } => is_derivably_hashable_type(inner),
        Type::Class {
            fields,
            methods,
            parent_class,
            ..
        } => {
            parent_class.as_deref() != Some("NonSend")
                && !methods.iter().any(|(name, _)| name == "__eq__")
                && fields
                    .iter()
                    .all(|(_, field)| is_derivably_hashable_type(field))
        }
        _ => ty.supports_hash_key(),
    }
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

/// Storage that is still unavailable when constructor code first needs a
/// materialized `self` place.
pub(in crate::lower) struct ConstructorStorageGap {
    pub(in crate::lower) missing_fields: Vec<String>,
    pub(in crate::lower) missing_parent: bool,
    pub(in crate::lower) statement_index: usize,
}

pub(in crate::lower) fn constructor_uninitialized_storage_at_first_self_use(
    stmts: &[HirStmt],
    own_fields: &[(String, Type)],
    params: &[HirParam],
    requires_parent: bool,
) -> Option<ConstructorStorageGap> {
    let required = own_fields
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<HashSet<_>>();
    let mut initialized = params
        .iter()
        .filter(|param| required.contains(&param.name))
        .map(|param| param.name.clone())
        .collect::<HashSet<_>>();
    let mut explicit_initializers = HashSet::new();
    let mut parent_initialized = !requires_parent;

    for (statement_index, stmt) in stmts.iter().enumerate() {
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
                && !explicit_initializers.contains(field)
                && !body_references_receiver(&[HirStmt::Expr {
                    expr: value.clone(),
                }])
            {
                initialized.insert(field.clone());
                explicit_initializers.insert(field.clone());
                continue;
            }
        }

        if body_references_receiver(std::slice::from_ref(stmt)) {
            let mut missing = required
                .difference(&initialized)
                .cloned()
                .collect::<Vec<_>>();
            missing.sort();
            let missing_parent = !parent_initialized;
            return (!missing.is_empty() || missing_parent).then_some(ConstructorStorageGap {
                missing_fields: missing,
                missing_parent,
                statement_index,
            });
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
