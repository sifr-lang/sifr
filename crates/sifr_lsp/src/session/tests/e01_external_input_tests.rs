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
    select_explicit_environment(temp.path(), ".");
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

fn select_explicit_environment(root: &Path, directory: &str) {
    let path = root.join("sifr.toml");
    let mut manifest = std::fs::read_to_string(&path).expect("read manifest");
    manifest.push_str(&format!(
        "\n[python]\npyproject = \"{directory}/pyproject.toml\"\nlock = \"{directory}/uv.lock\"\n"
    ));
    std::fs::write(path, manifest).expect("select Python metadata");
}

fn selected_project(root: &Path) -> Option<PathBuf> {
    sifr_compiler_services::python::python_environment_selection(
        root,
        &mut sifr_analysis::DiskSourceProvider::new(),
    )
    .and_then(|selection| selection.pyproject)
}

#[test]
fn unselected_python_projects_preserve_explicit_environment_cache() {
    let temp = tempfile::tempdir().expect("temporary ancestry");
    let root = temp.path().join("package");
    let (path, uri) = fixture(&root, "explicit-root");
    select_explicit_environment(&root, "selected");
    assert_eq!(
        selected_project(&root),
        Some(root.join("selected/pyproject.toml"))
    );
    let mut session = Session::new();
    open(&mut session, uri.clone());
    session
        .python_declaration_snapshot(&uri)
        .expect("initial status");
    let generation = session.external_inputs.generation(&root);
    let builds = session.python_declarations.snapshot_builds();
    for directory in [temp.path(), root.as_path()] {
        for name in ["pyproject.toml", "uv.lock"] {
            std::fs::write(directory.join(name), "unselected").expect("unselected input");
            session.observe_external_inputs_for_path(&path);
            session
                .python_declaration_snapshot(&uri)
                .expect("warm status");
            assert_eq!(session.external_inputs.generation(&root), generation);
            assert_eq!(session.python_declarations.snapshot_builds(), builds);
        }
    }
    std::fs::create_dir(root.join("selected")).expect("selected directory");
    for name in ["pyproject.toml", "uv.lock"] {
        let selected = root.join("selected").join(name);
        for content in ["", "changed"] {
            let before = session.external_inputs.generation(&root);
            std::fs::write(&selected, content).expect("selected input");
            session.observe_external_inputs_for_path(&path);
            assert_ne!(session.external_inputs.generation(&root), before);
            assert!(!session.python_declarations.has_entry(&root));
            session
                .python_declaration_snapshot(&uri)
                .expect("revalidated status");
        }
        let before = session.external_inputs.generation(&root);
        std::fs::remove_file(&selected).expect("delete selected input");
        session.observe_external_inputs_for_path(&path);
        assert_ne!(session.external_inputs.generation(&root), before);
        assert!(!session.python_declarations.has_entry(&root));
        session
            .python_declaration_snapshot(&uri)
            .expect("missing input status");
    }
}

#[test]
fn discovered_python_environment_reselects_after_project_creation_and_deletion() {
    let temp = tempfile::tempdir().expect("temporary ancestry");
    let root = temp.path().join("package");
    let (path, uri) = fixture(&root, "discovered-root");
    for name in ["pyproject.toml", "uv.lock"] {
        std::fs::write(temp.path().join(name), "parent").expect("ancestor project");
    }
    let mut session = Session::new();
    open(&mut session, uri.clone());
    session
        .python_declaration_snapshot(&uri)
        .expect("initial status");
    assert_eq!(
        selected_project(&root),
        Some(temp.path().join("pyproject.toml"))
    );
    let initial = session.external_inputs.generation(&root);
    std::fs::write(root.join("pyproject.toml"), "nearer").expect("incomplete nearer project");
    session.observe_external_inputs_for_path(&path);
    assert_eq!(session.external_inputs.generation(&root), initial);
    assert!(session.python_declarations.has_entry(&root));
    for (lock, create, project) in [
        (
            root.join("uv.lock"),
            true,
            Some(root.join("pyproject.toml")),
        ),
        (
            root.join("uv.lock"),
            false,
            Some(temp.path().join("pyproject.toml")),
        ),
        (temp.path().join("uv.lock"), false, None),
        (
            temp.path().join("uv.lock"),
            true,
            Some(temp.path().join("pyproject.toml")),
        ),
    ] {
        let before = session.external_inputs.generation(&root);
        if create {
            std::fs::write(lock, "selected").expect("create lock");
        } else {
            std::fs::remove_file(lock).expect("delete lock");
        }
        assert_eq!(selected_project(&root), project);
        session.observe_external_inputs_for_path(&path);
        assert_ne!(session.external_inputs.generation(&root), before);
        assert!(!session.python_declarations.has_entry(&root));
        session
            .python_declaration_snapshot(&uri)
            .expect("reselected status");
    }
}
