use super::{
    Expr, LowerCtx, SequenceGuard, SubscriptReferenceStability, Type, key_guard_token, literal_int,
    sequence_guard_target_name,
};

pub(super) fn subscript_present_guard_from_non_none_compare(
    left: &Expr,
    right: &Expr,
    ctx: &LowerCtx,
) -> Vec<SequenceGuard> {
    if matches!(right, Expr::NoneLiteral(_)) {
        return subscript_present_guard(left, ctx);
    }
    if matches!(left, Expr::NoneLiteral(_)) {
        return subscript_present_guard(right, ctx);
    }
    Vec::new()
}

fn subscript_present_guard(expr: &Expr, ctx: &LowerCtx) -> Vec<SequenceGuard> {
    let Expr::Subscript(subscript) = expr else {
        return Vec::new();
    };
    let Some(sequence) = sequence_guard_target_name(subscript.value.as_ref()) else {
        return Vec::new();
    };
    let Some(index_expr_debug) = key_guard_token(subscript.slice.as_ref()) else {
        return Vec::new();
    };
    let reference_stability =
        subscript_reference_stability(&sequence, subscript.slice.as_ref(), ctx);
    vec![
        SequenceGuard::SubscriptAccessible {
            sequence: sequence.clone(),
            index_expr_debug: index_expr_debug.clone(),
        },
        SequenceGuard::SubscriptPresent {
            sequence,
            index_expr_debug,
            reference_stability,
        },
    ]
}

fn subscript_reference_stability(
    receiver: &str,
    index: &Expr,
    ctx: &LowerCtx,
) -> SubscriptReferenceStability {
    if matches!(
        ctx.scope.effective_type(receiver).map(Type::resolve_alias),
        Some(Type::Dict(_, _))
    ) {
        return SubscriptReferenceStability::StableAcrossGrowth;
    }
    let nonnegative = literal_int(index).is_some_and(|value| value >= 0)
        || matches!(index, Expr::Name(name) if ctx.scope.const_integer_value(name.id.as_str()).is_some_and(|value| value.sign() != num_bigint::Sign::Minus))
        || matches!(index, Expr::Name(name) if ctx.sequence_guards.iter().any(|guard| matches!(guard, SequenceGuard::IndexVarNonNegative { index_var } if index_var == name.id.as_str())));
    if nonnegative {
        SubscriptReferenceStability::StableAcrossGrowth
    } else {
        SubscriptReferenceStability::MayChangeOnGrowth
    }
}

pub(super) fn dict_contains_guard(key_expr: &Expr, haystack_expr: &Expr) -> Vec<SequenceGuard> {
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
