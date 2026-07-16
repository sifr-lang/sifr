use super::{DiagnosticCode, Expr, LowerCtx, PythonTargetPath, Ranged, TextRange};

pub(super) fn parse_callable(expr: &Expr, ctx: &mut LowerCtx) -> Option<PythonTargetPath> {
    let mut target = parse_path(expr, ctx)?;
    if target.root() != Some("bridge") {
        return Some(target);
    }

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
    Some(target)
}

pub(in crate::lower) fn parse_path(expr: &Expr, ctx: &mut LowerCtx) -> Option<PythonTargetPath> {
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

pub(in crate::lower) fn decorator_path(expr: &Expr) -> Option<Vec<String>> {
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

pub(in crate::lower) fn invalid_target(ctx: &mut LowerCtx, reason: &str, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::PYIMP_INVALID_TARGET,
        format!("invalid Python declaration target: {reason}"),
        span,
    );
}
