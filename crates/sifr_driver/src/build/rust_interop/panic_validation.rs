use super::{RustInteropResolver, canonical_sifr_target_path, canonical_trust_target_path};
use crate::build::rust_interop_trust::{EffectivePanicPolicy, effective_panic_policy};
use sifr_codegen::{RustBridgePanicErrorContract, RustBridgeSignatureContract, RustBridgeTypeKind};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropEffect};
use sifr_package::SifrPackageMetadata;

impl RustInteropResolver<'_> {
    pub(super) fn validate_panic_declaration(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        package: &SifrPackageMetadata,
    ) {
        if declaration.declaration.kind != RustInteropDecoratorKind::Function {
            return;
        }
        let surface = effective_panic_policy(
            declaration,
            package,
            self.sysroot_trust_for_package(&package.package_id)
                .is_some(),
        );
        if surface == EffectivePanicPolicy::Invalid {
            self.push_panic_diagnostic(
                declaration,
                "unsupported panic policy; use `trusted_no_panic`, `abort`, or `map_error(path)`",
            );
            return;
        }
        if surface == EffectivePanicPolicy::InvalidSysrootImplicitTarget {
            self.push_panic_diagnostic(
                declaration,
                "sysroot no-panic policy applies only to canonical private `sifr_stdlib.*` targets; declare an explicit panic policy for other Rust interop targets",
            );
            return;
        }
        if surface.is_abort() {
            self.validate_abort_panic_strategy(declaration, package);
            if !self.diagnostics.is_empty() {
                return;
            }
        }

        let Some(signature) = self
            .signature_contracts
            .get(&canonical_sifr_target_path(declaration))
        else {
            return;
        };
        if signature_has_unsupported_type(signature) {
            return;
        }

        match signature.return_type.kind {
            RustBridgeTypeKind::Result => {
                if signature.panic_error == RustBridgePanicErrorContract::WrapperOnly {
                    self.push_panic_diagnostic(
                        declaration,
                        "`RustPanicError` is reserved for generated wrapper failures; a fallible Rust target also requires a distinct ordinary error member",
                    );
                    return;
                }
                if surface == EffectivePanicPolicy::MapError {
                    if declaration.declaration.effect == RustInteropEffect::Async {
                        self.push_panic_diagnostic(
                            declaration,
                            "async `panic=map_error(path)` requires async panic-wrapper certification and is not supported by the synchronous wrapper contract",
                        );
                        return;
                    }
                    if signature.panic_error != RustBridgePanicErrorContract::OrdinaryAndWrapper {
                        self.push_panic_diagnostic(
                            declaration,
                            "`panic=map_error(path)` requires a mapped error member plus `RustPanicError` in the Result error channel so mapper panics have a representable redacted fallback",
                        );
                    }
                    return;
                }
                if signature.panic_error == RustBridgePanicErrorContract::OrdinaryAndWrapper
                    || surface.is_some()
                {
                    return;
                }
                self.push_panic_diagnostic(
                    declaration,
                    "Result-returning Rust interop declarations must expose `RustPanicError`, use `panic=map_error(path)`, or declare an explicit trusted/abort panic policy",
                );
            }
            RustBridgeTypeKind::Unsupported => {}
            _ => match surface {
                EffectivePanicPolicy::TrustedNoPanic | EffectivePanicPolicy::Abort => {}
                EffectivePanicPolicy::MapError => {
                    self.push_panic_diagnostic(
                        declaration,
                        "`panic=map_error(path)` requires a Result-returning Sifr declaration so mapped panic errors have a public error channel",
                    );
                }
                EffectivePanicPolicy::None => {
                    self.push_panic_diagnostic(
                            declaration,
                            "non-Result Rust interop declarations must declare `panic=trusted_no_panic` or `panic=abort` because recoverable panics cannot be returned through the public type",
                        );
                }
                EffectivePanicPolicy::Invalid
                | EffectivePanicPolicy::InvalidSysrootImplicitTarget => {}
            },
        }
    }

    fn push_panic_diagnostic(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        reason: &'static str,
    ) {
        self.push_diagnostic(
            declaration,
            declaration.declaration.span,
            DiagnosticCode::RUST_PANIC_CONTRACT,
            "invalid Rust panic contract: {reason}",
            vec![("reason", reason.to_string())],
            vec![
                "Rust bridge panics must either be recoverable through the declared Sifr error channel or explicitly trusted/abort-only through package trust policy".to_string(),
            ],
            None,
        );
    }

    fn validate_abort_panic_strategy(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        package: &SifrPackageMetadata,
    ) {
        let target = canonical_trust_target_path(declaration);
        if !package
            .manifest
            .trust
            .rust_panic_abort
            .iter()
            .any(|entry| entry == &target)
        {
            return;
        }
        if selected_panic_strategy(package).as_deref() == Some("abort") {
            return;
        }
        self.push_panic_diagnostic(
            declaration,
            "`panic=abort` requires the selected Cargo panic strategy to be `abort`",
        );
    }
}

fn signature_has_unsupported_type(signature: &RustBridgeSignatureContract) -> bool {
    signature.return_type.kind == RustBridgeTypeKind::Unsupported
        || signature
            .params
            .iter()
            .any(|param| param.ty.kind == RustBridgeTypeKind::Unsupported)
}

pub(super) fn selected_panic_strategy(package: &SifrPackageMetadata) -> Option<String> {
    // Rust interop bridge builds currently select the release Cargo profile.
    panic_strategy_from_profile(&package.package_root, "release")
        .or_else(|| std::env::var("SIFR_RUST_PANIC_STRATEGY").ok())
}

fn panic_strategy_from_profile(package_root: &std::path::Path, profile: &str) -> Option<String> {
    for cargo_toml in package_root
        .ancestors()
        .map(|ancestor| ancestor.join("Cargo.toml"))
        .filter(|candidate| candidate.is_file())
    {
        let Ok(source) = std::fs::read_to_string(cargo_toml) else {
            continue;
        };
        let Ok(table) = source.parse::<toml::Table>() else {
            continue;
        };
        if let Some(value) = table
            .get("profile")
            .and_then(toml::Value::as_table)
            .and_then(|profiles| profiles.get(profile))
            .and_then(toml::Value::as_table)
            .and_then(|profile| profile.get("panic"))
            .and_then(toml::Value::as_str)
        {
            return Some(value.to_string());
        }
    }
    None
}
