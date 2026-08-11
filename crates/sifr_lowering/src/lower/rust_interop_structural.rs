use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{HirParam, RustInteropDeclaration, RustInteropDecoratorKind, RustInteropValue};
use sifr_type_system::Type;

use super::LowerCtx;

#[derive(Clone, Copy)]
pub(in crate::lower) struct StructuralFunctionContract<'a> {
    pub(in crate::lower) function_name: &'a str,
    pub(in crate::lower) params: &'a [HirParam],
    pub(in crate::lower) return_type: &'a Type,
    pub(in crate::lower) type_params: &'a [String],
    pub(in crate::lower) declarations: &'a [RustInteropDeclaration],
    pub(in crate::lower) is_async: bool,
    pub(in crate::lower) span: TextRange,
}

pub(in crate::lower) fn validate_structural_function_contract(
    contract: StructuralFunctionContract<'_>,
    ctx: &mut LowerCtx,
) {
    let StructuralFunctionContract {
        function_name,
        params,
        return_type,
        type_params,
        declarations,
        is_async,
        span,
    } = contract;
    if !declarations
        .iter()
        .any(|declaration| declaration.kind == RustInteropDecoratorKind::Structural)
    {
        return;
    }
    if is_async {
        structural_type_error(
            ctx,
            "structural Rust bridge declarations must be synchronous",
            span,
        );
    }
    let marker_shadowed = ctx.class_types.get("Structural").is_some_and(|marker| {
        !matches!(
            marker.resolve_alias(),
            Type::Class {
                identity: Some(identity),
                ..
            } if identity == "sifr.meta.Structural"
        )
    });
    if !ctx.canonical_structural_marker_imported
        || ctx.local_structural_marker_declared
        || marker_shadowed
    {
        structural_type_error(
            ctx,
            "the `Structural` bound must be the unaliased compiler-owned `from sifr.meta import Structural` marker",
            span,
        );
    }
    validate_error_and_panic_contract(return_type, declarations, span, ctx);
    let [type_param] = type_params else {
        structural_type_error(
            ctx,
            "structural Rust bridge declarations require exactly one type parameter",
            span,
        );
        return;
    };
    let has_exact_bound = ctx
        .type_param_bounds
        .get(function_name)
        .and_then(|bounds| bounds.get(type_param))
        .is_some_and(|bounds| bounds.as_slice() == ["Structural"]);
    if !has_exact_bound {
        structural_type_error(
            ctx,
            format!("type parameter `{type_param}` must have the exact `Structural` bound"),
            span,
        );
    }

    let mut used = false;
    for param in params {
        match structural_type_position(&param.ty, type_param) {
            StructuralTypePosition::Absent => {}
            StructuralTypePosition::Direct => {
                used = true;
                if !param.convention.is_borrowed() || param.convention.is_mutable() {
                    structural_type_error(
                        ctx,
                        format!("structural parameter `{}` must be an immutable borrow", param.name),
                        span,
                    );
                }
            }
            StructuralTypePosition::CallScopedCallback => used = true,
            StructuralTypePosition::Nested => structural_type_error(
                ctx,
                format!(
                    "structural type parameter `{type_param}` occurs in an unsupported nested position for `{}`",
                    param.name
                ),
                span,
            ),
        }
    }
    match return_type.resolve_alias() {
        Type::Result(ok, _) => match structural_type_position(ok, type_param) {
            StructuralTypePosition::Absent => {}
            StructuralTypePosition::Direct => used = true,
            _ => structural_type_error(
                ctx,
                "a structural return must be the direct successful member of `Result`",
                span,
            ),
        },
        other if structural_type_position(other, type_param) != StructuralTypePosition::Absent => {
            structural_type_error(ctx, "a structural return must be inside `Result`", span);
        }
        _ => {}
    }
    if !used {
        structural_type_error(
            ctx,
            format!("structural type parameter `{type_param}` is not used by the bridge signature"),
            span,
        );
    }
}

fn validate_error_and_panic_contract(
    return_type: &Type,
    declarations: &[RustInteropDeclaration],
    span: TextRange,
    ctx: &mut LowerCtx,
) {
    let Type::Result(_, error) = return_type.resolve_alias() else {
        structural_type_error(
            ctx,
            "structural Rust bridge declarations must return `Result[T, OrdinaryError | RustPanicError]`",
            span,
        );
        return;
    };
    if !contains_named_error(error, "RustPanicError") {
        structural_type_error(
            ctx,
            "the error member must include `RustPanicError` so backend panics remain typed",
            span,
        );
    }
    if !contains_ordinary_error(error, &ctx.error_types) {
        structural_type_error(
            ctx,
            "the error member must include an ordinary error distinct from `RustPanicError`",
            span,
        );
    }

    let forbidden_policy = declarations
        .iter()
        .filter(|declaration| declaration.kind == RustInteropDecoratorKind::Function)
        .flat_map(|declaration| &declaration.arguments)
        .find_map(|argument| match (&argument.name, &argument.value) {
            (Some(name), RustInteropValue::Symbol(policy))
                if name == "panic" && matches!(policy.as_str(), "trusted_no_panic" | "abort") =>
            {
                Some(policy.as_str())
            }
            _ => None,
        });
    if let Some(policy) = forbidden_policy {
        structural_type_error(
            ctx,
            format!(
                "structural Rust bridge declarations require recoverable panic translation; `panic={policy}` is forbidden"
            ),
            span,
        );
    }
}

fn contains_named_error(ty: &Type, expected: &str) -> bool {
    match ty.resolve_alias() {
        Type::Class { name, .. } => name == expected,
        Type::Union(members) => members
            .iter()
            .any(|member| contains_named_error(member, expected)),
        _ => false,
    }
}

fn contains_ordinary_error(ty: &Type, error_types: &std::collections::HashSet<String>) -> bool {
    match ty.resolve_alias() {
        Type::Class { name, .. } => name != "RustPanicError" && error_types.contains(name),
        Type::Union(members) => members
            .iter()
            .any(|member| contains_ordinary_error(member, error_types)),
        _ => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StructuralTypePosition {
    Absent,
    Direct,
    CallScopedCallback,
    Nested,
}

fn structural_type_position(ty: &Type, type_param: &str) -> StructuralTypePosition {
    match ty.resolve_alias() {
        Type::TypeVar(name) if name == type_param => StructuralTypePosition::Direct,
        Type::Callable(params, _, result) => {
            callback_structural_position(params, result, type_param)
        }
        Type::AsyncCallable(params, _, result) => structural_nested_position(
            params.iter().chain(std::iter::once(result.as_ref())),
            type_param,
        ),
        Type::Function(function) | Type::AsyncFunction(function) => structural_nested_position(
            function
                .params
                .iter()
                .map(|(_, ty, _)| ty)
                .chain(std::iter::once(function.return_type.as_ref())),
            type_param,
        ),
        Type::Class {
            fields, type_args, ..
        } => structural_nested_position(
            fields
                .iter()
                .map(|(_, field)| field)
                .chain(type_args.iter()),
            type_param,
        ),
        Type::List(value)
        | Type::Set(value)
        | Type::Iterable(value)
        | Type::Iterator(value)
        | Type::Alias { body: value, .. }
        | Type::Newtype { inner: value, .. } => {
            structural_nested_position(std::iter::once(value.as_ref()), type_param)
        }
        Type::Dict(left, right) | Type::Result(left, right) => {
            structural_nested_position([left.as_ref(), right.as_ref()], type_param)
        }
        Type::Tuple(values) | Type::Union(values) | Type::Intersection(values) => {
            structural_nested_position(values.iter(), type_param)
        }
        _ => StructuralTypePosition::Absent,
    }
}

fn callback_structural_position(
    params: &[Type],
    result: &Type,
    type_param: &str,
) -> StructuralTypePosition {
    let parameter_positions = params
        .iter()
        .map(|value| structural_type_position(value, type_param))
        .collect::<Vec<_>>();
    let result_position = match result.resolve_alias() {
        Type::Result(ok, error) => {
            if structural_type_position(error, type_param) == StructuralTypePosition::Absent {
                structural_type_position(ok, type_param)
            } else {
                StructuralTypePosition::Nested
            }
        }
        other => structural_type_position(other, type_param),
    };
    let positions = parameter_positions
        .iter()
        .chain(std::iter::once(&result_position))
        .copied()
        .collect::<Vec<_>>();
    if positions
        .iter()
        .any(|position| *position != StructuralTypePosition::Absent)
        && positions.iter().all(|position| {
            matches!(
                position,
                StructuralTypePosition::Absent | StructuralTypePosition::Direct
            )
        })
    {
        StructuralTypePosition::CallScopedCallback
    } else if positions
        .iter()
        .any(|position| *position != StructuralTypePosition::Absent)
    {
        StructuralTypePosition::Nested
    } else {
        StructuralTypePosition::Absent
    }
}

fn structural_nested_position<'a>(
    values: impl IntoIterator<Item = &'a Type>,
    type_param: &str,
) -> StructuralTypePosition {
    if values
        .into_iter()
        .any(|value| structural_type_position(value, type_param) != StructuralTypePosition::Absent)
    {
        StructuralTypePosition::Nested
    } else {
        StructuralTypePosition::Absent
    }
}

fn structural_type_error(ctx: &mut LowerCtx, message: impl Into<String>, span: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::RUST_TYPE_PROBE_FAILURE,
        format!(
            "invalid structural Rust bridge contract: {}",
            message.into()
        ),
        span,
    );
}
