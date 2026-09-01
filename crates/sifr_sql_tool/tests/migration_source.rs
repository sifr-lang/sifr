#![allow(clippy::expect_used)]

use semver::Version;
use sifr_sql_contract::{
    Cardinality, CodecIdentity, DatabaseType, DdlReflection, DdlRisk, DialectIdentity,
    EffectContract, MigrationBaseline, MigrationCompileError, MigrationDialect, MigrationNodeId,
    Nullability, ObjectId, ProviderAnalysis, ProviderIdentity, ProviderResultField, QueryEffect,
    SchemaIr, SchemaObject, SchemaObjectKind, SifrType,
};
use sifr_sql_tool::{compile_migration_sources, load_migration_source_inputs};
use std::collections::{BTreeMap, BTreeSet};

struct SourceDialect {
    before: SchemaIr,
    after: SchemaIr,
    capabilities: BTreeSet<String>,
}

impl MigrationDialect for SourceDialect {
    fn family(&self) -> &str {
        "postgresql"
    }

    fn server_version(&self) -> &str {
        "18"
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
            "ALTER TABLE orders ADD COLUMN status TEXT NULL"
            | "ALTER TABLE orders ALTER COLUMN status SET NOT NULL" => {
                Ok(DdlReflection::Reflected {
                    schema: self.after.clone(),
                    risk: DdlRisk::default(),
                })
            }
            "ALTER TABLE orders DROP COLUMN status" => Ok(DdlReflection::Reflected {
                schema: self.before.clone(),
                risk: DdlRisk::default(),
            }),
            _ => Ok(DdlReflection::Opaque),
        }
    }
}

#[test]
fn checked_source_build_is_deterministic_and_preserves_executable_literals() {
    let temporary = tempfile::tempdir().expect("temporary migration workspace");
    let source_root = temporary.path().join("migrations/app");
    std::fs::create_dir_all(source_root.join("baselines")).expect("baseline directory");
    std::fs::write(
        source_root.join("2026_08_add_status.sifr"),
        include_str!(
            "../../../verification/areas/sql_platform/fixtures/migration_source/add_status.sifr"
        ),
    )
    .expect("migration source");
    let before = schema(false);
    let after = schema(true);
    let baseline = MigrationBaseline {
        id: id("2026_07_previous"),
        schema: before.clone(),
    };
    std::fs::write(
        source_root.join("baselines/2026_07_previous.json"),
        serde_json::to_vec_pretty(&baseline).expect("baseline JSON"),
    )
    .expect("baseline record");

    let inputs = load_migration_source_inputs(temporary.path(), "app", compile_source)
        .expect("checked migration source inputs");
    let dialect = SourceDialect {
        before,
        after: after.clone(),
        capabilities: BTreeSet::from([
            "sql.query.select".to_string(),
            "sql.query.update".to_string(),
        ]),
    };
    let compile = || {
        compile_migration_sources(
            &dialect,
            after.clone(),
            inputs.baselines.clone(),
            inputs.declarations.clone(),
            analyze,
        )
        .expect("source migration graph")
    };
    let first = compile();
    let second = compile();
    assert_eq!(first, second);
    let migration = first
        .migrations
        .get(&id("2026_08_add_status"))
        .expect("compiled migration");
    assert_eq!(migration.author, "Sifr SQL; team=qualification");
    let path = migration
        .paths
        .get(&id("2026_07_previous"))
        .expect("compiled parent path");
    assert!(
        path.rollback
            .as_ref()
            .is_some_and(|steps| !steps.is_empty())
    );
    assert!(path.steps.iter().any(|step| {
        matches!(
            &step.kind,
            sifr_sql_contract::CompiledStepKind::SqlData { statement, normalized_statement }
                if statement.contains("'pending'") && normalized_statement == "normalized update"
        )
    }));
}

#[test]
fn source_loading_rejects_path_escape_and_filename_identity_drift() {
    let temporary = tempfile::tempdir().expect("temporary migration workspace");
    assert!(load_migration_source_inputs(temporary.path(), "../outside", compile_source).is_err());

    let root = temporary.path().join("migrations/app");
    std::fs::create_dir_all(root.join("baselines")).expect("baseline directory");
    std::fs::write(
        root.join("wrong_name.sifr"),
        include_str!(
            "../../../verification/areas/sql_platform/fixtures/migration_source/add_status.sifr"
        ),
    )
    .expect("migration source");
    let failure = load_migration_source_inputs(temporary.path(), "app", compile_source)
        .expect_err("filename drift must fail");
    assert!(failure.message.contains("must equal identity"));
}

fn compile_source(
    source: &str,
) -> Result<Vec<sifr_sql_contract::MigrationSourceDeclaration>, String> {
    sifr_driver::compile_sql_migration_source(source).map_err(|failures| {
        failures
            .into_iter()
            .map(|failure| failure.message)
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn analyze(_schema: &SchemaIr, statement: &str) -> Result<ProviderAnalysis, String> {
    if statement.starts_with("UPDATE") {
        return Ok(ProviderAnalysis {
            server_profile: "postgresql-18".to_string(),
            normalized_statement: "normalized update".to_string(),
            parameters: Vec::new(),
            result_fields: Vec::new(),
            cardinality: Cardinality::Empty,
            effects: EffectContract::new(
                QueryEffect::Write,
                BTreeSet::from([ObjectId::new("public.orders.status")]),
                BTreeSet::from([ObjectId::new("public.orders")]),
            )
            .map_err(|failure| failure.message)?,
            accessed_objects: BTreeSet::from([
                ObjectId::new("public.orders"),
                ObjectId::new("public.orders.status"),
            ]),
            semantic_flags: BTreeSet::new(),
            required_capabilities: BTreeSet::from(["sql.query.update".to_string()]),
        });
    }
    Ok(ProviderAnalysis {
        server_profile: "postgresql-18".to_string(),
        normalized_statement: "normalized assertion".to_string(),
        parameters: Vec::new(),
        result_fields: vec![ProviderResultField {
            name: "valid".to_string(),
            sifr_type: SifrType::Bool,
            database_type: DatabaseType::Boolean,
            nullability: Nullability::NonNull,
            codec: CodecIdentity::new("postgresql.bool.binary.v1")
                .map_err(|failure| failure.message)?,
            source_object: Some(ObjectId::new("public.orders.status")),
        }],
        cardinality: Cardinality::MANY,
        effects: EffectContract::new(
            QueryEffect::Read,
            BTreeSet::from([ObjectId::new("public.orders.status")]),
            BTreeSet::new(),
        )
        .map_err(|failure| failure.message)?,
        accessed_objects: BTreeSet::from([
            ObjectId::new("public.orders"),
            ObjectId::new("public.orders.status"),
        ]),
        semantic_flags: BTreeSet::new(),
        required_capabilities: BTreeSet::from(["sql.query.select".to_string()]),
    })
}

fn id(value: &str) -> MigrationNodeId {
    MigrationNodeId::new(value).expect("valid migration identity")
}

fn schema(with_status: bool) -> SchemaIr {
    let table = ObjectId::new("public.orders");
    let mut objects = BTreeMap::from([(
        table.clone(),
        SchemaObject {
            identity: table.clone(),
            kind: SchemaObjectKind::Table,
            semantic: BTreeMap::new(),
            dependencies: BTreeSet::new(),
            source: None,
        },
    )]);
    if with_status {
        let status = ObjectId::new("public.orders.status");
        objects.insert(
            status.clone(),
            SchemaObject {
                identity: status,
                kind: SchemaObjectKind::Column,
                semantic: BTreeMap::new(),
                dependencies: BTreeSet::from([table]),
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
            server_version: "18".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        objects,
    }
}
