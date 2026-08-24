use super::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirParam, PythonCleanupPolicy, PythonInteropDeclaration, PythonInteropDecoratorKind,
    PythonInteropEffect, PythonParameterKind, PythonTargetPath,
};
use sifr_python_ast::{Decorator, Expr, ExprCall, Parameters, Stmt};
use sifr_type_system::Type;

mod arrow;
mod buffer;
mod callbacks;
mod callsite;
mod context;
mod direct_validation;
mod dlpack;
mod parameters;
mod stub_syntax;
mod target;

use direct_validation::validate_direct_parameters;
pub(in crate::lower) use direct_validation::validate_raw_conversion_intrinsic;

pub(in crate::lower) use callsite::{
    CallbackCallPolicy, callback_call_policies, callback_method_arg_ranges,
    validate_callback_call_captures,
};
pub(super) use parameters::{parameter_metadata, receiver_is_owned};
pub(super) use target::{decorator_path, invalid_target, parse_path as parse_target_path};

pub(in crate::lower) use context::{
    lower_python_context_owned_expr, python_context_borrow_in_owned_expr,
    python_context_borrow_reference, python_context_item_kind,
    reject_python_context_borrow_created_value, reject_python_context_borrow_discard,
    reject_python_context_borrow_storage, try_lower_python_async_with,
    validate_context_class_methods,
};
pub(in crate::lower) use stub_syntax::{
    classify_python_interop_stub_body, has_python_interop_decorator_syntax,
    is_bodyless_python_coroutine, is_python_rooted_decorator_expr,
};

pub(in crate::lower) fn validate_retained_callback_owner_errors(
    functions: &[sifr_ir::HirFunction],
    classes: &[sifr_ir::HirClass],
    ctx: &mut LowerCtx,
) {
    callbacks::validate_retained_owner_error_channels(functions, classes, ctx);
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

pub(in crate::lower) fn collect_python_interop_declarations(
    decorators: &[Decorator],
    parameters: &Parameters,
    is_async_decl: bool,
    ctx: &mut LowerCtx,
) -> Vec<PythonInteropDeclaration> {
    let mut declarations = Vec::new();
    let mut callbacks = Vec::new();
    let mut callback_parse_failed = false;
    for decorator in decorators {
        let Some((kind, call, span)) = classify_decorator(&decorator.expression, ctx) else {
            continue;
        };
        let declaration = match (kind, is_async_decl) {
            (PythonInteropDecoratorKind::Callback, _) => {
                if let Some(callback) = callbacks::parse(call, parameters, false, ctx) {
                    callbacks.push(callback);
                } else {
                    callback_parse_failed = true;
                }
                None
            }
            (PythonInteropDecoratorKind::Function, false) => {
                parse_function(call, parameters, kind, PythonInteropEffect::BlockingIo, ctx)
            }
            (PythonInteropDecoratorKind::Function, true) => {
                invalid_shape(
                    ctx,
                    "`@python(path)` requires `def`; use `@python.coroutine(path)` for `async def`",
                    span,
                );
                None
            }
            (PythonInteropDecoratorKind::Coroutine, true) => {
                parse_function(call, parameters, kind, PythonInteropEffect::Async, ctx)
            }
            (PythonInteropDecoratorKind::Coroutine, false) => {
                invalid_shape(ctx, "`@python.coroutine(path)` requires `async def`", span);
                None
            }
            (PythonInteropDecoratorKind::Buffer, false) => {
                buffer::parse_declaration(call, parameters, false, ctx)
            }
            (PythonInteropDecoratorKind::Buffer, true) => {
                buffer::invalid(ctx, "`@python.buffer` requires synchronous `def`", span);
                None
            }
            (PythonInteropDecoratorKind::Arrow, false) => {
                arrow::parse_declaration(call, parameters, false, ctx)
            }
            (PythonInteropDecoratorKind::Arrow, true) => {
                arrow::invalid(ctx, "`@python.arrow` requires synchronous `def`", span);
                None
            }
            (PythonInteropDecoratorKind::Dlpack, false) => {
                dlpack::parse_declaration(call, parameters, false, false, ctx)
            }
            (PythonInteropDecoratorKind::DlpackStream, false) => {
                dlpack::parse_declaration(call, parameters, false, true, ctx)
            }
            (
                PythonInteropDecoratorKind::Dlpack | PythonInteropDecoratorKind::DlpackStream,
                true,
            ) => {
                dlpack::invalid(ctx, "DLPack declarations require synchronous `def`", span);
                None
            }
            _ => {
                invalid_shape(
                    ctx,
                    "this Python decorator is not valid on a function declaration",
                    span,
                );
                None
            }
        };
        if let Some(declaration) = declaration {
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
    if callback_parse_failed {
        callbacks.clear();
    }
    self::callbacks::attach(&mut declarations, callbacks, ctx);
    declarations
}

pub(in crate::lower) fn collect_python_method_declarations(
    decorators: &[Decorator],
    parameters: &Parameters,
    is_async_decl: bool,
    ctx: &mut LowerCtx,
) -> Vec<PythonInteropDeclaration> {
    let mut declarations = Vec::new();
    let mut callbacks = Vec::new();
    let mut callback_parse_failed = false;
    let has_receiver = !decorators.iter().any(|decorator| {
        decorator_path(&decorator.expression).is_some_and(|path| {
            matches!(path.as_slice(), [name] if name == "classmethod" || name == "staticmethod")
        })
    });
    for decorator in decorators {
        if decorator_path(&decorator.expression)
            .is_some_and(|path| path.as_slice() == ["python", "item"])
        {
            if is_async_decl {
                invalid_shape(
                    ctx,
                    "`@python.item` requires synchronous `def`",
                    decorator.expression.range(),
                );
                continue;
            }
            declarations.push(PythonInteropDeclaration {
                kind: PythonInteropDecoratorKind::Item,
                target: None,
                span: decorator.expression.range(),
                effect: PythonInteropEffect::BlockingIo,
                cleanup: None,
                consumes_receiver: receiver_is_owned(parameters),
                parameters: parameter_metadata(parameters).into_iter().skip(1).collect(),
                required_import_root: None,
                callbacks: Vec::new(),
                buffer: None,
                arrow: None,
                dlpack: None,
            });
            continue;
        }
        let Some((kind, call, span)) = classify_decorator(&decorator.expression, ctx) else {
            continue;
        };
        let declaration = match (kind, is_async_decl) {
            (PythonInteropDecoratorKind::Callback, _) => {
                if let Some(callback) = callbacks::parse(call, parameters, has_receiver, ctx) {
                    callbacks.push(callback);
                } else {
                    callback_parse_failed = true;
                }
                None
            }
            (PythonInteropDecoratorKind::Coroutine, true) => {
                parse_method(call, parameters, kind, PythonInteropEffect::Async, ctx)
            }
            (PythonInteropDecoratorKind::Coroutine, false) => {
                invalid_shape(
                    ctx,
                    "`@python.coroutine(Self.method)` requires `async def`",
                    span,
                );
                None
            }
            (PythonInteropDecoratorKind::Function, true) => {
                invalid_shape(
                    ctx,
                    "`@python(Self.method)` requires `def`; use `@python.coroutine(Self.method)` for `async def`",
                    span,
                );
                None
            }
            (PythonInteropDecoratorKind::Function, false) => {
                parse_method(call, parameters, kind, PythonInteropEffect::BlockingIo, ctx)
            }
            (PythonInteropDecoratorKind::Buffer, false) => {
                if has_receiver {
                    buffer::parse_declaration(call, parameters, true, ctx)
                } else {
                    buffer::invalid(
                        ctx,
                        "`@python.buffer(Self, ...)` requires an opaque instance method",
                        span,
                    );
                    None
                }
            }
            (PythonInteropDecoratorKind::Buffer, true) => {
                buffer::invalid(ctx, "`@python.buffer` requires synchronous `def`", span);
                None
            }
            (PythonInteropDecoratorKind::Arrow, false) => {
                if has_receiver {
                    arrow::parse_declaration(call, parameters, true, ctx)
                } else {
                    arrow::invalid(
                        ctx,
                        "`@python.arrow(Self, ...)` requires an opaque instance method",
                        span,
                    );
                    None
                }
            }
            (PythonInteropDecoratorKind::Arrow, true) => {
                arrow::invalid(ctx, "`@python.arrow` requires synchronous `def`", span);
                None
            }
            (
                PythonInteropDecoratorKind::Dlpack | PythonInteropDecoratorKind::DlpackStream,
                false,
            ) => {
                if has_receiver {
                    dlpack::parse_declaration(
                        call,
                        parameters,
                        true,
                        kind == PythonInteropDecoratorKind::DlpackStream,
                        ctx,
                    )
                } else {
                    dlpack::invalid(
                        ctx,
                        "DLPack `Self` acquisition requires an opaque instance method",
                        span,
                    );
                    None
                }
            }
            (
                PythonInteropDecoratorKind::Dlpack | PythonInteropDecoratorKind::DlpackStream,
                true,
            ) => {
                dlpack::invalid(ctx, "DLPack declarations require synchronous `def`", span);
                None
            }
            (PythonInteropDecoratorKind::Attribute, false) => {
                parse_attribute_method(call, parameters, ctx)
            }
            (
                PythonInteropDecoratorKind::ContextEnter | PythonInteropDecoratorKind::ContextExit,
                false,
            ) => context::parse_context_method(kind, call, parameters, ctx),
            (
                PythonInteropDecoratorKind::ContextAsyncEnter
                | PythonInteropDecoratorKind::ContextAsyncExit,
                true,
            ) => context::parse_context_method(kind, call, parameters, ctx),
            (PythonInteropDecoratorKind::Item, false) => {
                invalid_shape(ctx, "`@python.item` does not take arguments", span);
                None
            }
            (
                PythonInteropDecoratorKind::Attribute
                | PythonInteropDecoratorKind::ContextEnter
                | PythonInteropDecoratorKind::ContextExit
                | PythonInteropDecoratorKind::Item,
                true,
            ) => {
                invalid_shape(
                    ctx,
                    "opaque Python methods must use synchronous `def`",
                    span,
                );
                None
            }
            (
                PythonInteropDecoratorKind::ContextAsyncEnter
                | PythonInteropDecoratorKind::ContextAsyncExit,
                false,
            ) => {
                invalid_shape(
                    ctx,
                    "asynchronous Python context methods require `async def`",
                    span,
                );
                None
            }
            _ => {
                invalid_shape(
                    ctx,
                    "this Python decorator is not valid on an opaque method declaration",
                    span,
                );
                None
            }
        };
        if let Some(declaration) = declaration {
            declarations.push(declaration);
        }
    }
    if declarations.len() > 1 {
        invalid_shape(
            ctx,
            "a method may have only one Python implementation declaration",
            declarations[1].span,
        );
        declarations.truncate(1);
    }
    if callback_parse_failed {
        callbacks.clear();
    }
    self::callbacks::attach(&mut declarations, callbacks, ctx);
    declarations
}

fn parse_method(
    call: &ExprCall,
    parameters: &Parameters,
    kind: PythonInteropDecoratorKind,
    effect: PythonInteropEffect,
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        invalid_shape(
            ctx,
            "`@python(Self.method)` requires one target",
            call.range(),
        );
        return None;
    }
    let target = parse_method_target_path(&call.arguments.args[0], ctx)?;
    Some(PythonInteropDeclaration {
        kind,
        target: Some(target),
        span: call.range(),
        effect,
        cleanup: None,
        consumes_receiver: receiver_is_owned(parameters),
        parameters: parameter_metadata(parameters).into_iter().skip(1).collect(),
        required_import_root: None,
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: None,
    })
}

fn parse_attribute_method(
    call: &ExprCall,
    parameters: &Parameters,
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        invalid_shape(
            ctx,
            "`@python.attr(Self.name)` requires one target",
            call.range(),
        );
        return None;
    }
    let target = parse_method_target_path(&call.arguments.args[0], ctx)?;
    let consumes_receiver = receiver_is_owned(parameters);
    let parameters = parameter_metadata(parameters)
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>();
    if !parameters.is_empty() {
        invalid_shape(
            ctx,
            "a Python attribute declaration takes no parameters",
            call.range(),
        );
        return None;
    }
    Some(PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Attribute,
        target: Some(target),
        span: call.range(),
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver,
        parameters,
        required_import_root: None,
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: None,
    })
}

pub(super) fn parse_method_target_path(
    expr: &Expr,
    ctx: &mut LowerCtx,
) -> Option<PythonTargetPath> {
    let segments = decorator_path(expr)?;
    if segments.len() != 2 || segments[0] != "Self" {
        invalid_target(
            ctx,
            "opaque method targets must be `Self.name`",
            expr.range(),
        );
        return None;
    }
    Some(PythonTargetPath {
        segments,
        span: expr.range(),
    })
}

pub(in crate::lower) fn collect_python_opaque_declaration(
    decorators: &[Decorator],
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    let mut result = None;
    for decorator in decorators {
        let Expr::Call(call) = &decorator.expression else {
            if has_python_interop_decorator_syntax(std::slice::from_ref(decorator)) {
                invalid_shape(
                    ctx,
                    "Python class declarations must use `@python.opaque(type=..., cleanup=...)`",
                    decorator.range,
                );
            }
            continue;
        };
        if decorator_path(&call.func).is_none_or(|path| path.as_slice() != ["python", "opaque"]) {
            if has_python_interop_decorator_syntax(std::slice::from_ref(decorator)) {
                invalid_shape(
                    ctx,
                    "Python class declarations must use `@python.opaque(type=..., cleanup=...)`",
                    decorator.range,
                );
            }
            continue;
        }
        if result.is_some() {
            invalid_shape(
                ctx,
                "a class may have only one `@python.opaque` declaration",
                call.range(),
            );
            continue;
        }
        result = parse_opaque_class(call, ctx);
    }
    result
}

/// Collect opaque identity and consuming-method metadata before signature lowering.
pub(in crate::lower) fn collect_python_opaque_classes(stmts: &[Stmt], ctx: &mut LowerCtx) {
    for stmt in stmts {
        let Stmt::ClassDef(class_def) = stmt else {
            continue;
        };
        let Some(declaration) = collect_python_opaque_declaration(&class_def.decorator_list, ctx)
        else {
            continue;
        };
        ctx.python_opaque_classes
            .insert(class_def.name.to_string(), declaration);
        for body_stmt in &class_def.body {
            let Stmt::FunctionDef(method) = body_stmt else {
                continue;
            };
            let qualified = format!("{}.{}", class_def.name, method.name);
            if method_consumes_receiver(&method.decorator_list, &method.parameters) {
                ctx.python_consuming_methods.insert(qualified.clone());
            }
            if method_is_context_exit(&method.decorator_list) {
                ctx.python_context_exit_methods.insert(qualified);
            }
        }
    }
}

fn parse_opaque_class(call: &ExprCall, ctx: &mut LowerCtx) -> Option<PythonInteropDeclaration> {
    if !call.arguments.args.is_empty() {
        invalid_shape(
            ctx,
            "`@python.opaque` accepts only `type=` and `cleanup=` keyword arguments",
            call.arguments.args[0].range(),
        );
        return None;
    }
    let mut target = None;
    let mut cleanup = None;
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            invalid_shape(
                ctx,
                "`@python.opaque` does not accept `**kwargs`",
                keyword.range(),
            );
            return None;
        };
        match name.as_str() {
            "type" if target.is_none() => target = parse_target_path(&keyword.value, ctx),
            "cleanup" if cleanup.is_none() => {
                let Some(path) = decorator_path(&keyword.value) else {
                    invalid_shape(
                        ctx,
                        "opaque cleanup must be a closed policy atom",
                        keyword.value.range(),
                    );
                    return None;
                };
                cleanup = match path.as_slice() {
                    [name] if name == "drop" => Some(PythonCleanupPolicy::Drop),
                    [name] if name == "close" => Some(PythonCleanupPolicy::Close),
                    [name] if name == "async_close" => Some(PythonCleanupPolicy::AsyncClose),
                    [name] if name == "context" => Some(PythonCleanupPolicy::Context),
                    [name] if name == "async_context" => Some(PythonCleanupPolicy::AsyncContext),
                    _ => {
                        invalid_shape(ctx, "unknown opaque cleanup policy", keyword.value.range());
                        None
                    }
                };
            }
            "type" | "cleanup" => {
                invalid_shape(
                    ctx,
                    &format!("duplicate `@python.opaque` argument `{name}`"),
                    keyword.range(),
                );
                return None;
            }
            _ => {
                invalid_shape(
                    ctx,
                    &format!("unknown `@python.opaque` argument `{name}`"),
                    keyword.range(),
                );
                return None;
            }
        }
    }
    let Some(target) = target else {
        invalid_shape(ctx, "`@python.opaque` requires `type=`", call.range());
        return None;
    };
    let Some(cleanup) = cleanup else {
        invalid_shape(ctx, "`@python.opaque` requires `cleanup=`", call.range());
        return None;
    };
    if target.root() == Some("bridge") {
        ctx.error_with_code_at(
            DiagnosticCode::PYRES_UNSUPPORTED_RESOURCE_DECLARATION,
            "unsupported Python resource declaration: package-local `bridge.*` opaque type targets are not supported; use an import-rooted external type and adapt package-local producers through function declarations".to_string(),
            target.span,
        );
        return None;
    }
    if target.root() == Some("Self") {
        ctx.error_with_code_at(
            DiagnosticCode::PYIMP_INVALID_TARGET,
            "invalid Python opaque type target: expected an import-rooted dotted path".to_string(),
            target.span,
        );
        return None;
    }
    Some(PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Opaque,
        required_import_root: target.root().map(str::to_string),
        target: Some(target),
        span: call.range(),
        effect: PythonInteropEffect::BlockingIo,
        cleanup: Some(cleanup),
        consumes_receiver: false,
        parameters: Vec::new(),
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: None,
    })
}

pub(in crate::lower) fn validate_python_interop_signature(
    declarations: &mut [PythonInteropDeclaration],
    params: &[HirParam],
    return_type: &Type,
    ctx: &mut LowerCtx,
) {
    let Some(declaration) = declarations.first_mut() else {
        return;
    };
    if matches!(
        declaration.kind,
        PythonInteropDecoratorKind::ContextEnter
            | PythonInteropDecoratorKind::ContextExit
            | PythonInteropDecoratorKind::ContextAsyncEnter
            | PythonInteropDecoratorKind::ContextAsyncExit
    ) {
        context::validate_context_method_signature(declaration, params, return_type, ctx);
        return;
    }
    let Type::Result(ok_type, error_type) = return_type.resolve_alias() else {
        let declaration_kind = if declaration.effect == PythonInteropEffect::Async {
            "an asynchronous"
        } else {
            "a synchronous"
        };
        unsupported_conversion(
            ctx,
            &format!("{declaration_kind} declaration must return `Result[T, PythonError]`"),
            declaration.span,
        );
        return;
    };
    if buffer::validate_signature(declaration, ok_type, error_type, ctx) {
        validate_direct_parameters(declaration, params, ctx);
        return;
    }
    if arrow::validate_signature(declaration, params, ok_type, error_type, ctx) {
        validate_direct_parameters(declaration, params, ctx);
        return;
    }
    if dlpack::validate_signature(declaration, params, ok_type, error_type, ctx) {
        validate_direct_parameters(declaration, params, ctx);
        return;
    }
    callbacks::validate(declaration, params, ok_type, error_type, ctx);
    if declaration.callbacks.is_empty() && !error_type.is_python_error_contract() {
        unsupported_conversion(
            ctx,
            "the declaration error type must satisfy the canonical `PythonError` field contract",
            declaration.span,
        );
    } else if !declaration.callbacks.is_empty() {
        if let Some(name) = callbacks::error_channel_codegen_payload_collision(error_type) {
            callbacks::invalid(
                ctx,
                &format!(
                    "the enclosing declaration error channel contains multiple members with generated payload type `{name}`"
                ),
                declaration.span,
            );
        } else if !callbacks::error_channel_contains_python_error_contract(error_type) {
            callbacks::invalid(
                ctx,
                "the enclosing declaration error channel must contain the canonical `PythonError` field contract",
                declaration.span,
            );
        }
    }
    if !is_direct_type(ok_type, true, ctx) {
        let declaration_kind = if declaration.effect == PythonInteropEffect::Async {
            "asynchronous"
        } else {
            "synchronous"
        };
        unsupported_conversion(
            ctx,
            &format!(
                "return type `{}` is not in the {declaration_kind} direct-conversion set",
                ok_type.display_name()
            ),
            declaration.span,
        );
    }
    validate_direct_parameters(declaration, params, ctx);
}

pub(in crate::lower) fn is_direct_type(ty: &Type, allow_option: bool, ctx: &LowerCtx) -> bool {
    match ty.resolve_alias() {
        Type::None | Type::Bool | Type::Int | Type::Float | Type::Str | Type::Bytes => true,
        object if object.is_python_object_contract() => true,
        Type::Class { name, .. } if ctx.python_opaque_classes.contains_key(name) => true,
        Type::Class { name, fields, .. } => {
            !ctx.error_types.contains(name)
                && !fields.is_empty()
                && fields
                    .iter()
                    .all(|(_, field)| is_direct_type(field, true, ctx))
        }
        Type::List(item) => is_direct_type(item, true, ctx),
        Type::Tuple(items) => items.iter().all(|item| is_direct_type(item, true, ctx)),
        Type::Dict(key, value) => {
            key.resolve_alias() == &Type::Str && is_direct_type(value, true, ctx)
        }
        Type::Union(variants) if allow_option && variants.len() == 2 => {
            variants
                .iter()
                .any(|variant| variant.resolve_alias() == &Type::None)
                && variants.iter().all(|variant| {
                    variant.resolve_alias() == &Type::None || is_direct_type(variant, false, ctx)
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

fn parse_function(
    call: &ExprCall,
    parameters: &Parameters,
    kind: PythonInteropDecoratorKind,
    effect: PythonInteropEffect,
    ctx: &mut LowerCtx,
) -> Option<PythonInteropDeclaration> {
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        invalid_shape(
            ctx,
            "`@python(...)` requires exactly one dotted target path and no keyword arguments",
            call.range(),
        );
        return None;
    }
    let target = target::parse_callable(&call.arguments.args[0], ctx)?;
    let required_import_root = target
        .root()
        .filter(|root| !matches!(*root, "Self" | "__sifr_bridge__"))
        .map(str::to_string);
    Some(PythonInteropDeclaration {
        kind,
        target: Some(target),
        span: call.range(),
        effect,
        cleanup: None,
        consumes_receiver: false,
        parameters: parameter_metadata(parameters),
        required_import_root,
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: None,
    })
}

pub(in crate::lower) fn method_consumes_receiver(
    decorators: &[Decorator],
    parameters: &Parameters,
) -> bool {
    receiver_is_owned(parameters)
        && decorators.iter().any(|decorator| {
            let Expr::Call(call) = &decorator.expression else {
                return false;
            };
            decorator_path(&call.func).is_some_and(|path| {
                (path.as_slice() == ["python"]
                    && call.arguments.args.first().is_some_and(|target| {
                        decorator_path(target)
                            .is_some_and(|path| path.as_slice() == ["Self", "close"])
                    }))
                    || (path.as_slice() == ["python", "coroutine"]
                        && call.arguments.args.first().is_some_and(|target| {
                            decorator_path(target)
                                .is_some_and(|path| path.as_slice() == ["Self", "aclose"])
                        }))
                    || path.as_slice() == ["python", "context", "exit"]
            })
        })
}

pub(in crate::lower) fn method_is_context_exit(decorators: &[Decorator]) -> bool {
    decorators.iter().any(|decorator| {
        matches!(&decorator.expression, Expr::Call(call) if decorator_path(&call.func).is_some_and(|path| {
            path.as_slice() == ["python", "context", "exit"]
                || path.as_slice() == ["python", "context", "aexit"]
        }))
    })
}

fn classify_decorator<'a>(
    expr: &'a Expr,
    ctx: &mut LowerCtx,
) -> Option<(PythonInteropDecoratorKind, &'a ExprCall, TextRange)> {
    let Expr::Call(call) = expr else {
        if is_python_rooted_decorator_expr(expr) {
            invalid_shape(
                ctx,
                "Python interop decorators must be call expressions",
                expr.range(),
            );
        }
        return None;
    };
    let Some(path) = decorator_path(&call.func) else {
        if is_python_rooted_decorator_expr(&call.func) {
            invalid_shape(
                ctx,
                "Python declaration decorators cannot be called, indexed, or accessed after the declaration call",
                call.func.range(),
            );
        }
        return None;
    };
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
        [_, dlpack, name] if dlpack == "dlpack" && name == "stream" => {
            PythonInteropDecoratorKind::DlpackStream
        }
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

fn is_ellipsis_stmt(stmt: &Stmt) -> bool {
    matches!(stmt, Stmt::Expr(expr) if matches!(expr.value.as_ref(), Expr::EllipsisLiteral(_)))
}

pub(super) fn invalid_shape(ctx: &mut LowerCtx, reason: &str, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYCALL_INVALID_SHAPE,
        format!("invalid Python declaration call shape: {reason}"),
        span,
    );
}
