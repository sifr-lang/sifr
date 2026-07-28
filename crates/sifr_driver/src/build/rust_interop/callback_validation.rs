use super::{canonical_sifr_target_path, RustInteropResolver};
use crate::build::rust_interop_callback_probe::signature_contract_has_call_scoped_callback;
use crate::build::rust_interop_trust::{effective_panic_policy, EffectivePanicPolicy};
use sifr_codegen::{
    RustBridgePanicErrorContract, RustBridgeParamConvention, RustBridgeSignatureContract,
    RustBridgeTypeKind,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropValue};
use std::collections::{BTreeMap, BTreeSet};

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

impl RustInteropResolver<'_> {
    pub(super) fn validate_callback_contracts(
        &mut self,
        declarations: &[sifr_codegen::RustInteropPlanDeclaration],
    ) {
        self.validate_call_scoped_callback_boundaries(declarations);

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

    fn validate_call_scoped_callback_boundaries(
        &mut self,
        declarations: &[sifr_codegen::RustInteropPlanDeclaration],
    ) {
        let async_targets = declarations
            .iter()
            .filter(|declaration| declaration.declaration.abi_requirements.async_boundary)
            .map(canonical_sifr_target_path)
            .collect::<BTreeSet<_>>();
        let call_scoped_targets = self
            .signature_contracts
            .iter()
            .filter(|(_, signature)| signature_contract_has_call_scoped_callback(signature))
            .map(|(target, _)| target.clone())
            .collect::<BTreeSet<_>>();
        let call_scoped_declarations = declarations
            .iter()
            .filter(|declaration| {
                call_scoped_targets.contains(&canonical_sifr_target_path(declaration))
            })
            .collect::<Vec<_>>();
        let call_scoped_packages = call_scoped_declarations
            .iter()
            .filter_map(|declaration| {
                self.package_id_for_module(declaration.module_name.as_deref())
            })
            .collect::<BTreeSet<_>>();
        let abort_profile_packages = call_scoped_packages
            .into_iter()
            .filter(|package_id| {
                self.context
                    .graph
                    .packages
                    .get(package_id)
                    .is_some_and(|package| {
                        super::panic_validation::selected_panic_strategy(package).as_deref()
                            == Some("abort")
                    })
            })
            .collect::<BTreeSet<_>>();
        let abort_targets = call_scoped_declarations
            .into_iter()
            .filter(|declaration| {
                self.declaration_uses_explicit_abort_policy(declaration)
                    || self
                        .package_id_for_module(declaration.module_name.as_deref())
                        .is_some_and(|package_id| abort_profile_packages.contains(&package_id))
            })
            .map(canonical_sifr_target_path)
            .collect::<BTreeSet<_>>();
        let mut validated_targets = BTreeSet::new();
        for declaration in declarations {
            let target = canonical_sifr_target_path(declaration);
            if !validated_targets.insert(target.clone()) {
                continue;
            }
            let Some(signature) = self.signature_contracts.get(&target) else {
                continue;
            };
            if !signature_contract_has_call_scoped_callback(signature) {
                continue;
            }
            let unsupported_reason = signature.params.iter().find_map(|param| {
                if param.ty.kind != RustBridgeTypeKind::CallScopedCallback {
                    return None;
                }
                match param.convention {
                    sifr_codegen::RustBridgeParamConvention::MutableBorrow => {
                        return Some(
                            "call-scoped callback parameters do not support mutable-borrow convention"
                                .to_string(),
                        );
                    }
                    sifr_codegen::RustBridgeParamConvention::OwnMutable => {
                        return Some(
                            "call-scoped callback parameters cannot be declared `mut`; remove `mut` from the callback parameter".to_string(),
                        );
                    }
                    sifr_codegen::RustBridgeParamConvention::Borrow
                    | sifr_codegen::RustBridgeParamConvention::Own => {}
                }
                param.ty.unsupported_reason.clone()
            });
            let panic_error = signature.panic_error;
            if let Some(reason) = unsupported_reason {
                self.push_call_scoped_callback_diagnostic(declaration, reason);
                continue;
            }
            if async_targets.contains(&target) {
                self.push_call_scoped_callback_diagnostic(
                    declaration,
                    "call-scoped callbacks require a synchronous Rust bridge call and cannot cross an async boundary",
                );
                continue;
            }
            if abort_targets.contains(&target) {
                self.push_call_scoped_callback_diagnostic(
                    declaration,
                    "call-scoped callbacks require an unwind-capable Cargo panic strategy; `panic=abort` and abort profiles cannot contain Sifr callback panics",
                );
                continue;
            }
            if panic_error != RustBridgePanicErrorContract::OrdinaryAndWrapper {
                self.push_call_scoped_callback_diagnostic(
                    declaration,
                    "call-scoped callbacks require a recoverable outer panic boundary with a distinct ordinary error and `RustPanicError` in the Result error channel",
                );
            }
        }
    }

    fn declaration_uses_explicit_abort_policy(
        &self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
    ) -> bool {
        let Some(package_id) = self.package_id_for_module(declaration.module_name.as_deref())
        else {
            return false;
        };
        let Some(package) = self.context.graph.packages.get(&package_id) else {
            return false;
        };
        effective_panic_policy(
            declaration,
            package,
            self.sysroot_trust_for_package(&package_id).is_some(),
        ) == EffectivePanicPolicy::Abort
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
            return;
        }
        let target = canonical_sifr_target_path(declaration);
        if is_python_callback_constructor_target(&target) {
            return;
        }
        let Some(signature) = self.signature_contracts.get(&target) else {
            return;
        };
        if let Err(reason) = validate_threadsafe_callback_signature(signature) {
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

    fn push_call_scoped_callback_diagnostic(
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
                "Keep the bridge call synchronous, or add an explicit `@rust.callback(...)` thread-safe lifecycle contract.".to_string(),
            ],
            Some(
                "A plain `Callable[...]` Rust parameter is borrowed for exactly one synchronous bridge call and cannot be retained across an await point.".to_string(),
            ),
        );
    }
}

fn is_python_callback_constructor_target(target: &str) -> bool {
    matches!(
        target,
        "_sifr.python.py_local_callback" | "_sifr.python.py_threadsafe_callback"
    )
}

fn validate_threadsafe_callback_signature(
    signature: &RustBridgeSignatureContract,
) -> Result<(), String> {
    let callback_params = signature
        .params
        .iter()
        .filter(|param| param.ty.kind == RustBridgeTypeKind::Callback)
        .collect::<Vec<_>>();
    if callback_params.is_empty() {
        return Err(
            "`@rust.callback(...)` requires at least one top-level `Callable[...]` parameter"
                .to_string(),
        );
    }
    for param in callback_params {
        if let Some(reason) = &param.ty.unsupported_reason {
            return Err(format!(
                "thread-safe callback parameter `{}` is unsupported: {reason}",
                param.name
            ));
        }
        match param.convention {
            RustBridgeParamConvention::Own => {}
            RustBridgeParamConvention::OwnMutable => {
                return Err(format!(
                    "thread-safe callback parameter `{}` cannot be declared `mut`; retained callbacks must be immutable and share-safe",
                    param.name
                ));
            }
            RustBridgeParamConvention::Borrow | RustBridgeParamConvention::MutableBorrow => {
                return Err(format!(
                    "thread-safe callback parameter `{}` must be declared `own` so the subscription owns its retained handler",
                    param.name
                ));
            }
        }
    }
    if signature.return_type.kind != RustBridgeTypeKind::Result
        || !signature
            .return_type
            .rust_return_type
            .as_deref()
            .is_some_and(|ty| ty.contains("::sifr_runtime::interop::Handle<"))
    {
        return Err(
            "thread-safe callback registration must return `Result[OpaqueSubscription, E]` with an explicit cleanup handle"
                .to_string(),
        );
    }
    if signature.panic_error != RustBridgePanicErrorContract::OrdinaryAndWrapper {
        return Err(
            "thread-safe callbacks require a recoverable outer panic boundary with a distinct ordinary error and `RustPanicError` in the Result error channel"
                .to_string(),
        );
    }
    Ok(())
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
