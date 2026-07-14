use super::{decorator_path, is_direct_type, parameter_metadata};
use crate::lower::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirParam, PythonCallbackConcurrency, PythonCallbackDeclaration, PythonCallbackDispatch,
    PythonCallbackLifetime, PythonCleanupPolicy, PythonInteropDeclaration, PythonInteropEffect,
    PythonParameterKind,
};
use sifr_python_ast::{ExprCall, Parameters};
use sifr_type_system::Type;

pub(super) fn attach(
    declarations: &mut [PythonInteropDeclaration],
    callbacks: Vec<PythonCallbackDeclaration>,
    ctx: &mut LowerCtx,
) {
    if callbacks.is_empty() {
        return;
    }
    let Some(declaration) = declarations.first_mut() else {
        invalid(
            ctx,
            "`@python.callback` requires one ordinary `@python(...)` or `@python.coroutine(...)` implementation declaration",
            callbacks[0].span,
        );
        return;
    };
    if !matches!(
        declaration.kind,
        sifr_ir::PythonInteropDecoratorKind::Function
            | sifr_ir::PythonInteropDecoratorKind::Coroutine
    ) {
        invalid(
            ctx,
            "`@python.callback` requires one ordinary `@python(...)` or `@python.coroutine(...)` implementation declaration",
            callbacks[0].span,
        );
        return;
    }
    let mut duplicate = false;
    for callback in callbacks {
        if declaration
            .callbacks
            .iter()
            .any(|existing| existing.parameter_name == callback.parameter_name)
        {
            invalid(
                ctx,
                &format!(
                    "callback parameter '{}' has more than one `@python.callback` declaration",
                    callback.parameter_name
                ),
                callback.span,
            );
            duplicate = true;
        } else {
            declaration.callbacks.push(callback);
        }
    }
    if duplicate {
        declaration.callbacks.clear();
    }
}

pub(super) fn parse(
    call: &ExprCall,
    parameters: &Parameters,
    has_receiver: bool,
    ctx: &mut LowerCtx,
) -> Option<PythonCallbackDeclaration> {
    if call.arguments.args.len() != 1 {
        invalid(
            ctx,
            "`@python.callback` requires exactly one callback parameter name",
            call.range,
        );
        return None;
    }
    let parameter_path = decorator_path(&call.arguments.args[0]);
    let Some([parameter_name]) = parameter_path.as_deref() else {
        invalid(
            ctx,
            "the callback parameter must be a direct parameter name",
            call.arguments.args[0].range(),
        );
        return None;
    };
    let parameter_shapes = parameter_metadata(parameters);
    let Some(parameter_shape) = parameter_shapes
        .iter()
        .find(|parameter| parameter.name == *parameter_name)
    else {
        invalid(
            ctx,
            &format!("unknown callback parameter `{parameter_name}`"),
            call.arguments.args[0].range(),
        );
        return None;
    };
    if matches!(
        parameter_shape.kind,
        PythonParameterKind::PositionalVariadic | PythonParameterKind::KeywordVariadic
    ) {
        invalid(
            ctx,
            "`@python.callback` requires one ordinary positional or keyword-only callable parameter",
            parameter_shape.span,
        );
        return None;
    }

    let mut lifetime = None;
    let mut dispatch = None;
    let mut concurrency = None;
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            invalid(
                ctx,
                "`@python.callback` does not accept `**kwargs`",
                keyword.range(),
            );
            return None;
        };
        let Some(value) = policy_atom(&keyword.value) else {
            invalid(
                ctx,
                &format!("callback policy `{name}` must be a closed policy atom"),
                keyword.value.range(),
            );
            return None;
        };
        match name.as_str() {
            "lifetime" if lifetime.is_none() => {
                lifetime = match value.as_str() {
                    "call" => Some(PythonCallbackLifetime::Call),
                    "result" => Some(PythonCallbackLifetime::Result),
                    "Self" => Some(PythonCallbackLifetime::Receiver),
                    _ => {
                        return invalid_none(
                            ctx,
                            "unknown callback lifetime policy",
                            keyword.value.range(),
                        )
                    }
                };
            }
            "dispatch" if dispatch.is_none() => {
                dispatch = match value.as_str() {
                    "current" => Some(PythonCallbackDispatch::Current),
                    "foreign" => Some(PythonCallbackDispatch::Foreign),
                    "asyncio" => Some(PythonCallbackDispatch::Asyncio),
                    _ => {
                        return invalid_none(
                            ctx,
                            "unknown callback dispatch policy",
                            keyword.value.range(),
                        )
                    }
                };
            }
            "concurrency" if concurrency.is_none() => {
                concurrency = match value.as_str() {
                    "serial" => Some(PythonCallbackConcurrency::Serial),
                    "parallel" => Some(PythonCallbackConcurrency::Parallel),
                    _ => {
                        return invalid_none(
                            ctx,
                            "unknown callback concurrency policy",
                            keyword.value.range(),
                        )
                    }
                };
            }
            "lifetime" | "dispatch" | "concurrency" => {
                return invalid_none(
                    ctx,
                    &format!("duplicate `@python.callback` argument `{name}`"),
                    keyword.range(),
                );
            }
            _ => {
                return invalid_none(
                    ctx,
                    &format!("unknown `@python.callback` argument `{name}`"),
                    keyword.range(),
                );
            }
        }
    }
    let Some(lifetime) = lifetime else {
        return invalid_none(ctx, "`@python.callback` requires `lifetime=`", call.range);
    };
    let Some(dispatch) = dispatch else {
        return invalid_none(ctx, "`@python.callback` requires `dispatch=`", call.range);
    };
    if lifetime == PythonCallbackLifetime::Receiver && !has_receiver {
        return invalid_none(
            ctx,
            "`lifetime=Self` is valid only on an opaque receiver method",
            call.range,
        );
    }
    if dispatch == PythonCallbackDispatch::Current {
        if lifetime != PythonCallbackLifetime::Call {
            return invalid_none(
                ctx,
                "`dispatch=current` requires `lifetime=call`",
                call.range,
            );
        }
        if concurrency.is_some() {
            return invalid_none(
                ctx,
                "`dispatch=current` does not accept `concurrency=`",
                call.range,
            );
        }
    } else if concurrency.is_none() {
        return invalid_none(
            ctx,
            "foreign and asyncio callbacks require `concurrency=serial | parallel`",
            call.range,
        );
    }

    Some(PythonCallbackDeclaration {
        parameter_name: parameter_name.clone(),
        span: call.range,
        lifetime,
        dispatch,
        concurrency,
        argument_types: Vec::new(),
        argument_conventions: Vec::new(),
        success_type: Type::Any,
        handler_error_type: None,
        is_async: false,
        owner_class: None,
        owner_cleanup: None,
    })
}

fn policy_atom(expr: &sifr_python_ast::Expr) -> Option<String> {
    decorator_path(expr)
        .filter(|segments| segments.len() == 1)
        .and_then(|segments| segments.into_iter().next())
}

fn invalid_none<T>(ctx: &mut LowerCtx, reason: &str, span: TextRange) -> Option<T> {
    invalid(ctx, reason, span);
    None
}

pub(super) fn validate(
    declaration: &mut PythonInteropDeclaration,
    params: &[HirParam],
    enclosing_success: &Type,
    enclosing_error: &Type,
    ctx: &mut LowerCtx,
) {
    let declaration_effect = declaration.effect;
    for callback in &mut declaration.callbacks {
        let Some(parameter) = params
            .iter()
            .find(|parameter| parameter.name == callback.parameter_name)
        else {
            invalid(
                ctx,
                &format!("unknown callback parameter `{}`", callback.parameter_name),
                callback.span,
            );
            continue;
        };
        let Some((argument_types, argument_conventions, callback_return, is_async)) =
            callable_signature(&parameter.ty)
        else {
            invalid(
                ctx,
                &format!(
                    "parameter `{}` must use `Callable[[...], R]` or `AsyncCallable[[...], R]`",
                    callback.parameter_name
                ),
                callback.span,
            );
            continue;
        };
        validate_dispatch(callback, is_async, declaration_effect, ctx);

        let (success_type, handler_error_type) = match callback_return.resolve_alias() {
            Type::Result(success, error) => {
                (success.as_ref().clone(), Some(error.as_ref().clone()))
            }
            other => (other.clone(), None),
        };
        validate_conversions(
            callback,
            &argument_types,
            &success_type,
            handler_error_type.as_ref(),
            enclosing_error,
            ctx,
        );
        validate_boundary_types(
            callback,
            &argument_types,
            &success_type,
            handler_error_type.as_ref(),
            ctx,
        );
        if !resolve_owner(callback, enclosing_success, ctx) {
            continue;
        }

        callback.argument_types = argument_types;
        callback.argument_conventions = argument_conventions;
        callback.success_type = success_type;
        callback.handler_error_type = handler_error_type;
        callback.is_async = is_async;
    }
}

fn callable_signature(
    ty: &Type,
) -> Option<(
    Vec<Type>,
    Vec<sifr_type_system::ParamConvention>,
    Type,
    bool,
)> {
    match ty.resolve_alias() {
        Type::Callable(arguments, conventions, result) => Some((
            arguments.clone(),
            conventions.clone(),
            result.as_ref().clone(),
            false,
        )),
        Type::AsyncCallable(arguments, conventions, result) => Some((
            arguments.clone(),
            conventions.clone(),
            result.as_ref().clone(),
            true,
        )),
        _ => None,
    }
}

fn validate_dispatch(
    callback: &PythonCallbackDeclaration,
    is_async: bool,
    declaration_effect: PythonInteropEffect,
    ctx: &mut LowerCtx,
) {
    let error = match callback.dispatch {
        PythonCallbackDispatch::Current if is_async => {
            Some("`dispatch=current` requires a synchronous `Callable` parameter")
        }
        PythonCallbackDispatch::Current if declaration_effect == PythonInteropEffect::Async => Some(
            "`dispatch=current` requires a synchronous `@python(...)` target so non-send captures remain on their creating thread",
        ),
        PythonCallbackDispatch::Foreign if is_async => {
            Some("`dispatch=foreign` requires a synchronous `Callable` parameter")
        }
        PythonCallbackDispatch::Asyncio if !is_async => {
            Some("`dispatch=asyncio` requires an `AsyncCallable` parameter")
        }
        PythonCallbackDispatch::Asyncio if declaration_effect != PythonInteropEffect::Async => {
            Some("`dispatch=asyncio` requires an asynchronous `@python.coroutine(...)` target")
        }
        _ => None,
    };
    if let Some(error) = error {
        invalid(ctx, error, callback.span);
    }
}

fn validate_conversions(
    callback: &PythonCallbackDeclaration,
    arguments: &[Type],
    success: &Type,
    handler_error: Option<&Type>,
    enclosing_error: &Type,
    ctx: &mut LowerCtx,
) {
    for argument in arguments {
        if !is_direct_type(argument, true, ctx) {
            invalid(
                ctx,
                &format!(
                    "callback argument type `{}` has no direct Python conversion",
                    argument.display_name()
                ),
                callback.span,
            );
        }
    }
    if !is_direct_type(success, true, ctx) {
        invalid(
            ctx,
            &format!(
                "callback success type `{}` has no direct Python conversion",
                success.display_name()
            ),
            callback.span,
        );
    }
    if let Some(handler_error) = handler_error {
        if !error_channel_contains(enclosing_error, handler_error) {
            invalid(
                ctx,
                &format!(
                    "the enclosing declaration error channel must contain callback handler error `{}`",
                    handler_error.display_name()
                ),
                callback.span,
            );
        }
    }
}

fn validate_boundary_types(
    callback: &PythonCallbackDeclaration,
    arguments: &[Type],
    success: &Type,
    handler_error: Option<&Type>,
    ctx: &mut LowerCtx,
) {
    if !matches!(
        callback.dispatch,
        PythonCallbackDispatch::Foreign | PythonCallbackDispatch::Asyncio
    ) {
        return;
    }
    for boundary_type in arguments
        .iter()
        .chain(std::iter::once(success))
        .chain(handler_error)
    {
        if let Some(reason) = crate::lower::task_scope_calls::non_send_reason(boundary_type) {
            invalid(
                ctx,
                &format!(
                    "callback boundary type `{}` is not sendable: {reason}",
                    boundary_type.display_name()
                ),
                callback.span,
            );
        }
        if callback.dispatch == PythonCallbackDispatch::Foreign
            && contains_python_identity(boundary_type, ctx)
        {
            invalid(
                ctx,
                &format!(
                    "foreign callback boundary type `{}` contains Python identity",
                    boundary_type.display_name()
                ),
                callback.span,
            );
        }
    }
}

fn resolve_owner(
    callback: &mut PythonCallbackDeclaration,
    enclosing_success: &Type,
    ctx: &mut LowerCtx,
) -> bool {
    let owner = match callback.lifetime {
        PythonCallbackLifetime::Call => return true,
        PythonCallbackLifetime::Result => opaque_owner(enclosing_success, ctx),
        PythonCallbackLifetime::Receiver => ctx.current_class.as_ref().and_then(|name| {
            ctx.python_opaque_classes
                .get(name)
                .map(|owner| (name, owner))
        }),
    };
    let Some((owner_name, owner_declaration)) = owner else {
        invalid(
            ctx,
            "retained callback lifetime requires a declared opaque result or receiver owner",
            callback.span,
        );
        return false;
    };
    let cleanup = owner_declaration.cleanup;
    if cleanup.is_none() || cleanup == Some(PythonCleanupPolicy::Drop) {
        invalid(
            ctx,
            &format!(
                "retained callback owner `{owner_name}` requires deterministic close, context, async-close, or async-context cleanup"
            ),
            callback.span,
        );
        return false;
    }
    callback.owner_class = Some(owner_name.clone());
    callback.owner_cleanup = cleanup;
    true
}

fn opaque_owner<'a>(
    ty: &Type,
    ctx: &'a LowerCtx,
) -> Option<(&'a String, &'a PythonInteropDeclaration)> {
    let Type::Class { name, .. } = ty.resolve_alias() else {
        return None;
    };
    ctx.python_opaque_classes.get_key_value(name)
}

pub(super) fn error_channel_contains(channel: &Type, expected: &Type) -> bool {
    match channel.resolve_alias() {
        Type::Union(members) => members
            .iter()
            .any(|member| error_channel_contains(member, expected)),
        resolved => resolved == expected.resolve_alias(),
    }
}

pub(super) fn python_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("PythonError")
        .cloned()
        .unwrap_or_else(|| Type::Class {
            name: "PythonError".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        })
}

fn contains_python_identity(ty: &Type, ctx: &LowerCtx) -> bool {
    match ty.resolve_alias() {
        Type::Class { name, fields, .. } => {
            name == "Object"
                || ctx.python_opaque_classes.contains_key(name)
                || fields
                    .iter()
                    .any(|(_, field)| contains_python_identity(field, ctx))
        }
        Type::List(item)
        | Type::Set(item)
        | Type::Iterable(item)
        | Type::Iterator(item)
        | Type::Awaitable(item)
        | Type::Newtype { inner: item, .. } => contains_python_identity(item, ctx),
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Coroutine(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => {
            contains_python_identity(left, ctx) || contains_python_identity(right, ctx)
        }
        Type::Tuple(items) | Type::Union(items) | Type::Intersection(items) => {
            items.iter().any(|item| contains_python_identity(item, ctx))
        }
        Type::Callable(arguments, _, result) | Type::AsyncCallable(arguments, _, result) => {
            arguments
                .iter()
                .any(|argument| contains_python_identity(argument, ctx))
                || contains_python_identity(result, ctx)
        }
        Type::Alias { body, .. } => contains_python_identity(body, ctx),
        _ => false,
    }
}

pub(super) fn invalid(ctx: &mut LowerCtx, reason: &str, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYCB_INVALID_DECLARATION,
        format!("invalid Python callback declaration: {reason}"),
        span,
    );
}
