use super::LowerCtx;
use super::sequence_guards::{SequenceGuard, SubscriptReferenceStability, key_guard_token};
use sifr_python_ast::visitor::{self, Visitor};
use sifr_python_ast::{
    BoolOp, CmpOp, Expr, Number, Operator, Stmt, StmtAssign, StmtAugAssign, StmtFor, StmtWhile,
    UnaryOp,
};
use sifr_type_system::Type;

mod loop_invalidations;
mod nonnegative_guards;

pub(in crate::lower) use loop_invalidations::loop_invalidated_sequence_targets;

pub(in crate::lower) fn detect_while_sequence_guards(
    while_stmt: &StmtWhile,
    ctx: &LowerCtx,
) -> Vec<SequenceGuard> {
    let mut guards = detect_true_sequence_guards(&while_stmt.test, ctx);
    guards.extend(detect_two_pointer_while_guards(while_stmt, ctx));
    guards
}

pub(in crate::lower) fn detect_true_sequence_guards(
    expr: &Expr,
    ctx: &LowerCtx,
) -> Vec<SequenceGuard> {
    match expr {
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::And) => boolop
            .values
            .iter()
            .flat_map(|value| detect_true_sequence_guards(value, ctx))
            .collect(),
        Expr::Call(_) => len_call_sequence_name(expr)
            .map(|sequence| {
                vec![SequenceGuard::MinLength {
                    sequence,
                    min_len: 1,
                }]
            })
            .unwrap_or_default(),
        Expr::Name(name) => vec![SequenceGuard::MinLength {
            sequence: name.id.to_string(),
            min_len: 1,
        }],
        Expr::Attribute(_) => sequence_guard_target_name(expr)
            .map(|sequence| {
                vec![SequenceGuard::MinLength {
                    sequence,
                    min_len: 1,
                }]
            })
            .unwrap_or_default(),
        Expr::Compare(cmp) if cmp.ops.len() == 1 && cmp.comparators.len() == 1 => {
            if let Some(guards) = nonnegative_guards::detect_true_guards(cmp, ctx) {
                return guards;
            }
            match &cmp.ops[0] {
                CmpOp::Lt => {
                    if let (Some((index_var, max_offset)), Some(sequence_name)) = (
                        index_var_with_nonnegative_offset(cmp.left.as_ref()),
                        len_anchor_name(&cmp.comparators[0], ctx),
                    ) {
                        return vec![SequenceGuard::IndexVarInRange {
                            sequence: sequence_name,
                            index_var,
                            max_offset,
                        }];
                    }
                    Vec::new()
                }
                CmpOp::Eq => {
                    if let (Some(sequence_name), Some(len_value)) = (
                        len_anchor_name(cmp.left.as_ref(), ctx),
                        literal_usize(&cmp.comparators[0]),
                    ) {
                        if len_value > 0 {
                            return vec![SequenceGuard::MinLength {
                                sequence: sequence_name,
                                min_len: len_value,
                            }];
                        }
                    }
                    Vec::new()
                }
                CmpOp::Gt => {
                    if let (Some(sequence_name), Some(len_value)) = (
                        len_anchor_name(cmp.left.as_ref(), ctx),
                        literal_usize(&cmp.comparators[0]),
                    ) {
                        return vec![SequenceGuard::MinLength {
                            sequence: sequence_name,
                            min_len: len_value + 1,
                        }];
                    }
                    Vec::new()
                }
                CmpOp::GtE | CmpOp::LtE => Vec::new(),
                CmpOp::IsNot => subscript_present_guard_from_non_none_compare(
                    cmp.left.as_ref(),
                    &cmp.comparators[0],
                ),
                CmpOp::In => dict_contains_guard(cmp.left.as_ref(), &cmp.comparators[0]),
                _ => Vec::new(),
            }
        }
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::Not) => {
            detect_false_exit_sequence_guards(&unary.operand, ctx)
        }
        _ => Vec::new(),
    }
}

pub(in crate::lower) fn detect_false_exit_sequence_guards(
    expr: &Expr,
    ctx: &LowerCtx,
) -> Vec<SequenceGuard> {
    match expr {
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::Or) => boolop
            .values
            .iter()
            .flat_map(|value| detect_false_exit_sequence_guards(value, ctx))
            .collect(),
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::Not) => {
            detect_true_sequence_guards(&unary.operand, ctx)
        }
        Expr::Call(_) => len_call_sequence_name(expr)
            .map(|sequence| {
                vec![SequenceGuard::MinLength {
                    sequence,
                    min_len: 1,
                }]
            })
            .unwrap_or_default(),
        Expr::Name(name) => vec![SequenceGuard::MinLength {
            sequence: name.id.to_string(),
            min_len: 1,
        }],
        Expr::Compare(cmp) if cmp.ops.len() == 1 && cmp.comparators.len() == 1 => {
            if let Some(guard) = nonnegative_guards::detect_false_exit_guard(cmp) {
                return vec![guard];
            }
            match &cmp.ops[0] {
                CmpOp::Eq => {
                    if let (Expr::Name(index_name), Some(sequence_name)) =
                        (cmp.left.as_ref(), len_anchor_name(&cmp.comparators[0], ctx))
                    {
                        return vec![SequenceGuard::IndexVarInRange {
                            sequence: sequence_name,
                            index_var: index_name.id.to_string(),
                            max_offset: 0,
                        }];
                    }
                    if let (Some(sequence_name), Expr::Name(index_name)) =
                        (len_anchor_name(cmp.left.as_ref(), ctx), &cmp.comparators[0])
                    {
                        return vec![SequenceGuard::IndexVarInRange {
                            sequence: sequence_name,
                            index_var: index_name.id.to_string(),
                            max_offset: 0,
                        }];
                    }
                    let Some(sequence_name) = len_anchor_name(cmp.left.as_ref(), ctx) else {
                        return Vec::new();
                    };
                    let Some(len_value) = literal_usize(&cmp.comparators[0]) else {
                        return Vec::new();
                    };
                    let current_min_len = ctx.min_length_guard(&sequence_name);
                    if current_min_len >= len_value {
                        vec![SequenceGuard::MinLength {
                            sequence: sequence_name,
                            min_len: len_value + 1,
                        }]
                    } else if len_value == 0 {
                        vec![SequenceGuard::MinLength {
                            sequence: sequence_name,
                            min_len: 1,
                        }]
                    } else {
                        Vec::new()
                    }
                }
                CmpOp::Lt => {
                    let Some(sequence_name) = len_anchor_name(cmp.left.as_ref(), ctx) else {
                        return Vec::new();
                    };
                    let Some(min_len) = literal_usize(&cmp.comparators[0]) else {
                        return Vec::new();
                    };
                    vec![SequenceGuard::MinLength {
                        sequence: sequence_name,
                        min_len,
                    }]
                }
                CmpOp::LtE => {
                    let Some(sequence_name) = len_anchor_name(cmp.left.as_ref(), ctx) else {
                        return Vec::new();
                    };
                    let Some(max_len) = literal_usize(&cmp.comparators[0]) else {
                        return Vec::new();
                    };
                    vec![SequenceGuard::MinLength {
                        sequence: sequence_name,
                        min_len: max_len.saturating_add(1),
                    }]
                }
                CmpOp::GtE => {
                    if let (Expr::Name(index_name), Some(sequence_name)) =
                        (cmp.left.as_ref(), len_anchor_name(&cmp.comparators[0], ctx))
                    {
                        return vec![SequenceGuard::IndexVarInRange {
                            sequence: sequence_name,
                            index_var: index_name.id.to_string(),
                            max_offset: 0,
                        }];
                    }
                    if let (Some(sequence_name), Expr::Name(index_name)) =
                        (len_anchor_name(cmp.left.as_ref(), ctx), &cmp.comparators[0])
                    {
                        return vec![SequenceGuard::IndexVarInRange {
                            sequence: sequence_name,
                            index_var: index_name.id.to_string(),
                            max_offset: 0,
                        }];
                    }
                    Vec::new()
                }
                CmpOp::NotIn => dict_contains_guard(cmp.left.as_ref(), &cmp.comparators[0]),
                CmpOp::Is => subscript_present_guard_from_non_none_compare(
                    cmp.left.as_ref(),
                    &cmp.comparators[0],
                ),
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}

fn subscript_present_guard_from_non_none_compare(left: &Expr, right: &Expr) -> Vec<SequenceGuard> {
    if matches!(right, Expr::NoneLiteral(_)) {
        return subscript_present_guard(left);
    }
    if matches!(left, Expr::NoneLiteral(_)) {
        return subscript_present_guard(right);
    }
    Vec::new()
}

fn subscript_present_guard(expr: &Expr) -> Vec<SequenceGuard> {
    let Expr::Subscript(subscript) = expr else {
        return Vec::new();
    };
    let Some(sequence) = sequence_guard_target_name(subscript.value.as_ref()) else {
        return Vec::new();
    };
    let Some(index_expr_debug) = key_guard_token(subscript.slice.as_ref()) else {
        return Vec::new();
    };
    vec![
        SequenceGuard::SubscriptAccessible {
            sequence: sequence.clone(),
            index_expr_debug: index_expr_debug.clone(),
        },
        SequenceGuard::SubscriptPresent {
            sequence,
            index_expr_debug,
            reference_stability: subscript_reference_stability(subscript.slice.as_ref()),
        },
    ]
}

fn subscript_reference_stability(index: &Expr) -> SubscriptReferenceStability {
    if literal_int(index).is_some_and(|value| value >= 0) {
        SubscriptReferenceStability::StableAcrossGrowth
    } else {
        SubscriptReferenceStability::MayChangeOnGrowth
    }
}

fn dict_contains_guard(key_expr: &Expr, haystack_expr: &Expr) -> Vec<SequenceGuard> {
    let Some(key_expr_debug) = key_guard_token(key_expr) else {
        return Vec::new();
    };
    if let Some(dict_name) = sequence_guard_target_name(haystack_expr) {
        return vec![SequenceGuard::DictContains {
            dict: dict_name,
            key_expr_debug: key_expr_debug.clone(),
        }];
    }

    let Expr::Call(call) = haystack_expr else {
        return Vec::new();
    };
    if !call.arguments.args.is_empty() || !call.arguments.keywords.is_empty() {
        return Vec::new();
    }
    let Expr::Attribute(attr) = call.func.as_ref() else {
        return Vec::new();
    };
    if attr.attr.as_str() != "keys" {
        return Vec::new();
    }
    let Some(dict_name) = sequence_guard_target_name(attr.value.as_ref()) else {
        return Vec::new();
    };
    vec![SequenceGuard::DictContains {
        dict: dict_name,
        key_expr_debug,
    }]
}

pub(in crate::lower) fn detect_range_sequence_guards(
    for_stmt: &StmtFor,
    target_name: &str,
    ctx: &LowerCtx,
) -> Vec<SequenceGuard> {
    let Some((sequence, max_offset)) = range_sequence_shape(for_stmt.iter.as_ref(), ctx) else {
        return Vec::new();
    };
    let mut guards = vec![
        SequenceGuard::IndexVarInRange {
            sequence: sequence.clone(),
            index_var: target_name.to_string(),
            max_offset,
        },
        SequenceGuard::IndexVarNonNegative {
            index_var: target_name.to_string(),
        },
    ];
    guards.extend(detect_sliding_window_pointer_guards(
        &for_stmt.body,
        target_name,
        &sequence,
        ctx,
    ));
    guards
}

fn range_sequence_shape(expr: &Expr, ctx: &LowerCtx) -> Option<(String, usize)> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Name(func_name) = call.func.as_ref() else {
        return None;
    };
    if func_name.id.as_str() != "range" {
        return None;
    }
    if call.arguments.args.len() == 1 {
        return len_anchor_with_max_offset(&call.arguments.args[0], ctx);
    }
    if call.arguments.args.len() == 2 {
        return len_anchor_with_max_offset(&call.arguments.args[1], ctx)
            .map(|(sequence, _)| (sequence, 0));
    }
    if call.arguments.args.len() == 3 {
        return reverse_len_range_shape(
            &call.arguments.args[0],
            &call.arguments.args[1],
            &call.arguments.args[2],
            ctx,
        );
    }
    None
}

fn reverse_len_range_shape(
    start: &Expr,
    stop: &Expr,
    step: &Expr,
    ctx: &LowerCtx,
) -> Option<(String, usize)> {
    if literal_int(stop) != Some(-1) || literal_int(step) != Some(-1) {
        return None;
    }
    len_anchor_with_max_offset(start, ctx)
}

fn len_anchor_with_max_offset(expr: &Expr, ctx: &LowerCtx) -> Option<(String, usize)> {
    if let Some(sequence_name) = len_anchor_name(expr, ctx) {
        return Some((sequence_name, 0));
    }
    let Expr::BinOp(binop) = expr else {
        return None;
    };
    if !matches!(binop.op, Operator::Sub) {
        return None;
    }
    let sequence_name = len_anchor_name(binop.left.as_ref(), ctx)?;
    let subtracted = literal_usize(binop.right.as_ref())?;
    Some((sequence_name, subtracted.saturating_sub(1)))
}

fn len_anchor_name(expr: &Expr, ctx: &LowerCtx) -> Option<String> {
    len_call_sequence_name(expr).or_else(|| match expr {
        Expr::Name(alias) => ctx.len_alias_sequence(alias.id.as_str()),
        _ => None,
    })
}

fn detect_sliding_window_pointer_guards(
    stmts: &[Stmt],
    right_var: &str,
    sequence: &str,
    ctx: &LowerCtx,
) -> Vec<SequenceGuard> {
    sequence_index_vars_in_stmts(stmts, sequence)
        .into_iter()
        .filter(|left_var| left_var.as_str() != right_var)
        .filter(|left_var| {
            ctx.is_zero_based_pointer(left_var)
                || matches!(
                    ctx.scope.effective_type(left_var),
                    Some(Type::LiteralInt(0))
                )
        })
        .filter(|left_var| loop_body_preserves_sliding_window_pointer(stmts, sequence, left_var))
        .flat_map(|left_var| {
            [
                SequenceGuard::IndexVarInRange {
                    sequence: sequence.to_string(),
                    index_var: left_var.clone(),
                    max_offset: 0,
                },
                SequenceGuard::IndexVarNonNegative {
                    index_var: left_var,
                },
            ]
        })
        .collect()
}

fn sequence_index_vars_in_stmts(stmts: &[Stmt], sequence: &str) -> Vec<String> {
    let mut visitor = SequenceIndexVarCollector::new(sequence);
    for stmt in stmts {
        visitor::walk_stmt(&mut visitor, stmt);
    }
    visitor.vars
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SlidingWindowPointerState {
    NotIncremented,
    MaybeIncremented,
}

fn loop_body_preserves_sliding_window_pointer(
    stmts: &[Stmt],
    sequence: &str,
    left_var: &str,
) -> bool {
    body_sliding_window_pointer_state(
        stmts,
        sequence,
        left_var,
        SlidingWindowPointerState::NotIncremented,
    )
    .is_some()
}

fn body_sliding_window_pointer_state(
    stmts: &[Stmt],
    sequence: &str,
    left_var: &str,
    mut state: SlidingWindowPointerState,
) -> Option<SlidingWindowPointerState> {
    for stmt in stmts {
        state = stmt_sliding_window_pointer_state(stmt, sequence, left_var, state)?;
    }
    Some(state)
}

fn stmt_sliding_window_pointer_state(
    stmt: &Stmt,
    sequence: &str,
    left_var: &str,
    state: SlidingWindowPointerState,
) -> Option<SlidingWindowPointerState> {
    if state == SlidingWindowPointerState::MaybeIncremented
        && stmt_contains_specific_index(stmt, sequence, left_var)
    {
        return None;
    }
    match stmt {
        Stmt::AugAssign(aug) => {
            if target_is_named_var(aug.target.as_ref(), left_var) {
                if aug_assign_is_single_step_increment(aug, left_var) {
                    Some(SlidingWindowPointerState::MaybeIncremented)
                } else {
                    None
                }
            } else {
                Some(state)
            }
        }
        Stmt::Assign(assign) => {
            if assign
                .targets
                .iter()
                .any(|target| target_is_named_var(target, left_var))
            {
                if assign_is_single_step_increment(assign, left_var) {
                    Some(SlidingWindowPointerState::MaybeIncremented)
                } else {
                    None
                }
            } else {
                Some(state)
            }
        }
        Stmt::AnnAssign(ann) => {
            if target_is_named_var(ann.target.as_ref(), left_var) {
                None
            } else {
                Some(state)
            }
        }
        Stmt::If(if_stmt) => {
            if state == SlidingWindowPointerState::MaybeIncremented
                && expr_contains_specific_index(if_stmt.test.as_ref(), sequence, left_var)
            {
                return None;
            }
            let mut branch_states = vec![body_sliding_window_pointer_state(
                &if_stmt.body,
                sequence,
                left_var,
                state,
            )?];
            let mut has_else = false;
            for clause in &if_stmt.elif_else_clauses {
                if let Some(test) = &clause.test {
                    if state == SlidingWindowPointerState::MaybeIncremented
                        && expr_contains_specific_index(test, sequence, left_var)
                    {
                        return None;
                    }
                } else {
                    has_else = true;
                }
                branch_states.push(body_sliding_window_pointer_state(
                    &clause.body,
                    sequence,
                    left_var,
                    state,
                )?);
            }
            if !has_else {
                branch_states.push(state);
            }
            if branch_states.contains(&SlidingWindowPointerState::MaybeIncremented) {
                Some(SlidingWindowPointerState::MaybeIncremented)
            } else {
                Some(SlidingWindowPointerState::NotIncremented)
            }
        }
        Stmt::While(_)
        | Stmt::For(_)
        | Stmt::Try(_)
        | Stmt::With(_)
        | Stmt::Match(_)
        | Stmt::FunctionDef(_)
        | Stmt::ClassDef(_) => Some(state),
        _ => Some(state),
    }
}

fn stmt_contains_specific_index(stmt: &Stmt, sequence: &str, index_var: &str) -> bool {
    let mut visitor = SpecificIndexUseVisitor::new(sequence, index_var);
    visitor::walk_stmt(&mut visitor, stmt);
    visitor.found
}

fn expr_contains_specific_index(expr: &Expr, sequence: &str, index_var: &str) -> bool {
    let mut visitor = SpecificIndexUseVisitor::new(sequence, index_var);
    visitor.visit_expr(expr);
    visitor.found
}

struct SpecificIndexUseVisitor<'a> {
    sequence: &'a str,
    index_var: &'a str,
    found: bool,
}

impl<'a> SpecificIndexUseVisitor<'a> {
    fn new(sequence: &'a str, index_var: &'a str) -> Self {
        Self {
            sequence,
            index_var,
            found: false,
        }
    }
}

impl<'a> Visitor<'a> for SpecificIndexUseVisitor<'a> {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if self.found {
            return;
        }
        if let Expr::Subscript(sub) = expr {
            if let (Expr::Name(sequence_name), Expr::Name(index_name)) =
                (sub.value.as_ref(), sub.slice.as_ref())
            {
                if sequence_name.id.as_str() == self.sequence
                    && index_name.id.as_str() == self.index_var
                {
                    self.found = true;
                    return;
                }
            }
        }
        visitor::walk_expr(self, expr);
    }
}

struct SequenceIndexVarCollector<'a> {
    sequence: &'a str,
    vars: Vec<String>,
}

impl<'a> SequenceIndexVarCollector<'a> {
    fn new(sequence: &'a str) -> Self {
        Self {
            sequence,
            vars: Vec::new(),
        }
    }
}

impl<'a> Visitor<'a> for SequenceIndexVarCollector<'a> {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Subscript(sub) = expr {
            if let (Expr::Name(sequence_name), Expr::Name(index_name)) =
                (sub.value.as_ref(), sub.slice.as_ref())
            {
                if sequence_name.id.as_str() == self.sequence
                    && !self.vars.iter().any(|var| var == index_name.id.as_str())
                {
                    self.vars.push(index_name.id.to_string());
                }
            }
        }
        visitor::walk_expr(self, expr);
    }
}

fn target_is_named_var(target: &Expr, var_name: &str) -> bool {
    matches!(target, Expr::Name(name) if name.id.as_str() == var_name)
}

fn aug_assign_is_single_step_increment(aug: &StmtAugAssign, var_name: &str) -> bool {
    matches!(aug.op, Operator::Add)
        && target_is_named_var(aug.target.as_ref(), var_name)
        && literal_int(aug.value.as_ref()) == Some(1)
}

fn assign_is_single_step_increment(assign: &StmtAssign, var_name: &str) -> bool {
    if assign.targets.len() != 1 || !target_is_named_var(&assign.targets[0], var_name) {
        return false;
    }
    let Expr::BinOp(binop) = assign.value.as_ref() else {
        return false;
    };
    matches!(binop.op, Operator::Add)
        && literal_int(binop.right.as_ref()) == Some(1)
        && matches!(binop.left.as_ref(), Expr::Name(name) if name.id.as_str() == var_name)
}

fn len_call_sequence_name(expr: &Expr) -> Option<String> {
    let Expr::Call(call) = expr else {
        return None;
    };
    match call.func.as_ref() {
        Expr::Name(func_name) => {
            if func_name.id.as_str() != "len" || call.arguments.args.len() != 1 {
                return None;
            }
            sequence_guard_target_name(&call.arguments.args[0])
        }
        Expr::Attribute(attr) => {
            if attr.attr.as_str() != "len" || !call.arguments.args.is_empty() {
                return None;
            }
            sequence_guard_target_name(attr.value.as_ref())
        }
        _ => None,
    }
}

fn sequence_guard_target_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Name(name) => Some(name.id.to_string()),
        Expr::Attribute(attr) => {
            let base = sequence_guard_target_name(attr.value.as_ref())?;
            Some(format!("{base}.{}", attr.attr))
        }
        _ => None,
    }
}

fn literal_usize(expr: &Expr) -> Option<usize> {
    let Expr::NumberLiteral(num) = expr else {
        return None;
    };
    let Number::Int(value) = &num.value else {
        return None;
    };
    value.as_i64().and_then(|value| usize::try_from(value).ok())
}

fn literal_int(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::NumberLiteral(num) => {
            let Number::Int(value) = &num.value else {
                return None;
            };
            value.as_i64()
        }
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::USub) => {
            literal_int(unary.operand.as_ref()).map(|value| -value)
        }
        _ => None,
    }
}

fn index_var_with_nonnegative_offset(expr: &Expr) -> Option<(String, usize)> {
    match expr {
        Expr::Name(name) => Some((name.id.to_string(), 0)),
        Expr::BinOp(binop) if matches!(binop.op, Operator::Add) => {
            let Expr::Name(name) = binop.left.as_ref() else {
                return None;
            };
            let offset = literal_usize(binop.right.as_ref())?;
            Some((name.id.to_string(), offset))
        }
        _ => None,
    }
}

fn detect_two_pointer_while_guards(while_stmt: &StmtWhile, ctx: &LowerCtx) -> Vec<SequenceGuard> {
    let Expr::Compare(cmp) = while_stmt.test.as_ref() else {
        return Vec::new();
    };
    if cmp.ops.len() != 1 || cmp.comparators.len() != 1 || !matches!(&cmp.ops[0], CmpOp::Lt) {
        return Vec::new();
    }
    let (Expr::Name(left_name), Expr::Name(right_name)) = (cmp.left.as_ref(), &cmp.comparators[0])
    else {
        return Vec::new();
    };
    if !loop_body_preserves_two_pointer_step(
        &while_stmt.body,
        left_name.id.as_str(),
        right_name.id.as_str(),
    ) {
        return Vec::new();
    }
    let Some(sequence) =
        ctx.same_sequence_two_pointer_loop(left_name.id.as_str(), right_name.id.as_str())
    else {
        return Vec::new();
    };
    vec![
        SequenceGuard::IndexVarInRange {
            sequence: sequence.clone(),
            index_var: left_name.id.to_string(),
            max_offset: 0,
        },
        SequenceGuard::IndexVarInRange {
            sequence,
            index_var: right_name.id.to_string(),
            max_offset: 0,
        },
        SequenceGuard::IndexVarNonNegative {
            index_var: left_name.id.to_string(),
        },
        SequenceGuard::IndexVarNonNegative {
            index_var: right_name.id.to_string(),
        },
    ]
}

fn loop_body_preserves_two_pointer_step(stmts: &[Stmt], left_var: &str, right_var: &str) -> bool {
    stmts
        .iter()
        .all(|stmt| stmt_preserves_two_pointer_step(stmt, left_var, right_var))
}

fn stmt_preserves_two_pointer_step(stmt: &Stmt, left_var: &str, right_var: &str) -> bool {
    match stmt {
        Stmt::AugAssign(aug) => aug_assign_is_safe_two_pointer_step(aug, left_var, right_var),
        Stmt::Assign(assign) => assign_is_safe_two_pointer_step(assign, left_var, right_var),
        Stmt::If(if_stmt) => {
            loop_body_preserves_two_pointer_step(&if_stmt.body, left_var, right_var)
                && if_stmt.elif_else_clauses.iter().all(|clause| {
                    loop_body_preserves_two_pointer_step(&clause.body, left_var, right_var)
                })
        }
        Stmt::While(_)
        | Stmt::For(_)
        | Stmt::Try(_)
        | Stmt::With(_)
        | Stmt::Match(_)
        | Stmt::FunctionDef(_)
        | Stmt::ClassDef(_) => !stmt_assigns_pointer_var(stmt, left_var, right_var),
        _ => !stmt_assigns_pointer_var(stmt, left_var, right_var),
    }
}

fn stmt_assigns_pointer_var(stmt: &Stmt, left_var: &str, right_var: &str) -> bool {
    match stmt {
        Stmt::AnnAssign(ann) => target_is_pointer_name(ann.target.as_ref(), left_var, right_var),
        Stmt::Assign(assign) => assign.targets.iter().any(|target| {
            target_is_pointer_name(target, left_var, right_var)
                || tuple_target_contains_pointer(target, left_var, right_var)
        }),
        Stmt::AugAssign(aug) => target_is_pointer_name(aug.target.as_ref(), left_var, right_var),
        _ => false,
    }
}

fn target_is_pointer_name(target: &Expr, left_var: &str, right_var: &str) -> bool {
    matches!(
        target,
        Expr::Name(name) if name.id.as_str() == left_var || name.id.as_str() == right_var
    )
}

fn tuple_target_contains_pointer(target: &Expr, left_var: &str, right_var: &str) -> bool {
    let Expr::Tuple(tuple) = target else {
        return false;
    };
    tuple
        .elts
        .iter()
        .any(|elt| target_is_pointer_name(elt, left_var, right_var))
}

fn aug_assign_is_safe_two_pointer_step(
    aug: &StmtAugAssign,
    left_var: &str,
    right_var: &str,
) -> bool {
    let Expr::Name(name) = aug.target.as_ref() else {
        return !target_is_pointer_name(aug.target.as_ref(), left_var, right_var);
    };
    if name.id.as_str() == left_var {
        matches!(aug.op, Operator::Add) && literal_usize(aug.value.as_ref()) == Some(1)
    } else if name.id.as_str() == right_var {
        matches!(aug.op, Operator::Sub) && literal_usize(aug.value.as_ref()) == Some(1)
    } else {
        true
    }
}

fn assign_is_safe_two_pointer_step(assign: &StmtAssign, left_var: &str, right_var: &str) -> bool {
    if assign.targets.len() != 1 {
        return !assign.targets.iter().any(|target| {
            target_is_pointer_name(target, left_var, right_var)
                || tuple_target_contains_pointer(target, left_var, right_var)
        });
    }
    let Expr::Name(name) = &assign.targets[0] else {
        return !target_is_pointer_name(&assign.targets[0], left_var, right_var);
    };
    if name.id.as_str() == left_var {
        matches_name_plus_one(&assign.value, left_var)
    } else if name.id.as_str() == right_var {
        matches_name_minus_one(&assign.value, right_var)
    } else {
        true
    }
}

fn matches_name_plus_one(expr: &Expr, name: &str) -> bool {
    let Expr::BinOp(binop) = expr else {
        return false;
    };
    matches!(binop.op, Operator::Add)
        && matches!(binop.left.as_ref(), Expr::Name(var) if var.id.as_str() == name)
        && literal_usize(binop.right.as_ref()) == Some(1)
}

fn matches_name_minus_one(expr: &Expr, name: &str) -> bool {
    let Expr::BinOp(binop) = expr else {
        return false;
    };
    matches!(binop.op, Operator::Sub)
        && matches!(binop.left.as_ref(), Expr::Name(var) if var.id.as_str() == name)
        && literal_usize(binop.right.as_ref()) == Some(1)
}
