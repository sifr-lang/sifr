use super::{
    expression_diagnostics, reject_exact_method_arg_count, reject_no_method_args, HirExpr,
    LowerCtx, TextRange, Type,
};

pub(super) fn resolve_python_buffer_method_type(
    element: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    let python_error = ctx
        .class_types
        .get("PythonError")
        .cloned()
        .unwrap_or(Type::Any);
    let result = |ok: Type| Type::Result(Box::new(ok), Box::new(python_error.clone()));
    match method {
        "length" | "item_size" | "dimensions" => {
            require_no_args(method, args, arg_ranges, method_range, ctx)?;
            Some(Type::Int)
        }
        "shape" | "strides" | "suboffsets" => {
            require_no_args(method, args, arg_ranges, method_range, ctx)?;
            Some(Type::List(Box::new(Type::Int)))
        }
        "format" => {
            require_no_args(method, args, arg_ranges, method_range, ctx)?;
            Some(Type::Str)
        }
        "readonly" | "c_contiguous" | "f_contiguous" => {
            require_no_args(method, args, arg_ranges, method_range, ctx)?;
            Some(Type::Bool)
        }
        "read" => {
            require_exact_args(method, 1, args, arg_ranges, method_range, ctx)?;
            require_arg_type(method, &args[0], &Type::Int, arg_ranges[0], ctx);
            Some(result(element.clone()))
        }
        "write" => {
            require_exact_args(method, 2, args, arg_ranges, method_range, ctx)?;
            require_arg_type(method, &args[0], &Type::Int, arg_ranges[0], ctx);
            require_arg_type(method, &args[1], element, arg_ranges[1], ctx);
            Some(result(Type::None))
        }
        "copy_slice" => {
            require_exact_args(method, 2, args, arg_ranges, method_range, ctx)?;
            require_arg_type(method, &args[0], &Type::Int, arg_ranges[0], ctx);
            require_arg_type(method, &args[1], &Type::Int, arg_ranges[1], ctx);
            Some(result(Type::List(Box::new(element.clone()))))
        }
        "release" => {
            require_no_args(method, args, arg_ranges, method_range, ctx)?;
            Some(result(Type::None))
        }
        _ => {
            ctx.error_with_code_at(
                sifr_diagnostics::DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!(
                    "type 'python.Buffer[{}]' has no method '{method}'",
                    element.display_name()
                ),
                method_range,
            );
            None
        }
    }
}

pub(super) fn consume_python_buffer_release_receiver(
    object: &HirExpr,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let HirExpr::Name { name, .. } = object else {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
            "explicit Python buffer release requires an owning local binding".to_string(),
            range,
        );
        return false;
    };
    if ctx.borrowed_params.contains(name) {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
            format!(
                "cannot release borrowed Python buffer `{name}`; accept it with `own` or release its owning local binding"
            ),
            range,
        );
        return false;
    }
    ctx.mark_moved_with_flow(name);
    true
}

fn require_no_args(
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<()> {
    if args.is_empty() {
        Some(())
    } else {
        reject_no_method_args(
            ctx,
            &format!("python.Buffer.{method}"),
            arg_ranges,
            method_range,
        );
        None
    }
}

fn require_exact_args(
    method: &str,
    expected: usize,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<()> {
    if args.len() == expected {
        Some(())
    } else {
        reject_exact_method_arg_count(
            ctx,
            &format!("python.Buffer.{method}"),
            expected,
            args.len(),
            arg_ranges,
            method_range,
        );
        None
    }
}

fn require_arg_type(
    method: &str,
    arg: &HirExpr,
    expected: &Type,
    range: TextRange,
    ctx: &mut LowerCtx,
) {
    if !arg.ty().is_assignable_to(expected) {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "python.Buffer.{method}() expected `{}`, got `{}`",
                expected.display_name(),
                arg.ty().display_name()
            ),
            range,
        );
    }
}
