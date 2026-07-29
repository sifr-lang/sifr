use super::LowerCtx;
use crate::{HirExpr, HirStmt};
use sifr_python_ast::{Expr, ExprCall};
use sifr_type_system::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::lower) enum NumericSentinelKind {
    PositiveInfinity,
    NegativeInfinity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::lower) enum NumericSentinelDomain {
    Int,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::lower) struct NumericSentinelFact {
    kind: NumericSentinelKind,
    domain: Option<NumericSentinelDomain>,
}

impl LowerCtx {
    pub(in crate::lower) fn clear_numeric_sentinel_var(&mut self, name: &str) {
        self.numeric_sentinel_vars.remove(name);
    }

    pub(in crate::lower) fn record_numeric_sentinel_initializer(
        &mut self,
        name: String,
        kind: NumericSentinelKind,
    ) {
        self.numeric_sentinel_vars
            .insert(name, NumericSentinelFact { kind, domain: None });
    }

    pub(in crate::lower) fn numeric_sentinel_fact(
        &self,
        name: &str,
    ) -> Option<NumericSentinelFact> {
        self.numeric_sentinel_vars.get(name).copied()
    }

    pub(in crate::lower) fn numeric_sentinel_domain(
        &self,
        name: &str,
    ) -> Option<NumericSentinelDomain> {
        self.numeric_sentinel_fact(name)
            .and_then(|fact| fact.domain)
    }

    pub(in crate::lower) fn resolve_numeric_sentinel_domain(
        &mut self,
        name: &str,
        domain: NumericSentinelDomain,
    ) {
        let Some(fact) = self.numeric_sentinel_vars.get_mut(name) else {
            return;
        };
        if fact.domain == Some(domain) {
            return;
        }
        fact.domain = Some(domain);
        let resolved_ty = domain_type(domain);
        let _ = self.scope.set_type(name, resolved_ty);
        self.pending_numeric_sentinel_patches.insert(
            name.to_string(),
            NumericSentinelPatch {
                kind: fact.kind,
                domain,
            },
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::lower) struct NumericSentinelPatch {
    kind: NumericSentinelKind,
    domain: NumericSentinelDomain,
}

pub(in crate::lower) fn numeric_sentinel_kind(expr: &Expr) -> Option<NumericSentinelKind> {
    let Expr::Call(call) = expr else {
        return None;
    };
    float_sentinel_kind_from_call(call)
}

pub(in crate::lower) fn float_sentinel_kind_from_call(
    call: &ExprCall,
) -> Option<NumericSentinelKind> {
    let sifr_python_ast::Expr::Name(func_name) = call.func.as_ref() else {
        return None;
    };
    if func_name.id.as_str() != "float" || call.arguments.args.len() != 1 {
        return None;
    }
    let sifr_python_ast::Expr::StringLiteral(s) = &call.arguments.args[0] else {
        return None;
    };
    let normalized = s.value.to_str().trim().to_ascii_lowercase();
    match normalized.as_str() {
        "inf" | "+inf" | "infinity" | "+infinity" => Some(NumericSentinelKind::PositiveInfinity),
        "-inf" | "-infinity" => Some(NumericSentinelKind::NegativeInfinity),
        _ => None,
    }
}

pub(in crate::lower) fn float_sentinel_expr(kind: NumericSentinelKind) -> HirExpr {
    HirExpr::FloatLiteral(match kind {
        NumericSentinelKind::PositiveInfinity => f64::INFINITY,
        NumericSentinelKind::NegativeInfinity => f64::NEG_INFINITY,
    })
}

pub(in crate::lower) fn domain_typed_sentinel_expr(
    kind: NumericSentinelKind,
    domain: NumericSentinelDomain,
) -> HirExpr {
    match domain {
        NumericSentinelDomain::Float => float_sentinel_expr(kind),
        NumericSentinelDomain::Int => HirExpr::IntLiteral(match kind {
            NumericSentinelKind::PositiveInfinity => i64::MAX,
            NumericSentinelKind::NegativeInfinity => i64::MIN,
        }),
    }
}

pub(in crate::lower) fn maybe_resolve_numeric_sentinel_name_from_type(
    expr: &HirExpr,
    counterpart_ty: &Type,
    ctx: &mut LowerCtx,
) -> Option<NumericSentinelDomain> {
    let HirExpr::Name { name, .. } = expr else {
        return None;
    };
    let inferred_domain = numeric_domain_for_type(counterpart_ty)?;
    if ctx.numeric_sentinel_fact(name).is_some() {
        ctx.resolve_numeric_sentinel_domain(name, inferred_domain);
        return Some(inferred_domain);
    }
    None
}

pub(in crate::lower) fn retag_numeric_sentinel_name_expr(expr: HirExpr, ctx: &LowerCtx) -> HirExpr {
    let HirExpr::Name {
        name, binding_id, ..
    } = &expr
    else {
        return expr;
    };
    let Some(domain) = ctx.numeric_sentinel_domain(name) else {
        return expr;
    };
    HirExpr::Name {
        name: name.clone(),
        binding_id: *binding_id,
        ty: domain_type(domain),
    }
}

pub(in crate::lower) fn lower_sentinel_expr_for_name_domain(
    expr: &Expr,
    other: &HirExpr,
    ctx: &LowerCtx,
) -> Option<HirExpr> {
    let kind = numeric_sentinel_kind(expr)?;
    let HirExpr::Name { name, .. } = other else {
        return None;
    };
    let domain = ctx.numeric_sentinel_domain(name)?;
    Some(domain_typed_sentinel_expr(kind, domain))
}

pub(in crate::lower) fn normalize_min_max_numeric_sentinels(
    left_src: &Expr,
    right_src: &Expr,
    mut left: HirExpr,
    mut right: HirExpr,
    ctx: &mut LowerCtx,
) -> (HirExpr, HirExpr, Type) {
    let left_has_sentinel = expr_has_numeric_sentinel(left_src, ctx);
    let right_has_sentinel = expr_has_numeric_sentinel(right_src, ctx);
    maybe_resolve_numeric_sentinel_name_from_type(&left, right.ty(), ctx);
    maybe_resolve_numeric_sentinel_name_from_type(&right, left.ty(), ctx);
    left = retag_numeric_sentinel_name_expr(left, ctx);
    right = retag_numeric_sentinel_name_expr(right, ctx);

    let result_ty = if left_has_sentinel || right_has_sentinel {
        sentinel_aware_min_max_result_type(left.ty(), right.ty())
    } else {
        left.ty().clone()
    };
    (left, right, result_ty)
}

fn expr_has_numeric_sentinel(expr: &Expr, ctx: &LowerCtx) -> bool {
    matches!(expr, Expr::Name(name) if ctx.numeric_sentinel_fact(name.id.as_str()).is_some())
        || matches!(expr, Expr::Call(inner) if float_sentinel_kind_from_call(inner).is_some())
}

fn sentinel_aware_min_max_result_type(left_ty: &Type, right_ty: &Type) -> Type {
    match (
        numeric_domain_for_type(left_ty),
        numeric_domain_for_type(right_ty),
    ) {
        (Some(NumericSentinelDomain::Float), _) | (_, Some(NumericSentinelDomain::Float)) => {
            Type::Float
        }
        (Some(NumericSentinelDomain::Int), Some(_)) => Type::Int,
        _ => left_ty.clone(),
    }
}

pub(in crate::lower) fn apply_numeric_sentinel_patches(
    stmts: &mut [HirStmt],
    pending: &mut HashMap<String, NumericSentinelPatch>,
) {
    for stmt in stmts.iter_mut() {
        patch_stmt_numeric_sentinels(stmt, pending);
    }
}

fn patch_stmt_numeric_sentinels(
    stmt: &mut HirStmt,
    pending: &mut HashMap<String, NumericSentinelPatch>,
) {
    match stmt {
        HirStmt::Let {
            name, ty, value, ..
        } => {
            let Some(patch) = pending.remove(name) else {
                return;
            };
            *ty = domain_type(patch.domain);
            *value = domain_typed_sentinel_expr(patch.kind, patch.domain);
        }
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            apply_numeric_sentinel_patches(then_body, pending);
            for (_, body) in elif_clauses {
                apply_numeric_sentinel_patches(body, pending);
            }
            if let Some(body) = else_body {
                apply_numeric_sentinel_patches(body, pending);
            }
        }
        HirStmt::While {
            body, else_body, ..
        } => {
            apply_numeric_sentinel_patches(body, pending);
            if let Some(body) = else_body {
                apply_numeric_sentinel_patches(body, pending);
            }
        }
        HirStmt::For {
            body, else_body, ..
        }
        | HirStmt::AsyncFor {
            body, else_body, ..
        } => {
            apply_numeric_sentinel_patches(body, pending);
            if let Some(body) = else_body {
                apply_numeric_sentinel_patches(body, pending);
            }
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            apply_numeric_sentinel_patches(body, pending);
            for handler in handlers {
                apply_numeric_sentinel_patches(&mut handler.body, pending);
            }
        }
        HirStmt::TryFinally { body, finalbody } => {
            apply_numeric_sentinel_patches(body, pending);
            apply_numeric_sentinel_patches(finalbody, pending);
        }
        HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
            apply_numeric_sentinel_patches(body, pending);
        }
        HirStmt::NestedFunction { func, .. } => {
            apply_numeric_sentinel_patches(&mut func.body, pending);
        }
        HirStmt::Match { arms, .. } => {
            for arm in arms {
                apply_numeric_sentinel_patches(&mut arm.body, pending);
            }
        }
        HirStmt::Assign { .. }
        | HirStmt::AugAssign { .. }
        | HirStmt::Return { .. }
        | HirStmt::Expr { .. }
        | HirStmt::Break
        | HirStmt::Continue
        | HirStmt::TupleUnpack { .. }
        | HirStmt::SubscriptAssign { .. }
        | HirStmt::NestedSubscriptAssign { .. }
        | HirStmt::AttributeNestedSubscriptAssign { .. }
        | HirStmt::SubscriptAugAssign { .. }
        | HirStmt::FieldAssign { .. }
        | HirStmt::NestedFieldAssign { .. }
        | HirStmt::AttributeAugAssign { .. }
        | HirStmt::AttributeSubscriptAssign { .. }
        | HirStmt::Delete { .. }
        | HirStmt::Raise { .. }
        | HirStmt::StarUnpack { .. }
        | HirStmt::Assert { .. }
        | HirStmt::Pass
        | HirStmt::Yield { .. } => {}
    }
}

pub(in crate::lower) fn domain_type(domain: NumericSentinelDomain) -> Type {
    match domain {
        NumericSentinelDomain::Int => Type::Int,
        NumericSentinelDomain::Float => Type::Float,
    }
}

pub(in crate::lower) fn numeric_domain_for_type(ty: &Type) -> Option<NumericSentinelDomain> {
    match ty.resolve_alias() {
        Type::Int | Type::LiteralInt(_) => Some(NumericSentinelDomain::Int),
        Type::Float => Some(NumericSentinelDomain::Float),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{lower_module, HirDiagnostic, HirExpr, HirModule, HirStmt};
    use sifr_python_parser::parse_module;
    use sifr_type_system::Type;

    fn lower_source(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
        let parsed = parse_module(source).expect("parse failed");
        lower_module(parsed.suite()).map(|result| result.module)
    }

    #[test]
    fn test_regular_float_string_parse_remains_fallible() {
        let module =
            lower_source("def main():\n    parsed = float(\"3.14\")\n").expect("lowering failed");

        let HirStmt::Let { ty, value, .. } = &module.functions[0].body[0] else {
            panic!("expected parsed let");
        };
        let Type::Result(ok_ty, _) = ty else {
            panic!("expected fallible float parse type, got {ty:?}");
        };
        assert_eq!(ok_ty.as_ref(), &Type::Float);
        assert_eq!(value.ty(), ty);
    }

    #[test]
    fn test_min_call_resolves_unannotated_infinity_sentinel_to_int() {
        let module = lower_source(
            "def main() -> int:\n    best = float(\"inf\")\n    best = min(best, 1)\n    return best\n",
        )
        .expect("min-based sentinel should lower");

        let HirStmt::Let { ty, value, .. } = &module.functions[0].body[0] else {
            panic!("expected sentinel initializer");
        };
        assert_eq!(ty, &Type::Int);
        assert_eq!(value.ty(), &Type::Int);
        assert!(matches!(value, HirExpr::IntLiteral(i64::MAX)));

        let HirStmt::Assign { value, .. } = &module.functions[0].body[1] else {
            panic!("expected min-based reassignment");
        };
        assert_eq!(value.ty(), &Type::Int);
    }

    #[test]
    fn test_sentinel_comparison_branch_returns_int_after_resolution() {
        let module = lower_source(
            "def main() -> int:\n    best = float(\"inf\")\n    best = min(best, 4)\n    return best if best != float(\"inf\") else 0\n",
        )
        .expect("sentinel branch should lower");

        let HirStmt::Return {
            value: Some(HirExpr::IfExpr { ty, .. }),
        } = &module.functions[0].body[2]
        else {
            panic!("expected int-typed sentinel if-expression return");
        };
        assert_eq!(ty, &Type::Int);
    }
}
