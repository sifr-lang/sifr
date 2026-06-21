use super::{canonical_sifr_target_path, canonical_trust_target_path, RustInteropResolver};
use sifr_codegen::{RustBridgeSignatureContract, RustBridgeTypeKind};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDeclaration, RustInteropDecoratorKind, RustInteropValue};
use sifr_package::SifrPackageMetadata;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PanicSurface {
    None,
    TrustedNoPanic,
    Abort,
    MapError,
    Invalid,
}

impl<'a> RustInteropResolver<'a> {
    pub(super) fn validate_panic_declaration(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        package: &SifrPackageMetadata,
    ) {
        if declaration.declaration.kind != RustInteropDecoratorKind::Function {
            return;
        }
        let surface = panic_surface(&declaration.declaration);
        if surface == PanicSurface::Invalid {
            self.push_panic_diagnostic(
                declaration,
                "unsupported panic policy; use `trusted_no_panic`, `abort`, or `map_error(path)`",
            );
            return;
        }
        if surface == PanicSurface::Abort {
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
                if result_carries_rust_panic_error(signature) || surface != PanicSurface::None {
                    return;
                }
                self.push_panic_diagnostic(
                    declaration,
                    "Result-returning Rust interop declarations must expose `RustPanicError`, use `panic=map_error(path)`, or declare an explicit trusted/abort panic policy",
                );
            }
            RustBridgeTypeKind::Unsupported => {}
            _ => match surface {
                PanicSurface::TrustedNoPanic | PanicSurface::Abort => {}
                PanicSurface::MapError => {
                    self.push_panic_diagnostic(
                            declaration,
                            "`panic=map_error(path)` requires a Result-returning Sifr declaration so mapped panic errors have a public error channel",
                        );
                }
                PanicSurface::None => {
                    self.push_panic_diagnostic(
                            declaration,
                            "non-Result Rust interop declarations must declare `panic=trusted_no_panic` or `panic=abort` because recoverable panics cannot be returned through the public type",
                        );
                }
                PanicSurface::Invalid => {}
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

fn panic_surface(declaration: &RustInteropDeclaration) -> PanicSurface {
    let mut surface = PanicSurface::None;
    for argument in declaration
        .arguments
        .iter()
        .filter(|argument| argument.name.as_deref() == Some("panic"))
    {
        let next = match &argument.value {
            RustInteropValue::Symbol(policy) if policy == "trusted_no_panic" => {
                PanicSurface::TrustedNoPanic
            }
            RustInteropValue::Symbol(policy) if policy == "abort" => PanicSurface::Abort,
            RustInteropValue::PolicyCall { name, argument, .. }
                if name == "map_error"
                    && matches!(argument.as_ref(), RustInteropValue::TargetPath(_)) =>
            {
                PanicSurface::MapError
            }
            _ => PanicSurface::Invalid,
        };
        if surface != PanicSurface::None || next == PanicSurface::Invalid {
            return PanicSurface::Invalid;
        }
        surface = next;
    }
    surface
}

fn result_carries_rust_panic_error(signature: &RustBridgeSignatureContract) -> bool {
    signature.return_type.sifr_type.contains("RustPanicError")
}

fn signature_has_unsupported_type(signature: &RustBridgeSignatureContract) -> bool {
    signature.return_type.kind == RustBridgeTypeKind::Unsupported
        || signature
            .params
            .iter()
            .any(|param| param.ty.kind == RustBridgeTypeKind::Unsupported)
}

fn selected_panic_strategy(package: &SifrPackageMetadata) -> Option<String> {
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
