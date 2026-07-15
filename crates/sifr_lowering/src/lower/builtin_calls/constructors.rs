use super::str;
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::CompilerIntrinsicId;
use sifr_python_ast::{Expr, ExprCall, Number};
use sifr_type_system::{make_union, Type};

use super::expressions::{consume_affine_value_name, lower_expr};
use super::LowerCtx;

pub(in crate::lower) const DEFAULTDICT_INT_ALIAS: &str = "__sifr_defaultdict_int";
pub(in crate::lower) const DEFAULTDICT_LIST_ALIAS: &str = "__sifr_defaultdict_list";
pub(in crate::lower) const DEFAULTDICT_SET_ALIAS: &str = "__sifr_defaultdict_set";

pub(super) fn first_keyword_range(call: &ExprCall) -> ruff_text_size::TextRange {
    call.arguments
        .keywords
        .first()
        .map_or_else(|| call.func.range(), |keyword| keyword.range)
}

pub(super) fn arity_range(call: &ExprCall) -> ruff_text_size::TextRange {
    call.arguments
        .args
        .last()
        .map_or_else(|| call.func.range(), Ranged::range)
}

pub(super) fn reject_keywords(call: &ExprCall, callable_name: &str, ctx: &mut LowerCtx) {
    ctx.error_with_code_at(
        DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
        format!("{callable_name}() does not accept keyword arguments"),
        first_keyword_range(call),
    );
}

pub(super) fn reject_unpacked_keyword(call: &ExprCall, callable_name: &str, ctx: &mut LowerCtx) {
    reject_unpacked_keyword_at(callable_name, ctx, first_keyword_range(call));
}

pub(super) fn reject_unpacked_keyword_at(
    callable_name: &str,
    ctx: &mut LowerCtx,
    range: ruff_text_size::TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
        format!("{callable_name}() does not support unpacked keyword arguments"),
        range,
    );
}

pub(super) fn reject_wrong_positional_count(call: &ExprCall, message: String, ctx: &mut LowerCtx) {
    ctx.error_with_code_at(
        DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
        message,
        arity_range(call),
    );
}

pub(super) fn reject_type_mismatch(
    ctx: &mut LowerCtx,
    message: String,
    range: ruff_text_size::TextRange,
) {
    ctx.error_with_code_at(DiagnosticCode::TYPE_MISMATCH, message, range);
}

pub(super) fn reject_unsupported_surface(
    ctx: &mut LowerCtx,
    message: String,
    range: ruff_text_size::TextRange,
) {
    ctx.error_with_code_at(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, message, range);
}

pub(in crate::lower) fn reject_zip_keywords_if_present(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(keyword) = call.arguments.keywords.first() else {
        return false;
    };
    let Some(name) = keyword.arg.as_ref() else {
        reject_unpacked_keyword(call, "zip", ctx);
        return true;
    };
    let message = match name.as_str() {
        "strict" => "zip() keyword argument 'strict' is not supported".to_string(),
        other => {
            ctx.error_with_code_at(
                DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
                format!("zip() got an unexpected keyword argument '{other}'"),
                name.range(),
            );
            return true;
        }
    };
    ctx.error_with_code_at(
        DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
        message,
        name.range(),
    );
    true
}

pub(super) fn iterable_element_type_for_builtin(arg_ty: &Type) -> Option<Type> {
    arg_ty.iterable_element_type().or_else(|| {
        if matches!(arg_ty.resolve_alias(), Type::Any | Type::Unknown) {
            Some(Type::Any)
        } else {
            None
        }
    })
}

pub(super) fn list_constructor_output_type(arg_ty: &Type) -> Option<Type> {
    Some(Type::List(Box::new(iterable_element_type_for_builtin(
        arg_ty,
    )?)))
}

pub(super) fn pair_tuple_types(ty: &Type) -> Option<(Type, Type)> {
    let Type::Tuple(items) = ty.resolve_alias() else {
        return None;
    };
    if items.len() != 2 {
        return None;
    }
    Some((items[0].clone(), items[1].clone()))
}

pub(super) fn dict_constructor_output_type(arg_ty: &Type) -> Option<Type> {
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
        reject_keywords(call, builtin_name, ctx);
        return None;
    }
    match call.arguments.args.len() {
        0 => Some(OptionalIterableArg::Missing),
        1 => Some(OptionalIterableArg::Value(lower_expr(
            &call.arguments.args[0],
            ctx,
        )?)),
        actual => {
            reject_wrong_positional_count(
                call,
                format!("{builtin_name}() takes at most 1 positional argument, got {actual}"),
                ctx,
            );
            None
        }
    }
}

pub(in crate::lower) fn lower_list_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let arg = lower_single_optional_iterable_arg(call, "list", ctx)?;
    match arg {
        OptionalIterableArg::Missing => Some(HirExpr::ListLiteral {
            elements: Vec::new(),
            ty: Type::List(Box::new(Type::Any)),
        }),
        OptionalIterableArg::Value(iterable) => {
            let list_ty = list_constructor_output_type(iterable.ty())?;
            if list_ty.contains_affine_resource() {
                ctx.error_with_code_at(
                    DiagnosticCode::PYZC_INVALID_DECLARATION,
                    "list() cannot copy an iterable containing affine Python buffers".to_string(),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            Some(HirExpr::Call {
                func: "list".to_string(),
                args: vec![iterable],
                ty: list_ty,
            })
        }
    }
}

pub(in crate::lower) fn lower_tuple_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        reject_keywords(call, "tuple", ctx);
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
                consume_affine_value_name(&lowered, element.range(), ctx);
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
                consume_affine_value_name(&lowered, element.range(), ctx);
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
                consume_affine_value_name(&lowered, arg_expr.range(), ctx);
                Some(lowered)
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                    "tuple() currently requires a tuple, list literal, or string literal because Sifr tuples are fixed-length typed values".to_string(),
                    arg_expr.range(),
                );
                None
            }
        }
        _ => {
            reject_wrong_positional_count(
                call,
                format!(
                    "tuple() takes at most 1 positional argument, got {}",
                    call.arguments.args.len()
                ),
                ctx,
            );
            None
        }
    }
}

pub(in crate::lower) fn lower_dict_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if call.arguments.args.len() > 1 {
        reject_wrong_positional_count(
            call,
            format!(
                "dict() takes at most 1 positional argument, got {}",
                call.arguments.args.len()
            ),
            ctx,
        );
        return None;
    }

    let mut keyword_keys = Vec::with_capacity(call.arguments.keywords.len());
    let mut keyword_values = Vec::with_capacity(call.arguments.keywords.len());
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            reject_unpacked_keyword_at("dict", ctx, keyword.range);
            return None;
        };
        keyword_keys.push(HirExpr::StringLiteral(name.to_string()));
        let value = lower_expr(&keyword.value, ctx)?;
        consume_affine_value_name(&value, keyword.value.range(), ctx);
        keyword_values.push(value);
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
            if dict_ty.contains_affine_resource() {
                ctx.error_with_code_at(
                    DiagnosticCode::PYZC_INVALID_DECLARATION,
                    "dict() cannot copy keys or values containing affine Python buffers"
                        .to_string(),
                    arg_expr.range(),
                );
                return None;
            }
            Some(HirExpr::Call {
                func: "dict".to_string(),
                args: vec![arg],
                ty: dict_ty,
            })
        }
        [arg_expr] => {
            let arg = lower_expr(arg_expr, ctx)?;
            let Type::Dict(key_ty, value_ty) = dict_constructor_output_type(arg.ty())? else {
                reject_type_mismatch(
                    ctx,
                    format!(
                        "dict() argument must be a dict or iterable of key/value tuples, got '{}'",
                        arg.ty().display_name()
                    ),
                    arg_expr.range(),
                );
                return None;
            };
            if arg.ty().contains_affine_resource() {
                ctx.error_with_code_at(
                    DiagnosticCode::PYZC_INVALID_DECLARATION,
                    "dict() cannot copy keys or values containing affine Python buffers"
                        .to_string(),
                    arg_expr.range(),
                );
                return None;
            }
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

pub(in crate::lower) fn lower_ord_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        reject_keywords(call, "ord", ctx);
        return None;
    }
    if call.arguments.args.len() != 1 {
        reject_wrong_positional_count(
            call,
            format!(
                "ord() takes exactly 1 positional argument, got {}",
                call.arguments.args.len()
            ),
            ctx,
        );
        return None;
    }

    if let Expr::StringLiteral(text) = &call.arguments.args[0] {
        let chars: Vec<char> = text.value.to_str().chars().collect();
        if chars.len() != 1 {
            reject_type_mismatch(
                ctx,
                "ord() string literal argument must contain exactly one character".to_string(),
                call.arguments.args[0].range(),
            );
            return None;
        }
        return Some(HirExpr::IntLiteral(chars[0] as i64));
    }

    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    if arg.ty() != &Type::Str {
        reject_type_mismatch(
            ctx,
            format!(
                "ord() argument must be 'str', got '{}'",
                arg.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
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

pub(in crate::lower) fn lower_chr_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        reject_keywords(call, "chr", ctx);
        return None;
    }
    if call.arguments.args.len() != 1 {
        reject_wrong_positional_count(
            call,
            format!(
                "chr() takes exactly 1 positional argument, got {}",
                call.arguments.args.len()
            ),
            ctx,
        );
        return None;
    }

    if let Expr::NumberLiteral(num) = &call.arguments.args[0] {
        let Number::Int(value) = &num.value else {
            reject_type_mismatch(
                ctx,
                "chr() argument must be an integer".to_string(),
                call.arguments.args[0].range(),
            );
            return None;
        };
        let Some(value) = value.as_i64() else {
            reject_type_mismatch(
                ctx,
                "chr() integer literal is out of range for 'int'".to_string(),
                call.arguments.args[0].range(),
            );
            return None;
        };
        let Ok(code_point) = u32::try_from(value) else {
            reject_type_mismatch(
                ctx,
                "chr() integer literal must be a valid Unicode code point".to_string(),
                call.arguments.args[0].range(),
            );
            return None;
        };
        let Some(character) = char::from_u32(code_point) else {
            reject_type_mismatch(
                ctx,
                "chr() integer literal must be a valid Unicode code point".to_string(),
                call.arguments.args[0].range(),
            );
            return None;
        };
        return Some(HirExpr::StringLiteral(character.to_string()));
    }

    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    if arg.ty() != &Type::Int {
        reject_type_mismatch(
            ctx,
            format!(
                "chr() argument must be 'int', got '{}'",
                arg.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
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

pub(super) fn defaultdict_alias_and_value_type(factory_name: &str) -> Option<(&'static str, Type)> {
    match factory_name {
        "int" => Some((DEFAULTDICT_INT_ALIAS, Type::Int)),
        "list" => Some((DEFAULTDICT_LIST_ALIAS, Type::List(Box::new(Type::Any)))),
        "set" => Some((DEFAULTDICT_SET_ALIAS, Type::Set(Box::new(Type::Any)))),
        _ => None,
    }
}

pub(in crate::lower) fn lower_defaultdict_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        let keyword = &call.arguments.keywords[0];
        let Some(name) = keyword.arg.as_ref() else {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                "defaultdict() does not support unpacked keyword arguments".to_string(),
                keyword.range,
            );
            return None;
        };
        ctx.error_with_code_at(
            DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
            "defaultdict() does not support keyword arguments".to_string(),
            name.range(),
        );
        return None;
    }
    if call.arguments.args.is_empty() || call.arguments.args.len() > 2 {
        reject_wrong_positional_count(
            call,
            format!(
                "defaultdict() takes 1 or 2 positional arguments, got {}",
                call.arguments.args.len()
            ),
            ctx,
        );
        return None;
    }

    let factory_name = if let Expr::Name(name) = &call.arguments.args[0] {
        name.id.as_str()
    } else {
        reject_unsupported_surface(
            ctx,
            "defaultdict() factory must be a builtin name such as int, list, or set".to_string(),
            call.arguments.args[0].range(),
        );
        return None;
    };
    let Some((alias_name, value_ty)) = defaultdict_alias_and_value_type(factory_name) else {
        reject_unsupported_surface(
            ctx,
            format!(
                "defaultdict() currently supports int, list, and set factories, got '{factory_name}'"
            ),
            call.arguments.args[0].range(),
        );
        return None;
    };

    let mut args = Vec::new();
    let dict_ty = if call.arguments.args.len() == 2 {
        let mapping = lower_expr(&call.arguments.args[1], ctx)?;
        if factory_name == "int"
            && matches!(
                mapping.ty().resolve_alias(),
                Type::Class { name, .. } if name == "Counter" || name.ends_with(".Counter")
            )
        {
            // Counter already provides zero-default indexing semantics.
            // Preserve the Counter type surface for downstream assignment flow.
            return Some(mapping);
        }
        let mapping_ty = mapping.ty().clone();
        let (mapping_expr, key_ty, mapping_value_ty) = match mapping_ty.resolve_alias() {
            Type::Dict(key_ty, mapping_value_ty) => {
                (mapping, key_ty.clone(), mapping_value_ty.clone())
            }
            Type::Class { name, fields, .. } if name == "Counter" || name.ends_with(".Counter") => {
                let Some((_, counts_ty)) =
                    fields.iter().find(|(field_name, _)| field_name == "counts")
                else {
                    reject_type_mismatch(
                        ctx,
                        format!(
                            "defaultdict() initial mapping must be a dict, got '{}'",
                            mapping.ty().display_name()
                        ),
                        call.arguments.args[1].range(),
                    );
                    return None;
                };
                let counts_ty = counts_ty.clone();
                let Type::Dict(key_ty, mapping_value_ty) = counts_ty.resolve_alias() else {
                    reject_type_mismatch(
                        ctx,
                        format!(
                            "defaultdict() initial mapping must be a dict, got '{}'",
                            mapping.ty().display_name()
                        ),
                        call.arguments.args[1].range(),
                    );
                    return None;
                };
                (
                    HirExpr::FieldAccess {
                        object: Box::new(mapping),
                        field: "counts".to_string(),
                        ty: counts_ty.clone(),
                    },
                    key_ty.clone(),
                    mapping_value_ty.clone(),
                )
            }
            _ => {
                reject_type_mismatch(
                    ctx,
                    format!(
                        "defaultdict() initial mapping must be a dict, got '{}'",
                        mapping.ty().display_name()
                    ),
                    call.arguments.args[1].range(),
                );
                return None;
            }
        };
        if !mapping_value_ty.is_assignable_to(&value_ty)
            && !value_ty.is_assignable_to(&mapping_value_ty)
        {
            reject_type_mismatch(
                ctx,
                format!(
                    "defaultdict() initial mapping value type '{}' is not compatible with factory '{}'",
                    mapping_value_ty.display_name(),
                    factory_name
                ),
                call.arguments.args[1].range(),
            );
            return None;
        }
        args.push(mapping_expr);
        Type::Dict(key_ty, Box::new(value_ty))
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

pub(in crate::lower) fn lower_set_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let arg = lower_single_optional_iterable_arg(call, "set", ctx)?;

    match arg {
        OptionalIterableArg::Missing => Some(HirExpr::Call {
            func: "set".to_string(),
            args: Vec::new(),
            ty: Type::Set(Box::new(Type::Any)),
        }),
        OptionalIterableArg::Value(iterable) => {
            let Some(elem_ty) = iterable_element_type_for_builtin(iterable.ty()) else {
                reject_type_mismatch(
                    ctx,
                    format!(
                        "set() argument must be an iterable with a statically-known element type, got '{}'",
                        iterable.ty().display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            };
            if super::super::container_literal_diagnostics::reject_unhashable_container_type(
                ctx,
                "set element",
                &elem_ty,
                call.arguments.args[0].range(),
            ) {
                return None;
            }
            Some(HirExpr::Call {
                func: "set".to_string(),
                args: vec![iterable],
                ty: Type::Set(Box::new(elem_ty)),
            })
        }
    }
}

pub(in crate::lower) fn lower_bytes_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if !call.arguments.keywords.is_empty() {
        reject_keywords(call, "bytes", ctx);
        return None;
    }
    if call.arguments.args.is_empty() {
        return Some(HirExpr::ListLiteral {
            elements: Vec::new(),
            ty: Type::Bytes,
        });
    }
    if call.arguments.args.len() != 1 {
        reject_wrong_positional_count(
            call,
            format!(
                "bytes() takes at most 1 positional argument, got {}",
                call.arguments.args.len()
            ),
            ctx,
        );
        return None;
    }
    let size = lower_expr(&call.arguments.args[0], ctx)?;
    if size.ty() != &Type::Int {
        reject_type_mismatch(
            ctx,
            format!(
                "bytes(size) expects 'int' size, got '{}'",
                size.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    Some(HirExpr::IntrinsicCall {
        intrinsic: CompilerIntrinsicId::BytesWithSize,
        args: vec![size],
        ty: Type::Result(Box::new(Type::Bytes), Box::new(value_error_type(ctx))),
        call_range: call.range(),
        arg_ranges: vec![call.arguments.args[0].range()],
    })
}

pub(super) fn parse_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("ParseError")
        .cloned()
        .unwrap_or(Type::Class {
            name: "ParseError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        })
}

pub(super) fn value_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("ValueError")
        .cloned()
        .unwrap_or(Type::Class {
            name: "ValueError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        })
}
