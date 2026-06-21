use super::{panic_policy, RustInteropResolver};
use sifr_codegen::{RustBridgeSignatureContract, RustBridgeTypeKind};
use sifr_diagnostics::DiagnosticCode;

impl<'a> RustInteropResolver<'a> {
    pub(super) fn validate_direct_panic_policy(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        canonical_target_path: &str,
        signature: Option<&RustBridgeSignatureContract>,
    ) {
        let Some(signature) = signature else {
            return;
        };
        if matches!(
            signature.return_type.kind,
            RustBridgeTypeKind::Result | RustBridgeTypeKind::Unsupported
        ) {
            return;
        }
        if matches!(
            panic_policy(&declaration.declaration).as_deref(),
            Some("trusted_no_panic" | "abort")
        ) {
            return;
        }
        self.push_diagnostic(
            declaration,
            declaration.declaration.span,
            DiagnosticCode::RUST_TYPE_PROBE_FAILURE,
            "direct Rust binding `{target}` with non-Result return requires explicit panic policy",
            vec![("target", canonical_target_path.to_string())],
            vec![
                "direct Cargo bindings cannot recover Rust panics unless the public Sifr return type can carry `RustPanicError`".to_string(),
                "add `panic=trusted_no_panic` with matching `[trust].rust-no-panic` evidence, return `Result`, or use a package-local bridge".to_string(),
            ],
            None,
        );
    }
}
