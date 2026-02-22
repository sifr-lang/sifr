//! Statement lowering scaffolds for the IR migration.

use crate::{try_lower_leaf_expr, CodegenError, RustExpr, RustLiteral, RustParam, RustStmt, RustType};
use sifr_hir::{HirExpr, HirStmt};
use sifr_type_system::Type;
use std::collections::HashSet;

pub fn lower_stmt_raw(raw: &str) -> Result<Vec<RustStmt>, CodegenError> {
    Ok(vec![RustStmt::RawCode(raw.to_string())])
}

/// Lowers an expression statement when the expression is a leaf
/// supported by `try_lower_leaf_expr`.
pub fn try_lower_expr_stmt(expr: &HirExpr) -> Option<Vec<RustStmt>> {
    try_lower_leaf_expr(expr).map(|lowered_expr| vec![RustStmt::Expr(lowered_expr)])
}

/// Lowers statement variants that are context-light and safe to convert
/// without touching complex emitter state.
pub fn try_lower_simple_stmt(
    stmt: &HirStmt,
    in_loop_with_else: bool,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
) -> Option<Vec<RustStmt>> {
    match stmt {
        HirStmt::Expr { expr } => try_lower_expr_stmt(expr),
        HirStmt::Let { name, ty, value, .. } if can_lower_simple_let(ty, value) => {
            Some(vec![RustStmt::Let {
                mutable: mutated_vars.contains(name),
                name: name.clone(),
                ty: Some(crate::sifr_type_to_rust_type(ty)),
                value: try_lower_leaf_expr(value)?,
            }])
        }
        HirStmt::Assign { name, value } if can_lower_simple_assign(value, borrowed_params) => {
            Some(vec![RustStmt::Assign {
                target: crate::RustExpr::Ident(name.clone()),
                value: try_lower_leaf_expr(value)?,
            }])
        }
        HirStmt::AugAssign { name, op, value } if can_lower_simple_aug_assign(op, value) => {
            Some(vec![RustStmt::AugAssign {
                target: crate::RustExpr::Ident(name.clone()),
                op: normalize_aug_assign_op(op).to_string(),
                value: try_lower_leaf_expr(value)?,
            }])
        }
        HirStmt::Assert { test, msg } => Some(vec![try_lower_simple_assert_stmt(
            test,
            msg.as_ref(),
        )?]),
        HirStmt::Raise { value } => Some(vec![try_lower_simple_raise_stmt(value)?]),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body: maybe_else_body,
        } => Some(vec![try_lower_simple_if_stmt(
            condition,
            then_body,
            elif_clauses,
            maybe_else_body.as_deref(),
            in_loop_with_else,
            mutated_vars,
            borrowed_params,
        )?]),
        HirStmt::While {
            condition,
            body,
            else_body: None,
        } => Some(vec![RustStmt::While {
            cond: try_lower_leaf_expr(condition)?,
            // Entering a nested while without else resets loop-else break marker context.
            body: try_lower_simple_stmt_block(
                body,
                false,
                mutated_vars,
                borrowed_params,
            )?,
        }]),
        HirStmt::While {
            condition,
            body,
            else_body: Some(else_body),
        } => Some(vec![
            RustStmt::Let {
                mutable: true,
                name: "_broke".to_string(),
                ty: None,
                value: RustExpr::Literal(RustLiteral::Bool(false)),
            },
            RustStmt::While {
                cond: try_lower_leaf_expr(condition)?,
                // Breaks in the loop body should mark this loop's `_broke`.
                body: try_lower_simple_stmt_block(
                    body,
                    true,
                    mutated_vars,
                    borrowed_params,
                )?,
            },
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::Ident("_broke".to_string())),
                },
                // Else body executes outside this loop scope. Preserve enclosing
                // loop-else context for any break/continue lowering there.
                then_body: try_lower_simple_stmt_block(
                    else_body,
                    in_loop_with_else,
                    mutated_vars,
                    borrowed_params,
                )?,
                else_body: None,
            },
        ]),
        HirStmt::For {
            target,
            iter,
            body,
            else_body: None,
            ..
        } if !target.contains(',') => Some(vec![RustStmt::For {
            var: target.clone(),
            iter: try_lower_leaf_expr(iter)?,
            // Entering a nested for without else resets loop-else break marker context.
            body: try_lower_simple_stmt_block(
                body,
                false,
                mutated_vars,
                borrowed_params,
            )?,
        }]),
        HirStmt::For {
            target,
            iter,
            body,
            else_body: Some(else_body),
            ..
        } if !target.contains(',') => Some(vec![
            RustStmt::Let {
                mutable: true,
                name: "_broke".to_string(),
                ty: None,
                value: RustExpr::Literal(RustLiteral::Bool(false)),
            },
            RustStmt::For {
                var: target.clone(),
                iter: try_lower_leaf_expr(iter)?,
                // Breaks in the loop body should mark this loop's `_broke`.
                body: try_lower_simple_stmt_block(
                    body,
                    true,
                    mutated_vars,
                    borrowed_params,
                )?,
            },
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::Ident("_broke".to_string())),
                },
                // Else body executes outside this loop scope. Preserve enclosing
                // loop-else context for any break/continue lowering there.
                then_body: try_lower_simple_stmt_block(
                    else_body,
                    in_loop_with_else,
                    mutated_vars,
                    borrowed_params,
                )?,
                else_body: None,
            },
        ]),
        HirStmt::Pass => Some(vec![]),
        HirStmt::Continue => Some(vec![RustStmt::Continue]),
        HirStmt::Break => {
            if in_loop_with_else {
                Some(vec![
                    RustStmt::Assign {
                        target: crate::RustExpr::Ident("_broke".to_string()),
                        value: crate::RustExpr::Literal(crate::RustLiteral::Bool(true)),
                    },
                    RustStmt::Break,
                ])
            } else {
                Some(vec![RustStmt::Break])
            }
        }
        _ => None,
    }
}

fn try_lower_simple_stmt_block(
    stmts: &[HirStmt],
    in_loop_with_else: bool,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
) -> Option<Vec<RustStmt>> {
    let mut lowered = Vec::new();
    for stmt in stmts {
        lowered.extend(try_lower_simple_stmt(
            stmt,
            in_loop_with_else,
            mutated_vars,
            borrowed_params,
        )?);
    }
    Some(lowered)
}

fn try_lower_simple_if_stmt(
    condition: &HirExpr,
    then_body: &[HirStmt],
    elif_clauses: &[(HirExpr, Vec<HirStmt>)],
    maybe_else_body: Option<&[HirStmt]>,
    in_loop_with_else: bool,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
) -> Option<RustStmt> {
    let mut nested_else = if let Some(else_body) = maybe_else_body {
        Some(try_lower_simple_stmt_block(
            else_body,
            in_loop_with_else,
            mutated_vars,
            borrowed_params,
        )?)
    } else {
        None
    };

    for (elif_cond, elif_body) in elif_clauses.iter().rev() {
        nested_else = Some(vec![RustStmt::If {
            cond: try_lower_leaf_expr(elif_cond)?,
            then_body: try_lower_simple_stmt_block(
                elif_body,
                in_loop_with_else,
                mutated_vars,
                borrowed_params,
            )?,
            else_body: nested_else,
        }]);
    }

    Some(RustStmt::If {
        cond: try_lower_leaf_expr(condition)?,
        then_body: try_lower_simple_stmt_block(
            then_body,
            in_loop_with_else,
            mutated_vars,
            borrowed_params,
        )?,
        else_body: nested_else,
    })
}

fn can_lower_simple_let(ty: &Type, value: &HirExpr) -> bool {
    if ty != value.ty() {
        return false;
    }
    if !matches!(
        ty,
        Type::Int | Type::Float | Type::Bool | Type::Str | Type::Enum { .. }
    ) {
        return false;
    }
    try_lower_leaf_expr(value).is_some()
}

fn can_lower_simple_assign(value: &HirExpr, borrowed_params: &HashSet<String>) -> bool {
    // Preserve legacy behavior where TypeVar assignment from borrowed params appends `.clone()`.
    if matches!(value.ty(), Type::TypeVar(_))
        && matches!(value, HirExpr::Name { name, .. } if borrowed_params.contains(name))
    {
        return false;
    }
    try_lower_leaf_expr(value).is_some()
}

fn can_lower_simple_aug_assign(op: &str, value: &HirExpr) -> bool {
    matches!(op, "-=" | "*=" | "/=" | "//=" | "%=") && try_lower_leaf_expr(value).is_some()
}

fn normalize_aug_assign_op(op: &str) -> &str {
    if op == "//=" {
        return "/";
    }
    op.strip_suffix('=').unwrap_or(op)
}

fn try_lower_simple_raise_stmt(value: &HirExpr) -> Option<RustStmt> {
    Some(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![try_lower_leaf_expr(value)?],
    })))
}

fn try_lower_simple_assert_stmt(test: &HirExpr, msg: Option<&HirExpr>) -> Option<RustStmt> {
    let lowered_msg = if let Some(msg_expr) = msg {
        Some(try_lower_assert_msg_expr(msg_expr)?)
    } else {
        None
    };
    Some(RustStmt::Assert {
        cond: try_lower_leaf_expr(test)?,
        msg: lowered_msg,
    })
}

fn try_lower_assert_msg_expr(msg: &HirExpr) -> Option<RustExpr> {
    if crate::helpers::is_option_type(msg.ty()) {
        return try_lower_option_display_expr(msg);
    }
    try_lower_leaf_expr(msg)
}

fn try_lower_option_display_expr(msg: &HirExpr) -> Option<RustExpr> {
    let receiver = match msg {
        HirExpr::Name { name, .. } => RustExpr::Ident(name.clone()),
        _ => return None,
    };

    Some(RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "map_or".to_string(),
        args: vec![
            RustExpr::Literal(RustLiteral::Str("None".to_string())),
            RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "_v".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "{}".to_string(),
                    args: vec![RustExpr::Ident("_v".to_string())],
                }),
                is_move: false,
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_raw_stmt_placeholder() {
        let stmts = lower_stmt_raw("let x = 1;").expect("placeholder lower should succeed");
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], RustStmt::RawCode(_)));
    }

    #[test]
    fn lowers_leaf_expression_statement() {
        let stmts = try_lower_expr_stmt(&HirExpr::IntLiteral(1)).expect("leaf stmt lowered");
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], RustStmt::Expr(_)));
    }

    #[test]
    fn lowers_pass_and_continue_and_break() {
        let pass = try_lower_simple_stmt(&HirStmt::Pass, false, &HashSet::new(), &HashSet::new())
            .expect("pass lowered");
        assert!(pass.is_empty());

        let cont = try_lower_simple_stmt(
            &HirStmt::Continue,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("continue lowered");
        assert!(matches!(cont[0], RustStmt::Continue));

        let brk = try_lower_simple_stmt(&HirStmt::Break, true, &HashSet::new(), &HashSet::new())
            .expect("break lowered");
        assert_eq!(brk.len(), 2);
        assert!(matches!(brk[0], RustStmt::Assign { .. }));
        assert!(matches!(brk[1], RustStmt::Break));
    }

    #[test]
    fn lowers_simple_let_and_assign() {
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Int,
            value: HirExpr::IntLiteral(1),
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::from(["x".to_string()]),
            &HashSet::new(),
        )
        .expect("let lowered");
        assert!(matches!(lowered[0], RustStmt::Let { mutable: true, .. }));

        let assign_stmt = HirStmt::Assign {
            name: "x".to_string(),
            value: HirExpr::IntLiteral(2),
        };
        let lowered = try_lower_simple_stmt(
            &assign_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assign lowered");
        assert!(matches!(lowered[0], RustStmt::Assign { .. }));
    }

    #[test]
    fn lowers_simple_augassign_for_supported_ops() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "-=".to_string(),
            value: HirExpr::IntLiteral(2),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("augassign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Ident(ref name),
                op: ref lowered_op,
                ..
            } if name == "x" && lowered_op == "-"
        ));
    }

    #[test]
    fn does_not_lower_augassign_plus_equal() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "+=".to_string(),
            value: HirExpr::IntLiteral(1),
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_augassign_floor_div_equal() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "//=".to_string(),
            value: HirExpr::IntLiteral(2),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("floor-div augassign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Ident(ref name),
                op: ref lowered_op,
                ..
            } if name == "x" && lowered_op == "/"
        ));
    }

    #[test]
    fn does_not_lower_augassign_power_equal() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "**=".to_string(),
            value: HirExpr::IntLiteral(3),
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_raise_with_leaf_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::IntLiteral(7),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("raise lowered");

        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
                assert!(matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Err".to_string()]));
            }
            _ => panic!("expected return Err(...)"),
        }
    }

    #[test]
    fn does_not_lower_raise_with_non_leaf_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::Name {
                name: "e".to_string(),
                ty: Type::Int,
            },
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_assert_without_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: None,
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert { msg: None, .. }
        ));
    }

    #[test]
    fn lowers_simple_assert_with_leaf_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::StringLiteral("boom".to_string())),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert with msg lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                msg: Some(RustExpr::Literal(RustLiteral::Str(_))),
                ..
            }
        ));
    }

    #[test]
    fn does_not_lower_assert_with_non_leaf_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Name {
                name: "ok".to_string(),
                ty: Type::Bool,
            },
            msg: None,
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_assert_with_option_name_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::Name {
                name: "msg".to_string(),
                ty: Type::Union(vec![Type::Str, Type::None]),
            }),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert with option msg lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                msg: Some(RustExpr::MethodCall { ref method, .. }),
                ..
            } if method == "map_or"
        ));
    }

    #[test]
    fn does_not_lower_assert_with_non_leaf_option_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::Call {
                func: "maybe_msg".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Str, Type::None]),
            }),
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_if_without_elif() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(1),
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(0),
            }]),
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(lowered[0], RustStmt::If { .. }));
    }

    #[test]
    fn lowers_simple_if_with_elif() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(1),
            }],
            elif_clauses: vec![(
                HirExpr::BoolLiteral(false),
                vec![HirStmt::Expr {
                    expr: HirExpr::IntLiteral(2),
                }],
            )],
            else_body: Some(vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(3),
            }]),
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if with elif lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::If { else_body, .. } => {
                assert!(else_body.is_some());
                if let Some(else_body) = else_body {
                    assert_eq!(else_body.len(), 1);
                    assert!(matches!(else_body[0], RustStmt::If { .. }));
                }
            }
            _ => panic!("expected if stmt"),
        }
    }

    #[test]
    fn does_not_lower_if_with_non_leaf_elif_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![(
                HirExpr::Name {
                    name: "flag".to_string(),
                    ty: Type::Bool,
                },
                vec![HirStmt::Pass],
            )],
            else_body: None,
        };

        assert!(
            try_lower_simple_stmt(
                &if_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_while_without_else() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![HirStmt::Break],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            true, // outer context has else, inner while should not inherit it
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::While { body, .. } => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], RustStmt::Break));
            }
            _ => panic!("expected RustStmt::While"),
        }
    }

    #[test]
    fn lowers_simple_while_with_else() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Pass]),
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while with else lowered");
        assert_eq!(lowered.len(), 3);
        assert!(matches!(lowered[0], RustStmt::Let { .. }));
        assert!(matches!(lowered[1], RustStmt::While { .. }));
        assert!(matches!(lowered[2], RustStmt::If { .. }));
    }

    #[test]
    fn lowers_simple_for_without_else() {
        let for_stmt = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Break],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &for_stmt,
            true, // outer loop-else context should not leak into inner loop body
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::For { body, .. } => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], RustStmt::Break));
            }
            _ => panic!("expected RustStmt::For"),
        }
    }

    #[test]
    fn lowers_simple_for_with_else() {
        let for_with_else = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Pass]),
        };
        let lowered = try_lower_simple_stmt(
            &for_with_else,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for with else lowered");
        assert_eq!(lowered.len(), 3);
        assert!(matches!(lowered[0], RustStmt::Let { .. }));
        assert!(matches!(lowered[1], RustStmt::For { .. }));
        assert!(matches!(lowered[2], RustStmt::If { .. }));
    }

    #[test]
    fn does_not_lower_for_with_tuple_target() {
        let for_tuple_target = HirStmt::For {
            target: "i,v".to_string(),
            target_ty: Type::Tuple(vec![Type::Int, Type::Int]),
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };
        assert!(
            try_lower_simple_stmt(
                &for_tuple_target,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_for_else_with_broke_marker_in_loop_body() {
        let for_stmt = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Break],
            else_body: Some(vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(1),
            }]),
        };

        let lowered = try_lower_simple_stmt(
            &for_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for else lowered");

        match &lowered[1] {
            RustStmt::For { body, .. } => {
                assert_eq!(body.len(), 2);
                assert!(matches!(body[0], RustStmt::Assign { .. }));
                assert!(matches!(body[1], RustStmt::Break));
            }
            _ => panic!("expected for stmt"),
        }
    }

    #[test]
    fn for_else_body_break_uses_outer_loop_else_context() {
        let for_stmt = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Break]),
        };

        let lowered = try_lower_simple_stmt(
            &for_stmt,
            true,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for else lowered");

        match &lowered[2] {
            RustStmt::If { then_body, .. } => {
                assert_eq!(then_body.len(), 2);
                assert!(matches!(then_body[0], RustStmt::Assign { .. }));
                assert!(matches!(then_body[1], RustStmt::Break));
            }
            _ => panic!("expected if stmt"),
        }
    }

    #[test]
    fn lowers_while_else_with_broke_marker_in_loop_body() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![HirStmt::Break],
            else_body: Some(vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(1),
            }]),
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while else lowered");

        match &lowered[1] {
            RustStmt::While { body, .. } => {
                assert_eq!(body.len(), 2);
                assert!(matches!(body[0], RustStmt::Assign { .. }));
                assert!(matches!(body[1], RustStmt::Break));
            }
            _ => panic!("expected while stmt"),
        }
    }

    #[test]
    fn while_else_body_break_uses_outer_loop_else_context() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::BoolLiteral(false),
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Break]),
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            true,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while else lowered");

        match &lowered[2] {
            RustStmt::If { then_body, .. } => {
                assert_eq!(then_body.len(), 2);
                assert!(matches!(then_body[0], RustStmt::Assign { .. }));
                assert!(matches!(then_body[1], RustStmt::Break));
            }
            _ => panic!("expected if stmt"),
        }
    }
}
