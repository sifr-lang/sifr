#![allow(clippy::expect_used)]

use sifr_sql_mysql_tools::validate_mysql_migration_plan;
use sifr_sql_runtime::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationExecutionNode, MigrationExecutionPath,
    MigrationExecutionPlan, MigrationExecutionStep, MigrationExecutionStepKind, MigrationId,
    MigrationRuntimeConstraint, MigrationStateId, MigrationTransactionRequirement,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn mysql_ddl_requires_an_explicit_recovery_point() {
    let mut plan = plan(vec![step(
        "ddl",
        MigrationExecutionStepKind::Ddl {
            statement: "CREATE TABLE users(id BIGINT)".to_string(),
        },
    )]);
    assert!(validate_mysql_migration_plan(&plan).is_err());
    let path = plan
        .migrations
        .get_mut(&MigrationId::new("head"))
        .expect("head")
        .paths
        .get_mut(&MigrationId::new("baseline"))
        .expect("path");
    path.steps.insert(
        0,
        step(
            "recovery",
            MigrationExecutionStepKind::RecoveryPoint {
                name: "before-users".to_string(),
            },
        ),
    );
    repair_fingerprints(&mut path.steps);
    assert!(validate_mysql_migration_plan(&plan).is_ok());
}

fn plan(steps: Vec<MigrationExecutionStep>) -> MigrationExecutionPlan {
    let baseline = MigrationId::new("baseline");
    let head = MigrationId::new("head");
    MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: "mysql".to_string(),
        target_fingerprint: "c".repeat(64),
        head: head.clone(),
        topological_order: vec![head.clone()],
        baseline_fingerprints: BTreeMap::from([(baseline.clone(), "a".repeat(64))]),
        migrations: BTreeMap::from([(
            head.clone(),
            MigrationExecutionNode {
                id: head,
                parents: BTreeSet::from([baseline.clone()]),
                provider: MigrationRuntimeConstraint {
                    family: "mysql".to_string(),
                    minimum_server_version: Some("8.4".to_string()),
                    required_capabilities: BTreeSet::new(),
                },
                transaction_requirement: MigrationTransactionRequirement::Optional,
                checksum: "d".repeat(64),
                paths: BTreeMap::from([(
                    baseline.clone(),
                    MigrationExecutionPath {
                        parent: baseline,
                        input_fingerprint: "a".repeat(64),
                        output_fingerprint: "c".repeat(64),
                        steps,
                        rollback: None,
                    },
                )]),
                author: "qualification".to_string(),
                created_at: "2026-08-31T00:00:00Z".to_string(),
            },
        )]),
    }
}

fn step(id: &str, kind: MigrationExecutionStepKind) -> MigrationExecutionStep {
    MigrationExecutionStep {
        id: MigrationId::new(id),
        input_state: MigrationStateId::new(format!("{id}-input")),
        output_state: MigrationStateId::new(format!("{id}-output")),
        input_fingerprint: "a".repeat(64),
        output_fingerprint: "c".repeat(64),
        checksum: "b".repeat(64),
        kind,
    }
}

fn repair_fingerprints(steps: &mut [MigrationExecutionStep]) {
    let last = steps.len().saturating_sub(1);
    for (index, step) in steps.iter_mut().enumerate() {
        let marker = char::from(b'a'.saturating_add(u8::try_from(index).unwrap_or(0)));
        let next = char::from(b'a'.saturating_add(u8::try_from(index + 1).unwrap_or(1)));
        step.input_fingerprint = marker.to_string().repeat(64);
        step.output_fingerprint = if index == last {
            "c".repeat(64)
        } else {
            next.to_string().repeat(64)
        };
    }
}
