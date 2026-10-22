use super::*;

#[test]
fn package_build_cache_digest_changes_with_lock_source_and_target_inputs() {
    let mut first = PackageBuildCacheInputs {
        cargo_lock_digest: Some("lock-a".to_string()),
        cargo_metadata_digest: Some("metadata-a".to_string()),
        package_graph_digest: Some("graph-a".to_string()),
        package_source_map_digest: Some("source-map-a".to_string()),
        compiler_version: "0.3.0".to_string(),
        target: Some("aarch64-apple-darwin".to_string()),
        profile: "release".to_string(),
        features: vec!["tls".to_string(), "json".to_string()],
        selectors: vec!["package:app".to_string()],
        ..PackageBuildCacheInputs::default()
    };
    first
        .sifr_source_digests
        .insert("sifr/app/__init__.sifr".to_string(), "src-a".to_string());

    let mut reordered = first.clone();
    reordered.features = vec!["json".to_string(), "tls".to_string()];
    assert_eq!(
        digest_package_build_cache_inputs(&first),
        digest_package_build_cache_inputs(&reordered)
    );

    let mut changed = first.clone();
    changed.cargo_lock_digest = Some("lock-b".to_string());
    assert_ne!(
        digest_package_build_cache_inputs(&first),
        digest_package_build_cache_inputs(&changed)
    );

    let mut python_changed = first.clone();
    python_changed.python_probe_digest = Some("python-probe-b".to_string());
    assert_ne!(
        digest_package_build_cache_inputs(&first),
        digest_package_build_cache_inputs(&python_changed)
    );
}

#[test]
fn package_identity_domains_and_all_build_key_fields_are_distinct() {
    let baseline = PackageBuildCacheInputs {
        cargo_lock_digest: Some("lock".into()),
        cargo_metadata_digest: Some("metadata".into()),
        package_graph_digest: Some("graph".into()),
        package_source_map_digest: Some("map".into()),
        python_probe_digest: Some("probe".into()),
        sifr_metadata_digests: BTreeMap::from([("manifest".into(), "a".into())]),
        sifr_source_digests: BTreeMap::from([("source".into(), "b".into())]),
        compiler_version: "v1".into(),
        target: Some("target".into()),
        profile: "debug".into(),
        features: vec!["feature".into()],
        selectors: vec!["package".into()],
    };
    let original = digest_package_build_cache_inputs(&baseline).unwrap();
    assert_eq!(original.algorithm, "sha256-framed-v1");
    assert_eq!(original.hex.len(), 64);
    let change = |modify: fn(&mut PackageBuildCacheInputs)| {
        let mut next = baseline.clone();
        modify(&mut next);
        assert_ne!(original, digest_package_build_cache_inputs(&next).unwrap());
    };
    change(|x| x.cargo_lock_digest = None);
    change(|x| x.cargo_metadata_digest = Some(String::new()));
    change(|x| x.package_graph_digest = None);
    change(|x| x.package_source_map_digest = None);
    change(|x| x.python_probe_digest = None);
    change(|x| {
        x.sifr_metadata_digests
            .insert("manifest".into(), "c".into());
    });
    change(|x| {
        x.sifr_source_digests.insert("source".into(), "c".into());
    });
    change(|x| x.compiler_version = "v2".into());
    change(|x| x.target = None);
    change(|x| x.profile = "release".into());
    change(|x| x.features.push("other".into()));
    change(|x| x.selectors.push("other".into()));
    assert_ne!(
        digest_package_build_cache_inputs(&PackageBuildCacheInputs::default()),
        digest_package_graph(&package_graph(TrustPolicy::default(), vec![])),
        "equal serialized payloads in distinct domains must not share identity",
    );
}

#[test]
fn normalized_cargo_identity_binds_all_resolved_field_groups() {
    let package = cargo_package("/ws/app/Cargo.toml");
    let id = package.id.clone();
    let baseline = NormalizedCargoMetadata {
        packages: BTreeMap::from([(id.clone(), package)]),
        resolve_edges: vec![],
        workspace_members: BTreeSet::from([id.clone()]),
        workspace_default_members: BTreeSet::from([id.clone()]),
        target_directory: "/ws/target".into(),
        workspace_root: "/ws".into(),
        workspace_sifr: Default::default(),
    };
    let original = digest_graph_inputs(&baseline).unwrap();
    let check = |modify: fn(&mut NormalizedCargoMetadata)| {
        let mut next = baseline.clone();
        modify(&mut next);
        assert_ne!(original, digest_graph_inputs(&next).unwrap());
    };
    check(|x| x.packages.values_mut().next().test_unwrap("package").name = "other".into());
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .version = "2.0.0".into()
    });
    check(|x| x.packages.values_mut().next().test_unwrap("package").source = None);
    check(|x| x.packages.values_mut().next().test_unwrap("package").links = Some("native".into()));
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .manifest_path = "/other/Cargo.toml".into()
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .dependencies
            .push(crate::cargo::metadata::CargoDependency {
                name: "dep".into(),
                package: None,
                req: "^1".into(),
                kind: None,
                target: None,
                uses_workspace: false,
            })
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .targets
            .push(CargoTarget {
                name: "lib".into(),
                kind: BTreeSet::from(["lib".into()]),
                crate_types: BTreeSet::from(["rlib".into()]),
                src_path: "/ws/app/src/lib.rs".into(),
            })
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .features
            .insert("default".into(), vec!["dep".into()]);
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .sifr_metadata = None
    });
    check(|x| {
        x.resolve_edges
            .push(crate::cargo::metadata::CargoResolveEdge {
                from: CargoPackageId("a".into()),
                dependency_name: "dep".into(),
                to: CargoPackageId("b".into()),
            })
    });
    check(|x| x.workspace_members.clear());
    check(|x| x.workspace_default_members.clear());
    check(|x| x.target_directory = "/other/target".into());
    check(|x| x.workspace_root = "/other".into());
    check(|x| x.workspace_sifr.tools_package = Some("tools".into()));
}

#[test]
fn package_graph_identity_binds_manifest_scopes_and_classification() {
    let baseline = package_graph(TrustPolicy::default(), vec![]);
    let original = digest_package_graph(&baseline).unwrap();
    let check = |modify: fn(&mut SifrPackageGraph)| {
        let mut next = baseline.clone();
        modify(&mut next);
        assert_ne!(original, digest_package_graph(&next).unwrap());
    };
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .cargo_version = "0.2.0".into()
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .package_root = "/other".into()
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .manifest
            .default_run = Some("main".into())
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .manifest
            .source_features
            .insert("sql".into(), "on".into());
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .manifest
            .trust
            .native
            .push("native".into())
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .manifest
            .python
            .venv = Some(".venv".into())
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .manifest
            .rust
            .bridges
            .push("bridge".into())
    });
    check(|x| {
        x.packages
            .values_mut()
            .next()
            .test_unwrap("package")
            .aliases
            .insert(
                "dep".into(),
                crate::manifest::metadata::CargoSifrAliasMetadata {
                    dependency: "dep".into(),
                    import: "alias".into(),
                },
            );
    });
    check(|x| {
        x.cargo_edges.insert(
            SifrPackageId("root".into()),
            BTreeSet::from([SifrPackageId("dep".into())]),
        );
    });
    check(|x| {
        x.direct_dependency_scopes
            .insert(SifrPackageId("root".into()), Default::default());
    });
    check(|x| {
        x.backend_crates
            .insert(SifrPackageId("root".into()), vec![backend("native")]);
    });
    check(|x| {
        x.classifications.clear();
    });
}

#[test]
fn package_source_consumer_distinguishes_absent_empty_and_unreadable() {
    let root = std::env::temp_dir().join(format!("sifr-n02-package-{}", std::process::id()));
    std::fs::create_dir_all(&root).test_unwrap("create temporary source root");
    let file = root.join("empty.sifr");
    std::fs::write(&file, b"").test_unwrap("write empty source");
    let package_id = SifrPackageId("app".into());
    let key = PackageModuleKey {
        package_id: package_id.clone(),
        module_path: DottedModulePath("app".into()),
    };
    let module = PackageModuleSource {
        package_id,
        cargo_package_id: CargoPackageId("cargo-app".into()),
        module_path: key.module_path.clone(),
        file_path: file.clone(),
        source_root: root.clone(),
    };
    let absent = PackageSourceMap::default();
    let mut empty = PackageSourceMap::default();
    empty.modules.insert(key, module);
    assert_ne!(
        digest_package_source_map(&absent),
        digest_package_source_map(&empty)
    );
    assert_ne!(
        digest_package_source_snapshot(&absent).test_unwrap("absent snapshot"),
        digest_package_source_snapshot(&empty).test_unwrap("empty snapshot")
    );
    let mut changed_authority = empty.clone();
    changed_authority
        .modules
        .values_mut()
        .next()
        .test_unwrap("module")
        .cargo_package_id = CargoPackageId("other-cargo".into());
    assert_ne!(
        digest_package_source_map(&empty),
        digest_package_source_map(&changed_authority)
    );
    assert_ne!(
        digest_package_source_snapshot(&empty).test_unwrap("original snapshot"),
        digest_package_source_snapshot(&changed_authority)
            .test_unwrap("changed authority snapshot"),
    );
    let mut unreadable = empty.clone();
    unreadable
        .modules
        .values_mut()
        .next()
        .test_unwrap("module")
        .file_path = root.clone();
    assert!(digest_package_source_snapshot(&unreadable).is_err());
    std::fs::remove_file(&file).test_unwrap("remove temporary source");
    std::fs::remove_dir(&root).test_unwrap("remove temporary root");
}

#[cfg(unix)]
#[test]
fn package_graph_non_utf8_path_cannot_hash_as_empty_payload() {
    use std::os::unix::ffi::OsStringExt;

    let mut graph = package_graph(TrustPolicy::default(), vec![]);
    graph
        .packages
        .values_mut()
        .next()
        .test_unwrap("package")
        .package_root = PathBuf::from(std::ffi::OsString::from_vec(b"/invalid-\xff".to_vec()));
    let error = digest_package_graph(&graph).test_expect_err("non-UTF-8 path must fail");
    assert!(error.contains("could not serialize package-graph identity"));
}
