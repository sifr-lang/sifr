use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Operator, StmtAugAssign};
use sifr_type_system::{Type, type_check_binary_op};

use crate::hir_nodes::{HirCollectionMutation, HirExpr, HirStmt};

use super::LowerCtx;
use super::binding_mutability::ensure_mutable_parameter_binding;
use super::container_literal_specialization::{
    SubscriptAugAssignTarget, validate_subscript_augassign_target,
};
use super::defaultdict_refinement::refine_defaultdict_int_augassign_key;
use super::expressions::{affine_value_references_name, consume_owned_value, lower_expr};
use super::guarded_index::guarded_sequence_index_result_type;
use super::integer_failure_diagnostics::exact_int_augassign_requires_handling;
use super::name_diagnostics;
use super::python_interop::lower_python_context_owned_expr;
use super::statements::resolve_object_field_type;
use super::subscript_type::resolve_subscript_result_type;

const AUGMENTED_SUBSCRIPT_TARGET_SIMPLE_NAME: &str =
    "augmented subscript assignment target must be a simple name";

fn invalid_target_shape(ctx: &mut LowerCtx, message: &'static str, range: TextRange) {
    ctx.error_with_code_at(DiagnosticCode::TYPE_MISMATCH, message.to_string(), range);
}

fn invalid_subscript_target_shape(ctx: &mut LowerCtx, range: TextRange) {
    invalid_target_shape(ctx, AUGMENTED_SUBSCRIPT_TARGET_SIMPLE_NAME, range);
}

fn op_to_augassign_string(
    op: Operator,
    ctx: &mut LowerCtx,
    target_range: TextRange,
) -> Option<&'static str> {
    match op {
        Operator::Add => Some("+="),
        Operator::Sub => Some("-="),
        Operator::Mult => Some("*="),
        Operator::Div => Some("/="),
        Operator::Mod => Some("%="),
        Operator::Pow => Some("**="),
        Operator::BitAnd => Some("&="),
        Operator::BitOr => Some("|="),
        Operator::BitXor => Some("^="),
        Operator::LShift => Some("<<="),
        Operator::RShift => Some(">>="),
        Operator::FloorDiv => Some("//="),
        Operator::MatMult => {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                "matrix multiplication operator (@) is not supported".to_string(),
                target_range,
            );
            None
        }
    }
}

pub(in crate::lower) fn lower_aug_assign(
    aug: &StmtAugAssign,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    // Handle augmented assignment on attributes: self.field += val
    if let Expr::Attribute(attr) = aug.target.as_ref() {
        let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
            n.id.to_string()
        } else {
            invalid_target_shape(
                ctx,
                "augmented attribute assignment target must be a simple name",
                attr.value.range(),
            );
            return None;
        };
        if !ensure_mutable_parameter_binding(ctx, &obj_name, attr.value.range()) {
            return None;
        }
        let field_name = attr.attr.to_string();
        let value = lower_python_context_owned_expr(&aug.value, ctx)?;
        let op_str = op_to_augassign_string(aug.op, ctx, aug.target.range())?;
        return Some(HirStmt::AttributeAugAssign {
            object: obj_name,
            field: field_name,
            op: op_str.to_string(),
            value,
        });
    }

    // Handle augmented assignment on subscript: list[i] += val
    if let Expr::Subscript(sub) = aug.target.as_ref() {
        if let Expr::Subscript(inner_sub) = sub.value.as_ref() {
            let (obj_name, nested_field_name, obj_ty) =
                if let Expr::Name(n) = inner_sub.value.as_ref() {
                    let obj_ty = ctx
                        .scope
                        .lookup(&n.id)
                        .map(|info| info.effective_type().clone())
                        .unwrap_or(Type::Unknown);
                    (n.id.to_string(), None, obj_ty)
                } else if let Expr::Attribute(attr) = inner_sub.value.as_ref() {
                    let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
                        n.id.to_string()
                    } else {
                        invalid_subscript_target_shape(ctx, attr.value.range());
                        return None;
                    };
                    let field_name = attr.attr.to_string();
                    let field_ty = resolve_object_field_type(ctx, &obj_name, &field_name);
                    (obj_name, Some(field_name), field_ty)
                } else {
                    invalid_subscript_target_shape(ctx, inner_sub.value.range());
                    return None;
                };
            if !ensure_mutable_parameter_binding(ctx, &obj_name, inner_sub.value.range()) {
                return None;
            }
            if matches!(obj_ty.resolve_alias(), Type::Bytes) {
                super::ownership_diagnostics::immutable_bytes_augmented_subscript_assignment(
                    ctx,
                    inner_sub.range(),
                );
                return None;
            }
            let outer_index = lower_expr(&inner_sub.slice, ctx)?;
            let inner_index = lower_expr(&sub.slice, ctx)?;
            let value = lower_python_context_owned_expr(&aug.value, ctx)?;
            let op_str = op_to_augassign_string(aug.op, ctx, aug.target.range())?;
            let _outer_result_ty = resolve_subscript_result_type(
                inner_sub,
                &obj_ty,
                &outer_index,
                outer_index.ty(),
                ctx,
            );
            let outer_elem_ty = super::statements::checked_place_value_type(&obj_ty)?;
            let current_elem_ty = super::statements::checked_place_value_type(&outer_elem_ty)?;
            let base_op = &op_str[..op_str.len() - 1];
            let _result_ty = match type_check_binary_op(&current_elem_ty, base_op, value.ty()) {
                Ok(ty) => ty,
                Err((code, message)) => {
                    ctx.error_with_code_at(code, message, aug.value.range());
                    return None;
                }
            };
            let outer_statically_present =
                guarded_sequence_index_result_type(inner_sub, &obj_ty, ctx).is_some();
            let outer_failure = super::statements::checked_place_projection_failure(
                ctx,
                &obj_ty,
                true,
                outer_statically_present,
                "nested augmented subscript assignment",
                inner_sub.range(),
            );
            let inner_statically_present =
                guarded_sequence_index_result_type(sub, &outer_elem_ty, ctx).is_some();
            let inner_failure = super::statements::checked_place_projection_failure(
                ctx,
                &outer_elem_ty,
                true,
                inner_statically_present,
                "nested augmented subscript assignment",
                sub.range(),
            );
            if let Some(field_name) = nested_field_name {
                return Some(HirStmt::AttributeNestedSubscriptAssign {
                    object: obj_name,
                    field: field_name,
                    outer_index,
                    inner_index,
                    value,
                    field_ty: obj_ty,
                    outer_failure,
                    inner_failure,
                    operation: HirCollectionMutation::AugAssign(op_str.to_string()),
                });
            }
            return Some(HirStmt::NestedSubscriptAssign {
                object: obj_name,
                outer_index,
                inner_index,
                value,
                object_ty: obj_ty,
                outer_failure,
                inner_failure,
                operation: HirCollectionMutation::AugAssign(op_str.to_string()),
            });
        }
        if let Expr::Attribute(attr) = sub.value.as_ref() {
            let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
                n.id.to_string()
            } else {
                invalid_subscript_target_shape(ctx, attr.value.range());
                return None;
            };
            if !ensure_mutable_parameter_binding(ctx, &obj_name, attr.value.range()) {
                return None;
            }
            let field_name = attr.attr.to_string();
            let field_ty = resolve_object_field_type(ctx, &obj_name, &field_name);
            if matches!(field_ty.resolve_alias(), Type::Bytes) {
                super::ownership_diagnostics::immutable_bytes_augmented_subscript_assignment(
                    ctx,
                    sub.range(),
                );
                return None;
            }
            let index = lower_expr(&sub.slice, ctx)?;
            let value = lower_python_context_owned_expr(&aug.value, ctx)?;
            let op_str = op_to_augassign_string(aug.op, ctx, aug.target.range())?;

            let element_ty = super::statements::checked_place_value_type(&field_ty)?;
            let base_op = &op_str[..op_str.len() - 1];
            let _result_ty = match type_check_binary_op(&element_ty, base_op, value.ty()) {
                Ok(ty) => ty,
                Err((code, message)) => {
                    ctx.error_with_code_at(code, message, aug.value.range());
                    return None;
                }
            };

            let statically_present =
                guarded_sequence_index_result_type(sub, &field_ty, ctx).is_some();
            let failure = super::statements::checked_place_projection_failure(
                ctx,
                &field_ty,
                true,
                statically_present,
                "augmented subscript assignment",
                sub.range(),
            );

            return Some(HirStmt::AttributeSubscriptAssign {
                object: obj_name,
                field: field_name,
                index,
                value,
                field_ty,
                failure,
                operation: HirCollectionMutation::AugAssign(op_str.to_string()),
            });
        }
        let obj_name = if let Expr::Name(n) = sub.value.as_ref() {
            n.id.to_string()
        } else {
            invalid_subscript_target_shape(ctx, sub.value.range());
            return None;
        };
        if !ensure_mutable_parameter_binding(ctx, &obj_name, sub.value.range()) {
            return None;
        }
        let obj_ty = ctx
            .scope
            .lookup(&obj_name)
            .map(|info| info.effective_type().clone())
            .unwrap_or(Type::Unknown);
        if matches!(obj_ty.resolve_alias(), Type::Bytes) {
            super::ownership_diagnostics::immutable_bytes_augmented_subscript_assignment(
                ctx,
                sub.range(),
            );
            return None;
        }
        let index = lower_expr(&sub.slice, ctx)?;
        let value = lower_python_context_owned_expr(&aug.value, ctx)?;
        let op_str = op_to_augassign_string(aug.op, ctx, aug.target.range())?;
        let obj_ty = refine_defaultdict_int_augassign_key(&obj_name, obj_ty, index.ty(), ctx);
        let object_ty = validate_subscript_augassign_target(
            ctx,
            SubscriptAugAssignTarget {
                object_name: &obj_name,
                object_ty: obj_ty,
                index_ty: index.ty(),
                rhs_ty: value.ty(),
                rhs_expr: &value,
                op: op_str,
                target_range: sub.range(),
                rhs_range: aug.value.range(),
            },
        );
        let is_defaultdict = matches!(
            object_ty,
            Type::Alias { ref name, .. } if name.starts_with("__sifr_defaultdict_")
        );
        let statically_present =
            is_defaultdict || guarded_sequence_index_result_type(sub, &object_ty, ctx).is_some();
        let failure = super::statements::checked_place_projection_failure(
            ctx,
            &object_ty,
            true,
            statically_present,
            "augmented subscript assignment",
            sub.range(),
        );
        return Some(HirStmt::SubscriptAugAssign {
            object: obj_name,
            index,
            op: op_str.to_string(),
            value,
            object_ty,
            failure,
        });
    }
    let (name, name_range): (String, TextRange) = if let Expr::Name(n) = aug.target.as_ref() {
        (n.id.to_string(), n.range())
    } else {
        invalid_target_shape(
            ctx,
            "augmented assignment target must be a simple name",
            aug.target.range(),
        );
        return None;
    };

    let value = lower_python_context_owned_expr(&aug.value, ctx)?;
    ctx.clear_sequence_pointer(&name);
    ctx.clear_len_alias(&name);
    ctx.scope.clear_const_integer_value(&name);
    ctx.clear_proven_nonzero_integer_binding(&name);

    let op_str = op_to_augassign_string(aug.op, ctx, aug.target.range())?;

    let var_info = if ctx.current_function_frame_start().is_some() {
        if let Some(info) = ctx.lookup_current_function_binding(&name) {
            Some(info)
        } else if ctx.is_declared_nonlocal(&name) {
            ctx.lookup_outer_function_binding(&name)
        } else if ctx.scope.lookup(&name).is_some() {
            super::flow_diagnostics::captured_augassign_requires_nonlocal(ctx, &name, name_range);
            return None;
        } else {
            None
        }
    } else {
        ctx.scope.lookup(&name)
    }
    .cloned();
    let Some(var_info) = var_info else {
        name_diagnostics::undefined_variable(ctx, &name, name_range);
        return None;
    };
    if super::ownership_diagnostics::reject_borrowed_affine_parameter_reassignment(
        ctx,
        &name,
        var_info.is_parameter_binding(),
        &var_info.ty,
        name_range,
    ) {
        return None;
    }
    if var_info.is_parameter_binding() && !var_info.is_mutable_binding() {
        super::ownership_diagnostics::immutable_parameter_reassignment(ctx, &name, name_range);
        return None;
    }
    let var_ty = var_info.ty.clone();
    let rhs_is_target = matches!(aug.value.as_ref(), Expr::Name(rhs) if rhs.id.as_str() == name);
    if affine_value_references_name(&value, &name) {
        ctx.error_with_code_at(
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            format!(
                "augmented assignment cannot move affine target '{name}' through its own right-hand side"
            ),
            aug.value.range(),
        );
        return None;
    }

    let base_op = &op_str[..op_str.len() - 1];
    if exact_int_augassign_requires_handling(&var_ty, base_op, &value, ctx, aug.value.range()) {
        return None;
    }
    if base_op == "+" {
        match (&var_ty, value.ty()) {
            (Type::Str, Type::Str) => {}
            (Type::List(left), Type::List(right)) if left == right => {
                if rhs_is_target {
                    if let Err((code, message)) = type_check_binary_op(&var_ty, base_op, value.ty())
                    {
                        ctx.error_with_code_at(code, message, aug.value.range());
                        return None;
                    }
                    return Some(HirStmt::Assign {
                        name: name.clone(),
                        value: HirExpr::BinOp {
                            left: Box::new(HirExpr::Name {
                                binding_id: ctx.scope.lookup(&name).map(|info| info.binding_id),
                                name,
                                ty: var_ty.clone(),
                            }),
                            op: base_op.to_string(),
                            right: Box::new(value),
                            ty: var_ty,
                        },
                    });
                }
                consume_owned_value(&value, aug.value.range(), ctx);
            }
            (Type::Bytes, Type::Bytes) => {}
            _ => {
                if let Err((code, message)) = type_check_binary_op(&var_ty, base_op, value.ty()) {
                    ctx.error_with_code_at(code, message, aug.value.range());
                    return None;
                }
            }
        }
    } else {
        if let Err((code, message)) = type_check_binary_op(&var_ty, base_op, value.ty()) {
            ctx.error_with_code_at(code, message, aug.value.range());
            return None;
        }
        if base_op == "*" && matches!(var_ty.resolve_alias(), Type::List(_)) {
            return Some(HirStmt::Assign {
                name: name.clone(),
                value: HirExpr::BinOp {
                    left: Box::new(HirExpr::Name {
                        binding_id: ctx.scope.lookup(&name).map(|info| info.binding_id),
                        name,
                        ty: var_ty.clone(),
                    }),
                    op: base_op.to_string(),
                    right: Box::new(value),
                    ty: var_ty,
                },
            });
        }
    }

    Some(HirStmt::AugAssign {
        name,
        op: op_str.to_string(),
        value,
    })
}
