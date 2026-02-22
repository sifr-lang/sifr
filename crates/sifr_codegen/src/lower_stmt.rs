//! Statement lowering scaffolds for the IR migration.

use crate::{try_lower_leaf_expr, CodegenError, RustStmt};
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
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body: maybe_else_body,
        } if elif_clauses.is_empty() => Some(vec![RustStmt::If {
            cond: try_lower_leaf_expr(condition)?,
            then_body: try_lower_simple_stmt_block(
                then_body,
                in_loop_with_else,
                mutated_vars,
                borrowed_params,
            )?,
            else_body: if let Some(body) = maybe_else_body {
                Some(try_lower_simple_stmt_block(
                    body,
                    in_loop_with_else,
                    mutated_vars,
                    borrowed_params,
                )?)
            } else {
                None
            },
        }]),
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
    fn does_not_lower_if_with_elif() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![(HirExpr::BoolLiteral(false), vec![HirStmt::Pass])],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        );
        assert!(lowered.is_none());
    }
}
