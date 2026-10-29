use super::LowerCtx;
use sifr_python_ast::{Expr, ExprBinOp, ExprSubscript, Number, Operator};
use sifr_type_system::{Type, remove_none_from_union};

pub(in crate::lower) fn guarded_sequence_index_result_type(
    sub: &ExprSubscript,
    object_ty: &Type,
    ctx: &LowerCtx,
) -> Option<Type> {
    if let Some(sequence_name) = sequence_guard_target_name(sub.value.as_ref()) {
        return match object_ty.resolve_alias() {
            Type::List(elem_ty) => {
                guarded_element_type(sequence_name.as_str(), elem_ty, &sub.slice, ctx)
            }
            Type::Dict(_, value_ty) => {
                if ctx.has_dict_key_guard(sequence_name.as_str(), &sub.slice) {
                    Some(*value_ty.clone())
                } else if ctx.has_subscript_guard(sequence_name.as_str(), &sub.slice) {
                    Some(remove_none_from_union(value_ty.as_ref()))
                } else if ctx.has_subscript_access_guard(sequence_name.as_str(), &sub.slice) {
                    Some(*value_ty.clone())
                } else {
                    None
                }
            }
            Type::Str => guarded_string_index_type(sequence_name.as_str(), &sub.slice, ctx),
            Type::Bytes => guarded_bytes_index_type(sequence_name.as_str(), &sub.slice, ctx),
            _ => None,
        };
    }

    let Expr::Subscript(outer_sub) = sub.value.as_ref() else {
        return None;
    };
    let Expr::Name(matrix_name) = outer_sub.value.as_ref() else {
        return None;
    };
    let (outer_anchor, outer_extra_len, inner_anchor, inner_extra_len) =
        ctx.matrix_sequence_fact(matrix_name.id.as_str())?;
    if !index_expr_is_safe_for_anchor(
        outer_sub.slice.as_ref(),
        &outer_anchor,
        outer_extra_len,
        ctx,
    ) {
        return None;
    }
    match object_ty.resolve_alias() {
        Type::List(elem_ty) => {
            if index_expr_is_safe_for_anchor(&sub.slice, &inner_anchor, inner_extra_len, ctx) {
                Some(*elem_ty.clone())
            } else {
                None
            }
        }
        Type::Str => None,
        _ => None,
    }
}

fn guarded_element_type(
    sequence_name: &str,
    elem_ty: &Type,
    index_expr: &Expr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if ctx.has_subscript_guard(sequence_name, index_expr) {
        return Some(remove_none_from_union(elem_ty));
    }
    if has_guarded_sequence_index(sequence_name, index_expr, ctx) {
        Some(elem_ty.clone())
    } else {
        None
    }
}

fn guarded_string_index_type(
    sequence_name: &str,
    index_expr: &Expr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if has_guarded_sequence_index(sequence_name, index_expr, ctx) {
        Some(Type::Str)
    } else {
        None
    }
}

fn guarded_bytes_index_type(
    sequence_name: &str,
    index_expr: &Expr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if has_guarded_sequence_index(sequence_name, index_expr, ctx) {
        Some(Type::FixedInt(sifr_type_system::FixedIntType::U8))
    } else {
        None
    }
}

fn has_guarded_sequence_index(sequence_name: &str, index_expr: &Expr, ctx: &LowerCtx) -> bool {
    if ctx.has_subscript_access_guard(sequence_name, index_expr) {
        return true;
    }
    if index_expr_is_safe_for_anchor(index_expr, sequence_name, 0, ctx) {
        return true;
    }
    if let Some((anchor_sequence, extra_len)) = ctx.sized_sequence_fact(sequence_name) {
        return index_expr_is_safe_for_anchor(index_expr, &anchor_sequence, extra_len, ctx);
    }
    if let Some((outer_anchor, outer_extra_len, _, _)) = ctx.matrix_sequence_fact(sequence_name) {
        return index_expr_is_safe_for_anchor(index_expr, &outer_anchor, outer_extra_len, ctx);
    }
    false
}

fn index_expr_is_safe_for_anchor(
    index_expr: &Expr,
    anchor_sequence: &str,
    extra_len: usize,
    ctx: &LowerCtx,
) -> bool {
    match index_expr {
        Expr::Name(index_name) => {
            let index_var = index_name.id.as_str();
            ctx.has_index_var_guard(anchor_sequence, index_var)
                || (ctx.is_zero_based_pointer(index_var)
                    && ctx.min_length_guard(anchor_sequence) > 0)
                || ctx
                    .end_pointer_sequence(index_var)
                    .is_some_and(|pointer_sequence| {
                        pointer_sequence == anchor_sequence
                            && ctx.min_length_guard(anchor_sequence) > 0
                    })
        }
        Expr::BinOp(ExprBinOp {
            left, op, right, ..
        }) => {
            let Expr::Name(index_name) = left.as_ref() else {
                return false;
            };
            let Some(offset) = literal_usize(right.as_ref()) else {
                return false;
            };
            match op {
                Operator::Add => ctx.has_index_var_offset_guard(
                    anchor_sequence,
                    index_name.id.as_str(),
                    offset.saturating_sub(extra_len),
                ),
                Operator::Sub => false,
                _ => false,
            }
        }
        Expr::NumberLiteral(num) => {
            let Number::Int(value) = &num.value else {
                return false;
            };
            let Some(index_value) = value.as_i64() else {
                return false;
            };
            let Ok(index_value) = usize::try_from(index_value) else {
                return false;
            };
            if index_value < extra_len {
                true
            } else {
                ctx.min_length_guard(anchor_sequence) > index_value - extra_len
            }
        }
        _ => false,
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

#[cfg(test)]
#[path = "../../tests/unit/lower/guarded_index/tests.rs"]
mod tests;
