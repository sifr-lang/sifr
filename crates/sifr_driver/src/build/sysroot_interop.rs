use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{PackageRustInteropContext, RustInteropModuleSource};
use crate::stdlib::StdlibRustInterop;
use sifr_codegen::{InteropBuildPlan, RustInteropResolvedRoot};
use sifr_package::{
    BackendCrateMetadata, CargoPackageId, PackageClassification, PackageSourceMap,
    PackageSourceRoot, RustInteropConfig, SifrEdition, SifrManifest, SifrPackageGraph,
    SifrPackageId, SifrPackageMetadata, SifrPackageName, TrustPolicy,
};
use sifr_stdlib_manifest::SysrootCrate;
use sifr_sysroot::{COMPILER_SIFR_VERSION, ResolvedSysroot};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(super) struct SysrootRustInteropTrust {
    pub(super) package_id: SifrPackageId,
    pub(super) sysroot_root: PathBuf,
    pub(super) stdlib_private_sources: PathBuf,
    pub(super) stdlib_crate: PathBuf,
    pub(super) runtime_crate: PathBuf,
    pub(super) cargo_lock: PathBuf,
    pub(super) vendor_dir: PathBuf,
    pub(super) toolchain_id: String,
    pub(super) sysroot_content_sha256: String,
}

pub(super) fn attach_stdlib_rust_interop(
    mut generated: GeneratedBinaryProject,
    package_context: Option<PackageRustInteropContext>,
    stdlib_interop: &StdlibRustInterop,
) -> (GeneratedBinaryProject, Option<PackageRustInteropContext>) {
    if stdlib_interop.plan.rust.declarations.is_empty() {
        return (generated, package_context);
    }
    merge_interop_plan(&mut generated.interop, &stdlib_interop.plan);
    (
        generated,
        merge_contexts(package_context, stdlib_context(stdlib_interop)),
    )
}

pub(super) fn sysroot_crate_for_dependency_name(name: &str) -> Option<SysrootCrate> {
    match name {
        "sifr_runtime" => Some(SysrootCrate::SifrRuntime),
        "sifr_stdlib" => Some(SysrootCrate::SifrStdlib),
        _ => None,
    }
}

pub(super) fn resolved_sysroot_crate_root(
    dependency_name: &str,
    backend: &BackendCrateMetadata,
    trust: &SysrootRustInteropTrust,
) -> Option<RustInteropResolvedRoot> {
    sysroot_crate_for_dependency_name(dependency_name)?;
    Some(RustInteropResolvedRoot::SysrootCrate {
        dependency_name: backend.dependency_name.clone(),
        cargo_package_name: backend.cargo_package_name.clone(),
        cargo_version: backend.cargo_version.clone(),
        cargo_manifest_path: backend.cargo_manifest_path.display().to_string(),
        sysroot_root: trust.sysroot_root.display().to_string(),
        toolchain_id: trust.toolchain_id.clone(),
        sysroot_content_sha256: trust.sysroot_content_sha256.clone(),
    })
}

pub(super) fn is_trusted_sysroot_package(
    context: &PackageRustInteropContext,
    package_id: &SifrPackageId,
) -> bool {
    context
        .sysroot_trust
        .as_ref()
        .is_some_and(|trust| &trust.package_id == package_id)
}

fn merge_interop_plan(target: &mut InteropBuildPlan, stdlib: &InteropBuildPlan) {
    target
        .rust
        .declarations
        .extend(stdlib.rust.declarations.clone());
    target
        .rust
        .bridge_contracts
        .signatures
        .extend(stdlib.rust.bridge_contracts.signatures.clone());
    target
        .rust
        .bridge_contracts
        .generated_types
        .extend(stdlib.rust.bridge_contracts.generated_types.clone());
    if let Some(version) = stdlib.rust.structural_identity_algorithm_version {
        debug_assert!(
            target
                .rust
                .structural_identity_algorithm_version
                .is_none_or(|target_version| target_version == version),
            "compiler-owned structural identity algorithms must agree"
        );
        target.rust.structural_identity_algorithm_version = Some(version);
    }
    target
        .rust
        .structural_shape_identities
        .extend(stdlib.rust.structural_shape_identities.clone());
    target.rust.structural_shape_identities.sort();
    target.rust.structural_shape_identities.dedup();
}

fn merge_contexts(
    package_context: Option<PackageRustInteropContext>,
    stdlib_context: Option<PackageRustInteropContext>,
) -> Option<PackageRustInteropContext> {
    let Some(mut stdlib_context) = stdlib_context else {
        return package_context;
    };
    let Some(mut package_context) = package_context else {
        return Some(stdlib_context);
    };

    package_context
        .graph
        .packages
        .append(&mut stdlib_context.graph.packages);
    package_context
        .graph
        .backend_crates
        .append(&mut stdlib_context.graph.backend_crates);
    package_context
        .graph
        .classifications
        .append(&mut stdlib_context.graph.classifications);
    package_context
        .module_packages
        .extend(stdlib_context.module_packages);
    package_context
        .module_sources
        .extend(stdlib_context.module_sources);
    if package_context.sysroot_runtime_crate.is_none() {
        package_context.sysroot_runtime_crate = stdlib_context.sysroot_runtime_crate;
    }
    package_context.sysroot_trust = stdlib_context.sysroot_trust;
    Some(package_context)
}

fn stdlib_context(stdlib_interop: &StdlibRustInterop) -> Option<PackageRustInteropContext> {
    let sysroot = stdlib_interop.sysroot.as_ref()?;
    let package_id = SifrPackageId(format!("sysroot-stdlib@{}#sysroot", sysroot.toolchain_id()));
    let cargo_package_id = CargoPackageId(format!(
        "path+file://{}#sifr-sysroot-stdlib@{}",
        sysroot.root.display(),
        COMPILER_SIFR_VERSION
    ));
    let package = sysroot_package(sysroot, &package_id, &cargo_package_id);
    let module_packages = stdlib_interop
        .module_sources
        .keys()
        .map(|module| (module.clone(), package_id.clone()))
        .collect::<HashMap<_, _>>();
    let module_sources = stdlib_interop
        .module_sources
        .iter()
        .map(|(module, source)| {
            (
                module.clone(),
                RustInteropModuleSource {
                    source: source.source.clone(),
                    display_path: source.display_path.clone(),
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let trust = SysrootRustInteropTrust {
        package_id: package_id.clone(),
        sysroot_root: sysroot.root.clone(),
        stdlib_private_sources: sysroot.paths.stdlib_private_sources.clone(),
        stdlib_crate: sysroot.paths.stdlib_crate.clone(),
        runtime_crate: sysroot.paths.runtime_crate.clone(),
        cargo_lock: sysroot.paths.cargo_lock.clone(),
        vendor_dir: sysroot.paths.vendor.clone(),
        toolchain_id: sysroot.toolchain_id(),
        sysroot_content_sha256: sysroot.manifest.sysroot_content_sha256.clone(),
    };
    Some(PackageRustInteropContext {
        package_id: package_id.clone(),
        graph: SifrPackageGraph {
            packages: BTreeMap::from([(package_id.clone(), package)]),
            cargo_edges: BTreeMap::new(),
            direct_dependency_scopes: BTreeMap::new(),
            backend_crates: BTreeMap::from([(package_id.clone(), sysroot_backends(sysroot))]),
            classifications: BTreeMap::from([(
                cargo_package_id,
                PackageClassification::RustBackedSifr(package_id.clone()),
            )]),
        },
        source_map: PackageSourceMap::default(),
        module_packages,
        module_sources,
        sysroot_runtime_crate: Some(sysroot.paths.runtime_crate.clone()),
        sysroot_trust: Some(trust),
    })
}

fn sysroot_package(
    sysroot: &ResolvedSysroot,
    package_id: &SifrPackageId,
    cargo_package_id: &CargoPackageId,
) -> SifrPackageMetadata {
    SifrPackageMetadata {
        package_id: package_id.clone(),
        cargo_package_id: cargo_package_id.clone(),
        cargo_package_name: "sifr-sysroot-stdlib".to_string(),
        cargo_version: COMPILER_SIFR_VERSION.to_string(),
        cargo_source: None,
        package_root: sysroot.root.clone(),
        sifr_manifest: sysroot.paths.manifest.clone(),
        sifr_name: SifrPackageName("_sifr".to_string()),
        manifest: SifrManifest {
            package_name: SifrPackageName("_sifr".to_string()),
            edition: SifrEdition("2026".to_string()),
            compiler_requirement: sifr_package::CompilerRequirement(">=0.3,<0.4".to_string()),
            default_run: None,
            source_root: PackageSourceRoot(sysroot.paths.stdlib_private_sources.clone()),
            source_features: BTreeMap::new(),
            scripts: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            compiler_components: BTreeMap::new(),
            sql: sifr_package::SqlConfig::default(),
            trust: TrustPolicy::default(),
            python: sifr_package::PythonConfig::default(),
            rust: RustInteropConfig {
                bridges: Vec::new(),
                direct_crate_bindings: true,
            },
        },
        aliases: BTreeMap::new(),
    }
}

fn sysroot_backends(sysroot: &ResolvedSysroot) -> Vec<BackendCrateMetadata> {
    let mut backends = vec![
        sysroot_backend(
            SysrootCrate::SifrRuntime,
            &sysroot.paths.runtime_crate_manifest,
        ),
        sysroot_backend(
            SysrootCrate::SifrStdlib,
            &sysroot.paths.stdlib_crate_manifest,
        ),
    ];
    backends.sort_by(|left, right| left.dependency_name.cmp(&right.dependency_name));
    backends
}

fn sysroot_backend(krate: SysrootCrate, manifest_path: &std::path::Path) -> BackendCrateMetadata {
    let package_name = krate.package_name();
    BackendCrateMetadata {
        cargo_package_id: CargoPackageId(format!(
            "path+file://{}#{}@{}",
            manifest_path.display(),
            package_name,
            COMPILER_SIFR_VERSION
        )),
        dependency_name: package_name.to_string(),
        dependency_kind: None,
        cargo_package_name: package_name.to_string(),
        cargo_version: COMPILER_SIFR_VERSION.to_string(),
        cargo_source: None,
        cargo_manifest_path: manifest_path.to_path_buf(),
        links: None,
        has_build_script: false,
        has_proc_macro: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::project_codegen::GeneratedBinaryProject;
    use crate::build::rust_interop::apply_package_rust_interop_metadata;
    use crate::stdlib::{StdlibRustInterop, StdlibRustInteropModuleSource};
    use ruff_text_size::{TextRange, TextSize};
    use sifr_codegen::{RustInteropOwner, RustInteropPlan, RustInteropPlanDeclaration};
    use sifr_ir::{
        RustInteropAbiRequirements, RustInteropArgument, RustInteropDeclaration,
        RustInteropDecoratorKind, RustInteropEffect, RustInteropValue, RustTargetPath,
    };
    use sifr_stdlib_manifest::StdlibFeature;
    use sifr_sysroot::{SysrootManifest, SysrootPaths};
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sysroot_private_interop_resolves_canonical_stdlib_crate() {
        let root = TempSysroot::new("resolve_stdlib_crate");
        root.write_private(
            "_sifr.crypto",
            "@rust(sifr_stdlib.hash)\ndef hash() -> None:\n    pass\n",
        );
        root.write_stdlib_crate("pub fn hash() {}\n");
        let stdlib = stdlib_interop(
            &root,
            declaration("_sifr.crypto", "hash", "sifr_stdlib.hash"),
        );

        let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &stdlib);
        let resolved = apply_package_rust_interop_metadata(generated, context)
            .expect("sysroot interop should resolve");

        assert!(matches!(
            &resolved.interop.rust.resolved_targets[0].root,
            RustInteropResolvedRoot::SysrootCrate {
                dependency_name,
                sysroot_root,
                ..
            } if dependency_name == "sifr_stdlib" && sysroot_root == &root.path.display().to_string()
        ));
        assert!(
            resolved
                .interop
                .cache_key_fragment()
                .contains("rust.cargo.package=sifr-sysroot-stdlib@")
        );
    }

    #[test]
    fn sysroot_private_interop_rejects_non_sysroot_target_root() {
        let root = TempSysroot::new("reject_non_sysroot_root");
        root.write_private(
            "_sifr.crypto",
            "@rust(native.hash, panic=trusted_no_panic)\ndef hash() -> None:\n    pass\n",
        );
        let mut entry = declaration("_sifr.crypto", "hash", "native.hash");
        entry.declaration.arguments = vec![RustInteropArgument {
            name: Some("panic".to_string()),
            value: RustInteropValue::Symbol("trusted_no_panic".to_string()),
            span: span(),
        }];
        let stdlib = stdlib_interop(&root, entry);

        let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &stdlib);
        let diagnostics = match apply_package_rust_interop_metadata(generated, context) {
            Ok(_) => panic!("non-sysroot root should fail"),
            Err(diagnostics) => diagnostics,
        };

        assert_eq!(diagnostics[0].code, "SIFR-RUST-RESOLVE-0001");
        assert!(diagnostics[0].message.contains("canonical sysroot crate"));
        assert_eq!(
            diagnostics[0].spans[0].file.as_deref(),
            Some(root.private_path("_sifr.crypto").to_string_lossy().as_ref())
        );
    }

    #[test]
    fn sysroot_private_opaque_interop_resolves_self_close_method() {
        let root = TempSysroot::new("opaque_self_close");
        root.write_private(
            "_sifr.io",
            "class FileHandleError(Error):\n\
    message: str\n\
\n\
@rust.opaque(type=sifr_stdlib.io.FileHandle, close=close)\n\
class FileHandle:\n\
\n\
    @rust(Self.close, panic=trusted_no_panic)\n\
    def close(own self) -> Result[None, FileHandleError]:\n\
        ...\n",
        );
        root.write_stdlib_crate("pub mod io { pub struct FileHandle; pub fn close() {} }\n");
        let mut close_method = method_declaration(
            "_sifr.io",
            "FileHandle",
            "close",
            "Self.close",
            vec![symbol_argument("panic", "trusted_no_panic")],
        );
        close_method.declaration.consumes_receiver = true;
        let stdlib = stdlib_interop_many(
            &root,
            vec![
                opaque_class_declaration(
                    "_sifr.io",
                    "FileHandle",
                    "sifr_stdlib.io.FileHandle",
                    vec![symbol_argument("close", "close")],
                ),
                close_method,
            ],
        );

        let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &stdlib);
        let resolved = apply_package_rust_interop_metadata(generated, context)
            .expect("opaque sysroot interop should resolve");

        assert!(
            resolved
                .interop
                .rust
                .resolved_targets
                .iter()
                .any(|target| target.written_path == "sifr_stdlib.io.FileHandle"
                    && matches!(
                        &target.root,
                        RustInteropResolvedRoot::SysrootCrate {
                            dependency_name,
                            sysroot_root,
                            ..
                        } if dependency_name == "sifr_stdlib"
                            && sysroot_root == &root.path.display().to_string()
                    ))
        );
        assert!(
            resolved
                .interop
                .rust
                .resolved_targets
                .iter()
                .any(|target| target.written_path == "Self.close"
                    && matches!(
                        &target.root,
                        RustInteropResolvedRoot::SelfMethod { class_name }
                            if class_name == "FileHandle"
                    ))
        );
        assert!(
            resolved
                .interop
                .rust
                .trust_requirements
                .iter()
                .any(
                    |requirement| requirement.required_entry == "Self.close" && requirement.trusted
                )
        );
    }

    #[test]
    fn sysroot_private_interop_rejects_self_method_without_opaque_class() {
        let root = TempSysroot::new("self_method_without_opaque");
        root.write_private(
            "_sifr.io",
            "class FileHandle:\n\
\n\
    @rust(Self.close, panic=trusted_no_panic)\n\
    def close(self) -> None:\n\
        ...\n",
        );
        let stdlib = stdlib_interop(
            &root,
            method_declaration(
                "_sifr.io",
                "FileHandle",
                "close",
                "Self.close",
                vec![symbol_argument("panic", "trusted_no_panic")],
            ),
        );

        let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &stdlib);
        let diagnostics = match apply_package_rust_interop_metadata(generated, context) {
            Ok(_) => panic!("Self root should require an opaque class"),
            Err(diagnostics) => diagnostics,
        };

        assert_eq!(diagnostics[0].code, "SIFR-RUST-RESOLVE-0001");
        assert!(!diagnostics[0].message.contains("canonical sysroot crate"));
        assert!(
            diagnostics[0]
                .children
                .iter()
                .any(|child| child.message.contains("@rust.opaque"))
        );
        assert_eq!(
            diagnostics[0].spans[0].file.as_deref(),
            Some(root.private_path("_sifr.io").to_string_lossy().as_ref())
        );
    }

    #[test]
    fn sysroot_private_opaque_interop_rejects_non_sysroot_rust_type() {
        let root = TempSysroot::new("opaque_reject_non_sysroot_type");
        root.write_private(
            "_sifr.io",
            "@rust.opaque(type=native.io.FileHandle)\nclass FileHandle:\n    ...\n",
        );
        let stdlib = stdlib_interop(
            &root,
            opaque_class_declaration("_sifr.io", "FileHandle", "native.io.FileHandle", Vec::new()),
        );

        let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &stdlib);
        let diagnostics = match apply_package_rust_interop_metadata(generated, context) {
            Ok(_) => panic!("opaque rust type root should be canonical"),
            Err(diagnostics) => diagnostics,
        };

        assert_eq!(diagnostics[0].code, "SIFR-RUST-RESOLVE-0001");
        assert!(diagnostics[0].message.contains("canonical sysroot crate"));
        assert_eq!(
            diagnostics[0].spans[0].file.as_deref(),
            Some(root.private_path("_sifr.io").to_string_lossy().as_ref())
        );
    }

    #[test]
    fn sysroot_interop_cache_changes_with_private_declaration_source() {
        let root = TempSysroot::new("cache_private_source");
        root.write_private(
            "_sifr.crypto",
            "@rust(sifr_stdlib.hash)\ndef hash() -> None:\n    pass\n",
        );
        root.write_stdlib_crate("pub fn hash() {}\n");
        let first = resolved_cache_fragment(&root);

        root.write_private(
            "_sifr.crypto",
            "# changed declaration comment\n@rust(sifr_stdlib.hash)\ndef hash() -> None:\n    pass\n",
        );
        let second = resolved_cache_fragment(&root);

        assert_ne!(first, second);
    }

    #[test]
    fn sysroot_interop_cache_changes_with_sysroot_crate_source() {
        let root = TempSysroot::new("cache_crate_source");
        root.write_private(
            "_sifr.crypto",
            "@rust(sifr_stdlib.hash)\ndef hash() -> None:\n    pass\n",
        );
        root.write_stdlib_crate("pub fn hash() {}\n");
        let first = resolved_cache_fragment(&root);

        root.write_stdlib_crate("pub fn hash() {}\npub fn changed() {}\n");
        let second = resolved_cache_fragment(&root);

        assert_ne!(first, second);
    }

    fn resolved_cache_fragment(root: &TempSysroot) -> String {
        let stdlib = stdlib_interop(
            root,
            declaration("_sifr.crypto", "hash", "sifr_stdlib.hash"),
        );
        let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &stdlib);
        apply_package_rust_interop_metadata(generated, context)
            .expect("sysroot interop should resolve")
            .interop
            .cache_key_fragment()
    }

    fn stdlib_interop(
        root: &TempSysroot,
        declaration: RustInteropPlanDeclaration,
    ) -> StdlibRustInterop {
        stdlib_interop_many(root, vec![declaration])
    }

    fn stdlib_interop_many(
        root: &TempSysroot,
        declarations: Vec<RustInteropPlanDeclaration>,
    ) -> StdlibRustInterop {
        let module = declarations
            .first()
            .expect("test should provide declarations")
            .module_name
            .clone()
            .expect("test declaration should be named");
        let source = root.private_source(&module);
        StdlibRustInterop {
            plan: InteropBuildPlan {
                rust: RustInteropPlan {
                    declarations,
                    ..RustInteropPlan::default()
                },
                ..InteropBuildPlan::default()
            },
            module_sources: HashMap::from([(
                module.clone(),
                StdlibRustInteropModuleSource {
                    source,
                    display_path: root.private_path(&module).display().to_string(),
                },
            )]),
            sysroot: Some(root.resolved()),
        }
    }

    fn declaration(
        module_name: &str,
        function_name: &str,
        target: &str,
    ) -> RustInteropPlanDeclaration {
        RustInteropPlanDeclaration {
            module_name: Some(module_name.to_string()),
            owner: RustInteropOwner::Function {
                name: function_name.to_string(),
            },
            declaration: RustInteropDeclaration {
                kind: RustInteropDecoratorKind::Function,
                target: Some(RustTargetPath {
                    segments: target.split('.').map(str::to_string).collect(),
                    span: span(),
                }),
                arguments: Vec::new(),
                span: span(),
                effect: RustInteropEffect::Sync,
                abi_requirements: RustInteropAbiRequirements::default(),
                consumes_receiver: false,
            },
        }
    }

    fn opaque_class_declaration(
        module_name: &str,
        class_name: &str,
        rust_type: &str,
        mut arguments: Vec<RustInteropArgument>,
    ) -> RustInteropPlanDeclaration {
        arguments.insert(0, target_argument("type", rust_type));
        RustInteropPlanDeclaration {
            module_name: Some(module_name.to_string()),
            owner: RustInteropOwner::Class {
                name: class_name.to_string(),
            },
            declaration: RustInteropDeclaration {
                kind: RustInteropDecoratorKind::Opaque,
                target: None,
                arguments,
                span: span(),
                effect: RustInteropEffect::Sync,
                abi_requirements: RustInteropAbiRequirements {
                    opaque_handle: true,
                    ..RustInteropAbiRequirements::default()
                },
                consumes_receiver: false,
            },
        }
    }

    fn method_declaration(
        module_name: &str,
        class_name: &str,
        method_name: &str,
        target: &str,
        arguments: Vec<RustInteropArgument>,
    ) -> RustInteropPlanDeclaration {
        RustInteropPlanDeclaration {
            module_name: Some(module_name.to_string()),
            owner: RustInteropOwner::Method {
                class_name: class_name.to_string(),
                name: method_name.to_string(),
            },
            declaration: RustInteropDeclaration {
                kind: RustInteropDecoratorKind::Function,
                target: Some(target_path(target)),
                arguments,
                span: span(),
                effect: RustInteropEffect::Sync,
                abi_requirements: RustInteropAbiRequirements::default(),
                consumes_receiver: false,
            },
        }
    }

    fn target_argument(name: &str, target: &str) -> RustInteropArgument {
        RustInteropArgument {
            name: Some(name.to_string()),
            value: RustInteropValue::TargetPath(target_path(target)),
            span: span(),
        }
    }

    fn symbol_argument(name: &str, symbol: &str) -> RustInteropArgument {
        RustInteropArgument {
            name: Some(name.to_string()),
            value: RustInteropValue::Symbol(symbol.to_string()),
            span: span(),
        }
    }

    fn target_path(target: &str) -> RustTargetPath {
        RustTargetPath {
            segments: target.split('.').map(str::to_string).collect(),
            span: span(),
        }
    }

    fn base_project() -> GeneratedBinaryProject {
        GeneratedBinaryProject {
            main_rs: String::new(),
            support_modules: BTreeMap::new(),
            used_stdlib_modules: HashSet::new(),
            required_features: HashSet::<StdlibFeature>::new(),
            interop: InteropBuildPlan::default(),
            cache_key_fragment: None,
            bridge_modules: Default::default(),
            python_runtime: None,
        }
    }

    fn span() -> TextRange {
        TextRange::new(TextSize::from(0), TextSize::from(22))
    }

    struct TempSysroot {
        path: PathBuf,
        private_sources: RefCell<HashMap<String, String>>,
    }

    impl TempSysroot {
        fn new(label: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos());
            let path = std::env::temp_dir().join(format!(
                "sifr_sysroot_interop_{label}_{}_{}",
                std::process::id(),
                nonce
            ));
            fs::create_dir_all(path.join("stdlib/_sifr")).expect("private sources");
            fs::create_dir_all(path.join("stdlib/sifr")).expect("public sources");
            fs::create_dir_all(path.join("crates/sifr_stdlib/src")).expect("stdlib crate");
            fs::create_dir_all(path.join("crates/sifr_runtime/src")).expect("runtime crate");
            fs::create_dir_all(path.join(".cargo")).expect("cargo config dir");
            fs::create_dir_all(path.join("vendor")).expect("vendor dir");
            fs::write(
                path.join("Cargo.toml"),
                "[workspace]\nmembers = [\"crates/sifr_runtime\", \"crates/sifr_stdlib\"]\nresolver = \"3\"\n",
            )
            .expect("workspace manifest");
            fs::write(path.join("Cargo.lock"), "# test lock\n").expect("lockfile");
            fs::write(
                path.join(".cargo/config.toml"),
                "[source.crates-io]\nreplace-with = \"sifr-vendor\"\n[source.sifr-vendor]\ndirectory = \"vendor\"\n",
            )
            .expect("cargo config");
            fs::write(
                path.join("crates/sifr_runtime/Cargo.toml"),
                "[package]\nname = \"sifr_runtime\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            )
            .expect("runtime manifest");
            fs::write(
                path.join("crates/sifr_runtime/src/lib.rs"),
                "pub mod interop {}\n",
            )
            .expect("runtime lib");
            fs::write(
                path.join("crates/sifr_stdlib/Cargo.toml"),
                "[package]\nname = \"sifr_stdlib\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            )
            .expect("stdlib manifest");
            fs::write(path.join("sysroot.toml"), "# test sysroot\n").expect("sysroot manifest");
            Self {
                path,
                private_sources: RefCell::new(HashMap::new()),
            }
        }

        fn write_private(&self, module: &str, source: &str) {
            fs::write(self.private_path(module), source).expect("private source");
            self.private_sources
                .borrow_mut()
                .insert(module.to_string(), source.to_string());
        }

        fn private_source(&self, module: &str) -> String {
            self.private_sources
                .borrow()
                .get(module)
                .cloned()
                .expect("private source should be registered")
        }

        fn write_stdlib_crate(&self, source: &str) {
            fs::write(self.path.join("crates/sifr_stdlib/src/lib.rs"), source).expect("stdlib lib");
        }

        fn private_path(&self, module: &str) -> PathBuf {
            let filename = module
                .strip_prefix("_sifr.")
                .expect("private module prefix");
            self.path.join(format!("stdlib/_sifr/{filename}.sifr"))
        }

        fn resolved(&self) -> ResolvedSysroot {
            ResolvedSysroot {
                root: self.path.clone(),
                manifest: SysrootManifest {
                    schema_version: 1,
                    sifr_version: "0.0.0-test".to_string(),
                    target_triple: "test-target".to_string(),
                    built_by_compiler_commit: "test".to_string(),
                    sysroot_content_sha256:
                        "0000000000000000000000000000000000000000000000000000000000000000"
                            .to_string(),
                    cargo_lock_sha256:
                        "0000000000000000000000000000000000000000000000000000000000000000"
                            .to_string(),
                },
                paths: SysrootPaths::from_root(&self.path),
                cargo_lock_content_sha256: "2".repeat(64),
            }
        }
    }

    impl Drop for TempSysroot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
