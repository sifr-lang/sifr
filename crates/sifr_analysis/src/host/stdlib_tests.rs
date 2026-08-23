use super::AnalysisHost;
use crate::{FrontendInput, ProjectRoot, SymbolBucketKind, SymbolBucketReadinessState};
use sifr_frontend::{FileId, FrontendMode, SourceOrigin, SourcePath, SourceText};
use sifr_syntax::TextPosition;
use std::path::PathBuf;

const STDLIB_IMPORT_SAMPLE: &str = "from sifr.random import randint\n\n\
def main() -> int:\n    value = randint(0, 100)\n    mismatch: int = \"not int\"\n    return mismatch\n";

fn single_file_input(source: &str) -> FrontendInput {
    FrontendInput {
        path: SourcePath::new("main.sifr"),
        source: SourceText::new(source),
        mode: FrontendMode::SingleFile,
    }
}

fn temp_project_dir(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "sifr_analysis_{name}_{}_{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp project should be created");
    dir
}

fn assert_stdlib_import_resolves(host: &mut AnalysisHost, file: FileId) {
    let diagnostics = host
        .diagnostics(file)
        .expect("diagnostics should query")
        .into_value();
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();

    assert!(
        !codes.contains(&"SIFR-IMPORT-0002"),
        "stdlib import should resolve in single-file analysis: {codes:?}"
    );
    assert!(
        !codes.contains(&"SIFR-NAME-0002"),
        "stdlib import should expose imported names in single-file analysis: {codes:?}"
    );
    assert!(
        codes.contains(&"SIFR-TYPE-0002"),
        "sample should reach semantic type checking: {codes:?}"
    );
}

#[test]
fn single_file_analysis_resolves_sysroot_stdlib_imports() {
    let mut host = AnalysisHost::open_single_file(single_file_input(STDLIB_IMPORT_SAMPLE))
        .expect("single-file analysis host should load with stdlib definitions");
    let file = host.files()[0];

    assert_stdlib_import_resolves(&mut host, file);
}

#[test]
fn project_analysis_resolves_sysroot_stdlib_imports() {
    let dir = temp_project_dir("stdlib_project");
    let entrypoint = dir.join("main.sifr");
    std::fs::write(&entrypoint, STDLIB_IMPORT_SAMPLE).expect("project source should be written");
    let root = ProjectRoot {
        root: SourcePath::new(&dir),
        entrypoint: SourcePath::new(&entrypoint),
    };

    let mut host =
        AnalysisHost::open_project(&root).expect("project analysis host should load stdlib defs");
    let file = host
        .document_file_for_path(&entrypoint)
        .expect("entrypoint should be indexed");

    assert_stdlib_import_resolves(&mut host, file);
    std::fs::remove_dir_all(dir).expect("temp project should be removed");
}

#[test]
fn analysis_source_map_tracks_public_and_private_sysroot_origins() {
    let host = AnalysisHost::open_single_file(single_file_input(STDLIB_IMPORT_SAMPLE))
        .expect("single-file analysis host should load with stdlib tooling sources");

    let source_map = host
        .context()
        .expect("context should be loaded")
        .source_map();

    assert!(
        source_map
            .files
            .iter()
            .any(|file| file.origin == SourceOrigin::UserSource
                && file.module_name.as_deref() == Some("main"))
    );
    assert!(source_map.files.iter().any(|file| {
        file.origin == SourceOrigin::SysrootPublicStdlib
            && file.module_name.as_deref() == Some("sifr.random")
    }));
    assert!(source_map.files.iter().any(|file| {
        file.origin == SourceOrigin::SysrootPrivateDeclaration
            && file.module_name.as_deref() == Some("_sifr.math")
    }));
    assert_eq!(host.files().len(), 1);
    assert!(host.all_files().len() > host.files().len());
}

#[test]
fn stdlib_symbol_bucket_is_available_without_private_declarations() {
    let mut host = AnalysisHost::open_single_file(single_file_input("def main():\n    return 1\n"))
        .expect("single-file analysis host should load");
    let file = host.files()[0];

    let completion = host
        .completion(
            file,
            &TextPosition {
                line: 0,
                character: 0,
            },
        )
        .expect("completion should query")
        .into_value();
    let readiness = host
        .symbol_bucket_readiness()
        .expect("symbol bucket readiness should query");
    let stdlib = readiness
        .iter()
        .find(|bucket| bucket.id.kind == SymbolBucketKind::Stdlib)
        .expect("stdlib bucket should exist");

    assert_eq!(stdlib.state, SymbolBucketReadinessState::Exact);
    assert!(stdlib.entry_count > 0);
    assert!(completion.items.iter().any(|item| item.label == "randint"));
    assert!(
        !completion
            .items
            .iter()
            .any(|item| item.detail.as_deref() == Some("_sifr.random"))
    );
}

#[test]
fn definition_for_public_stdlib_import_lands_in_sysroot_source() {
    let mut host = AnalysisHost::open_single_file(single_file_input(STDLIB_IMPORT_SAMPLE))
        .expect("single-file analysis host should load");
    let file = host.files()[0];

    let locations = host
        .definition(
            file,
            &TextPosition {
                line: 0,
                character: 25,
            },
        )
        .expect("definition should query")
        .into_value();

    assert_eq!(locations.len(), 1);
    let location = &locations[0];
    let path = host
        .path_for_file(location.file)
        .expect("stdlib file path should be mapped");
    assert!(path.ends_with("stdlib/sifr/random.sifr"));
    assert!(
        host.source_text_for_file(location.file)
            .expect("stdlib source should be mapped")
            .contains("def randint")
    );
    assert!(location.range.is_some());
}

#[test]
fn definition_inside_public_stdlib_can_link_to_private_declaration_file() {
    let mut host = AnalysisHost::open_single_file(single_file_input(STDLIB_IMPORT_SAMPLE))
        .expect("single-file analysis host should load");
    let source_map = host
        .context()
        .expect("context should be loaded")
        .source_map();
    let math_file = source_map
        .files
        .iter()
        .find(|file| {
            file.origin == SourceOrigin::SysrootPublicStdlib
                && file.module_name.as_deref() == Some("sifr.math")
        })
        .expect("sifr.math public stdlib source should be loaded")
        .id;

    let locations = host
        .definition(
            math_file,
            &TextPosition {
                line: 1,
                character: 24,
            },
        )
        .expect("definition should query inside stdlib source")
        .into_value();

    assert_eq!(locations.len(), 1);
    let path = host
        .path_for_file(locations[0].file)
        .expect("private declaration path should be mapped");
    assert!(path.ends_with("stdlib/_sifr/math.sifr"));
}
