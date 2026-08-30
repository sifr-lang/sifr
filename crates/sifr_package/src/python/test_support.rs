use super::{PythonEnvironmentProbe, PythonEnvironmentProbeRequest, PythonImportProbe};
use crate::cargo::metadata::CargoPackageId;
use crate::graph::derive::{
    PackageClassification, SifrPackageGraph, SifrPackageId, SifrPackageMetadata,
};
use crate::manifest::sifr::{
    CompilerRequirement, PackageSourceRoot, PythonConfig, RustInteropConfig, SifrEdition,
    SifrManifest, SifrPackageName, TrustPolicy,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(super) fn graph(packages: Vec<SifrPackageMetadata>) -> SifrPackageGraph {
    let package_map = packages
        .into_iter()
        .map(|package| (package.package_id.clone(), package))
        .collect::<BTreeMap<_, _>>();
    let classifications = package_map
        .values()
        .map(|package| {
            (
                package.cargo_package_id.clone(),
                PackageClassification::SifrSource(package.package_id.clone()),
            )
        })
        .collect();
    SifrPackageGraph {
        packages: package_map,
        cargo_edges: BTreeMap::new(),
        direct_dependency_scopes: BTreeMap::new(),
        backend_crates: BTreeMap::new(),
        classifications,
    }
}

pub(super) fn package(name: &str, python: PythonConfig, trust: TrustPolicy) -> SifrPackageMetadata {
    let package_id = package_id(name);
    let cargo_package_id = cargo_id(name);
    SifrPackageMetadata {
        package_id: package_id.clone(),
        cargo_package_id: cargo_package_id.clone(),
        cargo_package_name: format!("sifr-{name}"),
        cargo_version: "0.1.0".to_string(),
        cargo_source: None,
        package_root: PathBuf::from(format!("/ws/{name}")),
        sifr_manifest: PathBuf::from(format!("/ws/{name}/sifr.toml")),
        sifr_name: SifrPackageName(name.to_string()),
        manifest: SifrManifest {
            package_name: SifrPackageName(name.to_string()),
            edition: SifrEdition("2026".to_string()),
            compiler_requirement: CompilerRequirement(">=0.3,<0.4".to_string()),
            default_run: None,
            source_root: PackageSourceRoot(PathBuf::from("src")),
            source_features: BTreeMap::new(),
            scripts: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            compiler_components: BTreeMap::new(),
            sql: crate::SqlConfig::default(),
            trust,
            python,
            rust: RustInteropConfig::default(),
        },
        aliases: BTreeMap::new(),
    }
}

pub(super) fn request() -> PythonEnvironmentProbeRequest {
    PythonEnvironmentProbeRequest {
        venv_root: PathBuf::from("/tmp/venv"),
        interpreter: PathBuf::from("/tmp/venv/bin/python"),
        pyproject: None,
        lock: None,
        required_imports: Vec::new(),
        declared_imports: vec!["sys".to_string()],
        native_imports: Vec::new(),
    }
}

pub(super) fn valid_probe() -> PythonEnvironmentProbe {
    PythonEnvironmentProbe {
        implementation_name: "CPython".to_string(),
        implementation_version: "3.14.7".to_string(),
        cpython_version_tuple: vec![3, 14, 7],
        executable: "/tmp/venv/bin/python".to_string(),
        sys_prefix: "/tmp/venv".to_string(),
        sys_base_prefix: "/usr/local".to_string(),
        site_packages: vec!["/tmp/venv/lib/python3.14/site-packages".to_string()],
        sys_path: vec!["/tmp/venv/lib/python3.14".to_string()],
        soabi: Some("cpython-314-darwin".to_string()),
        extension_suffixes: vec![".cpython-314-darwin.so".to_string()],
        pointer_width: 64,
        platform: "macOS-15.0-arm64-arm-64bit".to_string(),
        machine: "arm64".to_string(),
        libpython: Some("/usr/local/lib/libpython3.14.dylib".to_string()),
        free_threaded: false,
        imports: vec![PythonImportProbe {
            root: "sys".to_string(),
            ok: true,
            origin: None,
            distributions: Vec::new(),
            error: None,
        }],
        native_imports: Vec::new(),
        pyproject_digest: None,
        uv_lock_digest: None,
    }
}

pub(super) fn cargo_id(name: &str) -> CargoPackageId {
    CargoPackageId(format!("path+file:///ws/{name}#sifr-{name}@0.1.0"))
}

pub(super) fn package_id(name: &str) -> SifrPackageId {
    SifrPackageId(format!("sifr-{name}@0.1.0#path"))
}
