use super::AnalysisHost;
use crate::{FrontendInput, ProjectRoot};
use sifr_frontend::{FileId, FrontendMode, SourcePath, SourceText};
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
fn single_file_analysis_resolves_embedded_stdlib_imports() {
    let mut host = AnalysisHost::open_single_file(single_file_input(STDLIB_IMPORT_SAMPLE))
        .expect("single-file analysis host should load with stdlib definitions");
    let file = host.files()[0];

    assert_stdlib_import_resolves(&mut host, file);
}

#[test]
fn project_analysis_resolves_embedded_stdlib_imports() {
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
