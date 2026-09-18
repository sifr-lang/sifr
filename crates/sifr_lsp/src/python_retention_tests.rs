use super::*;
#[test]
fn dx11_closing_one_project_releases_its_environment_and_target_caches() {
    let temp = tempfile::tempdir().unwrap();
    let mut cache = PythonDeclarationCache::default();
    let mut documents = crate::document_store::DocumentStore::new();
    let mut uris = Vec::new();
    for name in ["open", "closed"] {
        let root = temp.path().join(name);
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("sifr.toml"), "").unwrap();
        let path = root.join("main.sifr");
        let uri = url::Url::from_file_path(&path).unwrap().to_string();
        documents
            .open(
                uri.clone(),
                "sifr",
                Some(1),
                "def main():\n    pass\n".into(),
            )
            .unwrap();
        uris.push(uri);
        cache.environments.insert(
            EnvironmentCacheKey {
                package_root: root.clone(),
                external_fingerprint: 1,
                required_import_roots: vec!["example".into()],
            },
            EnvironmentSnapshot {
                runtime: None,
                diagnostics: vec![],
            },
        );
        cache.target_inspections.insert(
            (root, 1, "example".into()),
            Err("cached inspection failure".into()),
        );
    }
    documents.close(&uris[1]);
    cache.retain_open_projects(&documents);
    assert_eq!(cache.environments.len(), 1);
    assert!(
        cache
            .environments
            .keys()
            .all(|key| key.package_root.ends_with("open"))
    );
    assert_eq!(cache.target_inspections.len(), 1);
    documents.close(&uris[0]);
    cache.retain_open_projects(&documents);
    assert!(cache.environments.is_empty());
    assert!(cache.target_inspections.is_empty());
}
