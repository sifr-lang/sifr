use super::flow_helpers::expr_to_literal_value;
use super::LowerCtx;
use sifr_python_ast::{BoolOp, CmpOp, Expr, UnaryOp};
use sifr_type_system::infer::resolve_type_annotation;
use sifr_type_system::NarrowingCondition;

/// Detect a narrowing condition from an if-test expression.
pub(in crate::lower) fn detect_narrowing_condition(
    expr: &Expr,
    ctx: &LowerCtx,
) -> Option<NarrowingCondition> {
    match expr {
        Expr::Call(call) => {
            if let Expr::Name(func_name) = call.func.as_ref() {
                if func_name.id.as_str() == "isinstance" && call.arguments.args.len() == 2 {
                    if let Expr::Name(var) = &call.arguments.args[0] {
                        let var_name = var.id.to_string();
                        if ctx.scope.lookup(&var_name).is_some() {
                            if let Expr::Name(type_name) = &call.arguments.args[1] {
                                let target_ty =
                                    resolve_type_annotation(&type_name.id).or_else(|| {
                                        ctx.class_types.get(type_name.id.as_str()).cloned()
                                    });
                                if let Some(target_ty) = target_ty {
                                    return Some(NarrowingCondition::IsInstance(
                                        var_name, target_ty,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        Expr::Compare(cmp) => {
            if cmp.ops.len() == 1 && cmp.comparators.len() == 1 {
                match &cmp.ops[0] {
                    CmpOp::Is => {
                        if let (Expr::Name(var), Expr::NoneLiteral(_)) =
                            (cmp.left.as_ref(), &cmp.comparators[0])
                        {
                            let var_name = var.id.to_string();
                            if ctx.scope.lookup(&var_name).is_some() {
                                return Some(NarrowingCondition::IsNone(var_name));
                            }
                        }
                    }
                    CmpOp::IsNot => {
                        if let (Expr::Name(var), Expr::NoneLiteral(_)) =
                            (cmp.left.as_ref(), &cmp.comparators[0])
                        {
                            let var_name = var.id.to_string();
                            if ctx.scope.lookup(&var_name).is_some() {
                                return Some(NarrowingCondition::IsNotNone(var_name));
                            }
                        }
                    }
                    CmpOp::Eq => {
                        if let Expr::Name(var) = cmp.left.as_ref() {
                            let var_name = var.id.to_string();
                            if ctx.scope.lookup(&var_name).is_some() {
                                if let Some(lit_val) = expr_to_literal_value(&cmp.comparators[0]) {
                                    return Some(NarrowingCondition::Equality(var_name, lit_val));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        Expr::Name(name) => {
            let var_name = name.id.to_string();
            if ctx.scope.lookup(&var_name).is_some() {
                Some(NarrowingCondition::Truthiness(var_name))
            } else {
                None
            }
        }
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::Not) => {
            let inner = detect_narrowing_condition(&unary.operand, ctx)?;
            Some(NarrowingCondition::Not(Box::new(inner)))
        }
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::And) => {
            let conditions: Vec<NarrowingCondition> = boolop
                .values
                .iter()
                .filter_map(|value| detect_narrowing_condition(value, ctx))
                .collect();
            if conditions.is_empty() {
                None
            } else if conditions.len() == 1 {
                conditions.into_iter().next()
            } else {
                Some(NarrowingCondition::And(conditions))
            }
        }
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::Or) => {
            let conditions: Vec<NarrowingCondition> = boolop
                .values
                .iter()
                .filter_map(|value| detect_narrowing_condition(value, ctx))
                .collect();
            if conditions.is_empty() {
                None
            } else if conditions.len() == 1 {
                conditions.into_iter().next()
            } else {
                Some(NarrowingCondition::Or(conditions))
            }
        }
        _ => None,
    }
}

/// Apply narrowing to the scope based on a condition.
pub(in crate::lower) fn apply_narrowing(
    ctx: &mut LowerCtx,
    condition: &NarrowingCondition,
    is_true: bool,
) {
    match condition {
        NarrowingCondition::And(conditions) => {
            if is_true {
                for cond in conditions {
                    apply_narrowing(ctx, cond, true);
                }
            }
        }
        NarrowingCondition::Or(conditions) => {
            if !is_true {
                for cond in conditions {
                    apply_narrowing(ctx, cond, false);
                }
            }
        }
        _ => {
            if let Some(var_name) = condition.var_name() {
                if let Some(info) = ctx.scope.lookup(var_name) {
                    let current_ty = info.effective_type().clone();
                    let effects = crate::flow_graph::narrowing_effects_for_condition(
                        condition,
                        is_true,
                        &current_ty,
                    );
                    let narrowed = effects
                        .iter()
                        .find_map(|effect| match effect {
                            crate::flow_graph::FlowEffect::Narrow {
                                binding,
                                narrowed_type,
                                ..
                            } if binding == var_name => Some(narrowed_type.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| {
                            sifr_type_system::narrow_type(&current_ty, condition, is_true)
                        });
                    ctx.narrow_var_with_flow(var_name, narrowed, format!("{condition:?}"), is_true);
                }
            }
        }
    }
}
