use super::sequence_guards::{key_guard_token, SequenceGuard};
use super::LowerCtx;
use sifr_python_ast::Expr;
use sifr_type_system::Type;

pub(in crate::lower) fn maybe_record_dict_assignment_guard(
    ctx: &mut LowerCtx,
    object_ty: &Type,
    object_name: &str,
    key_expr: &Expr,
) {
    if !matches!(object_ty.resolve_alias(), Type::Dict(_, _)) {
        return;
    }
    let Some(key_expr_debug) = key_guard_token(key_expr) else {
        return;
    };
    ctx.add_sequence_guard(SequenceGuard::DictContains {
        dict: object_name.to_string(),
        key_expr_debug,
    });
}

pub(in crate::lower) fn merge_exhaustive_branch_sequence_guards(
    ctx: &mut LowerCtx,
    has_else_branch: bool,
    branch_sequence_states: &[Vec<SequenceGuard>],
) {
    if !has_else_branch || branch_sequence_states.is_empty() {
        return;
    }
    let mut common_guards = branch_sequence_states[0].clone();
    common_guards.retain(|guard| {
        branch_sequence_states
            .iter()
            .all(|branch| branch.contains(guard))
    });
    for guard in common_guards {
        ctx.add_sequence_guard(guard);
    }
}
