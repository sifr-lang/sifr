use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop_digest::{
    digest_file, digest_path, fnv1a64_hex, normalized_path_string, push_cache_bytes,
    relative_path_string,
};
use super::rust_interop_probe::{execute_direct_cargo_probe, PendingRustBridgeProbe};
use crate::diagnostics::RenderedDiagnostic;
use crate::project::ParsedProjectModule;
use ruff_text_size::TextRange;
use sifr_codegen::{
    RustBridgeProbe, RustBridgeProbeKind, RustBridgeProbePlan, RustBridgeSourceDigest,
    RustInteropCargoInputs, RustInteropOwner, RustInteropResolvedRoot, RustInteropResolvedTarget,
    RustInteropTrustRequirement, RustInteropTrustRequirementKind,
};
use sifr_diagnostics::render::render_sink;
use sifr_diagnostics::{
    ChildSeverity, DiagnosticArg, DiagnosticBuilder, DiagnosticCode, DiagnosticSink, Severity,
    SourceMap, SourceSpan,
};
use sifr_ir::{RustInteropDeclaration, RustInteropDecoratorKind, RustInteropValue, RustTargetPath};
use sifr_package::{
    digest_package_graph, digest_package_source_map, BackendCrateMetadata, PackageSourceMap,
    SifrPackageGraph, SifrPackageId, TrustPolicy,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub(super) struct PackageRustInteropContext {
    pub(super) package_id: SifrPackageId,
    pub(super) graph: SifrPackageGraph,
    pub(super) source_map: PackageSourceMap,
    pub(super) module_packages: HashMap<String, SifrPackageId>,
    pub(super) module_sources: HashMap<String, RustInteropModuleSource>,
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
    trust_requirements: Vec<RustInteropTrustRequirement>,
    seen_trust_requirements: BTreeSet<(String, String, String)>,
    probes: Vec<RustBridgeProbe>,
    pending_direct_probes: Vec<PendingRustBridgeProbe>,
}

impl<'a> RustInteropResolver<'a> {
    fn new(context: &'a PackageRustInteropContext) -> Self {
        Self {
            context,
            diagnostics: Vec::new(),
            resolved_targets: Vec::new(),
            trust_requirements: Vec::new(),
            seen_trust_requirements: BTreeSet::new(),
            probes: Vec::new(),
            pending_direct_probes: Vec::new(),
        }
    }

    fn resolve_plan(
        &mut self,
        generated: &mut GeneratedBinaryProject,
    ) -> Result<(), Vec<RenderedDiagnostic>> {
        for declaration in generated.interop.rust.declarations.clone() {
            self.resolve_declaration(&declaration);
        }

        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }
        self.execute_pending_direct_probes();
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let package = self
            .context
            .graph
            .packages
            .get(&self.context.package_id)
            .expect("package rust interop context must reference graph package");
        generated.interop.rust.resolved_targets = std::mem::take(&mut self.resolved_targets);
        generated.interop.rust.trust_requirements = std::mem::take(&mut self.trust_requirements);
        generated.interop.rust.probe_plan = RustBridgeProbePlan {
            probes: std::mem::take(&mut self.probes),
        };
        generated.interop.rust.bridge_sources = bridge_source_digests(self.context, package);
        generated.interop.rust.cargo_inputs = Some(cargo_inputs(self.context, package));
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

        for path in declaration_paths(&declaration.declaration) {
            self.resolve_path(declaration, &package_id, package, path);
        }
        self.validate_declaration_trust(
            declaration,
            &canonical_sifr_target_path(declaration),
            package,
        );
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
        let canonical_target_path = canonical_sifr_target_path(declaration);
        let resolved_root = match root.as_str() {
            "bridge" => RustInteropResolvedRoot::PackageBridge {
                package_id: package_id.0.clone(),
                bridge_roots: package
                    .manifest
                    .rust
                    .bridges
                    .iter()
                    .map(|path| normalized_path_string(path))
                    .collect(),
            },
            "Self" => match &declaration.owner {
                RustInteropOwner::Method { class_name, .. } => {
                    RustInteropResolvedRoot::SelfMethod {
                        class_name: class_name.clone(),
                    }
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
                self.validate_backend_trust(declaration, &canonical_target_path, package, backend);
                self.pending_direct_probes.push(PendingRustBridgeProbe {
                    declaration: declaration.clone(),
                    path: path.clone(),
                    backend: backend.clone(),
                });
                RustInteropResolvedRoot::DirectCargoDependency {
                    dependency_name: backend.dependency_name.clone(),
                    cargo_package_id: backend.cargo_package_id.0.clone(),
                    cargo_package_name: backend.cargo_package_name.clone(),
                    cargo_version: backend.cargo_version.clone(),
                    cargo_source: backend.cargo_source.clone(),
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
            );
        }
    }

    fn validate_declaration_trust(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        canonical_target_path: &str,
        package: &sifr_package::SifrPackageMetadata,
    ) {
        for build_env in build_env_trust_entries(&declaration.declaration) {
            self.require_trust(
                declaration,
                canonical_target_path,
                RustInteropTrustRequirementKind::BuildEnv,
                &package.manifest.trust.build_env,
                &build_env,
                format!("build environment variable `{build_env}`"),
            );
        }
        match panic_policy(&declaration.declaration).as_deref() {
            Some("trusted_no_panic") => {
                self.require_trust(
                    declaration,
                    canonical_target_path,
                    RustInteropTrustRequirementKind::NoPanic,
                    &package.manifest.trust.rust_no_panic,
                    canonical_target_path,
                    format!("no-panic contract for `{canonical_target_path}`"),
                );
            }
            Some("abort") => {
                self.require_trust(
                    declaration,
                    canonical_target_path,
                    RustInteropTrustRequirementKind::PanicAbort,
                    &package.manifest.trust.rust_panic_abort,
                    canonical_target_path,
                    format!("panic-abort contract for `{canonical_target_path}`"),
                );
            }
            Some(_) | None => {}
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
            );
        }
    }

    fn require_trust(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        canonical_target_path: &str,
        kind: RustInteropTrustRequirementKind,
        trusted_entries: &[String],
        required_entry: &str,
        evidence: String,
    ) {
        let key = (
            canonical_target_path.to_string(),
            trust_kind_name(&kind).to_string(),
            required_entry.to_string(),
        );
        if !self.seen_trust_requirements.insert(key) {
            return;
        }
        let trusted = trusted_entries.iter().any(|entry| entry == required_entry);
        self.trust_requirements.push(RustInteropTrustRequirement {
            canonical_target_path: canonical_target_path.to_string(),
            kind: kind.clone(),
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
        let Some(kind) = probe_kind(&declaration.declaration, &declaration.owner) else {
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
        self.probes.push(RustBridgeProbe {
            canonical_target_path: canonical_sifr_target_path(declaration),
            module_name: declaration.module_name.clone(),
            owner: declaration.owner.clone(),
            kind,
            requires_send: declaration.declaration.abi_requirements.async_boundary,
            requires_sync: declaration.declaration.abi_requirements.view,
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

    fn package_id_for_module(&self, module_name: Option<&str>) -> Option<SifrPackageId> {
        module_name
            .and_then(|module| self.context.module_packages.get(module))
            .or(Some(&self.context.package_id))
            .cloned()
    }

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
            &args,
            &notes,
            help,
        ));
    }
}

fn declaration_paths(declaration: &RustInteropDeclaration) -> Vec<&RustTargetPath> {
    let mut paths = Vec::new();
    if let Some(target) = &declaration.target {
        paths.push(target);
    }
    for argument in &declaration.arguments {
        collect_value_paths(&argument.value, &mut paths);
    }
    paths
}

fn collect_value_paths<'a>(value: &'a RustInteropValue, paths: &mut Vec<&'a RustTargetPath>) {
    match value {
        RustInteropValue::TargetPath(path) => paths.push(path),
        RustInteropValue::PolicyCall { argument, .. } => collect_value_paths(argument, paths),
        RustInteropValue::Boolean(_)
        | RustInteropValue::Symbol(_)
        | RustInteropValue::Integer(_) => {}
    }
}

fn backend_for_root<'a>(
    graph: &'a SifrPackageGraph,
    package_id: &SifrPackageId,
    root: &str,
) -> Option<&'a BackendCrateMetadata> {
    graph
        .backend_crates
        .get(package_id)?
        .iter()
        .find(|backend| backend.dependency_name == root)
}

fn canonical_sifr_target_path(declaration: &sifr_codegen::RustInteropPlanDeclaration) -> String {
    let mut path = declaration
        .module_name
        .clone()
        .unwrap_or_else(|| "main".to_string());
    match &declaration.owner {
        RustInteropOwner::Function { name } => {
            path.push('.');
            path.push_str(name);
        }
        RustInteropOwner::Class { name } => {
            path.push('.');
            path.push_str(name);
        }
        RustInteropOwner::Method { class_name, name } => {
            path.push('.');
            path.push_str(class_name);
            path.push('.');
            path.push_str(name);
        }
    }
    path
}

fn uses_bridge_root(declaration: &RustInteropDeclaration) -> bool {
    declaration_paths(declaration)
        .iter()
        .any(|path| path.segments.first().is_some_and(|root| root == "bridge"))
}

fn panic_policy(declaration: &RustInteropDeclaration) -> Option<String> {
    declaration.arguments.iter().find_map(|argument| {
        if argument.name.as_deref() != Some("panic") {
            return None;
        }
        match &argument.value {
            RustInteropValue::Symbol(policy) => Some(policy.clone()),
            _ => None,
        }
    })
}

fn build_env_trust_entries(declaration: &RustInteropDeclaration) -> Vec<String> {
    declaration
        .arguments
        .iter()
        .filter_map(|argument| {
            if argument.name.as_deref() != Some("build_env") {
                return None;
            }
            match &argument.value {
                RustInteropValue::Symbol(name) => Some(name.clone()),
                _ => None,
            }
        })
        .collect()
}

fn trust_kind_name(kind: &RustInteropTrustRequirementKind) -> &'static str {
    match kind {
        RustInteropTrustRequirementKind::BuildScript => "build_script",
        RustInteropTrustRequirementKind::ProcMacro => "proc_macro",
        RustInteropTrustRequirementKind::NativeLinks => "native_links",
        RustInteropTrustRequirementKind::BuildEnv => "build_env",
        RustInteropTrustRequirementKind::UnsafeBridge => "unsafe_bridge",
        RustInteropTrustRequirementKind::NoPanic => "no_panic",
        RustInteropTrustRequirementKind::PanicAbort => "panic_abort",
    }
}

fn unsafe_bridge_files(package: &sifr_package::SifrPackageMetadata) -> Vec<String> {
    let mut files = Vec::new();
    for bridge_root in &package.manifest.rust.bridges {
        let root = package.package_root.join(bridge_root);
        collect_unsafe_bridge_files(&package.package_root, &root, &mut files);
    }
    files.sort();
    files.dedup();
    files
}

fn collect_unsafe_bridge_files(package_root: &Path, path: &Path, files: &mut Vec<String>) {
    if path.is_file() {
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            return;
        }
        let Ok(source) = fs::read_to_string(path) else {
            return;
        };
        if source.contains("unsafe") {
            files.push(relative_path_string(package_root, path));
        }
        return;
    }
    let Ok(read_dir) = fs::read_dir(path) else {
        return;
    };
    for entry in read_dir.flatten() {
        collect_unsafe_bridge_files(package_root, &entry.path(), files);
    }
}

fn probe_kind(
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

fn bridge_source_digests(
    context: &PackageRustInteropContext,
    package: &sifr_package::SifrPackageMetadata,
) -> Vec<RustBridgeSourceDigest> {
    let mut digests = package
        .manifest
        .rust
        .bridges
        .iter()
        .map(|bridge_root| RustBridgeSourceDigest {
            package_id: context.package_id.0.clone(),
            bridge_root: normalized_path_string(bridge_root),
            digest: digest_path(&package.package_root.join(bridge_root)),
        })
        .collect::<Vec<_>>();
    digests.sort_by(|left, right| {
        (&left.package_id, &left.bridge_root).cmp(&(&right.package_id, &right.bridge_root))
    });
    digests
}

fn cargo_inputs(
    context: &PackageRustInteropContext,
    package: &sifr_package::SifrPackageMetadata,
) -> RustInteropCargoInputs {
    let graph_digest = digest_package_graph(&context.graph);
    let source_map_digest = digest_package_source_map(&context.source_map);
    let trust_policy_digest = trust_policy_digest(&package.manifest.trust);
    let mut declared_build_env = package.manifest.trust.build_env.clone();
    declared_build_env.sort();
    RustInteropCargoInputs {
        package_id: context.package_id.0.clone(),
        cargo_metadata_digest: None,
        package_graph_digest: Some(graph_digest.hex),
        package_source_map_digest: Some(source_map_digest.hex),
        cargo_lock_digest: cargo_lock_digest(&package.package_root),
        target_triple: target_triple(),
        target_features: target_features(),
        cargo_profile: "release".to_string(),
        panic_strategy: std::env::var("SIFR_RUST_PANIC_STRATEGY").ok(),
        profile_codegen_settings: profile_codegen_settings(&package.package_root, "release"),
        cargo_version: tool_version("cargo"),
        rustc_version: tool_version("rustc"),
        bridge_version: package.manifest.rust.bridge_version,
        trust_policy_digest,
        declared_build_env,
    }
}

fn cargo_lock_digest(package_root: &Path) -> Option<String> {
    nearest_ancestor_file(package_root, "Cargo.lock").and_then(|path| digest_file(&path))
}

fn nearest_ancestor_file(start: &Path, file_name: &str) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let candidate = ancestor.join(file_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn profile_codegen_settings(package_root: &Path, profile: &str) -> Vec<(String, String)> {
    let mut settings = Vec::new();
    for cargo_toml in ancestor_cargo_tomls(package_root) {
        let Ok(source) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        let Ok(table) = source.parse::<toml::Table>() else {
            continue;
        };
        let Some(profile_table) = table
            .get("profile")
            .and_then(toml::Value::as_table)
            .and_then(|profiles| profiles.get(profile))
            .and_then(toml::Value::as_table)
        else {
            continue;
        };
        for key in [
            "opt-level",
            "lto",
            "codegen-units",
            "panic",
            "debug",
            "strip",
        ] {
            if let Some(value) = profile_table.get(key) {
                settings.push((
                    format!("{}:{key}", normalized_path_string(&cargo_toml)),
                    value.to_string(),
                ));
            }
        }
    }
    settings.sort();
    settings
}

fn ancestor_cargo_tomls(package_root: &Path) -> Vec<PathBuf> {
    package_root
        .ancestors()
        .map(|ancestor| ancestor.join("Cargo.toml"))
        .filter(|candidate| candidate.is_file())
        .collect()
}

fn target_triple() -> Option<String> {
    std::env::var("SIFR_TARGET").ok().or_else(rustc_host_triple)
}

fn rustc_host_triple() -> Option<String> {
    Command::new("rustc")
        .arg("-vV")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|version| {
            version
                .lines()
                .find_map(|line| line.strip_prefix("host: "))
                .map(str::to_string)
        })
}

fn target_features() -> Vec<String> {
    let mut features = Vec::new();
    if let Ok(flags) = std::env::var("RUSTFLAGS") {
        features.push(format!("RUSTFLAGS={flags}"));
    }
    if let Ok(flags) = std::env::var("CARGO_ENCODED_RUSTFLAGS") {
        features.push(format!("CARGO_ENCODED_RUSTFLAGS={flags}"));
    }
    features.sort();
    features
}

fn trust_policy_digest(trust: &TrustPolicy) -> String {
    let mut entries = BTreeMap::new();
    entries.insert("rust-build-scripts", trust.rust_build_scripts.clone());
    entries.insert("rust-proc-macros", trust.rust_proc_macros.clone());
    entries.insert("native-links", trust.native_links.clone());
    entries.insert("unsafe-rust-bridges", trust.unsafe_rust_bridges.clone());
    entries.insert("build-env", trust.build_env.clone());
    entries.insert("rust-no-panic", trust.rust_no_panic.clone());
    entries.insert("rust-panic-abort", trust.rust_panic_abort.clone());
    let mut bytes = Vec::new();
    for (key, mut values) in entries {
        values.sort();
        push_cache_bytes(&mut bytes, key);
        for value in values {
            push_cache_bytes(&mut bytes, &value);
        }
    }
    fnv1a64_hex(&bytes)
}

fn tool_version(tool: &str) -> Option<String> {
    Command::new(tool)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|version| version.trim().to_string())
}

fn source_diagnostic(
    code: DiagnosticCode,
    display_path: &str,
    source: &str,
    range: TextRange,
    message_template: &'static str,
    args: &[(&'static str, String)],
    notes: &[String],
    help: Option<String>,
) -> RenderedDiagnostic {
    let mut source_map = SourceMap::new();
    let source_id = source_map.register_source(display_path, source);
    let span = match SourceSpan::new_validated(&source_map, source_id, range) {
        Ok(span) => span,
        Err(error) => {
            return crate::diagnostics::diagnostic_with_code(
                format!("internal compiler error: invalid Rust interop diagnostic span: {error:?}"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            );
        }
    };
    let mut builder =
        DiagnosticBuilder::source(code, Severity::Error, span).message_template(message_template);
    for (name, value) in args {
        builder = builder.arg(name, DiagnosticArg::String(value.clone()));
    }
    for note in notes {
        builder = builder.child(ChildSeverity::Note, note.clone());
    }
    if let Some(help) = help {
        builder = builder.help(help);
    }
    let mut sink = DiagnosticSink::new();
    sink.emit_error(builder.build());
    match render_sink(&sink, &source_map) {
        Ok(mut envelope) => envelope.diagnostics.remove(0),
        Err(error) => crate::diagnostics::diagnostic_with_code(
            format!("internal compiler error: failed to render Rust interop diagnostic: {error:?}"),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
    }
}

fn render_template(template: &str, args: &[(&'static str, String)]) -> String {
    args.iter()
        .fold(template.to_string(), |message, (name, value)| {
            message.replace(&format!("{{{name}}}"), value)
        })
}
