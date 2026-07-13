use super::{decorator_path, invalid_shape, is_ellipsis_stmt};
use crate::lower::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Decorator, Expr, Stmt, StmtFunctionDef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::lower) enum PythonInteropStubBody {
    Bodyless,
    Invalid,
    Normal,
}

impl PythonInteropStubBody {
    pub(in crate::lower) const fn skips_normal_body_lowering(self) -> bool {
        matches!(self, Self::Bodyless | Self::Invalid)
    }
}

pub(in crate::lower) fn has_python_interop_decorator_syntax(decorators: &[Decorator]) -> bool {
    decorators.iter().any(|decorator| {
        decorator_path(&decorator.expression).is_some_and(|path| path[0] == "python")
            || matches!(&decorator.expression, Expr::Call(call) if decorator_path(&call.func).is_some_and(|path| path[0] == "python"))
    })
}

pub(in crate::lower) fn is_bodyless_python_coroutine(func: &StmtFunctionDef) -> bool {
    func.is_async
        && matches!(func.body.as_slice(), [stmt] if is_ellipsis_stmt(stmt))
        && func.decorator_list.iter().any(|decorator| {
            matches!(&decorator.expression, Expr::Call(call) if decorator_path(&call.func).is_some_and(|path| path == ["python", "coroutine"]))
        })
}

pub(in crate::lower) fn classify_python_interop_stub_body(
    body: &[Stmt],
    has_python_decorator: bool,
    ctx: &mut LowerCtx,
) -> PythonInteropStubBody {
    let exact = matches!(body, [stmt] if is_ellipsis_stmt(stmt));
    let contains = body.iter().any(is_ellipsis_stmt);
    if exact {
        if has_python_decorator {
            return PythonInteropStubBody::Bodyless;
        }
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            "ellipsis is only supported as the complete body of an interop declaration".to_string(),
            body[0].range(),
        );
        return PythonInteropStubBody::Invalid;
    }
    if contains && has_python_decorator {
        let span = body
            .iter()
            .find(|stmt| is_ellipsis_stmt(stmt))
            .map_or_else(TextRange::default, Ranged::range);
        invalid_shape(
            ctx,
            "declaration stubs must contain exactly one ellipsis statement and no other statements",
            span,
        );
        return PythonInteropStubBody::Invalid;
    }
    PythonInteropStubBody::Normal
}
