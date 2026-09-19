use super::*;
use crate::{DiskSourceProvider, SourceDirEntry, SourceProvider, SourceProviderError, SourceText};
use sifr_diagnostics::{DiagnosticSpan, SourceMap};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Default, Clone)]
struct Memory(Arc<Mutex<BTreeMap<PathBuf, String>>>);
impl Memory {
    fn set(&self, path: &str, source: &str) {
        self.0.lock().unwrap().insert(path.into(), source.into());
    }
    fn remove(&self, path: &str) {
        self.0.lock().unwrap().remove(Path::new(path));
    }
}
impl SourceProvider for Memory {
    fn read_file(&mut self, path: &Path) -> Result<SourceText, SourceProviderError> {
        self.0
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .map(SourceText::new)
            .ok_or_else(|| {
                SourceProviderError::new(
                    crate::SourceProviderErrorKind::FileRead,
                    path,
                    "missing".into(),
                )
            })
    }
    fn read_dir(&mut self, path: &Path) -> Result<Vec<SourceDirEntry>, SourceProviderError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .keys()
            .filter(|file| file.parent() == Some(path))
            .map(|file| SourceDirEntry {
                path: file.clone(),
                file_name: file.file_name().unwrap().into(),
                is_file: true,
                is_dir: false,
            })
            .collect())
    }
    fn is_file(&mut self, path: &Path) -> bool {
        self.0.lock().unwrap().contains_key(path)
    }
    fn is_dir(&mut self, path: &Path) -> bool {
        self.0
            .lock()
            .unwrap()
            .keys()
            .any(|file| file.starts_with(path))
    }
    fn canonicalize(&mut self, path: &Path) -> Result<PathBuf, SourceProviderError> {
        Ok(path.into())
    }
}
fn context() -> SemanticInputs {
    SemanticInputs {
        compiler: "compiled-fixture".into(),
        metadata: "metadata".into(),
        target: "target".into(),
        workspace_and_source_policy: "root".into(),
        package_and_lock: "resolved-lock".into(),
        language_options: "strict".into(),
        diagnostic_policy: "default".into(),
        components: BTreeMap::new(),
        required_external: Default::default(),
        external: BTreeMap::new(),
    }
}
fn inputs() -> ModuleInputs {
    ModuleInputs {
        source: CapturedSource {
            path: "/workspace/main.sifr".into(),
            text: "x: int = 1\n".into(),
        },
        observations: Vec::new(),
        semantic_inputs: context(),
        imported_interfaces: BTreeMap::new(),
    }
}
#[test]
fn dx12_observation_golden_and_order_roundtrip() {
    let mut memory = Memory::default();
    memory.set("/workspace/src/helper.sifr", "answer: int = 42\n");
    let mut capture = CapturingSourceProvider::new(&mut memory);
    assert!(!capture.is_file(Path::new("/workspace/helper.sifr")));
    assert!(capture.is_file(Path::new("/workspace/src/helper.sifr")));
    capture
        .read_file(Path::new("/workspace/src/helper.sifr"))
        .unwrap();
    capture.read_dir(Path::new("/workspace/src")).unwrap();
    capture
        .canonicalize(Path::new("/workspace/src/helper.sifr"))
        .unwrap();
    insta::assert_snapshot!(serde_json::to_string_pretty(capture.observations()).unwrap());
    let encoded = serde_json::to_string(capture.observations()).unwrap();
    let decoded: Vec<Observation> = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, capture.observations());
    assert!(capture.unchanged());
}
#[test]
fn absence_and_higher_priority_candidates() {
    let mut memory = Memory::default();
    memory.set("/src/helper.sifr", "answer: int = 1");
    let writer = memory.clone();
    let mut capture = CapturingSourceProvider::new(&mut memory);
    assert!(!capture.is_file(Path::new("/entry/helper.sifr")));
    assert!(capture.is_file(Path::new("/src/helper.sifr")));
    capture.read_file(Path::new("/src/helper.sifr")).unwrap();
    assert!(capture.unchanged());
    writer.set("/entry/helper.sifr", "answer: int = 2");
    assert!(!capture.unchanged());
    writer.remove("/entry/helper.sifr");
    assert!(capture.unchanged());
    assert!(!capture.is_file(Path::new("/src/missing.sifr")));
    writer.set("/src/missing.sifr", "answer: int = 3");
    assert!(!capture.unchanged());
}
#[test]
fn file_set_and_configuration_invalidation() {
    let mut memory = Memory::default();
    memory.set("/src/a.sifr", "x: int = 1");
    let writer = memory.clone();
    let mut capture = CapturingSourceProvider::new(&mut memory);
    capture.read_dir(Path::new("/src")).unwrap();
    assert!(capture.unchanged());
    writer.set("/src/b.sifr", "x: int = 1");
    assert!(!capture.unchanged());
    writer.remove("/src/b.sifr");
    writer.remove("/src/a.sifr");
    assert!(!capture.unchanged());
    let base = inputs();
    for change in ["policy", "package", "external", "diagnostics"] {
        let mut edited = base.clone();
        match change {
            "policy" => edited.semantic_inputs.workspace_and_source_policy.push('2'),
            "package" => edited.semantic_inputs.package_and_lock.push('2'),
            "external" => {
                edited
                    .semantic_inputs
                    .external
                    .insert("sql-schema".into(), "changed".into());
            }
            _ => edited.semantic_inputs.diagnostic_policy.push('2'),
        }
        assert_ne!(base.identity().unwrap(), edited.identity().unwrap());
    }
}
#[test]
fn analyzes_the_captured_bytes_despite_writer() {
    let mut memory = Memory::default();
    memory.set("/main.sifr", "x: int = 1\n");
    let writer = memory.clone();
    let mut capture = CapturingSourceProvider::new(&mut memory);
    let source = capture.read_file(Path::new("/main.sifr")).unwrap();
    writer.set("/main.sifr", "x: int = \"invalid\"\n");
    assert_eq!(capture.read_file(Path::new("/main.sifr")).unwrap(), source);
    let source_record = capture.sources().remove(0);
    assert_eq!(source_record.text, source.as_str());
    let hir = crate::compile_module_hir(
        "main",
        &crate::parse_source(source.as_str(), None).unwrap(),
        &sifr_lowering::ExternalDefs::default(),
        crate::FrontendDiagnosticStyle::Bare,
    );
    assert!(hir.is_ok());
    assert!(!capture.unchanged());
    assert!(
        matches!(&capture.observations()[0], Observation::File { identity, .. }
        if *identity == source_record.identity())
    );
}
#[test]
fn dx12_i03_canonical_diagnostics_remap_current_sources_and_suggestions() {
    let source = CapturedSource {
        path: "/main.sifr".into(),
        text: "é = 1\r\n".into(),
    };
    let mut diagnostic = crate::diagnostic_with_code(
        "example",
        sifr_diagnostics::DiagnosticCode::INTERNAL_COMPILER_PANIC,
    );
    let span = DiagnosticSpan {
        file: Some("old-name".into()),
        byte_start: 0,
        byte_end: 2,
        line: Some(99),
        column: Some(999),
        end_line: Some(99),
        end_column: Some(1000),
        is_primary: true,
        label: Some("label".into()),
        lines: Vec::new(),
    };
    diagnostic.spans.push(span.clone());
    diagnostic
        .suggestions
        .push(sifr_diagnostics::render::RenderedDiagnosticSuggestion {
            message: "replace".into(),
            applicability: sifr_diagnostics::SuggestionApplicability::MachineApplicable,
            edits: vec![sifr_diagnostics::render::RenderedSuggestionEdit {
                span,
                replacement: "a".into(),
            }],
        });
    let canonical = CanonicalDiagnostic::capture(
        &diagnostic,
        &BTreeMap::from([("old-name".into(), source.clone())]),
    )
    .unwrap();
    let encoded = serde_json::to_vec(&canonical).unwrap();
    let decoded: CanonicalDiagnostic = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(canonical, decoded);
    let mut source_map = SourceMap::new();
    source_map.register_source("unrelated", "allocation before restored sources");
    let rendered = decoded
        .render(
            &BTreeMap::from([(source.identity(), (source.clone(), "current-name".into()))]),
            &mut source_map,
        )
        .unwrap();
    assert_eq!(rendered.spans[0].file.as_deref(), Some("current-name"));
    assert_eq!(rendered.spans[0].column, Some(1));
    assert_eq!(rendered.spans[0].end_column, Some(2));
    assert_eq!(rendered.suggestions[0].edits[0].span, rendered.spans[0]);
    let again = CanonicalDiagnostic::capture(
        &rendered,
        &BTreeMap::from([("current-name".into(), source)]),
    )
    .unwrap();
    assert_eq!(canonical, again);
}
#[test]
fn dx12_family_completeness_and_conservative_interfaces() {
    let input = inputs();
    let mut results: ModuleResults<String, String, CanonicalDiagnostic> = ModuleResults {
        inputs: input.clone(),
        outcome: SourceOutcome::Success,
        resolution: Family::Complete(ResolutionResult {
            observations: vec![],
            sources: vec![input.source.clone()],
            semantic_inputs: context(),
        }),
        interface: Family::Pending,
        checked: Family::Pending,
        diagnostics: Family::Complete(vec![]),
        codegen: Family::Pending,
    };
    assert!(results.check_complete());
    assert!(!results.codegen_complete("codegen"));
    for state in [
        Family::Pending,
        Family::Cancelled,
        Family::Crashed,
        Family::EnvironmentBlocked,
    ] {
        results.diagnostics = state;
        assert!(!results.check_complete());
    }
    results.diagnostics = Family::Complete(vec![]);
    results.checked = Family::Complete(CheckedModule {
        module_identity: "package@1/main".into(),
        input_identity: input.identity().unwrap(),
        hir: "typed-module".into(),
    });
    results.codegen = Family::Complete(CodegenHandoff {
        checked_module_identity: input.identity().unwrap(),
        codegen_identity: "codegen".into(),
        required_specializations: BTreeMap::new(),
    });
    assert!(results.codegen_complete("codegen"));
    assert!(!results.codegen_complete("other-options"));
    results.inputs.source.text.push(' ');
    assert!(!results.codegen_complete("codegen"));
    results.inputs = input.clone();
    results.outcome = SourceOutcome::DeterministicErrors;
    assert!(results.check_complete());
    let bytes = serde_json::to_vec(&results).unwrap();
    let decoded: ModuleResults<String, String, CanonicalDiagnostic> =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(results, decoded);
    let interface = SemanticInterface {
        module_identity: "package@1/main".into(),
        input_identity: input.identity().unwrap(),
        exports: "exports",
    };
    let mut changed = input.clone();
    changed
        .source
        .text
        .push_str("# even an unproven body-only edit invalidates\n");
    let edited = SemanticInterface {
        input_identity: changed.identity().unwrap(),
        ..interface.clone()
    };
    assert_ne!(interface.identity().unwrap(), edited.identity().unwrap());
    let mut importer = inputs();
    importer
        .imported_interfaces
        .insert("dependency".into(), interface.identity().unwrap());
    let prior = importer.identity().unwrap();
    importer
        .imported_interfaces
        .insert("dependency".into(), edited.identity().unwrap());
    assert_ne!(prior, importer.identity().unwrap());
}
#[test]
fn dx12_failed_environment_observation_cannot_publish() {
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    assert!(
        capture
            .read_file(Path::new("/sifr-dx12-deliberately-absent/input.sifr"))
            .is_err()
    );
    assert!(!capture.unchanged());
}

#[test]
fn dx12_declared_external_context_must_be_complete() {
    let mut semantic = context();
    semantic.required_external.extend([
        "sql-schema".into(),
        "python-abi".into(),
        "interop-declaration".into(),
    ]);
    assert!(!semantic.complete());
    for name in semantic.required_external.clone() {
        semantic
            .external
            .insert(name, "resolved-input-identity".into());
    }
    assert!(semantic.complete());
    let prior = semantic.identity().unwrap();
    semantic
        .external
        .insert("python-abi".into(), "changed-abi".into());
    assert_ne!(prior, semantic.identity().unwrap());
}
#[test]
fn dx12_completed_deterministic_source_error_diagnostics_roundtrip() {
    let source = CapturedSource {
        path: "/bad.sifr".into(),
        text: "value: int = \"bad\"\n".into(),
    };
    let suite = crate::parse_source(&source.text, None).unwrap();
    let errors = crate::compile_module_hir_with_source(
        "bad",
        &suite,
        &sifr_lowering::ExternalDefs::default(),
        crate::FrontendDiagnosticStyle::Bare,
        Some(crate::FrontendSourceContext {
            display_path: "/bad.sifr",
            source: &source.text,
        }),
    )
    .err()
    .expect("source must fail checking");
    assert!(!errors.is_empty());
    let sources = BTreeMap::from([("/bad.sifr".into(), source.clone())]);
    let facts = errors
        .iter()
        .map(|error| CanonicalDiagnostic::capture(error, &sources).unwrap())
        .collect::<Vec<_>>();
    let ready = Family::Complete(facts);
    let decoded: Family<Vec<CanonicalDiagnostic>> =
        serde_json::from_slice(&serde_json::to_vec(&ready).unwrap()).unwrap();
    assert_eq!(ready, decoded);
    let current = BTreeMap::from([(source.identity(), (source, "/bad.sifr".into()))]);
    let restored: Vec<_> = decoded
        .ready()
        .unwrap()
        .iter()
        .map(|fact| fact.render(&current, &mut SourceMap::new()).unwrap())
        .collect();
    assert_eq!(restored, errors);
}
