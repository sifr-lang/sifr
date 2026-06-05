use crate::HirExpr;
use num_bigint::BigInt;
use ruff_text_size::Ranged;
use sifr_python_ast::{Expr, Stmt, UnaryOp};
use sifr_type_system::Type;

use super::simple_expr::{
    integer_binop_source, lower_integer_const_expr_simple, negate_simple_expr,
};
use super::typing_and_functions::resolve_annotation_expr;
use super::{fixed_width_fitting, LowerCtx};

pub(in crate::lower) fn collect_module_constants(
    stmts: &[Stmt],
    ctx: &mut LowerCtx,
) -> Vec<(String, Type, HirExpr)> {
    let mut constants = Vec::new();
    for stmt in stmts {
        collect_annotated_constant(stmt, ctx, &mut constants);
        collect_bare_constant(stmt, ctx, &mut constants);
    }
    constants
}

fn collect_annotated_constant(
    stmt: &Stmt,
    ctx: &mut LowerCtx,
    constants: &mut Vec<(String, Type, HirExpr)>,
) {
    let Stmt::AnnAssign(ann) = stmt else {
        return;
    };
    let Expr::Name(name) = ann.target.as_ref() else {
        return;
    };
    let Some(ref value_expr) = ann.value else {
        return;
    };
    let Some(mut hir_value) = lower_module_integer_const_expr(value_expr, ctx) else {
        return;
    };

    let var_name = name.id.to_string();
    let ty = resolve_annotation_expr(&ann.annotation, ctx);
    let error_count_before_initializer = ctx.error_count();
    if let Some(folded_value) = fixed_width_fitting::validate_annotated_constant_initializer(
        ctx,
        &ty,
        &hir_value,
        value_expr.range(),
    ) {
        hir_value = folded_value;
    }
    if ctx.error_count() == error_count_before_initializer {
        let remembered_value = fixed_width_fitting::remember_module_const_integer(
            ctx,
            &var_name,
            &hir_value,
            value_expr.range(),
        );
        if let Some(folded) =
            oversized_int_module_constant_literal_for_codegen(&ty, remembered_value.as_ref())
        {
            hir_value = folded;
        }
    }
    ctx.scope.define(var_name.clone(), ty.clone());
    constants.push((var_name, ty, hir_value));
}

fn collect_bare_constant(
    stmt: &Stmt,
    ctx: &mut LowerCtx,
    constants: &mut Vec<(String, Type, HirExpr)>,
) {
    let Stmt::Assign(assign) = stmt else {
        return;
    };
    if assign.targets.len() != 1 {
        return;
    }
    let Expr::Name(name) = &assign.targets[0] else {
        return;
    };
    let var_name = name.id.to_string();
    if ctx.type_vars.contains(&var_name) {
        return;
    }
    let Some(mut hir_value) = lower_module_integer_const_expr(&assign.value, ctx) else {
        return;
    };

    let ty = hir_value.ty().clone();
    let remembered_value = fixed_width_fitting::remember_module_const_integer(
        ctx,
        &var_name,
        &hir_value,
        assign.value.range(),
    );
    if let Some(folded) =
        oversized_int_module_constant_literal_for_codegen(&ty, remembered_value.as_ref())
    {
        hir_value = folded;
    }
    ctx.scope.define(var_name.clone(), ty.clone());
    constants.push((var_name, ty, hir_value));
}

fn oversized_int_module_constant_literal_for_codegen(
    ty: &Type,
    value: Option<&BigInt>,
) -> Option<HirExpr> {
    if !matches!(ty.resolve_alias(), Type::Int) {
        return None;
    }
    let value = value?;
    if i64::try_from(value.clone()).is_ok() {
        return None;
    }
    Some(HirExpr::LargeIntLiteral(value.to_str_radix(10)))
}

fn lower_module_integer_const_expr(expr: &Expr, ctx: &LowerCtx) -> Option<HirExpr> {
    match expr {
        Expr::Name(name) if ctx.const_integer_values.contains_key(name.id.as_str()) => {
            let scope_ty = &ctx.scope.lookup(name.id.as_str())?.ty;
            if !matches!(scope_ty, Type::Int) {
                return None;
            }
            Some(HirExpr::Name {
                name: name.id.to_string(),
                ty: Type::Int,
            })
        }
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::UAdd) => {
            lower_module_integer_const_expr(&unary.operand, ctx)
        }
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::USub) => {
            let operand = lower_module_integer_const_expr(&unary.operand, ctx)?;
            negate_module_integer_const_expr(operand)
        }
        Expr::BinOp(binop) => {
            let left = lower_module_integer_const_expr(&binop.left, ctx)?;
            let right = lower_module_integer_const_expr(&binop.right, ctx)?;
            if !matches!(left.ty(), Type::Int) || !matches!(right.ty(), Type::Int) {
                return None;
            }
            Some(HirExpr::BinOp {
                left: Box::new(left),
                op: integer_binop_source(binop.op)?.to_string(),
                right: Box::new(right),
                ty: Type::Int,
            })
        }
        _ => lower_integer_const_expr_simple(expr),
    }
}

fn negate_module_integer_const_expr(expr: HirExpr) -> Option<HirExpr> {
    negate_simple_expr(expr.clone()).or_else(|| {
        matches!(expr.ty(), Type::Int).then(|| HirExpr::UnaryOp {
            op: "-".to_string(),
            operand: Box::new(expr),
            ty: Type::Int,
        })
    })
}
