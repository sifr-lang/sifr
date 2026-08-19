use super::{HirFunction, LowerCtx, StmtClassDef, Type};
use crate::lower::python_interop::validate_context_class_methods;
use sifr_type_system::ReceiverConvention;

pub(super) fn validate(class: &StmtClassDef, methods: &[HirFunction], ctx: &mut LowerCtx) {
    let class_name = class.name.as_str();
    let cleanup = ctx
        .python_opaque_classes
        .get(class_name)
        .and_then(|declaration| declaration.cleanup);
    validate_context_class_methods(class_name, methods, cleanup, ctx, class.range);
    let close_count = methods
        .iter()
        .filter(|method| {
            is_semantic_cleanup(
                method,
                sifr_ir::PythonInteropDecoratorKind::Function,
                "close",
            )
        })
        .count();
    if cleanup == Some(sifr_ir::PythonCleanupPolicy::Close) && close_count != 1 {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYCALL_INVALID_SHAPE,
            "`cleanup=close` requires exactly one `@python(Self.close)` method declared as `def close(own self) -> Result[None, PythonError]`".to_string(),
            class.range,
        );
    }
    let async_close_count = methods
        .iter()
        .filter(|method| {
            is_semantic_cleanup(
                method,
                sifr_ir::PythonInteropDecoratorKind::Coroutine,
                "aclose",
            )
        })
        .count();
    if cleanup == Some(sifr_ir::PythonCleanupPolicy::AsyncClose) && async_close_count != 1 {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYCALL_INVALID_SHAPE,
            "`cleanup=async_close` requires exactly one `@python.coroutine(Self.aclose)` method declared as `async def aclose(own self) -> Result[None, PythonError]`".to_string(),
            class.range,
        );
    }
    if methods
        .iter()
        .any(|method| has_unmatched_consuming_declaration(method, cleanup))
    {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYCALL_INVALID_SHAPE,
            "a consuming Python method is reserved for the declared semantic cleanup operation"
                .to_string(),
            class.range,
        );
    }
}

fn is_semantic_cleanup(
    method: &HirFunction,
    kind: sifr_ir::PythonInteropDecoratorKind,
    target_member: &str,
) -> bool {
    method.receiver == Some(ReceiverConvention::Owned)
        && method.python_interop.first().is_some_and(|declaration| {
            declaration.consumes_receiver
                && declaration.kind == kind
                && declaration
                    .target
                    .as_ref()
                    .is_some_and(|target| target.segments.as_slice() == ["Self", target_member])
                && method.params.is_empty()
                && matches!(
                    method.return_type.resolve_alias(),
                    Type::Result(ok, _) if ok.resolve_alias() == &Type::None
                )
        })
}

fn has_unmatched_consuming_declaration(
    method: &HirFunction,
    cleanup: Option<sifr_ir::PythonCleanupPolicy>,
) -> bool {
    method.python_interop.first().is_some_and(|declaration| {
        if !declaration.consumes_receiver {
            return false;
        }
        match cleanup {
            Some(sifr_ir::PythonCleanupPolicy::Close) => {
                declaration.kind != sifr_ir::PythonInteropDecoratorKind::Function
                    || declaration
                        .target
                        .as_ref()
                        .is_none_or(|target| target.segments.as_slice() != ["Self", "close"])
            }
            Some(sifr_ir::PythonCleanupPolicy::AsyncClose) => {
                declaration.kind != sifr_ir::PythonInteropDecoratorKind::Coroutine
                    || declaration
                        .target
                        .as_ref()
                        .is_none_or(|target| target.segments.as_slice() != ["Self", "aclose"])
            }
            Some(sifr_ir::PythonCleanupPolicy::Context) => {
                declaration.kind != sifr_ir::PythonInteropDecoratorKind::ContextExit
            }
            Some(sifr_ir::PythonCleanupPolicy::AsyncContext) => {
                declaration.kind != sifr_ir::PythonInteropDecoratorKind::ContextAsyncExit
            }
            _ => true,
        }
    })
}
