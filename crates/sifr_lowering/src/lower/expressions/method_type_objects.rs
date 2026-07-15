use super::{
    canonicalize_class_surface_type, coroutine_result_type, expression_diagnostics,
    method_count_range, reject_exact_method_arg_count, reject_method_arg_count,
    reject_no_method_args, resolve_method_type, resolve_str_encode_method_type, str,
    DiagnosticCode, FunctionType, HirExpr, LowerCtx, TextRange, Type,
};
pub(super) fn resolve_str_method_type(
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    match method {
        "len" => Some(Type::Int),
        "upper" | "lower" | "strip" | "lstrip" | "rstrip" | "title" | "capitalize" | "swapcase" => {
            Some(Type::Str)
        }
        "startswith" | "endswith" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    &format!("str.{method}"),
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::Bool)
        }
        "isdigit" | "isalpha" | "isalnum" | "isspace" | "isupper" | "islower" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, &format!("str.{method}"), arg_ranges, method_range);
                return None;
            }
            Some(Type::Bool)
        }
        "split" => {
            if args.len() > 2 {
                reject_method_arg_count(
                    ctx,
                    format!("str.split() takes 0 to 2 arguments, got {}", args.len()),
                    method_count_range(args.len(), 2, arg_ranges, method_range),
                );
                return None;
            }
            if let Some(maxsplit) = args.get(1) {
                if maxsplit.ty() != &Type::Int {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "str.split() maxsplit must be 'int', got '{}'",
                            maxsplit.ty().display_name()
                        ),
                        arg_ranges[1],
                    );
                }
            }
            Some(Type::List(Box::new(Type::Str)))
        }
        "replace" => {
            if args.len() < 2 || args.len() > 3 {
                reject_method_arg_count(
                    ctx,
                    format!("str.replace() takes 2 or 3 arguments, got {}", args.len()),
                    method_count_range(args.len(), 3, arg_ranges, method_range),
                );
                return None;
            }
            if let Some(count) = args.get(2) {
                if count.ty() != &Type::Int {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "str.replace() count must be 'int', got '{}'",
                            count.ty().display_name()
                        ),
                        arg_ranges[2],
                    );
                }
            }
            Some(Type::Str)
        }
        "join" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "str.join",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::Str)
        }
        "count" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "str.count",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::Int)
        }
        "center" | "ljust" | "rjust" | "zfill" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    &format!("str.{method}"),
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::Str)
        }
        "find" | "rfind" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    &format!("str.{method}"),
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            // find()/rfind() return Option[int] = int | None
            Some(Type::Union(vec![Type::Int, Type::None]))
        }
        "encode" => resolve_str_encode_method_type(args, arg_ranges, method_range, ctx),
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!("str has no method '{method}'"),
                method_range,
            );
            None
        }
    }
}

pub(super) fn resolve_tuple_method_type(
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    match method {
        "len" => Some(Type::Int),
        "count" => {
            if args.len() != 1 {
                reject_exact_method_arg_count(
                    ctx,
                    "tuple.count",
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            Some(Type::Int)
        }
        "index" => {
            if args.is_empty() || args.len() > 3 {
                reject_method_arg_count(
                    ctx,
                    format!("tuple.index() takes 1 to 3 arguments, got {}", args.len()),
                    method_count_range(args.len(), 3, arg_ranges, method_range),
                );
                return None;
            }
            for (bound_index, bound) in args.iter().enumerate().skip(1) {
                if bound.ty() != &Type::Int {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "tuple.index() bounds must be 'int', got '{}'",
                            bound.ty().display_name()
                        ),
                        arg_ranges.get(bound_index).copied().unwrap_or(method_range),
                    );
                }
            }
            Some(Type::Union(vec![Type::Int, Type::None]))
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!("tuple has no method '{method}'"),
                method_range,
            );
            None
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ClassMethodSurface<'a> {
    pub(super) name: &'a str,
    pub(super) fields: &'a [(String, Type)],
    pub(super) methods: &'a [(String, FunctionType)],
}

fn type_contains_exact(container: &Type, candidate: &Type) -> bool {
    if container.resolve_alias() == candidate.resolve_alias() {
        return true;
    }
    match container.resolve_alias() {
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::Newtype { inner, .. } => type_contains_exact(inner, candidate),
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Coroutine(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => {
            type_contains_exact(left, candidate) || type_contains_exact(right, candidate)
        }
        Type::Tuple(values) | Type::Union(values) | Type::Intersection(values) => values
            .iter()
            .any(|value| type_contains_exact(value, candidate)),
        _ => false,
    }
}

pub(super) fn resolve_class_method_type(
    class: ClassMethodSurface<'_>,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    if let Some((_, ft)) = class.methods.iter().find(|(n, _)| n == method) {
        let specialization_needs_clone = class.fields.iter().any(|(_, ty)| {
            !ty.supports_derived_clone() && type_contains_exact(&ft.return_type, ty)
        }) || ft.params.iter().any(|(_, ty, convention)| {
            convention.is_borrowed()
                && !ty.supports_derived_clone()
                && type_contains_exact(&ft.return_type, ty)
        });
        let has_dedicated_channel_transfer_diagnostic = method == "send"
            && class
                .name
                .rsplit('.')
                .next()
                .is_some_and(|name| name == "ChannelSender");
        if specialization_needs_clone
            && !has_dedicated_channel_transfer_diagnostic
            && ctx
                .class_declared_type_params
                .get(class.name)
                .is_some_and(|params| !params.is_empty())
        {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "{}.{}() is unavailable for this specialization because generated generic class methods require Clone-capable type arguments",
                    class.name, method
                ),
                method_range,
            );
            return None;
        }
        // Check argument count
        if args.len() != ft.params.len() {
            reject_method_arg_count(
                ctx,
                format!(
                    "{}.{}() takes {} argument(s), got {}",
                    class.name,
                    method,
                    ft.params.len(),
                    args.len()
                ),
                method_count_range(args.len(), ft.params.len(), arg_ranges, method_range),
            );
            return None;
        }
        // Check argument types
        for (i, (arg, (param_name, param_ty, _))) in args.iter().zip(ft.params.iter()).enumerate() {
            if !arg.ty().is_assignable_to(param_ty) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "argument {} ('{}') of {}.{}(): expected '{}', got '{}'",
                        i + 1,
                        param_name,
                        class.name,
                        method,
                        param_ty.display_name(),
                        arg.ty().display_name()
                    ),
                    arg_ranges.get(i).copied().unwrap_or(method_range),
                );
            }
        }
        Some(canonicalize_class_surface_type(&ft.return_type))
    } else if let Some((_, field_ty)) = class.fields.iter().find(|(n, _)| n == method) {
        // Callable fields use method-call syntax; async callable fields produce a coroutine.
        if let Type::Callable(param_types, _, ret_type)
        | Type::AsyncCallable(param_types, _, ret_type) = field_ty
        {
            if args.len() != param_types.len() {
                expression_diagnostics::call_not_callable_or_arity(
                    ctx,
                    format!(
                        "{}.{}() (callable field) takes {} argument(s), got {}",
                        class.name,
                        method,
                        param_types.len(),
                        args.len()
                    ),
                    method_count_range(args.len(), param_types.len(), arg_ranges, method_range),
                );
                return None;
            }
            for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
                if !arg.ty().is_assignable_to(param_ty) {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "argument {} of {}.{}(): expected '{}', got '{}'",
                            i + 1,
                            class.name,
                            method,
                            param_ty.display_name(),
                            arg.ty().display_name()
                        ),
                        arg_ranges.get(i).copied().unwrap_or(method_range),
                    );
                }
            }
            let return_type = canonicalize_class_surface_type(ret_type);
            if matches!(field_ty, Type::AsyncCallable(..)) {
                Some(coroutine_result_type(&return_type))
            } else {
                Some(return_type)
            }
        } else {
            expression_diagnostics::call_not_callable_or_arity(
                ctx,
                format!(
                    "field '{}' of class '{}' is not callable (type: '{}')",
                    method,
                    class.name,
                    field_ty.display_name()
                ),
                method_range,
            );
            None
        }
    } else {
        ctx.error_with_code_at(
            DiagnosticCode::CLASS_MISSING_MEMBER,
            format!("class '{}' has no method '{method}'", class.name),
            method_range,
        );
        None
    }
}

pub(super) fn resolve_protocol_method_type(
    name: &str,
    methods: &[(String, FunctionType)],
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) {
        if args.len() != ft.params.len() {
            reject_method_arg_count(
                ctx,
                format!(
                    "{}.{}() takes {} argument(s), got {}",
                    name,
                    method,
                    ft.params.len(),
                    args.len()
                ),
                method_count_range(args.len(), ft.params.len(), arg_ranges, method_range),
            );
            return None;
        }
        Some(canonicalize_class_surface_type(&ft.return_type))
    } else {
        ctx.error_with_code_at(
            DiagnosticCode::PROTO_BOUND_NOT_SATISFIED,
            format!("protocol '{name}' has no method '{method}'"),
            method_range,
        );
        None
    }
}

pub(super) fn resolve_newtype_method_type(
    name: &str,
    inner: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    // Newtype has a built-in `value()` method that returns the inner type
    if method == "value" {
        if !args.is_empty() {
            reject_no_method_args(ctx, &format!("{name}.value"), arg_ranges, method_range);
            return None;
        }
        Some(inner.clone())
    } else {
        // Delegate to the inner type's methods
        resolve_method_type(inner, method, args, arg_ranges, method_range, ctx)
    }
}

pub(super) fn resolve_enum_method_type(
    name: &str,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    match method {
        "name" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, &format!("{name}.name"), arg_ranges, method_range);
                return None;
            }
            Some(Type::Str)
        }
        "value" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, &format!("{name}.value"), arg_ranges, method_range);
                return None;
            }
            Some(Type::Int)
        }
        _ => {
            // Check user-defined methods registered in functions
            let method_key = format!("{name}.{method}");
            if let Some(ft) = ctx.functions.get(&method_key).cloned() {
                return Some(*ft.return_type.clone());
            }
            ctx.error_with_code_at(
                DiagnosticCode::CLASS_MISSING_MEMBER,
                format!("enum '{name}' has no method '{method}'"),
                method_range,
            );
            None
        }
    }
}

pub(super) fn resolve_bigint_method_type(
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    if method == "clone" {
        if !args.is_empty() {
            reject_no_method_args(ctx, "bigint.clone", arg_ranges, method_range);
            return None;
        }
        Some(Type::BigInt)
    } else {
        ctx.error_with_code_at(
            DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
            format!("type 'bigint' has no method '{method}'"),
            method_range,
        );
        None
    }
}
