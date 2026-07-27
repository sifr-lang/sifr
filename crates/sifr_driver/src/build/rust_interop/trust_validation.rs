use super::{
    canonical_sifr_target_path, canonical_trust_target_path, is_trusted_sysroot_package,
    trust_kind_name, unsafe_bridge_files, uses_bridge_root, RustInteropResolver,
};
use sifr_codegen::{
    RustInteropPlanDeclaration, RustInteropTrustRequirement, RustInteropTrustRequirementKind,
};
use sifr_package::SifrPackageMetadata;

impl RustInteropResolver<'_> {
    pub(super) fn record_declared_bridge_native_links(
        &mut self,
        declaration: &RustInteropPlanDeclaration,
        package: &SifrPackageMetadata,
    ) {
        if !uses_bridge_root(&declaration.declaration) {
            return;
        }
        let canonical_target_path = canonical_trust_target_path(declaration);
        for native_link in &package.manifest.trust.native_links {
            let key = (
                canonical_target_path.clone(),
                trust_kind_name(&RustInteropTrustRequirementKind::NativeLinks).to_string(),
                native_link.clone(),
            );
            if !self.seen_trust_requirements.insert(key) {
                continue;
            }
            self.trust_requirements.push(RustInteropTrustRequirement {
                canonical_target_path: canonical_target_path.clone(),
                kind: RustInteropTrustRequirementKind::NativeLinks,
                trusted: true,
                required_entry: native_link.clone(),
                evidence: format!(
                    "manifest-declared transitive native link `{native_link}` for package-local Rust bridge"
                ),
            });
        }
    }

    pub(super) fn validate_unsafe_bridge_files(
        &mut self,
        declaration: &RustInteropPlanDeclaration,
        package: &SifrPackageMetadata,
    ) {
        if !uses_bridge_root(&declaration.declaration) {
            return;
        }
        for bridge_file in unsafe_bridge_files(package) {
            self.require_trust(
                declaration,
                &canonical_sifr_target_path(declaration),
                RustInteropTrustRequirementKind::UnsafeBridge,
                &package.manifest.trust.unsafe_rust_bridges,
                &bridge_file,
                format!("unsafe package-local Rust bridge file `{bridge_file}`"),
                is_trusted_sysroot_package(self.context, &package.package_id),
            );
        }
    }
}
