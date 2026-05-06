use crate::HirExpr;
use ruff_text_size::Ranged;
use sifr_python_ast::{Expr, Stmt};
use sifr_type_system::Type;

use super::simple_expr::lower_integer_const_expr_simple;
use super::typing_and_functions::resolve_annotation_expr;
use super::{fixed_width_fitting, LowerCtx};

pub(super) fn collect_module_constants(
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
    let Some(mut hir_value) = lower_integer_const_expr_simple(value_expr) else {
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
        fixed_width_fitting::remember_module_const_integer(
            ctx,
            &var_name,
            &hir_value,
            value_expr.range(),
        );
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
    let Some(hir_value) = lower_integer_const_expr_simple(&assign.value) else {
        return;
    };

    let ty = hir_value.ty().clone();
    fixed_width_fitting::remember_module_const_integer(
        ctx,
        &var_name,
        &hir_value,
        assign.value.range(),
    );
    ctx.scope.define(var_name.clone(), ty.clone());
    constants.push((var_name, ty, hir_value));
}
