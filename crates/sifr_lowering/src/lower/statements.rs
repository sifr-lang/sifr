use super::LowerCtx;
use super::assignment_widening::reconcile_optional_reassignment;
use super::async_generator_advances::{
    finish_async_generator_advance_for_expr, record_async_generator_advance_binding,
};
pub(in crate::lower) use super::aug_assign_lowering::lower_aug_assign;
use super::binding_mutability::ensure_mutable_parameter_binding;
use super::builtin_calls::callable_builtin_element_type;
use super::container_literal_specialization::validate_subscript_assignment_target;
use super::control_flow_conditions::validate_control_flow_condition;
use super::expressions::{lower_expr, lower_star_unpack_assign, lower_tuple_unpack_assign};
use super::fixed_width_class_payload::class_specialization_payload_conflicts;
use super::fixed_width_fitting::{FixedWidthInitializerFit, validate_fixed_width_initializer};
use super::flow_helpers::{body_always_leaves_current_path, then_body_always_exits};
use super::for_loop_safety::{is_collection_backed_iter_source, loop_body_mutates_iter_source};
use super::if_branch_bindings::{
    predeclare_exhaustive_if_assigned_names, seed_exhaustive_if_bindings,
};
use super::integer_const_facts::{
    invalidate_loop_body_const_integer_facts, record_const_integer_binding,
    restore_const_integer_state_after_branches,
};
use super::integer_nonzero_guards::{
    detect_false_nonzero_integer_guards, detect_true_nonzero_integer_guards,
};
use super::len_aliases::record_len_alias_fact;
use super::match_diagnostics;
use super::name_diagnostics;
use super::narrowing::{apply_narrowing, detect_narrowing_condition};
use super::nonlocal_support::should_rebind_simple_name;
use super::numeric_sentinels::{
    domain_typed_sentinel_expr, numeric_domain_for_type, numeric_sentinel_kind,
};
use super::ownership_diagnostics;
use super::protocol_diagnostics;
use super::sequence_guard_detection::{
    detect_false_exit_sequence_guards, detect_range_sequence_guards, detect_true_sequence_guards,
    detect_while_sequence_guards,
};
use super::sequence_guard_updates::{
    maybe_record_dict_assignment_guard, merge_exhaustive_branch_sequence_guards,
};
use super::sequence_pointers::record_sequence_pointer_fact;
use super::sequence_shapes::sequence_shape_fact;
use super::statement_diagnostics;
use super::task_scope_calls::task_group_spawn_owner;
use super::typing_and_functions::resolve_annotation_expr;
use super::{
    append_growth_shapes, async_for, async_generator_advances, async_with,
    container_literal_specialization, diagnostics, expressions, fallback_error_type,
    flow_diagnostics, function_flow, match_lowering, nested_function_inference, nonlocal_support,
    numeric_sentinels, result_diagnostics, return_lowering, str, try_error_propagation,
    typing_and_functions,
};
use crate::hir_nodes::{HirExpr, HirIteratorOp, HirPattern, HirStmt};
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{
    Expr, Pattern, Singleton, StmtAnnAssign, StmtAssign, StmtFor, StmtIf, StmtWhile,
};
use sifr_type_system::{FunctionType, NarrowingCondition, Type, make_union};

mod statement_dispatch;
pub(in crate::lower) use statement_dispatch::*;
mod binding_hint_adoption;
pub(in crate::lower) use binding_hint_adoption::*;
mod function_body;
mod nested_function_state;
pub(in crate::lower) use function_body::lower_function_stmts;
mod patterns_and_assignments;
pub(in crate::lower) use patterns_and_assignments::*;
mod control_flow;
pub(in crate::lower) use control_flow::*;
mod while_loop;
pub(in crate::lower) use while_loop::lower_while;

pub(in crate::lower) fn record_try_error_types(ctx: &mut LowerCtx, error_type: &Type) {
    match error_type.resolve_alias() {
        Type::Class { .. } => {
            ctx.try_block_error_types
                .insert(error_type.resolve_alias().clone());
        }
        Type::Union(members) => {
            for member in members {
                record_try_error_types(ctx, member);
            }
        }
        _ => {}
    }
}

pub(in crate::lower) fn lower_python_context_body(
    stmts: &[sifr_python_ast::Stmt],
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> (Vec<HirStmt>, bool) {
    let enclosing_errors = std::mem::take(&mut ctx.try_block_error_types);
    let body = lower_stmts(stmts, func_type, ctx);
    let propagated_errors = std::mem::take(&mut ctx.try_block_error_types);

    let mut explicit_errors = std::collections::HashSet::new();
    diagnostics::collect_raise_error_types(&body, &mut explicit_errors);
    let body_may_raise = !propagated_errors.is_empty() || !explicit_errors.is_empty();

    ctx.try_block_error_types = enclosing_errors;
    ctx.try_block_error_types.extend(propagated_errors);
    (body, body_may_raise)
}
