use super::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, StmtFunctionDef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WorkloadKind {
    BlockingIo,
    CpuHeavy,
}

impl WorkloadKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::BlockingIo => "blocking_io",
            Self::CpuHeavy => "cpu_heavy",
        }
    }

    pub(super) fn suggestion(self) -> &'static str {
        match self {
            Self::BlockingIo => "use an async API or task.spawn_blocking",
            Self::CpuHeavy => "use task.spawn_blocking or ThreadPoolExecutor",
        }
    }

    fn direct_call_code(self) -> DiagnosticCode {
        match self {
            Self::BlockingIo => DiagnosticCode::ASYNC_DIRECT_BLOCKING_IO_CALL,
            Self::CpuHeavy => DiagnosticCode::ASYNC_DIRECT_CPU_HEAVY_CALL,
        }
    }
}

pub(super) fn annotation_with_range<'a>(
    decorators: impl Iterator<Item = &'a sifr_python_ast::Decorator>,
) -> Option<(WorkloadKind, TextRange)> {
    let mut workload = None;
    for decorator in decorators {
        let Expr::Name(name) = &decorator.expression else {
            continue;
        };
        let kind = match name.id.as_str() {
            "blocking_io" => WorkloadKind::BlockingIo,
            "cpu_heavy" => WorkloadKind::CpuHeavy,
            _ => continue,
        };
        workload = Some((kind, decorator.expression.range()));
    }
    workload
}

pub(super) fn annotation_for_decorators<'a>(
    decorators: impl Iterator<Item = &'a sifr_python_ast::Decorator>,
) -> Option<WorkloadKind> {
    annotation_with_range(decorators).map(|(kind, _)| kind)
}

pub(super) fn reject_async_function_annotation(
    ctx: &mut LowerCtx,
    func: &StmtFunctionDef,
    effective_is_async: bool,
) {
    if !effective_is_async {
        return;
    }
    if let Some((workload, range)) = annotation_with_range(func.decorator_list.iter()) {
        ctx.error_with_code_at(
            DiagnosticCode::ASYNC_WORKLOAD_ANNOTATION_ON_ASYNC_DEF,
            format!(
                "@{} is only valid on sync def; async APIs use suspension effects instead",
                workload.label()
            ),
            range,
        );
    }
}

pub(super) fn reject_async_direct_call(ctx: &mut LowerCtx, function: &str, range: TextRange) {
    if !ctx.current_function_is_async {
        return;
    }
    let Some(workload) = ctx.function_workload_annotations.get(function).copied() else {
        return;
    };
    ctx.error_with_code_at(
        workload.direct_call_code(),
        format!(
            "{} function '{}' called directly from async context; {}",
            workload.label(),
            function,
            workload.suggestion()
        ),
        range,
    );
}
