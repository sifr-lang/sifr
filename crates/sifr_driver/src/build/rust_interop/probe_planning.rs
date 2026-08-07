use super::super::rust_interop_probe::PendingRustBridgeProbe;
use super::bridge_aliases::package_bridge_dependency_name;
use super::RustInteropResolver;
use crate::build::sysroot_interop::SysrootRustInteropTrust;
use sifr_codegen::{RustBridgeProbeKind, RustInteropOwner, RustInteropPlanDeclaration};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDeclaration, RustInteropDecoratorKind, RustTargetPath};
use sifr_package::{BackendCrateMetadata, SifrPackageMetadata};

pub(super) fn probe_kind(
    declaration: &RustInteropDeclaration,
    owner: &RustInteropOwner,
) -> Option<RustBridgeProbeKind> {
    match declaration.kind {
        RustInteropDecoratorKind::Function => {
            if declaration.abi_requirements.async_boundary {
                Some(RustBridgeProbeKind::AsyncFunction)
            } else if matches!(owner, RustInteropOwner::Method { .. }) {
                Some(RustBridgeProbeKind::Method)
            } else if matches!(owner, RustInteropOwner::Function { .. }) {
                Some(RustBridgeProbeKind::Function)
            } else {
                None
            }
        }
        RustInteropDecoratorKind::Opaque => matches!(owner, RustInteropOwner::Class { .. })
            .then_some(RustBridgeProbeKind::OpaqueHandle),
        RustInteropDecoratorKind::Async => matches!(
            owner,
            RustInteropOwner::Function { .. } | RustInteropOwner::Method { .. }
        )
        .then_some(RustBridgeProbeKind::AsyncFunction),
        RustInteropDecoratorKind::Callback => None,
        RustInteropDecoratorKind::ZeroCopy => matches!(
            owner,
            RustInteropOwner::Function { .. } | RustInteropOwner::Method { .. }
        )
        .then_some(RustBridgeProbeKind::ZeroCopy),
        RustInteropDecoratorKind::View => matches!(
            owner,
            RustInteropOwner::Function { .. } | RustInteropOwner::Method { .. }
        )
        .then_some(RustBridgeProbeKind::View),
    }
}

impl RustInteropResolver<'_> {
    pub(super) fn plan_package_bridge_probe(
        &mut self,
        declaration: &RustInteropPlanDeclaration,
        path: &RustTargetPath,
        package: &SifrPackageMetadata,
        sysroot_trust: Option<&SysrootRustInteropTrust>,
    ) -> Option<String> {
        let dependency_name = package_bridge_dependency_name(package);
        let signature = super::target_resolution::is_primary_target(&declaration.declaration, path)
            .then(|| {
                self.signature_contracts
                    .get(&super::target_resolution::canonical_sifr_target_path(
                        declaration,
                    ))
                    .cloned()
            })
            .flatten();
        let async_thread_affinity = self.async_thread_affinity_for_probe(declaration);
        let Some(sysroot_runtime_crate) = self.context.sysroot_runtime_crate.clone() else {
            self.push_diagnostic(
                declaration,
                path.span,
                DiagnosticCode::RUST_CARGO_METADATA,
                "Rust bridge probe requires a resolved Sifr sysroot runtime crate",
                vec![("target", path.dotted())],
                vec![
                    "Package-local Rust bridge probes must use the same resolved sysroot runtime crate as generated Cargo projects.".to_string(),
                ],
                None,
            );
            return None;
        };
        self.pending_direct_probes.push(PendingRustBridgeProbe {
            declaration: declaration.clone(),
            path: path.clone(),
            backend: BackendCrateMetadata {
                cargo_package_id: package.cargo_package_id.clone(),
                dependency_name: dependency_name.clone(),
                dependency_kind: None,
                cargo_package_name: package.cargo_package_name.clone(),
                cargo_version: package.cargo_version.clone(),
                cargo_source: package.cargo_source.clone(),
                cargo_manifest_path: package.package_root.join("Cargo.toml"),
                links: None,
                has_build_script: false,
                has_proc_macro: false,
            },
            source_prefix: Some(format!("use {dependency_name}::bridges as bridge;")),
            signature,
            async_thread_affinity,
            zero_copy_obligations: self
                .zero_copy_probe_obligations
                .get(&super::target_resolution::canonical_sifr_target_path(
                    declaration,
                ))
                .copied()
                .unwrap_or((false, false)),
            trusted_sysroot: sysroot_trust.is_some(),
            sysroot_runtime_crate,
            sysroot_vendor_dir: sysroot_trust.map(|trust| trust.vendor_dir.clone()),
            cargo_resolution: self.cargo_resolution.clone(),
        });
        Some(dependency_name)
    }
}
