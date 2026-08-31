#![allow(clippy::expect_used)]

use sifr_frontend::lower_migration_graph;
use sifr_sql_contract::{
    CompiledMigration, CompiledMigrationGraph, CompiledMigrationPath, CompiledMigrationStep,
    CompiledStepKind, MigrationNodeId, MigrationProviderConstraint, MigrationStateIdentity,
    TransactionRequirement,
};
use sifr_type_system::Type;
use std::collections::{BTreeMap, BTreeSet};

fn id(value: &str) -> MigrationNodeId {
    MigrationNodeId::new(value).expect("test migration identity should be valid")
}

#[test]
fn migration_lowering_uses_compiler_generated_nominal_state_types() {
    let baseline = id("baseline");
    let migration_id = id("m1");
    let input = MigrationStateIdentity::new("sifr.sql.migration.state.m1.baseline.0.aaaa");
    let output = MigrationStateIdentity::new("sifr.sql.migration.state.m1.baseline.1.bbbb");
    let step = CompiledMigrationStep {
        id: id("callback"),
        input_state: input.clone(),
        output_state: output.clone(),
        input_fingerprint: "a".repeat(64),
        output_fingerprint: "a".repeat(64),
        checksum: "c".repeat(64),
        referenced_objects: BTreeSet::new(),
        affected_objects: BTreeSet::new(),
        kind: CompiledStepKind::SifrData {
            callback: "fill".to_string(),
        },
    };
    let path = CompiledMigrationPath {
        parent: baseline.clone(),
        input_fingerprint: "a".repeat(64),
        output_fingerprint: "a".repeat(64),
        steps: vec![step],
        rollback: None,
    };
    let migration = CompiledMigration {
        id: migration_id.clone(),
        parents: BTreeSet::from([baseline.clone()]),
        provider: MigrationProviderConstraint {
            family: "postgresql".to_string(),
            minimum_server_version: None,
            required_capabilities: BTreeSet::new(),
        },
        transaction_requirement: TransactionRequirement::Optional,
        checksum: "d".repeat(64),
        paths: BTreeMap::from([(baseline.clone(), path)]),
        author: "test".to_string(),
        created_at: "2026-08-31T00:00:00Z".to_string(),
    };
    let graph = CompiledMigrationGraph {
        format_version: 1,
        provider_family: "postgresql".to_string(),
        target_fingerprint: "a".repeat(64),
        head: migration_id.clone(),
        topological_order: vec![migration_id.clone()],
        baseline_fingerprints: BTreeMap::from([(baseline, "a".repeat(64))]),
        migrations: BTreeMap::from([(migration_id, migration)]),
        impacts: Vec::new(),
    };
    let hir = lower_migration_graph(&graph).expect("compiled graph should lower");
    let lowered = &hir.steps[0];
    assert_eq!(
        class_argument_identity(&lowered.input_plan_type),
        input.as_str()
    );
    assert_eq!(
        class_argument_identity(&lowered.output_plan_type),
        output.as_str()
    );
    assert_eq!(
        lowered
            .callback_db_type
            .as_ref()
            .map(class_argument_identity),
        Some(input.as_str())
    );
}

fn class_argument_identity(ty: &Type) -> &str {
    let Type::Class { type_args, .. } = ty else {
        panic!("expected nominal migration type");
    };
    let Some(Type::Class {
        identity: Some(identity),
        ..
    }) = type_args.first()
    else {
        panic!("expected nominal migration state argument");
    };
    identity
}
