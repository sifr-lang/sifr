use ruff_text_size::{Ranged, TextRange};
use sifr_python_ast::{Expr, Operator, StmtAugAssign};
use sifr_type_system::{type_check_binary_op, Type};

use crate::hir_nodes::{HirExpr, HirStmt};

use super::binding_mutability::ensure_mutable_parameter_binding;
use super::container_literal_specialization::validate_subscript_augassign_target;
use super::expressions::lower_expr;
use super::name_diagnostics;
use super::statements::resolve_object_field_type;
use super::subscript_type::resolve_subscript_result_type;
use super::LowerCtx;

fn op_to_augassign_string(op: Operator, ctx: &mut LowerCtx) -> Option<&'static str> {
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
            ctx.error("matrix multiplication operator (@) is not supported".to_string());
            None
        }
    }
}

pub(super) fn lower_aug_assign(aug: &StmtAugAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    // Handle augmented assignment on attributes: self.field += val
    if let Expr::Attribute(attr) = aug.target.as_ref() {
        let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
            n.id.to_string()
        } else {
            ctx.error("augmented attribute assignment target must be a simple name".to_string());
            return None;
        };
        if !ensure_mutable_parameter_binding(ctx, &obj_name) {
            return None;
        }
        let field_name = attr.attr.to_string();
        let value = lower_expr(&aug.value, ctx)?;
        let op_str = op_to_augassign_string(aug.op, ctx)?;
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
            let (obj_name, nested_field_name, obj_ty, nested_object_expr) = if let Expr::Name(n) =
                inner_sub.value.as_ref()
            {
                let obj_ty = ctx
                    .scope
                    .lookup(&n.id)
                    .map(|info| info.effective_type().clone())
                    .unwrap_or(Type::Unknown);
                (
                    n.id.to_string(),
                    None,
                    obj_ty.clone(),
                    HirExpr::Name {
                        name: n.id.to_string(),
                        ty: obj_ty,
                    },
                )
            } else if let Expr::Attribute(attr) = inner_sub.value.as_ref() {
                let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
                    n.id.to_string()
                } else {
                    ctx.error(
                        "augmented subscript assignment target must be a simple name".to_string(),
                    );
                    return None;
                };
                let field_name = attr.attr.to_string();
                let field_ty = resolve_object_field_type(ctx, &obj_name, &field_name);
                let object_expr = HirExpr::FieldAccess {
                    object: Box::new(lower_expr(attr.value.as_ref(), ctx)?),
                    field: field_name.clone(),
                    ty: field_ty.clone(),
                };
                (obj_name, Some(field_name), field_ty, object_expr)
            } else {
                ctx.error(
                    "augmented subscript assignment target must be a simple name".to_string(),
                );
                return None;
            };
            if !ensure_mutable_parameter_binding(ctx, &obj_name) {
                return None;
            }
            if matches!(obj_ty.resolve_alias(), Type::Bytes) {
                super::ownership_diagnostics::immutable_bytes_augmented_subscript_assignment(ctx);
                return None;
            }
            let outer_index = lower_expr(&inner_sub.slice, ctx)?;
            let inner_index = lower_expr(&sub.slice, ctx)?;
            let value = lower_expr(&aug.value, ctx)?;
            let op_str = op_to_augassign_string(aug.op, ctx)?;
            let outer_elem_ty = resolve_subscript_result_type(
                inner_sub,
                &obj_ty,
                &outer_index,
                outer_index.ty(),
                ctx,
            );
            let current_elem_ty = resolve_subscript_result_type(
                sub,
                &outer_elem_ty,
                &inner_index,
                inner_index.ty(),
                ctx,
            );
            let base_op = &op_str[..op_str.len() - 1];
            let result_ty = match type_check_binary_op(&current_elem_ty, base_op, value.ty()) {
                Ok(ty) => ty,
                Err((code, message)) => {
                    ctx.error_with_code(code, message);
                    return None;
                }
            };
            let outer_expr = HirExpr::Index {
                object: Box::new(nested_object_expr),
                index: Box::new(outer_index.clone()),
                ty: outer_elem_ty,
            };
            let current_value_expr = HirExpr::Index {
                object: Box::new(outer_expr),
                index: Box::new(inner_index.clone()),
                ty: current_elem_ty,
            };
            let lowered_value = HirExpr::BinOp {
                left: Box::new(current_value_expr),
                op: base_op.to_string(),
                right: Box::new(value.clone()),
                ty: result_ty,
            };
            if let Some(field_name) = nested_field_name {
                return Some(HirStmt::AttributeNestedSubscriptAssign {
                    object: obj_name,
                    field: field_name,
                    outer_index,
                    inner_index,
                    value: lowered_value,
                    field_ty: obj_ty,
                });
            }
            return Some(HirStmt::NestedSubscriptAssign {
                object: obj_name,
                outer_index,
                inner_index,
                value: lowered_value,
                object_ty: obj_ty,
            });
        }
        if let Expr::Attribute(attr) = sub.value.as_ref() {
            let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
                n.id.to_string()
            } else {
                ctx.error(
                    "augmented subscript assignment target must be a simple name".to_string(),
                );
                return None;
            };
            if !ensure_mutable_parameter_binding(ctx, &obj_name) {
                return None;
            }
            let field_name = attr.attr.to_string();
            let field_ty = resolve_object_field_type(ctx, &obj_name, &field_name);
            if matches!(field_ty.resolve_alias(), Type::Bytes) {
                super::ownership_diagnostics::immutable_bytes_augmented_subscript_assignment(ctx);
                return None;
            }
            let object_expr = lower_expr(attr.value.as_ref(), ctx)?;
            let index = lower_expr(&sub.slice, ctx)?;
            let value = lower_expr(&aug.value, ctx)?;
            let op_str = op_to_augassign_string(aug.op, ctx)?;

            let element_ty = resolve_subscript_result_type(sub, &field_ty, &index, index.ty(), ctx);
            let base_op = &op_str[..op_str.len() - 1];
            let result_ty = match type_check_binary_op(&element_ty, base_op, value.ty()) {
                Ok(ty) => ty,
                Err((code, message)) => {
                    ctx.error_with_code(code, message);
                    return None;
                }
            };

            let field_access_expr = HirExpr::FieldAccess {
                object: Box::new(object_expr),
                field: field_name.clone(),
                ty: field_ty.clone(),
            };
            let current_value_expr = HirExpr::Index {
                object: Box::new(field_access_expr),
                index: Box::new(index.clone()),
                ty: element_ty,
            };
            let lowered_value = HirExpr::BinOp {
                left: Box::new(current_value_expr),
                op: base_op.to_string(),
                right: Box::new(value.clone()),
                ty: result_ty,
            };

            return Some(HirStmt::AttributeSubscriptAssign {
                object: obj_name,
                field: field_name,
                index,
                value: lowered_value,
                field_ty,
            });
        }
        let obj_name = if let Expr::Name(n) = sub.value.as_ref() {
            n.id.to_string()
        } else {
            ctx.error("augmented subscript assignment target must be a simple name".to_string());
            return None;
        };
        if !ensure_mutable_parameter_binding(ctx, &obj_name) {
            return None;
        }
        let obj_ty = ctx
            .scope
            .lookup(&obj_name)
            .map(|info| info.effective_type().clone())
            .unwrap_or(Type::Unknown);
        if matches!(obj_ty.resolve_alias(), Type::Bytes) {
            super::ownership_diagnostics::immutable_bytes_augmented_subscript_assignment(ctx);
            return None;
        }
        let index = lower_expr(&sub.slice, ctx)?;
        let value = lower_expr(&aug.value, ctx)?;
        let op_str = op_to_augassign_string(aug.op, ctx)?;
        let object_ty = validate_subscript_augassign_target(
            ctx,
            &obj_name,
            obj_ty,
            index.ty(),
            value.ty(),
            op_str,
        );
        return Some(HirStmt::SubscriptAugAssign {
            object: obj_name,
            index,
            op: op_str.to_string(),
            value,
            object_ty,
        });
    }
    let (name, name_range): (String, TextRange) = if let Expr::Name(n) = aug.target.as_ref() {
        (n.id.to_string(), n.range())
    } else {
        ctx.error("augmented assignment target must be a simple name".to_string());
        return None;
    };

    let value = lower_expr(&aug.value, ctx)?;
    ctx.clear_sequence_pointer(&name);
    ctx.clear_len_alias(&name);

    let op_str = op_to_augassign_string(aug.op, ctx)?;

    let var_info = if ctx.current_function_frame_start().is_some() {
        if let Some(info) = ctx.lookup_current_function_binding(&name) {
            Some(info)
        } else if ctx.is_declared_nonlocal(&name) {
            ctx.lookup_outer_function_binding(&name)
        } else if ctx.scope.lookup(&name).is_some() {
            super::flow_diagnostics::captured_augassign_requires_nonlocal(ctx, &name);
            return None;
        } else {
            None
        }
    } else {
        ctx.scope.lookup(&name)
    };
    let Some(var_info) = var_info else {
        name_diagnostics::undefined_variable(ctx, &name, name_range);
        return None;
    };
    if var_info.is_parameter_binding() && !var_info.is_mutable_binding() {
        super::ownership_diagnostics::immutable_parameter_reassignment(ctx, &name);
        return None;
    }
    let var_ty = var_info.ty.clone();

    let base_op = &op_str[..op_str.len() - 1];
    if base_op == "+" {
        match (&var_ty, value.ty()) {
            (Type::Str, Type::Str) => {}
            (Type::List(_), Type::List(_)) => {}
            (Type::Bytes, Type::Bytes) => {}
            _ => {
                if let Err((code, message)) = type_check_binary_op(&var_ty, base_op, value.ty()) {
                    ctx.error_with_code(code, message);
                    return None;
                }
            }
        }
    } else if let Err((code, message)) = type_check_binary_op(&var_ty, base_op, value.ty()) {
        ctx.error_with_code(code, message);
        return None;
    }

    Some(HirStmt::AugAssign {
        name,
        op: op_str.to_string(),
        value,
    })
}
