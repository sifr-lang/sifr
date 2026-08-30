use crate::hir_nodes::HirExpr;
use crate::scope::ConstIntegerSnapshot;
use num_bigint::BigInt;
use sifr_python_ast::Stmt;
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
    for name in assigned_names {
        ctx.scope.clear_const_integer_value(&name);
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
