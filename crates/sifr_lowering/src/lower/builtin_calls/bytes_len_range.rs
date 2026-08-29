use super::{
    DiagnosticCode, Expr, ExprAttribute, ExprCall, HirExpr, IterationCapability, LowerCtx, Ranged,
    RevealTypeDiagnostic, Type, arity_range, dict_constructor_output_type, first_keyword_range,
    iterable_element_type_for_builtin, list_constructor_output_type, lower_expr, parse_error_type,
    reject_keywords, reject_type_mismatch, reject_unpacked_keyword_at,
    reject_wrong_positional_count, str, value_error_type,
};
use crate::lower::typing_and_functions::resolve_annotation_expr;
use sifr_ir::CompilerIntrinsicId;
pub(in crate::lower) fn lower_bytes_type_factory_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let Expr::Name(type_name) = attr.value.as_ref() else {
        return None;
    };
    if type_name.id != "bytes" {
        return None;
    }

    if !call.arguments.keywords.is_empty() {
        reject_keywords(call, &format!("bytes.{}", attr.attr), ctx);
        return None;
    }

    match attr.attr.as_str() {
        "from_hex" => {
            if call.arguments.args.len() != 1 {
                reject_wrong_positional_count(
                    call,
                    format!(
                        "bytes.from_hex() takes exactly 1 positional argument, got {}",
                        call.arguments.args.len()
                    ),
                    ctx,
                );
                return None;
            }
            let hex_expr = lower_expr(&call.arguments.args[0], ctx)?;
            if hex_expr.ty() != &Type::Str {
                reject_type_mismatch(
                    ctx,
                    format!(
                        "bytes.from_hex() expects 'str', got '{}'",
                        hex_expr.ty().display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            Some(HirExpr::IntrinsicCall {
                intrinsic: CompilerIntrinsicId::BytesFromHex,
                args: vec![hex_expr],
                ty: Type::Result(Box::new(Type::Bytes), Box::new(parse_error_type(ctx))),
                call_range: call.range(),
                arg_ranges: vec![call.arguments.args[0].range()],
            })
        }
        "from_ints" => {
            if call.arguments.args.len() != 1 {
                reject_wrong_positional_count(
                    call,
                    format!(
                        "bytes.from_ints() takes exactly 1 positional argument, got {}",
                        call.arguments.args.len()
                    ),
                    ctx,
                );
                return None;
            }
            let data_expr = lower_expr(&call.arguments.args[0], ctx)?;
            let is_list_int = matches!(
                data_expr.ty().resolve_alias(),
                Type::List(elem) if elem.as_ref() == &Type::Int
            );
            if !is_list_int {
                reject_type_mismatch(
                    ctx,
                    format!(
                        "bytes.from_ints() expects 'list[int]', got '{}'",
                        data_expr.ty().display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            Some(HirExpr::IntrinsicCall {
                intrinsic: CompilerIntrinsicId::BytesFromIntegers,
                args: vec![data_expr],
                ty: Type::Result(Box::new(Type::Bytes), Box::new(value_error_type(ctx))),
                call_range: call.range(),
                arg_ranges: vec![call.arguments.args[0].range()],
            })
        }
        _ => None,
    }
}

pub(in crate::lower) fn lower_len_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        reject_wrong_positional_count(
            call,
            format!(
                "len() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ),
            ctx,
        );
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let arg_ty = arg.ty().clone();

    let effective_ty = arg_ty
        .optional_member_type()
        .unwrap_or_else(|| arg_ty.clone());
    match effective_ty.resolve_alias() {
        Type::Str
        | Type::Bytes
        | Type::List(_)
        | Type::Dict(_, _)
        | Type::Tuple(_)
        | Type::Set(_) => Some(HirExpr::MethodCall {
            object: Box::new(arg),
            method: "len".to_string(),
            args: vec![],
            receiver_convention: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: Some(
                super::super::method_call_metadata::source_call_with_first_arg_as_receiver(
                    call,
                    call.arguments.args[0].range(),
                ),
            ),
            ty: Type::Int,
        }),
        Type::Class { methods, .. } if methods.iter().any(|(name, _)| name == "len") => {
            Some(HirExpr::MethodCall {
                object: Box::new(arg),
                method: "len".to_string(),
                args: vec![],
                receiver_convention: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
                receiver_target: None,
                mutable_arg_places: Vec::new(),
                source: Some(
                    super::super::method_call_metadata::source_call_with_first_arg_as_receiver(
                        call,
                        call.arguments.args[0].range(),
                    ),
                ),
                ty: Type::Int,
            })
        }
        _ => {
            reject_type_mismatch(
                ctx,
                format!(
                    "len() argument must be a string, bytes, list, dict, tuple, set, or sized class, got '{}'",
                    arg_ty.display_name()
                ),
                call.arguments.args[0].range(),
            );
            None
        }
    }
}

pub(in crate::lower) fn lower_isinstance_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if call.arguments.args.len() != 2 {
        reject_wrong_positional_count(
            call,
            format!(
                "isinstance() takes exactly 2 arguments, got {}",
                call.arguments.args.len()
            ),
            ctx,
        );
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    resolve_annotation_expr(&call.arguments.args[1], ctx);
    let type_name = match &call.arguments.args[1] {
        Expr::Name(n) => n.id.to_string(),
        _ => "unknown".to_string(),
    };
    Some(HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "isinstance".to_string(),
        args: vec![arg, HirExpr::StringLiteral(type_name)],
        ty: Type::Bool,
    })
}

pub(in crate::lower) fn lower_reveal_type_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        reject_wrong_positional_count(
            call,
            format!(
                "reveal_type() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ),
            ctx,
        );
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let ty = arg.ty().clone();
    ctx.reveal_types.push(RevealTypeDiagnostic {
        revealed_type: ty.display_name(),
        primary_range: Some(call.arguments.args[0].range()),
    });
    Some(arg)
}

pub(in crate::lower) fn lower_range_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() > 3 {
        reject_wrong_positional_count(
            call,
            format!(
                "range() takes at most 3 positional arguments, got {}",
                call.arguments.args.len()
            ),
            ctx,
        );
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
            reject_unpacked_keyword_at("range", ctx, keyword.range);
            return None;
        };
        match name.as_str() {
            "start" => {
                if start_expr.is_some() {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_DUPLICATE_ARGUMENT,
                        "range() got multiple values for argument 'start'".to_string(),
                        name.range(),
                    );
                    return None;
                }
                start_expr = Some(&keyword.value);
            }
            "stop" => {
                if stop_expr.is_some() {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_DUPLICATE_ARGUMENT,
                        "range() got multiple values for argument 'stop'".to_string(),
                        name.range(),
                    );
                    return None;
                }
                stop_expr = Some(&keyword.value);
            }
            "step" => {
                if step_expr.is_some() {
                    ctx.error_with_code_at(
                        DiagnosticCode::CALL_DUPLICATE_ARGUMENT,
                        "range() got multiple values for argument 'step'".to_string(),
                        name.range(),
                    );
                    return None;
                }
                step_expr = Some(&keyword.value);
            }
            other => {
                ctx.error_with_code_at(
                    DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
                    format!("range() got an unexpected keyword argument '{other}'"),
                    name.range(),
                );
                return None;
            }
        }
    }

    let Some(stop_raw) = stop_expr else {
        ctx.error_with_code_at(
            DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT,
            "range() missing required argument 'stop'".to_string(),
            call.func.range(),
        );
        return None;
    };

    let start = if let Some(raw) = start_expr {
        let lowered = lower_expr(raw, ctx)?;
        if lowered.ty() != &Type::Int {
            reject_type_mismatch(
                ctx,
                format!(
                    "range() start argument must be 'int', got '{}'",
                    lowered.ty().display_name()
                ),
                raw.range(),
            );
            return None;
        }
        lowered
    } else {
        HirExpr::IntLiteral(0)
    };
    let stop = lower_expr(stop_raw, ctx)?;
    if stop.ty() != &Type::Int {
        reject_type_mismatch(
            ctx,
            format!(
                "range() stop argument must be 'int', got '{}'",
                stop.ty().display_name()
            ),
            stop_raw.range(),
        );
        return None;
    }
    let step = if let Some(raw) = step_expr {
        let lowered = lower_expr(raw, ctx)?;
        if lowered.ty() != &Type::Int {
            reject_type_mismatch(
                ctx,
                format!(
                    "range() step argument must be 'int', got '{}'",
                    lowered.ty().display_name()
                ),
                raw.range(),
            );
            return None;
        }
        if !super::super::integer_failure_diagnostics::is_proven_nonzero_integer_expr(&lowered, ctx)
        {
            ctx.error_with_code_at(
                DiagnosticCode::INT_EXACT_DIVISION_REQUIRES_HANDLING,
                "integer division, modulo, exponentiation, shift, or range step requires handling a typed integer failure unless the compiler can prove this operation is safe".to_string(),
                raw.range(),
            );
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

pub(in crate::lower) fn callable_builtin_element_type(arg_ty: &Type) -> Option<Type> {
    iterable_element_type_for_builtin(arg_ty)
}

pub(in crate::lower) fn callable_builtin_list_output_type(arg_ty: &Type) -> Option<Type> {
    list_constructor_output_type(arg_ty)
}

pub(in crate::lower) fn callable_builtin_dict_output_type(arg_ty: &Type) -> Option<Type> {
    dict_constructor_output_type(arg_ty)
}

pub(in crate::lower) fn lower_builtin_reverseable_arg(
    call: &ExprCall,
    builtin_name: &str,
    ctx: &mut LowerCtx,
) -> Option<(HirExpr, Type)> {
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        ctx.error_with_code_at(
            DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
            format!("{builtin_name}() takes exactly 1 positional argument"),
            if call.arguments.keywords.is_empty() {
                arity_range(call)
            } else {
                first_keyword_range(call)
            },
        );
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
        ctx.error_with_code_at(
            DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
            format!(
                "{builtin_name}() argument must be an iterable with a statically-known element type, got '{}'",
                arg.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if !arg
        .ty()
        .supports_iteration_capability(IterationCapability::DoubleEnded)
    {
        ctx.error_with_code_at(
            DiagnosticCode::PROTO_BOUND_NOT_SATISFIED,
            format!(
                "{builtin_name}() argument must be reversible, got '{}'",
                arg.ty().display_name()
            ),
            call.arguments.args[0].range(),
        );
        return None;
    }
    Some((arg, elem_ty))
}
