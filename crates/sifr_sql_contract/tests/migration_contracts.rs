#![allow(clippy::expect_used)]

use semver::Version;
use sifr_sql_contract::{
    BackfillContract, Cardinality, CodecIdentity, CompiledStepKind, DataCallbackContract,
    DatabaseType, DdlReflection, DdlRisk, DialectIdentity, EffectContract, MigrationBaseline,
    MigrationCompileError, MigrationCompileErrorKind, MigrationCompiler, MigrationDb,
    MigrationDefinition, MigrationDialect, MigrationGraphDefinition, MigrationNodeId,
    MigrationPlan, MigrationProviderConstraint, MigrationState, MigrationStepDefinition,
    MigrationStepKind, Nullability, ObjectId, ProviderAnalysis, ProviderIdentity,
    ProviderResultField, QueryEffect, ReplayPolicy, SchemaIr, SchemaObject, SchemaObjectKind,
    SifrType, TransactionBoundary, TransactionRequirement, schema_fingerprint, topological_order,
};
use static_assertions::assert_not_impl_any;
use std::collections::{BTreeMap, BTreeSet};

fn id(value: &str) -> MigrationNodeId {
    MigrationNodeId::new(value).expect("test migration identity should be valid")
}

fn object(value: &str) -> ObjectId {
    ObjectId::new(value)
}

fn schema(with_status: bool) -> SchemaIr {
    let table_id = object("public.orders");
    let mut objects = BTreeMap::from([(
        table_id.clone(),
        SchemaObject {
            identity: table_id,
            kind: SchemaObjectKind::Table,
            semantic: BTreeMap::new(),
            dependencies: BTreeSet::new(),
            source: None,
        },
    )]);
    if with_status {
        let status = object("public.orders.status");
        objects.insert(
            status.clone(),
            SchemaObject {
                identity: status,
                kind: SchemaObjectKind::Column,
                semantic: BTreeMap::new(),
                dependencies: BTreeSet::from([object("public.orders")]),
                source: None,
            },
        );
    }
    SchemaIr {
        format_version: 1,
        provider: ProviderIdentity {
            package_id: "sifr-sql-postgresql".to_string(),
            package_version: Version::new(1, 0, 0),
            package_source: "registry+https://example.invalid".to_string(),
            package_graph_digest: "1".repeat(64),
            compiler_components: BTreeMap::new(),
        },
        dialect: DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18.0.0".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::from(["transactional-ddl".to_string()]),
        },
        objects,
    }
}

fn fingerprint(value: &SchemaIr) -> String {
    schema_fingerprint(value)
        .expect("test schema should fingerprint")
        .as_str()
        .to_string()
}

fn effect(kind: QueryEffect, referenced: &[&str], affected: &[&str]) -> EffectContract {
    EffectContract::new(
        kind,
        referenced.iter().map(|value| object(value)).collect(),
        affected.iter().map(|value| object(value)).collect(),
    )
    .expect("test effect should be valid")
}

fn data_analysis() -> ProviderAnalysis {
    ProviderAnalysis {
        server_profile: "postgresql-18".to_string(),
        normalized_statement: "UPDATE public.orders SET status = 'pending'".to_string(),
        parameters: Vec::new(),
        result_fields: Vec::new(),
        cardinality: Cardinality::Empty,
        effects: effect(
            QueryEffect::Write,
            &["public.orders.status"],
            &["public.orders"],
        ),
        semantic_flags: BTreeSet::new(),
        required_capabilities: BTreeSet::from(["sql.query.update".to_string()]),
    }
}

fn assertion_analysis() -> ProviderAnalysis {
    ProviderAnalysis {
        server_profile: "postgresql-18".to_string(),
        normalized_statement: "SELECT true AS valid".to_string(),
        parameters: Vec::new(),
        result_fields: vec![ProviderResultField {
            name: "valid".to_string(),
            sifr_type: SifrType::Bool,
            database_type: DatabaseType::Boolean,
            nullability: Nullability::NonNull,
            codec: CodecIdentity::new("postgresql.bool.v1")
                .expect("test codec identity should be valid"),
            source_object: None,
        }],
        cardinality: Cardinality::MANY,
        effects: effect(QueryEffect::Read, &["public.orders.status"], &[]),
        semantic_flags: BTreeSet::new(),
        required_capabilities: BTreeSet::from(["sql.query.select".to_string()]),
    }
}

fn step(value: &str, kind: MigrationStepKind) -> MigrationStepDefinition {
    MigrationStepDefinition {
        id: id(value),
        kind,
    }
}

struct TestDialect {
    before: SchemaIr,
    after: SchemaIr,
    capabilities: BTreeSet<String>,
}

impl MigrationDialect for TestDialect {
    fn family(&self) -> &str {
        &self.before.dialect.family
    }

    fn server_version(&self) -> &str {
        &self.after.dialect.server_version
    }

    fn capabilities(&self) -> &BTreeSet<String> {
        &self.capabilities
    }

    fn reflect_ddl(
        &self,
        _input: &SchemaIr,
        statement: &str,
    ) -> Result<DdlReflection, MigrationCompileError> {
        match statement {
            "ADD STATUS" => Ok(DdlReflection::Reflected {
                schema: self.after.clone(),
                risk: DdlRisk {
                    lock_risks: BTreeSet::from(["orders-access-exclusive".to_string()]),
                    data_rewrites: BTreeSet::new(),
                },
            }),
            "DROP STATUS" => Ok(DdlReflection::Reflected {
                schema: self.before.clone(),
                risk: DdlRisk::default(),
            }),
            "OPAQUE" => Ok(DdlReflection::Opaque),
            _ => Err(MigrationCompileError {
                kind: MigrationCompileErrorKind::DdlReflection,
                message: "unknown test DDL".to_string(),
            }),
        }
    }
}

fn definition() -> (TestDialect, MigrationGraphDefinition) {
    let before = schema(false);
    let after = schema(true);
    let baseline = id("baseline");
    let migration = id("2026_08_add_status");
    let forward = vec![
        step(
            "begin",
            MigrationStepKind::Transaction {
                boundary: TransactionBoundary::Begin,
            },
        ),
        step(
            "add-status",
            MigrationStepKind::Ddl {
                statement: "ADD STATUS".to_string(),
                declared_effect: None,
            },
        ),
        step(
            "fill-status",
            MigrationStepKind::SqlData {
                analysis: data_analysis(),
            },
        ),
        step(
            "assert-status",
            MigrationStepKind::Assertion {
                analysis: assertion_analysis(),
            },
        ),
        step(
            "backfill-status",
            MigrationStepKind::Backfill {
                contract: BackfillContract {
                    analysis: data_analysis(),
                    maximum_batch_rows: 500,
                    replay: ReplayPolicy::Idempotent {
                        progress_key: vec![object("public.orders.status")],
                    },
                },
            },
        ),
        step(
            "recovery",
            MigrationStepKind::RecoveryPoint {
                name: "status-filled".to_string(),
            },
        ),
        step(
            "commit",
            MigrationStepKind::Transaction {
                boundary: TransactionBoundary::Commit,
            },
        ),
    ];
    let rollback = vec![
        step(
            "rollback-begin",
            MigrationStepKind::Transaction {
                boundary: TransactionBoundary::Begin,
            },
        ),
        step(
            "drop-status",
            MigrationStepKind::Ddl {
                statement: "DROP STATUS".to_string(),
                declared_effect: None,
            },
        ),
        step(
            "rollback-commit",
            MigrationStepKind::Transaction {
                boundary: TransactionBoundary::Commit,
            },
        ),
    ];
    let graph = MigrationGraphDefinition {
        format_version: 1,
        baselines: BTreeMap::from([(
            baseline.clone(),
            MigrationBaseline {
                id: baseline.clone(),
                schema: before.clone(),
            },
        )]),
        migrations: BTreeMap::from([(
            migration.clone(),
            MigrationDefinition {
                id: migration,
                parents: BTreeSet::from([baseline.clone()]),
                input_fingerprints: BTreeMap::from([(baseline, fingerprint(&before))]),
                output_fingerprint: fingerprint(&after),
                provider: MigrationProviderConstraint {
                    family: "postgresql".to_string(),
                    minimum_server_version: Some("13.0.0".to_string()),
                    required_capabilities: BTreeSet::from(["transactional-ddl".to_string()]),
                },
                transaction_requirement: TransactionRequirement::Required,
                steps: forward,
                rollback: Some(rollback),
                author: "database-team".to_string(),
                created_at: "2026-08-31T00:00:00Z".to_string(),
            },
        )]),
        target_schema: after.clone(),
    };
    (
        TestDialect {
            before,
            after,
            capabilities: BTreeSet::from(["transactional-ddl".to_string()]),
        },
        graph,
    )
}

#[test]
fn compiler_produces_deterministic_intermediate_states_and_explicit_rollback() {
    let (dialect, graph) = definition();
    let compiler = MigrationCompiler::new(&dialect);
    let first = compiler
        .compile(&graph)
        .expect("migration graph should compile");
    let second = compiler
        .compile(&graph)
        .expect("second compilation should pass");
    assert_eq!(first, second);
    assert_eq!(first.head, id("2026_08_add_status"));
    assert_eq!(first.target_fingerprint, fingerprint(&dialect.after));
    let migration = &first.migrations[&id("2026_08_add_status")];
    let path = &migration.paths[&id("baseline")];
    assert_eq!(path.steps.len(), 7);
    assert!(path.rollback.as_ref().is_some_and(|steps| steps.len() == 3));
    assert!(matches!(
        path.steps[1].kind,
        CompiledStepKind::ReflectedDdl { .. }
    ));
    assert_eq!(path.steps[1].output_state, path.steps[2].input_state);
    assert!(first.impacts.iter().any(|impact| {
        impact.step == id("add-status") && impact.lock_risks.contains("orders-access-exclusive")
    }));
    assert!(first.impacts.iter().any(|impact| {
        impact.step == id("fill-status") && impact.data_rewrites.contains("public.orders")
    }));
}

#[test]
fn compiler_rejects_opaque_ddl_without_effect_and_invalid_intermediate_references() {
    let (dialect, mut graph) = definition();
    let migration = graph
        .migrations
        .get_mut(&id("2026_08_add_status"))
        .expect("test migration should exist");
    migration.steps[1] = step(
        "opaque",
        MigrationStepKind::Ddl {
            statement: "OPAQUE".to_string(),
            declared_effect: None,
        },
    );
    let error = MigrationCompiler::new(&dialect)
        .compile(&graph)
        .expect_err("opaque DDL without an effect must fail");
    assert_eq!(error.kind, MigrationCompileErrorKind::DdlReflection);

    let (_, mut graph) = definition();
    let migration = graph
        .migrations
        .get_mut(&id("2026_08_add_status"))
        .expect("test migration should exist");
    let MigrationStepKind::SqlData { analysis } = &mut migration.steps[2].kind else {
        panic!("expected SQL data step");
    };
    analysis
        .effects
        .referenced_objects
        .insert(object("public.not_created"));
    let error = MigrationCompiler::new(&dialect)
        .compile(&graph)
        .expect_err("future object references must fail");
    assert_eq!(error.kind, MigrationCompileErrorKind::UnknownSchemaObject);
}

#[test]
fn compiler_rejects_invalid_assertions_callbacks_backfills_and_transactions() {
    let (dialect, mut graph) = definition();
    let migration = graph
        .migrations
        .get_mut(&id("2026_08_add_status"))
        .expect("test migration should exist");
    let MigrationStepKind::Assertion { analysis } = &mut migration.steps[3].kind else {
        panic!("expected assertion step");
    };
    analysis.result_fields[0].nullability = Nullability::Nullable;
    let error = MigrationCompiler::new(&dialect)
        .compile(&graph)
        .expect_err("nullable assertions must fail");
    assert_eq!(error.kind, MigrationCompileErrorKind::InvalidAssertion);

    let (_, mut graph) = definition();
    let migration = graph
        .migrations
        .get_mut(&id("2026_08_add_status"))
        .expect("test migration should exist");
    migration.steps[2] = step(
        "callback",
        MigrationStepKind::SifrData {
            callback: DataCallbackContract {
                symbol: "fill".to_string(),
                referenced_objects: BTreeSet::from([object("public.orders")]),
                affected_objects: BTreeSet::from([object("public.orders")]),
                is_async: true,
                returns_result: true,
                nonescaping: false,
            },
        },
    );
    let error = MigrationCompiler::new(&dialect)
        .compile(&graph)
        .expect_err("escaping callbacks must fail");
    assert_eq!(error.kind, MigrationCompileErrorKind::InvalidDataCallback);

    let (_, mut graph) = definition();
    let migration = graph
        .migrations
        .get_mut(&id("2026_08_add_status"))
        .expect("test migration should exist");
    let MigrationStepKind::Backfill { contract } = &mut migration.steps[4].kind else {
        panic!("expected backfill step");
    };
    contract.maximum_batch_rows = 0;
    let error = MigrationCompiler::new(&dialect)
        .compile(&graph)
        .expect_err("unbounded backfills must fail");
    assert_eq!(error.kind, MigrationCompileErrorKind::InvalidBackfill);

    let (_, mut graph) = definition();
    let migration = graph
        .migrations
        .get_mut(&id("2026_08_add_status"))
        .expect("test migration should exist");
    migration.steps.pop();
    let error = MigrationCompiler::new(&dialect)
        .compile(&graph)
        .expect_err("open transactions must fail");
    assert_eq!(error.kind, MigrationCompileErrorKind::InvalidTransaction);
}

struct Initial;
struct After;

impl MigrationState for Initial {
    const IDENTITY: &'static str = "state.initial";
}

impl MigrationState for After {
    const IDENTITY: &'static str = "state.after";
}

assert_not_impl_any!(MigrationPlan<Initial>: Clone, Copy);
assert_not_impl_any!(MigrationDb<'static, Initial>: Clone, Copy);

#[test]
fn affine_plan_and_scoped_database_enforce_nominal_state_identity() {
    let (_, graph) = definition();
    let migration = &MigrationCompiler::new(&TestDialect {
        before: schema(false),
        after: schema(true),
        capabilities: BTreeSet::from(["transactional-ddl".to_string()]),
    })
    .compile(&graph)
    .expect("migration should compile")
    .migrations[&id("2026_08_add_status")];
    let mut transition = migration.paths[&id("baseline")].steps[0].clone();
    transition.input_state = sifr_sql_contract::MigrationStateIdentity::new(Initial::IDENTITY);
    transition.output_state = sifr_sql_contract::MigrationStateIdentity::new(After::IDENTITY);
    let after = MigrationPlan::<Initial>::new()
        .transition::<After>(transition)
        .expect("matching nominal transition should consume the plan");
    assert_eq!(after.state().as_str(), After::IDENTITY);

    let objects = BTreeSet::from([object("public.orders")]);
    let db = MigrationDb::<Initial>::new(&objects);
    db.require_object(&object("public.orders"))
        .expect("state object should be available");
    assert_eq!(
        db.require_object(&object("public.future"))
            .expect_err("future object must be unavailable")
            .kind,
        MigrationCompileErrorKind::UnknownSchemaObject
    );
}

#[test]
fn migration_identity_fuzz_smoke_is_total_and_fail_closed() {
    for byte in u8::MIN..=u8::MAX {
        let value = String::from_utf8_lossy(&[byte; 8]).into_owned();
        let result = MigrationNodeId::new(value.clone());
        if let Ok(identity) = result {
            assert_eq!(identity.as_str(), value);
            assert!(value.bytes().all(|candidate| {
                candidate.is_ascii_alphanumeric() || matches!(candidate, b'-' | b'_' | b'.')
            }));
        }
    }
    for length in [0, 1, 127, 128, 129, 1_024] {
        let value = "a".repeat(length);
        assert_eq!(
            MigrationNodeId::new(value).is_ok(),
            (1..=128).contains(&length)
        );
    }
}

#[test]
fn topological_order_property_is_stable_for_linear_graphs() {
    let base_schema = schema(false);
    for count in 1..=64 {
        let baseline = id("baseline");
        let mut migrations = BTreeMap::new();
        let mut parent = baseline.clone();
        let mut expected = Vec::new();
        for index in 0..count {
            let migration_id = id(&format!("m{index:03}"));
            migrations.insert(
                migration_id.clone(),
                MigrationDefinition {
                    id: migration_id.clone(),
                    parents: BTreeSet::from([parent.clone()]),
                    input_fingerprints: BTreeMap::from([(parent.clone(), "a".repeat(64))]),
                    output_fingerprint: "a".repeat(64),
                    provider: MigrationProviderConstraint {
                        family: "postgresql".to_string(),
                        minimum_server_version: None,
                        required_capabilities: BTreeSet::new(),
                    },
                    transaction_requirement: TransactionRequirement::Optional,
                    steps: Vec::new(),
                    rollback: None,
                    author: "property".to_string(),
                    created_at: "2026-08-31".to_string(),
                },
            );
            expected.push(migration_id.clone());
            parent = migration_id;
        }
        let graph = MigrationGraphDefinition {
            format_version: 1,
            baselines: BTreeMap::from([(
                baseline.clone(),
                MigrationBaseline {
                    id: baseline,
                    schema: base_schema.clone(),
                },
            )]),
            migrations,
            target_schema: base_schema.clone(),
        };
        assert_eq!(
            topological_order(&graph).expect("linear graph should have a stable order"),
            expected
        );
    }
}

#[test]
fn compiler_rejects_schema_changing_sibling_branches_that_cannot_be_sequenced() {
    let before = schema(false);
    let after = schema(true);
    let before_fingerprint = fingerprint(&before);
    let after_fingerprint = fingerprint(&after);
    let baseline = id("baseline");
    let left = id("left");
    let right = id("right");
    let merge = id("merge");
    let provider = MigrationProviderConstraint {
        family: "postgresql".to_string(),
        minimum_server_version: Some("13.0.0".to_string()),
        required_capabilities: BTreeSet::new(),
    };
    let branch = |identity: MigrationNodeId| MigrationDefinition {
        id: identity,
        parents: BTreeSet::from([baseline.clone()]),
        input_fingerprints: BTreeMap::from([(baseline.clone(), before_fingerprint.clone())]),
        output_fingerprint: after_fingerprint.clone(),
        provider: provider.clone(),
        transaction_requirement: TransactionRequirement::Optional,
        steps: vec![step(
            "add-status",
            MigrationStepKind::Ddl {
                statement: "ADD STATUS".to_string(),
                declared_effect: None,
            },
        )],
        rollback: None,
        author: "test".to_string(),
        created_at: "2026-08-31".to_string(),
    };
    let graph = MigrationGraphDefinition {
        format_version: 1,
        baselines: BTreeMap::from([(
            baseline.clone(),
            MigrationBaseline {
                id: baseline.clone(),
                schema: before.clone(),
            },
        )]),
        migrations: BTreeMap::from([
            (left.clone(), branch(left.clone())),
            (right.clone(), branch(right.clone())),
            (
                merge.clone(),
                MigrationDefinition {
                    id: merge,
                    parents: BTreeSet::from([left.clone(), right.clone()]),
                    input_fingerprints: BTreeMap::from([
                        (left, after_fingerprint.clone()),
                        (right, after_fingerprint.clone()),
                    ]),
                    output_fingerprint: after_fingerprint,
                    provider,
                    transaction_requirement: TransactionRequirement::Optional,
                    steps: vec![step(
                        "merge-point",
                        MigrationStepKind::RecoveryPoint {
                            name: "branches-joined".to_string(),
                        },
                    )],
                    rollback: None,
                    author: "test".to_string(),
                    created_at: "2026-08-31".to_string(),
                },
            ),
        ]),
        target_schema: after.clone(),
    };
    let failure = MigrationCompiler::new(&TestDialect {
        before,
        after,
        capabilities: BTreeSet::new(),
    })
    .compile(&graph)
    .expect_err("schema-changing sibling branches must be rejected");
    assert_eq!(failure.kind, MigrationCompileErrorKind::InvalidGraph);
    assert!(failure.message.contains("cannot be sequenced"));
}
