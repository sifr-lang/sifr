use crate::test_support::{TestExpectErr as _, TestUnwrap as _};

use crate::{
    CargoDependency, CargoPackage, CargoPackageId, CargoResolveEdge, CargoSifrMetadata,
    CargoTarget, CargoWorkspaceSifrMetadata, HostToolGraph, NormalizedCargoMetadata,
    PackageClassification, PackageGraphSnapshot, PackageSourceMap, SifrPackageGraph,
    load_host_tool_lock, resolve_host_tool_graph, verify_host_tool_graph, write_host_tool_lock,
};
use sifr_frontend::DiskSourceProvider;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[test]
fn host_tool_graph_resolves_locked_host_entrypoint() {
    let fixture = ToolFixture::new("resolves");
    let snapshot = fixture.snapshot(false);
    let graph = resolve_host_tool_graph(&snapshot, &mut DiskSourceProvider::new())
        .test_unwrap("tool graph should resolve");
    let entry = &graph.entries["sql"];
    assert_eq!(entry.entrypoint, "sql-tool");
    assert_eq!(entry.capabilities.len(), 3);
    let plan = graph
        .build_plan("sql", "aarch64-apple-darwin")
        .test_unwrap("command should plan");
    assert_eq!(plan.args[0], "build");
    assert!(plan.args.iter().any(|arg| arg == "--locked"));
    assert!(
        plan.args
            .windows(2)
            .any(|pair| { pair == ["--target".to_string(), "aarch64-apple-darwin".to_string()] })
    );
    assert!(!plan.args.iter().any(|arg| arg == "--"));
    assert!(graph.build_plan("missing", "aarch64-apple-darwin").is_err());
}

#[test]
fn host_tool_graph_rejects_reserved_namespaces_and_unknown_capabilities() {
    let reserved = ToolFixture::new("reserved");
    reserved.write_manifest(
        "[tools.build]\npackage = \"provider-tools\"\nentrypoint = \"sql-tool\"\ncapabilities = []\n",
    );
    let errors = resolve_host_tool_graph(&reserved.snapshot(false), &mut DiskSourceProvider::new())
        .test_expect_err("reserved namespace must fail");
    assert!(errors[0].message.contains("reserved"));

    let unknown = ToolFixture::new("unknown_capability");
    unknown.write_manifest(
        "[tools.sql]\npackage = \"provider-tools\"\nentrypoint = \"sql-tool\"\ncapabilities = [\"telepathy\"]\n",
    );
    let errors = resolve_host_tool_graph(&unknown.snapshot(false), &mut DiskSourceProvider::new())
        .test_expect_err("unknown capability must fail");
    assert!(errors[0].message.contains("unknown capability"));

    let duplicate = ToolFixture::new("duplicate_namespace");
    duplicate.write_manifest(
        "[tools.sql]\npackage = \"provider-tools\"\nentrypoint = \"sql-tool\"\ncapabilities = []\n\n[tools.sql]\npackage = \"provider-tools\"\nentrypoint = \"sql-tool\"\ncapabilities = []\n",
    );
    let errors =
        resolve_host_tool_graph(&duplicate.snapshot(false), &mut DiskSourceProvider::new())
            .test_expect_err("duplicate namespace must fail TOML parsing");
    assert!(errors[0].message.contains("cannot parse tools manifest"));
}

#[test]
fn host_tool_graph_rejects_application_contamination() {
    let fixture = ToolFixture::new("contamination");
    let errors = resolve_host_tool_graph(&fixture.snapshot(true), &mut DiskSourceProvider::new())
        .test_expect_err("application dependency on a tool must fail");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("reaches host-only tool package"))
    );
}

#[test]
fn application_graph_derivation_rejects_host_tool_contamination_before_source_loading() {
    let fixture = ToolFixture::new("derivation_contamination");
    let snapshot = fixture.snapshot(true);
    let errors = crate::graph::derive::derive_package_graph_from_normalized(
        &snapshot.metadata,
        &mut DiskSourceProvider::new(),
    )
    .test_expect_err("application graph derivation must reject host tools");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("reaches host-only tool package"))
    );
}

#[test]
fn host_tool_graph_detects_lockfile_hash_drift() {
    let fixture = ToolFixture::new("hash_drift");
    let snapshot = fixture.snapshot(false);
    let graph: HostToolGraph =
        resolve_host_tool_graph(&snapshot, &mut DiskSourceProvider::new()).test_unwrap("resolve");
    std::fs::write(fixture.root.join("Cargo.lock"), "version = 4\n").test_unwrap("mutate lockfile");
    let error = verify_host_tool_graph(&graph, &mut DiskSourceProvider::new())
        .test_expect_err("hash drift must fail");
    assert!(error.message.contains("hash drifted"));
}

#[test]
fn committed_host_tool_lock_detects_manifest_and_path_source_drift() {
    let fixture = ToolFixture::new("persisted_drift");
    let graph = resolve_host_tool_graph(&fixture.snapshot(false), &mut DiskSourceProvider::new())
        .test_unwrap("resolve");
    write_host_tool_lock(&graph).test_unwrap("write host-tool lock");
    load_host_tool_lock(&graph, &mut DiskSourceProvider::new()).test_unwrap("verify lock");

    std::fs::write(
        fixture.root.join("provider/src/added.rs"),
        "pub const DRIFT: u8 = 1;\n",
    )
    .test_unwrap("mutate provider");
    let observed =
        resolve_host_tool_graph(&fixture.snapshot(false), &mut DiskSourceProvider::new())
            .test_unwrap("resolve drifted graph");
    let error = load_host_tool_lock(&observed, &mut DiskSourceProvider::new())
        .test_expect_err("persisted lock must detect drift");
    assert!(error.message.contains("does not match"));
}

#[test]
fn host_tool_graph_rejects_missing_entrypoint_and_non_normal_dependency() {
    let missing = ToolFixture::new("missing_entry");
    missing.write_manifest(
        "[tools.sql]\npackage = \"provider-tools\"\nentrypoint = \"missing\"\ncapabilities = []\n",
    );
    assert!(
        resolve_host_tool_graph(&missing.snapshot(false), &mut DiskSourceProvider::new())
            .test_expect_err("missing entrypoint")
            .iter()
            .any(|error| error.message.contains("missing binary"))
    );

    let dev_only = ToolFixture::new("dev_only");
    let errors = resolve_host_tool_graph(
        &dev_only.snapshot_with_dependency_kind(Some("dev")),
        &mut DiskSourceProvider::new(),
    )
    .test_expect_err("dev dependency is not a tool declaration edge");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("normal workspace-member"))
    );
}

struct ToolFixture {
    root: PathBuf,
    tools_id: CargoPackageId,
    provider_id: CargoPackageId,
    app_id: CargoPackageId,
}

impl ToolFixture {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .test_unwrap("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "sifr_host_tools_{name}_{}_{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("tools")).test_unwrap("tools dir");
        std::fs::create_dir_all(root.join("provider/src")).test_unwrap("provider dir");
        std::fs::write(
            root.join("Cargo.lock"),
            "version = 4\n\n[[package]]\nname = \"project-tools\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"provider-tools\"\nversion = \"1.2.3\"\n",
        )
        .test_unwrap("lockfile");
        let fixture = Self {
            root,
            tools_id: CargoPackageId("path+project-tools@0.1.0".to_string()),
            provider_id: CargoPackageId("path+provider-tools@1.2.3".to_string()),
            app_id: CargoPackageId("path+app@0.1.0".to_string()),
        };
        fixture.write_manifest(
            "[tools.sql]\npackage = \"provider-tools\"\nentrypoint = \"sql-tool\"\ncapabilities = [\"credentials\", \"network\", \"project-write\"]\n",
        );
        fixture
    }

    fn write_manifest(&self, source: &str) {
        std::fs::write(self.root.join("tools/sifr.toml"), source).test_unwrap("tool manifest");
    }

    fn snapshot(&self, contaminated: bool) -> PackageGraphSnapshot {
        self.snapshot_inner(contaminated, None)
    }

    fn snapshot_with_dependency_kind(&self, kind: Option<&str>) -> PackageGraphSnapshot {
        self.snapshot_inner(false, kind)
    }

    fn snapshot_inner(&self, contaminated: bool, kind: Option<&str>) -> PackageGraphSnapshot {
        let tools = CargoPackage {
            id: self.tools_id.clone(),
            name: "project-tools".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            links: None,
            manifest_path: self.root.join("tools/Cargo.toml"),
            dependencies: vec![CargoDependency {
                name: "provider-tools".to_string(),
                package: None,
                req: "*".to_string(),
                kind: kind.map(str::to_string),
                target: None,
                uses_workspace: false,
            }],
            targets: Vec::new(),
            features: BTreeMap::new(),
            sifr_metadata: Some(CargoSifrMetadata {
                manifest: PathBuf::from("sifr.toml"),
                aliases: BTreeMap::new(),
            }),
        };
        let provider = CargoPackage {
            id: self.provider_id.clone(),
            name: "provider-tools".to_string(),
            version: "1.2.3".to_string(),
            source: None,
            links: None,
            manifest_path: self.root.join("provider/Cargo.toml"),
            dependencies: Vec::new(),
            targets: vec![CargoTarget {
                name: "sql-tool".to_string(),
                kind: BTreeSet::from(["bin".to_string()]),
                crate_types: BTreeSet::from(["bin".to_string()]),
                src_path: self.root.join("provider/src/main.rs"),
            }],
            features: BTreeMap::new(),
            sifr_metadata: None,
        };
        let app = CargoPackage {
            id: self.app_id.clone(),
            name: "app".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            links: None,
            manifest_path: self.root.join("app/Cargo.toml"),
            dependencies: Vec::new(),
            targets: Vec::new(),
            features: BTreeMap::new(),
            sifr_metadata: contaminated.then(|| CargoSifrMetadata {
                manifest: PathBuf::from("sifr.toml"),
                aliases: BTreeMap::new(),
            }),
        };
        let mut resolve_edges = vec![CargoResolveEdge {
            from: self.tools_id.clone(),
            dependency_name: "provider-tools".to_string(),
            to: self.provider_id.clone(),
        }];
        if contaminated {
            resolve_edges.push(CargoResolveEdge {
                from: self.app_id.clone(),
                dependency_name: "provider-tools".to_string(),
                to: self.provider_id.clone(),
            });
        }
        PackageGraphSnapshot {
            metadata: NormalizedCargoMetadata {
                packages: BTreeMap::from([
                    (self.tools_id.clone(), tools),
                    (self.provider_id.clone(), provider),
                    (self.app_id.clone(), app),
                ]),
                resolve_edges,
                workspace_members: BTreeSet::from([
                    self.tools_id.clone(),
                    self.provider_id.clone(),
                    self.app_id.clone(),
                ]),
                workspace_default_members: BTreeSet::from([self.app_id.clone()]),
                target_directory: self.root.join("target"),
                workspace_root: self.root.clone(),
                workspace_sifr: CargoWorkspaceSifrMetadata {
                    tools_package: Some("project-tools".to_string()),
                },
            },
            graph: SifrPackageGraph {
                packages: BTreeMap::new(),
                cargo_edges: BTreeMap::new(),
                direct_dependency_scopes: BTreeMap::new(),
                backend_crates: BTreeMap::new(),
                classifications: BTreeMap::from([
                    (self.tools_id.clone(), PackageClassification::HostTools),
                    (self.provider_id.clone(), PackageClassification::BackendRust),
                    (
                        self.app_id.clone(),
                        PackageClassification::SifrSource(crate::SifrPackageId(
                            "app@0.1.0#path".to_string(),
                        )),
                    ),
                ]),
            },
            source_map: PackageSourceMap::default(),
        }
    }
}

impl Drop for ToolFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
