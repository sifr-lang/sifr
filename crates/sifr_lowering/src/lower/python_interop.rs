use super::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    HirParam, PythonCleanupPolicy, PythonInteropDeclaration, PythonInteropDecoratorKind,
    PythonInteropEffect, PythonParameterKind, PythonTargetPath,
};
use sifr_python_ast::{Decorator, Expr, ExprCall, Parameters, Stmt};
use sifr_type_system::Type;

mod callbacks;
mod context;
mod parameters;
mod stub_syntax;

pub(super) use parameters::{parameter_metadata, receiver_is_owned};

pub(in crate::lower) use context::{
    lower_python_context_owned_expr, python_context_borrow_in_owned_expr, python_context_item_kind,
    reject_python_context_borrow_created_value, reject_python_context_borrow_discard,
    reject_python_context_borrow_storage, try_lower_python_async_with,
    validate_context_class_methods,
};
pub(in crate::lower) use stub_syntax::{
    classify_python_interop_stub_body, has_python_interop_decorator_syntax,
    is_bodyless_python_coroutine,
};

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
            _ => {
                reserved_declaration(ctx, kind, span);
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
                reserved_declaration(ctx, kind, span);
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
            call.range,
        );
        return None;
    }
    let target = parse_method_target_path(&call.arguments.args[0], ctx)?;
    Some(PythonInteropDeclaration {
        kind,
        target: Some(target),
        span: call.range,
        effect,
        cleanup: None,
        consumes_receiver: receiver_is_owned(parameters),
        parameters: parameter_metadata(parameters).into_iter().skip(1).collect(),
        required_import_root: None,
        callbacks: Vec::new(),
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
            call.range,
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
            call.range,
        );
        return None;
    }
    Some(PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Attribute,
        target: Some(target),
        span: call.range,
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver,
        parameters,
        required_import_root: None,
        callbacks: Vec::new(),
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
            continue;
        };
        if decorator_path(&call.func).is_none_or(|path| path.as_slice() != ["python", "opaque"]) {
            continue;
        }
        if result.is_some() {
            invalid_shape(
                ctx,
                "a class may have only one `@python.opaque` declaration",
                call.range,
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
        invalid_shape(ctx, "`@python.opaque` requires `type=`", call.range);
        return None;
    };
    let Some(cleanup) = cleanup else {
        invalid_shape(ctx, "`@python.opaque` requires `cleanup=`", call.range);
        return None;
    };
    if matches!(target.root(), Some("Self" | "bridge")) {
        let code = if target.root() == Some("bridge") {
            DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION
        } else {
            DiagnosticCode::PYIMP_INVALID_TARGET
        };
        ctx.error_with_code_at(
            code,
            "invalid Python opaque type target: expected an import-rooted dotted path".to_string(),
            target.span,
        );
        return None;
    }
    Some(PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Opaque,
        required_import_root: target.root().map(str::to_string),
        target: Some(target),
        span: call.range,
        effect: PythonInteropEffect::BlockingIo,
        cleanup: Some(cleanup),
        consumes_receiver: false,
        parameters: Vec::new(),
        callbacks: Vec::new(),
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
    let validation_start = ctx.errors.len();
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
    callbacks::validate(declaration, params, ok_type, error_type, ctx);
    if declaration.callbacks.is_empty()
        && !matches!(error_type.resolve_alias(), Type::Class { name, .. } if name == "PythonError")
    {
        unsupported_conversion(
            ctx,
            "the declaration error type must be `PythonError`",
            declaration.span,
        );
    } else if !declaration.callbacks.is_empty()
        && !callbacks::error_channel_contains(error_type, &callbacks::python_error_type(ctx))
    {
        callbacks::invalid(
            ctx,
            "the enclosing declaration error channel must contain `PythonError`",
            declaration.span,
        );
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
    let mut saw_omittable_positional = false;
    for (param, shape) in params.iter().zip(&declaration.parameters) {
        if declaration
            .callbacks
            .iter()
            .any(|callback| callback.parameter_name == param.name)
        {
            continue;
        }
        if shape.kind == PythonParameterKind::Positional && shape.omit_when_absent {
            saw_omittable_positional = true;
        }
        if shape.kind == PythonParameterKind::PositionalVariadic && saw_omittable_positional {
            invalid_shape(
                ctx,
                "typed `*args` cannot follow an omittable positional parameter",
                shape.span,
            );
        }
        let supported = match shape.kind {
            PythonParameterKind::Positional | PythonParameterKind::KeywordOnly => {
                is_direct_type(&param.ty, true, ctx)
            }
            PythonParameterKind::PositionalVariadic => {
                matches!(param.ty.resolve_alias(), Type::List(element) if is_direct_type(element, false, ctx))
            }
            PythonParameterKind::KeywordVariadic => {
                matches!(param.ty.resolve_alias(), Type::Dict(key, value) if key.resolve_alias() == &Type::Str && is_direct_type(value, false, ctx))
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
    if !declaration.callbacks.is_empty() && ctx.errors.len() == validation_start {
        reserved_declaration(ctx, PythonInteropDecoratorKind::Callback, declaration.span);
    }
}

pub(super) fn is_direct_type(ty: &Type, allow_option: bool, ctx: &LowerCtx) -> bool {
    match ty.resolve_alias() {
        Type::None | Type::Bool | Type::Int | Type::Float | Type::Str | Type::Bytes => true,
        Type::Class { name, .. } if name == "Object" => true,
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
            call.range,
        );
        return None;
    }
    let mut target = parse_target_path(&call.arguments.args[0], ctx)?;
    if target.root() == Some("bridge") {
        let authority = ctx
            .current_module_name
            .as_deref()
            .and_then(|module| ctx.python_bridge_authorities.get(module))
            .cloned();
        let Some(authority) = authority else {
            ctx.error_with_code_at(
                DiagnosticCode::PYIMP_INVALID_TARGET,
                "package-local `bridge` target has no bridge source in its resolved package"
                    .to_string(),
                target.span,
            );
            return None;
        };
        let target_module_resolves = (2..target.segments.len()).any(|end| {
            authority
                .modules
                .contains(&target.segments[1..end].join("."))
        });
        if !target_module_resolves {
            ctx.error_with_code_at(
                DiagnosticCode::PYIMP_INVALID_TARGET,
                format!(
                    "invalid Python declaration target: package-local bridge target '{}' has no inventoried module",
                    target.dotted()
                ),
                target.span,
            );
            return None;
        }
        target.segments.splice(
            0..1,
            authority.runtime_package.split('.').map(str::to_string),
        );
    }
    let required_import_root = target
        .root()
        .filter(|root| !matches!(*root, "Self" | "__sifr_bridge__"))
        .map(str::to_string);
    Some(PythonInteropDeclaration {
        kind,
        target: Some(target),
        span: call.range,
        effect,
        cleanup: None,
        consumes_receiver: false,
        parameters: parameter_metadata(parameters),
        required_import_root,
        callbacks: Vec::new(),
    })
}

fn reserved_declaration(ctx: &mut LowerCtx, kind: PythonInteropDecoratorKind, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION,
        format!(
            "Python declaration lowering is not active yet: `{}` belongs to a later phase",
            decorator_label(kind)
        ),
        span,
    );
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

pub(super) fn invalid_shape(ctx: &mut LowerCtx, reason: &str, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYCALL_INVALID_SHAPE,
        format!("invalid Python declaration call shape: {reason}"),
        span,
    );
}
