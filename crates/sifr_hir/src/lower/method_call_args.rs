use crate::hir_nodes::HirExpr;
use sifr_python_ast::ExprCall;
use sifr_type_system::{make_union, FunctionType, Type};

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
        Type::List(_) => normalize_list_method_args(method, positional, &mut keywords, ctx)?,
        Type::Dict(_, _) => normalize_dict_method_args(method, positional, &mut keywords, ctx)?,
        Type::Set(_) => normalize_set_method_args(method, positional, &mut keywords, ctx)?,
        Type::Tuple(_) => normalize_tuple_method_args(method, positional, &mut keywords, ctx)?,
        Type::Str => normalize_string_method_args(method, positional, &mut keywords, ctx)?,
        _ => {
            reject_remaining_keywords(method, &keywords, ctx)?;
            positional
        }
    };
    reject_remaining_keywords(method, &keywords, ctx)?;
    Some(args)
}

pub(super) fn lower_signature_call_args(
    call: &ExprCall,
    callable_name: &str,
    ft: &FunctionType,
    defaults: Option<&[(usize, HirExpr)]>,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    lower_function_call_args(call, callable_name, ft, defaults, None, ctx)
}

pub(super) fn lower_function_call_args(
    call: &ExprCall,
    callable_name: &str,
    ft: &FunctionType,
    defaults: Option<&[(usize, HirExpr)]>,
    vararg_index: Option<usize>,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    let positional_args = lower_positional_args(call, ctx)?;
    let keyword_args = lower_keyword_args(call, callable_name, ctx)?;

    if let Some(vararg_index) = vararg_index {
        return lower_vararg_function_call_args(
            callable_name,
            ft,
            defaults,
            vararg_index,
            &positional_args,
            &keyword_args,
            ctx,
        );
    }

    if keyword_args.is_empty() {
        if positional_args.len() > ft.params.len() {
            ctx.error(format!(
                "{callable_name}() takes at most {} argument(s), got {}",
                ft.params.len(),
                positional_args.len()
            ));
            return None;
        }
        if positional_args.len() < ft.params.len() {
            let mut filled = positional_args;
            for i in filled.len()..ft.params.len() {
                if let Some(default_expr) = defaults
                    .and_then(|defs| defs.iter().find(|(idx, _)| *idx == i).map(|(_, expr)| expr))
                {
                    filled.push(default_expr.clone());
                } else {
                    ctx.error(format!(
                        "{callable_name}(): missing argument '{}' with no default value",
                        ft.params[i].0
                    ));
                    return None;
                }
            }
            return Some(filled);
        }
        return Some(positional_args);
    }

    let mut resolved = Vec::with_capacity(ft.params.len());
    for (i, (param_name, _, _)) in ft.params.iter().enumerate() {
        if i < positional_args.len() {
            if keyword_args
                .iter()
                .any(|(keyword, _)| keyword == param_name)
            {
                ctx.error(format!(
                    "{callable_name}() got multiple values for argument '{param_name}'"
                ));
                return None;
            }
            resolved.push(positional_args[i].clone());
            continue;
        }
        if let Some(position) = keyword_args
            .iter()
            .position(|(keyword, _)| keyword == param_name)
        {
            resolved.push(keyword_args[position].1.clone());
            continue;
        }
        if let Some(default_expr) =
            defaults.and_then(|defs| defs.iter().find(|(idx, _)| *idx == i).map(|(_, expr)| expr))
        {
            resolved.push(default_expr.clone());
            continue;
        }
        ctx.error(format!(
            "{callable_name}(): missing argument '{param_name}' with no default value"
        ));
        return None;
    }

    for (keyword, _) in keyword_args {
        if !ft
            .params
            .iter()
            .any(|(param_name, _, _)| param_name == keyword.as_str())
        {
            ctx.error(format!(
                "{callable_name}() got an unexpected keyword argument '{keyword}'"
            ));
            return None;
        }
    }

    Some(resolved)
}

fn lower_vararg_function_call_args(
    callable_name: &str,
    ft: &FunctionType,
    defaults: Option<&[(usize, HirExpr)]>,
    vararg_index: usize,
    positional_args: &[HirExpr],
    keyword_args: &LoweredKeywords,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    let mut resolved = Vec::with_capacity(ft.params.len());
    let mut used_kwargs: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (i, (param_name, _, _)) in ft.params.iter().take(vararg_index).enumerate() {
        if i < positional_args.len() {
            if keyword_args
                .iter()
                .any(|(keyword, _)| keyword == param_name)
            {
                ctx.error(format!(
                    "{callable_name}() got multiple values for argument '{param_name}'"
                ));
                return None;
            }
            resolved.push(positional_args[i].clone());
            continue;
        }
        if let Some(position) = keyword_args
            .iter()
            .position(|(keyword, _)| keyword == param_name)
        {
            resolved.push(keyword_args[position].1.clone());
            used_kwargs.insert(param_name.clone());
            continue;
        }
        if let Some(default_expr) =
            defaults.and_then(|defs| defs.iter().find(|(idx, _)| *idx == i).map(|(_, expr)| expr))
        {
            resolved.push(default_expr.clone());
            continue;
        }
        ctx.error(format!(
            "{callable_name}(): missing argument '{param_name}' with no default value"
        ));
        return None;
    }

    let vararg_elements = if positional_args.len() > vararg_index {
        positional_args[vararg_index..].to_vec()
    } else {
        Vec::new()
    };
    let vararg_ty = ft.params.get(vararg_index).map(|(_, ty, _)| ty.clone());
    let vararg_elem_ty = if vararg_elements.is_empty() {
        match vararg_ty {
            Some(Type::List(elem_ty)) => *elem_ty,
            Some(_) | None => Type::Any,
        }
    } else {
        make_union(
            vararg_elements
                .iter()
                .map(|element| element.ty().clone())
                .collect(),
        )
    };
    resolved.push(HirExpr::ListLiteral {
        elements: vararg_elements,
        ty: Type::List(Box::new(vararg_elem_ty)),
    });

    for (i, (param_name, _, _)) in ft.params.iter().enumerate().skip(vararg_index + 1) {
        if let Some(position) = keyword_args
            .iter()
            .position(|(keyword, _)| keyword == param_name)
        {
            resolved.push(keyword_args[position].1.clone());
            used_kwargs.insert(param_name.clone());
            continue;
        }
        if let Some(default_expr) =
            defaults.and_then(|defs| defs.iter().find(|(idx, _)| *idx == i).map(|(_, expr)| expr))
        {
            resolved.push(default_expr.clone());
            continue;
        }
        ctx.error(format!(
            "{callable_name}(): missing argument '{param_name}' with no default value"
        ));
        return None;
    }

    let vararg_name = &ft.params[vararg_index].0;
    for (keyword, _) in keyword_args {
        if keyword == vararg_name {
            ctx.error(format!(
                "{callable_name}() got an unexpected keyword argument '{keyword}'"
            ));
            return None;
        }
        if !used_kwargs.contains(keyword)
            && !ft
                .params
                .iter()
                .take(vararg_index)
                .chain(ft.params.iter().skip(vararg_index + 1))
                .any(|(param_name, _, _)| param_name == keyword)
        {
            ctx.error(format!(
                "{callable_name}() got an unexpected keyword argument '{keyword}'"
            ));
            return None;
        }
    }

    Some(resolved)
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

fn duplicate_argument_error(method: &str, arg: &str, ctx: &mut LowerCtx) -> Option<Vec<HirExpr>> {
    ctx.error(format!(
        "{method}() got multiple values for argument '{arg}'"
    ));
    None
}

fn append_start_stop_args(
    method: &str,
    mut positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    if let Some(start) = take_keyword(keywords, "start") {
        if positional.len() > 1 {
            return duplicate_argument_error(method, "start", ctx);
        }
        positional.push(start);
    }
    if let Some(stop) = take_keyword(keywords, "stop") {
        if positional.len() > 2 {
            return duplicate_argument_error(method, "stop", ctx);
        }
        if positional.len() == 1 {
            positional.push(HirExpr::IntLiteral(0));
        }
        positional.push(stop);
    }
    Some(positional)
}

fn normalize_list_method_args(
    method: &str,
    positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    match method {
        "index" => append_start_stop_args(method, positional, keywords, ctx),
        _ => Some(positional),
    }
}

fn normalize_dict_method_args(
    method: &str,
    positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    match method {
        "get" | "pop" => {
            let mut args = positional;
            if let Some(default) = take_keyword(keywords, "default") {
                if args.len() > 1 {
                    return duplicate_argument_error(method, "default", ctx);
                }
                if args.len() == 1 {
                    args.push(default);
                }
            }
            Some(args)
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
            Some(args)
        }
        _ => Some(positional),
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
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    match method {
        "index" => append_start_stop_args(method, positional, keywords, ctx),
        _ => Some(positional),
    }
}

fn normalize_string_method_args(
    method: &str,
    positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    match method {
        "split" => {
            let mut args = positional;
            if let Some(sep) = take_keyword(keywords, "sep") {
                if !args.is_empty() {
                    return duplicate_argument_error(method, "sep", ctx);
                }
                if args.is_empty() {
                    args.push(sep);
                }
            }
            if let Some(maxsplit) = take_keyword(keywords, "maxsplit") {
                if args.len() > 1 {
                    return duplicate_argument_error(method, "maxsplit", ctx);
                }
                if args.is_empty() {
                    args.push(HirExpr::NoneLiteral);
                }
                args.push(maxsplit);
            }
            Some(args)
        }
        "replace" => {
            let mut args = positional;
            if let Some(count) = take_keyword(keywords, "count") {
                if args.len() > 2 {
                    return duplicate_argument_error(method, "count", ctx);
                }
                args.push(count);
            }
            Some(args)
        }
        _ => Some(positional),
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
