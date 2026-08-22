use super::expr_references_var;
use crate::hir_analysis::traversal::{self, TraversalConfig};
use sifr_ir::{
    HirAsyncWithKind, HirExpr, HirPattern, HirStmt, HirTupleTargetBinding, MutableArgumentTarget,
};
use sifr_type_system::ReceiverConvention;
use std::cell::Cell;

pub(crate) fn stmts_require_var_value_at_entry_including_nested_functions(
    stmts: &[HirStmt],
    var_name: &str,
) -> bool {
    block_requires_value(stmts, var_name, false)
}

pub(crate) fn declaration_only_binding_needs_mutability(stmts: &[HirStmt], var_name: &str) -> bool {
    max_reassignments_on_path(stmts, var_name) > 1 || stmts_mutate_existing_value(stmts, var_name)
}

fn max_reassignments_on_path(stmts: &[HirStmt], var_name: &str) -> usize {
    stmts.iter().fold(0, |total, stmt| {
        (total + stmt_max_reassignments(stmt, var_name)).min(2)
    })
}

fn stmt_max_reassignments(stmt: &HirStmt, var_name: &str) -> usize {
    match stmt {
        HirStmt::Let { name, .. }
        | HirStmt::Assign { name, .. }
        | HirStmt::AugAssign { name, .. } => usize::from(name == var_name),
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => std::iter::once(max_reassignments_on_path(then_body, var_name))
            .chain(
                elif_clauses
                    .iter()
                    .map(|(_, body)| max_reassignments_on_path(body, var_name)),
            )
            .chain(
                else_body
                    .as_deref()
                    .map(|body| max_reassignments_on_path(body, var_name)),
            )
            .max()
            .unwrap_or(0),
        HirStmt::While {
            body, else_body, ..
        } => {
            let body_count = max_reassignments_on_path(body, var_name);
            let repeated_body_count = if body_count > 0 { 2 } else { 0 };
            (repeated_body_count
                + else_body
                    .as_deref()
                    .map_or(0, |body| max_reassignments_on_path(body, var_name)))
            .min(2)
        }
        HirStmt::For {
            target,
            body,
            else_body,
            ..
        }
        | HirStmt::AsyncFor {
            target,
            body,
            else_body,
            ..
        } => {
            let body_count =
                max_reassignments_on_path(body, var_name) + usize::from(target == var_name);
            let repeated_body_count = if body_count > 0 { 2 } else { 0 };
            (repeated_body_count
                + else_body
                    .as_deref()
                    .map_or(0, |body| max_reassignments_on_path(body, var_name)))
            .min(2)
        }
        HirStmt::TupleUnpack { targets, .. } => usize::from(targets.iter().any(|target| {
            matches!(
                &target.binding,
                HirTupleTargetBinding::Name(name)
                    if target.rebind_existing && name == var_name
            )
        })),
        HirStmt::StarUnpack {
            before,
            star,
            after,
            ..
        } => usize::from(
            before.iter().chain(after).any(|(name, _)| name == var_name) || star.0 == var_name,
        ),
        HirStmt::TryExcept { body, handlers, .. } => {
            let body_count = max_reassignments_on_path(body, var_name);
            let handler_count = handlers
                .iter()
                .map(|handler| max_reassignments_on_path(&handler.body, var_name))
                .max()
                .unwrap_or(0);
            (body_count + handler_count).min(2)
        }
        HirStmt::TryFinally { body, finalbody } => (max_reassignments_on_path(body, var_name)
            + max_reassignments_on_path(finalbody, var_name))
        .min(2),
        HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
            max_reassignments_on_path(body, var_name)
        }
        HirStmt::NestedFunction { func, .. } => usize::from(
            !func.params.iter().any(|param| param.name == var_name)
                && max_reassignments_on_path(&func.body, var_name) > 0,
        ),
        HirStmt::Match { arms, .. } => arms
            .iter()
            .map(|arm| max_reassignments_on_path(&arm.body, var_name))
            .max()
            .unwrap_or(0),
        HirStmt::Return { .. }
        | HirStmt::Expr { .. }
        | HirStmt::Break
        | HirStmt::Continue
        | HirStmt::Pass
        | HirStmt::Assert { .. }
        | HirStmt::Raise { .. }
        | HirStmt::FieldAssign { .. }
        | HirStmt::NestedFieldAssign { .. }
        | HirStmt::SubscriptAssign { .. }
        | HirStmt::NestedSubscriptAssign { .. }
        | HirStmt::AttributeNestedSubscriptAssign { .. }
        | HirStmt::SubscriptAugAssign { .. }
        | HirStmt::AttributeAugAssign { .. }
        | HirStmt::AttributeSubscriptAssign { .. }
        | HirStmt::Delete { .. }
        | HirStmt::Yield { .. } => 0,
    }
}

fn stmts_mutate_existing_value(stmts: &[HirStmt], var_name: &str) -> bool {
    let mutates = Cell::new(false);
    let mut on_stmt = |stmt: &HirStmt| {
        mutates.set(
            mutates.get()
                || match stmt {
                    HirStmt::AugAssign { name, .. } => name == var_name,
                    HirStmt::SubscriptAssign { object, .. }
                    | HirStmt::NestedSubscriptAssign { object, .. }
                    | HirStmt::AttributeNestedSubscriptAssign { object, .. }
                    | HirStmt::SubscriptAugAssign { object, .. }
                    | HirStmt::AttributeAugAssign { object, .. }
                    | HirStmt::FieldAssign { object, .. }
                    | HirStmt::NestedFieldAssign { object, .. }
                    | HirStmt::AttributeSubscriptAssign { object, .. } => object == var_name,
                    HirStmt::Delete {
                        object: HirExpr::Name { name, .. },
                        ..
                    } => name == var_name,
                    _ => false,
                },
        );
    };
    let mut on_expr = |expr: &HirExpr| {
        mutates.set(mutates.get() || expr_mutates_var(expr, var_name));
    };
    traversal::walk_stmts(
        stmts,
        TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
        &mut on_stmt,
        &mut on_expr,
    );
    mutates.get()
}

fn expr_mutates_var(expr: &HirExpr, var_name: &str) -> bool {
    match expr {
        HirExpr::MethodCall {
            object,
            args,
            receiver_convention,
            mutable_arg_places,
            ..
        } => {
            (*receiver_convention == Some(ReceiverConvention::MutableBorrow)
                && expression_root_name(object) == Some(var_name))
                || mutable_args_include_var(args, mutable_arg_places, var_name)
        }
        HirExpr::Call {
            args,
            mutable_arg_places,
            ..
        }
        | HirExpr::GenericCall {
            args,
            mutable_arg_places,
            ..
        }
        | HirExpr::IteratorCall {
            args,
            mutable_arg_places,
            ..
        } => mutable_args_include_var(args, mutable_arg_places, var_name),
        _ => false,
    }
}

fn mutable_args_include_var(
    args: &[HirExpr],
    targets: &[Option<MutableArgumentTarget>],
    var_name: &str,
) -> bool {
    args.iter()
        .zip(targets)
        .any(|(arg, target)| target.is_some() && expression_root_name(arg) == Some(var_name))
}

fn expression_root_name(expr: &HirExpr) -> Option<&str> {
    match expr {
        HirExpr::Name { name, .. } => Some(name),
        HirExpr::FieldAccess { object, .. } | HirExpr::Index { object, .. } => {
            expression_root_name(object)
        }
        _ => None,
    }
}

fn block_requires_value(stmts: &[HirStmt], var_name: &str, mut live: bool) -> bool {
    for stmt in stmts.iter().rev() {
        live = stmt_requires_value(stmt, var_name, live);
    }
    live
}

fn stmt_requires_value(stmt: &HirStmt, var_name: &str, live_after: bool) -> bool {
    match stmt {
        HirStmt::Let { name, value, .. } | HirStmt::Assign { name, value } => {
            expr_references_var(value, var_name) || (name != var_name && live_after)
        }
        HirStmt::AugAssign { name, value, .. } => {
            expr_references_var(value, var_name) || name == var_name || live_after
        }
        HirStmt::Return { value } => value
            .as_ref()
            .is_some_and(|value| expr_references_var(value, var_name)),
        HirStmt::Expr { expr } => expr_references_var(expr, var_name) || live_after,
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            let conditions_reference_value = expr_references_var(condition, var_name)
                || elif_clauses
                    .iter()
                    .any(|(condition, _)| expr_references_var(condition, var_name));
            let then_live = block_requires_value(then_body, var_name, live_after);
            let elif_live = elif_clauses
                .iter()
                .any(|(_, body)| block_requires_value(body, var_name, live_after));
            let else_live = else_body.as_deref().map_or(live_after, |body| {
                block_requires_value(body, var_name, live_after)
            });
            conditions_reference_value || then_live || elif_live || else_live
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            live_after
                || expr_references_var(condition, var_name)
                || block_requires_value(body, var_name, live_after)
                || else_body
                    .as_deref()
                    .is_some_and(|body| block_requires_value(body, var_name, live_after))
        }
        HirStmt::For {
            target,
            iter,
            body,
            else_body,
            ..
        }
        | HirStmt::AsyncFor {
            target,
            iter,
            body,
            else_body,
            ..
        } => {
            live_after
                || expr_references_var(iter, var_name)
                || (target != var_name && block_requires_value(body, var_name, live_after))
                || else_body
                    .as_deref()
                    .is_some_and(|body| block_requires_value(body, var_name, live_after))
        }
        HirStmt::Break | HirStmt::Continue => live_after,
        HirStmt::TupleUnpack { targets, value } => {
            let target_reads_value = targets.iter().any(|target| match &target.binding {
                HirTupleTargetBinding::Name(_) => false,
                HirTupleTargetBinding::Field { object, .. } => object == var_name,
            });
            let rebinds_value = targets.iter().any(|target| {
                matches!(
                    &target.binding,
                    HirTupleTargetBinding::Name(name)
                        if target.rebind_existing && name == var_name
                )
            });
            expr_references_var(value, var_name)
                || target_reads_value
                || (live_after && !rebinds_value)
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            value,
        } => {
            let rebinds_value =
                before.iter().chain(after).any(|(name, _)| name == var_name) || star.0 == var_name;
            expr_references_var(value, var_name) || (live_after && !rebinds_value)
        }
        HirStmt::Pass => live_after,
        HirStmt::Assert { test, msg } => {
            expr_references_var(test, var_name)
                || msg
                    .as_ref()
                    .is_some_and(|message| expr_references_var(message, var_name))
                || live_after
        }
        HirStmt::Raise { value } => expr_references_var(value, var_name),
        HirStmt::TryExcept { body, handlers, .. } => {
            block_requires_value(body, var_name, live_after)
                || handlers
                    .iter()
                    .any(|handler| block_requires_value(&handler.body, var_name, live_after))
        }
        HirStmt::TryFinally { body, finalbody } => {
            let final_live = block_requires_value(finalbody, var_name, live_after);
            final_live || block_requires_value(body, var_name, final_live)
        }
        HirStmt::FieldAssign { object, value, .. }
        | HirStmt::NestedFieldAssign { object, value, .. }
        | HirStmt::AttributeAugAssign { object, value, .. } => {
            object == var_name || expr_references_var(value, var_name) || live_after
        }
        HirStmt::SubscriptAssign {
            object,
            index,
            value,
            ..
        }
        | HirStmt::SubscriptAugAssign {
            object,
            index,
            value,
            ..
        }
        | HirStmt::AttributeSubscriptAssign {
            object,
            index,
            value,
            ..
        } => {
            object == var_name
                || expr_references_var(index, var_name)
                || expr_references_var(value, var_name)
                || live_after
        }
        HirStmt::NestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            ..
        }
        | HirStmt::AttributeNestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            ..
        } => {
            object == var_name
                || expr_references_var(outer_index, var_name)
                || expr_references_var(inner_index, var_name)
                || expr_references_var(value, var_name)
                || live_after
        }
        HirStmt::Delete { object, index } => {
            expr_references_var(object, var_name)
                || expr_references_var(index, var_name)
                || live_after
        }
        HirStmt::Yield { value } => expr_references_var(value, var_name) || live_after,
        HirStmt::With { items, body } => {
            items
                .iter()
                .any(|item| expr_references_var(&item.context, var_name))
                || block_requires_value(body, var_name, live_after)
        }
        HirStmt::AsyncWith { kind, target, body } => {
            async_with_references_var(kind, var_name)
                || (target.as_deref() != Some(var_name)
                    && block_requires_value(body, var_name, live_after))
                || live_after
        }
        HirStmt::NestedFunction { func, .. } => {
            let defaults_reference_value = func.params.iter().any(|param| {
                param
                    .default
                    .as_ref()
                    .is_some_and(|default| expr_references_var(default, var_name))
            });
            let parameter_shadows_value = func.params.iter().any(|param| param.name == var_name);
            defaults_reference_value
                || (!parameter_shadows_value
                    && super::stmts_reference_var_including_nested_functions(&func.body, var_name))
                || live_after
        }
        HirStmt::Match { subject, arms, .. } => {
            expr_references_var(subject, var_name)
                || live_after
                || arms.iter().any(|arm| {
                    pattern_references_var(&arm.pattern, var_name)
                        || (!pattern_captures_var(&arm.pattern, var_name)
                            && (arm
                                .guard
                                .as_ref()
                                .is_some_and(|guard| expr_references_var(guard, var_name))
                                || block_requires_value(&arm.body, var_name, live_after)))
                })
        }
    }
}

fn async_with_references_var(kind: &HirAsyncWithKind, var_name: &str) -> bool {
    match kind {
        HirAsyncWithKind::TaskScope => false,
        HirAsyncWithKind::TaskGroup { context } => context
            .as_ref()
            .is_some_and(|context| expr_references_var(context, var_name)),
        HirAsyncWithKind::TaskTimeout { duration } => expr_references_var(duration, var_name),
        HirAsyncWithKind::UserDefined { context, .. }
        | HirAsyncWithKind::Python { context, .. } => expr_references_var(context, var_name),
    }
}

fn pattern_captures_var(pattern: &HirPattern, var_name: &str) -> bool {
    match pattern {
        HirPattern::Capture { name, .. } => name == var_name,
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => patterns
            .iter()
            .any(|pattern| pattern_captures_var(pattern, var_name)),
        HirPattern::Class { fields, .. } => fields
            .iter()
            .any(|(_, pattern)| pattern_captures_var(pattern, var_name)),
        HirPattern::Wildcard
        | HirPattern::Literal { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => false,
    }
}

fn pattern_references_var(pattern: &HirPattern, var_name: &str) -> bool {
    match pattern {
        HirPattern::Literal { value } => expr_references_var(value, var_name),
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => patterns
            .iter()
            .any(|pattern| pattern_references_var(pattern, var_name)),
        HirPattern::Class { fields, .. } => fields
            .iter()
            .any(|(_, pattern)| pattern_references_var(pattern, var_name)),
        HirPattern::Wildcard
        | HirPattern::Capture { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        declaration_only_binding_needs_mutability,
        stmts_require_var_value_at_entry_including_nested_functions,
    };
    use sifr_ir::{HirExpr, HirFunction, HirParam, HirStmt, MethodKind};
    use sifr_type_system::{ParamConvention, Type};

    fn name(value: &str) -> HirExpr {
        HirExpr::Name {
            name: value.to_string(),
            binding_id: None,
            ty: Type::Str,
        }
    }

    #[test]
    fn unconditional_replacement_kills_the_incoming_value() {
        let stmts = vec![
            HirStmt::Assign {
                name: "value".to_string(),
                value: HirExpr::StringLiteral("new".to_string()),
            },
            HirStmt::Return {
                value: Some(name("value")),
            },
        ];

        assert!(!stmts_require_var_value_at_entry_including_nested_functions(&stmts, "value"));
    }

    #[test]
    fn replacement_rhs_can_keep_the_incoming_value_live() {
        let stmts = vec![HirStmt::Assign {
            name: "value".to_string(),
            value: name("value"),
        }];

        assert!(stmts_require_var_value_at_entry_including_nested_functions(
            &stmts, "value"
        ));
    }

    #[test]
    fn every_continuing_branch_must_replace_the_incoming_value() {
        let complete = vec![
            HirStmt::If {
                condition: HirExpr::BoolLiteral(true),
                then_body: vec![HirStmt::Assign {
                    name: "value".to_string(),
                    value: HirExpr::StringLiteral("then".to_string()),
                }],
                elif_clauses: vec![],
                else_body: Some(vec![HirStmt::Assign {
                    name: "value".to_string(),
                    value: HirExpr::StringLiteral("else".to_string()),
                }]),
            },
            HirStmt::Return {
                value: Some(name("value")),
            },
        ];
        let partial = vec![
            HirStmt::If {
                condition: HirExpr::BoolLiteral(true),
                then_body: vec![HirStmt::Assign {
                    name: "value".to_string(),
                    value: HirExpr::StringLiteral("then".to_string()),
                }],
                elif_clauses: vec![],
                else_body: None,
            },
            HirStmt::Return {
                value: Some(name("value")),
            },
        ];

        assert!(!stmts_require_var_value_at_entry_including_nested_functions(&complete, "value"));
        assert!(stmts_require_var_value_at_entry_including_nested_functions(
            &partial, "value"
        ));
    }

    #[test]
    fn one_unconditional_replacement_does_not_need_mutability() {
        let stmts = vec![HirStmt::Assign {
            name: "value".to_string(),
            value: HirExpr::IntLiteral(1),
        }];

        assert!(!declaration_only_binding_needs_mutability(&stmts, "value"));
    }

    #[test]
    fn repeated_replacement_on_one_path_needs_mutability() {
        let stmts = vec![
            HirStmt::Assign {
                name: "value".to_string(),
                value: HirExpr::IntLiteral(1),
            },
            HirStmt::Assign {
                name: "value".to_string(),
                value: HirExpr::IntLiteral(2),
            },
        ];

        assert!(declaration_only_binding_needs_mutability(&stmts, "value"));
    }

    #[test]
    fn alternative_branch_replacements_do_not_need_mutability() {
        let stmts = vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Assign {
                name: "value".to_string(),
                value: HirExpr::IntLiteral(1),
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Assign {
                name: "value".to_string(),
                value: HirExpr::IntLiteral(2),
            }]),
        }];

        assert!(!declaration_only_binding_needs_mutability(&stmts, "value"));
    }

    #[test]
    fn nested_function_default_reads_the_value_in_the_defining_scope() {
        let stmts = vec![HirStmt::NestedFunction {
            func: HirFunction {
                name: "read_value".to_string(),
                params: vec![HirParam {
                    name: "candidate".to_string(),
                    ty: Type::Str,
                    default: Some(name("value")),
                    keyword_only: false,
                    convention: ParamConvention::own(),
                }],
                return_type: Type::Str,
                body: vec![HirStmt::Return {
                    value: Some(name("candidate")),
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
            move_captures: false,
            capture_clones: Vec::new(),
        }];

        assert!(stmts_require_var_value_at_entry_including_nested_functions(
            &stmts, "value"
        ));
    }
}
