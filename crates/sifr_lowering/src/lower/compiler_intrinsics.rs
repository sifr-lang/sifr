use super::{LowerCtx, LoweringSourceOrigin};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::CompilerIntrinsicId;
use sifr_python_ast::{Decorator, Expr, Stmt, StmtFunctionDef};

use super::rust_interop::RustInteropStubBody;

pub(in crate::lower) fn register_declaration(func: &StmtFunctionDef, ctx: &mut LowerCtx) {
    let declarations = func
        .decorator_list
        .iter()
        .filter(|decorator| is_compiler_intrinsic_decorator(&decorator.expression))
        .collect::<Vec<_>>();
    if declarations.is_empty() {
        return;
    }
    if declarations.len() != 1 {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            "a callable may declare exactly one @compiler_intrinsic identity".to_string(),
            func.name.range(),
        );
        return;
    }
    let decorator = declarations[0];
    if ctx.source_origin != LoweringSourceOrigin::SysrootPublicStdlib {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            "@compiler_intrinsic is reserved for canonical public sysroot declarations".to_string(),
            decorator.expression.range(),
        );
        return;
    }
    let Some(id) = parse_declaration_id(decorator, ctx) else {
        return;
    };
    if !is_source_declared_callable(id) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            format!(
                "compiler intrinsic '{}' is synthesized by lowering and cannot be source-declared",
                id.declaration_name()
            ),
            decorator.expression.range(),
        );
        return;
    }
    ctx.compiler_intrinsics.insert(func.name.to_string(), id);
}

pub(in crate::lower) fn classify_stub_body(
    func: &StmtFunctionDef,
    intrinsic: Option<CompilerIntrinsicId>,
    ctx: &mut LowerCtx,
) -> RustInteropStubBody {
    let has_syntax = func
        .decorator_list
        .iter()
        .any(|decorator| is_compiler_intrinsic_decorator(&decorator.expression));
    if intrinsic.is_none() && !has_syntax {
        return RustInteropStubBody::Normal;
    }
    let exact_ellipsis = matches!(
        func.body.as_slice(),
        [Stmt::Expr(expr_stmt)] if matches!(expr_stmt.value.as_ref(), Expr::EllipsisLiteral(_))
    );
    if intrinsic.is_some() && exact_ellipsis {
        return RustInteropStubBody::Bodyless;
    }
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
        "@compiler_intrinsic declarations must contain exactly one ellipsis statement and no runtime body"
            .to_string(),
        func.body
            .first()
            .map_or_else(|| func.name.range(), Ranged::range),
    );
    RustInteropStubBody::Invalid
}

pub(in crate::lower) fn has_decorator_syntax(decorators: &[Decorator]) -> bool {
    decorators
        .iter()
        .any(|decorator| is_compiler_intrinsic_decorator(&decorator.expression))
}

fn is_compiler_intrinsic_decorator(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call(call)
            if matches!(call.func.as_ref(), Expr::Name(name) if name.id.as_str() == "compiler_intrinsic")
    ) || matches!(expr, Expr::Name(name) if name.id.as_str() == "compiler_intrinsic")
}

fn parse_declaration_id(decorator: &Decorator, ctx: &mut LowerCtx) -> Option<CompilerIntrinsicId> {
    let Expr::Call(call) = &decorator.expression else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            "@compiler_intrinsic must be a call with one typed identity".to_string(),
            decorator.expression.range(),
        );
        return None;
    };
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            "@compiler_intrinsic accepts exactly one positional identity".to_string(),
            call.range(),
        );
        return None;
    }
    let Expr::Name(name) = &call.arguments.args[0] else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            "@compiler_intrinsic identity must be a bare compiler ID".to_string(),
            call.arguments.args[0].range(),
        );
        return None;
    };
    let Some(id) = CompilerIntrinsicId::from_declaration_name(name.id.as_str()) else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            format!("unknown compiler intrinsic identity '{}'", name.id),
            name.range(),
        );
        return None;
    };
    Some(id)
}

const fn is_source_declared_callable(id: CompilerIntrinsicId) -> bool {
    matches!(
        id,
        CompilerIntrinsicId::TestAssertEqual
            | CompilerIntrinsicId::TestAssertNotEqual
            | CompilerIntrinsicId::TestAssertTrue
            | CompilerIntrinsicId::TestAssertFalse
            | CompilerIntrinsicId::TestAssertAlmostEqual
            | CompilerIntrinsicId::TestAssertGreaterThan
            | CompilerIntrinsicId::TestAssertLessThan
            | CompilerIntrinsicId::TaskCurrentContext
            | CompilerIntrinsicId::PythonFromValue
            | CompilerIntrinsicId::PythonToValue
            | CompilerIntrinsicId::PythonKwarg
    )
}
