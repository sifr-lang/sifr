use super::{HirFunction, LowerCtx, MethodKind, StmtClassDef, Type};

pub(super) fn validate_rust_opaque_close_method(
    class_def: &StmtClassDef,
    methods: &[HirFunction],
    declarations: &[sifr_ir::RustInteropDeclaration],
    ctx: &mut LowerCtx,
) {
    let is_opaque = declarations
        .iter()
        .any(|declaration| declaration.kind == sifr_ir::RustInteropDecoratorKind::Opaque);
    let has_self_target = methods.iter().any(|method| {
        method.rust_interop.iter().any(|declaration| {
            declaration
                .target
                .as_ref()
                .and_then(|target| target.segments.first())
                .is_some_and(|root| root == "Self")
        })
    });
    if !is_opaque {
        if has_self_target {
            ctx.error_with_code_at(
                sifr_diagnostics::DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR,
                "Rust `Self.*` method targets require the owning class to declare `@rust.opaque(...)`"
                    .to_string(),
                class_def.range,
            );
        }
        return;
    }
    if methods.iter().any(|method| {
        method.method_kind != MethodKind::Regular
            && method.rust_interop.iter().any(|declaration| {
                declaration
                    .target
                    .as_ref()
                    .and_then(|target| target.segments.first())
                    .is_some_and(|root| root == "Self")
            })
    }) {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR,
            "Rust opaque `Self.*` targets require regular instance methods with a handle receiver"
                .to_string(),
            class_def.range,
        );
    }
    if methods.iter().any(|method| {
        method.method_kind == MethodKind::Regular
            && !method.rust_interop.is_empty()
            && !matches!(method.return_type.resolve_alias(), Type::Result(_, _))
    }) {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR,
            "Rust-bound opaque instance methods must return `Result[...]` so closed and poisoned handle states remain typed"
                .to_string(),
            class_def.range,
        );
    }
    if methods.iter().any(|method| {
        let uses_self_target = method.rust_interop.iter().any(|declaration| {
            declaration
                .target
                .as_ref()
                .and_then(|target| target.segments.first())
                .is_some_and(|root| root == "Self")
        });
        uses_self_target
            && matches!(
                method.return_type.resolve_alias(),
                Type::Result(_, error) if !opaque_self_state_error_is_representable(error, ctx)
            )
    }) {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR,
            "Rust opaque `Self.*` methods require a message-shaped Error result, optionally unioned with RustPanicError, so closed and poisoned handle states remain typed"
                .to_string(),
            class_def.range,
        );
    }
    let selected = sifr_ir::rust_opaque_close_method(declarations);
    let unmatched_consuming = methods.iter().any(|method| {
        method
            .rust_interop
            .iter()
            .any(|declaration| declaration.consumes_receiver)
            && selected != Some(method.name.as_str())
    });
    if unmatched_consuming {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR,
            "a consuming Rust opaque method is reserved for the member selected by the class close policy"
                .to_string(),
            class_def.range,
        );
    }
    let Some(selected) = selected else {
        return;
    };
    let expects_async = selected == "aclose";
    let valid_methods = methods
        .iter()
        .filter(|method| {
            method.name == selected
                && method.method_kind == MethodKind::Regular
                && method.is_async == expects_async
                && method.params.is_empty()
                && method.rust_interop.iter().any(|declaration| {
                    declaration.kind == sifr_ir::RustInteropDecoratorKind::Function
                        && declaration.target.is_some()
                        && declaration.consumes_receiver
                })
                && matches!(
                    method.return_type.resolve_alias(),
                    Type::Result(ok, _) if ok.resolve_alias() == &Type::None
                )
        })
        .count();
    if valid_methods != 1 {
        let policy = if expects_async {
            "async_close"
        } else {
            "close"
        };
        let async_prefix = if expects_async { "async " } else { "" };
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR,
            format!(
                "`close={policy}` requires exactly one Rust-bound `{async_prefix}def {selected}(own self) -> Result[None, Error]` method"
            ),
            class_def.range,
        );
    }
}

fn opaque_self_state_error_is_representable(error: &Type, ctx: &LowerCtx) -> bool {
    let resolved = match error.resolve_alias() {
        Type::Class { name, .. } => ctx.class_types.get(name).unwrap_or(error.resolve_alias()),
        other => other,
    };
    match resolved {
        Type::Class { name, fields, .. } => {
            name != "RustPanicError"
                && !resolved.is_python_error_contract()
                && fields
                    .iter()
                    .any(|(field, ty)| field == "message" && ty.resolve_alias() == &Type::Str)
                && fields
                    .iter()
                    .all(|(_, ty)| ty.resolve_alias() == &Type::Str)
        }
        Type::Union(members) => {
            let ordinary = members
                .iter()
                .filter(|member| {
                    !matches!(
                        member.resolve_alias(),
                        Type::Class { name, .. } if name == "RustPanicError"
                    )
                })
                .collect::<Vec<_>>();
            let panic_count = members
                .iter()
                .filter(|member| {
                    matches!(
                        member.resolve_alias(),
                        Type::Class { name, .. } if name == "RustPanicError"
                    )
                })
                .count();
            panic_count <= 1
                && members.len() == ordinary.len() + panic_count
                && ordinary.first().is_some_and(|ordinary_error| {
                    ordinary.len() == 1
                        && opaque_self_state_error_is_representable(ordinary_error, ctx)
                })
        }
        _ => false,
    }
}
