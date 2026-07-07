use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop_bridge_audit::unsafe_bridge_files;
use super::rust_interop_cargo_inputs::{
    bridge_source_digests, cargo_inputs, combined_cargo_inputs, first_generated_bridge_import,
    generated_bridge_module_path,
};
use super::rust_interop_contracts::bridge_contract_diagnostics;
use super::rust_interop_diagnostics::{render_template, source_diagnostic};
use super::rust_interop_digest::normalized_path_string;
use super::rust_interop_probe::{
    execute_direct_cargo_probe, AsyncThreadAffinity, PendingRustBridgeProbe,
};
use super::rust_interop_trust::{
    build_env_trust_entries, effective_panic_policy, EffectivePanicPolicy,
};
use super::sysroot_interop::{
    is_trusted_sysroot_package, resolved_sysroot_crate_root, sysroot_crate_for_dependency_name,
    SysrootRustInteropTrust,
};
use crate::diagnostics::{diagnostic_with_code, RenderedDiagnostic};
use crate::project::ParsedProjectModule;
use opaque_contract::OpaqueContract;
use opaque_validation::opaque_probe_obligations;
use ruff_text_size::TextRange;
use sifr_codegen::{
    RustBridgeProbe, RustBridgeProbePlan, RustBridgeSignatureContract, RustGeneratedBridgeModule,
    RustInteropOwner, RustInteropResolvedRoot, RustInteropResolvedTarget,
    RustInteropTrustRequirement, RustInteropTrustRequirementKind,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustTargetPath};
use sifr_package::{BackendCrateMetadata, PackageSourceMap, SifrPackageGraph, SifrPackageId};
use std::collections::{BTreeSet, HashMap};

#[path = "rust_interop/advanced_data_validation.rs"]
mod advanced_data_validation;
#[path = "rust_interop/async_validation.rs"]
mod async_validation;
#[path = "rust_interop/bridge_aliases.rs"]
mod bridge_aliases;
#[path = "rust_interop/callback_validation.rs"]
mod callback_validation;
#[path = "rust_interop/opaque_contract.rs"]
mod opaque_contract;
#[path = "rust_interop/opaque_validation.rs"]
mod opaque_validation;
#[path = "rust_interop/panic_validation.rs"]
mod panic_validation;
#[path = "rust_interop/probe_planning.rs"]
mod probe_planning;
#[path = "rust_interop/target_resolution.rs"]
mod target_resolution;
#[path = "rust_interop/zero_copy_validation.rs"]
mod zero_copy_validation;

use bridge_aliases::{inject_package_bridge_aliases, package_bridge_dependency_name};
use target_resolution::{
    backend_for_root, canonical_sifr_target_path, canonical_trust_target_path, declaration_paths,
    trust_kind_name, uses_bridge_root,
};

#[derive(Clone, Debug)]
pub(super) struct PackageRustInteropContext {
    pub(super) package_id: SifrPackageId,
    pub(super) graph: SifrPackageGraph,
    pub(super) source_map: PackageSourceMap,
    pub(super) module_packages: HashMap<String, SifrPackageId>,
    pub(super) module_sources: HashMap<String, RustInteropModuleSource>,
    pub(super) sysroot_runtime_crate: Option<std::path::PathBuf>,
    pub(super) sysroot_trust: Option<SysrootRustInteropTrust>,
}

#[derive(Clone, Debug)]
pub(super) struct RustInteropModuleSource {
    pub(super) source: String,
    pub(super) display_path: String,
}

impl RustInteropModuleSource {
    pub(super) fn from_parsed(parsed: &ParsedProjectModule) -> Self {
        Self {
            source: parsed.source.clone(),
            display_path: parsed.display_path.clone(),
        }
    }
}

pub(super) fn apply_package_rust_interop_metadata(
    mut generated: GeneratedBinaryProject,
    context: Option<PackageRustInteropContext>,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    if generated.interop.rust.declarations.is_empty() {
        return Ok(generated);
    }

    let Some(context) = context else {
        return Err(vec![crate::diagnostics::diagnostic_with_code(
            "Rust interop declarations require a Sifr package Cargo context",
            DiagnosticCode::RUST_CARGO_METADATA,
        )]);
    };

    let mut resolver = RustInteropResolver::new(&context);
    resolver.resolve_plan(&mut generated)?;
    Ok(generated)
}

struct RustInteropResolver<'a> {
    context: &'a PackageRustInteropContext,
    diagnostics: Vec<RenderedDiagnostic>,
    resolved_targets: Vec<RustInteropResolvedTarget>,
    generated_bridge_modules: BTreeSet<RustGeneratedBridgeModule>,
    trust_requirements: Vec<RustInteropTrustRequirement>,
    seen_trust_requirements: BTreeSet<(String, String, String)>,
    probes: Vec<RustBridgeProbe>,
    pending_direct_probes: Vec<PendingRustBridgeProbe>,
    signature_contracts: HashMap<String, RustBridgeSignatureContract>,
    opaque_contracts: HashMap<String, OpaqueContract>,
    async_contracts: HashMap<String, AsyncThreadAffinity>,
}

impl<'a> RustInteropResolver<'a> {
    fn new(context: &'a PackageRustInteropContext) -> Self {
        Self {
            context,
            diagnostics: Vec::new(),
            resolved_targets: Vec::new(),
            generated_bridge_modules: BTreeSet::new(),
            trust_requirements: Vec::new(),
            seen_trust_requirements: BTreeSet::new(),
            probes: Vec::new(),
            pending_direct_probes: Vec::new(),
            signature_contracts: HashMap::new(),
            opaque_contracts: HashMap::new(),
            async_contracts: HashMap::new(),
        }
    }

    fn resolve_plan(
        &mut self,
        generated: &mut GeneratedBinaryProject,
    ) -> Result<(), Vec<RenderedDiagnostic>> {
        self.signature_contracts = generated
            .interop
            .rust
            .bridge_contracts
            .signatures
            .iter()
            .cloned()
            .map(|signature| (signature.canonical_target_path.clone(), signature))
            .collect();
        self.collect_async_contracts(&generated.interop.rust.declarations);
        self.validate_callback_contracts(&generated.interop.rust.declarations);
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }
        self.validate_zero_copy_contracts(&generated.interop.rust.declarations);
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }
        self.validate_advanced_data_contracts(&generated.interop.rust.declarations);
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }
        for declaration in generated.interop.rust.declarations.clone() {
            self.resolve_declaration(&declaration);
        }

        self.validate_opaque_close_contracts(&generated.interop.rust.declarations);
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }
        self.validate_bridge_contracts(&generated.interop.rust.bridge_contracts.signatures);
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }
        self.execute_pending_direct_probes();
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let Some(package) = self.context.graph.packages.get(&self.context.package_id) else {
            return Err(vec![diagnostic_with_code(
                "internal compiler error: package Rust interop context references a missing graph package",
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };
        generated.interop.rust.resolved_targets = std::mem::take(&mut self.resolved_targets);
        generated.interop.rust.generated_bridge_modules =
            std::mem::take(&mut self.generated_bridge_modules)
                .into_iter()
                .collect();
        generated.interop.rust.trust_requirements = std::mem::take(&mut self.trust_requirements);
        generated.interop.rust.probe_plan = RustBridgeProbePlan {
            probes: std::mem::take(&mut self.probes),
        };
        generated.interop.rust.bridge_sources = bridge_source_digests(self.context, package);
        let mut cargo_input = cargo_inputs(self.context, package);
        if let Some(trust) = &self.context.sysroot_trust {
            if trust.package_id != self.context.package_id {
                if let Some(sysroot_package) = self.context.graph.packages.get(&trust.package_id) {
                    cargo_input = combined_cargo_inputs(
                        cargo_input,
                        cargo_inputs(self.context, sysroot_package),
                    );
                }
            }
        }
        generated.interop.rust.cargo_inputs = Some(cargo_input);
        inject_package_bridge_aliases(generated);
        Ok(())
    }

    fn resolve_declaration(&mut self, declaration: &sifr_codegen::RustInteropPlanDeclaration) {
        let module_name = declaration.module_name.as_deref();
        let Some(package_id) = self.package_id_for_module(module_name) else {
            self.push_diagnostic(
                declaration,
                declaration.declaration.span,
                DiagnosticCode::RUST_CARGO_METADATA,
                "Rust interop module has no package context",
                vec![("module", module_name.unwrap_or("<single>").to_string())],
                vec!["compile this file through `sifr run` or `sifr build` from a package".to_string()],
                Some("Rust interop requires package Cargo metadata so target roots, trust gates, and cache inputs can be resolved.".to_string()),
            );
            return;
        };
        if !self.validate_private_declaration_context(declaration, module_name, &package_id) {
            return;
        }
        let Some(package) = self.context.graph.packages.get(&package_id) else {
            self.push_diagnostic(
                declaration,
                declaration.declaration.span,
                DiagnosticCode::RUST_CARGO_METADATA,
                "Rust interop package metadata is missing",
                vec![("package_id", package_id.0.clone())],
                Vec::new(),
                None,
            );
            return;
        };
        if uses_bridge_root(&declaration.declaration) {
            if !self.validate_bridge_version(declaration, package) {
                return;
            }
            self.push_generated_bridge_module(declaration, package);
        }
        if declaration.declaration.kind == RustInteropDecoratorKind::Opaque
            && !self.validate_opaque_declaration(declaration)
        {
            return;
        }
        if !self.validate_async_declaration(declaration) {
            return;
        }
        if declaration.declaration.kind == RustInteropDecoratorKind::Callback {
            return;
        }
        self.validate_panic_declaration(declaration, package);
        if !self.diagnostics.is_empty() {
            return;
        }

        for path in declaration_paths(&declaration.declaration) {
            self.resolve_path(declaration, &package_id, package, path);
        }
        self.validate_declaration_trust(declaration, package);
        self.validate_unsafe_bridge_files(declaration, package);
        self.push_probe(declaration);
    }

    fn resolve_path(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        package_id: &SifrPackageId,
        package: &sifr_package::SifrPackageMetadata,
        path: &RustTargetPath,
    ) {
        let Some(root) = path.segments.first() else {
            return;
        };
        let sysroot_trust = self.sysroot_trust_for_package(package_id).cloned();
        if sysroot_trust.is_some() && sysroot_crate_for_dependency_name(root).is_none() {
            self.push_diagnostic(
                    declaration,
                    path.span,
                    DiagnosticCode::RUST_RESOLVE_TARGET_ROOT,
                    "private stdlib Rust interop target must use canonical sysroot crate `{root}`",
                    vec![("root", root.clone()), ("target", path.dotted())],
                    vec!["allowed sysroot crates: sifr_runtime, sifr_stdlib".to_string()],
                    Some("Private _sifr declarations may target only Sifr-owned crates in the resolved sysroot.".to_string()),
                );
            return;
        }
        let canonical_target_path = canonical_sifr_target_path(declaration);
        let resolved_root = match root.as_str() {
            "bridge" => RustInteropResolvedRoot::PackageBridge {
                package_id: package_id.0.clone(),
                dependency_name: package_bridge_dependency_name(package),
                cargo_package_name: package.cargo_package_name.clone(),
                cargo_manifest_path: package
                    .package_root
                    .join("Cargo.toml")
                    .display()
                    .to_string(),
                bridge_roots: package
                    .manifest
                    .rust
                    .bridges
                    .iter()
                    .map(|path| normalized_path_string(path))
                    .collect(),
            },
            "Self" => match &declaration.owner {
                RustInteropOwner::Method { class_name, .. }
                    if self.opaque_contracts.contains_key(&format!(
                        "{}.{}",
                        declaration.module_name.as_deref().unwrap_or("main"),
                        class_name
                    )) =>
                {
                    RustInteropResolvedRoot::SelfMethod {
                        class_name: class_name.clone(),
                    }
                }
                RustInteropOwner::Method { class_name, .. } => {
                    self.push_diagnostic(
                        declaration,
                        path.span,
                        DiagnosticCode::RUST_RESOLVE_TARGET_ROOT,
                        "unresolved Rust target root `{root}`",
                        vec![
                            ("root", root.clone()),
                            ("target", path.dotted()),
                            ("class", class_name.clone()),
                        ],
                        vec![
                            "`Self` target roots are valid only on methods for classes declared with `@rust.opaque(...)`"
                                .to_string(),
                        ],
                        None,
                    );
                    return;
                }
                _ => {
                    self.push_diagnostic(
                        declaration,
                        path.span,
                        DiagnosticCode::RUST_RESOLVE_TARGET_ROOT,
                        "unresolved Rust target root `{root}`",
                        vec![("root", root.clone()), ("target", path.dotted())],
                        vec!["`Self` target roots are valid only on Rust interop methods"
                            .to_string()],
                        None,
                    );
                    return;
                }
            },
            _ => {
                let backend = backend_for_root(&self.context.graph, package_id, root);
                let Some(backend) = backend else {
                    self.push_diagnostic(
                        declaration,
                        path.span,
                        DiagnosticCode::RUST_RESOLVE_TARGET_ROOT,
                        "unresolved Rust target root `{root}`",
                        vec![
                            ("root", root.clone()),
                            ("target", path.dotted()),
                        ],
                        vec![
                            "Rust target roots must be `bridge`, `Self`, or a direct Cargo backend dependency alias".to_string(),
                            format!("declaring package: {}", package_id.0),
                        ],
                        None,
                    );
                    return;
                };
                self.validate_backend_trust(
                    declaration,
                    &canonical_target_path,
                    package,
                    backend,
                    sysroot_trust.is_some(),
                );
                self.validate_backend_generated_bridge_imports(declaration, backend);
                let signature = self
                    .signature_contracts
                    .get(&canonical_target_path)
                    .cloned();
                let async_thread_affinity = self.async_thread_affinity_for_probe(declaration);
                let Some(sysroot_runtime_crate) = self.context.sysroot_runtime_crate.clone() else {
                    self.push_diagnostic(
                        declaration,
                        path.span,
                        DiagnosticCode::RUST_CARGO_METADATA,
                        "Rust bridge probe requires a resolved Sifr sysroot runtime crate",
                        vec![("target", path.dotted())],
                        vec!["Direct Rust bridge probes must use the same resolved sysroot runtime crate as generated Cargo projects.".to_string()],
                        None,
                    );
                    return;
                };
                self.pending_direct_probes.push(PendingRustBridgeProbe {
                    declaration: declaration.clone(),
                    path: path.clone(),
                    backend: backend.clone(),
                    signature,
                    async_thread_affinity,
                    sysroot_runtime_crate,
                    sysroot_vendor_dir: sysroot_trust
                        .as_ref()
                        .map(|trust| trust.vendor_dir.clone()),
                });
                if let Some(trust) = &sysroot_trust {
                    resolved_sysroot_crate_root(&backend.dependency_name, backend, trust)
                        .unwrap_or_else(|| RustInteropResolvedRoot::DirectCargoDependency {
                            dependency_name: backend.dependency_name.clone(),
                            cargo_package_id: backend.cargo_package_id.0.clone(),
                            cargo_package_name: backend.cargo_package_name.clone(),
                            cargo_version: backend.cargo_version.clone(),
                            cargo_source: backend.cargo_source.clone(),
                            cargo_manifest_path: backend.cargo_manifest_path.display().to_string(),
                        })
                } else {
                    RustInteropResolvedRoot::DirectCargoDependency {
                        dependency_name: backend.dependency_name.clone(),
                        cargo_package_id: backend.cargo_package_id.0.clone(),
                        cargo_package_name: backend.cargo_package_name.clone(),
                        cargo_version: backend.cargo_version.clone(),
                        cargo_source: backend.cargo_source.clone(),
                        cargo_manifest_path: backend.cargo_manifest_path.display().to_string(),
                    }
                }
            }
        };

        self.resolved_targets.push(RustInteropResolvedTarget {
            module_name: declaration.module_name.clone(),
            owner: declaration.owner.clone(),
            written_path: path.dotted(),
            canonical_target_path,
            root: resolved_root,
            span: path.span,
        });
    }

    fn validate_backend_trust(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        canonical_target_path: &str,
        package: &sifr_package::SifrPackageMetadata,
        backend: &BackendCrateMetadata,
        trusted_by_sysroot_policy: bool,
    ) {
        if backend.has_build_script {
            self.require_trust(
                declaration,
                canonical_target_path,
                RustInteropTrustRequirementKind::BuildScript,
                &package.manifest.trust.rust_build_scripts,
                &backend.dependency_name,
                format!(
                    "build script in Cargo dependency `{}`",
                    backend.dependency_name
                ),
                trusted_by_sysroot_policy,
            );
        }
        if backend.has_proc_macro {
            self.require_trust(
                declaration,
                canonical_target_path,
                RustInteropTrustRequirementKind::ProcMacro,
                &package.manifest.trust.rust_proc_macros,
                &backend.dependency_name,
                format!(
                    "proc-macro target in Cargo dependency `{}`",
                    backend.dependency_name
                ),
                trusted_by_sysroot_policy,
            );
        }
        if let Some(links) = &backend.links {
            self.require_trust(
                declaration,
                canonical_target_path,
                RustInteropTrustRequirementKind::NativeLinks,
                &package.manifest.trust.native_links,
                links,
                format!(
                    "native links `{links}` declared by Cargo dependency `{}`",
                    backend.dependency_name
                ),
                trusted_by_sysroot_policy,
            );
        }
    }

    fn validate_bridge_version(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        package: &sifr_package::SifrPackageMetadata,
    ) -> bool {
        if package.manifest.rust.bridge_version == Some(1) {
            return true;
        }
        self.push_diagnostic(
            declaration,
            declaration.declaration.span,
            DiagnosticCode::RUST_CARGO_METADATA,
            "unsupported Rust bridge version",
            vec![(
                "bridge_version",
                package
                    .manifest
                    .rust
                    .bridge_version
                    .map_or_else(|| "<missing>".to_string(), |version| version.to_string()),
            )],
            vec!["declare `[rust] bridge-version = 1` in sifr.toml".to_string()],
            Some("Rust interop generated bridge modules are bridge-versioned compatibility surfaces.".to_string()),
        );
        false
    }

    fn push_generated_bridge_module(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        package: &sifr_package::SifrPackageMetadata,
    ) {
        let bridge_version = package.manifest.rust.bridge_version.unwrap_or(1);
        self.generated_bridge_modules
            .insert(RustGeneratedBridgeModule {
                module_name: declaration.module_name.clone(),
                rust_module_path: generated_bridge_module_path(declaration.module_name.as_deref()),
                bridge_version,
            });
    }

    fn validate_backend_generated_bridge_imports(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        backend: &BackendCrateMetadata,
    ) {
        let Some(manifest_dir) = backend.cargo_manifest_path.parent() else {
            return;
        };
        let source_root = manifest_dir.join("src");
        let Some(path) = first_generated_bridge_import(&source_root) else {
            return;
        };
        self.push_diagnostic(
            declaration,
            declaration.declaration.span,
            DiagnosticCode::RUST_RESOLVE_TARGET_ROOT,
            "shared Rust bridge crate imports package-specific generated bridge types",
            vec![
                ("dependency", backend.dependency_name.clone()),
                ("path", path.display().to_string()),
            ],
            vec![
                "move this bridge code into the Sifr package-local bridge root".to_string(),
                "or expose only stable Rust/runtime interop types from the shared crate"
                    .to_string(),
            ],
            None,
        );
    }

    fn validate_declaration_trust(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        package: &sifr_package::SifrPackageMetadata,
    ) {
        let trust_target_path = canonical_trust_target_path(declaration);
        for build_env in build_env_trust_entries(&declaration.declaration) {
            self.require_trust(
                declaration,
                &trust_target_path,
                RustInteropTrustRequirementKind::BuildEnv,
                &package.manifest.trust.build_env,
                &build_env,
                format!("build environment variable `{build_env}`"),
                is_trusted_sysroot_package(self.context, &package.package_id),
            );
        }
        match effective_panic_policy(
            declaration,
            package,
            is_trusted_sysroot_package(self.context, &package.package_id),
        ) {
            EffectivePanicPolicy::TrustedNoPanic => {
                self.require_trust(
                    declaration,
                    &trust_target_path,
                    RustInteropTrustRequirementKind::NoPanic,
                    &package.manifest.trust.rust_no_panic,
                    &trust_target_path,
                    format!("no-panic contract for `{trust_target_path}`"),
                    is_trusted_sysroot_package(self.context, &package.package_id),
                );
            }
            EffectivePanicPolicy::Abort => {
                self.require_trust(
                    declaration,
                    &trust_target_path,
                    RustInteropTrustRequirementKind::PanicAbort,
                    &package.manifest.trust.rust_panic_abort,
                    &trust_target_path,
                    format!("panic-abort contract for `{trust_target_path}`"),
                    is_trusted_sysroot_package(self.context, &package.package_id),
                );
            }
            EffectivePanicPolicy::None
            | EffectivePanicPolicy::MapError
            | EffectivePanicPolicy::Invalid
            | EffectivePanicPolicy::InvalidSysrootImplicitTarget => {}
        }
    }

    fn validate_unsafe_bridge_files(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        package: &sifr_package::SifrPackageMetadata,
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

    #[allow(clippy::too_many_arguments)]
    fn require_trust(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
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
            vec![format!("add `{required_entry}` to the matching `[trust]` Rust interop allow-list before Cargo executes this dependency")],
            None,
        );
    }

    fn push_probe(&mut self, declaration: &sifr_codegen::RustInteropPlanDeclaration) {
        let Some(kind) = probe_planning::probe_kind(&declaration.declaration, &declaration.owner)
        else {
            self.push_diagnostic(
                declaration,
                declaration.declaration.span,
                DiagnosticCode::RUST_TYPE_PROBE_FAILURE,
                "Rust bridge probe failed for `{target}`",
                vec![("target", canonical_sifr_target_path(declaration))],
                vec![
                    "the Rust interop declaration cannot be represented as an isolated probe module"
                        .to_string(),
                ],
                None,
            );
            return;
        };
        let (mut requires_send, requires_sync) =
            opaque_probe_obligations(declaration, &self.opaque_contracts);
        let (view_requires_send, view_requires_sync) =
            zero_copy_validation::view_probe_obligations(declaration);
        requires_send |= view_requires_send;
        let requires_sync = requires_sync || view_requires_sync;
        if requires_send
            && declaration.declaration.abi_requirements.async_boundary
            && self.async_thread_affinity_for_probe(declaration)
                == AsyncThreadAffinity::TokioCurrentThread
        {
            requires_send = false;
        }
        self.probes.push(RustBridgeProbe {
            canonical_target_path: canonical_sifr_target_path(declaration),
            module_name: declaration.module_name.clone(),
            owner: declaration.owner.clone(),
            kind,
            requires_send,
            requires_sync,
            span: declaration.declaration.span,
        });
    }
    fn execute_pending_direct_probes(&mut self) {
        for probe in self.pending_direct_probes.clone() {
            if let Err(failure) = execute_direct_cargo_probe(&probe) {
                self.push_diagnostic(
                    &probe.declaration,
                    probe.declaration.declaration.span,
                    failure.code,
                    failure.message_template,
                    failure.args,
                    failure.notes,
                    None,
                );
            }
        }
    }
    fn validate_bridge_contracts(&mut self, signatures: &[RustBridgeSignatureContract]) {
        for diagnostic in bridge_contract_diagnostics(signatures) {
            self.push_contract_diagnostic(
                &diagnostic.signature,
                "unsupported Rust bridge type `{sifr_type}` in `{target}`",
                diagnostic.args,
                diagnostic.notes,
            );
        }
    }
    fn validate_private_declaration_context(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        module_name: Option<&str>,
        package_id: &SifrPackageId,
    ) -> bool {
        let is_private = module_name.is_some_and(|module| module.starts_with("_sifr."));
        let is_sysroot_package = self.sysroot_trust_for_package(package_id).is_some();
        if is_private == is_sysroot_package {
            return true;
        }
        let message = if is_private {
            "private _sifr Rust interop declarations require the compiler-owned sysroot context"
        } else {
            "sysroot Rust interop context accepts only private _sifr declarations"
        };
        self.push_diagnostic(
            declaration,
            declaration.declaration.span,
            DiagnosticCode::RUST_CARGO_METADATA,
            message,
            Vec::new(),
            Vec::new(),
            Some(
                "User packages cannot impersonate or override private stdlib declaration modules."
                    .to_string(),
            ),
        );
        false
    }
    fn package_id_for_module(&self, module_name: Option<&str>) -> Option<SifrPackageId> {
        if let Some(module) = module_name {
            return self
                .context
                .module_packages
                .get(module)
                .or(Some(&self.context.package_id))
                .cloned();
        }
        if is_trusted_sysroot_package(self.context, &self.context.package_id) {
            None
        } else {
            Some(self.context.package_id.clone())
        }
    }
    fn sysroot_trust_for_package(
        &self,
        package_id: &SifrPackageId,
    ) -> Option<&SysrootRustInteropTrust> {
        self.context
            .sysroot_trust
            .as_ref()
            .filter(|trust| &trust.package_id == package_id)
    }
    #[allow(clippy::too_many_arguments)]
    fn push_diagnostic(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        range: TextRange,
        code: DiagnosticCode,
        message_template: &'static str,
        args: Vec<(&'static str, String)>,
        notes: Vec<String>,
        help: Option<String>,
    ) {
        let Some(module_name) = declaration.module_name.as_deref() else {
            self.diagnostics
                .push(crate::diagnostics::diagnostic_with_code(
                    render_template(message_template, &args),
                    code,
                ));
            return;
        };
        let Some(source) = self.context.module_sources.get(module_name) else {
            self.diagnostics
                .push(crate::diagnostics::diagnostic_with_code(
                    render_template(message_template, &args),
                    code,
                ));
            return;
        };
        self.diagnostics.push(source_diagnostic(
            code,
            &source.display_path,
            &source.source,
            range,
            message_template,
            args,
            notes,
            help,
        ));
    }

    fn push_contract_diagnostic(
        &mut self,
        signature: &RustBridgeSignatureContract,
        message_template: &'static str,
        args: Vec<(&'static str, String)>,
        notes: Vec<String>,
    ) {
        let Some(module_name) = signature.module_name.as_deref() else {
            self.diagnostics
                .push(crate::diagnostics::diagnostic_with_code(
                    render_template(message_template, &args),
                    DiagnosticCode::RUST_TYPE_PROBE_FAILURE,
                ));
            return;
        };
        let Some(source) = self.context.module_sources.get(module_name) else {
            self.diagnostics
                .push(crate::diagnostics::diagnostic_with_code(
                    render_template(message_template, &args),
                    DiagnosticCode::RUST_TYPE_PROBE_FAILURE,
                ));
            return;
        };
        self.diagnostics.push(source_diagnostic(
            DiagnosticCode::RUST_TYPE_PROBE_FAILURE,
            &source.display_path,
            &source.source,
            signature.span,
            message_template,
            args,
            notes,
            None,
        ));
    }
}
