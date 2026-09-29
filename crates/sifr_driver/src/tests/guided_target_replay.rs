use sifr_diagnostics::{
    DiagnosticCode, DiagnosticEnvelope, render_compact_envelope, render_human_envelope,
    render_json_envelope,
};
use std::path::Path;

#[test]
fn diagnostic_renderer() {
    let corpus =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../verification/fuzz/corpus/diagnostics");
    for seed in [
        "minimal_type_mismatch.json",
        "rich_type_mismatch.json",
        "edge_type_mismatch.json",
    ] {
        let bytes = std::fs::read(corpus.join(seed)).expect("pinned diagnostic seed exists");
        let envelope: DiagnosticEnvelope =
            serde_json::from_slice(&bytes).expect("pinned seed is a structured envelope");
        assert_eq!(envelope.version, 1, "{seed}");
        assert!(!envelope.diagnostics.is_empty(), "{seed}");
        assert!(
            envelope.diagnostics.iter().all(|diagnostic| {
                diagnostic.code == DiagnosticCode::TYPE_MISMATCH.code()
                    && diagnostic.url == DiagnosticCode::TYPE_MISMATCH.docs_url()
            }),
            "{seed}"
        );

        let json = render_json_envelope(&envelope).expect("JSON renderer succeeds");
        let human = render_human_envelope(&envelope);
        let compact = render_compact_envelope(&envelope);
        assert_eq!(
            serde_json::from_str::<DiagnosticEnvelope>(&json).expect("JSON replay parses"),
            envelope,
            "{seed}"
        );
        assert!(human.contains("error[SIFR-TYPE-0002]"), "{seed}");
        assert!(compact.contains("E SIFR-TYPE-0002 "), "{seed}");
        assert_eq!(render_human_envelope(&envelope), human, "{seed}");
        assert_eq!(render_compact_envelope(&envelope), compact, "{seed}");
        assert_eq!(render_json_envelope(&envelope).unwrap(), json, "{seed}");

        if seed == "minimal_type_mismatch.json" {
            assert!(envelope.diagnostics[0].spans.is_empty());
            assert!(envelope.diagnostics[0].suggestions.is_empty());
        } else if seed == "rich_type_mismatch.json" {
            assert!(human.contains("related span"));
            assert!(human.contains("suggestion: replace the value"));
            assert!(human.contains("expected by annotation"));
        }
    }
}

#[test]
fn project_graph() {
    use crate::guided_project_graph::{ReplayOutcome, project_tree, replay, replay_exported_tree};
    let corpus =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../verification/fuzz/corpus/project_graph");
    let mut distinct_trees = std::collections::BTreeSet::new();
    for seed in ["isolated", "star", "chain", "cycle", "manifest_invalid"] {
        let input = std::fs::read(corpus.join(seed)).expect("pinned project seed exists");
        let tree = project_tree(&input).expect("bounded project seed");
        assert_eq!(tree.files.len(), 16);
        distinct_trees.insert(format!("{tree:?}"));
        let first = replay(&input);
        assert_ne!(first, ReplayOutcome::InvalidInput, "{seed}");
        assert_eq!(first, replay(&input), "stable byte replay: {seed}");
        let exported = tempfile::tempdir().expect("export root");
        tree.write_to(exported.path()).expect("export project tree");
        for (relative, source) in &tree.files {
            assert_eq!(
                std::fs::read_to_string(exported.path().join(relative)).expect("exported file"),
                *source,
                "{seed}: {}",
                relative.display()
            );
        }
        assert_eq!(
            first,
            replay_exported_tree(&input, exported.path()),
            "disk replay: {seed}"
        );
    }
    assert_eq!(distinct_trees.len(), 5, "seeds must mutate graph edges");
    assert_eq!(replay(&[0x80, 0, 0, 0]), ReplayOutcome::ManifestDiagnostic);
    let undeclared = replay(&[0, 1, 0, 0]);
    let declared = replay(&[1, 1, 0, 0]);
    assert!(matches!(undeclared, ReplayOutcome::ProjectDiagnostics(_)));
    assert_eq!(declared, ReplayOutcome::Accepted);
    let minimized = std::fs::read(corpus.join("cycle")).expect("cycle seed exists");
    assert_eq!(minimized.len(), 4, "all graph decisions fit in four bytes");
}
