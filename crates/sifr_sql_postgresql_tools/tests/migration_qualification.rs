#![allow(clippy::expect_used)]

use semver::Version;
use sifr_sql_contract::{
    DdlReflection, DialectIdentity, MigrationDialect, ProviderIdentity, SchemaIr, SchemaObjectKind,
};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresDdlExecutionClass, PostgresMigrationDialect, classify_migration_ddl,
};
use sifr_sql_postgresql_tools::validate_postgres_migration_plan;
use sifr_sql_runtime::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationExecutionNode, MigrationExecutionPath,
    MigrationExecutionPlan, MigrationExecutionStep, MigrationExecutionStepKind, MigrationId,
    MigrationRuntimeConstraint, MigrationStateId, MigrationTransactionBoundary,
    MigrationTransactionRequirement,
};
use std::collections::{BTreeMap, BTreeSet};

fn schema() -> SchemaIr {
    SchemaIr {
        format_version: 1,
        provider: ProviderIdentity {
            package_id: "sifr-sql-postgresql".to_string(),
            package_version: Version::new(0, 0, 0),
            package_source: "test".to_string(),
            package_graph_digest: "a".repeat(64),
            compiler_components: BTreeMap::new(),
        },
        dialect: DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18.0.0".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::from(["transactional-ddl".to_string()]),
        },
        objects: BTreeMap::new(),
    }
}

#[test]
fn postgresql_reflection_covers_the_static_ddl_capability_matrix() {
    let dialect = PostgresMigrationDialect::new(
        LibpgQueryParser,
        "18.0.0",
        BTreeSet::from(["transactional-ddl".to_string()]),
    );
    let mut current = schema();
    for statement in [
        "CREATE TYPE public.mood AS ENUM ('sad', 'ok')",
        "CREATE DOMAIN public.email_address AS text",
        "CREATE TYPE public.postal_address AS (street text, city text)",
        "CREATE TYPE public.price_range AS RANGE (subtype = numeric)",
        "CREATE SEQUENCE public.audit_sequence",
        "CREATE TABLE public.organizations (id bigint PRIMARY KEY)",
        "CREATE TABLE public.accounts (id bigint PRIMARY KEY, organization_id bigint REFERENCES public.organizations(id), email text NOT NULL UNIQUE, balance bigint, CHECK (balance >= 0))",
        "CREATE UNIQUE INDEX accounts_email_idx ON public.accounts(email)",
        "CREATE VIEW public.account_view AS SELECT id, email FROM public.accounts",
        "CREATE MATERIALIZED VIEW public.account_count AS SELECT count(*) AS count FROM public.accounts",
        "CREATE FUNCTION public.add_one(integer) RETURNS integer LANGUAGE SQL IMMUTABLE STRICT AS 'SELECT $1 + 1'",
    ] {
        let reflection = dialect
            .reflect_ddl(&current, statement)
            .expect("PostgreSQL DDL should reflect");
        assert!(matches!(&reflection, DdlReflection::Reflected { .. }));
        current = match reflection {
            DdlReflection::Reflected { schema, .. } => schema,
            DdlReflection::Opaque => return,
        };
    }
    let kinds = current
        .objects
        .values()
        .map(|object| object.kind)
        .collect::<BTreeSet<_>>();
    for kind in [
        SchemaObjectKind::Namespace,
        SchemaObjectKind::Enum,
        SchemaObjectKind::Domain,
        SchemaObjectKind::Composite,
        SchemaObjectKind::Range,
        SchemaObjectKind::Sequence,
        SchemaObjectKind::Table,
        SchemaObjectKind::Column,
        SchemaObjectKind::PrimaryKey,
        SchemaObjectKind::UniqueConstraint,
        SchemaObjectKind::ForeignKey,
        SchemaObjectKind::CheckConstraint,
        SchemaObjectKind::Index,
        SchemaObjectKind::View,
        SchemaObjectKind::MaterializedView,
        SchemaObjectKind::Function,
    ] {
        assert!(kinds.contains(&kind), "missing reflected kind {kind:?}");
    }
}

#[test]
fn unsupported_valid_ddl_is_opaque_and_requires_a_declared_effect() {
    let dialect = PostgresMigrationDialect::new(LibpgQueryParser, "18.0.0", BTreeSet::new());
    assert_eq!(
        dialect
            .reflect_ddl(&schema(), "CREATE EXTENSION IF NOT EXISTS pg_trgm")
            .expect("valid provider extension DDL should be explicit"),
        DdlReflection::Opaque
    );
}

#[test]
fn postgresql_nontransactional_ddl_has_a_closed_classification() {
    assert_eq!(
        classify_migration_ddl("CREATE INDEX users_idx ON users(id)"),
        PostgresDdlExecutionClass::Transactional
    );
    assert!(matches!(
        classify_migration_ddl("CREATE INDEX CONCURRENTLY users_idx ON users(id)"),
        PostgresDdlExecutionClass::RequiresAutocommit { .. }
    ));
    assert!(matches!(
        classify_migration_ddl("VACUUM users"),
        PostgresDdlExecutionClass::RequiresAutocommit { .. }
    ));
    for statement in [
        "CREATE /* qualification */ UNIQUE INDEX CONCURRENTLY users_idx ON users(id)",
        "DROP INDEX CONCURRENTLY users_idx",
        "CREATE DATABASE qualification",
        "DROP DATABASE qualification",
        "CREATE TABLESPACE qualification LOCATION '/tmp/qualification'",
        "DROP TABLESPACE qualification",
        "CREATE SUBSCRIPTION qualification CONNECTION 'redacted' PUBLICATION events",
        "ALTER SUBSCRIPTION qualification DISABLE",
        "DROP SUBSCRIPTION qualification",
        "ALTER SYSTEM SET work_mem = '8MB'",
        "REFRESH MATERIALIZED VIEW CONCURRENTLY user_totals",
        "REINDEX (VERBOSE) INDEX CONCURRENTLY users_idx",
        "CLUSTER users USING users_idx",
    ] {
        assert!(matches!(
            classify_migration_ddl(statement),
            PostgresDdlExecutionClass::RequiresAutocommit { .. }
        ));
    }
}

#[test]
fn operator_plan_rejects_tampered_transaction_and_recovery_boundaries() {
    let mut plan = operator_plan();
    assert!(validate_postgres_migration_plan(&plan).is_ok());
    plan.migrations
        .get_mut(&MigrationId::new("migration"))
        .expect("migration")
        .paths
        .get_mut(&MigrationId::new("baseline"))
        .expect("path")
        .steps
        .remove(0);
    assert!(validate_postgres_migration_plan(&plan).is_err());

    let mut plan = operator_plan();
    let path = plan
        .migrations
        .get_mut(&MigrationId::new("migration"))
        .expect("migration")
        .paths
        .get_mut(&MigrationId::new("baseline"))
        .expect("path");
    path.steps = vec![migration_step(
        "index",
        MigrationExecutionStepKind::Ddl {
            statement: "CREATE INDEX CONCURRENTLY accounts_idx ON accounts(id)".to_string(),
        },
    )];
    assert!(validate_postgres_migration_plan(&plan).is_err());
}

fn operator_plan() -> MigrationExecutionPlan {
    let baseline = MigrationId::new("baseline");
    let migration = MigrationId::new("migration");
    let path = MigrationExecutionPath {
        parent: baseline.clone(),
        input_fingerprint: "a".repeat(64),
        output_fingerprint: "a".repeat(64),
        steps: vec![
            migration_step(
                "begin",
                MigrationExecutionStepKind::Transaction {
                    boundary: MigrationTransactionBoundary::Begin,
                },
            ),
            migration_step(
                "ddl",
                MigrationExecutionStepKind::Ddl {
                    statement: "CREATE TABLE accounts(id bigint)".to_string(),
                },
            ),
            migration_step(
                "commit",
                MigrationExecutionStepKind::Transaction {
                    boundary: MigrationTransactionBoundary::Commit,
                },
            ),
        ],
        rollback: None,
    };
    MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: "postgresql".to_string(),
        target_fingerprint: "a".repeat(64),
        head: migration.clone(),
        topological_order: vec![migration.clone()],
        baseline_fingerprints: BTreeMap::from([(baseline.clone(), "a".repeat(64))]),
        migrations: BTreeMap::from([(
            migration.clone(),
            MigrationExecutionNode {
                id: migration,
                parents: BTreeSet::from([baseline.clone()]),
                provider: MigrationRuntimeConstraint {
                    family: "postgresql".to_string(),
                    minimum_server_version: Some("13.0.0".to_string()),
                    required_capabilities: BTreeSet::new(),
                },
                transaction_requirement: MigrationTransactionRequirement::Required,
                checksum: "b".repeat(64),
                paths: BTreeMap::from([(baseline, path)]),
                author: "qualification".to_string(),
                created_at: "2026-08-31T00:00:00Z".to_string(),
            },
        )]),
    }
}

fn migration_step(name: &str, kind: MigrationExecutionStepKind) -> MigrationExecutionStep {
    MigrationExecutionStep {
        id: MigrationId::new(name),
        input_state: MigrationStateId::new(format!("{name}.input")),
        output_state: MigrationStateId::new(format!("{name}.output")),
        input_fingerprint: "a".repeat(64),
        output_fingerprint: "a".repeat(64),
        checksum: "c".repeat(64),
        kind,
    }
}
