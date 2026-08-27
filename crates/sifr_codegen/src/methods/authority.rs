//! Authoritative type-and-name dispatch for source-language method semantics.

use super::{bytes, common, decimal, deque, dict, fixed_width, list, set, string};
use crate::RustExpr;
use crate::helpers::is_option_type;
use sifr_type_system::Type;

pub(crate) struct LoweredMethod {
    pub(crate) expr: RustExpr,
}

pub(crate) fn is_in_place_collection_method(object_ty: &Type, method: &str) -> bool {
    match object_ty.resolve_alias() {
        Type::List(_) => matches!(
            method,
            "append" | "extend" | "insert" | "clear" | "reverse" | "sort" | "pop" | "remove"
        ),
        Type::Set(_) => matches!(
            method,
            "add"
                | "update"
                | "intersection_update"
                | "difference_update"
                | "symmetric_difference_update"
                | "remove"
                | "discard"
                | "clear"
                | "pop"
        ),
        _ => false,
    }
}

pub(crate) fn lower_method(
    object_ty: &Type,
    method: &str,
    object: &RustExpr,
    args: &[RustExpr],
) -> Option<LoweredMethod> {
    lower_method_with_context(object_ty, method, object, args, false)
}

pub(crate) fn lower_method_with_context(
    object_ty: &Type,
    method: &str,
    object: &RustExpr,
    args: &[RustExpr],
    is_deque_data_field: bool,
) -> Option<LoweredMethod> {
    let resolved_object_ty = object_ty.resolve_alias();
    let expr = match (resolved_object_ty, method) {
        (Type::Tuple(elems), "len") => common::lower_tuple_len(elems.len(), args),
        (Type::Tuple(elems), "count") => common::lower_tuple_count(elems.len(), object, args),
        (Type::Tuple(elems), "index") => common::lower_tuple_index(elems.len(), object, args),
        (Type::Str, "len") => common::lower_string_char_len(object, args),
        (ty, "len") if is_option_type(ty) => common::lower_option_len(object, args),
        (_, "len") => common::lower_len(object, args),
        (Type::Str, "upper") => string::lower_upper(object, args),
        (Type::Str, "lower") => string::lower_lower(object, args),
        (Type::Str, "strip") => string::lower_strip(object, args),
        (Type::Str, "startswith") => string::lower_startswith(object, args),
        (Type::Str, "endswith") => string::lower_endswith(object, args),
        (Type::Str, "split") => string::lower_split(object, args),
        (Type::Str, "replace") => string::lower_replace(object, args),
        (Type::Str, "find") => string::lower_find(object, args),
        (Type::Str, "rfind") => string::lower_rfind(object, args),
        (Type::Str, "lstrip") => string::lower_lstrip(object, args),
        (Type::Str, "rstrip") => string::lower_rstrip(object, args),
        (Type::Str, "count") => string::lower_count(object, args),
        (Type::Str, "join") => string::lower_join(object, args),
        (Type::Str, "title") => string::lower_title(object, args),
        (Type::Str, "capitalize") => string::lower_capitalize(object, args),
        (Type::Str, "swapcase") => string::lower_swapcase(object, args),
        (Type::Str, "isdigit") => string::lower_isdigit(object, args),
        (Type::Str, "isalpha") => string::lower_isalpha(object, args),
        (Type::Str, "isalnum") => string::lower_isalnum(object, args),
        (Type::Str, "isspace") => string::lower_isspace(object, args),
        (Type::Str, "isupper") => string::lower_isupper(object, args),
        (Type::Str, "islower") => string::lower_islower(object, args),
        (Type::Str, "center") => string::lower_center(object, args),
        (Type::Str, "ljust") => string::lower_ljust(object, args),
        (Type::Str, "rjust") => string::lower_rjust(object, args),
        (Type::Str, "zfill") => string::lower_zfill(object, args),
        (Type::Decimal, "quantize") => decimal::lower_decimal_quantize(object, args),
        (Type::Decimal, "sqrt") => decimal::lower_decimal_sqrt(object, args),
        (Type::Decimal, "round") => decimal::lower_decimal_round(object, args),
        (Type::Decimal, "abs") => decimal::lower_decimal_abs(object, args),
        (Type::Decimal, "is_zero") => decimal::lower_decimal_is_zero(object, args),
        (Type::Decimal, "is_finite") => decimal::lower_decimal_is_finite(args),
        (Type::BigDecimal, "quantize") => decimal::lower_bigdecimal_quantize(object, args),
        (Type::BigDecimal, "sqrt") => decimal::lower_bigdecimal_sqrt(object, args),
        (Type::BigDecimal, "round") => decimal::lower_bigdecimal_round(object, args),
        (Type::BigDecimal, "abs") => decimal::lower_bigdecimal_abs(object, args),
        (Type::BigDecimal, "is_zero") => decimal::lower_bigdecimal_is_zero(object, args),
        (Type::BigDecimal, "is_finite") => decimal::lower_bigdecimal_is_finite(args),
        (Type::List(_), "append") if is_deque_data_field => deque::lower_append(object, args),
        (Type::List(_), "appendleft") if is_deque_data_field => {
            deque::lower_appendleft(object, args)
        }
        (Type::List(elem), "pop") if is_deque_data_field => deque::lower_pop(object, args)
            .map(|expr| crate::helpers::normalize_safe_option_result(elem, expr)),
        (Type::List(elem), "popleft") if is_deque_data_field => deque::lower_popleft(object, args)
            .map(|expr| crate::helpers::normalize_safe_option_result(elem, expr)),
        (Type::List(_), "append") => list::lower_append(object, args),
        (Type::List(_), "extend") => list::lower_extend(object, args),
        (Type::List(_), "insert") => list::lower_insert(object, args),
        (Type::List(_), "clear") => list::lower_clear(object, args),
        (Type::List(_), "copy" | "cloned") => list::lower_copy(object, args),
        (Type::List(_), "reverse") => list::lower_reverse(object, args),
        (Type::List(_), "sort") => list::lower_sort(object, args),
        (Type::List(_), "count") => list::lower_count(object, args),
        (Type::List(_), "contains") => list::lower_contains(object, args),
        (Type::List(elem), "pop") => list::lower_pop(object, args)
            .map(|expr| crate::helpers::normalize_safe_option_result(elem, expr)),
        (Type::List(_), "remove") => list::lower_remove(object, args),
        (Type::List(_), "index") => list::lower_index(object, args),
        (Type::Bytes, "count") => bytes::lower_count(object, args),
        (Type::Bytes, "contains") => bytes::lower_contains(object, args),
        (Type::Bytes, "find") => bytes::lower_find(object, args),
        (Type::Bytes, "startswith") => bytes::lower_startswith(object, args),
        (Type::Bytes, "endswith") => bytes::lower_endswith(object, args),
        (Type::Bytes, "hex") => bytes::lower_hex(object, args),
        (Type::Bytes, "to_ints") => bytes::lower_to_ints(object, args),
        (Type::FixedInt(_), "checked_add") => fixed_width::lower_checked_add(object, args),
        (Type::FixedInt(_), "checked_sub") => fixed_width::lower_checked_sub(object, args),
        (Type::FixedInt(_), "checked_mul") => fixed_width::lower_checked_mul(object, args),
        (Type::FixedInt(_), "wrapping_add") => fixed_width::lower_wrapping_add(object, args),
        (Type::FixedInt(_), "wrapping_sub") => fixed_width::lower_wrapping_sub(object, args),
        (Type::FixedInt(_), "wrapping_mul") => fixed_width::lower_wrapping_mul(object, args),
        (Type::FixedInt(_), "saturating_add") => fixed_width::lower_saturating_add(object, args),
        (Type::FixedInt(_), "saturating_sub") => fixed_width::lower_saturating_sub(object, args),
        (Type::FixedInt(_), "saturating_mul") => fixed_width::lower_saturating_mul(object, args),
        (Type::FixedInt(_), "overflowing_add") => fixed_width::lower_overflowing_add(object, args),
        (Type::FixedInt(_), "overflowing_sub") => fixed_width::lower_overflowing_sub(object, args),
        (Type::FixedInt(_), "overflowing_mul") => fixed_width::lower_overflowing_mul(object, args),
        (Type::Dict(_, _), "keys") => dict::lower_keys(object, args),
        (Type::Dict(_, _), "values") => dict::lower_values(object, args),
        (Type::Dict(_, _), "items") => dict::lower_items(object, args),
        (Type::Dict(_, _), "update") => dict::lower_update(object, args),
        (Type::Dict(_, _), "clear") => dict::lower_clear(object, args),
        (Type::Dict(_, _), "copy") => dict::lower_copy(object, args),
        (Type::Dict(_, _), "contains") => dict::lower_contains(object, args),
        (Type::Dict(_, value), "get") => dict::lower_get(object, args).map(|expr| {
            if args.len() == 1 {
                crate::helpers::normalize_safe_option_result(value, expr)
            } else {
                expr
            }
        }),
        (Type::Dict(_, value), "pop") => dict::lower_pop(object, args).map(|expr| {
            if args.len() == 1 {
                crate::helpers::normalize_safe_option_result(value, expr)
            } else {
                expr
            }
        }),
        (Type::Dict(_, _), "setdefault") => dict::lower_setdefault(object, args),
        (Type::Set(_), "add") => set::lower_add(object, args),
        (Type::Set(_), "remove") => set::lower_remove(object, args),
        (Type::Set(_), "discard") => set::lower_discard(object, args),
        (Type::Set(_), "contains") => set::lower_contains(object, args),
        (Type::Set(_), "clear") => set::lower_clear(object, args),
        (Type::Set(_), "copy") => set::lower_copy(object, args),
        (Type::Set(_), "issubset") => set::lower_issubset(object, args),
        (Type::Set(_), "issuperset") => set::lower_issuperset(object, args),
        (Type::Set(_), "isdisjoint") => set::lower_isdisjoint(object, args),
        (Type::Set(elem), "pop") => set::lower_pop(object, args)
            .map(|expr| crate::helpers::normalize_safe_option_result(elem, expr)),
        (Type::Set(_), "union") => set::lower_union(object, args),
        (Type::Set(_), "intersection") => set::lower_intersection(object, args),
        (Type::Set(_), "difference") => set::lower_difference(object, args),
        (Type::Set(_), "symmetric_difference") => set::lower_symmetric_difference(object, args),
        _ => return None,
    };

    Some(LoweredMethod { expr: expr? })
}
