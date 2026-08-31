#![allow(clippy::expect_used)]

use semver::Version;
use sifr_sql_contract::{
    CompiledMigrationGraph, DialectIdentity, MigrationNodeId, ProviderIdentity, SchemaIr,
    schema_fingerprint,
};
use sifr_sql_tool::{
    MIGRATION_ARTIFACT_MANIFEST_PATH, MIGRATION_GRAPH_PATH, MIGRATION_IMPACT_PATH,
    MIGRATION_SCHEMA_PATH, SchemaLifecycleErrorKind, build_migration_artifacts,
    write_migration_artifacts_atomically,
};
use std::collections::{BTreeMap, BTreeSet};

fn schema() -> SchemaIr {
    SchemaIr {
        format_version: 1,
        provider: ProviderIdentity {
            package_id: "sifr-sql-postgresql".to_string(),
            package_version: Version::new(1, 0, 0),
            package_source: "path+postgresql".to_string(),
            package_graph_digest: "a".repeat(64),
            compiler_components: BTreeMap::new(),
        },
        dialect: DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18.0.0".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        objects: BTreeMap::new(),
    }
}

fn graph(target: &SchemaIr) -> CompiledMigrationGraph {
    CompiledMigrationGraph {
        format_version: 1,
        provider_family: "postgresql".to_string(),
        target_fingerprint: schema_fingerprint(target)
            .expect("test schema should fingerprint")
            .as_str()
            .to_string(),
        head: MigrationNodeId::new("head").expect("test identity should be valid"),
        topological_order: Vec::new(),
        baseline_fingerprints: BTreeMap::new(),
        migrations: BTreeMap::new(),
        impacts: Vec::new(),
    }
}

#[test]
fn migration_artifacts_are_deterministic_complete_and_atomic() {
    let target = schema();
    let graph = graph(&target);
    let first = build_migration_artifacts(&graph, &target).expect("first build should pass");
    let second = build_migration_artifacts(&graph, &target).expect("second build should pass");
    assert_eq!(first, second);
    for path in [
        MIGRATION_GRAPH_PATH,
        MIGRATION_SCHEMA_PATH,
        MIGRATION_IMPACT_PATH,
        MIGRATION_ARTIFACT_MANIFEST_PATH,
    ] {
        assert!(first.files().contains_key(path));
    }
    let temporary = tempfile::tempdir().expect("temporary directory should exist");
    let output = temporary.path().join(".sifr/sql-migrations/app");
    write_migration_artifacts_atomically(&output, &first).expect("atomic write should pass");
    for (path, bytes) in first.files() {
        assert_eq!(
            std::fs::read(output.join(path)).expect("artifact should exist"),
            *bytes
        );
    }
}

#[test]
fn migration_artifacts_reject_a_target_authority_mismatch() {
    let target = schema();
    let mut graph = graph(&target);
    graph.target_fingerprint = "b".repeat(64);
    assert_eq!(
        build_migration_artifacts(&graph, &target)
            .expect_err("target mismatch must fail")
            .kind,
        SchemaLifecycleErrorKind::InvalidAuthority
    );
}
