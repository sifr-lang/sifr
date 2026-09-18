use crate::{AnalysisHost, DocumentVersion, FrontendMode, SourcePath, SourceText};
use std::sync::Arc;

#[test]
fn dx11_overlapping_snapshots_pin_text_toolchain_and_release_after_close() {
    let compiler = sifr_driver::CompilerContext::for_test_tokens(
        crate::compiled_input_tokens(),
        "dx11-snapshots",
    );
    let mut host = AnalysisHost::open_single_file_overlay(
        &compiler,
        SourcePath::new("main.sifr"),
        None,
        DocumentVersion::new(1),
        SourceText::new("def main():\n    value: int = 1\n"),
        FrontendMode::SingleFile,
    )
    .unwrap();
    let other = AnalysisHost::open_single_file_overlay(
        &compiler,
        SourcePath::new("other.sifr"),
        None,
        DocumentVersion::new(1),
        SourceText::new("def main():\n    value: int = 1\n"),
        FrontendMode::SingleFile,
    )
    .unwrap();
    let old = host.snapshot();
    assert!(!other.is_snapshot_current(&old));
    drop(other);
    assert!(old.has_unsaved_overlays());
    let weak = Arc::downgrade(&old.workspace().overlays);
    let same = host.snapshot();
    assert!(Arc::ptr_eq(
        &old.workspace().overlays,
        &same.workspace().overlays
    ));
    host.update_document(
        host.files()[0],
        DocumentVersion::new(2),
        SourceText::new("def main():\n    value: int = 2\n"),
    )
    .unwrap();
    let new = host.snapshot();
    assert!(
        old.workspace().source_map.as_ref().unwrap().files[0]
            .source
            .as_str()
            .contains("= 1")
    );
    assert!(
        new.workspace().source_map.as_ref().unwrap().files[0]
            .source
            .as_str()
            .contains("= 2")
    );
    assert_eq!(
        new.workspace().overlays[0].source.as_str(),
        new.workspace().source_map.as_ref().unwrap().files[0]
            .source
            .as_str()
    );
    assert!(!host.is_snapshot_current(&old));
    assert!(!old.matches_compiler(&compiler.without_cached_metadata()));
    host.session.reload().unwrap();
    assert!(
        host.session.snapshot().source_map.as_ref().unwrap().files[0]
            .source
            .as_str()
            .contains("= 2")
    );
    drop(host);
    drop(old);
    assert!(weak.upgrade().is_some());
    drop(same);
    assert!(weak.upgrade().is_none());
    assert!(new.matches_compiler(&compiler));
}
