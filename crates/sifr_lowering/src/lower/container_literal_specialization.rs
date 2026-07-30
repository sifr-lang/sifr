use super::{type_bounds::reject_unavailable_hash_key, LowerCtx};
use crate::hir_nodes::{HirExpr, HirStmt};
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::{type_check_binary_op, Type};
use std::collections::HashMap;

pub(in crate::lower) fn type_contains_unknown_or_any(ty: &Type) -> bool {
    match ty {
        Type::Unknown | Type::Any => true,
        Type::List(elem) => type_contains_unknown_or_any(elem),
        Type::Dict(key, value) => {
            type_contains_unknown_or_any(key) || type_contains_unknown_or_any(value)
        }
        Type::Tuple(elements) => elements.iter().any(type_contains_unknown_or_any),
        _ => false,
    }
}

fn is_any_like_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Any | Type::Unknown)
}

fn emit_empty_literal_type_conflict(
    ctx: &mut LowerCtx,
    object_name: &str,
    expected_key: &Type,
    expected_value: &Type,
    actual_key: &Type,
    actual_value: &Type,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT,
        format!(
            "empty literal type conflict for '{object_name}': expected key '{}' and value '{}', got key '{}' and value '{}'",
            expected_key.display_name(),
            expected_value.display_name(),
            actual_key.display_name(),
            actual_value.display_name()
        ),
        range,
    );
}

pub(in crate::lower) fn validate_subscript_assignment_target(
    ctx: &mut LowerCtx,
    object_name: &str,
    object_ty: &Type,
    index_ty: &Type,
    value_ty: &Type,
    range: TextRange,
) -> Type {
    match object_ty.resolve_alias().clone() {
        Type::List(elem_ty) => {
            if index_ty != &Type::Int {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "list subscript assignment index must be 'int', got '{}'",
                        index_ty.display_name()
                    ),
                    range,
                );
            }
            if !value_ty.is_assignable_to(elem_ty.as_ref()) {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "list subscript assignment value type '{}' is not compatible with list element type '{}'",
                        value_ty.display_name(),
                        elem_ty.display_name()
                    ),
                    range,
                );
            }
            Type::List(elem_ty)
        }
        Type::Dict(key_ty, value_ty_expected) => {
            if is_any_like_type(key_ty.as_ref())
                && is_any_like_type(value_ty_expected.as_ref())
                && !is_any_like_type(index_ty)
                && !is_any_like_type(value_ty)
            {
                let specialized =
                    Type::Dict(Box::new(index_ty.clone()), Box::new(value_ty.clone()));
                let _ = ctx.scope.set_type(object_name, specialized.clone());
                ctx.pending_container_specialization_patches
                    .insert(object_name.to_string(), specialized.clone());
                ctx.empty_dict_specializations
                    .insert(object_name.to_string(), specialized.clone());
                return specialized;
            }

            if !reject_unavailable_hash_key(
                key_ty.as_ref(),
                "dict subscript assignment",
                range,
                ctx,
            ) {
                reject_unavailable_hash_key(index_ty, "dict subscript assignment", range, ctx);
            }

            let key_ok = index_ty.is_assignable_to(key_ty.as_ref());
            let value_ok = value_ty.is_assignable_to(value_ty_expected.as_ref());
            if !key_ok || !value_ok {
                if let Some(Type::Dict(expected_key, expected_value)) =
                    ctx.empty_dict_specializations.get(object_name).cloned()
                {
                    emit_empty_literal_type_conflict(
                        ctx,
                        object_name,
                        expected_key.as_ref(),
                        expected_value.as_ref(),
                        index_ty,
                        value_ty,
                        range,
                    );
                } else {
                    if !key_ok {
                        ctx.error_with_code_at(
                            DiagnosticCode::TYPE_MISMATCH,
                            format!(
                                "dict subscript assignment key type '{}' is not compatible with dict key type '{}'",
                                index_ty.display_name(),
                                key_ty.display_name()
                            ),
                            range,
                        );
                    }
                    if !value_ok {
                        ctx.error_with_code_at(
                            DiagnosticCode::TYPE_MISMATCH,
                            format!(
                                "dict subscript assignment value type '{}' is not compatible with dict value type '{}'",
                                value_ty.display_name(),
                                value_ty_expected.display_name()
                            ),
                            range,
                        );
                    }
                }
            }
            Type::Dict(key_ty, value_ty_expected)
        }
        other => {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "subscript assignment is not supported for type '{}'",
                    other.display_name()
                ),
                range,
            );
            other
        }
    }
}

pub(in crate::lower) struct SubscriptAugAssignTarget<'a> {
    pub(in crate::lower) object_name: &'a str,
    pub(in crate::lower) object_ty: Type,
    pub(in crate::lower) index_ty: &'a Type,
    pub(in crate::lower) rhs_ty: &'a Type,
    pub(in crate::lower) op: &'a str,
    pub(in crate::lower) target_range: TextRange,
    pub(in crate::lower) rhs_range: TextRange,
}

pub(in crate::lower) fn validate_subscript_augassign_target(
    ctx: &mut LowerCtx,
    target: SubscriptAugAssignTarget<'_>,
) -> Type {
    let object_name = target.object_name;
    let object_ty = target.object_ty;
    let index_ty = target.index_ty;
    let rhs_ty = target.rhs_ty;
    let op = target.op;
    let target_range = target.target_range;
    let rhs_range = target.rhs_range;
    let base_op = &op[..op.len() - 1];
    let resolved_object_ty = object_ty.resolve_alias().clone();
    match resolved_object_ty {
        Type::List(elem_ty) => {
            if index_ty != &Type::Int {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "list subscript assignment index must be 'int', got '{}'",
                        index_ty.display_name()
                    ),
                    target_range,
                );
            }
            if let Err((code, message)) = type_check_binary_op(elem_ty.as_ref(), base_op, rhs_ty) {
                ctx.error_with_code_at(code, message, rhs_range);
            }
            Type::List(elem_ty)
        }
        Type::Dict(key_ty, value_ty_expected) => {
            if !reject_unavailable_hash_key(
                key_ty.as_ref(),
                "dict augmented subscript assignment",
                target_range,
                ctx,
            ) {
                reject_unavailable_hash_key(
                    index_ty,
                    "dict augmented subscript assignment",
                    target_range,
                    ctx,
                );
            }
            if !index_ty.is_assignable_to(key_ty.as_ref()) {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "dict subscript assignment key type '{}' is not compatible with dict key type '{}'",
                        index_ty.display_name(),
                        key_ty.display_name()
                    ),
                    target_range,
                );
            }
            if let Err((code, message)) =
                type_check_binary_op(value_ty_expected.as_ref(), base_op, rhs_ty)
            {
                if let Some(Type::Dict(expected_key, expected_value)) =
                    ctx.empty_dict_specializations.get(object_name).cloned()
                {
                    emit_empty_literal_type_conflict(
                        ctx,
                        object_name,
                        expected_key.as_ref(),
                        expected_value.as_ref(),
                        index_ty,
                        rhs_ty,
                        rhs_range,
                    );
                } else {
                    ctx.error_with_code_at(code, message, rhs_range);
                }
            }
            match object_ty {
                Type::Alias {
                    name, type_args, ..
                } if name.starts_with("__sifr_defaultdict_") => Type::Alias {
                    name,
                    type_args,
                    body: Box::new(Type::Dict(key_ty, value_ty_expected)),
                },
                _ => Type::Dict(key_ty, value_ty_expected),
            }
        }
        other => {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "augmented subscript assignment is not supported for type '{}'",
                    other.display_name()
                ),
                target_range,
            );
            other
        }
    }
}

pub(in crate::lower) fn apply_container_specialization_patches(
    stmts: &mut [HirStmt],
    pending: &mut HashMap<String, Type>,
) {
    for stmt in stmts.iter_mut().rev() {
        patch_stmt_container_specialization(stmt, pending);
        if pending.is_empty() {
            break;
        }
    }
}

fn patch_stmt_container_specialization(stmt: &mut HirStmt, pending: &mut HashMap<String, Type>) {
    match stmt {
        HirStmt::Let {
            name, ty, value, ..
        } => {
            let Some(patch_ty) = pending.remove(name) else {
                return;
            };
            *ty = patch_ty.clone();
            match (value, &patch_ty) {
                (HirExpr::DictLiteral { ty: literal_ty, .. }, Type::Dict(_, _))
                | (HirExpr::ListLiteral { ty: literal_ty, .. }, Type::List(_))
                | (HirExpr::SetLiteral { ty: literal_ty, .. }, Type::Set(_)) => {
                    *literal_ty = patch_ty;
                }
                (
                    HirExpr::Call {
                        func, ty: call_ty, ..
                    },
                    Type::Alias { name, .. },
                ) if func == name && name.starts_with("__sifr_defaultdict_") => {
                    *call_ty = patch_ty;
                }
                _ => {}
            }
        }
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            apply_container_specialization_patches(then_body, pending);
            for (_, body) in elif_clauses {
                apply_container_specialization_patches(body, pending);
            }
            if let Some(body) = else_body {
                apply_container_specialization_patches(body, pending);
            }
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
            apply_container_specialization_patches(body, pending);
            if let Some(body) = else_body {
                apply_container_specialization_patches(body, pending);
            }
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            apply_container_specialization_patches(body, pending);
            for handler in handlers {
                apply_container_specialization_patches(&mut handler.body, pending);
            }
        }
        HirStmt::TryFinally { body, finalbody } => {
            apply_container_specialization_patches(body, pending);
            apply_container_specialization_patches(finalbody, pending);
        }
        HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
            apply_container_specialization_patches(body, pending);
        }
        HirStmt::NestedFunction { func, .. } => {
            apply_container_specialization_patches(&mut func.body, pending);
        }
        HirStmt::Match { arms, .. } => {
            for arm in arms {
                apply_container_specialization_patches(&mut arm.body, pending);
            }
        }
        _ => {}
    }
}
