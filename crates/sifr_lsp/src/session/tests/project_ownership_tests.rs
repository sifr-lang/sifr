use crate::session::Session;
use serde_json::json;

#[test]
fn project_save_without_version_keeps_project_owner_current() {
    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::write(
        temp.path().join("sifr.toml"),
        "[package]\nname = \"lsp-save-owner\"\nedition = \"2026\"\n",
    )
    .expect("write manifest");
    let path = temp.path().join("main.sifr");
    let initial = "def main() -> int:\n    if True:\n        return 1\n    return 0\n";
    let saved = concat!(
        "def helper(value: int) -> int:\n",
        "    return value + 1\n\n",
        "def main() -> int:\n",
        "    result: int = helper(41)\n",
        "    return result\n",
    );
    let shortened = "def main():\n    print(1)\n";
    std::fs::write(&path, initial).expect("write source");
    let uri = url::Url::from_file_path(&path)
        .expect("file uri")
        .to_string();

    let mut session = Session::new();
    session
        .open_document(
            uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            initial.to_string(),
        )
        .expect("open document");
    session
        .change_compacted(&uri, Some(2), &[json!({"text": saved})])
        .expect("change to saved text");
    assert!(!session.analysis.has_standalone_document(&uri));
    session
        .save_document(&uri, Some(saved.to_string()))
        .expect("save document");
    assert!(!session.analysis.has_standalone_document(&uri));
    session
        .with_document_analysis(&uri, |_snapshot, _host, _file, source| {
            assert_eq!(source, saved);
            Ok(())
        })
        .expect("saved text should be served by project owner");
    session
        .change_compacted(&uri, Some(3), &[json!({"text": shortened})])
        .expect("change to shortened text");
    assert!(!session.analysis.has_standalone_document(&uri));
    let file = session
        .with_document_analysis(&uri, |_snapshot, _host, file, source| {
            assert_eq!(source, shortened);
            Ok(file)
        })
        .expect("shortened text should be served by project owner");
    let file_maps = session.file_maps_for_uri(&uri).expect("file maps");
    assert_eq!(
        file_maps.source_for(file).expect("source for project file"),
        shortened
    );
}

#[test]
fn unmapped_project_file_does_not_create_standalone_project_fallback() {
    let temp = tempfile::tempdir().expect("temp dir");
    let src = temp.path().join("src");
    std::fs::create_dir(&src).expect("create src");
    std::fs::write(
        temp.path().join("sifr.toml"),
        "[package]\nname = \"lsp-secondary-open\"\nedition = \"2026\"\n",
    )
    .expect("write manifest");
    let main_path = src.join("main.sifr");
    let orphan_path = src.join("orphan.sifr");
    std::fs::write(&main_path, "def main() -> int:\n    return 1\n").expect("write main source");
    std::fs::write(
        &orphan_path,
        "def orphan(value: int) -> int:\n    return value + 1\n",
    )
    .expect("write orphan source");
    let main_uri = url::Url::from_file_path(&main_path)
        .expect("main file uri")
        .to_string();
    let orphan_uri = url::Url::from_file_path(&orphan_path)
        .expect("orphan file uri")
        .to_string();
    let mut session = Session::new();
    session
        .open_document(
            main_uri,
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            std::fs::read_to_string(&main_path).expect("read main source"),
        )
        .expect("open project entrypoint");
    session
        .open_document(
            orphan_uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            std::fs::read_to_string(&orphan_path).expect("read orphan source"),
        )
        .expect("open unmapped project file");
    let error = session
        .with_document_analysis(&orphan_uri, |_snapshot, _host, _file, source| {
            assert!(source.contains("def orphan"));
            Ok(())
        })
        .expect_err("unmapped project file should not have standalone fallback analysis");
    assert!(
        error.message().contains("analysis is unavailable"),
        "unexpected error: {error:?}"
    );
    let main_uri = url::Url::from_file_path(&main_path).expect("main file uri");
    assert!(session.close_document(main_uri.as_str()));
    session
        .open_document(
            url::Url::from_file_path(&main_path)
                .expect("main file uri")
                .to_string(),
            crate::capabilities::LANGUAGE_ID,
            Some(2),
            std::fs::read_to_string(&main_path).expect("read main source"),
        )
        .expect("reopen project entrypoint");
    let error = session
        .with_document_analysis(&orphan_uri, |_snapshot, _host, _file, source| {
            assert!(source.contains("def orphan"));
            Ok(())
        })
        .expect_err("project refresh should not create standalone fallback analysis");
    assert!(
        error.message().contains("analysis is unavailable"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn unmapped_project_file_does_not_fallback_after_unrelated_root_refresh() {
    let temp = tempfile::tempdir().expect("temp dir");
    let first_root = temp.path().join("first");
    let second_root = temp.path().join("second");
    let first_src = first_root.join("src");
    let second_src = second_root.join("src");
    std::fs::create_dir_all(&first_src).expect("create first src");
    std::fs::create_dir_all(&second_src).expect("create second src");
    for (root, name) in [(&first_root, "first"), (&second_root, "second")] {
        std::fs::write(
            root.join("sifr.toml"),
            format!("[package]\nname = \"{name}\"\nedition = \"2026\"\n"),
        )
        .expect("write manifest");
    }
    let first_main_path = first_src.join("main.sifr");
    let orphan_path = first_src.join("orphan.sifr");
    let second_main_path = second_src.join("main.sifr");
    std::fs::write(&first_main_path, "def main() -> int:\n    return 1\n")
        .expect("write first main");
    std::fs::write(
        &orphan_path,
        "def orphan(value: int) -> int:\n    return value + 1\n",
    )
    .expect("write orphan");
    std::fs::write(&second_main_path, "def main() -> int:\n    return 2\n")
        .expect("write second main");
    let first_main_uri = url::Url::from_file_path(&first_main_path)
        .expect("first main file uri")
        .to_string();
    let orphan_uri = url::Url::from_file_path(&orphan_path)
        .expect("orphan file uri")
        .to_string();
    let second_main_uri = url::Url::from_file_path(&second_main_path)
        .expect("second main file uri")
        .to_string();
    let mut session = Session::new();
    session
        .open_document(
            first_main_uri,
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            std::fs::read_to_string(&first_main_path).expect("read first main"),
        )
        .expect("open first entrypoint");
    session
        .open_document(
            orphan_uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            std::fs::read_to_string(&orphan_path).expect("read orphan"),
        )
        .expect("open orphan");
    session
        .open_document(
            second_main_uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            std::fs::read_to_string(&second_main_path).expect("read second main"),
        )
        .expect("open second entrypoint");
    let error = session
        .with_document_analysis(&orphan_uri, |_snapshot, _host, _file, source| {
            assert!(source.contains("def orphan"));
            Ok(())
        })
        .expect_err("orphan should not have fallback analysis after second root opens");
    assert!(
        error.message().contains("analysis is unavailable"),
        "unexpected error: {error:?}"
    );
    assert!(session.close_document(&second_main_uri));
    let error = session
        .with_document_analysis(&orphan_uri, |_snapshot, _host, _file, source| {
            assert!(source.contains("def orphan"));
            Ok(())
        })
        .expect_err("orphan should not gain fallback analysis after unrelated root close");
    assert!(
        error.message().contains("analysis is unavailable"),
        "unexpected error: {error:?}"
    );
}
