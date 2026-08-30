use crate::hir_nodes::HirExpr;
use crate::scope::ConstIntegerSnapshot;
use num_bigint::BigInt;
use sifr_python_ast::visitor::{self, Visitor};
use sifr_python_ast::{Expr, Stmt};
use sifr_type_system::Type;
use std::collections::HashSet;

use super::LowerCtx;

fn const_integer_value_for_binding(value: &HirExpr) -> Option<BigInt> {
    match value {
        HirExpr::IntLiteral(value) => Some(BigInt::from(*value)),
        HirExpr::LargeIntLiteral(value) => value.parse::<BigInt>().ok(),
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            const_integer_value_for_binding(operand).map(|value| -value)
        }
        _ => None,
    }
}

pub(in crate::lower) fn record_const_integer_binding(
    ctx: &mut LowerCtx,
    name: &str,
    value: &HirExpr,
) {
    if let Some(const_value) = const_integer_value_for_binding(value) {
        ctx.scope.set_const_integer_value(name, const_value);
    } else {
        ctx.scope.clear_const_integer_value(name);
    }
}

pub(in crate::lower) fn invalidate_loop_body_const_integer_facts(
    ctx: &mut LowerCtx,
    body: &[Stmt],
) {
    let mut assigned_names = HashSet::new();
    super::nested_function_inference::collect_current_function_local_bindings(
        body,
        &mut assigned_names,
    );
    assigned_names
        .extend(super::nested_function_inference::collect_nested_function_mutated_nonlocals(body));
    for name in assigned_names {
        ctx.scope.clear_const_integer_value(&name);
    }

    let mut calls = NestedFunctionCallCollector::default();
    for stmt in body {
        calls.visit_stmt(stmt);
    }
    let may_call_aliased_nested_function = calls.called.iter().any(|function| {
        !ctx.nested_function_mutated_captures.contains_key(function)
            && ctx.scope.lookup(function).is_some_and(|binding| {
                matches!(
                    binding.effective_type().resolve_alias(),
                    Type::Callable(..) | Type::AsyncCallable(..)
                )
            })
    });
    let mutated_captures = if may_call_aliased_nested_function {
        all_nested_function_mutated_captures(ctx)
    } else {
        calls
            .called
            .iter()
            .filter_map(|function| ctx.nested_function_mutated_captures.get(function))
            .flatten()
            .cloned()
            .collect()
    };
    for name in mutated_captures {
        ctx.scope.clear_const_integer_value(&name);
    }
}

pub(in crate::lower) fn invalidate_aliased_nested_function_call_const_integer_facts(
    ctx: &mut LowerCtx,
) {
    for name in all_nested_function_mutated_captures(ctx) {
        ctx.scope.clear_const_integer_value(&name);
    }
}

fn all_nested_function_mutated_captures(ctx: &LowerCtx) -> HashSet<String> {
    ctx.nested_function_mutated_captures
        .values()
        .flatten()
        .cloned()
        .collect()
}

pub(in crate::lower) fn invalidate_nested_function_call_const_integer_facts(
    ctx: &mut LowerCtx,
    function: &str,
) {
    let mutated_captures = ctx
        .nested_function_mutated_captures
        .get(function)
        .cloned()
        .unwrap_or_default();
    for name in mutated_captures {
        ctx.scope.clear_const_integer_value(&name);
    }
}

#[derive(Default)]
struct NestedFunctionCallCollector {
    called: HashSet<String>,
}

impl<'ast> Visitor<'ast> for NestedFunctionCallCollector {
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if matches!(stmt, Stmt::FunctionDef(_)) {
            return;
        }
        visitor::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        if let Expr::Call(call) = expr
            && let Expr::Name(function) = call.func.as_ref()
        {
            self.called.insert(function.id.to_string());
        }
        visitor::walk_expr(self, expr);
    }
}

fn snapshot_const_value<'a>(
    snapshot: &'a ConstIntegerSnapshot,
    name: &str,
) -> Option<&'a Option<BigInt>> {
    snapshot
        .iter()
        .find_map(|(snapshot_name, value)| (snapshot_name == name).then_some(value))
}

pub(in crate::lower) fn restore_const_integer_state_after_branches(
    ctx: &mut LowerCtx,
    saved: &ConstIntegerSnapshot,
    branch_states: &[(ConstIntegerSnapshot, bool)],
) {
    ctx.scope.restore_const_integer_state(saved);
    for (name, saved_value) in saved {
        let changed_by_live_branch = branch_states
            .iter()
            .filter(|(_, branch_exits)| !*branch_exits)
            .any(|(branch_state, _)| snapshot_const_value(branch_state, name) != Some(saved_value));
        if changed_by_live_branch {
            ctx.scope.clear_const_integer_value(name);
        }
    }
}
