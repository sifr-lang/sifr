use super::{canonical_sifr_target_path, RustInteropResolver};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropValue};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallbackBackpressure {
    Direct,
    Bounded(i64),
    Unbounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallbackOverflow {
    Error,
    DropOldest,
    DropNewest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallbackShutdown {
    Drain,
    Cancel,
    DetachForbidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CallbackContract {
    backpressure: CallbackBackpressure,
    overflow: CallbackOverflow,
    shutdown: CallbackShutdown,
}

impl<'a> RustInteropResolver<'a> {
    pub(super) fn validate_callback_contracts(
        &mut self,
        declarations: &[sifr_codegen::RustInteropPlanDeclaration],
    ) {
        let mut by_target: BTreeMap<String, Vec<&sifr_codegen::RustInteropPlanDeclaration>> =
            BTreeMap::new();
        for declaration in declarations {
            if matches!(
                declaration.declaration.kind,
                RustInteropDecoratorKind::Function | RustInteropDecoratorKind::Callback
            ) {
                by_target
                    .entry(canonical_sifr_target_path(declaration))
                    .or_default()
                    .push(declaration);
            }
        }

        for declarations in by_target.values() {
            self.validate_callback_group(declarations);
        }
    }

    fn validate_callback_group(
        &mut self,
        declarations: &[&sifr_codegen::RustInteropPlanDeclaration],
    ) {
        let callback_declarations = declarations
            .iter()
            .filter(|declaration| {
                declaration.declaration.kind == RustInteropDecoratorKind::Callback
            })
            .copied()
            .collect::<Vec<_>>();
        if callback_declarations.is_empty() {
            return;
        }
        if !declarations
            .iter()
            .any(|declaration| declaration.declaration.kind == RustInteropDecoratorKind::Function)
        {
            for declaration in callback_declarations {
                self.push_callback_diagnostic(
                    declaration,
                    "`@rust.callback(...)` must accompany a `@rust(...)` target declaration",
                );
            }
            return;
        }
        if callback_declarations.len() > 1 {
            for declaration in callback_declarations {
                self.push_callback_diagnostic(
                    declaration,
                    "only one `@rust.callback(...)` contract is allowed per Rust interop declaration",
                );
            }
            return;
        }

        let declaration = callback_declarations[0];
        if let Err(reason) = parse_callback_contract(declaration) {
            self.push_callback_diagnostic(declaration, reason);
        }
    }

    fn push_callback_diagnostic(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        reason: impl Into<String>,
    ) {
        self.push_diagnostic(
            declaration,
            declaration.declaration.span,
            DiagnosticCode::RUST_CALLBACK_CONTRACT,
            "invalid Rust callback contract for `{target}`: {reason}",
            vec![
                ("target", canonical_sifr_target_path(declaration)),
                ("reason", reason.into()),
            ],
            vec![
                "`@rust.callback(...)` requires explicit backpressure, overflow, and shutdown policy"
                    .to_string(),
            ],
            Some(
                "Thread-safe Rust callbacks may outlive the bridge call or cross threads, so their queueing and shutdown behavior must be explicit.".to_string(),
            ),
        );
    }
}

fn parse_callback_contract(
    declaration: &sifr_codegen::RustInteropPlanDeclaration,
) -> Result<CallbackContract, String> {
    let mut backpressure = None;
    let mut overflow = None;
    let mut shutdown = None;

    for argument in &declaration.declaration.arguments {
        let Some(name) = argument.name.as_deref() else {
            return Err("`@rust.callback(...)` requires named arguments".to_string());
        };
        match name {
            "backpressure" => {
                if backpressure.is_some() {
                    return Err("duplicate `backpressure=` policy".to_string());
                }
                backpressure = Some(parse_backpressure(&argument.value)?);
            }
            "overflow" => {
                if overflow.is_some() {
                    return Err("duplicate `overflow=` policy".to_string());
                }
                overflow = Some(parse_overflow(&argument.value)?);
            }
            "shutdown" => {
                if shutdown.is_some() {
                    return Err("duplicate `shutdown=` policy".to_string());
                }
                shutdown = Some(parse_shutdown(&argument.value)?);
            }
            other => return Err(format!("unsupported `@rust.callback(...)` key `{other}`")),
        }
    }

    Ok(CallbackContract {
        backpressure: backpressure
            .ok_or_else(|| "missing required `backpressure=` policy".to_string())?,
        overflow: overflow.ok_or_else(|| "missing required `overflow=` policy".to_string())?,
        shutdown: shutdown.ok_or_else(|| "missing required `shutdown=` policy".to_string())?,
    })
}

fn parse_backpressure(value: &RustInteropValue) -> Result<CallbackBackpressure, String> {
    match value {
        RustInteropValue::Symbol(symbol) if symbol == "direct" => Ok(CallbackBackpressure::Direct),
        RustInteropValue::Symbol(symbol) if symbol == "unbounded" => {
            Ok(CallbackBackpressure::Unbounded)
        }
        RustInteropValue::PolicyCall { name, argument, .. } if name == "bounded" => {
            let RustInteropValue::Integer(bound) = argument.as_ref() else {
                return Err("`backpressure=bounded(...)` requires an integer bound".to_string());
            };
            if *bound <= 0 {
                return Err("`backpressure=bounded(...)` requires a positive bound".to_string());
            }
            Ok(CallbackBackpressure::Bounded(*bound))
        }
        _ => Err("`backpressure=` must be direct, unbounded, or bounded(N)".to_string()),
    }
}

fn parse_overflow(value: &RustInteropValue) -> Result<CallbackOverflow, String> {
    let RustInteropValue::Symbol(symbol) = value else {
        return Err("`overflow=` must be error, drop_oldest, or drop_newest".to_string());
    };
    match symbol.as_str() {
        "error" => Ok(CallbackOverflow::Error),
        "drop_oldest" => Ok(CallbackOverflow::DropOldest),
        "drop_newest" => Ok(CallbackOverflow::DropNewest),
        _ => Err("`overflow=` must be error, drop_oldest, or drop_newest".to_string()),
    }
}

fn parse_shutdown(value: &RustInteropValue) -> Result<CallbackShutdown, String> {
    let RustInteropValue::Symbol(symbol) = value else {
        return Err("`shutdown=` must be drain, cancel, or detach_forbidden".to_string());
    };
    match symbol.as_str() {
        "drain" => Ok(CallbackShutdown::Drain),
        "cancel" => Ok(CallbackShutdown::Cancel),
        "detach_forbidden" => Ok(CallbackShutdown::DetachForbidden),
        _ => Err("`shutdown=` must be drain, cancel, or detach_forbidden".to_string()),
    }
}
