use super::{decorator_path, parameter_metadata, target, ExprCall, LowerCtx, Parameters, Ranged};
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    PythonBufferAccess, PythonBufferDeclaration, PythonBufferLayout, PythonInteropDeclaration,
    PythonInteropDecoratorKind, PythonInteropEffect, PythonTargetPath,
};
use sifr_python_ast::{AstParamMutability, AstParamOwnership};
use sifr_type_system::Type;

pub(super) fn parse_declaration(
    call: &ExprCall,
    parameters: &Parameters,
    is_method: bool,
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    if call.arguments.args.len() != 1 {
        invalid(
            ctx,
            "`@python.buffer` requires exactly one producer target",
            call.range,
        );
        return None;
    }
    let (access, layout) = parse_policy(call, ctx)?;
    let target = if is_method {
        receiver_target(call, parameters, access, ctx)?
    } else {
        target::parse_callable(&call.arguments.args[0], ctx)?
    };
    let required_import_root = target
        .root()
        .filter(|root| !matches!(*root, "Self" | "__sifr_bridge__"))
        .map(str::to_string);
    Some(PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Buffer,
        target: Some(target),
        span: call.range,
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver: false,
        parameters: if is_method {
            Vec::new()
        } else {
            parameter_metadata(parameters)
        },
        required_import_root,
        callbacks: Vec::new(),
        buffer: Some(PythonBufferDeclaration {
            element_type: Type::Any,
            access,
            layout,
        }),
    })
}

pub(super) fn validate_signature(
    declaration: &mut PythonInteropDeclaration,
    ok_type: &Type,
    error_type: &Type,
    ctx: &mut LowerCtx,
) -> bool {
    if declaration.kind != PythonInteropDecoratorKind::Buffer {
        return false;
    }
    if !declaration.callbacks.is_empty() {
        invalid(
            ctx,
            "buffer declarations cannot attach callback policies",
            declaration.span,
        );
    }
    if !error_type.is_python_error_contract() {
        invalid(
            ctx,
            "a buffer declaration must use the canonical `PythonError` field contract as its error type",
            declaration.span,
        );
    }
    let Type::PythonBuffer(element_type) = ok_type.resolve_alias() else {
        invalid(
            ctx,
            "a buffer declaration must return `Result[python.Buffer[T], PythonError]`",
            declaration.span,
        );
        return true;
    };
    if let Some(buffer) = declaration.buffer.as_mut() {
        buffer.element_type = *element_type.clone();
    }
    true
}

pub(super) fn invalid(ctx: &mut LowerCtx, reason: &str, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYZC_INVALID_DECLARATION,
        format!("invalid Python zero-copy declaration: {reason}"),
        span,
    );
}

fn receiver_target(
    call: &ExprCall,
    parameters: &Parameters,
    access: PythonBufferAccess,
    ctx: &mut LowerCtx,
) -> Option<PythonTargetPath> {
    if ctx
        .current_class
        .as_ref()
        .is_none_or(|class_name| !ctx.python_opaque_classes.contains_key(class_name))
    {
        invalid(
            ctx,
            "`Self` buffer acquisition is valid only on a `@python.opaque` class",
            call.range,
        );
        return None;
    }
    let segments = decorator_path(&call.arguments.args[0]);
    if segments.as_deref() != Some(&["Self".to_string()]) {
        invalid(
            ctx,
            "a buffer receiver declaration target must be exactly `Self`",
            call.arguments.args[0].range(),
        );
        return None;
    }
    if access == PythonBufferAccess::Write {
        invalid(
            ctx,
            "a writable `Self` buffer cannot exclusively freeze its opaque owner; use a producer that returns a fresh exporter",
            call.range,
        );
        return None;
    }
    if parameter_metadata(parameters).len() != 1 {
        invalid(
            ctx,
            "a `@python.buffer(Self, ...)` declaration takes only its receiver",
            call.range,
        );
        return None;
    }
    let Some(receiver) = parameters.args.first() else {
        invalid(
            ctx,
            "a receiver buffer declaration requires `self`",
            call.range,
        );
        return None;
    };
    let convention = receiver.parameter.convention;
    if convention.ownership != AstParamOwnership::Borrow
        || convention.mutability != AstParamMutability::Immutable
    {
        invalid(
            ctx,
            "a `@python.buffer(Self, ...)` declaration requires immutable borrowed `self`",
            receiver.range(),
        );
        return None;
    }
    Some(PythonTargetPath {
        segments: vec!["Self".to_string()],
        span: call.arguments.args[0].range(),
    })
}

fn parse_policy(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<(PythonBufferAccess, PythonBufferLayout)> {
    let mut access = None;
    let mut layout = None;
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            invalid(
                ctx,
                "`@python.buffer` does not accept `**kwargs`",
                keyword.range(),
            );
            return None;
        };
        let atom = decorator_path(&keyword.value);
        match name.as_str() {
            "access" if access.is_none() => {
                access = match atom.as_deref() {
                    Some([value]) if value == "read" => Some(PythonBufferAccess::Read),
                    Some([value]) if value == "write" => Some(PythonBufferAccess::Write),
                    _ => {
                        invalid(
                            ctx,
                            "buffer access must be `read` or `write`",
                            keyword.value.range(),
                        );
                        return None;
                    }
                };
            }
            "layout" if layout.is_none() => {
                layout = match atom.as_deref() {
                    Some([value]) if value == "any" => Some(PythonBufferLayout::Any),
                    Some([value]) if value == "c_contiguous" => {
                        Some(PythonBufferLayout::CContiguous)
                    }
                    Some([value]) if value == "f_contiguous" => {
                        Some(PythonBufferLayout::FContiguous)
                    }
                    _ => {
                        invalid(
                            ctx,
                            "buffer layout must be `any`, `c_contiguous`, or `f_contiguous`",
                            keyword.value.range(),
                        );
                        return None;
                    }
                };
            }
            "access" | "layout" => {
                invalid(
                    ctx,
                    &format!("duplicate `@python.buffer` argument `{name}`"),
                    keyword.range(),
                );
                return None;
            }
            _ => {
                invalid(
                    ctx,
                    &format!("unknown `@python.buffer` argument `{name}`"),
                    keyword.range(),
                );
                return None;
            }
        }
    }
    let Some(access) = access else {
        invalid(ctx, "`@python.buffer` requires `access=`", call.range);
        return None;
    };
    let Some(layout) = layout else {
        invalid(ctx, "`@python.buffer` requires `layout=`", call.range);
        return None;
    };
    Some((access, layout))
}
