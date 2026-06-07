use super::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, StmtFunctionDef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::lower) enum WorkloadKind {
    BlockingIo,
    CpuHeavy,
}

impl WorkloadKind {
    pub(in crate::lower) fn label(self) -> &'static str {
        match self {
            Self::BlockingIo => "blocking_io",
            Self::CpuHeavy => "cpu_heavy",
        }
    }

    pub(in crate::lower) fn suggestion(self) -> &'static str {
        match self {
            Self::BlockingIo => "use an async API or task.spawn_blocking",
            Self::CpuHeavy => "use task.spawn_cpu",
        }
    }

    fn direct_call_code(self) -> DiagnosticCode {
        match self {
            Self::BlockingIo => DiagnosticCode::ASYNC_DIRECT_BLOCKING_IO_CALL,
            Self::CpuHeavy => DiagnosticCode::ASYNC_DIRECT_CPU_HEAVY_CALL,
        }
    }
}

pub(in crate::lower) fn annotation_with_range<'a>(
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

pub(in crate::lower) fn annotation_for_decorators<'a>(
    decorators: impl Iterator<Item = &'a sifr_python_ast::Decorator>,
) -> Option<WorkloadKind> {
    annotation_with_range(decorators).map(|(kind, _)| kind)
}

pub(in crate::lower) fn reject_async_function_annotation(
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

pub(in crate::lower) fn reject_async_direct_call(
    ctx: &mut LowerCtx,
    function: &str,
    range: TextRange,
) {
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

pub(in crate::lower) fn reject_unclassified_offload_target(
    ctx: &mut LowerCtx,
    target: &Expr,
    api_name: &str,
) -> bool {
    let Some(function) = target_name(target) else {
        ctx.error_with_code_at(
            DiagnosticCode::ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET,
            format!(
                "{api_name} target must be a named sync function classified as @blocking_io, @cpu_heavy, or known blocking external work"
            ),
            target.range(),
        );
        return true;
    };
    if ctx.function_workload_annotations.contains_key(function)
        || known_stdlib_offload_target(function).is_some()
    {
        return false;
    }
    ctx.error_with_code_at(
        DiagnosticCode::ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET,
        format!(
            "{api_name} target '{function}' is not classified as @blocking_io, @cpu_heavy, or known blocking external work; call it directly or annotate it if it is genuinely blocking or expensive"
        ),
        target.range(),
    );
    true
}

pub(in crate::lower) fn reject_offload_target_without_kind(
    ctx: &mut LowerCtx,
    target: &Expr,
    api_name: &str,
    expected: WorkloadKind,
) -> bool {
    let Some(function) = target_name(target) else {
        ctx.error_with_code_at(
            DiagnosticCode::ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET,
            format!(
                "{api_name} target must be a named sync function classified as @{}",
                expected.label()
            ),
            target.range(),
        );
        return true;
    };
    let actual = ctx
        .function_workload_annotations
        .get(function)
        .copied()
        .or_else(|| known_stdlib_offload_target(function));
    let Some(actual) = actual else {
        ctx.error_with_code_at(
            DiagnosticCode::ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET,
            format!(
                "{api_name} target '{function}' is not classified as @{}; annotate it if it is genuinely {} work",
                expected.label(),
                expected.label()
            ),
            target.range(),
        );
        return true;
    };
    if actual != expected {
        ctx.error_with_code_at(
            DiagnosticCode::ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET,
            format!(
                "{api_name} target '{function}' is classified as @{}, expected @{}",
                actual.label(),
                expected.label()
            ),
            target.range(),
        );
        return true;
    }
    false
}

fn target_name(target: &Expr) -> Option<&str> {
    let Expr::Name(name) = target else {
        return None;
    };
    Some(name.id.as_str())
}

fn known_stdlib_offload_target(function: &str) -> Option<WorkloadKind> {
    match function {
        "uuid4_obj" => Some(WorkloadKind::BlockingIo),
        _ => None,
    }
}
