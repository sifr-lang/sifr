use super::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    RustInteropAbiRequirements, RustInteropArgument, RustInteropDeclaration,
    RustInteropDecoratorKind, RustInteropEffect, RustInteropValue, RustTargetPath,
};
use sifr_python_ast::{Decorator, Expr, ExprCall, Number, Stmt, UnaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::lower) enum RustInteropOwner {
    Function,
    Method,
    Class,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::lower) enum RustInteropStubBody {
    Bodyless,
    Invalid,
    Normal,
}

/// Collect declaration-selected Rust opaque close members before bodies lower so
/// call-site ownership sees the receiver as affine.
pub(in crate::lower) fn collect_rust_opaque_close_methods(stmts: &[Stmt], ctx: &mut LowerCtx) {
    for stmt in stmts {
        let Stmt::ClassDef(class_def) = stmt else {
            continue;
        };
        let opaque = rust_opaque_decorator_call(&class_def.decorator_list);
        let Some(opaque) = opaque else {
            continue;
        };
        ctx.rust_opaque_classes.insert(class_def.name.to_string());
        if has_valid_rust_opaque_structural_mapping_syntax(opaque) {
            ctx.rust_structural_classes
                .insert(class_def.name.to_string());
        }
        let close_method = opaque.arguments.keywords.iter().find_map(|keyword| {
            if keyword
                .arg
                .as_ref()
                .is_none_or(|name| name.as_str() != "close")
            {
                return None;
            }
            match &keyword.value {
                Expr::Name(policy) if policy.id.as_str() == "close" => Some("close"),
                Expr::Name(policy) if policy.id.as_str() == "async_close" => Some("aclose"),
                _ => None,
            }
        });
        if let Some(close_method) = close_method {
            ctx.rust_consuming_methods
                .insert(format!("{}.{}", class_def.name, close_method));
        }
    }
}

fn has_valid_rust_opaque_structural_mapping_syntax(opaque: &ExprCall) -> bool {
    opaque.arguments.keywords.iter().any(|keyword| {
        keyword
            .arg
            .as_ref()
            .is_some_and(|name| name.as_str() == "structural")
            && target_path_segments(&keyword.value, RustInteropOwner::Class).is_ok()
    })
}

pub(in crate::lower) fn has_rust_opaque_structural_mapping_syntax(
    decorators: &[Decorator],
) -> bool {
    rust_opaque_decorator_call(decorators).is_some_and(|opaque| {
        opaque.arguments.keywords.iter().any(|keyword| {
            keyword
                .arg
                .as_ref()
                .is_some_and(|name| name.as_str() == "structural")
        })
    })
}

fn rust_opaque_decorator_call(decorators: &[Decorator]) -> Option<&ExprCall> {
    decorators.iter().find_map(|decorator| {
        let Expr::Call(call) = &decorator.expression else {
            return None;
        };
        let Expr::Attribute(attribute) = call.func.as_ref() else {
            return None;
        };
        let Expr::Name(root) = attribute.value.as_ref() else {
            return None;
        };
        (root.id.as_str() == "rust" && attribute.attr.as_str() == "opaque").then_some(call)
    })
}

impl RustInteropStubBody {
    pub(in crate::lower) const fn skips_normal_body_lowering(self) -> bool {
        matches!(self, Self::Bodyless | Self::Invalid)
    }
}

pub(in crate::lower) fn classify_rust_interop_stub_body(
    body: &[Stmt],
    has_rust_interop_decorator: bool,
    ctx: &mut LowerCtx,
) -> RustInteropStubBody {
    let is_exact_ellipsis = is_exact_ellipsis_body(body);
    let contains_ellipsis = body.iter().any(is_ellipsis_stmt);

    if is_exact_ellipsis {
        if has_rust_interop_decorator {
            return RustInteropStubBody::Bodyless;
        }
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            "ellipsis is only supported as the complete body of a Rust interop declaration"
                .to_string(),
            body[0].range(),
        );
        return RustInteropStubBody::Invalid;
    }

    if contains_ellipsis && has_rust_interop_decorator {
        let span = body
            .iter()
            .find(|stmt| is_ellipsis_stmt(stmt))
            .map_or_else(TextRange::default, Ranged::range);
        ctx.error_with_code_at(
            DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR,
            "Rust interop declaration stubs must contain exactly one ellipsis statement and no other statements"
                .to_string(),
            span,
        );
        return RustInteropStubBody::Invalid;
    }

    RustInteropStubBody::Normal
}

pub(in crate::lower) fn has_rust_interop_decorator_syntax(decorators: &[Decorator]) -> bool {
    decorators
        .iter()
        .any(|decorator| uses_rust_decorator_namespace(&decorator.expression))
}

fn is_exact_ellipsis_body(body: &[Stmt]) -> bool {
    matches!(body, [stmt] if is_ellipsis_stmt(stmt))
}

fn is_ellipsis_stmt(stmt: &Stmt) -> bool {
    matches!(stmt, Stmt::Expr(expr_stmt) if matches!(expr_stmt.value.as_ref(), Expr::EllipsisLiteral(_)))
}

fn uses_rust_decorator_namespace(expr: &Expr) -> bool {
    match expr {
        Expr::Call(call) => starts_with_rust_namespace(&call.func),
        _ => starts_with_rust_namespace(expr),
    }
}

pub(in crate::lower) fn collect_rust_interop_declarations(
    decorators: &[Decorator],
    owner: RustInteropOwner,
    ctx: &mut LowerCtx,
    blocking_io: bool,
    cpu_heavy: bool,
    is_async_decl: bool,
) -> Vec<RustInteropDeclaration> {
    let mut declarations = Vec::new();
    for decorator in decorators {
        let Some((kind, call)) = classify_rust_decorator(&decorator.expression, ctx) else {
            continue;
        };
        if !kind_allowed_on_owner(kind, owner) {
            malformed(
                ctx,
                format!(
                    "`{}` is not valid on a {} declaration",
                    decorator_name(kind),
                    owner_name(owner)
                ),
                decorator.expression.range(),
            );
            continue;
        }
        if kind == RustInteropDecoratorKind::Async && !is_async_decl {
            ctx.error_with_code_at(
                DiagnosticCode::RUST_ASYNC_CONTRACT,
                "invalid Rust async contract: `@rust.async(...)` requires `async def`".to_string(),
                decorator.expression.range(),
            );
            continue;
        }
        let declaration = if kind == RustInteropDecoratorKind::Structural {
            Some(RustInteropDeclaration {
                kind,
                target: None,
                arguments: Vec::new(),
                span: decorator.expression.range(),
                effect: RustInteropEffect::Sync,
                abi_requirements: RustInteropAbiRequirements::default(),
                consumes_receiver: false,
            })
        } else {
            call.and_then(|call| parse_declaration(kind, call, owner, is_async_decl, ctx))
        };
        if let Some(declaration) = declaration {
            declarations.push(declaration);
        }
    }

    validate_structural_marker_pair(&declarations, decorators, ctx);

    if is_async_decl && !declarations.is_empty() && (blocking_io || cpu_heavy) {
        ctx.error_with_code_at(
            DiagnosticCode::RUST_ASYNC_CONTRACT,
            "invalid Rust async contract: Rust async interop cannot be combined with blocking or CPU-heavy classification".to_string(),
            decorators
                .first()
                .map_or(TextRange::default(), Ranged::range),
        );
    }

    let effect = declaration_effect(&declarations, blocking_io, cpu_heavy, is_async_decl);
    for declaration in &mut declarations {
        declaration.effect = effect;
    }
    declarations
}

fn classify_rust_decorator<'a>(
    expr: &'a Expr,
    ctx: &mut LowerCtx,
) -> Option<(RustInteropDecoratorKind, Option<&'a ExprCall>)> {
    if let Expr::Call(call) = expr {
        return classify_rust_call(call, ctx).map(|kind| (kind, Some(call)));
    }
    if let Expr::Attribute(attribute) = expr {
        if matches!(attribute.value.as_ref(), Expr::Name(root) if root.id.as_str() == "rust")
            && attribute.attr.as_str() == "structural"
        {
            return Some((RustInteropDecoratorKind::Structural, None));
        }
    }
    if starts_with_rust_namespace(expr) {
        malformed(
            ctx,
            "Rust interop decorators must be call expressions".to_string(),
            expr.range(),
        );
    }
    None
}

fn classify_rust_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<RustInteropDecoratorKind> {
    match call.func.as_ref() {
        Expr::Name(name) if name.id.as_str() == "rust" => Some(RustInteropDecoratorKind::Function),
        Expr::Attribute(attr) => {
            let Expr::Name(root) = attr.value.as_ref() else {
                return None;
            };
            if root.id.as_str() != "rust" {
                return None;
            }
            let kind = match attr.attr.as_str() {
                "opaque" => RustInteropDecoratorKind::Opaque,
                "async" => RustInteropDecoratorKind::Async,
                "callback" => RustInteropDecoratorKind::Callback,
                "structural" => {
                    malformed(
                        ctx,
                        "`@rust.structural` is a bare marker and takes no arguments",
                        call.range,
                    );
                    return None;
                }
                "zero_copy" => RustInteropDecoratorKind::ZeroCopy,
                "view" => RustInteropDecoratorKind::View,
                other => {
                    malformed(
                        ctx,
                        format!("unknown Rust interop decorator `rust.{other}`"),
                        attr.range(),
                    );
                    return None;
                }
            };
            Some(kind)
        }
        _ => None,
    }
}

fn parse_declaration(
    kind: RustInteropDecoratorKind,
    call: &ExprCall,
    owner: RustInteropOwner,
    is_async_decl: bool,
    ctx: &mut LowerCtx,
) -> Option<RustInteropDeclaration> {
    let target = parse_positional_target(kind, call, owner, ctx)?;
    let arguments = parse_keyword_arguments(call, owner, ctx)?;
    let declaration = RustInteropDeclaration {
        kind,
        target,
        arguments,
        span: call.range,
        effect: RustInteropEffect::Sync,
        abi_requirements: abi_requirements(kind, is_async_decl),
        consumes_receiver: false,
    };
    if kind == RustInteropDecoratorKind::Callback {
        if let Err(reason) = sifr_ir::rust_threadsafe_callback_contract(&declaration) {
            ctx.error_with_code_at(
                DiagnosticCode::RUST_CALLBACK_CONTRACT,
                format!("invalid Rust callback contract: {reason}"),
                call.range,
            );
            return None;
        }
    }
    Some(declaration)
}

fn parse_positional_target(
    kind: RustInteropDecoratorKind,
    call: &ExprCall,
    owner: RustInteropOwner,
    ctx: &mut LowerCtx,
) -> Option<Option<RustTargetPath>> {
    match kind {
        RustInteropDecoratorKind::Function => {
            if call.arguments.args.len() != 1 {
                malformed(
                    ctx,
                    "`@rust(...)` requires exactly one dotted Rust target path".to_string(),
                    call.range,
                );
                return None;
            }
            parse_target_path(&call.arguments.args[0], owner, ctx).map(Some)
        }
        RustInteropDecoratorKind::Opaque
        | RustInteropDecoratorKind::Async
        | RustInteropDecoratorKind::Callback
        | RustInteropDecoratorKind::Structural
        | RustInteropDecoratorKind::ZeroCopy
        | RustInteropDecoratorKind::View => {
            if !call.arguments.args.is_empty() {
                malformed(
                    ctx,
                    format!(
                        "`{}` does not accept positional arguments",
                        decorator_name(kind)
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            Some(None)
        }
    }
}

fn parse_keyword_arguments(
    call: &ExprCall,
    owner: RustInteropOwner,
    ctx: &mut LowerCtx,
) -> Option<Vec<RustInteropArgument>> {
    let mut arguments = Vec::with_capacity(call.arguments.keywords.len());
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            malformed(
                ctx,
                "Rust interop decorators do not accept `**kwargs`".to_string(),
                keyword.range(),
            );
            return None;
        };
        let value = parse_value(&keyword.value, owner, ctx)?;
        arguments.push(RustInteropArgument {
            name: Some(name.to_string()),
            value,
            span: keyword.range(),
        });
    }
    Some(arguments)
}

fn parse_value(
    expr: &Expr,
    owner: RustInteropOwner,
    ctx: &mut LowerCtx,
) -> Option<RustInteropValue> {
    match expr {
        Expr::BooleanLiteral(value) => Some(RustInteropValue::Boolean(value.value)),
        Expr::Name(name) => Some(RustInteropValue::Symbol(name.id.to_string())),
        Expr::NumberLiteral(number) => {
            parse_integer_value(&number.value, number.range(), false, ctx)
        }
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::USub) => {
            if let Expr::NumberLiteral(number) = unary.operand.as_ref() {
                parse_integer_value(&number.value, unary.range, true, ctx)
            } else {
                malformed(
                    ctx,
                    "unsupported Rust interop decorator value".to_string(),
                    unary.range,
                );
                None
            }
        }
        Expr::List(list) => parse_integer_list_value(&list.elts, list.range, ctx),
        Expr::Tuple(tuple) => parse_integer_list_value(&tuple.elts, tuple.range, ctx),
        Expr::Attribute(_) => parse_target_path(expr, owner, ctx).map(RustInteropValue::TargetPath),
        Expr::Call(call) => parse_policy_call(call, owner, ctx),
        Expr::StringLiteral(_) => {
            malformed(
                ctx,
                "string decorator values are not part of the Rust interop grammar".to_string(),
                expr.range(),
            );
            None
        }
        _ => {
            malformed(
                ctx,
                "unsupported Rust interop decorator value".to_string(),
                expr.range(),
            );
            None
        }
    }
}

fn parse_integer_value(
    number: &Number,
    span: TextRange,
    negate: bool,
    ctx: &mut LowerCtx,
) -> Option<RustInteropValue> {
    parse_integer_literal(number, span, negate, ctx).map(RustInteropValue::Integer)
}

fn parse_integer_literal(
    number: &Number,
    span: TextRange,
    negate: bool,
    ctx: &mut LowerCtx,
) -> Option<i64> {
    if let Number::Int(value) = number {
        let Some(value) = value.as_i64() else {
            malformed(ctx, "integer decorator value is too large", span);
            return None;
        };
        let value = if negate {
            value.checked_neg()
        } else {
            Some(value)
        };
        value.or_else(|| {
            malformed(ctx, "integer decorator value is too large", span);
            None
        })
    } else {
        malformed(
            ctx,
            "only integer numeric decorator values are supported",
            span,
        );
        None
    }
}

fn parse_integer_list_value(
    elements: &[Expr],
    span: TextRange,
    ctx: &mut LowerCtx,
) -> Option<RustInteropValue> {
    let mut values = Vec::with_capacity(elements.len());
    for element in elements {
        let value = match element {
            Expr::NumberLiteral(number) => {
                parse_integer_literal(&number.value, number.range(), false, ctx)
            }
            Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::USub) => {
                if let Expr::NumberLiteral(number) = unary.operand.as_ref() {
                    parse_integer_literal(&number.value, unary.range, true, ctx)
                } else {
                    malformed(
                        ctx,
                        "integer list decorator values must contain only integer literals"
                            .to_string(),
                        unary.range,
                    );
                    None
                }
            }
            _ => {
                malformed(
                    ctx,
                    "integer list decorator values must contain only integer literals".to_string(),
                    element.range(),
                );
                None
            }
        }?;
        values.push(value);
    }
    if values.is_empty() {
        malformed(
            ctx,
            "integer list decorator values cannot be empty".to_string(),
            span,
        );
        return None;
    }
    Some(RustInteropValue::IntegerList(values))
}

fn parse_policy_call(
    call: &ExprCall,
    owner: RustInteropOwner,
    ctx: &mut LowerCtx,
) -> Option<RustInteropValue> {
    let Expr::Name(name) = call.func.as_ref() else {
        malformed(
            ctx,
            "policy calls must use an identifier function name".to_string(),
            call.func.range(),
        );
        return None;
    };
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        malformed(
            ctx,
            "policy calls accept exactly one positional argument".to_string(),
            call.range,
        );
        return None;
    }
    let argument_expr = &call.arguments.args[0];
    let argument = match argument_expr {
        Expr::NumberLiteral(_) | Expr::UnaryOp(_) | Expr::Attribute(_) => {
            parse_value(argument_expr, owner, ctx)?
        }
        _ => {
            malformed(
                ctx,
                "policy call arguments must be an integer or dotted Rust target path".to_string(),
                argument_expr.range(),
            );
            return None;
        }
    };
    Some(RustInteropValue::PolicyCall {
        name: name.id.to_string(),
        argument: Box::new(argument),
        span: call.range,
    })
}

fn parse_target_path(
    expr: &Expr,
    owner: RustInteropOwner,
    ctx: &mut LowerCtx,
) -> Option<RustTargetPath> {
    let segments = match target_path_segments(expr, owner) {
        Ok(segments) => segments,
        Err(message) => {
            malformed(ctx, message.to_string(), expr.range());
            return None;
        }
    };
    Some(RustTargetPath {
        segments,
        span: expr.range(),
    })
}

fn target_path_segments(expr: &Expr, owner: RustInteropOwner) -> Result<Vec<String>, &'static str> {
    let mut segments = Vec::new();
    collect_path_segments(expr, &mut segments)
        .ok_or("Rust target must be a dotted identifier path, not a dynamic expression")?;
    if segments.len() < 2 {
        return Err("Rust target path must include a root and item name");
    }
    let root = &segments[0];
    if root == "rust" {
        return Err("`rust` is the decorator namespace and cannot be a Rust target root");
    }
    if root == "Self" && owner != RustInteropOwner::Method {
        return Err(
            "`Self` Rust target paths are valid only on methods inside Rust opaque classes",
        );
    }
    Ok(segments)
}

fn collect_path_segments(expr: &Expr, segments: &mut Vec<String>) -> Option<()> {
    match expr {
        Expr::Name(name) => {
            segments.push(name.id.to_string());
            Some(())
        }
        Expr::Attribute(attribute) => {
            collect_path_segments(&attribute.value, segments)?;
            segments.push(attribute.attr.to_string());
            Some(())
        }
        _ => None,
    }
}

fn starts_with_rust_namespace(expr: &Expr) -> bool {
    match expr {
        Expr::Name(name) => name.id.as_str() == "rust",
        Expr::Attribute(attribute) => starts_with_rust_namespace(&attribute.value),
        _ => false,
    }
}

fn kind_allowed_on_owner(kind: RustInteropDecoratorKind, owner: RustInteropOwner) -> bool {
    match owner {
        RustInteropOwner::Class => kind == RustInteropDecoratorKind::Opaque,
        RustInteropOwner::Function | RustInteropOwner::Method => {
            !matches!(kind, RustInteropDecoratorKind::Opaque)
                && (kind != RustInteropDecoratorKind::Structural
                    || owner == RustInteropOwner::Function)
        }
    }
}

fn declaration_effect(
    declarations: &[RustInteropDeclaration],
    blocking_io: bool,
    cpu_heavy: bool,
    is_async_decl: bool,
) -> RustInteropEffect {
    if blocking_io {
        RustInteropEffect::BlockingIo
    } else if cpu_heavy {
        RustInteropEffect::CpuHeavy
    } else if (is_async_decl && !declarations.is_empty())
        || declarations
            .iter()
            .any(|declaration| declaration.kind == RustInteropDecoratorKind::Async)
    {
        RustInteropEffect::Async
    } else {
        RustInteropEffect::Sync
    }
}

fn abi_requirements(
    kind: RustInteropDecoratorKind,
    is_async_decl: bool,
) -> RustInteropAbiRequirements {
    RustInteropAbiRequirements {
        async_boundary: kind == RustInteropDecoratorKind::Async || is_async_decl,
        opaque_handle: kind == RustInteropDecoratorKind::Opaque,
        zero_copy: kind == RustInteropDecoratorKind::ZeroCopy,
        view: kind == RustInteropDecoratorKind::View,
    }
}

fn decorator_name(kind: RustInteropDecoratorKind) -> &'static str {
    match kind {
        RustInteropDecoratorKind::Function => "@rust",
        RustInteropDecoratorKind::Opaque => "@rust.opaque",
        RustInteropDecoratorKind::Async => "@rust.async",
        RustInteropDecoratorKind::Callback => "@rust.callback",
        RustInteropDecoratorKind::Structural => "@rust.structural",
        RustInteropDecoratorKind::ZeroCopy => "@rust.zero_copy",
        RustInteropDecoratorKind::View => "@rust.view",
    }
}

fn validate_structural_marker_pair(
    declarations: &[RustInteropDeclaration],
    decorators: &[Decorator],
    ctx: &mut LowerCtx,
) {
    let markers = declarations
        .iter()
        .filter(|declaration| declaration.kind == RustInteropDecoratorKind::Structural)
        .collect::<Vec<_>>();
    if markers.is_empty() {
        return;
    }
    let span = markers[0].span;
    if markers.len() != 1 {
        malformed(ctx, "duplicate `@rust.structural` markers", span);
    }
    let targets = declarations
        .iter()
        .filter(|declaration| {
            declaration.kind == RustInteropDecoratorKind::Function && declaration.target.is_some()
        })
        .count();
    if targets != 1 {
        malformed(
            ctx,
            "`@rust.structural` must accompany exactly one `@rust(...)` target",
            decorators.first().map_or(span, Ranged::range),
        );
    }
}

fn owner_name(owner: RustInteropOwner) -> &'static str {
    match owner {
        RustInteropOwner::Function => "function",
        RustInteropOwner::Method => "method",
        RustInteropOwner::Class => "class",
    }
}

fn malformed(ctx: &mut LowerCtx, reason: impl std::fmt::Display, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR,
        format!("malformed Rust interop decorator: {reason}"),
        span,
    );
}
