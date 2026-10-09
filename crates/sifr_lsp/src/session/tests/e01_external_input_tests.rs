use crate::session::Session;
use std::path::{Path, PathBuf};

fn fixture(root: &Path, name: &str) -> (PathBuf, String) {
    std::fs::create_dir_all(root.join("src")).expect("create source directory");
    std::fs::write(root.join("src/lib.rs"), "").expect("write Cargo library marker");
    std::fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n\n[workspace]\n"),
    ).expect("write Cargo manifest");
    std::fs::write(
        root.join("Cargo.lock"),
        format!("version = 4\n\n[[package]]\nname = \"{name}\"\nversion = \"0.0.0\"\n"),
    )
    .expect("write Cargo lock");
    std::fs::write(root.join("sifr.toml"), "[package]\nname = \"lsp-external\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n").expect("write Sifr manifest");
    let path = root.join("src/main.sifr");
    let source = "def main() -> int:\n    return 1\n";
    std::fs::write(&path, source).expect("write Sifr source");
    let uri = url::Url::from_file_path(&path)
        .expect("file URI")
        .to_string();
    (path, uri)
}

fn open(session: &mut Session, uri: String) {
    session
        .open_document(
            uri,
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            "def main() -> int:\n    return 1\n".to_string(),
        )
        .expect("open document");
}

#[test]
fn unchanged_inputs_keep_the_owning_root_warm() {
    let temp = tempfile::tempdir().expect("temporary root");
    let (path, uri) = fixture(temp.path(), "unchanged-root");
    let mut session = Session::new();
    open(&mut session, uri.clone());
    session
        .python_declaration_snapshot(&uri)
        .expect("initial declaration status");
    let generation = session
        .external_inputs
        .generation(temp.path())
        .expect("root generation");
    let builds = session.python_declarations.snapshot_builds();
    session
        .python_declaration_snapshot(&uri)
        .expect("warm declaration status");
    session.observe_external_inputs_for_path(&path);
    assert_eq!(
        session.external_inputs.generation(temp.path()),
        Some(generation)
    );
    assert_eq!(session.python_declarations.snapshot_builds(), builds);
    assert!(session.python_declarations.has_entry(temp.path()));
}

#[test]
fn changed_and_renamed_inputs_advance_only_the_owning_root() {
    let temp = tempfile::tempdir().expect("temporary roots");
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    let (first_path, first_uri) = fixture(&first, "first-root");
    let (second_path, second_uri) = fixture(&second, "second-root");
    let mut session = Session::new();
    open(&mut session, first_uri.clone());
    open(&mut session, second_uri.clone());
    session
        .python_declaration_snapshot(&first_uri)
        .expect("first status");
    session
        .python_declaration_snapshot(&second_uri)
        .expect("second status");
    let first_generation = session
        .external_inputs
        .generation(&first)
        .expect("first generation");
    let second_generation = session
        .external_inputs
        .generation(&second)
        .expect("second generation");
    std::fs::write(first.join("Cargo.lock"), "version = 4\n# changed\n").expect("change lock");
    session.observe_external_inputs_for_path(&first_path);
    assert!(
        session
            .external_inputs
            .generation(&first)
            .expect("new first generation")
            > first_generation
    );
    assert_eq!(
        session.external_inputs.generation(&second),
        Some(second_generation)
    );
    assert!(!session.python_declarations.has_entry(&first));
    assert!(session.python_declarations.has_entry(&second));
    std::fs::create_dir_all(first.join("src/python_bridges")).expect("create bridge directory");
    let old = first.join("src/python_bridges/old.py");
    std::fs::write(&old, "value = 1\n").expect("write bridge");
    session.observe_external_inputs_for_path(&first_path);
    let before_rename = session
        .external_inputs
        .generation(&first)
        .expect("bridge generation");
    std::fs::rename(old, first.join("src/python_bridges/new.py")).expect("rename bridge");
    session.observe_external_inputs_for_path(&first_path);
    assert!(
        session
            .external_inputs
            .generation(&first)
            .expect("renamed generation")
            > before_rename
    );
    assert_eq!(
        session.external_inputs.generation(&second),
        Some(second_generation)
    );
    session.observe_external_inputs_for_path(&second_path);
    assert!(session.python_declarations.has_entry(&second));
}

#[test]
fn absent_empty_deleted_and_unreadable_inputs_invalidate_negative_and_positive_cache_entries() {
    let temp = tempfile::tempdir().expect("temporary root");
    let (path, uri) = fixture(temp.path(), "state-root");
    let mut session = Session::new();
    open(&mut session, uri.clone());
    session
        .python_declaration_snapshot(&uri)
        .expect("initial status");
    let artifact = temp.path().join(sifr_package::PYTHON_BINDINGS_FILE);
    let initial = session
        .external_inputs
        .generation(temp.path())
        .expect("initial generation");
    std::fs::write(&artifact, "").expect("create empty artifact");
    session.observe_external_inputs_for_path(&path);
    assert!(
        session
            .external_inputs
            .generation(temp.path())
            .expect("empty generation")
            > initial
    );
    assert!(!session.python_declarations.has_entry(temp.path()));
    assert!(!session.python_declarations.has_environment(temp.path()));
    let builds = session.python_declarations.snapshot_builds();
    session
        .python_declaration_snapshot(&uri)
        .expect("negative artifact status");
    assert!(session.python_declarations.snapshot_builds() > builds);
    assert!(session.python_declarations.has_entry(temp.path()));
    assert!(session.python_declarations.has_environment(temp.path()));
    std::fs::write(
        &artifact,
        "{\"schema_version\":1,\"environment_digest\":\"stale\",\"bindings\":[]}\n",
    )
    .expect("write artifact");
    session.observe_external_inputs_for_path(&path);
    assert!(!session.python_declarations.has_entry(temp.path()));
    session
        .python_declaration_snapshot(&uri)
        .expect("revalidated status");
    std::fs::remove_file(&artifact).expect("delete artifact");
    session.observe_external_inputs_for_path(&path);
    assert!(!session.python_declarations.has_entry(temp.path()));
    session
        .python_declaration_snapshot(&uri)
        .expect("positive missing status");
    std::fs::create_dir(&artifact).expect("make artifact unreadable as a file");
    session.observe_external_inputs_for_path(&path);
    assert!(!session.python_declarations.has_entry(temp.path()));
    let unreadable_generation = session
        .external_inputs
        .generation(temp.path())
        .expect("unreadable generation");
    session.observe_external_inputs_for_path(&path);
    assert!(
        session
            .external_inputs
            .generation(temp.path())
            .expect("retry generation")
            > unreadable_generation
    );
    std::fs::remove_dir(&artifact).expect("remove unreadable artifact");
    session.observe_external_inputs_for_path(&path);
    session
        .python_declaration_snapshot(&uri)
        .expect("restored positive status");
    let manifest = temp.path().join("sifr.toml");
    let manifest_bytes = std::fs::read(&manifest).expect("read manifest");
    let before_delete = session
        .external_inputs
        .generation(temp.path())
        .expect("before manifest deletion");
    std::fs::remove_file(&manifest).expect("delete manifest");
    session.observe_external_inputs_for_path(&path);
    let deleted_generation = session
        .external_inputs
        .generation(temp.path())
        .expect("manifest deletion generation");
    assert!(deleted_generation > before_delete);
    assert!(!session.python_declarations.has_entry(temp.path()));
    std::fs::write(&manifest, &manifest_bytes).expect("restore manifest");
    session.observe_external_inputs_for_path(&path);
    assert!(
        session
            .external_inputs
            .generation(temp.path())
            .expect("manifest restoration generation")
            > deleted_generation
    );
    std::fs::write(&manifest, "[package]\nname = [\n").expect("break manifest");
    let invalid = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("invalid manifest diagnostics");
    assert!(
        !invalid.is_empty(),
        "invalid manifest must replace cached analysis success"
    );
    std::fs::write(&manifest, manifest_bytes).expect("repair manifest");
    let repaired = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("repaired manifest diagnostics");
    assert!(
        repaired.is_empty(),
        "repaired manifest must replace cached analysis failure: {repaired:?}"
    );
}

#[test]
fn environment_bridge_and_certification_drift_revalidate_without_source_edits() {
    let temp = tempfile::tempdir().expect("temporary root");
    let (path, uri) = fixture(temp.path(), "environment-root");
    let mut session = Session::new();
    open(&mut session, uri.clone());
    session
        .python_declaration_snapshot(&uri)
        .expect("initial status");
    let source = std::fs::read(&path).expect("source bytes");
    let mut previous = session
        .external_inputs
        .generation(temp.path())
        .expect("initial generation");
    for (relative, content) in [
        (
            "pyproject.toml",
            "[project]\nname = \"environment-root\"\nversion = \"0.0.0\"\n",
        ),
        ("uv.lock", "version = 1\n"),
        (
            "src/python_bridges/bridge.py",
            "def value():\n    return 1\n",
        ),
        ("src/python_bridges/__sifr_inventory__.json", "{}\n"),
        (sifr_package::PYTHON_CERTIFICATIONS_FILE, "{}\n"),
    ] {
        let target = temp.path().join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).expect("create input directory");
        }
        std::fs::write(&target, content).expect("write external input");
        session.observe_external_inputs_for_path(&path);
        let next = session
            .external_inputs
            .generation(temp.path())
            .expect("advanced generation");
        assert!(
            next > previous,
            "{relative} must advance the root generation"
        );
        assert!(!session.python_declarations.has_entry(temp.path()));
        previous = next;
        let builds = session.python_declarations.snapshot_builds();
        session
            .python_declaration_snapshot(&uri)
            .expect("revalidate declaration status");
        assert!(session.python_declarations.snapshot_builds() > builds);
    }
    assert_eq!(std::fs::read(path).expect("source unchanged"), source);
}

#[cfg(unix)]
#[test]
fn populated_ancestor_venv_preserves_diagnostics_and_environment_invalidation() {
    let temp = tempfile::tempdir().expect("ancestor uv project");
    std::fs::write(
        temp.path().join("pyproject.toml"),
        "[project]\nname = \"ancestor\"\nversion = \"0.0.0\"\n",
    )
    .expect("pyproject");
    std::fs::write(temp.path().join("uv.lock"), "version = 1\n").expect("uv lock");
    let bin = temp.path().join(".venv/bin");
    std::fs::create_dir_all(&bin).expect("venv");
    let interpreter = bin.join("python");
    std::fs::write(&interpreter, vec![42; 32_207_448]).expect("interpreter-sized fixture");
    let root = temp.path().join("editor");
    let (path, uri) = fixture(&root, "populated-ancestor");
    let mut session = Session::new();
    open(&mut session, uri.clone());
    let initial = session.external_input_identity_for_path(&path);
    let start = std::time::Instant::now();
    for version in 2..462 {
        session.change_compacted(&uri, Some(version), &[serde_json::json!({"text": format!("def main() -> int:\n    return {version}\n")})]).expect("editor update");
        assert!(
            crate::diagnostics::document_diagnostics(&mut session, &uri)
                .expect("diagnostics")
                .is_empty()
        );
    }
    eprintln!("460 populated-venv diagnostics: {:?}", start.elapsed());
    assert_eq!(session.external_input_identity_for_path(&path), initial);
    let replacement = bin.join("replacement");
    std::fs::write(&replacement, vec![42; 32_207_448]).expect("same-version replacement");
    std::fs::rename(replacement, &interpreter).expect("replace");
    let replaced = session.observe_external_input_identity_for_path(&path);
    assert!(replaced.generation > initial.generation);
    std::fs::write(temp.path().join(".venv/pyvenv.cfg"), "home = changed\n")
        .expect("config change");
    assert!(
        session
            .observe_external_input_identity_for_path(&path)
            .generation
            > replaced.generation
    );
}

#[cfg(unix)]
#[test]
fn interpreter_drift_during_request_rejects_stale_declarations() {
    let temp = tempfile::tempdir().expect("uv project");
    let (path, uri) = fixture(temp.path(), "interpreter-stale");
    std::fs::write(temp.path().join("pyproject.toml"), "[project]\n").expect("project");
    std::fs::write(temp.path().join("uv.lock"), "version = 1\n").expect("lock");
    let bin = temp.path().join(".venv/bin");
    std::fs::create_dir_all(&bin).expect("venv");
    let interpreter = bin.join("python");
    std::fs::write(&interpreter, b"same-version-A").expect("interpreter");
    let mut session = Session::new();
    open(&mut session, uri.clone());
    let before = session.external_input_identity_for_path(&path);
    session
        .python_declarations
        .inject_external_change_before_verification(interpreter, Some(b"same-version-B".to_vec()));
    let stale = session
        .python_declaration_snapshot(&uri)
        .expect_err("interpreter drift must reject captured inputs");
    assert_eq!(stale.code(), lsp_server::ErrorCode::ContentModified as i32);
    assert!(session.external_input_identity_for_path(&path).generation > before.generation);
    assert!(!session.python_declarations.has_entry(temp.path()));
    session
        .python_declaration_snapshot(&uri)
        .expect("retry with current inputs");
}
