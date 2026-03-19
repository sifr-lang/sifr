use crate::hir_nodes::HirExpr;
use sifr_python_ast::{Expr, ExprCall, Number};
use sifr_type_system::{make_union, Type};

use super::expressions::lower_expr;
use super::LowerCtx;

pub(super) const DEFAULTDICT_INT_ALIAS: &str = "__compat_defaultdict_int";
pub(super) const DEFAULTDICT_LIST_ALIAS: &str = "__compat_defaultdict_list";
pub(super) const DEFAULTDICT_SET_ALIAS: &str = "__compat_defaultdict_set";

fn homogeneous_tuple_element_type(elems: &[Type]) -> Option<Type> {
    if elems.is_empty() {
        return Some(Type::Any);
    }
    let first = elems[0].clone();
    if elems.iter().all(|elem| elem == &first) {
        Some(first)
    } else {
        None
    }
}

fn iterable_element_type_for_builtin(arg_ty: &Type) -> Option<Type> {
    match arg_ty.resolve_alias() {
        Type::List(elem) | Type::Set(elem) => Some(*elem.clone()),
        Type::Tuple(elems) => homogeneous_tuple_element_type(elems),
        Type::Range => Some(Type::Int),
        Type::Str => Some(Type::Str),
        Type::Bytes => Some(Type::Int),
        Type::Dict(key, _) => Some(*key.clone()),
        Type::Iterable(elem) | Type::Iterator(elem) => Some(*elem.clone()),
        Type::Any | Type::Unknown => Some(Type::Any),
        _ => None,
    }
}

fn list_constructor_output_type(arg_ty: &Type) -> Option<Type> {
    Some(Type::List(Box::new(iterable_element_type_for_builtin(
        arg_ty,
    )?)))
}

fn pair_tuple_types(ty: &Type) -> Option<(Type, Type)> {
    let Type::Tuple(items) = ty.resolve_alias() else {
        return None;
    };
    if items.len() != 2 {
        return None;
    }
    Some((items[0].clone(), items[1].clone()))
}

fn dict_constructor_output_type(arg_ty: &Type) -> Option<Type> {
    match arg_ty.resolve_alias() {
        Type::Dict(key, value) => Some(Type::Dict(key.clone(), value.clone())),
        Type::List(elem) | Type::Set(elem) => {
            let (key_ty, value_ty) = pair_tuple_types(elem)?;
            Some(Type::Dict(Box::new(key_ty), Box::new(value_ty)))
        }
        Type::Tuple(items) => {
            if items.is_empty() {
                Some(Type::Dict(Box::new(Type::Any), Box::new(Type::Any)))
            } else {
                let mut key_types = Vec::with_capacity(items.len());
                let mut value_types = Vec::with_capacity(items.len());
                for item in items {
                    let (key_ty, value_ty) = pair_tuple_types(item)?;
                    key_types.push(key_ty);
                    value_types.push(value_ty);
                }
                Some(Type::Dict(
                    Box::new(make_union(key_types)),
                    Box::new(make_union(value_types)),
                ))
            }
        }
        Type::Any | Type::Unknown => Some(Type::Dict(Box::new(Type::Any), Box::new(Type::Any))),
        _ => None,
    }
}

enum OptionalIterableArg {
    Missing,
    Value(HirExpr),
}

fn lower_single_optional_iterable_arg(
    call: &ExprCall,
    builtin_name: &str,
    ctx: &mut LowerCtx,
) -> Option<OptionalIterableArg> {
    if !call.arguments.keywords.is_empty() {
        ctx.error(format!(
            "{builtin_name}() does not accept keyword arguments"
        ));
        return None;
    }
    match call.arguments.args.len() {
        0 => Some(OptionalIterableArg::Missing),
        1 => Some(OptionalIterableArg::Value(lower_expr(
            &call.arguments.args[0],
            ctx,
        )?)),
        actual => {
            ctx.error(format!(
                "{builtin_name}() takes at most 1 positional argument, got {actual}"
            ));
            None
        }
    }
}

pub(super) fn lower_list_constructor_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let arg = lower_single_optional_iterable_arg(call, "list", ctx)?;
    match arg {
        OptionalIterableArg::Missing => Some(HirExpr::ListLiteral {
            elements: Vec::new(),
            ty: Type::List(Box::new(Type::Any)),
        }),
        OptionalIterableArg::Value(iterable) => {
            let list_ty = list_constructor_output_type(iterable.ty())?;
            Some(HirExpr::Call {
                func: "list".to_string(),
                args: vec![iterable],
                ty: list_ty,
            })
        }
    }
}

pub(super) fn lower_tuple_constructor_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        ctx.error("tuple() does not accept keyword arguments".to_string());
        return None;
    }
    match &call.arguments.args[..] {
        [] => Some(HirExpr::TupleLiteral {
            elements: Vec::new(),
            ty: Type::Tuple(Vec::new()),
        }),
        [Expr::Tuple(tuple)] => {
            let mut elements = Vec::with_capacity(tuple.elts.len());
            let mut elem_types = Vec::with_capacity(tuple.elts.len());
            for element in &tuple.elts {
                let lowered = lower_expr(element, ctx)?;
                elem_types.push(lowered.ty().clone());
                elements.push(lowered);
            }
            Some(HirExpr::TupleLiteral {
                elements,
                ty: Type::Tuple(elem_types),
            })
        }
        [Expr::List(list)] => {
            let mut elements = Vec::with_capacity(list.elts.len());
            let mut elem_types = Vec::with_capacity(list.elts.len());
            for element in &list.elts {
                let lowered = lower_expr(element, ctx)?;
                elem_types.push(lowered.ty().clone());
                elements.push(lowered);
            }
            Some(HirExpr::TupleLiteral {
                elements,
                ty: Type::Tuple(elem_types),
            })
        }
        [Expr::StringLiteral(text)] => {
            let chars: Vec<String> = text
                .value
                .to_str()
                .chars()
                .map(|character| character.to_string())
                .collect();
            Some(HirExpr::TupleLiteral {
                elements: chars.iter().cloned().map(HirExpr::StringLiteral).collect(),
                ty: Type::Tuple(vec![Type::Str; chars.len()]),
            })
        }
        [arg_expr] => {
            let lowered = lower_expr(arg_expr, ctx)?;
            if matches!(lowered.ty().resolve_alias(), Type::Tuple(_)) {
                Some(lowered)
            } else {
                ctx.error(
                    "tuple() currently requires a tuple, list literal, or string literal because Sifr tuples are fixed-length typed values".to_string(),
                );
                None
            }
        }
        _ => {
            ctx.error(format!(
                "tuple() takes at most 1 positional argument, got {}",
                call.arguments.args.len()
            ));
            None
        }
    }
}

pub(super) fn lower_dict_constructor_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() > 1 {
        ctx.error(format!(
            "dict() takes at most 1 positional argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }

    let mut keyword_keys = Vec::with_capacity(call.arguments.keywords.len());
    let mut keyword_values = Vec::with_capacity(call.arguments.keywords.len());
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            ctx.error("dict() does not support unpacked keyword arguments".to_string());
            return None;
        };
        keyword_keys.push(HirExpr::StringLiteral(name.to_string()));
        keyword_values.push(lower_expr(&keyword.value, ctx)?);
    }

    let keyword_value_ty = if keyword_values.is_empty() {
        Type::Any
    } else {
        make_union(
            keyword_values
                .iter()
                .map(|value| value.ty().clone())
                .collect(),
        )
    };
    let keyword_dict = if keyword_keys.is_empty() {
        None
    } else {
        Some(HirExpr::DictLiteral {
            keys: keyword_keys,
            values: keyword_values,
            ty: Type::Dict(Box::new(Type::Str), Box::new(keyword_value_ty.clone())),
        })
    };

    match &call.arguments.args[..] {
        [] if keyword_dict.is_none() => Some(HirExpr::Call {
            func: "dict".to_string(),
            args: Vec::new(),
            ty: Type::Dict(Box::new(Type::Any), Box::new(Type::Any)),
        }),
        [] => keyword_dict,
        [arg_expr] if keyword_dict.is_none() => {
            let arg = lower_expr(arg_expr, ctx)?;
            let dict_ty = dict_constructor_output_type(arg.ty())?;
            Some(HirExpr::Call {
                func: "dict".to_string(),
                args: vec![arg],
                ty: dict_ty,
            })
        }
        [arg_expr] => {
            let arg = lower_expr(arg_expr, ctx)?;
            let Type::Dict(key_ty, value_ty) = dict_constructor_output_type(arg.ty())? else {
                ctx.error(format!(
                    "dict() argument must be a dict or iterable of key/value tuples, got '{}'",
                    arg.ty().display_name()
                ));
                return None;
            };
            let merged_ty = Type::Dict(
                Box::new(make_union(vec![(*key_ty).clone(), Type::Str])),
                Box::new(make_union(vec![(*value_ty).clone(), keyword_value_ty])),
            );
            Some(HirExpr::Call {
                func: "dict".to_string(),
                args: vec![arg, keyword_dict?],
                ty: merged_ty,
            })
        }
        _ => unreachable!(),
    }
}

pub(super) fn lower_ord_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        ctx.error("ord() does not accept keyword arguments".to_string());
        return None;
    }
    if call.arguments.args.len() != 1 {
        ctx.error(format!(
            "ord() takes exactly 1 positional argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }

    if let Expr::StringLiteral(text) = &call.arguments.args[0] {
        let chars: Vec<char> = text.value.to_str().chars().collect();
        if chars.len() != 1 {
            ctx.error(
                "ord() string literal argument must contain exactly one character".to_string(),
            );
            return None;
        }
        return Some(HirExpr::IntLiteral(chars[0] as i64));
    }

    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    if arg.ty() != &Type::Str {
        ctx.error(format!(
            "ord() argument must be 'str', got '{}'",
            arg.ty().display_name()
        ));
        return None;
    }
    Some(HirExpr::Call {
        func: "ord".to_string(),
        args: vec![arg],
        ty: Type::Result(
            Box::new(Type::Int),
            Box::new(
                ctx.class_types
                    .get("ValueError")
                    .cloned()
                    .unwrap_or(Type::Class {
                        name: "ValueError".to_string(),
                        fields: vec![("message".to_string(), Type::Str)],
                        methods: vec![],
                        parent_class: Some("Error".to_string()),
                    }),
            ),
        ),
    })
}

pub(super) fn lower_chr_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        ctx.error("chr() does not accept keyword arguments".to_string());
        return None;
    }
    if call.arguments.args.len() != 1 {
        ctx.error(format!(
            "chr() takes exactly 1 positional argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }

    if let Expr::NumberLiteral(num) = &call.arguments.args[0] {
        let Number::Int(value) = &num.value else {
            ctx.error("chr() argument must be an integer".to_string());
            return None;
        };
        let Some(value) = value.as_i64() else {
            ctx.error("chr() integer literal is out of range for 'int'".to_string());
            return None;
        };
        let Ok(code_point) = u32::try_from(value) else {
            ctx.error("chr() integer literal must be a valid Unicode code point".to_string());
            return None;
        };
        let Some(character) = char::from_u32(code_point) else {
            ctx.error("chr() integer literal must be a valid Unicode code point".to_string());
            return None;
        };
        return Some(HirExpr::StringLiteral(character.to_string()));
    }

    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    if arg.ty() != &Type::Int {
        ctx.error(format!(
            "chr() argument must be 'int', got '{}'",
            arg.ty().display_name()
        ));
        return None;
    }
    Some(HirExpr::Call {
        func: "chr".to_string(),
        args: vec![arg],
        ty: Type::Result(
            Box::new(Type::Str),
            Box::new(
                ctx.class_types
                    .get("ValueError")
                    .cloned()
                    .unwrap_or(Type::Class {
                        name: "ValueError".to_string(),
                        fields: vec![("message".to_string(), Type::Str)],
                        methods: vec![],
                        parent_class: Some("Error".to_string()),
                    }),
            ),
        ),
    })
}

fn defaultdict_alias_and_value_type(factory_name: &str) -> Option<(&'static str, Type)> {
    match factory_name {
        "int" => Some((DEFAULTDICT_INT_ALIAS, Type::Int)),
        "list" => Some((DEFAULTDICT_LIST_ALIAS, Type::List(Box::new(Type::Any)))),
        "set" => Some((DEFAULTDICT_SET_ALIAS, Type::Set(Box::new(Type::Any)))),
        _ => None,
    }
}

pub(super) fn lower_defaultdict_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        ctx.error("defaultdict() does not support keyword arguments in this slice".to_string());
        return None;
    }
    if call.arguments.args.is_empty() || call.arguments.args.len() > 2 {
        ctx.error(format!(
            "defaultdict() takes 1 or 2 positional arguments, got {}",
            call.arguments.args.len()
        ));
        return None;
    }

    let factory_name = if let Expr::Name(name) = &call.arguments.args[0] {
        name.id.as_str()
    } else {
        ctx.error(
            "defaultdict() factory must be a builtin name such as int, list, or set".to_string(),
        );
        return None;
    };
    let Some((alias_name, value_ty)) = defaultdict_alias_and_value_type(factory_name) else {
        ctx.error(format!(
            "defaultdict() currently supports int, list, and set factories, got '{factory_name}'"
        ));
        return None;
    };

    let mut args = Vec::new();
    let dict_ty = if call.arguments.args.len() == 2 {
        let mapping = lower_expr(&call.arguments.args[1], ctx)?;
        let Type::Dict(key_ty, mapping_value_ty) = mapping.ty().resolve_alias() else {
            ctx.error(format!(
                "defaultdict() initial mapping must be a dict, got '{}'",
                mapping.ty().display_name()
            ));
            return None;
        };
        if !mapping_value_ty.is_assignable_to(&value_ty)
            && !value_ty.is_assignable_to(mapping_value_ty)
        {
            ctx.error(format!(
                "defaultdict() initial mapping value type '{}' is not compatible with factory '{}'",
                mapping_value_ty.display_name(),
                factory_name
            ));
            return None;
        }
        let dict_key_ty = key_ty.clone();
        args.push(mapping);
        Type::Dict(dict_key_ty, Box::new(value_ty))
    } else {
        Type::Dict(Box::new(Type::Any), Box::new(value_ty))
    };

    Some(HirExpr::Call {
        func: alias_name.to_string(),
        args,
        ty: Type::Alias {
            name: alias_name.to_string(),
            type_args: Vec::new(),
            body: Box::new(dict_ty),
        },
    })
}

pub(super) fn lower_set_constructor_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let arg = lower_single_optional_iterable_arg(call, "set", ctx)?;

    match arg {
        OptionalIterableArg::Missing => Some(HirExpr::Call {
            func: "set".to_string(),
            args: Vec::new(),
            ty: Type::Set(Box::new(Type::Any)),
        }),
        OptionalIterableArg::Value(iterable) => {
            let Some(elem_ty) = iterable_element_type_for_builtin(iterable.ty()) else {
                ctx.error(format!(
                    "set() argument must be an iterable with a statically-known element type, got '{}'",
                    iterable.ty().display_name()
                ));
                return None;
            };
            Some(HirExpr::Call {
                func: "set".to_string(),
                args: vec![iterable],
                ty: Type::Set(Box::new(elem_ty)),
            })
        }
    }
}

pub(super) fn lower_bytes_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        ctx.error("bytes() does not accept keyword arguments in wave_psp_bytes_1".to_string());
        return None;
    }
    if call.arguments.args.is_empty() {
        return Some(HirExpr::ListLiteral {
            elements: Vec::new(),
            ty: Type::Bytes,
        });
    }
    ctx.error(
        "bytes() with arguments is scheduled for wave_psp_bytes_2 conversion surfaces; use bytes literals in wave_psp_bytes_1".to_string(),
    );
    None
}

pub(super) fn lower_len_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        ctx.error(format!(
            "len() takes exactly 1 argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let arg_ty = arg.ty().clone();

    let effective_ty = if let Type::Union(members) = &arg_ty {
        let non_none: Vec<&Type> = members
            .iter()
            .filter(|m| !matches!(m, Type::None))
            .collect();
        if non_none.len() == 1 {
            non_none[0].clone()
        } else {
            arg_ty.clone()
        }
    } else {
        arg_ty.clone()
    };
    match effective_ty.resolve_alias() {
        Type::Str | Type::Bytes | Type::List(_) | Type::Dict(_, _) | Type::Tuple(_) | Type::Set(_) => {
            Some(HirExpr::MethodCall {
                object: Box::new(arg),
                method: "len".to_string(),
                args: vec![],
                ty: Type::Int,
            })
        }
        Type::Class { methods, .. } if methods.iter().any(|(name, _)| name == "len") => {
            Some(HirExpr::MethodCall {
                object: Box::new(arg),
                method: "len".to_string(),
                args: vec![],
                ty: Type::Int,
            })
        }
        _ => {
            ctx.error(format!(
                "len() argument must be a string, bytes, list, dict, tuple, set, or sized class, got '{}'",
                arg_ty.display_name()
            ));
            None
        }
    }
}

pub(super) fn lower_isinstance_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 2 {
        ctx.error(format!(
            "isinstance() takes exactly 2 arguments, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let type_name = match &call.arguments.args[1] {
        Expr::Name(n) => n.id.clone(),
        _ => "unknown".to_string(),
    };
    Some(HirExpr::Call {
        func: "isinstance".to_string(),
        args: vec![arg, HirExpr::StringLiteral(type_name)],
        ty: Type::Bool,
    })
}

pub(super) fn lower_reveal_type_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        ctx.error(format!(
            "reveal_type() takes exactly 1 argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let ty = arg.ty().clone();
    ctx.reveal_types
        .push(format!("reveal_type: {}", ty.display_name()));
    Some(arg)
}

pub(super) fn lower_range_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() > 3 {
        ctx.error(format!(
            "range() takes at most 3 positional arguments, got {}",
            call.arguments.args.len()
        ));
        return None;
    }

    let mut start_expr = None;
    let mut stop_expr = None;
    let mut step_expr = None;

    match call.arguments.args.len() {
        0 => {}
        1 => {
            stop_expr = Some(&call.arguments.args[0]);
        }
        2 => {
            start_expr = Some(&call.arguments.args[0]);
            stop_expr = Some(&call.arguments.args[1]);
        }
        3 => {
            start_expr = Some(&call.arguments.args[0]);
            stop_expr = Some(&call.arguments.args[1]);
            step_expr = Some(&call.arguments.args[2]);
        }
        _ => unreachable!(),
    }

    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            ctx.error("range() does not support unpacked keyword arguments".to_string());
            return None;
        };
        match name.as_str() {
            "start" => {
                if start_expr.is_some() {
                    ctx.error(
                        "range(): 'start' was provided both positionally and as a keyword"
                            .to_string(),
                    );
                    return None;
                }
                start_expr = Some(&keyword.value);
            }
            "stop" => {
                if stop_expr.is_some() {
                    ctx.error(
                        "range(): 'stop' was provided both positionally and as a keyword"
                            .to_string(),
                    );
                    return None;
                }
                stop_expr = Some(&keyword.value);
            }
            "step" => {
                if step_expr.is_some() {
                    ctx.error(
                        "range(): 'step' was provided both positionally and as a keyword"
                            .to_string(),
                    );
                    return None;
                }
                step_expr = Some(&keyword.value);
            }
            other => {
                ctx.error(format!(
                    "range() got an unexpected keyword argument '{other}'"
                ));
                return None;
            }
        }
    }

    let Some(stop_raw) = stop_expr else {
        ctx.error("range() missing required argument 'stop'".to_string());
        return None;
    };

    let start = if let Some(raw) = start_expr {
        let lowered = lower_expr(raw, ctx)?;
        if lowered.ty() != &Type::Int {
            ctx.error(format!(
                "range() start argument must be 'int', got '{}'",
                lowered.ty().display_name()
            ));
            return None;
        }
        lowered
    } else {
        HirExpr::IntLiteral(0)
    };
    let stop = lower_expr(stop_raw, ctx)?;
    if stop.ty() != &Type::Int {
        ctx.error(format!(
            "range() stop argument must be 'int', got '{}'",
            stop.ty().display_name()
        ));
        return None;
    }
    let step = if let Some(raw) = step_expr {
        let lowered = lower_expr(raw, ctx)?;
        if lowered.ty() != &Type::Int {
            ctx.error(format!(
                "range() step argument must be 'int', got '{}'",
                lowered.ty().display_name()
            ));
            return None;
        }
        Some(Box::new(lowered))
    } else {
        None
    };

    Some(HirExpr::RangeLiteral {
        start: Box::new(start),
        end: Box::new(stop),
        step,
        ty: Type::Range,
    })
}

pub(super) fn callable_builtin_element_type(arg_ty: &Type) -> Option<Type> {
    iterable_element_type_for_builtin(arg_ty)
}

pub(super) fn callable_builtin_list_output_type(arg_ty: &Type) -> Option<Type> {
    list_constructor_output_type(arg_ty)
}

pub(super) fn callable_builtin_dict_output_type(arg_ty: &Type) -> Option<Type> {
    dict_constructor_output_type(arg_ty)
}

pub(super) fn lower_builtin_reverseable_arg(
    call: &ExprCall,
    builtin_name: &str,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        ctx.error(format!(
            "{builtin_name}() takes exactly 1 positional argument"
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    if callable_builtin_element_type(arg.ty()).is_none() {
        ctx.error(format!(
            "{builtin_name}() argument must be an iterable with a statically-known element type, got '{}'",
            arg.ty().display_name()
        ));
        return None;
    }
    Some(arg)
}
