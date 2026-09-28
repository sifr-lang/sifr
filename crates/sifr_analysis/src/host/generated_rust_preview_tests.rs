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
    let mut host = AnalysisHost::open_single_file(
        &sifr_compiler_services::CompilerContext::for_test_tokens(
            crate::compiled_input_tokens(),
            "sifr_analysis-tests",
        ),
        single_file_input("def main():\n    return 1\n"),
    )
    .expect("host should load");
    let file = host.files()[0];

    let preview = host
        .generated_rust_preview(file, None)
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
    let mut host = AnalysisHost::open_single_file(
        &sifr_compiler_services::CompilerContext::for_test_tokens(
            crate::compiled_input_tokens(),
            "sifr_analysis-tests",
        ),
        single_file_input(source),
    )
    .expect("host should load");
    let file = host.files()[0];

    let preview = host
        .generated_rust_preview(file, None)
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

#[test]
fn generated_rust_preview_matches_compiler_frontend_product() {
    let source = "from sifr.random import randint\n\ndef main():\n    print(randint(1, 2))\n";
    let compiler = sifr_compiler_services::CompilerContext::for_test_tokens(
        crate::compiled_input_tokens(),
        "sifr_analysis-tests",
    );
    let mut host = AnalysisHost::open_single_file(&compiler, single_file_input(source)).unwrap();
    let file = host.files()[0];
    let preview = host
        .generated_rust_preview(file, None)
        .unwrap()
        .into_value();
    let sifr_compiler_services::editor::PreviewResult::Success {
        rust,
        source_map_files,
    } = sifr_compiler_services::editor::generated_rust_preview(&compiler, source)
    else {
        panic!("compiler service must compile the same C01 product");
    };
    assert_eq!(preview.rust.as_deref(), Some(rust.as_str()));
    assert_eq!(preview.source_map_files, source_map_files);
}

#[test]
fn generated_rust_preview_rejects_direct_rust_interop_without_package() {
    let source = "@rust(crc32fast.hash, panic=trusted_no_panic)\ndef crc32(data: bytes) -> uint32:\n    zero: uint32 = 0\n    return zero\n\ndef main():\n    print(crc32(b\"abc\"))\n";
    let compiler = sifr_compiler_services::CompilerContext::for_test_tokens(
        crate::compiled_input_tokens(),
        "sifr_analysis-tests",
    );
    let mut host = AnalysisHost::open_single_file(&compiler, single_file_input(source)).unwrap();
    let file = host.files()[0];
    let preview = host
        .generated_rust_preview(file, None)
        .unwrap()
        .into_value();
    assert!(preview.rust.is_none());
    assert!(preview.source_map_files.is_empty());
    assert!(preview.unavailable_reason.is_some());
    assert!(matches!(
        sifr_compiler_services::editor::generated_rust_preview(&compiler, source),
        sifr_compiler_services::editor::PreviewResult::Unavailable {
            diagnostic_count: 1
        }
    ));
}
