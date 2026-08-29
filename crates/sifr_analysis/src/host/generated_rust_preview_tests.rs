use super::*;
use crate::{FrontendInput, SourceText};
use sifr_frontend::{FrontendMode, SourceOrigin, SourcePath};

fn single_file_input(source: &str) -> FrontendInput {
    FrontendInput {
        path: SourcePath::new("main.sifr"),
        source: SourceText::new(source),
        mode: FrontendMode::SingleFile,
    }
}

#[test]
fn generated_rust_preview_tracks_compiler_synthetic_source_map_entry() {
    let mut host = AnalysisHost::open_single_file(single_file_input("def main():\n    return 1\n"))
        .expect("host should load");
    let file = host.files()[0];

    let preview = host
        .generated_rust_preview(file)
        .expect("generated Rust preview should query")
        .into_value();

    assert!(preview.source_map_files.iter().any(|file| {
        file.origin == SourceOrigin::CompilerSynthetic
            && file.path == "src/main.rs"
            && file.source.contains("fn main")
    }));
}

#[test]
fn generated_rust_preview_tracks_generated_support_source_map_entry() {
    let source = "\
from sifr.random import randint

def main():
    try:
        x: int = randint(1, 2)
        print(x)
    except ValueError:
        print(0)
";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];

    let preview = host
        .generated_rust_preview(file)
        .expect("generated Rust preview should query")
        .into_value();

    assert!(preview.source_map_files.iter().any(|file| {
        file.origin == SourceOrigin::GeneratedSupport
            && file.path == "src/main.rs#stdlib-preamble"
            && file.source.contains("// --- stdlib: sifr.random ---")
    }));
    assert!(preview.source_map_files.iter().any(|file| {
        file.origin == SourceOrigin::CompilerSynthetic
            && file.path == "src/main.rs"
            && file.source.contains("fn main")
    }));
}
