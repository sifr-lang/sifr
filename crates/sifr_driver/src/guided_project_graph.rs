//! Bounded package-project graphs for the guided target and disk replay.
use crate::{CompilerContext, PackageEntrypoint, check_package_project};
use serde_json::json;
use sifr_frontend::{
    DiskSourceProvider, DocumentVersion, OverlayDocument, OverlaySourceProvider, SourcePath,
    SourceProvider, SourceText,
};
use sifr_package::{CargoLockMode, PackageSourceMap, SifrManifest};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const NAMES: [&str; 4] = ["app", "alpha", "beta", "gamma"];
const VERSION: &str = "0.1.0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectTree {
    pub files: Vec<(PathBuf, String)>,
}

impl ProjectTree {
    pub fn write_to(&self, root: &Path) -> std::io::Result<()> {
        for (relative, source) in &self.files {
            let path = root.join(relative);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, source)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayOutcome {
    InvalidInput,
    ManifestDiagnostic,
    GraphDiagnostic,
    ProjectDiagnostics(Vec<String>),
    Accepted,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../verification/fuzz/project_fixtures/base")
}

fn edges(input: &[u8]) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(6);
    for to in 1..NAMES.len() {
        if input[0] & (1 << (to - 1)) != 0 {
            result.push((0, to));
        }
    }
    for (bit, from, to) in [(0, 1, 2), (1, 2, 3), (2, 3, 1)] {
        if input[2] & (1 << bit) != 0 {
            result.push((from, to));
        }
    }
    result
}

/// Exactly four bytes describe at most four packages and six dependency edges.
/// All filesystem fixtures are checked in; fuzz iterations use source overlays.
#[must_use]
pub fn project_tree(input: &[u8]) -> Option<ProjectTree> {
    if input.len() != 4 {
        return None;
    }
    let graph_edges = edges(input);
    let mut files = Vec::with_capacity(16);
    for (index, name) in NAMES.iter().enumerate() {
        let dependencies = graph_edges
            .iter()
            .filter_map(|(from, to)| (*from == index).then_some(NAMES[*to]))
            .collect::<Vec<_>>();
        let mut cargo = format!(
            "[package]\nname = \"{name}\"\nversion = \"{VERSION}\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n"
        );
        let source_root = if index == 0 && input[0] & 0x80 != 0 {
            "../escape"
        } else {
            "src"
        };
        let mut manifest = format!(
            "[package]\nname = \"{name}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"{source_root}\"\n"
        );
        if !dependencies.is_empty() {
            cargo.push_str("\n[dependencies]\n");
            manifest.push_str("\n[dependencies]\n");
            for dependency in dependencies {
                let _ = writeln!(cargo, "{dependency} = {{ path = \"../{dependency}\" }}");
                let _ = writeln!(
                    manifest,
                    "{dependency} = {{ package = \"{dependency}\", path = \"../{dependency}\", import = \"{dependency}\" }}"
                );
            }
        }
        files.push((PathBuf::from(name).join("Cargo.toml"), cargo));
        files.push((PathBuf::from(name).join("sifr.toml"), manifest));
        files.push((PathBuf::from(name).join("src/lib.rs"), String::new()));
        let mut source = String::new();
        if index == 0 {
            for (bit, dependency) in NAMES.iter().skip(1).enumerate() {
                if input[1] & (1 << bit) != 0 {
                    let _ = writeln!(source, "from {dependency} import value_{dependency}");
                }
            }
            source.push_str("\ndef main() -> None:\n    pass\n");
            files.push((PathBuf::from(name).join("src/main.sifr"), source));
        } else {
            for (_, to) in graph_edges.iter().filter(|(from, _)| *from == index) {
                if input[3] & (1 << (index - 1)) != 0 {
                    let dependency = NAMES[*to];
                    let _ = writeln!(source, "from {dependency} import value_{dependency}");
                }
            }
            let _ = writeln!(source, "\ndef value_{name}() -> int:\n    return {index}");
            files.push((PathBuf::from(name).join("src/__init__.sifr"), source));
        }
    }
    Some(ProjectTree { files })
}

fn package_id(root: &Path, name: &str) -> String {
    format!("path+file://{}#{name}@{VERSION}", root.join(name).display())
}

fn metadata_json(root: &Path, input: &[u8]) -> serde_json::Value {
    let graph_edges = edges(input);
    let packages = NAMES
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let dependencies = graph_edges
                .iter()
                .filter(|(from, _)| *from == index)
                .map(|(_, to)| {
                    json!({
                        "name": NAMES[*to], "package": NAMES[*to], "req": "*",
                        "kind": null, "target": null, "uses_workspace": false
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "id": package_id(root, name), "name": name, "version": VERSION,
                "source": null, "manifest_path": root.join(name).join("Cargo.toml"),
                "dependencies": dependencies,
                "targets": [{
                    "name": name, "kind": ["lib"], "crate_types": ["lib"],
                    "src_path": root.join(name).join("src/lib.rs")
                }],
                "features": {}, "metadata": {"sifr": {"manifest": "sifr.toml"}}
            })
        })
        .collect::<Vec<_>>();
    let nodes = NAMES
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let deps = graph_edges
                .iter()
                .filter(|(from, _)| *from == index)
                .map(|(_, to)| {
                    json!({
                        "name": NAMES[*to], "pkg": package_id(root, NAMES[*to])
                    })
                })
                .collect::<Vec<_>>();
            json!({"id": package_id(root, name), "deps": deps})
        })
        .collect::<Vec<_>>();
    json!({
        "packages": packages, "resolve": {"nodes": nodes},
        "workspace_members": NAMES.iter().map(|name| package_id(root, name)).collect::<Vec<_>>(),
        "target_directory": root.join("target"), "workspace_root": root
    })
}

fn replay_with_provider<P: SourceProvider>(
    input: &[u8],
    root: &Path,
    provider: &mut P,
) -> ReplayOutcome {
    static COMPILER: OnceLock<CompilerContext> = OnceLock::new();

    for name in NAMES {
        let manifest = root.join(name).join("sifr.toml");
        let Ok(source) = provider.read_file(&manifest) else {
            return ReplayOutcome::GraphDiagnostic;
        };
        let cargo_id = sifr_package::CargoPackageId(package_id(root, name));
        if SifrManifest::parse(&cargo_id, &manifest, source.as_str()).is_err() {
            return ReplayOutcome::ManifestDiagnostic;
        }
    }
    let metadata_source = metadata_json(root, input).to_string();
    let Ok(metadata) = sifr_package::parse_metadata_json(&metadata_source) else {
        return ReplayOutcome::GraphDiagnostic;
    };
    let Ok(graph) = sifr_package::derive_package_graph(metadata, provider) else {
        return ReplayOutcome::GraphDiagnostic;
    };
    let Ok(source_map) = PackageSourceMap::build(&graph, provider) else {
        return ReplayOutcome::GraphDiagnostic;
    };
    let Some(package_id) = graph
        .packages
        .values()
        .find(|metadata| metadata.sifr_name.0 == "app")
        .map(|metadata| metadata.package_id.clone())
    else {
        return ReplayOutcome::GraphDiagnostic;
    };
    let entrypoint = PackageEntrypoint {
        main_file: root.join("app/src/main.sifr"),
        package_id,
        graph,
        source_map,
        python_runtime: None,
        lock_mode: CargoLockMode::Normal,
    };
    let compiler = COMPILER.get_or_init(CompilerContext::for_test);
    let diagnostics = check_package_project(compiler, &entrypoint, provider);
    if diagnostics.is_empty() {
        ReplayOutcome::Accepted
    } else {
        ReplayOutcome::ProjectDiagnostics(
            diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.code)
                .collect(),
        )
    }
}

/// All canonical package resolution and compilation reads use the input overlay.
#[must_use]
pub fn replay(input: &[u8]) -> ReplayOutcome {
    let Some(tree) = project_tree(input) else {
        return ReplayOutcome::InvalidInput;
    };
    let root = fixture_root();
    let mut provider = OverlaySourceProvider::new(DiskSourceProvider::new());
    for (relative, source) in tree.files {
        provider.insert_overlay(OverlayDocument::new(
            SourcePath::new(root.join(relative)),
            None,
            DocumentVersion::new(1),
            SourceText::new(source),
            None,
        ));
    }
    replay_with_provider(input, &root, &mut provider)
}

/// Recheck a minimized tree from its exported on-disk files.
#[must_use]
pub fn replay_exported_tree(input: &[u8], root: &Path) -> ReplayOutcome {
    if project_tree(input).is_none() {
        return ReplayOutcome::InvalidInput;
    }
    replay_with_provider(input, root, &mut DiskSourceProvider::new())
}
