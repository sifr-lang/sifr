use super::*;
use sifr_frontend::persistence::*;
use sifr_frontend::{DiskSourceProvider, FrontendDiagnosticStyle};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

fn compatibility() -> wire::Compatibility {
    wire::Compatibility {
        compiler: [1; 32],
        semantic_target: [2; 32],
        stdlib_inputs: [3; 32],
    }
}
#[test]
fn dx12_typed_interface_checked_and_codegen_roundtrip() {
    let source = CapturedSource {
        path: "/workspace/value.sifr".into(),
        text: r#"
class Token:
    value: int
    def __init__(self, value: int = 7):
        self.value = value

def identity[T](value: T) -> T:
    return value

def consume(value: Token, offset: int = 2) -> int:
    return value.value + offset
"#
        .into(),
    };
    let suite = sifr_frontend::parse_source(&source.text, None).unwrap();
    let lowered = sifr_frontend::compile_module_hir(
        "value",
        &suite,
        &sifr_lowering::ExternalDefs::default(),
        FrontendDiagnosticStyle::Bare,
    )
    .unwrap();
    let mut defs = sifr_lowering::ExternalDefs::default();
    sifr_frontend::collect_module_exports("value", &lowered, &mut defs);
    let modules = BTreeMap::from([(
        "value".into(),
        ProjectModuleInput {
            source: &source,
            input_identity: source.identity(),
            hir: &lowered.module,
        },
    )]);
    let artifact =
        encode_project_results("package@1/source-A", &modules, &defs, compatibility()).unwrap();
    let serialized = serde_json::to_vec(&artifact).unwrap();
    let restored: ProjectTypedArtifact = serde_json::from_slice(&serialized).unwrap();
    let decoded = crate::decode_project_results(&restored, compatibility()).unwrap();
    assert_eq!(
        decoded["value"].0.functions.len(),
        lowered.module.functions.len()
    );
    assert_eq!(
        decoded["value"].1.functions.get("value"),
        defs.functions.get("value")
    );
    assert!(
        !decoded["value"]
            .1
            .function_defaults
            .get("value")
            .unwrap()
            .is_empty()
    );
    // The byte-identical re-encoding below verifies default expression payloads
    // as well as typed bodies, which intentionally have no live PartialEq.

    let reencoded = encode_project_results(
        "package@1/source-A",
        &BTreeMap::from([(
            "value".into(),
            ProjectModuleInput {
                source: &source,
                input_identity: source.identity(),
                hir: &decoded["value"].0,
            },
        )]),
        &decoded["value"].1,
        compatibility(),
    )
    .unwrap();
    assert_eq!(
        artifact.bytes, reencoded.bytes,
        "fresh live owners remap to the same artifact-local IDs"
    );
    assert_eq!(artifact.modules, reencoded.modules);
    let other =
        encode_project_results("package@2/source-B", &modules, &defs, compatibility()).unwrap();
    assert_ne!(
        artifact.modules["value"].checked.hir,
        other.modules["value"].checked.hir
    );
    let handoff = CodegenHandoff {
        checked_module_identity: source.identity(),
        codegen_identity: "codegen-options".into(),
        required_specializations: BTreeMap::from([(
            "identity[int]".into(),
            "specialization-input".into(),
        )]),
    };
    assert_eq!(
        handoff,
        serde_json::from_slice::<CodegenHandoff>(&serde_json::to_vec(&handoff).unwrap()).unwrap()
    );
    let mut corrupt = restored.clone();
    corrupt.sources.values_mut().next().unwrap().text.push(' ');
    assert!(crate::decode_project_results(&corrupt, compatibility()).is_err());
    let mut corrupt = restored.clone();
    corrupt
        .modules
        .get_mut("value")
        .unwrap()
        .checked
        .input_identity
        .push('0');
    assert!(crate::decode_project_results(&corrupt, compatibility()).is_err());
    let mut wrong = compatibility();
    wrong.compiler[0] = 9;
    assert!(crate::decode_project_results(&restored, wrong).is_err());
}
#[test]
fn actual_resolver_and_cycle_policy() {
    use crate::project::{
        DiscoveryDiagnosticStyle, ModuleResolver, parse_import_closure_source_modules,
    };
    let scratch = tempfile::tempdir().unwrap();
    let entry = scratch.path().join("entry");
    let library = scratch.path().join("lib");
    std::fs::create_dir_all(&entry).unwrap();
    std::fs::create_dir_all(&library).unwrap();
    std::fs::write(entry.join("main.sifr"), "from helper import value\n").unwrap();
    std::fs::write(library.join("helper.sifr"), "value: int = 1\n").unwrap();
    let resolver = ModuleResolver::with_workspace(
        &entry,
        crate::workspace::WorkspaceRoot {
            dir: scratch.path().into(),
            config: crate::workspace::SifrWorkspaceConfig {
                source_root: "lib".into(),
                package_name: None,
            },
        },
    );
    let roots = BTreeSet::from(["main".into()]);
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    let parsed = parse_import_closure_source_modules(
        &resolver,
        &roots,
        DiscoveryDiagnosticStyle::ModuleName,
        &mut capture,
    )
    .unwrap();
    assert_eq!(parsed["helper"].source, "value: int = 1\n");
    assert!(capture.unchanged());
    std::fs::write(entry.join("helper.sifr"), "value: int = 2\n").unwrap();
    assert!(!capture.unchanged());
    let mut second = CapturingSourceProvider::new(&mut disk);
    let parsed = parse_import_closure_source_modules(
        &resolver,
        &roots,
        DiscoveryDiagnosticStyle::ModuleName,
        &mut second,
    )
    .unwrap();
    assert_eq!(parsed["helper"].source, "value: int = 2\n");
    std::fs::remove_file(entry.join("helper.sifr")).unwrap();
    assert!(!second.unchanged());
    std::fs::remove_file(library.join("helper.sifr")).unwrap();
    let mut missing = CapturingSourceProvider::new(&mut disk);
    assert!(
        parse_import_closure_source_modules(
            &resolver,
            &roots,
            DiscoveryDiagnosticStyle::ModuleName,
            &mut missing
        )
        .is_err()
    );
    std::fs::write(
        library.join("helper.sifr"),
        "from main import value\nvalue: int = 3\n",
    )
    .unwrap();
    assert!(!missing.unchanged());
    let parsed = parse_import_closure_source_modules(
        &resolver,
        &roots,
        DiscoveryDiagnosticStyle::ModuleName,
        &mut disk,
    )
    .unwrap();
    let suites = parsed
        .into_iter()
        .map(|(name, value)| (name, value.suite))
        .collect();
    assert!(
        crate::project::compute_module_compile_order(&suites).is_err(),
        "existing cycle rejection remains authoritative"
    );
}

#[test]
fn real_writer_cannot_relabel_captured_analysis() {
    use sifr_frontend::SourceProvider;
    let scratch = tempfile::tempdir().unwrap();
    let path = scratch.path().join("main.sifr");
    std::fs::write(&path, "value: int = 1\n").unwrap();
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    let text = capture.read_file(&path).unwrap();
    let status = std::process::Command::new("python3")
        .args([
            "-c",
            "import pathlib,sys; pathlib.Path(sys.argv[1]).write_text('value: int = \"bad\"\\n')",
        ])
        .arg(&path)
        .status()
        .unwrap();
    assert!(status.success());
    let suite = sifr_frontend::parse_source(text.as_str(), None).unwrap();
    assert!(
        sifr_frontend::compile_module_hir(
            "main",
            &suite,
            &sifr_lowering::ExternalDefs::default(),
            FrontendDiagnosticStyle::Bare
        )
        .is_ok()
    );
    let record = capture.sources().remove(0);
    assert_eq!(record.text, text.as_str());
    assert_eq!(capture.read_file(&path).unwrap(), text);
    assert!(!capture.unchanged());
    assert!(
        matches!(&capture.observations()[0], Observation::File { identity, .. }
        if *identity == record.identity())
    );
}
