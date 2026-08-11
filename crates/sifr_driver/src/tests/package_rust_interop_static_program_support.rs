use super::*;

const STATIC_PROGRAM_POSITIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/static_program_arena_bridge/positive/static_program_constructs_and_projects_arena.sifr"
);
const STATIC_PROGRAM_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/static_program_arena_bridge/negative/corrupt_static_program_envelope_rejected.sifr"
);

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-runtime-observed"]
fn test_static_program_constructs_and_projects_arena() {
    let package_root = copied_scenario(
        "static_program_arena_bridge",
        "static_program_runtime",
        "rust_interop_static_program_positive",
    );
    rebase_sifr_runtime_dependency(&package_root);
    let pristine_entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "static_program_runtime");
    let pristine_errors = check_package_project(&pristine_entrypoint);
    assert!(
        pristine_errors.is_empty(),
        "source-layout static program scenario must pass checking: {pristine_errors:#?}"
    );
    let pristine_artifact = build_cached_package_project(&pristine_entrypoint)
        .expect("source-layout static program scenario should build");
    let pristine_source = generated_main_source(pristine_artifact.binary_path());
    let pristine_identity = static_program_identity_declaration(&pristine_source);

    install_evidence_source(
        &package_root,
        &format!(
            "{STATIC_PROGRAM_POSITIVE}\n\ndef main() -> Result[None, StaticProgramError | RustPanicError]:\n    try:\n        print(verify_static_program_constructs_and_projects_arena())\n    except StaticProgramError as error:\n        raise error\n    except RustPanicError as error:\n        raise error\n    return None\n"
        ),
    );
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "static_program_runtime");
    let evidence_artifact =
        build_cached_package_project(&entrypoint).expect("installed evidence should build");
    let evidence_source = generated_main_source(evidence_artifact.binary_path());
    assert_eq!(
        static_program_identity_declaration(&evidence_source),
        pristine_identity,
        "source-layout and installed evidence must retain one static-program identity"
    );
    let repeated_artifact = build_cached_package_project(&entrypoint)
        .expect("repeated evidence build should hit cache");
    assert_eq!(
        repeated_artifact.binary_path(),
        evidence_artifact.binary_path(),
        "unchanged static-program input must retain its generated-project cache identity"
    );
    assert_eq!(
        generated_main_source(repeated_artifact.binary_path()),
        evidence_source,
        "repeated generated output must be byte-identical"
    );
    let output = std::process::Command::new(evidence_artifact.binary_path())
        .output()
        .expect("static-program evidence binary should run");
    assert!(
        output.status.success(),
        "static-program evidence binary should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Ok(\"program=sealed;integer=123;fixed=42;bytes=11;tags=alpha,beta;lookup=left:-7,right:9007199254740991;active=0\")"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

fn generated_main_source(binary_path: &Path) -> String {
    let project_root = binary_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("cached binary path should be <project>/target/release/<bin>");
    std::fs::read_to_string(project_root.join("src/main.rs"))
        .expect("generated main source should be retained")
}

fn static_program_identity_declaration(source: &str) -> &str {
    source
        .lines()
        .find(|line| line.contains("__SIFR_STATIC_PROGRAM_IDENTITY_"))
        .expect("generated source should contain a static-program identity")
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-runtime-observed"]
fn test_corrupt_static_program_envelope_rejected() {
    let package_root = copied_scenario(
        "static_program_arena_bridge",
        "static_program_runtime",
        "rust_interop_static_program_negative",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(
        &package_root,
        &format!(
            "{STATIC_PROGRAM_NEGATIVE}\n\ndef main() -> Result[None, RustPanicError]:\n    try:\n        print(verify_corrupt_static_program_envelope_rejected())\n    except RustPanicError as error:\n        raise error\n    return None\n"
        ),
    );
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "static_program_runtime");
    assert_eq!(
        run_built_package(&entrypoint),
        "Ok(\"static program format version mismatch;active=0\")"
    );
    let _ = std::fs::remove_dir_all(package_root);
}
