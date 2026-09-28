use crate::session::Session;
use crate::session::watcher_events::{WatcherEvent, WatcherEventKind};
use std::path::{Path, PathBuf};

fn fixture(root: &Path, name: &str) -> (PathBuf, String) {
    std::fs::create_dir_all(root.join("src/python_bridges")).expect("source directory");
    std::fs::write(root.join("src/lib.rs"), "").expect("Cargo library marker");
    std::fs::write(root.join("Cargo.toml"), format!(
        "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n\n[workspace]\n"
    )).expect("Cargo manifest");
    std::fs::write(
        root.join("Cargo.lock"),
        format!("version = 4\n\n[[package]]\nname = \"{name}\"\nversion = \"0.0.0\"\n"),
    )
    .expect("Cargo lock");
    std::fs::write(root.join("sifr.toml"), "[package]\nname = \"watcher\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n").expect("Sifr manifest");
    let path = root.join("src/main.sifr");
    std::fs::write(&path, "def main() -> int:\n    return 1\n").expect("Sifr source");
    let uri = url::Url::from_file_path(&path)
        .expect("file URI")
        .to_string();
    (path, uri)
}

fn event(path: &Path, kind: u64) -> WatcherEvent {
    WatcherEvent::from_protocol(
        &url::Url::from_file_path(path)
            .expect("event URI")
            .to_string(),
        kind,
    )
    .expect("valid watcher event")
}

#[test]
fn create_delete_rename_and_storm_events_preserve_multi_root_isolation() {
    let temp = tempfile::tempdir().expect("temporary roots");
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    let (_, first_uri) = fixture(&first, "watcher-first");
    let (_, second_uri) = fixture(&second, "watcher-second");
    let mut session = Session::new();
    for uri in [&first_uri, &second_uri] {
        session
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                "def main() -> int:\n    return 1\n".to_string(),
            )
            .expect("open document");
        session
            .python_declaration_snapshot(uri)
            .expect("declaration snapshot");
    }
    let second_generation = session
        .external_input_generation(&second)
        .expect("second generation");
    let mut previous = session
        .external_input_generation(&first)
        .expect("first generation");
    let bridge_dir = first.join("src/python_bridges");
    let original = bridge_dir.join("original.py");
    std::fs::write(&original, "value = 1\n").expect("create bridge");
    session.record_watcher_file_events(&[event(&original, 1)]);
    let created = session
        .external_input_generation(&first)
        .expect("created generation");
    assert!(created > previous);
    assert_eq!(
        session.external_input_generation(&second),
        Some(second_generation)
    );
    assert!(session.python_declarations.has_entry(&second));
    previous = created;
    std::fs::remove_file(&original).expect("delete bridge");
    session.record_watcher_file_events(&[event(&original, 3)]);
    let deleted = session
        .external_input_generation(&first)
        .expect("deleted generation");
    assert!(deleted > previous);
    std::fs::write(&original, "value = 2\n").expect("recreate bridge");
    session.record_watcher_file_events(&[event(&original, 1)]);
    previous = session
        .external_input_generation(&first)
        .expect("recreated generation");
    let renamed = bridge_dir.join("renamed.py");
    std::fs::rename(&original, &renamed).expect("rename bridge");
    session.record_watcher_file_events(&[event(&original, 3), event(&renamed, 1)]);
    let after_rename = session
        .external_input_generation(&first)
        .expect("renamed generation");
    assert!(after_rename > previous);
    let lock = first.join("Cargo.lock");
    std::fs::write(&lock, "version = 4\n# storm change\n").expect("change lock");
    let storm: Vec<_> = (0..65).map(|_| event(&lock, 2)).collect();
    session.record_watcher_file_events(&storm);
    assert!(
        session
            .external_input_generation(&first)
            .expect("storm generation")
            > after_rename
    );
    assert_eq!(
        session.external_input_generation(&second),
        Some(second_generation)
    );
    assert!(session.python_declarations.has_entry(&second));
    let first_before_cross_root = session
        .external_input_generation(&first)
        .expect("first before cross-root rename");
    let second_before_cross_root = session
        .external_input_generation(&second)
        .expect("second before cross-root rename");
    let moved = second.join("src/python_bridges/moved.py");
    std::fs::rename(&renamed, &moved).expect("move bridge between roots");
    session.record_watcher_file_events(&[event(&renamed, 3), event(&moved, 1)]);
    assert!(
        session
            .external_input_generation(&first)
            .expect("first after cross-root rename")
            > first_before_cross_root
    );
    assert!(
        session
            .external_input_generation(&second)
            .expect("second after cross-root rename")
            > second_before_cross_root
    );
    assert_eq!(event(&lock, 2).kind, WatcherEventKind::Changed);
    assert!(WatcherEvent::from_protocol("https://example.invalid/lock", 2).is_none());
    assert!(
        WatcherEvent::from_protocol(&url::Url::from_file_path(&lock).unwrap().to_string(), 4)
            .is_none()
    );
}
