use crate::HirExpr;
use num_bigint::BigInt;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Stmt, UnaryOp};
use sifr_type_system::Type;

use super::simple_expr::{
    integer_binop_source, lower_integer_const_expr_simple, negate_simple_expr,
};
use super::typing_and_functions::resolve_annotation_expr;
use super::{expressions::lower_expr, fixed_width_fitting, LowerCtx};

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
    let var_name = name.id.to_string();
    let ty = resolve_annotation_expr(&ann.annotation, ctx);
    let Some(hir_value) = lower_annotated_module_constant_expr(value_expr, ctx, &var_name, &ty)
    else {
        return;
    };
    ctx.scope
        .define_module_constant(var_name.clone(), ty.clone());
    constants.push((var_name, ty, hir_value));
}

fn lower_annotated_module_constant_expr(
    value_expr: &Expr,
    ctx: &mut LowerCtx,
    var_name: &str,
    ty: &Type,
) -> Option<HirExpr> {
    if let Some(mut hir_value) = lower_module_integer_const_expr(value_expr, ctx) {
        let error_count_before_initializer = ctx.error_count();
        if let Some(folded_value) = fixed_width_fitting::validate_annotated_constant_initializer(
            ctx,
            ty,
            &hir_value,
            value_expr.range(),
        ) {
            hir_value = folded_value;
        }
        if ctx.error_count() == error_count_before_initializer {
            let remembered_value = fixed_width_fitting::remember_module_const_integer(
                ctx,
                var_name,
                &hir_value,
                value_expr.range(),
            );
            if let Some(folded) =
                oversized_int_module_constant_literal_for_codegen(ty, remembered_value.as_ref())
            {
                hir_value = folded;
            }
        }
        return Some(hir_value);
    }

    let hir_value = lower_expr(value_expr, ctx)?;
    let error_count_before_initializer = ctx.error_count();
    let folded_value = fixed_width_fitting::validate_annotated_constant_initializer(
        ctx,
        ty,
        &hir_value,
        value_expr.range(),
    );
    if ctx.error_count() != error_count_before_initializer {
        return None;
    }
    if let Some(folded_value) = folded_value {
        return Some(folded_value);
    }
    let hir_value = canonicalize_non_finite_float_constant(hir_value);
    if is_supported_annotated_module_constant_expr(&hir_value) {
        Some(hir_value)
    } else {
        reject_unsupported_private_declaration_constant(ctx, var_name, value_expr);
        None
    }
}

fn canonicalize_non_finite_float_constant(value: HirExpr) -> HirExpr {
    match evaluate_float_division_constant(&value) {
        Some(folded) if !folded.is_finite() => HirExpr::FloatLiteral(folded),
        _ => value,
    }
}

fn evaluate_float_division_constant(value: &HirExpr) -> Option<f64> {
    match value {
        HirExpr::FloatLiteral(value) => Some(*value),
        HirExpr::UnaryOp { op, operand, .. } if op == "+" => {
            evaluate_float_division_constant(operand)
        }
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            Some(-evaluate_float_division_constant(operand)?)
        }
        HirExpr::BinOp {
            left, op, right, ..
        } if op == "/" => {
            Some(evaluate_float_division_constant(left)? / evaluate_float_division_constant(right)?)
        }
        _ => None,
    }
}

fn is_supported_annotated_module_constant_expr(value: &HirExpr) -> bool {
    match value {
        HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::ConstructorCall { .. } => true,
        HirExpr::UnaryOp { op, operand, .. } => {
            matches!(op.as_str(), "+" | "-" | "not")
                && is_supported_annotated_module_constant_expr(operand)
        }
        HirExpr::BinOp {
            left, op, right, ..
        } => {
            matches!(
                op.as_str(),
                "+" | "-" | "*" | "/" | "//" | "%" | "**" | "&" | "|" | "^" | "<<" | ">>"
            ) && is_supported_annotated_module_constant_expr(left)
                && is_supported_annotated_module_constant_expr(right)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            is_supported_annotated_module_constant_expr(left)
                && comparators
                    .iter()
                    .all(is_supported_annotated_module_constant_expr)
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .all(is_supported_annotated_module_constant_expr),
        _ => false,
    }
}

fn reject_unsupported_private_declaration_constant(
    ctx: &mut LowerCtx,
    var_name: &str,
    value_expr: &Expr,
) {
    if !ctx.is_sysroot_private_declaration() {
        return;
    }
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
        format!(
            "unsupported private declaration constant '{var_name}': initializer must be a literal, supported constant expression, or constructor call"
        ),
        value_expr.range(),
    );
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
    ctx.scope
        .define_module_constant(var_name.clone(), ty.clone());
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
                binding_id: Some(ctx.scope.lookup(name.id.as_str())?.binding_id),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lower_module_sysroot_private_declaration_with_externals, ExternalDefs};
    use sifr_diagnostics::DiagnosticCode;
    use sifr_python_parser::parse_module;

    fn lower_private_declaration_constants(source: &str) -> Vec<(String, Type, HirExpr)> {
        let parsed = parse_module(source).expect("source should parse");
        lower_module_sysroot_private_declaration_with_externals(
            parsed.suite(),
            &ExternalDefs::default(),
        )
        .expect("private declaration should lower")
        .module
        .constants
    }

    #[test]
    fn private_declarations_collect_annotated_scalar_module_constants() {
        let constants = lower_private_declaration_constants(
            "pi: float = 3.141592653589793\ninf: float = 1.0 / 0.0\nnan: float = 0.0 / 0.0\nneg_inf: float = -1.0 / 0.0\nfinite: float = 1.0 / 2.0\nflag: bool = True\n",
        );

        assert!(constants.iter().any(|(name, ty, value)| name == "pi"
            && ty == &Type::Float
            && matches!(value, HirExpr::FloatLiteral(_))));
        assert!(constants.iter().any(|(name, ty, value)| name == "inf"
            && ty == &Type::Float
            && matches!(value, HirExpr::FloatLiteral(value) if value.is_infinite() && value.is_sign_positive())));
        assert!(constants.iter().any(|(name, ty, value)| name == "nan"
            && ty == &Type::Float
            && matches!(value, HirExpr::FloatLiteral(value) if value.is_nan())));
        assert!(constants.iter().any(|(name, ty, value)| name == "neg_inf"
            && ty == &Type::Float
            && matches!(value, HirExpr::FloatLiteral(value) if value.is_infinite() && value.is_sign_negative())));
        assert!(constants.iter().any(|(name, ty, value)| name == "finite"
            && ty == &Type::Float
            && matches!(value, HirExpr::BinOp { .. })));
        assert!(constants.iter().any(|(name, ty, value)| name == "flag"
            && ty == &Type::Bool
            && matches!(value, HirExpr::BoolLiteral(true))));
    }

    #[test]
    fn annotated_scalar_module_constant_type_mismatch_is_diagnostic() {
        let parsed = parse_module("bad: float = \"not float\"\n").expect("source should parse");
        let errors = match lower_module_sysroot_private_declaration_with_externals(
            parsed.suite(),
            &ExternalDefs::default(),
        ) {
            Ok(_) => panic!("mismatched constant should fail"),
            Err(errors) => errors,
        };

        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "type mismatch: expected 'float', got 'str'"
        }));
    }

    #[test]
    fn private_declaration_scalar_module_constant_alias_is_diagnostic() {
        let parsed =
            parse_module("pi: float = 3.0\nalias: float = pi\n").expect("source should parse");
        let errors = match lower_module_sysroot_private_declaration_with_externals(
            parsed.suite(),
            &ExternalDefs::default(),
        ) {
            Ok(_) => panic!("unsupported private declaration constant should fail"),
            Err(errors) => errors,
        };

        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
                && error.message
                    == "unsupported private declaration constant 'alias': initializer must be a literal, supported constant expression, or constructor call"
        }));
    }
}
