use super::{ExprCall, LowerCtx, Parameters, Ranged, decorator_path, parameter_metadata, target};
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirParam, PythonArrowDeclaration, PythonArrowSchemaMode, PythonInteropDeclaration,
    PythonInteropDecoratorKind, PythonInteropEffect, PythonParameterKind, PythonTargetPath,
};
use sifr_python_ast::{AstParamMutability, AstParamOwnership, Expr};
use sifr_type_system::{PythonArrowKind, Type};

pub(super) fn parse_declaration(
    call: &ExprCall,
    parameters: &Parameters,
    is_method: bool,
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    if call.arguments.args.len() != 1 {
        invalid(
            ctx,
            "`@python.arrow` requires exactly one producer target",
            call.range,
        );
        return None;
    }
    let schema = parse_schema_mode(call, parameters, ctx)?;
    let target = if is_method {
        receiver_target(call, parameters, &schema, ctx)?
    } else {
        target::parse_callable(&call.arguments.args[0], ctx)?
    };
    let required_import_root = target
        .root()
        .filter(|root| !matches!(*root, "Self" | "__sifr_bridge__"))
        .map(str::to_string);
    let schema_parameter = match &schema {
        PythonArrowSchemaMode::Omitted => None,
        PythonArrowSchemaMode::Parameter { name, .. } => Some(name.as_str()),
    };
    let parameters = parameter_metadata(parameters)
        .into_iter()
        .skip(usize::from(is_method))
        .filter(|parameter| schema_parameter != Some(parameter.name.as_str()))
        .collect();
    Some(PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Arrow,
        target: Some(target),
        span: call.range,
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver: false,
        parameters,
        required_import_root,
        callbacks: Vec::new(),
        buffer: None,
        arrow: Some(PythonArrowDeclaration {
            // The signature validator derives and replaces this placeholder.
            kind: PythonArrowKind::Array,
            schema,
        }),
        dlpack: None,
    })
}

pub(super) fn validate_signature(
    declaration: &mut PythonInteropDeclaration,
    params: &[HirParam],
    ok_type: &Type,
    error_type: &Type,
    ctx: &mut LowerCtx,
) -> bool {
    if declaration.kind != PythonInteropDecoratorKind::Arrow {
        return false;
    }
    if !declaration.callbacks.is_empty() {
        invalid(
            ctx,
            "Arrow declarations cannot attach callback policies",
            declaration.span,
        );
    }
    if !error_type.is_python_error_contract() {
        invalid(
            ctx,
            "an Arrow declaration must use the canonical `PythonError` field contract as its error type",
            declaration.span,
        );
    }
    let Type::PythonArrow(kind) = ok_type.resolve_alias() else {
        invalid(
            ctx,
            "an Arrow declaration must return `Result[python.ArrowArray | python.ArrowSchema | python.ArrowStream | python.ArrowDeviceArray | python.ArrowDeviceStream, PythonError]`",
            declaration.span,
        );
        return true;
    };
    let Some(arrow) = declaration.arrow.as_mut() else {
        invalid(
            ctx,
            "Arrow declaration metadata is missing",
            declaration.span,
        );
        return true;
    };
    arrow.kind = *kind;
    if matches!(kind, PythonArrowKind::Schema)
        && matches!(arrow.schema, PythonArrowSchemaMode::Parameter { .. })
    {
        invalid(
            ctx,
            "`python.ArrowSchema` acquisition cannot request another schema",
            declaration.span,
        );
    }
    if let PythonArrowSchemaMode::Parameter { name, span } = &arrow.schema {
        let Some(parameter) = params.iter().find(|parameter| parameter.name == *name) else {
            invalid(
                ctx,
                &format!("schema parameter `{name}` is not present in this declaration"),
                *span,
            );
            return true;
        };
        if parameter.ty.resolve_alias() != &Type::PythonArrow(PythonArrowKind::Schema) {
            invalid(
                ctx,
                &format!(
                    "schema parameter `{name}` must have type `python.ArrowSchema`, got `{}`",
                    parameter.ty.display_name()
                ),
                *span,
            );
        }
        if !parameter.convention.is_owned() || parameter.convention.is_mutable() {
            invalid(
                ctx,
                &format!("schema parameter `{name}` must transfer ownership with plain `own`"),
                *span,
            );
        }
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

fn parse_schema_mode(
    call: &ExprCall,
    parameters: &Parameters,
    ctx: &mut LowerCtx,
) -> Option<PythonArrowSchemaMode> {
    let mut schema = None;
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            invalid(
                ctx,
                "`@python.arrow` does not accept `**kwargs`",
                keyword.range(),
            );
            return None;
        };
        if name.as_str() != "schema" {
            invalid(
                ctx,
                &format!("unknown `@python.arrow` argument `{name}`"),
                keyword.range(),
            );
            return None;
        }
        if schema.is_some() {
            invalid(
                ctx,
                "duplicate `@python.arrow` argument `schema`",
                keyword.range(),
            );
            return None;
        }
        schema = parse_schema_value(&keyword.value, parameters, ctx);
        schema.as_ref()?;
    }
    let Some(schema) = schema else {
        invalid(
            ctx,
            "`@python.arrow` requires explicit `schema=omitted` or `schema=parameter(name)`",
            call.range,
        );
        return None;
    };
    Some(schema)
}

fn parse_schema_value(
    value: &Expr,
    parameters: &Parameters,
    ctx: &mut LowerCtx,
) -> Option<PythonArrowSchemaMode> {
    if decorator_path(value).as_deref() == Some(&["omitted".to_string()]) {
        return Some(PythonArrowSchemaMode::Omitted);
    }
    let Expr::Call(parameter_call) = value else {
        invalid(
            ctx,
            "Arrow schema policy must be `omitted` or `parameter(name)`",
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
            "Arrow schema policy must be `parameter(name)` with one parameter name",
            value.range(),
        );
        return None;
    }
    let Expr::Name(name) = &parameter_call.arguments.args[0] else {
        invalid(
            ctx,
            "Arrow schema policy parameter must be a declaration parameter name",
            parameter_call.arguments.args[0].range(),
        );
        return None;
    };
    let metadata = parameter_metadata(parameters);
    let Some(parameter) = metadata.iter().find(|parameter| parameter.name == name.id) else {
        invalid(
            ctx,
            &format!(
                "schema parameter `{}` is not present in this declaration",
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
                "schema parameter `{}` must be a required keyword-only parameter",
                name.id
            ),
            parameter.span,
        );
        return None;
    }
    Some(PythonArrowSchemaMode::Parameter {
        name: name.id.to_string(),
        span: name.range(),
    })
}

fn receiver_target(
    call: &ExprCall,
    parameters: &Parameters,
    schema: &PythonArrowSchemaMode,
    ctx: &mut LowerCtx,
) -> Option<PythonTargetPath> {
    if ctx
        .current_class
        .as_ref()
        .is_none_or(|class_name| !ctx.python_opaque_classes.contains_key(class_name))
    {
        invalid(
            ctx,
            "`Self` Arrow acquisition is valid only on a `@python.opaque` class",
            call.range,
        );
        return None;
    }
    if decorator_path(&call.arguments.args[0]).as_deref() != Some(&["Self".to_string()]) {
        invalid(
            ctx,
            "an Arrow receiver declaration target must be exactly `Self`",
            call.arguments.args[0].range(),
        );
        return None;
    }
    let allowed_parameters =
        1 + usize::from(matches!(schema, PythonArrowSchemaMode::Parameter { .. }));
    if parameter_metadata(parameters).len() != allowed_parameters {
        invalid(
            ctx,
            "a `@python.arrow(Self, ...)` declaration takes only its receiver and optional requested-schema parameter",
            call.range,
        );
        return None;
    }
    let Some(receiver) = parameters.args.first() else {
        invalid(
            ctx,
            "an Arrow receiver declaration requires `self`",
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
            "a `@python.arrow(Self, ...)` declaration requires immutable borrowed `self`",
            receiver.range(),
        );
        return None;
    }
    Some(PythonTargetPath {
        segments: vec!["Self".to_string()],
        span: call.arguments.args[0].range(),
    })
}
