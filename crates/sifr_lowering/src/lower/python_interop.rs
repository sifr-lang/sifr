use super::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirParam, PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect,
    PythonInteropParameter, PythonParameterKind, PythonTargetPath,
};
use sifr_python_ast::{Decorator, Expr, ExprCall, Parameters, Stmt};
use sifr_type_system::Type;

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

pub(in crate::lower) fn is_python_omit(expr: &Expr) -> bool {
    decorator_path(expr).is_some_and(|path| path == ["python", "omit"])
}

pub(in crate::lower) fn python_parameter_kinds(
    parameters: &Parameters,
) -> Vec<PythonParameterKind> {
    parameter_metadata(parameters)
        .into_iter()
        .map(|parameter| parameter.kind)
        .collect()
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

pub(in crate::lower) fn collect_python_interop_declarations(
    decorators: &[Decorator],
    parameters: &Parameters,
    is_async_decl: bool,
    ctx: &mut LowerCtx,
) -> Vec<PythonInteropDeclaration> {
    let mut declarations = Vec::new();
    for decorator in decorators {
        let Some((kind, call, span)) = classify_decorator(&decorator.expression, ctx) else {
            continue;
        };
        if kind != PythonInteropDecoratorKind::Function {
            ctx.error_with_code_at(
                DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION,
                format!(
                    "Python declaration lowering is not active yet: `{}` belongs to a later phase",
                    decorator_label(kind)
                ),
                span,
            );
            continue;
        }
        if is_async_decl {
            invalid_shape(
                ctx,
                "`@python(path)` requires `def`; use `@python.coroutine(path)` for `async def`",
                span,
            );
            continue;
        }
        if let Some(declaration) = parse_sync_function(call, parameters, ctx) {
            declarations.push(declaration);
        }
    }
    if declarations.len() > 1 {
        invalid_shape(
            ctx,
            "a function may have only one Python implementation declaration",
            declarations[1].span,
        );
        declarations.truncate(1);
    }
    declarations
}

pub(in crate::lower) fn validate_python_interop_signature(
    declarations: &[PythonInteropDeclaration],
    params: &[HirParam],
    return_type: &Type,
    ctx: &mut LowerCtx,
) {
    let Some(declaration) = declarations.first() else {
        return;
    };
    let Type::Result(ok_type, error_type) = return_type.resolve_alias() else {
        unsupported_conversion(
            ctx,
            "a synchronous declaration must return `Result[T, PythonError]`",
            declaration.span,
        );
        return;
    };
    if !matches!(error_type.resolve_alias(), Type::Class { name, .. } if name == "PythonError") {
        unsupported_conversion(
            ctx,
            "the declaration error type must be `PythonError`",
            declaration.span,
        );
    }
    if !is_direct_type(ok_type, false) {
        unsupported_conversion(
            ctx,
            &format!(
                "return type `{}` is not in the synchronous direct-conversion set",
                ok_type.display_name()
            ),
            declaration.span,
        );
    }
    for (param, shape) in params.iter().zip(&declaration.parameters) {
        let supported = match shape.kind {
            PythonParameterKind::Positional | PythonParameterKind::KeywordOnly => {
                is_direct_type(&param.ty, true)
            }
            PythonParameterKind::PositionalVariadic => {
                matches!(param.ty.resolve_alias(), Type::List(element) if is_direct_type(element, false))
            }
            PythonParameterKind::KeywordVariadic => {
                matches!(param.ty.resolve_alias(), Type::Dict(key, value) if key.resolve_alias() == &Type::Str && is_direct_type(value, false))
            }
        };
        if !supported {
            unsupported_conversion(
                ctx,
                &format!(
                    "parameter '{}' has unsupported type `{}` for {:?}",
                    param.name,
                    param.ty.display_name(),
                    shape.kind
                ),
                shape.span,
            );
        }
    }
}

fn is_direct_type(ty: &Type, allow_option: bool) -> bool {
    match ty.resolve_alias() {
        Type::None | Type::Bool | Type::Int | Type::Float | Type::Str | Type::Bytes => true,
        Type::Class { name, .. } if name == "Object" => true,
        Type::Union(variants) if allow_option && variants.len() == 2 => {
            variants
                .iter()
                .any(|variant| variant.resolve_alias() == &Type::None)
                && variants.iter().all(|variant| {
                    variant.resolve_alias() == &Type::None || is_direct_type(variant, false)
                })
        }
        _ => false,
    }
}

fn unsupported_conversion(ctx: &mut LowerCtx, reason: &str, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE,
        format!("unsupported Python declaration conversion type: {reason}"),
        span,
    );
}

fn parse_sync_function(
    call: &ExprCall,
    parameters: &Parameters,
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        invalid_shape(
            ctx,
            "`@python(...)` requires exactly one dotted target path and no keyword arguments",
            call.range,
        );
        return None;
    }
    let target = parse_target_path(&call.arguments.args[0], ctx)?;
    let required_import_root = target
        .root()
        .filter(|root| !matches!(*root, "bridge" | "Self"))
        .map(str::to_string);
    Some(PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Function,
        target: Some(target),
        span: call.range,
        effect: PythonInteropEffect::BlockingIo,
        parameters: parameter_metadata(parameters),
        required_import_root,
    })
}

fn parameter_metadata(parameters: &Parameters) -> Vec<PythonInteropParameter> {
    let mut result = Vec::with_capacity(parameters.len());
    for parameter in parameters.posonlyargs.iter().chain(&parameters.args) {
        result.push(PythonInteropParameter {
            name: parameter.parameter.name.to_string(),
            kind: PythonParameterKind::Positional,
            has_default: parameter.default.is_some(),
            omit_when_absent: parameter.default.as_deref().is_some_and(is_python_omit),
            span: parameter.range(),
        });
    }
    if let Some(parameter) = &parameters.vararg {
        result.push(PythonInteropParameter {
            name: parameter.name.to_string(),
            kind: PythonParameterKind::PositionalVariadic,
            has_default: false,
            omit_when_absent: false,
            span: parameter.range(),
        });
    }
    for parameter in &parameters.kwonlyargs {
        result.push(PythonInteropParameter {
            name: parameter.parameter.name.to_string(),
            kind: PythonParameterKind::KeywordOnly,
            has_default: parameter.default.is_some(),
            omit_when_absent: parameter.default.as_deref().is_some_and(is_python_omit),
            span: parameter.range(),
        });
    }
    if let Some(parameter) = &parameters.kwarg {
        result.push(PythonInteropParameter {
            name: parameter.name.to_string(),
            kind: PythonParameterKind::KeywordVariadic,
            has_default: false,
            omit_when_absent: false,
            span: parameter.range(),
        });
    }
    result
}

fn parse_target_path(expr: &Expr, ctx: &mut LowerCtx) -> Option<PythonTargetPath> {
    let Some(segments) = decorator_path(expr) else {
        invalid_target(
            ctx,
            "target must be a dotted path, not a computed value",
            expr.range(),
        );
        return None;
    };
    if segments.len() < 2 || segments.iter().any(String::is_empty) {
        invalid_target(
            ctx,
            "target must contain a root and an attribute",
            expr.range(),
        );
        return None;
    }
    if segments[0] == "Self" {
        invalid_target(
            ctx,
            "`Self` is valid only on an opaque Python method",
            expr.range(),
        );
        return None;
    }
    Some(PythonTargetPath {
        segments,
        span: expr.range(),
    })
}

fn classify_decorator<'a>(
    expr: &'a Expr,
    ctx: &mut LowerCtx,
) -> Option<(PythonInteropDecoratorKind, &'a ExprCall, TextRange)> {
    let Expr::Call(call) = expr else {
        if decorator_path(expr).is_some_and(|path| path[0] == "python") {
            invalid_shape(
                ctx,
                "Python interop decorators must be call expressions",
                expr.range(),
            );
        }
        return None;
    };
    let path = decorator_path(&call.func)?;
    if path.first().is_none_or(|root| root != "python") {
        return None;
    }
    let kind = match path.as_slice() {
        [root] if root == "python" => PythonInteropDecoratorKind::Function,
        [_, name] if name == "coroutine" => PythonInteropDecoratorKind::Coroutine,
        [_, name] if name == "opaque" => PythonInteropDecoratorKind::Opaque,
        [_, name] if name == "attr" => PythonInteropDecoratorKind::Attribute,
        [_, name] if name == "item" => PythonInteropDecoratorKind::Item,
        [_, name] if name == "callback" => PythonInteropDecoratorKind::Callback,
        [_, name] if name == "buffer" => PythonInteropDecoratorKind::Buffer,
        [_, name] if name == "arrow" => PythonInteropDecoratorKind::Arrow,
        [_, name] if name == "dlpack" => PythonInteropDecoratorKind::Dlpack,
        [_, context, name] if context == "context" && name == "enter" => {
            PythonInteropDecoratorKind::ContextEnter
        }
        [_, context, name] if context == "context" && name == "exit" => {
            PythonInteropDecoratorKind::ContextExit
        }
        [_, context, name] if context == "context" && name == "aenter" => {
            PythonInteropDecoratorKind::ContextAsyncEnter
        }
        [_, context, name] if context == "context" && name == "aexit" => {
            PythonInteropDecoratorKind::ContextAsyncExit
        }
        _ => {
            invalid_target(
                ctx,
                "unknown Python declaration decorator",
                call.func.range(),
            );
            return None;
        }
    };
    Some((kind, call, expr.range()))
}

fn decorator_path(expr: &Expr) -> Option<Vec<String>> {
    match expr {
        Expr::Name(name) => Some(vec![name.id.to_string()]),
        Expr::Attribute(attribute) => {
            let mut path = decorator_path(&attribute.value)?;
            path.push(attribute.attr.to_string());
            Some(path)
        }
        _ => None,
    }
}

fn is_ellipsis_stmt(stmt: &Stmt) -> bool {
    matches!(stmt, Stmt::Expr(expr) if matches!(expr.value.as_ref(), Expr::EllipsisLiteral(_)))
}

fn decorator_label(kind: PythonInteropDecoratorKind) -> &'static str {
    match kind {
        PythonInteropDecoratorKind::Function => "python",
        PythonInteropDecoratorKind::Coroutine => "python.coroutine",
        PythonInteropDecoratorKind::Opaque => "python.opaque",
        PythonInteropDecoratorKind::Attribute => "python.attr",
        PythonInteropDecoratorKind::Item => "python.item",
        PythonInteropDecoratorKind::ContextEnter => "python.context.enter",
        PythonInteropDecoratorKind::ContextExit => "python.context.exit",
        PythonInteropDecoratorKind::ContextAsyncEnter => "python.context.aenter",
        PythonInteropDecoratorKind::ContextAsyncExit => "python.context.aexit",
        PythonInteropDecoratorKind::Callback => "python.callback",
        PythonInteropDecoratorKind::Buffer => "python.buffer",
        PythonInteropDecoratorKind::Arrow => "python.arrow",
        PythonInteropDecoratorKind::Dlpack => "python.dlpack",
    }
}

fn invalid_target(ctx: &mut LowerCtx, reason: &str, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYIMP_INVALID_TARGET,
        format!("invalid Python declaration target: {reason}"),
        span,
    );
}

fn invalid_shape(ctx: &mut LowerCtx, reason: &str, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYCALL_INVALID_SHAPE,
        format!("invalid Python declaration call shape: {reason}"),
        span,
    );
}
