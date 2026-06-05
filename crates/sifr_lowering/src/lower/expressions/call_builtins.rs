use super::{
    call_arity_range, callable_builtin_element_type, decimal_conversion_error_type,
    expression_diagnostics, first_call_keyword_range, float_sentinel_expr,
    float_sentinel_kind_from_call, is_hashable_type, lower_abs_call, lower_anext_call,
    lower_bigdecimal_constructor_call, lower_bytes_constructor_call, lower_chr_call,
    lower_decimal_constructor_call, lower_dict_constructor_call, lower_enumerate_call, lower_expr,
    lower_isinstance_call, lower_len_call, lower_list_constructor_call, lower_ord_call,
    lower_range_call, lower_reveal_type_call, lower_reversed_call, lower_set_constructor_call,
    lower_sorted_call, lower_sum_call, lower_tuple_constructor_call, lower_zip_call,
    normalize_min_max_numeric_sentinels, str, validate_variadic_min_max_operands, DiagnosticCode,
    ExprCall, HirExpr, HirIteratorOp, LowerCtx, Ranged, Type,
};
pub(super) enum CallLowering {
    Lowered(HirExpr),
    NoMatch,
}

impl CallLowering {
    pub(super) fn from_option(expr: Option<HirExpr>) -> Option<Self> {
        expr.map(Self::Lowered)
    }
}

pub(super) fn lower_unshadowed_builtin_call(
    func_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<CallLowering> {
    if func_name == "list" {
        return CallLowering::from_option(lower_list_constructor_call(call, ctx));
    }

    if func_name == "tuple" {
        return CallLowering::from_option(lower_tuple_constructor_call(call, ctx));
    }

    if func_name == "dict" {
        return CallLowering::from_option(lower_dict_constructor_call(call, ctx));
    }

    if func_name == "set" {
        return CallLowering::from_option(lower_set_constructor_call(call, ctx));
    }

    if func_name == "bytes" {
        return CallLowering::from_option(lower_bytes_constructor_call(call, ctx));
    }

    if func_name == "ord" {
        return CallLowering::from_option(lower_ord_call(call, ctx));
    }

    if func_name == "chr" {
        return CallLowering::from_option(lower_chr_call(call, ctx));
    }

    // Special handling for range() built-in
    if func_name == "range" {
        return CallLowering::from_option(lower_range_call(call, ctx));
    }

    // Special handling for len() built-in
    if func_name == "len" {
        return CallLowering::from_option(lower_len_call(call, ctx));
    }

    // iter(iterable) -> Iterator[T]
    if func_name == "iter" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "iter() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() != 1 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                format!(
                    "iter() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ),
                call_arity_range(call),
            );
            return None;
        }
        let iterable = lower_expr(&call.arguments.args[0], ctx)?;
        if matches!(iterable.ty().resolve_alias(), Type::Any | Type::Unknown) {
            expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "iter() argument must be an iterable with a statically-known element type, got '{}'",
                        iterable.ty().display_name()
                    ),
                    call.arguments.args[0].range(),
                );
            return None;
        }
        let Some(elem_ty) = callable_builtin_element_type(iterable.ty()) else {
            if matches!(iterable.ty().resolve_alias(), Type::Tuple(_)) {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT,
                    "iter() tuple argument must have one statically provable element type"
                        .to_string(),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "iter() argument must be iterable, got '{}'",
                    iterable.ty().display_name()
                ),
                call.arguments.args[0].range(),
            );
            return None;
        };
        return Some(CallLowering::Lowered(HirExpr::IteratorCall {
            op: HirIteratorOp::Iter,
            args: vec![iterable],
            ty: Type::Iterator(Box::new(elem_ty)),
        }));
    }

    // next(iterator) -> Option[T]
    if func_name == "next" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "next() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() != 1 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                format!(
                    "next() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ),
                call_arity_range(call),
            );
            return None;
        }
        let iterator = lower_expr(&call.arguments.args[0], ctx)?;
        let Some(elem_ty) = iterator.ty().iterator_element_type() else {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "next() argument must be an iterator, got '{}'",
                    iterator.ty().display_name()
                ),
                call.arguments.args[0].range(),
            );
            return None;
        };
        return Some(CallLowering::Lowered(HirExpr::IteratorCall {
            op: HirIteratorOp::Next,
            args: vec![iterator],
            ty: Type::Union(vec![elem_ty, Type::None]),
        }));
    }

    // anext(async_iterator) -> Awaitable[Result[Option[T], E]]
    if func_name == "anext" {
        return CallLowering::from_option(lower_anext_call(call, ctx));
    }

    // Special handling for isinstance() built-in
    if func_name == "isinstance" {
        return CallLowering::from_option(lower_isinstance_call(call, ctx));
    }

    // Special handling for reveal_type() built-in
    if func_name == "reveal_type" {
        return CallLowering::from_option(lower_reveal_type_call(call, ctx));
    }

    // Special handling for str() conversion
    if func_name == "str" {
        if call.arguments.args.len() == 1 {
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            return Some(CallLowering::Lowered(HirExpr::Call {
                func: "str".to_string(),
                args: vec![arg],
                ty: Type::Str,
            }));
        }
    }

    // pow(base, exp) -> base ** exp
    if func_name == "pow" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "pow() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() != 2 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                "pow() takes exactly 2 arguments".to_string(),
                call_arity_range(call),
            );
            return None;
        }
        let base = lower_expr(&call.arguments.args[0], ctx)?;
        let exp = lower_expr(&call.arguments.args[1], ctx)?;
        let result_ty = if base.ty() == &Type::Int && exp.ty() == &Type::Int {
            Type::Int
        } else {
            Type::Float
        };
        return Some(CallLowering::Lowered(HirExpr::Call {
            func: "pow".to_string(),
            args: vec![base, exp],
            ty: result_ty,
        }));
    }

    // Special handling for abs() built-in
    if func_name == "abs" {
        return CallLowering::from_option(lower_abs_call(call, ctx));
    }

    // Special handling for hash() built-in
    if func_name == "hash" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "hash() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() != 1 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                format!(
                    "hash() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ),
                call_arity_range(call),
            );
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let ty = arg.ty().clone();
        // Check if the type is hashable
        if !is_hashable_type(&ty) {
            let type_name = ty.display_name();
            ctx.error_with_code_at(
                DiagnosticCode::PROTO_HASHABLE_OR_COMPARABLE_REQUIRED,
                format!("hash() argument must be hashable, got '{type_name}'"),
                call.arguments.args[0].range(),
            );
            return None;
        }
        return Some(CallLowering::Lowered(HirExpr::Call {
            func: "hash".to_string(),
            args: vec![arg],
            ty: Type::Int,
        }));
    }

    // Special handling for round() built-in
    if func_name == "round" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "round() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.is_empty() || call.arguments.args.len() > 2 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                format!(
                    "round() takes 1 or 2 arguments, got {}",
                    call.arguments.args.len()
                ),
                call_arity_range(call),
            );
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        if !arg.ty().is_numeric() {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "round() argument must be numeric, got '{}'",
                    arg.ty().display_name()
                ),
                call.arguments.args[0].range(),
            );
            return None;
        }
        if call.arguments.args.len() == 2 {
            let ndigits = lower_expr(&call.arguments.args[1], ctx)?;
            return Some(CallLowering::Lowered(HirExpr::Call {
                func: "round".to_string(),
                args: vec![arg, ndigits],
                ty: Type::Float,
            }));
        }
        return Some(CallLowering::Lowered(HirExpr::Call {
            func: "round".to_string(),
            args: vec![arg],
            ty: Type::Int,
        }));
    }

    // Special handling for repr() built-in
    if func_name == "repr" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "repr() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() != 1 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                format!(
                    "repr() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ),
                call_arity_range(call),
            );
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(CallLowering::Lowered(HirExpr::Call {
            func: "repr".to_string(),
            args: vec![arg],
            ty: Type::Str,
        }));
    }

    if func_name == "Decimal" {
        return CallLowering::from_option(lower_decimal_constructor_call(call, ctx));
    }

    if func_name == "BigDecimal" {
        return CallLowering::from_option(lower_bigdecimal_constructor_call(call, ctx));
    }

    // Special handling for int() conversion
    if func_name == "int" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "int() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() != 1 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                format!(
                    "int() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ),
                call_arity_range(call),
            );
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        // int(str) -> Result[int, ParseError] (fallible)
        // int(float) -> int (infallible truncation)
        // int(int) -> int (identity)
        // int(bool) -> int (True=1, False=0)
        // int(bigint) -> Result[int, OverflowError] (may overflow i64)
        // int(decimal|bigdecimal) -> Result[int, DecimalConversionError] (truncate toward zero)
        let result_ty = if arg_ty == Type::Str {
            let parse_error_ty =
                ctx.class_types
                    .get("ParseError")
                    .cloned()
                    .unwrap_or(Type::Class {
                        name: "ParseError".to_string(),
                        fields: vec![("message".to_string(), Type::Str)],
                        methods: vec![],
                        parent_class: None,
                    });
            Type::Result(Box::new(Type::Int), Box::new(parse_error_ty))
        } else if arg_ty == Type::BigInt {
            let overflow_error_ty =
                ctx.class_types
                    .get("OverflowError")
                    .cloned()
                    .unwrap_or(Type::Class {
                        name: "OverflowError".to_string(),
                        fields: vec![("message".to_string(), Type::Str)],
                        methods: vec![],
                        parent_class: None,
                    });
            Type::Result(Box::new(Type::Int), Box::new(overflow_error_ty))
        } else if matches!(arg_ty, Type::Decimal | Type::BigDecimal) {
            Type::Result(
                Box::new(Type::Int),
                Box::new(decimal_conversion_error_type(ctx)),
            )
        } else {
            Type::Int
        };
        return Some(CallLowering::Lowered(HirExpr::Call {
            func: "int".to_string(),
            args: vec![arg],
            ty: result_ty,
        }));
    }

    // bigint(n) — convert int|bigint|decimal|bigdecimal to bigint
    if func_name == "bigint" {
        ctx.warn_bigint_transition_alias(call.func.range());
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "bigint() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() != 1 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                format!(
                    "bigint() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ),
                call_arity_range(call),
            );
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        if !matches!(
            arg_ty,
            Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::Decimal | Type::BigDecimal
        ) {
            expression_diagnostics::type_mismatch(
                ctx,
                format!(
                    "bigint() requires int, bigint, decimal, or bigdecimal argument, got '{}'",
                    arg_ty.display_name()
                ),
                call.arguments.args[0].range(),
            );
            return None;
        }
        return Some(CallLowering::Lowered(HirExpr::Call {
            func: "bigint".to_string(),
            args: vec![arg],
            ty: Type::BigInt,
        }));
    }

    if func_name == "float" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "float() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() != 1 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                format!(
                    "float() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ),
                call_arity_range(call),
            );
            return None;
        }
        if let Some(kind) = float_sentinel_kind_from_call(call) {
            return Some(CallLowering::Lowered(float_sentinel_expr(kind)));
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        let result_ty = if arg_ty == Type::Str {
            let parse_error_ty =
                ctx.class_types
                    .get("ParseError")
                    .cloned()
                    .unwrap_or(Type::Class {
                        name: "ParseError".to_string(),
                        fields: vec![("message".to_string(), Type::Str)],
                        methods: vec![],
                        parent_class: None,
                    });
            Type::Result(Box::new(Type::Float), Box::new(parse_error_ty))
        } else if arg_ty == Type::Decimal {
            ctx.error_with_code_at(
                    DiagnosticCode::DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
                    "float(decimal_value) is not allowed; decimal values are exact and cannot be converted to float"
                        .to_string(),
                    call.arguments.args[0].range(),
                );
            return None;
        } else if arg_ty == Type::BigDecimal {
            ctx.error_with_code_at(
                    DiagnosticCode::DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
                    "float(bigdecimal_value) is not allowed; bigdecimal values are exact and cannot be converted to float"
                        .to_string(),
                    call.arguments.args[0].range(),
                );
            return None;
        } else {
            Type::Float
        };
        return Some(CallLowering::Lowered(HirExpr::Call {
            func: "float".to_string(),
            args: vec![arg],
            ty: result_ty,
        }));
    }

    // Special handling for bool() conversion
    if func_name == "bool" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "bool() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() != 1 {
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                format!(
                    "bool() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ),
                call_arity_range(call),
            );
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(CallLowering::Lowered(HirExpr::Call {
            func: "bool".to_string(),
            args: vec![arg],
            ty: Type::Bool,
        }));
    }

    if func_name == "min" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "min() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() >= 2 {
            let mut args = Vec::with_capacity(call.arguments.args.len());
            for arg in &call.arguments.args {
                args.push(lower_expr(arg, ctx)?);
            }

            let mut result_ty = args[0].ty().clone();
            for index in 1..args.len() {
                let (left, right, pair_result_ty) = normalize_min_max_numeric_sentinels(
                    &call.arguments.args[index - 1],
                    &call.arguments.args[index],
                    args[index - 1].clone(),
                    args[index].clone(),
                    ctx,
                );
                args[index - 1] = left;
                args[index] = right;
                result_ty = pair_result_ty;
            }

            if !validate_variadic_min_max_operands("min", &args, &call.arguments.args, ctx) {
                return None;
            }
            return Some(CallLowering::Lowered(HirExpr::Call {
                func: "min".to_string(),
                args,
                ty: result_ty,
            }));
        } else if call.arguments.args.len() == 1 {
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
                expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "min() argument must be an iterable with a statically-known element type, got '{}'",
                            arg.ty().display_name()
                        ),
                        call.arguments.args[0].range(),
                    );
                return None;
            };
            return Some(CallLowering::Lowered(HirExpr::Call {
                func: "min".to_string(),
                args: vec![arg],
                ty: Type::Union(vec![elem_ty, Type::None]),
            }));
        }
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "min() takes at least 1 argument".to_string(),
            call.func.range(),
        );
        return None;
    }
    if func_name == "max" {
        if !call.arguments.keywords.is_empty() {
            expression_diagnostics::call_unexpected_keyword(
                ctx,
                "max() does not accept keyword arguments".to_string(),
                first_call_keyword_range(call),
            );
            return None;
        }
        if call.arguments.args.len() >= 2 {
            let mut args = Vec::with_capacity(call.arguments.args.len());
            for arg in &call.arguments.args {
                args.push(lower_expr(arg, ctx)?);
            }

            let mut result_ty = args[0].ty().clone();
            for index in 1..args.len() {
                let (left, right, pair_result_ty) = normalize_min_max_numeric_sentinels(
                    &call.arguments.args[index - 1],
                    &call.arguments.args[index],
                    args[index - 1].clone(),
                    args[index].clone(),
                    ctx,
                );
                args[index - 1] = left;
                args[index] = right;
                result_ty = pair_result_ty;
            }

            if !validate_variadic_min_max_operands("max", &args, &call.arguments.args, ctx) {
                return None;
            }
            return Some(CallLowering::Lowered(HirExpr::Call {
                func: "max".to_string(),
                args,
                ty: result_ty,
            }));
        } else if call.arguments.args.len() == 1 {
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
                expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "max() argument must be an iterable with a statically-known element type, got '{}'",
                            arg.ty().display_name()
                        ),
                        call.arguments.args[0].range(),
                    );
                return None;
            };
            return Some(CallLowering::Lowered(HirExpr::Call {
                func: "max".to_string(),
                args: vec![arg],
                ty: Type::Union(vec![elem_ty, Type::None]),
            }));
        }
        expression_diagnostics::call_wrong_positional_count(
            ctx,
            "max() takes at least 1 argument".to_string(),
            call.func.range(),
        );
        return None;
    }
    if func_name == "sum" {
        return CallLowering::from_option(lower_sum_call(call, ctx));
    }
    if func_name == "sorted" {
        return CallLowering::from_option(lower_sorted_call(call, ctx));
    }

    // reversed(iterable) -> iterator of element type
    if func_name == "reversed" {
        return CallLowering::from_option(lower_reversed_call(call, ctx));
    }

    // enumerate(iterable) -> iterator of (int, element) tuples
    if func_name == "enumerate" {
        return CallLowering::from_option(lower_enumerate_call(call, ctx));
    }

    if func_name == "zip" {
        return CallLowering::from_option(lower_zip_call(call, ctx));
    }

    Some(CallLowering::NoMatch)
}
