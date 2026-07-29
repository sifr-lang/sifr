use super::target_resolution::{trust_allowlist_name, trust_kind_name};
use super::{
    canonical_sifr_target_path, canonical_trust_target_path, declaration_paths,
    is_trusted_sysroot_package, unsafe_bridge_files, uses_bridge_root, RustInteropResolver,
};
use sifr_codegen::{
    RustInteropPlanDeclaration, RustInteropTrustRequirement, RustInteropTrustRequirementKind,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_package::{SifrPackageId, SifrPackageMetadata};
use std::collections::BTreeMap;

impl RustInteropResolver<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn require_trust(
        &mut self,
        declaration: &RustInteropPlanDeclaration,
        canonical_target_path: &str,
        kind: RustInteropTrustRequirementKind,
        trusted_entries: &[String],
        required_entry: &str,
        evidence: String,
        trusted_by_sysroot_policy: bool,
    ) {
        let key = (
            canonical_target_path.to_string(),
            trust_kind_name(&kind).to_string(),
            required_entry.to_string(),
        );
        if !self.seen_trust_requirements.insert(key) {
            return;
        }
        let allowlist_name = trust_allowlist_name(&kind);
        let trusted = trusted_by_sysroot_policy
            || trusted_entries.iter().any(|entry| entry == required_entry);
        self.trust_requirements.push(RustInteropTrustRequirement {
            canonical_target_path: canonical_target_path.to_string(),
            kind,
            trusted,
            required_entry: required_entry.to_string(),
            evidence: evidence.clone(),
        });
        if trusted {
            return;
        }
        self.push_diagnostic(
            declaration,
            declaration.declaration.span,
            DiagnosticCode::RUST_TRUST_MISSING,
            "missing Rust interop trust declaration for `{target}`",
            vec![
                ("target", canonical_target_path.to_string()),
                ("required_trust", required_entry.to_string()),
                ("evidence", evidence),
            ],
            vec![format!(
                "add `{required_entry}` to `[trust].{allowlist_name}` before Cargo executes this dependency"
            )],
            None,
        );
    }

    pub(super) fn validate_package_dependency_trust(
        &mut self,
        declarations: &[RustInteropPlanDeclaration],
    ) {
        let mut package_declarations =
            BTreeMap::<SifrPackageId, Vec<RustInteropPlanDeclaration>>::new();
        for declaration in declarations {
            let Some(package_id) = self.package_id_for_module(declaration.module_name.as_deref())
            else {
                continue;
            };
            package_declarations
                .entry(package_id)
                .or_default()
                .push(declaration.clone());
        }

        for (package_id, declarations) in package_declarations {
            let Some(package) = self.context.graph.packages.get(&package_id).cloned() else {
                continue;
            };
            let backends = self
                .context
                .graph
                .backend_crates
                .get(&package_id)
                .cloned()
                .unwrap_or_default();
            let trusted_by_sysroot_policy =
                is_trusted_sysroot_package(self.context, &package.package_id);
            for backend in backends {
                let Some(declaration) = declarations
                    .iter()
                    .find(|declaration| {
                        declaration_paths(&declaration.declaration)
                            .iter()
                            .any(|path| {
                                path.segments
                                    .first()
                                    .is_some_and(|root| root == &backend.dependency_name)
                            })
                    })
                    .or_else(|| declarations.first())
                else {
                    continue;
                };
                let canonical_target_path = canonical_sifr_target_path(declaration);
                self.validate_backend_trust(
                    declaration,
                    &canonical_target_path,
                    &package,
                    &backend,
                    trusted_by_sysroot_policy,
                );
            }
        }
    }

    pub(super) fn record_declared_native_links(
        &mut self,
        declaration: &RustInteropPlanDeclaration,
        package: &SifrPackageMetadata,
    ) {
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
                    "manifest-declared transitive native link `{native_link}` for Rust interop package"
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
