use super::method_diagnostics::reject_no_method_args;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

pub(in crate::lower) fn resolve_async_generator_method_type(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    match method {
        "aclose" => {
            if !args.is_empty() {
                reject_no_method_args(ctx, "AsyncGenerator.aclose", arg_ranges, method_range);
                return None;
            }
            Some(Type::Awaitable(Box::new(Type::Result(
                Box::new(Type::None),
                Box::new(generator_close_error_type(ctx)),
            ))))
        }
        "send" | "throw" => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!(
                    "AsyncGenerator.{method}() is not supported in v1; consume async generators with async for, anext(), async comprehensions, or aclose()"
                ),
                method_range,
            );
            None
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!(
                    "type '{}' has no method '{method}'",
                    object_ty.display_name()
                ),
                method_range,
            );
            None
        }
    }
}

fn generator_close_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("GeneratorCloseError")
        .cloned()
        .unwrap_or(Type::Class {
            name: "GeneratorCloseError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        })
}
