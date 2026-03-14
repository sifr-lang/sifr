use crate::hir_nodes::HirExpr;
use sifr_python_ast::ExprCall;
use sifr_type_system::Type;

use super::builtin_calls::{callable_builtin_dict_output_type, callable_builtin_element_type};
use super::expressions::lower_expr;
use super::LowerCtx;

type LoweredKeywords = Vec<(String, HirExpr)>;

pub(super) fn lower_method_call_args(
    object_ty: &Type,
    method: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    let positional = lower_positional_args(call, ctx)?;
    let mut keywords = lower_keyword_args(call, method, ctx)?;
    let resolved = object_ty.resolve_alias();
    let args = match resolved {
        Type::List(_) => normalize_list_method_args(method, positional, &mut keywords),
        Type::Dict(_, _) => normalize_dict_method_args(method, positional, &mut keywords),
        Type::Set(_) => normalize_set_method_args(method, positional, &mut keywords, ctx)?,
        Type::Tuple(_) => normalize_tuple_method_args(method, positional, &mut keywords),
        Type::Str => normalize_string_method_args(method, positional, &mut keywords),
        _ => {
            reject_remaining_keywords(method, &keywords, ctx)?;
            positional
        }
    };
    reject_remaining_keywords(method, &keywords, ctx)?;
    Some(args)
}

fn lower_positional_args(call: &ExprCall, ctx: &mut LowerCtx) -> Option<Vec<HirExpr>> {
    let mut args = Vec::with_capacity(call.arguments.args.len());
    for arg in &call.arguments.args {
        args.push(lower_expr(arg, ctx)?);
    }
    Some(args)
}

fn lower_keyword_args(
    call: &ExprCall,
    method: &str,
    ctx: &mut LowerCtx,
) -> Option<LoweredKeywords> {
    let mut keywords = Vec::with_capacity(call.arguments.keywords.len());
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            ctx.error(format!(
                "{method}() does not support unpacked keyword arguments"
            ));
            return None;
        };
        if keywords.iter().any(|(seen, _)| seen == name.as_str()) {
            ctx.error(format!(
                "{method}() got multiple values for keyword argument '{name}'"
            ));
            return None;
        }
        keywords.push((name.to_string(), lower_expr(&keyword.value, ctx)?));
    }
    Some(keywords)
}

fn take_keyword(keywords: &mut LoweredKeywords, name: &str) -> Option<HirExpr> {
    let index = keywords.iter().position(|(keyword, _)| keyword == name)?;
    Some(keywords.remove(index).1)
}

fn reject_remaining_keywords(
    method: &str,
    keywords: &[(String, HirExpr)],
    ctx: &mut LowerCtx,
) -> Option<()> {
    if let Some((keyword, _)) = keywords.first() {
        ctx.error(format!(
            "{method}() got an unexpected keyword argument '{keyword}'"
        ));
        return None;
    }
    Some(())
}

fn append_start_stop_args(
    mut positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
) -> Vec<HirExpr> {
    if let Some(start) = take_keyword(keywords, "start") {
        positional.push(start);
    }
    if let Some(stop) = take_keyword(keywords, "stop") {
        if positional.len() == 1 {
            positional.push(HirExpr::IntLiteral(0));
        }
        positional.push(stop);
    }
    positional
}

fn normalize_list_method_args(
    method: &str,
    positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
) -> Vec<HirExpr> {
    match method {
        "index" => append_start_stop_args(positional, keywords),
        _ => positional,
    }
}

fn normalize_dict_method_args(
    method: &str,
    positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
) -> Vec<HirExpr> {
    match method {
        "get" | "pop" => {
            let mut args = positional;
            if let Some(default) = take_keyword(keywords, "default") {
                if args.len() == 1 {
                    args.push(default);
                }
            }
            args
        }
        "update" => {
            let mut args = positional;
            if !keywords.is_empty() {
                let mut keys = Vec::with_capacity(keywords.len());
                let mut values = Vec::with_capacity(keywords.len());
                for (name, value) in keywords.drain(..) {
                    keys.push(HirExpr::StringLiteral(name));
                    values.push(value);
                }
                args.push(HirExpr::DictLiteral {
                    keys,
                    values: values.clone(),
                    ty: Type::Dict(
                        Box::new(Type::Str),
                        Box::new(if values.is_empty() {
                            Type::Any
                        } else {
                            sifr_type_system::make_union(
                                values.into_iter().map(|value| value.ty().clone()).collect(),
                            )
                        }),
                    ),
                });
            }
            args
        }
        _ => positional,
    }
}

fn normalize_set_method_args(
    method: &str,
    positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    reject_remaining_keywords(method, keywords, ctx)?;
    Some(positional)
}

fn normalize_tuple_method_args(
    method: &str,
    positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
) -> Vec<HirExpr> {
    match method {
        "index" => append_start_stop_args(positional, keywords),
        _ => positional,
    }
}

fn normalize_string_method_args(
    method: &str,
    positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
) -> Vec<HirExpr> {
    match method {
        "split" => {
            let mut args = positional;
            if let Some(sep) = take_keyword(keywords, "sep") {
                if args.is_empty() {
                    args.push(sep);
                }
            }
            if let Some(maxsplit) = take_keyword(keywords, "maxsplit") {
                if args.is_empty() {
                    args.push(HirExpr::NoneLiteral);
                }
                args.push(maxsplit);
            }
            args
        }
        "replace" => {
            let mut args = positional;
            if let Some(count) = take_keyword(keywords, "count") {
                args.push(count);
            }
            args
        }
        _ => positional,
    }
}

pub(super) fn validate_list_extend_arg(
    list_elem_ty: &Type,
    iterable_ty: &Type,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(iterable_elem_ty) = callable_builtin_element_type(iterable_ty) else {
        ctx.error(format!(
            "list.extend() argument must be an iterable with a statically-known element type, got '{}'",
            iterable_ty.display_name()
        ));
        return false;
    };
    if !iterable_elem_ty.is_assignable_to(list_elem_ty) {
        ctx.error(format!(
            "list.extend() iterable element type '{}' is not compatible with list element type '{}'",
            iterable_elem_ty.display_name(),
            list_elem_ty.display_name()
        ));
        return false;
    }
    true
}

pub(super) fn validate_dict_update_arg(
    key_ty: &Type,
    value_ty: &Type,
    update_ty: &Type,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(Type::Dict(update_key_ty, update_value_ty)) =
        callable_builtin_dict_output_type(update_ty)
    else {
        ctx.error(format!(
            "dict.update() argument must be a dict or iterable of key/value tuples, got '{}'",
            update_ty.display_name()
        ));
        return false;
    };
    let mut valid = true;
    if !update_key_ty.is_assignable_to(key_ty) {
        ctx.error(format!(
            "dict.update() key type '{}' is not compatible with dict key type '{}'",
            update_key_ty.display_name(),
            key_ty.display_name()
        ));
        valid = false;
    }
    if !update_value_ty.is_assignable_to(value_ty) {
        ctx.error(format!(
            "dict.update() value type '{}' is not compatible with dict value type '{}'",
            update_value_ty.display_name(),
            value_ty.display_name()
        ));
        valid = false;
    }
    valid
}

pub(super) fn validate_set_iterable_arg(
    set_elem_ty: &Type,
    iterable_ty: &Type,
    method: &str,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(iterable_elem_ty) = callable_builtin_element_type(iterable_ty) else {
        ctx.error(format!(
            "set.{method}() arguments must be iterables with a statically-known element type, got '{}'",
            iterable_ty.display_name()
        ));
        return false;
    };
    if !iterable_elem_ty.is_assignable_to(set_elem_ty) {
        ctx.error(format!(
            "set.{method}() iterable element type '{}' is not compatible with set element type '{}'",
            iterable_elem_ty.display_name(),
            set_elem_ty.display_name()
        ));
        return false;
    }
    true
}
