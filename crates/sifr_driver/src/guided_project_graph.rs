//! Bounded, read-only project graph input for the guided target and replay test.
use crate::{CompilerContext, check_project};
use sifr_frontend::{
    DiskSourceProvider, DocumentVersion, OverlayDocument, OverlaySourceProvider, SourcePath,
    SourceText,
};
use sifr_package::{CargoPackageId, SifrManifest};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const MAX_INPUT_BYTES: usize = 32;
const MODULES: [&str; 3] = ["alpha", "beta", "gamma"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectTree {
    pub manifest: String,
    pub files: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayOutcome {
    InvalidInput,
    ManifestDiagnostic,
    ProjectDiagnostics(Vec<(String, String)>),
    Accepted,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../verification/fuzz/project_fixtures/base")
}

/// The first four bytes are the complete minimized representation of the tree.
/// Each input selects bounded manifest edges and import edges; no input creates
/// a file or launches a dependency/tool process.
#[must_use]
pub fn project_tree(input: &[u8]) -> Option<ProjectTree> {
    if input.len() < 4 || input.len() > MAX_INPUT_BYTES {
        return None;
    }
    let dependency_mask = input[0] & 0b111;
    let source_root = if input[0] & 0x80 != 0 {
        "../escape"
    } else {
        "."
    };
    let mut manifest = format!(
        "[package]\nname = \"guided_project\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"{source_root}\"\n"
    );
    if dependency_mask != 0 {
        manifest.push_str("\n[dependencies]\n");
        for (index, module) in MODULES.iter().enumerate() {
            if dependency_mask & (1 << index) != 0 {
                manifest.push_str(&format!("{module} = \"0.1\"\n"));
            }
        }
    }
    let mut files = Vec::with_capacity(4);
    let mut main = String::new();
    for (index, module) in MODULES.iter().enumerate() {
        if input[1] & (1 << index) != 0 {
            main.push_str(&format!("from {module} import value_{module}\n"));
        }
    }
    main.push_str("\ndef main() -> None:\n    pass\n");
    files.push(("main.sifr".to_string(), main));
    for (index, module) in MODULES.iter().enumerate() {
        let next = MODULES[(index + 1) % MODULES.len()];
        let edge_mask = input[2 + index / 2];
        let edge = edge_mask & (1 << (index % 2)) != 0;
        let mut source = if edge {
            format!("from {next} import value_{next}\n\n")
        } else {
            String::new()
        };
        source.push_str(&format!(
            "def value_{module}() -> int:\n    return {index}\n"
        ));
        files.push((format!("{module}.sifr"), source));
    }
    Some(ProjectTree { manifest, files })
}

/// Manifest validation and the canonical project checker have distinct outcomes.
/// The runner owns build/tool/timeout classification; only a compiler crash or
/// failed invariant is a fuzz finding. User diagnostics are ordinary outcomes.
#[must_use]
pub fn replay(input: &[u8]) -> ReplayOutcome {
    let Some(tree) = project_tree(input) else {
        return ReplayOutcome::InvalidInput;
    };
    let root = fixture_root();
    let manifest_path = root.join("sifr.toml");
    if SifrManifest::parse(
        &CargoPackageId("guided_project".to_string()),
        &manifest_path,
        &tree.manifest,
    )
    .is_err()
    {
        return ReplayOutcome::ManifestDiagnostic;
    }
    let mut provider = OverlaySourceProvider::new(DiskSourceProvider::new());
    for (name, source) in &tree.files {
        provider.insert_overlay(OverlayDocument::new(
            SourcePath::new(root.join(name)),
            None,
            DocumentVersion::new(1),
            SourceText::new(source.clone()),
            None,
        ));
    }
    static COMPILER: OnceLock<CompilerContext> = OnceLock::new();
    let compiler = COMPILER.get_or_init(CompilerContext::for_test);
    let diagnostics = check_project(compiler, &root.join("main.sifr"), &mut provider);
    if diagnostics.is_empty() {
        ReplayOutcome::Accepted
    } else {
        ReplayOutcome::ProjectDiagnostics(
            diagnostics
                .into_iter()
                .map(|diagnostic| (diagnostic.code, diagnostic.message))
                .collect(),
        )
    }
}
