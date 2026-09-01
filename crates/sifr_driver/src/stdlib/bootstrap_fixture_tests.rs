use super::*;
use std::path::PathBuf;

fn fixture_source(module: &str, source: &str, kind: LoadedStdlibSourceKind) -> LoadedStdlibSource {
    let stdlib_root =
        std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"))
            .expect("development stdlib root should resolve");
    LoadedStdlibSource {
        module: module.to_string(),
        source: source.to_string(),
        path: stdlib_root.join(format!("{}.sifr", module.replace('.', "/"))),
        kind,
    }
}

fn compile_fixture_sources(
    sources: &[LoadedStdlibSource],
) -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    let sysroot = sifr_sysroot::resolve_sysroot(None).expect("development sysroot should resolve");
    compile_stdlib_sources_with_sysroot(sources, sysroot)
}

fn fixture_diagnostics(sources: &[LoadedStdlibSource]) -> Vec<RenderedDiagnostic> {
    match compile_fixture_sources(sources) {
        Ok(_) => panic!("fixture should fail stdlib bootstrap"),
        Err(diagnostics) => diagnostics,
    }
}

#[test]
fn private_stdlib_imports_resolve_only_from_compiled_source_exports() {
    let sources = [
        fixture_source(
            "_sifr.fixture",
            "def existing(value: int) -> int:\n    return value\n",
            LoadedStdlibSourceKind::PrivateDeclaration,
        ),
        fixture_source(
            "sifr.fixture",
            "from _sifr.fixture import existing\n\ndef forwarded(value: int) -> int:\n    return existing(value)\n",
            LoadedStdlibSourceKind::Public,
        ),
    ];

    let compiled = compile_fixture_sources(&sources).expect("source-backed import should compile");
    assert!(
        compiled
            .defs
            .functions
            .get("sifr.fixture")
            .is_some_and(|functions| functions.contains_key("forwarded"))
    );
    assert!(
        compiled
            .code
            .transitive_deps
            .get("sifr.fixture")
            .is_some_and(|deps| deps.contains("_sifr.fixture"))
    );
}

#[test]
fn missing_private_stdlib_member_is_a_structured_bootstrap_failure() {
    let sources = [
        fixture_source(
            "_sifr.fixture",
            "def existing(value: int) -> int:\n    return value\n",
            LoadedStdlibSourceKind::PrivateDeclaration,
        ),
        fixture_source(
            "sifr.fixture",
            "from _sifr.fixture import absent\n",
            LoadedStdlibSourceKind::Public,
        ),
    ];

    let diagnostics = fixture_diagnostics(&sources);
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE.code())
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("_sifr.fixture") && diagnostic.message.contains("absent")
    }));
}

#[test]
fn missing_private_stdlib_module_is_a_structured_bootstrap_failure() {
    let sources = [fixture_source(
        "sifr.fixture",
        "from _sifr.missing import absent\n",
        LoadedStdlibSourceKind::Public,
    )];

    let diagnostics = fixture_diagnostics(&sources);
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE.code())
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("_sifr.missing"))
    );
}
