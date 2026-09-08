//! Method registry and dispatch for incremental IR rollout.

mod bytes;
mod common;
mod decimal;
mod deque;
mod dict;
mod dispatch;
mod fixed_width;
mod list;
mod set;
mod string;

use crate::RustExpr;
use crate::helpers::is_option_type;
use sifr_type_system::Type;

pub(crate) use dispatch::{
    LoweredMethod, is_in_place_collection_method, lower_method, lower_method_with_context,
    lower_method_with_discard_context,
};

fn lower_method_impl(
    object_ty: &Type,
    method: &str,
    object: &RustExpr,
    args: &[RustExpr],
    is_deque_data_field: bool,
    discard_result: bool,
) -> Option<LoweredMethod> {
    let resolved_object_ty = object_ty.resolve_alias();
    let expr = match (resolved_object_ty, method) {
        (Type::Tuple(elems), "len") => common::lower_tuple_len(elems.len(), args),
        (Type::Tuple(elems), "count") => common::lower_tuple_count(elems.len(), object, args),
        (Type::Tuple(elems), "index") => common::lower_tuple_index(elems.len(), object, args),
        (Type::Str, "len") => common::lower_string_char_len(object, args),
        (ty, "len") if is_option_type(ty) => common::lower_option_len(ty, object, args),
        (Type::Class { .. }, "len") if args.is_empty() => Some(RustExpr::MethodCall {
            receiver: Box::new(object.clone()),
            method: "len".to_string(),
            args: Vec::new(),
        }),
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
        (Type::List(_), "reverse") if is_deque_data_field => deque::lower_reverse(object, args),
        (Type::List(_), "append") => list::lower_append(object, args),
        (Type::List(_), "extend") => list::lower_extend(object, args),
        (Type::List(_), "insert") => list::lower_insert(object, args),
        (Type::List(_), "clear") => list::lower_clear(object, args),
        (Type::List(_), "copy") => list::lower_copy(object, args),
        (Type::List(_), "reverse") => list::lower_reverse(object, args),
        (Type::List(elem), "sort") => list::lower_sort(object, elem, args),
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
        (Type::Dict(key, value), "setdefault") => {
            dict::lower_setdefault(object, key, value, args, discard_result)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_expr;

    fn lower_method(
        object_ty: &Type,
        method: &str,
        rendered_object: &str,
        rendered_args: &[String],
    ) -> Option<LoweredMethod> {
        let object = RustExpr::Ident(rendered_object.to_string());
        let args = rendered_args
            .iter()
            .cloned()
            .map(RustExpr::Ident)
            .collect::<Vec<_>>();
        super::lower_method(object_ty, method, &object, &args)
    }

    fn lower_method_with_context(
        object_ty: &Type,
        method: &str,
        rendered_object: &str,
        rendered_args: &[String],
        is_deque_data_field: bool,
    ) -> Option<LoweredMethod> {
        let object = RustExpr::Ident(rendered_object.to_string());
        let args = rendered_args
            .iter()
            .cloned()
            .map(RustExpr::Ident)
            .collect::<Vec<_>>();
        super::lower_method_with_context(object_ty, method, &object, &args, is_deque_data_field)
    }

    #[test]
    fn lowers_string_methods_via_registry() {
        let tuple_len = lower_method(
            &Type::Tuple(vec![Type::Int, Type::Str, Type::Bool]),
            "len",
            "t",
            &[],
        )
        .expect("tuple len lowers");
        assert_eq!(render_expr(&tuple_len.expr), "SifrInt::from_i64(3)");

        let tuple_count = lower_method(
            &Type::Tuple(vec![Type::Int, Type::Str, Type::Bool]),
            "count",
            "t",
            &["SifrInt::from_i64(1)".to_string()],
        )
        .expect("tuple count lowers");
        assert!(render_expr(&tuple_count.expr).contains("__count"));

        let tuple_index = lower_method(
            &Type::Tuple(vec![Type::Int, Type::Str, Type::Bool]),
            "index",
            "t",
            &["1".to_string()],
        )
        .expect("tuple index lowers");
        assert!(render_expr(&tuple_index.expr).contains("__result"));

        let str_len = lower_method(&Type::Str, "len", "s", &[]).expect("str len lowers");
        assert_eq!(
            render_expr(&str_len.expr),
            "SifrInt::from(s.chars().count())"
        );

        let option_len = lower_method(
            &Type::Union(vec![Type::List(Box::new(Type::Int)), Type::None]),
            "len",
            "opt",
            &[],
        )
        .expect("option len lowers");
        assert_eq!(
            render_expr(&option_len.expr),
            "SifrInt::from(opt.as_ref().map_or(0_usize, ::std::vec::Vec::len))"
        );

        let optional_string_len =
            lower_method(&Type::Union(vec![Type::Str, Type::None]), "len", "opt", &[])
                .expect("optional string len lowers");
        assert_eq!(
            render_expr(&optional_string_len.expr),
            "SifrInt::from(opt.as_ref().map_or(0_usize, |value| value.chars().count()))"
        );

        let generic_len = lower_method(
            &Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
            "len",
            "d",
            &[],
        )
        .expect("generic len lowers");
        assert_eq!(render_expr(&generic_len.expr), "SifrInt::from(d.len())");

        let upper = lower_method(&Type::Str, "upper", "s", &[]).expect("upper lowers");
        assert_eq!(render_expr(&upper.expr), "s.to_uppercase()");

        let lower = lower_method(&Type::Str, "lower", "s", &[]).expect("lower lowers");
        assert_eq!(render_expr(&lower.expr), "s.to_lowercase()");

        let strip = lower_method(&Type::Str, "strip", "s", &[]).expect("strip lowers");
        assert_eq!(render_expr(&strip.expr), "s.trim().to_string()");

        let starts = lower_method(&Type::Str, "startswith", "s", &["prefix".to_string()])
            .expect("startswith lowers");
        assert_eq!(render_expr(&starts.expr), "s.starts_with(&prefix)");

        let ends = lower_method(&Type::Str, "endswith", "s", &["suffix".to_string()])
            .expect("endswith lowers");
        assert_eq!(render_expr(&ends.expr), "s.ends_with(&suffix)");

        let split_default = lower_method(&Type::Str, "split", "s", &[]).expect("split default");
        assert!(render_expr(&split_default.expr).contains("split_whitespace"));

        let split_sep =
            lower_method(&Type::Str, "split", "s", &["sep".to_string()]).expect("split sep");
        assert_eq!(
            render_expr(&split_sep.expr),
            "s.split(&sep).map(::std::string::ToString::to_string).collect::<Vec<String>>()"
        );

        let replace = lower_method(
            &Type::Str,
            "replace",
            "s",
            &["old".to_string(), "new".to_string()],
        )
        .expect("replace lowers");
        assert_eq!(render_expr(&replace.expr), "s.replace(&old, &new)");

        let find =
            lower_method(&Type::Str, "find", "s", &["needle".to_string()]).expect("find lowers");
        assert_eq!(
            render_expr(&find.expr),
            "s.find(&needle).map(|i| SifrInt::from(i))"
        );

        let rfind =
            lower_method(&Type::Str, "rfind", "s", &["needle".to_string()]).expect("rfind lowers");
        assert_eq!(
            render_expr(&rfind.expr),
            "s.rfind(&needle).map(|i| SifrInt::from(i))"
        );

        let lstrip = lower_method(&Type::Str, "lstrip", "s", &[]).expect("lstrip lowers");
        assert_eq!(render_expr(&lstrip.expr), "s.trim_start().to_string()");

        let rstrip = lower_method(&Type::Str, "rstrip", "s", &[]).expect("rstrip lowers");
        assert_eq!(render_expr(&rstrip.expr), "s.trim_end().to_string()");

        let count =
            lower_method(&Type::Str, "count", "s", &["needle".to_string()]).expect("count lowers");
        assert_eq!(
            render_expr(&count.expr),
            "SifrInt::from(s.matches(&needle).count())"
        );

        let join =
            lower_method(&Type::Str, "join", "sep", &["parts".to_string()]).expect("join lowers");
        assert_eq!(render_expr(&join.expr), "parts.join(&sep)");

        let title = lower_method(&Type::Str, "title", "s", &[]).expect("title lowers");
        assert!(render_expr(&title.expr).contains("split_whitespace"));

        let cap = lower_method(&Type::Str, "capitalize", "s", &[]).expect("capitalize lowers");
        assert!(render_expr(&cap.expr).contains("let _s = s.clone();"));

        let swap = lower_method(&Type::Str, "swapcase", "s", &[]).expect("swapcase lowers");
        assert!(render_expr(&swap.expr).contains("is_uppercase"));

        let isdigit = lower_method(&Type::Str, "isdigit", "s", &[]).expect("isdigit lowers");
        assert!(render_expr(&isdigit.expr).contains("|c| c.is_ascii_digit()"));

        let isalpha = lower_method(&Type::Str, "isalpha", "s", &[]).expect("isalpha lowers");
        assert!(render_expr(&isalpha.expr).contains(".all(char::is_alphabetic)"));

        let isalnum = lower_method(&Type::Str, "isalnum", "s", &[]).expect("isalnum lowers");
        assert!(render_expr(&isalnum.expr).contains(".all(char::is_alphanumeric)"));

        let isspace = lower_method(&Type::Str, "isspace", "s", &[]).expect("isspace lowers");
        assert!(render_expr(&isspace.expr).contains(".all(char::is_whitespace)"));

        let isupper = lower_method(&Type::Str, "isupper", "s", &[]).expect("isupper lowers");
        assert!(render_expr(&isupper.expr).contains("is_uppercase"));

        let islower = lower_method(&Type::Str, "islower", "s", &[]).expect("islower lowers");
        assert!(render_expr(&islower.expr).contains("is_lowercase"));

        let center = lower_method(
            &Type::Str,
            "center",
            "s",
            &["SifrInt::from_i64(5)".to_string()],
        )
        .expect("center lowers");
        let center_rendered = render_expr(&center.expr);
        assert!(center_rendered.contains("sifr_runtime::checked_center"));
        assert!(center_rendered.contains("map_err"));
        assert!(center_rendered.contains("OverflowError::new"));
        assert!(!center_rendered.contains("to_usize"));

        let ljust = lower_method(
            &Type::Str,
            "ljust",
            "s",
            &["SifrInt::from_i64(5)".to_string()],
        )
        .expect("ljust lowers");
        let ljust_rendered = render_expr(&ljust.expr);
        assert!(ljust_rendered.contains("sifr_runtime::checked_ljust"));
        assert!(ljust_rendered.contains("OverflowError::new"));
        assert!(!ljust_rendered.contains("to_usize"));

        let rjust = lower_method(
            &Type::Str,
            "rjust",
            "s",
            &["SifrInt::from_i64(5)".to_string()],
        )
        .expect("rjust lowers");
        let rjust_rendered = render_expr(&rjust.expr);
        assert!(rjust_rendered.contains("sifr_runtime::checked_rjust"));
        assert!(rjust_rendered.contains("OverflowError::new"));
        assert!(!rjust_rendered.contains("to_usize"));

        let zfill = lower_method(
            &Type::Str,
            "zfill",
            "s",
            &["SifrInt::from_i64(5)".to_string()],
        )
        .expect("zfill lowers");
        let zfill_rendered = render_expr(&zfill.expr);
        assert!(zfill_rendered.contains("sifr_runtime::checked_zfill"));
        assert!(zfill_rendered.contains("OverflowError::new"));
        assert!(!zfill_rendered.contains("to_usize"));

        let list_clear = lower_method(&Type::List(Box::new(Type::Int)), "clear", "xs", &[])
            .expect("list clear lowers");
        assert_eq!(render_expr(&list_clear.expr), "xs.clear()");

        let list_append = lower_method(
            &Type::List(Box::new(Type::Int)),
            "append",
            "xs",
            &["1".to_string()],
        )
        .expect("list append lowers");
        assert_eq!(render_expr(&list_append.expr), "xs.push(1)");

        let list_extend = lower_method(
            &Type::List(Box::new(Type::Int)),
            "extend",
            "xs",
            &["ys".to_string()],
        )
        .expect("list extend lowers");
        assert_eq!(render_expr(&list_extend.expr), "xs.extend(ys)");

        let list_insert = lower_method(
            &Type::List(Box::new(Type::Int)),
            "insert",
            "xs",
            &["SifrInt::from_i64(0)".to_string(), "1".to_string()],
        )
        .expect("list insert lowers");
        assert_eq!(
            render_expr(&list_insert.expr),
            "xs.insert(SifrInt::from_i64(0).clamp_slice_bound(xs.len()), 1)"
        );

        let list_copy = lower_method(&Type::List(Box::new(Type::Int)), "copy", "xs", &[])
            .expect("list copy lowers");
        assert_eq!(render_expr(&list_copy.expr), "xs.clone()");

        let list_reverse = lower_method(&Type::List(Box::new(Type::Int)), "reverse", "xs", &[])
            .expect("list reverse lowers");
        assert_eq!(render_expr(&list_reverse.expr), "xs.reverse()");

        let list_sort = lower_method(&Type::List(Box::new(Type::Int)), "sort", "xs", &[])
            .expect("list sort lowers");
        assert_eq!(render_expr(&list_sort.expr), "xs.sort()");

        let list_count = lower_method(
            &Type::List(Box::new(Type::Int)),
            "count",
            "xs",
            &["1".to_string()],
        )
        .expect("list count lowers");
        assert_eq!(
            render_expr(&list_count.expr),
            "SifrInt::from(xs.iter().filter(|x| **x == 1).count())"
        );

        let list_contains = lower_method(
            &Type::List(Box::new(Type::Int)),
            "contains",
            "xs",
            &["1".to_string()],
        )
        .expect("list contains lowers");
        assert_eq!(render_expr(&list_contains.expr), "xs.contains(&1)");

        let list_pop = lower_method(&Type::List(Box::new(Type::Int)), "pop", "xs", &[])
            .expect("list pop lowers");
        assert_eq!(render_expr(&list_pop.expr), "xs.pop()");

        let deque_append = lower_method_with_context(
            &Type::List(Box::new(Type::Int)),
            "append",
            "dq",
            &["1".to_string()],
            true,
        )
        .expect("deque append lowers");
        assert_eq!(render_expr(&deque_append.expr), "dq.push_back(1)");

        let deque_appendleft = lower_method_with_context(
            &Type::List(Box::new(Type::Int)),
            "appendleft",
            "dq",
            &["1".to_string()],
            true,
        )
        .expect("deque appendleft lowers");
        assert_eq!(render_expr(&deque_appendleft.expr), "dq.push_front(1)");

        let deque_pop =
            lower_method_with_context(&Type::List(Box::new(Type::Int)), "pop", "dq", &[], true)
                .expect("deque pop lowers");
        assert_eq!(render_expr(&deque_pop.expr), "dq.pop_back()");

        let deque_popleft =
            lower_method_with_context(&Type::List(Box::new(Type::Int)), "popleft", "dq", &[], true)
                .expect("deque popleft lowers");
        assert_eq!(render_expr(&deque_popleft.expr), "dq.pop_front()");

        let list_remove = lower_method(
            &Type::List(Box::new(Type::Int)),
            "remove",
            "xs",
            &["SifrInt::from_i64(1)".to_string()],
        )
        .expect("list remove lowers");
        let list_remove_rendered = render_expr(&list_remove.expr);
        assert!(
            list_remove_rendered.contains(
                "if let Some(__pos) = xs.iter().position(|__x| __x.eq(&SifrInt::from_i64(1)))"
            ),
            "{list_remove_rendered}"
        );
        assert!(list_remove_rendered.contains("xs.remove(__pos);"));

        let list_index = lower_method(
            &Type::List(Box::new(Type::Int)),
            "index",
            "xs",
            &["1".to_string()],
        )
        .expect("list index lowers");
        let list_index_rendered = render_expr(&list_index.expr);
        assert!(
            list_index_rendered.contains("let __result = None;")
                || list_index_rendered.contains("let mut __result = None;")
        );
        assert!(list_index_rendered.contains("xs.get(__i)"));

        let dict_ty = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
        let dict_keys = lower_method(&dict_ty, "keys", "d", &[]).expect("dict keys lowers");
        assert_eq!(
            render_expr(&dict_keys.expr),
            "d.keys().cloned().collect::<Vec<_>>()"
        );

        let dict_values = lower_method(&dict_ty, "values", "d", &[]).expect("dict values lowers");
        assert_eq!(
            render_expr(&dict_values.expr),
            "d.values().cloned().collect::<Vec<_>>()"
        );

        let dict_items = lower_method(&dict_ty, "items", "d", &[]).expect("dict items lowers");
        assert_eq!(
            render_expr(&dict_items.expr),
            "d.iter().map(|__kv| (__kv.0.clone(), __kv.1.clone())).collect::<Vec<_>>()"
        );

        let dict_update = lower_method(&dict_ty, "update", "d", &["other".to_string()])
            .expect("dict update lowers");
        assert_eq!(render_expr(&dict_update.expr), "d.extend(other)");

        let dict_update_empty =
            lower_method(&dict_ty, "update", "d", &[]).expect("empty update lowers");
        assert_eq!(render_expr(&dict_update_empty.expr), "()");

        let dict_clear = lower_method(&dict_ty, "clear", "d", &[]).expect("dict clear lowers");
        assert_eq!(render_expr(&dict_clear.expr), "d.clear()");

        let dict_copy = lower_method(&dict_ty, "copy", "d", &[]).expect("dict copy lowers");
        assert_eq!(render_expr(&dict_copy.expr), "d.clone()");

        let dict_contains_lit = lower_method(&dict_ty, "contains", "d", &["\"k\"".to_string()])
            .expect("dict contains literal lowers");
        assert_eq!(
            render_expr(&dict_contains_lit.expr),
            "d.contains_key(&\"k\")"
        );

        let dict_contains_name = lower_method(&dict_ty, "contains", "d", &["k".to_string()])
            .expect("dict contains name lowers");
        assert_eq!(render_expr(&dict_contains_name.expr), "d.contains_key(&k)");

        let dict_get_one = lower_method(&dict_ty, "get", "d", &["\"k\"".to_string()])
            .expect("dict get one lowers");
        assert_eq!(render_expr(&dict_get_one.expr), "d.get(&\"k\").cloned()");

        let dict_get_two = lower_method(
            &dict_ty,
            "get",
            "d",
            &["\"k\"".to_string(), "0".to_string()],
        )
        .expect("dict get default lowers");
        assert_eq!(
            render_expr(&dict_get_two.expr),
            "d.get(&\"k\").cloned().unwrap_or(0)"
        );

        let dict_pop =
            lower_method(&dict_ty, "pop", "d", &["\"k\"".to_string()]).expect("dict pop lowers");
        assert_eq!(render_expr(&dict_pop.expr), "d.remove(&\"k\")");

        let dict_pop_default = lower_method(
            &dict_ty,
            "pop",
            "d",
            &["\"k\"".to_string(), "0".to_string()],
        )
        .expect("dict pop default lowers");
        assert_eq!(
            render_expr(&dict_pop_default.expr),
            "d.remove(&\"k\").unwrap_or(0)"
        );

        let dict_setdefault = lower_method(
            &dict_ty,
            "setdefault",
            "d",
            &["\"k\"".to_string(), "0".to_string()],
        )
        .expect("dict setdefault lowers");
        assert_eq!(
            render_expr(&dict_setdefault.expr),
            "d.entry(\"k\".to_owned()).or_insert(0.clone()).clone()"
        );

        let set_ty = Type::Set(Box::new(Type::Int));
        let set_add =
            lower_method(&set_ty, "add", "s", &["1".to_string()]).expect("set add lowers");
        assert_eq!(render_expr(&set_add.expr), "s.insert(1)");

        let set_remove =
            lower_method(&set_ty, "remove", "s", &["1".to_string()]).expect("set remove lowers");
        assert_eq!(render_expr(&set_remove.expr), "s.remove(&1)");

        let set_discard =
            lower_method(&set_ty, "discard", "s", &["1".to_string()]).expect("set discard lowers");
        assert_eq!(render_expr(&set_discard.expr), "s.remove(&1)");

        let set_contains = lower_method(&set_ty, "contains", "s", &["1".to_string()])
            .expect("set contains lowers");
        assert_eq!(render_expr(&set_contains.expr), "s.contains(&1)");

        let set_clear = lower_method(&set_ty, "clear", "s", &[]).expect("set clear lowers");
        assert_eq!(render_expr(&set_clear.expr), "s.clear()");

        let set_copy = lower_method(&set_ty, "copy", "s", &[]).expect("set copy lowers");
        assert_eq!(render_expr(&set_copy.expr), "s.clone()");

        let set_subset = lower_method(&set_ty, "issubset", "s", &["other".to_string()])
            .expect("set issubset lowers");
        assert_eq!(render_expr(&set_subset.expr), "s.is_subset(&other)");

        let set_superset = lower_method(&set_ty, "issuperset", "s", &["other".to_string()])
            .expect("set issuperset lowers");
        assert_eq!(render_expr(&set_superset.expr), "s.is_superset(&other)");

        let set_disjoint = lower_method(&set_ty, "isdisjoint", "s", &["other".to_string()])
            .expect("set isdisjoint lowers");
        assert_eq!(render_expr(&set_disjoint.expr), "s.is_disjoint(&other)");

        let set_pop = lower_method(&set_ty, "pop", "s", &[]).expect("set pop lowers");
        assert!(render_expr(&set_pop.expr).contains("iter().next().cloned()"));

        let set_union =
            lower_method(&set_ty, "union", "s", &["other".to_string()]).expect("set union lowers");
        assert_eq!(
            render_expr(&set_union.expr),
            "s.r#union(&other).cloned().collect::<std::collections::HashSet<_>>()"
        );

        let set_intersection = lower_method(&set_ty, "intersection", "s", &["other".to_string()])
            .expect("set intersection lowers");
        assert_eq!(
            render_expr(&set_intersection.expr),
            "s.intersection(&other).cloned().collect::<std::collections::HashSet<_>>()"
        );

        let set_difference = lower_method(&set_ty, "difference", "s", &["other".to_string()])
            .expect("set difference lowers");
        assert_eq!(
            render_expr(&set_difference.expr),
            "s.difference(&other).cloned().collect::<std::collections::HashSet<_>>()"
        );

        let set_sdiff = lower_method(&set_ty, "symmetric_difference", "s", &["other".to_string()])
            .expect("set symmetric_difference lowers");
        assert_eq!(
            render_expr(&set_sdiff.expr),
            "s.symmetric_difference(&other).cloned().collect::<std::collections::HashSet<_>>()"
        );
    }

    #[test]
    fn lower_method_accepts_ir_inputs() {
        let ir = super::lower_method(
            &Type::List(Box::new(Type::Int)),
            "append",
            &RustExpr::Ident("xs".to_string()),
            &[RustExpr::Ident("v".to_string())],
        )
        .expect("ir append");
        assert_eq!(render_expr(&ir.expr), "xs.push(v)");
    }

    #[test]
    fn lower_method_supports_list_sort_with_reverse_flag() {
        let lowered = super::lower_method(
            &Type::List(Box::new(Type::Int)),
            "sort",
            &RustExpr::Ident("xs".to_string()),
            &[RustExpr::Ident("desc".to_string())],
        )
        .expect("list sort reverse lowers");
        let rendered = render_expr(&lowered.expr);
        assert!(rendered.contains("if desc"));
        assert!(rendered.contains("xs.sort_by"));
        assert!(rendered.contains("__right.cmp(__left)"));
        assert!(!rendered.contains("xs.reverse()"));
    }

    #[test]
    fn safe_collection_methods_flatten_optional_payload_absence() {
        let optional_string = Type::Union(vec![Type::Str, Type::None]);

        let list_pop = lower_method(
            &Type::List(Box::new(optional_string.clone())),
            "pop",
            "values",
            &[],
        )
        .expect("optional list pop lowers");
        assert_eq!(render_expr(&list_pop.expr), "(values.pop()).flatten()");

        let dict_get = lower_method(
            &Type::Dict(Box::new(Type::Str), Box::new(optional_string.clone())),
            "get",
            "values",
            &["key".to_string()],
        )
        .expect("optional dict get lowers");
        assert_eq!(
            render_expr(&dict_get.expr),
            "(values.get(&key).cloned()).flatten()"
        );

        let dict_pop = lower_method(
            &Type::Dict(Box::new(Type::Str), Box::new(optional_string)),
            "pop",
            "values",
            &["key".to_string()],
        )
        .expect("optional dict pop lowers");
        assert_eq!(
            render_expr(&dict_pop.expr),
            "(values.remove(&key)).flatten()"
        );
    }

    #[test]
    fn lowers_decimal_and_bigdecimal_methods_via_registry() {
        let decimal_quantize =
            lower_method(&Type::Decimal, "quantize", "d", &["scale".to_string()])
                .expect("decimal quantize lowers");
        assert!(render_expr(&decimal_quantize.expr).contains("round_dp_with_strategy"));

        let decimal_sqrt =
            lower_method(&Type::Decimal, "sqrt", "d", &[]).expect("decimal sqrt lowers");
        assert!(render_expr(&decimal_sqrt.expr).contains("map_or_else"));
        assert!(render_expr(&decimal_sqrt.expr).contains("DecimalConversionError"));

        let decimal_round =
            lower_method(&Type::Decimal, "round", "d", &[]).expect("decimal round lowers");
        assert!(render_expr(&decimal_round.expr).contains("round_dp_with_strategy"));

        let decimal_is_zero =
            lower_method(&Type::Decimal, "is_zero", "d", &[]).expect("decimal is_zero lowers");
        assert_eq!(render_expr(&decimal_is_zero.expr), "d.is_zero()");

        let decimal_is_finite =
            lower_method(&Type::Decimal, "is_finite", "d", &[]).expect("decimal is_finite lowers");
        assert_eq!(render_expr(&decimal_is_finite.expr), "true");

        let big_quantize =
            lower_method(&Type::BigDecimal, "quantize", "bd", &["digits".to_string()])
                .expect("bigdecimal quantize lowers");
        let big_quantize_rendered = render_expr(&big_quantize.expr);
        assert!(big_quantize_rendered.contains("round_decimal_ref"));
        assert!(big_quantize_rendered.contains("bigdecimal::Context::new"));
        assert!(big_quantize_rendered.contains("NonZeroU64::MIN.saturating_add(27)"));
        assert!(!big_quantize_rendered.contains("unwrap_or_else"));
        assert!(!big_quantize_rendered.contains("with_prec"));

        let big_sqrt =
            lower_method(&Type::BigDecimal, "sqrt", "bd", &[]).expect("bigdecimal sqrt lowers");
        let big_sqrt_rendered = render_expr(&big_sqrt.expr);
        assert!(big_sqrt_rendered.contains("sqrt_with_context"));
        assert!(big_sqrt_rendered.contains("DecimalConversionError"));
        assert!(big_sqrt_rendered.contains("bigdecimal::Context::new"));
        assert!(big_sqrt_rendered.contains("NonZeroU64::MIN.saturating_add(27)"));
        assert!(!big_sqrt_rendered.contains("unwrap_or_else"));
        assert!(!big_sqrt_rendered.contains("with_prec"));

        let big_round =
            lower_method(&Type::BigDecimal, "round", "bd", &[]).expect("bigdecimal round lowers");
        assert!(render_expr(&big_round.expr).contains("with_scale_round"));

        let big_is_zero =
            lower_method(&Type::BigDecimal, "is_zero", "bd", &[]).expect("bigdecimal is_zero");
        assert_eq!(render_expr(&big_is_zero.expr), "bd == BigDecimal::from(0)");

        let big_is_finite =
            lower_method(&Type::BigDecimal, "is_finite", "bd", &[]).expect("bigdecimal is_finite");
        assert_eq!(render_expr(&big_is_finite.expr), "true");
    }

    #[test]
    fn lowers_bytes_methods_with_u8_backend_boundaries() {
        let count = lower_method(&Type::Bytes, "count", "payload", &["needle".to_string()])
            .expect("bytes count lowers");
        let count_rendered = render_expr(&count.expr);
        assert!(count_rendered.contains("__needle.try_to_u8().map_or_else(|_|"));
        assert!(count_rendered.contains("::sifr_runtime::count_byte"));
        assert_eq!(count_rendered.matches("payload").count(), 1);

        let contains = lower_method(&Type::Bytes, "contains", "payload", &["needle".to_string()])
            .expect("bytes contains lowers");
        let contains_rendered = render_expr(&contains.expr);
        assert!(contains_rendered.contains("__needle.try_to_u8().map_or_else(|_|"));
        assert!(contains_rendered.contains("__bytes_receiver.contains(&__needle_u8)"));
        assert_eq!(contains_rendered.matches("payload").count(), 1);

        let find = lower_method(
            &Type::Bytes,
            "find",
            "payload",
            &["needle".to_string(), "0".to_string(), "5".to_string()],
        )
        .expect("bytes find lowers");
        let find_rendered = render_expr(&find.expr);
        assert!(find_rendered.contains("__needle.try_to_u8().map_or_else(|_|"));
        assert!(find_rendered.contains("__needle_u8"));
        assert!(find_rendered.contains("None"));
        assert!(find_rendered.contains("__result.is_none()"));
        assert_eq!(find_rendered.matches("payload").count(), 1);

        let startswith = lower_method(
            &Type::Bytes,
            "startswith",
            "payload",
            &["prefix".to_string()],
        )
        .expect("bytes startswith lowers");
        assert_eq!(
            render_expr(&startswith.expr),
            "payload.starts_with(&prefix)"
        );

        let endswith = lower_method(&Type::Bytes, "endswith", "payload", &["suffix".to_string()])
            .expect("bytes endswith lowers");
        assert_eq!(render_expr(&endswith.expr), "payload.ends_with(&suffix)");

        let hex = lower_method(&Type::Bytes, "hex", "payload", &[]).expect("bytes hex lowers");
        let hex_rendered = render_expr(&hex.expr);
        assert!(hex_rendered.contains("let __bytes_receiver: &[u8] = &payload;"));
        assert_eq!(hex_rendered.matches("payload").count(), 1);
        assert!(hex_rendered.contains("__bytes_receiver.len().saturating_mul(2_usize)"));
        assert!(hex_rendered.contains("for __byte in __bytes_receiver"));
        assert!(hex_rendered.contains(
            "::std::fmt::Write::write_fmt(&mut __hex, format_args!(\"{:02x}\", *__byte))"
        ));
        assert!(!hex_rendered.contains(".iter()"));

        let to_ints =
            lower_method(&Type::Bytes, "to_ints", "payload", &[]).expect("bytes to_ints lowers");
        let to_ints_rendered = render_expr(&to_ints.expr);
        assert!(to_ints_rendered.contains("collect::<Vec<SifrInt>>()"));
        assert!(to_ints_rendered.contains("SifrInt::from(*__byte)"));
    }
}
