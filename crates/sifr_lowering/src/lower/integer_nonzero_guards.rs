use std::collections::HashSet;

use sifr_python_ast::{BoolOp, CmpOp, Expr, Number, UnaryOp};
use sifr_type_system::Type;

use super::LowerCtx;

impl LowerCtx {
    pub(in crate::lower) fn add_proven_nonzero_integer_binding(&mut self, name: String) {
        self.proven_nonzero_integer_bindings.insert(name);
    }

    pub(in crate::lower) fn clear_proven_nonzero_integer_binding(&mut self, name: &str) {
        self.proven_nonzero_integer_bindings.remove(name);
    }

    pub(in crate::lower) fn is_proven_nonzero_integer_binding(&self, name: &str) -> bool {
        self.proven_nonzero_integer_bindings.contains(name)
    }

    pub(in crate::lower) fn save_proven_nonzero_integer_bindings(&self) -> HashSet<String> {
        self.proven_nonzero_integer_bindings.clone()
    }

    pub(in crate::lower) fn restore_proven_nonzero_integer_bindings(
        &mut self,
        snapshot: &HashSet<String>,
    ) {
        self.proven_nonzero_integer_bindings.clone_from(snapshot);
    }
}

pub(in crate::lower) fn detect_true_nonzero_integer_guards(
    expr: &Expr,
    ctx: &LowerCtx,
) -> Vec<String> {
    match expr {
        Expr::Compare(cmp) if cmp.ops.len() == 1 && cmp.comparators.len() == 1 => {
            detect_compare_nonzero_guard(
                cmp.left.as_ref(),
                cmp.ops[0],
                &cmp.comparators[0],
                true,
                ctx,
            )
            .into_iter()
            .collect()
        }
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::And) => boolop
            .values
            .iter()
            .flat_map(|value| detect_true_nonzero_integer_guards(value, ctx))
            .collect(),
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::Not) => {
            detect_false_nonzero_integer_guards(&unary.operand, ctx)
        }
        _ => Vec::new(),
    }
}

pub(in crate::lower) fn detect_false_nonzero_integer_guards(
    expr: &Expr,
    ctx: &LowerCtx,
) -> Vec<String> {
    match expr {
        Expr::Compare(cmp) if cmp.ops.len() == 1 && cmp.comparators.len() == 1 => {
            detect_compare_nonzero_guard(
                cmp.left.as_ref(),
                cmp.ops[0],
                &cmp.comparators[0],
                false,
                ctx,
            )
            .into_iter()
            .collect()
        }
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::Or) => boolop
            .values
            .iter()
            .flat_map(|value| detect_false_nonzero_integer_guards(value, ctx))
            .collect(),
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::Not) => {
            detect_true_nonzero_integer_guards(&unary.operand, ctx)
        }
        _ => Vec::new(),
    }
}

fn detect_compare_nonzero_guard(
    left: &Expr,
    op: CmpOp,
    right: &Expr,
    is_true_branch: bool,
    ctx: &LowerCtx,
) -> Option<String> {
    let name = match (name_if_exact_int(left, ctx), is_zero_integer_literal(right)) {
        (Some(name), true) => Some(name),
        _ => match (name_if_exact_int(right, ctx), is_zero_integer_literal(left)) {
            (Some(name), true) => Some(name),
            _ => None,
        },
    }?;

    let proves_nonzero = matches!(
        (op, is_true_branch),
        (CmpOp::NotEq, true) | (CmpOp::Eq, false)
    );
    proves_nonzero.then_some(name)
}

fn name_if_exact_int(expr: &Expr, ctx: &LowerCtx) -> Option<String> {
    let Expr::Name(name) = expr else {
        return None;
    };
    let binding = ctx.scope.lookup(name.id.as_str())?;
    is_exact_int_like(binding.effective_type()).then(|| name.id.to_string())
}

fn is_exact_int_like(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::LiteralInt(_))
}

fn is_zero_integer_literal(expr: &Expr) -> bool {
    let Expr::NumberLiteral(number) = expr else {
        return false;
    };
    let Number::Int(value) = &number.value else {
        return false;
    };
    value.as_i64() == Some(0)
}
