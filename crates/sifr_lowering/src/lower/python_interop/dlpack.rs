use super::{decorator_path, parameter_metadata, target, ExprCall, LowerCtx, Parameters, Ranged};
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirParam, PythonDlpackDeclaration, PythonDlpackDevice, PythonDlpackStreamMode,
    PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect, PythonParameterKind,
    PythonTargetPath,
};
use sifr_python_ast::{AstParamMutability, AstParamOwnership, Expr};
use sifr_type_system::Type;

pub(super) fn parse_declaration(
    call: &ExprCall,
    parameters: &Parameters,
    is_method: bool,
    is_stream: bool,
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    let label = if is_stream {
        "`@python.dlpack.stream`"
    } else {
        "`@python.dlpack`"
    };
    if call.arguments.args.len() != 1 {
        invalid(
            ctx,
            &format!("{label} requires exactly one producer target"),
            call.range,
        );
        return None;
    }
    let (device, stream) = parse_policies(call, parameters, is_stream, ctx)?;
    let target = if is_method {
        receiver_target(call, parameters, &stream, ctx)?
    } else {
        target::parse_callable(&call.arguments.args[0], ctx)?
    };
    let required_import_root = target
        .root()
        .filter(|root| !matches!(*root, "Self" | "__sifr_bridge__"))
        .map(str::to_string);
    let stream_parameter = match &stream {
        PythonDlpackStreamMode::None => None,
        PythonDlpackStreamMode::Parameter { name, .. } => Some(name.as_str()),
    };
    let parameters = parameter_metadata(parameters)
        .into_iter()
        .skip(usize::from(is_method))
        .filter(|parameter| stream_parameter != Some(parameter.name.as_str()))
        .collect();
    Some(PythonInteropDeclaration {
        kind: if is_stream {
            PythonInteropDecoratorKind::DlpackStream
        } else {
            PythonInteropDecoratorKind::Dlpack
        },
        target: Some(target),
        span: call.range,
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver: false,
        parameters,
        required_import_root,
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: Some(PythonDlpackDeclaration {
            device,
            stream,
            element_type: None,
        }),
    })
}

pub(super) fn validate_signature(
    declaration: &mut PythonInteropDeclaration,
    params: &[HirParam],
    ok_type: &Type,
    error_type: &Type,
    ctx: &mut LowerCtx,
) -> bool {
    if !matches!(
        declaration.kind,
        PythonInteropDecoratorKind::Dlpack | PythonInteropDecoratorKind::DlpackStream
    ) {
        return false;
    }
    if !declaration.callbacks.is_empty() {
        invalid(
            ctx,
            "DLPack declarations cannot attach callback policies",
            declaration.span,
        );
    }
    if !error_type.is_python_error_contract() {
        invalid(
            ctx,
            "a DLPack declaration must use the canonical `PythonError` field contract as its error type",
            declaration.span,
        );
    }
    let Some(dlpack) = declaration.dlpack.as_mut() else {
        invalid(
            ctx,
            "DLPack declaration metadata is missing",
            declaration.span,
        );
        return true;
    };
    match declaration.kind {
        PythonInteropDecoratorKind::Dlpack => {
            let Type::PythonDlpackTensor(element) = ok_type.resolve_alias() else {
                invalid(
                    ctx,
                    "a DLPack tensor declaration must return `Result[python.DlpackTensor[T], PythonError]`",
                    declaration.span,
                );
                return true;
            };
            dlpack.element_type = Some(element.as_ref().clone());
        }
        PythonInteropDecoratorKind::DlpackStream => {
            if ok_type.resolve_alias() != &Type::PythonDlpackStream {
                invalid(
                    ctx,
                    "a DLPack stream declaration must return `Result[python.DlpackStream, PythonError]`",
                    declaration.span,
                );
            }
        }
        _ => unreachable!("validated DLPack declaration kind"),
    }
    if let PythonDlpackStreamMode::Parameter { name, span } = &dlpack.stream {
        let Some(parameter) = params.iter().find(|parameter| parameter.name == *name) else {
            invalid(
                ctx,
                &format!("DLPack stream parameter `{name}` is not present in this declaration"),
                *span,
            );
            return true;
        };
        if parameter.ty.resolve_alias() != &Type::PythonDlpackStream {
            invalid(
                ctx,
                &format!(
                    "DLPack stream parameter `{name}` must have type `python.DlpackStream`, got `{}`",
                    parameter.ty.display_name()
                ),
                *span,
            );
        }
        if !parameter.convention.is_shared_borrow() {
            invalid(
                ctx,
                &format!("DLPack stream parameter `{name}` must be an immutable borrow"),
                *span,
            );
        }
    }
    true
}

fn parse_policies(
    call: &ExprCall,
    parameters: &Parameters,
    is_stream_declaration: bool,
    ctx: &mut LowerCtx,
) -> Option<(PythonDlpackDevice, PythonDlpackStreamMode)> {
    let mut device = None;
    let mut stream = None;
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            invalid(
                ctx,
                "DLPack decorators do not accept `**kwargs`",
                keyword.range(),
            );
            return None;
        };
        match name.as_str() {
            "device" if device.is_none() => device = parse_device(&keyword.value, ctx),
            "stream" if is_stream_declaration => {
                invalid(
                    ctx,
                    "`@python.dlpack.stream` does not accept a `stream` argument",
                    keyword.range(),
                );
                return None;
            }
            "stream" if !is_stream_declaration && stream.is_none() => {
                stream = parse_stream(&keyword.value, parameters, ctx)
            }
            "device" | "stream" => {
                invalid(
                    ctx,
                    &format!("duplicate DLPack argument `{name}`"),
                    keyword.range(),
                );
                return None;
            }
            _ => {
                invalid(
                    ctx,
                    &format!("unknown DLPack decorator argument `{name}`"),
                    keyword.range(),
                );
                return None;
            }
        }
    }
    let Some(device) = device else {
        invalid(
            ctx,
            "DLPack declarations require explicit `device=cpu | cuda | any`",
            call.range,
        );
        return None;
    };
    if is_stream_declaration {
        if device == PythonDlpackDevice::Any {
            invalid(
                ctx,
                "`@python.dlpack.stream` requires a concrete `cpu` or `cuda` device family",
                call.range,
            );
            return None;
        }
        return Some((device, PythonDlpackStreamMode::None));
    }
    let Some(stream) = stream else {
        invalid(
            ctx,
            "`@python.dlpack` requires explicit `stream=none | parameter(name)`",
            call.range,
        );
        return None;
    };
    if device != PythonDlpackDevice::Cpu && matches!(stream, PythonDlpackStreamMode::None) {
        invalid(
            ctx,
            "non-CPU and `device=any` DLPack declarations require `stream=parameter(name)`",
            call.range,
        );
        return None;
    }
    Some((device, stream))
}

fn parse_device(value: &Expr, ctx: &mut LowerCtx) -> Option<PythonDlpackDevice> {
    let Some(path) = decorator_path(value) else {
        invalid(
            ctx,
            "DLPack device must be the closed atom `cpu`, `cuda`, or `any`",
            value.range(),
        );
        return None;
    };
    match path.as_slice() {
        [name] if name == "cpu" => Some(PythonDlpackDevice::Cpu),
        [name] if name == "cuda" => Some(PythonDlpackDevice::Cuda),
        [name] if name == "any" => Some(PythonDlpackDevice::Any),
        _ => {
            invalid(
                ctx,
                "DLPack device must be the closed atom `cpu`, `cuda`, or `any`",
                value.range(),
            );
            None
        }
    }
}

fn parse_stream(
    value: &Expr,
    parameters: &Parameters,
    ctx: &mut LowerCtx,
) -> Option<PythonDlpackStreamMode> {
    if decorator_path(value).as_deref() == Some(&["none".to_string()]) {
        return Some(PythonDlpackStreamMode::None);
    }
    let Expr::Call(parameter_call) = value else {
        invalid(
            ctx,
            "DLPack stream policy must be `none` or `parameter(name)`",
            value.range(),
        );
        return None;
    };
    if decorator_path(&parameter_call.func).as_deref() != Some(&["parameter".to_string()])
        || parameter_call.arguments.args.len() != 1
        || !parameter_call.arguments.keywords.is_empty()
    {
        invalid(
            ctx,
            "DLPack stream policy must be `parameter(name)` with one parameter name",
            value.range(),
        );
        return None;
    }
    let Expr::Name(name) = &parameter_call.arguments.args[0] else {
        invalid(
            ctx,
            "DLPack stream policy parameter must be a declaration parameter name",
            parameter_call.arguments.args[0].range(),
        );
        return None;
    };
    let metadata = parameter_metadata(parameters);
    let Some(parameter) = metadata.iter().find(|parameter| parameter.name == name.id) else {
        invalid(
            ctx,
            &format!(
                "DLPack stream parameter `{}` is not present in this declaration",
                name.id
            ),
            name.range(),
        );
        return None;
    };
    if parameter.kind != PythonParameterKind::KeywordOnly || parameter.has_default {
        invalid(
            ctx,
            &format!(
                "DLPack stream parameter `{}` must be a required keyword-only parameter",
                name.id
            ),
            parameter.span,
        );
        return None;
    }
    Some(PythonDlpackStreamMode::Parameter {
        name: name.id.to_string(),
        span: name.range(),
    })
}

fn receiver_target(
    call: &ExprCall,
    parameters: &Parameters,
    stream: &PythonDlpackStreamMode,
    ctx: &mut LowerCtx,
) -> Option<PythonTargetPath> {
    if ctx
        .current_class
        .as_ref()
        .is_none_or(|class_name| !ctx.python_opaque_classes.contains_key(class_name))
    {
        invalid(
            ctx,
            "`Self` DLPack acquisition is valid only on a `@python.opaque` class",
            call.range,
        );
        return None;
    }
    if decorator_path(&call.arguments.args[0]).as_deref() != Some(&["Self".to_string()]) {
        invalid(
            ctx,
            "a DLPack receiver declaration target must be exactly `Self`",
            call.arguments.args[0].range(),
        );
        return None;
    }
    let allowed = 1 + usize::from(matches!(stream, PythonDlpackStreamMode::Parameter { .. }));
    if parameter_metadata(parameters).len() != allowed {
        invalid(
            ctx,
            "a DLPack `Self` declaration takes only its receiver and optional stream parameter",
            call.range,
        );
        return None;
    }
    let Some(receiver) = parameters.args.first() else {
        invalid(
            ctx,
            "a DLPack receiver declaration requires `self`",
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
            "a DLPack `Self` declaration requires immutable borrowed `self`",
            receiver.range(),
        );
        return None;
    }
    Some(PythonTargetPath {
        segments: vec!["Self".to_string()],
        span: call.arguments.args[0].range(),
    })
}

pub(super) fn invalid(ctx: &mut LowerCtx, reason: &str, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYZC_INVALID_DECLARATION,
        format!("invalid Python zero-copy declaration: {reason}"),
        span,
    );
}
