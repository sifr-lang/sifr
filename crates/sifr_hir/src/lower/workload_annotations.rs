use super::{LowerCtx, LoweringWarningDiagnostic};
use ruff_text_size::TextRange;
use sifr_python_ast::Expr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WorkloadKind {
    BlockingIo,
    CpuHeavy,
}

impl WorkloadKind {
    fn label(self) -> &'static str {
        match self {
            Self::BlockingIo => "blocking_io",
            Self::CpuHeavy => "cpu_heavy",
        }
    }

    fn suggestion(self) -> &'static str {
        match self {
            Self::BlockingIo => "use an async API or task.spawn_blocking",
            Self::CpuHeavy => "use task.spawn_blocking or ThreadPoolExecutor",
        }
    }
}

pub(super) fn annotation_for_decorators<'a>(
    decorators: impl Iterator<Item = &'a sifr_python_ast::Decorator>,
) -> Option<WorkloadKind> {
    let mut workload = None;
    for decorator in decorators {
        let Expr::Name(name) = &decorator.expression else {
            continue;
        };
        match name.id.as_str() {
            "blocking_io" => workload = Some(WorkloadKind::BlockingIo),
            "cpu_heavy" => workload = Some(WorkloadKind::CpuHeavy),
            _ => {}
        }
    }
    workload
}

pub(super) fn warn_async_direct_call(ctx: &mut LowerCtx, function: &str, range: TextRange) {
    if !ctx.current_function_is_async {
        return;
    }
    let Some(workload) = ctx.function_workload_annotations.get(function).copied() else {
        return;
    };
    ctx.warnings
        .push(LoweringWarningDiagnostic::BlockingWorkInAsync {
            function: function.to_string(),
            workload: workload.label().to_string(),
            suggestion: workload.suggestion().to_string(),
            primary_range: Some(range),
        });
}
