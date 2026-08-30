use crate::{
    CargoPackage, CargoPackageId, CargoResolveEdge, CargoSifrMetadata, CargoTarget,
    CargoWorkspaceSifrMetadata, HostToolGraph, NormalizedCargoMetadata, PackageClassification,
    PackageGraphSnapshot, PackageSourceMap, SifrPackageGraph, resolve_host_tool_graph,
    verify_host_tool_graph,
};
use sifr_frontend::DiskSourceProvider;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[test]
fn host_tool_graph_resolves_locked_host_entrypoint() {
    let fixture = ToolFixture::new("resolves");
    let snapshot = fixture.snapshot(false);
    let graph = resolve_host_tool_graph(&snapshot, &mut DiskSourceProvider::new())
        .expect("tool graph should resolve");
    let entry = &graph.entries["sql"];
    assert_eq!(entry.entrypoint, "sql-tool");
    assert_eq!(entry.capabilities.len(), 3);
    let plan = graph
        .command_plan(
            "sql",
            "aarch64-apple-darwin",
            &["schema".to_string(), "build".to_string()],
        )
        .expect("command should plan");
    assert_eq!(plan.program, "cargo");
    assert!(plan.args.iter().any(|arg| arg == "--locked"));
    assert!(
        plan.args
            .windows(2)
            .any(|pair| { pair == ["--target".to_string(), "aarch64-apple-darwin".to_string()] })
    );
    assert_eq!(&plan.args[plan.args.len() - 2..], ["schema", "build"]);
}

#[test]
fn host_tool_graph_rejects_reserved_namespaces_and_unknown_capabilities() {
    let reserved = ToolFixture::new("reserved");
    reserved.write_manifest(
        "[tools.build]\npackage = \"provider-tools\"\nentrypoint = \"sql-tool\"\ncapabilities = []\n",
    );
    let errors = resolve_host_tool_graph(&reserved.snapshot(false), &mut DiskSourceProvider::new())
        .expect_err("reserved namespace must fail");
    assert!(errors[0].message.contains("reserved"));

    let unknown = ToolFixture::new("unknown_capability");
    unknown.write_manifest(
        "[tools.sql]\npackage = \"provider-tools\"\nentrypoint = \"sql-tool\"\ncapabilities = [\"telepathy\"]\n",
    );
    let errors = resolve_host_tool_graph(&unknown.snapshot(false), &mut DiskSourceProvider::new())
        .expect_err("unknown capability must fail");
    assert!(errors[0].message.contains("unknown capability"));
}

#[test]
fn host_tool_graph_rejects_application_contamination() {
    let fixture = ToolFixture::new("contamination");
    let errors = resolve_host_tool_graph(&fixture.snapshot(true), &mut DiskSourceProvider::new())
        .expect_err("application dependency on a tool must fail");
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
        resolve_host_tool_graph(&snapshot, &mut DiskSourceProvider::new()).expect("resolve");
    std::fs::write(fixture.root.join("Cargo.lock"), "version = 4\n").expect("mutate lockfile");
    let error = verify_host_tool_graph(&graph, &mut DiskSourceProvider::new())
        .expect_err("hash drift must fail");
    assert!(error.message.contains("hash drifted"));
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
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "sifr_host_tools_{name}_{}_{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("tools")).expect("tools dir");
        std::fs::create_dir_all(root.join("provider/src")).expect("provider dir");
        std::fs::write(
            root.join("Cargo.lock"),
            "version = 4\n\n[[package]]\nname = \"project-tools\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"provider-tools\"\nversion = \"1.2.3\"\n",
        )
        .expect("lockfile");
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
        std::fs::write(self.root.join("tools/sifr.toml"), source).expect("tool manifest");
    }

    fn snapshot(&self, contaminated: bool) -> PackageGraphSnapshot {
        let tools = CargoPackage {
            id: self.tools_id.clone(),
            name: "project-tools".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            links: None,
            manifest_path: self.root.join("tools/Cargo.toml"),
            dependencies: Vec::new(),
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
            sifr_metadata: None,
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
                workspace_members: BTreeSet::from([self.tools_id.clone(), self.app_id.clone()]),
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
