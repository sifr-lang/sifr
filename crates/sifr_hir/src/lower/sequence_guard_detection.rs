use super::sequence_guards::SequenceGuard;
use super::LowerCtx;
use sifr_python_ast::{
    BoolOp, CmpOp, Expr, Number, Operator, Stmt, StmtAssign, StmtAugAssign, StmtFor, StmtWhile,
    UnaryOp,
};

pub(super) fn detect_while_sequence_guards(
    while_stmt: &StmtWhile,
    ctx: &LowerCtx,
) -> Vec<SequenceGuard> {
    let mut guards = detect_true_sequence_guards(&while_stmt.test, ctx);
    guards.extend(detect_two_pointer_while_guards(while_stmt, ctx));
    guards
}

pub(super) fn detect_true_sequence_guards(expr: &Expr, ctx: &LowerCtx) -> Vec<SequenceGuard> {
    match expr {
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::And) => boolop
            .values
            .iter()
            .flat_map(|value| detect_true_sequence_guards(value, ctx))
            .collect(),
        Expr::Name(name) => vec![SequenceGuard::MinLength {
            sequence: name.id.clone(),
            min_len: 1,
        }],
        Expr::Compare(cmp) if cmp.ops.len() == 1 && cmp.comparators.len() == 1 => {
            match &cmp.ops[0] {
                CmpOp::Lt => {
                    if let (Expr::Name(index_name), Some(sequence_name)) = (
                        cmp.left.as_ref(),
                        len_call_sequence_name(&cmp.comparators[0]),
                    ) {
                        return vec![SequenceGuard::IndexVarInRange {
                            sequence: sequence_name,
                            index_var: index_name.id.clone(),
                        }];
                    }
                    Vec::new()
                }
                CmpOp::Eq => {
                    if let (Some(sequence_name), Some(len_value)) = (
                        len_call_sequence_name(cmp.left.as_ref()),
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
                        len_call_sequence_name(cmp.left.as_ref()),
                        literal_usize(&cmp.comparators[0]),
                    ) {
                        return vec![SequenceGuard::MinLength {
                            sequence: sequence_name,
                            min_len: len_value + 1,
                        }];
                    }
                    Vec::new()
                }
                _ => Vec::new(),
            }
        }
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::Not) => {
            detect_false_exit_sequence_guards(&unary.operand, ctx)
        }
        _ => Vec::new(),
    }
}

pub(super) fn detect_false_exit_sequence_guards(expr: &Expr, ctx: &LowerCtx) -> Vec<SequenceGuard> {
    match expr {
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::Not) => {
            detect_true_sequence_guards(&unary.operand, ctx)
        }
        Expr::Name(name) => vec![SequenceGuard::MinLength {
            sequence: name.id.clone(),
            min_len: 1,
        }],
        Expr::Compare(cmp) if cmp.ops.len() == 1 && cmp.comparators.len() == 1 => {
            if !matches!(&cmp.ops[0], CmpOp::Eq) {
                return Vec::new();
            }
            let Some(sequence_name) = len_call_sequence_name(cmp.left.as_ref()) else {
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
        _ => Vec::new(),
    }
}

pub(super) fn detect_range_sequence_guards(
    for_stmt: &StmtFor,
    target_name: &str,
) -> Vec<SequenceGuard> {
    let Expr::Call(call) = for_stmt.iter.as_ref() else {
        return Vec::new();
    };
    let Expr::Name(func_name) = call.func.as_ref() else {
        return Vec::new();
    };
    if func_name.id.as_str() != "range" {
        return Vec::new();
    }
    let mut sequence_name = None;
    for arg in &call.arguments.args {
        if let Some(found) = len_call_sequence_name(arg) {
            sequence_name = Some(found);
            break;
        }
        if let Expr::BinOp(binop) = arg {
            if let Some(found) = len_call_sequence_name(binop.left.as_ref()) {
                sequence_name = Some(found);
                break;
            }
        }
    }
    sequence_name
        .map(|sequence| {
            vec![SequenceGuard::IndexVarInRange {
                sequence,
                index_var: target_name.to_string(),
            }]
        })
        .unwrap_or_default()
}

fn len_call_sequence_name(expr: &Expr) -> Option<String> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Name(func_name) = call.func.as_ref() else {
        return None;
    };
    if func_name.id.as_str() != "len" || call.arguments.args.len() != 1 {
        return None;
    }
    let Expr::Name(sequence_name) = &call.arguments.args[0] else {
        return None;
    };
    Some(sequence_name.id.clone())
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
            index_var: left_name.id.clone(),
        },
        SequenceGuard::IndexVarInRange {
            sequence,
            index_var: right_name.id.clone(),
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
