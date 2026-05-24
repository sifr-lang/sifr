use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::ExprCall;
use sifr_type_system::{make_union, FunctionType, Type};

use super::builtin_calls::{callable_builtin_dict_output_type, callable_builtin_element_type};
use super::expressions::lower_expr;
use super::LowerCtx;

struct LoweredKeyword {
    name: String,
    value: HirExpr,
    name_range: TextRange,
}

type LoweredKeywords = Vec<LoweredKeyword>;

struct VarargCallArgs<'a> {
    callable_name: &'a str,
    ft: &'a FunctionType,
    defaults: Option<&'a [(usize, HirExpr)]>,
    vararg_index: usize,
    positional_args: &'a [HirExpr],
    keyword_args: &'a LoweredKeywords,
    missing_range: TextRange,
}

pub(in crate::lower) fn lower_method_call_args(
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

pub(in crate::lower) fn resolved_method_arg_ranges(
    object_ty: &Type,
    method: &str,
    call: &ExprCall,
) -> Vec<TextRange> {
    let mut ranges: Vec<TextRange> = call.arguments.args.iter().map(Ranged::range).collect();
    match object_ty.resolve_alias() {
        Type::List(_) if method == "sort" => {
            if let Some(reverse) = call
                .arguments
                .keywords
                .iter()
                .find(|keyword| keyword.arg.as_ref().is_some_and(|name| name == "reverse"))
            {
                if ranges.is_empty() {
                    ranges.push(reverse.value.range());
                }
            }
        }
        Type::Dict(_, _) if matches!(method, "get" | "pop" | "setdefault") => {
            if let Some(default) = call
                .arguments
                .keywords
                .iter()
                .find(|keyword| keyword.arg.as_ref().is_some_and(|name| name == "default"))
            {
                if ranges.len() == 1 {
                    ranges.push(default.value.range());
                }
            }
        }
        Type::Dict(_, _) if method == "update" => {
            ranges.extend(call.arguments.keywords.iter().take(1).map(Ranged::range));
        }
        Type::Str if method == "split" => {
            if let Some(sep) = call
                .arguments
                .keywords
                .iter()
                .find(|keyword| keyword.arg.as_ref().is_some_and(|name| name == "sep"))
            {
                if ranges.is_empty() {
                    ranges.push(sep.value.range());
                }
            }
            if let Some(maxsplit) = call
                .arguments
                .keywords
                .iter()
                .find(|keyword| keyword.arg.as_ref().is_some_and(|name| name == "maxsplit"))
            {
                if ranges.is_empty() {
                    ranges.push(call.func.range());
                }
                if ranges.len() == 1 {
                    ranges.push(maxsplit.value.range());
                }
            }
        }
        Type::Str if method == "replace" => {
            if let Some(count) = call
                .arguments
                .keywords
                .iter()
                .find(|keyword| keyword.arg.as_ref().is_some_and(|name| name == "count"))
            {
                if ranges.len() <= 2 {
                    ranges.push(count.value.range());
                }
            }
        }
        _ => {}
    }
    ranges
}

pub(in crate::lower) fn lower_signature_call_args(
    call: &ExprCall,
    callable_name: &str,
    ft: &FunctionType,
    defaults: Option<&[(usize, HirExpr)]>,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    lower_function_call_args(call, callable_name, ft, defaults, None, ctx)
}

pub(in crate::lower) fn lower_function_call_args(
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
            &VarargCallArgs {
                callable_name,
                ft,
                defaults,
                vararg_index,
                positional_args: &positional_args,
                keyword_args: &keyword_args,
                missing_range: call.func.range(),
            },
            ctx,
        );
    }

    if keyword_args.is_empty() {
        if positional_args.len() > ft.params.len() {
            let expected_count = ft.params.len();
            let actual_count = positional_args.len();
            ctx.error_with_code_at(
                DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
                format!(
                    "{callable_name}() takes at most {expected_count} argument(s), got {actual_count}"
                ),
                call.arguments.args[expected_count].range(),
            );
            return None;
        }
        if positional_args.len() < ft.params.len() {
            let mut filled = positional_args;
            for i in filled.len()..ft.params.len() {
                if let Some(default_expr) = default_arg_expr(defaults, i) {
                    filled.push(default_expr.clone());
                } else {
                    return missing_argument_error(
                        callable_name,
                        &ft.params[i].0,
                        ctx,
                        call.func.range(),
                    );
                }
            }
            return Some(filled);
        }
        return Some(positional_args);
    }

    let mut resolved = Vec::with_capacity(ft.params.len());
    for (i, (param_name, _, _)) in ft.params.iter().enumerate() {
        if i < positional_args.len() {
            if let Some(keyword) = keyword_arg(&keyword_args, param_name) {
                return duplicate_argument_error(
                    callable_name,
                    param_name,
                    ctx,
                    keyword.name_range,
                );
            }
            resolved.push(positional_args[i].clone());
            continue;
        }
        if let Some(keyword) = keyword_arg(&keyword_args, param_name) {
            resolved.push(keyword.value.clone());
            continue;
        }
        if let Some(default_expr) = default_arg_expr(defaults, i) {
            resolved.push(default_expr.clone());
            continue;
        }
        return missing_argument_error(callable_name, param_name, ctx, call.func.range());
    }

    for keyword in keyword_args {
        if !ft
            .params
            .iter()
            .any(|(param_name, _, _)| param_name == keyword.name.as_str())
        {
            return unexpected_keyword_error(callable_name, &keyword.name, ctx, keyword.name_range);
        }
    }

    Some(resolved)
}

fn lower_vararg_function_call_args(
    args: &VarargCallArgs<'_>,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    let callable_name = args.callable_name;
    let ft = args.ft;
    let defaults = args.defaults;
    let vararg_index = args.vararg_index;
    let positional_args = args.positional_args;
    let keyword_args = args.keyword_args;
    let missing_range = args.missing_range;
    let mut resolved = Vec::with_capacity(ft.params.len());
    let mut used_kwargs: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (i, (param_name, _, _)) in ft.params.iter().take(vararg_index).enumerate() {
        if i < positional_args.len() {
            if let Some(keyword) = keyword_arg(keyword_args, param_name) {
                return duplicate_argument_error(
                    callable_name,
                    param_name,
                    ctx,
                    keyword.name_range,
                );
            }
            resolved.push(positional_args[i].clone());
            continue;
        }
        if let Some(keyword) = keyword_arg(keyword_args, param_name) {
            resolved.push(keyword.value.clone());
            used_kwargs.insert(param_name.clone());
            continue;
        }
        if let Some(default_expr) = default_arg_expr(defaults, i) {
            resolved.push(default_expr.clone());
            continue;
        }
        return missing_argument_error(callable_name, param_name, ctx, missing_range);
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
        if let Some(keyword) = keyword_arg(keyword_args, param_name) {
            resolved.push(keyword.value.clone());
            used_kwargs.insert(param_name.clone());
            continue;
        }
        if let Some(default_expr) = default_arg_expr(defaults, i) {
            resolved.push(default_expr.clone());
            continue;
        }
        return missing_argument_error(callable_name, param_name, ctx, missing_range);
    }

    let vararg_name = &ft.params[vararg_index].0;
    for keyword in keyword_args {
        if keyword.name == *vararg_name {
            return unexpected_keyword_error(callable_name, &keyword.name, ctx, keyword.name_range);
        }
        if !used_kwargs.contains(&keyword.name)
            && !ft
                .params
                .iter()
                .take(vararg_index)
                .chain(ft.params.iter().skip(vararg_index + 1))
                .any(|(param_name, _, _)| param_name == &keyword.name)
        {
            return unexpected_keyword_error(callable_name, &keyword.name, ctx, keyword.name_range);
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
    let mut keywords: LoweredKeywords = Vec::with_capacity(call.arguments.keywords.len());
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            return unpacked_keyword_error(method, ctx, keyword.range());
        };
        if keywords.iter().any(|seen| seen.name == name.as_str()) {
            ctx.error_with_code_at(
                DiagnosticCode::CALL_DUPLICATE_ARGUMENT,
                format!("{method}() got multiple values for keyword argument '{name}'"),
                name.range(),
            );
            return None;
        }
        keywords.push(LoweredKeyword {
            name: name.to_string(),
            value: lower_expr(&keyword.value, ctx)?,
            name_range: name.range(),
        });
    }
    Some(keywords)
}

fn take_keyword(keywords: &mut LoweredKeywords, name: &str) -> Option<LoweredKeyword> {
    let index = keywords.iter().position(|keyword| keyword.name == name)?;
    Some(keywords.remove(index))
}

fn reject_remaining_keywords(
    method: &str,
    keywords: &[LoweredKeyword],
    ctx: &mut LowerCtx,
) -> Option<()> {
    if let Some(keyword) = keywords.first() {
        return unexpected_keyword_error(method, &keyword.name, ctx, keyword.name_range);
    }
    Some(())
}

fn default_arg_expr(defaults: Option<&[(usize, HirExpr)]>, index: usize) -> Option<&HirExpr> {
    defaults.and_then(|defs| {
        defs.iter()
            .find(|(idx, _)| *idx == index)
            .map(|(_, expr)| expr)
    })
}

fn keyword_arg<'a>(keywords: &'a LoweredKeywords, name: &str) -> Option<&'a LoweredKeyword> {
    keywords.iter().find(|keyword| keyword.name == name)
}

fn duplicate_argument_error<T>(
    callable_name: &str,
    arg: &str,
    ctx: &mut LowerCtx,
    range: TextRange,
) -> Option<T> {
    ctx.error_with_code_at(
        DiagnosticCode::CALL_DUPLICATE_ARGUMENT,
        format!("{callable_name}() got multiple values for argument '{arg}'"),
        range,
    );
    None
}

fn missing_argument_error<T>(
    callable_name: &str,
    arg: &str,
    ctx: &mut LowerCtx,
    range: TextRange,
) -> Option<T> {
    ctx.error_with_code_at(
        DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT,
        format!("{callable_name}() missing required argument '{arg}'"),
        range,
    );
    None
}

fn unexpected_keyword_error<T>(
    callable_name: &str,
    keyword: &str,
    ctx: &mut LowerCtx,
    range: TextRange,
) -> Option<T> {
    ctx.error_with_code_at(
        DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
        format!("{callable_name}() got an unexpected keyword argument '{keyword}'"),
        range,
    );
    None
}

fn unpacked_keyword_error<T>(
    callable_name: &str,
    ctx: &mut LowerCtx,
    range: TextRange,
) -> Option<T> {
    ctx.error_with_code_at(
        DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
        format!("{callable_name}() does not support unpacked keyword arguments"),
        range,
    );
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
            return duplicate_argument_error(method, "start", ctx, start.name_range);
        }
        positional.push(start.value);
    }
    if let Some(stop) = take_keyword(keywords, "stop") {
        if positional.len() > 2 {
            return duplicate_argument_error(method, "stop", ctx, stop.name_range);
        }
        if positional.len() == 1 {
            positional.push(HirExpr::IntLiteral(0));
        }
        positional.push(stop.value);
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
        "sort" => {
            let mut args = positional;
            if let Some(reverse) = take_keyword(keywords, "reverse") {
                if !args.is_empty() {
                    return duplicate_argument_error(method, "reverse", ctx, reverse.name_range);
                }
                args.push(reverse.value);
            }
            Some(args)
        }
        _ => normalize_index_method_args(method, positional, keywords, ctx),
    }
}

fn normalize_dict_method_args(
    method: &str,
    positional: Vec<HirExpr>,
    keywords: &mut LoweredKeywords,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    match method {
        "get" | "pop" | "setdefault" => {
            let mut args = positional;
            if let Some(default) = take_keyword(keywords, "default") {
                if args.len() > 1 {
                    return duplicate_argument_error(method, "default", ctx, default.name_range);
                }
                if args.len() == 1 {
                    args.push(default.value);
                }
            }
            Some(args)
        }
        "update" => {
            let mut args = positional;
            if !keywords.is_empty() {
                let mut keys = Vec::with_capacity(keywords.len());
                let mut values = Vec::with_capacity(keywords.len());
                for keyword in keywords.drain(..) {
                    keys.push(HirExpr::StringLiteral(keyword.name));
                    values.push(keyword.value);
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
    normalize_index_method_args(method, positional, keywords, ctx)
}

fn normalize_index_method_args(
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
                    return duplicate_argument_error(method, "sep", ctx, sep.name_range);
                }
                if args.is_empty() {
                    args.push(sep.value);
                }
            }
            if let Some(maxsplit) = take_keyword(keywords, "maxsplit") {
                if args.len() > 1 {
                    return duplicate_argument_error(method, "maxsplit", ctx, maxsplit.name_range);
                }
                if args.is_empty() {
                    args.push(HirExpr::NoneLiteral);
                }
                args.push(maxsplit.value);
            }
            Some(args)
        }
        "replace" => {
            let mut args = positional;
            if let Some(count) = take_keyword(keywords, "count") {
                if args.len() > 2 {
                    return duplicate_argument_error(method, "count", ctx, count.name_range);
                }
                args.push(count.value);
            }
            Some(args)
        }
        _ => Some(positional),
    }
}

pub(in crate::lower) fn validate_list_extend_arg(
    list_elem_ty: &Type,
    iterable_ty: &Type,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(iterable_elem_ty) = callable_builtin_element_type(iterable_ty) else {
        ctx.error_with_code_at(
            DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
            format!(
                "list.extend() argument must be an iterable with a statically-known element type, got '{}'",
                iterable_ty.display_name()
            ),
            range,
        );
        return false;
    };
    if !iterable_elem_ty.is_assignable_to(list_elem_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "list.extend() iterable element type '{}' is not compatible with list element type '{}'",
                iterable_elem_ty.display_name(),
                list_elem_ty.display_name()
            ),
            range,
        );
        return false;
    }
    true
}

pub(in crate::lower) fn validate_dict_update_arg(
    key_ty: &Type,
    value_ty: &Type,
    update_ty: &Type,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(Type::Dict(update_key_ty, update_value_ty)) =
        callable_builtin_dict_output_type(update_ty)
    else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "dict.update() argument must be a dict or iterable of key/value tuples, got '{}'",
                update_ty.display_name()
            ),
            range,
        );
        return false;
    };
    let mut valid = true;
    if !update_key_ty.is_assignable_to(key_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "dict.update() key type '{}' is not compatible with dict key type '{}'",
                update_key_ty.display_name(),
                key_ty.display_name()
            ),
            range,
        );
        valid = false;
    }
    if !update_value_ty.is_assignable_to(value_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "dict.update() value type '{}' is not compatible with dict value type '{}'",
                update_value_ty.display_name(),
                value_ty.display_name()
            ),
            range,
        );
        valid = false;
    }
    valid
}

pub(in crate::lower) fn validate_set_iterable_arg(
    set_elem_ty: &Type,
    iterable_ty: &Type,
    method: &str,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(iterable_elem_ty) = callable_builtin_element_type(iterable_ty) else {
        ctx.error_with_code_at(
            DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
            format!(
                "set.{method}() arguments must be iterables with a statically-known element type, got '{}'",
                iterable_ty.display_name()
            ),
            range,
        );
        return false;
    };
    if !iterable_elem_ty.is_assignable_to(set_elem_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "set.{method}() iterable element type '{}' is not compatible with set element type '{}'",
                iterable_elem_ty.display_name(),
                set_elem_ty.display_name()
            ),
            range,
        );
        return false;
    }
    true
}
