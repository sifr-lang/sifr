#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use semver::Version;
use serde_json::Value;
use sifr_sql_contract::{
    DatabaseType, DialectSemantics, Nullability, ObjectId, ProviderAnalysisError, ProviderIdentity,
    SchemaObjectKind, normalize_schema,
};
use sifr_sql_postgresql::{
    ExpressionKind, LibpgQueryParser, PostgresAnalyzer, PostgresCompilerComponent,
    PostgresComponentRequest, PostgresComponentResponse, PostgresDiagnosticCode, PostgresParser,
    PostgresTypeName, PostgresTypeRegistry, StatementKind, rewrite_parameter_slots,
};
use std::collections::BTreeMap;

#[test]
fn parser_preserves_boolean_operator_subquery_and_conflict_shapes() {
    let parser = LibpgQueryParser;
    let parsed = parser
        .parse("SELECT NOT active FROM users WHERE $1 IN (id, 2)")
        .unwrap();
    let StatementKind::Select(select) = &parsed[0].kind else {
        panic!("expected SELECT");
    };
    assert!(matches!(
        select.targets[0].expression.kind,
        ExpressionKind::Unary { ref operator, .. } if operator == "NOT"
    ));
    assert!(matches!(
        select.predicate.as_ref().unwrap().kind,
        ExpressionKind::InList { .. }
    ));

    let parsed = parser
        .parse("SELECT id FROM users WHERE $1 IN (SELECT id FROM users)")
        .unwrap();
    let StatementKind::Select(select) = &parsed[0].kind else {
        panic!("expected SELECT");
    };
    assert!(matches!(
        select.predicate.as_ref().unwrap().kind,
        ExpressionKind::SubqueryComparison { .. }
    ));

    let parsed = parser
        .parse(
            "INSERT INTO users(id, name) VALUES ($1, $2) \
             ON CONFLICT(id) WHERE id > $3 DO UPDATE SET name = excluded.name WHERE id = $1",
        )
        .unwrap();
    let StatementKind::Insert(insert) = &parsed[0].kind else {
        panic!("expected INSERT");
    };
    let conflict = insert.conflict.as_ref().unwrap();
    assert!(conflict.target_predicate.is_some());
    assert!(conflict.update_predicate.is_some());

    assert!(
        parser
            .parse("SELECT id FROM users WHERE id = ANY($1)")
            .is_err()
    );
}

#[test]
fn parameter_rewrite_handles_escape_strings_without_exposing_placeholders() {
    assert_eq!(
        rewrite_parameter_slots(r"note = E'escaped\\\' $1' AND id = $1", 2).unwrap(),
        r"note = E'escaped\\\' $1' AND id = $3"
    );
    assert_eq!(
        rewrite_parameter_slots(r"prefixE'$1' || $1", 1).unwrap(),
        r"prefixE'$1' || $2"
    );
}

#[test]
fn exact_named_types_and_qualified_nominals_do_not_collide() {
    let mut types = PostgresTypeRegistry::new(LibpgQueryParser.server_major());
    let varchar = types
        .resolve(&type_name(&["varchar"], &[32]))
        .expect("varchar(32)");
    let character = types
        .resolve(&type_name(&["char"], &[32]))
        .expect("char(32)");
    let text = types.resolve(&type_name(&["text"], &[])).expect("text");
    assert_ne!(varchar.database_type, text.database_type);
    assert_ne!(varchar.database_type, character.database_type);
    assert_ne!(
        types.codec_identity(&varchar.database_type).unwrap(),
        types.codec_identity(&character.database_type).unwrap()
    );
    assert_ne!(
        types.codec_identity(&varchar.database_type).unwrap(),
        types.codec_identity(&text.database_type).unwrap()
    );
    assert_eq!(
        types
            .resolve(&type_name(&["timestamp"], &[3]))
            .unwrap()
            .database_type,
        DatabaseType::LocalDateTime { precision: 3 }
    );
    assert_ne!(
        types.codec_identity(&DatabaseType::IpAddress).unwrap(),
        types.codec_identity(&DatabaseType::IpNetwork).unwrap()
    );

    types.add_nominal(
        &["a".to_string(), "mood".to_string()],
        DatabaseType::Enum {
            identity: ObjectId::new("a.mood"),
        },
    );
    types.add_nominal(
        &["b".to_string(), "mood".to_string()],
        DatabaseType::Enum {
            identity: ObjectId::new("b.mood"),
        },
    );
    assert!(types.resolve(&type_name(&["mood"], &[])).is_none());
    assert_ne!(
        types
            .resolve(&type_name(&["a", "mood"], &[]))
            .unwrap()
            .database_type,
        types
            .resolve(&type_name(&["b", "mood"], &[]))
            .unwrap()
            .database_type
    );
}

#[test]
fn ddl_views_and_writes_use_real_results_and_declaration_order() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let major = LibpgQueryParser.server_major();
    let response = component.execute(PostgresComponentRequest::NormalizeSchema {
        provider: provider(),
        server_major: major,
        documents: vec![
            (
                "db/tables.sql".to_string(),
                "CREATE TYPE a.mood AS ENUM ('ok'); \
                 CREATE TYPE b.mood AS ENUM ('ok'); \
                 CREATE TABLE ordered (z bigint NOT NULL, a varchar(20) NOT NULL);"
                    .to_string(),
            ),
            (
                "db/views.sql".to_string(),
                "CREATE VIEW ordered_view AS SELECT z, a FROM ordered; \
                 CREATE MATERIALIZED VIEW ordered_materialized AS SELECT a FROM ordered;"
                    .to_string(),
            ),
        ],
    });
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("schema must normalize: {response:?}");
    };
    let schema = normalize_schema(provider(), output.dialect, output.documents).unwrap();
    let views = schema
        .objects
        .values()
        .filter(|object| {
            matches!(
                object.kind,
                SchemaObjectKind::View | SchemaObjectKind::MaterializedView
            )
        })
        .map(|object| (&object.identity, &object.semantic, &object.dependencies))
        .collect::<Vec<_>>();
    insta::assert_debug_snapshot!(views);

    let inserted = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: "INSERT INTO ordered VALUES ($1, $2)".to_string(),
        sifr_document: "src/write.sifr".to_string(),
        sifr_start: 0,
        sifr_end: 36,
    });
    let PostgresComponentResponse::Query(inserted) = inserted else {
        panic!("ordered INSERT must analyze: {inserted:?}");
    };
    assert!(matches!(
        inserted.parameters[0].database_type,
        DatabaseType::Integer { .. }
    ));
    assert!(matches!(
        inserted.parameters[1].database_type,
        DatabaseType::Named { .. }
    ));

    for source in [
        "INSERT INTO ordered(z, a) VALUES (NULL, 'value')",
        "INSERT INTO ordered(z, a) SELECT NULL, 'value'",
    ] {
        assert!(matches!(
            component.execute(PostgresComponentRequest::AnalyzeQuery {
                schema: schema.clone(),
                source: source.to_string(),
                sifr_document: "src/write.sifr".to_string(),
                sifr_start: 0,
                sifr_end: u32::try_from(source.len()).unwrap(),
            }),
            PostgresComponentResponse::Diagnostic(_)
        ));
    }
}

#[test]
fn core_operators_aggregates_and_dialect_diagnostics_are_preserved() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let schema_ir = schema(&component);
    for source in [
        "SELECT sum(id) AS total, avg(id) AS average FROM users",
        "SELECT id % 2 AS remainder FROM users",
        "SELECT 1::bigint AS widened",
        "SELECT name LIKE $1 AS matches FROM users",
        "SELECT id FROM users WHERE $1 IN (SELECT id FROM users)",
        "INSERT INTO users(id, name) VALUES ($1, $2) ON CONFLICT(id) WHERE id > $3 DO UPDATE SET name = excluded.name WHERE users.id = $1 RETURNING id",
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema_ir.clone(),
            source: source.to_string(),
            sifr_document: "src/core.sifr".to_string(),
            sifr_start: 10,
            sifr_end: 10 + u32::try_from(source.len()).unwrap(),
        });
        assert!(
            matches!(response, PostgresComponentResponse::Query(_)),
            "{response:?}"
        );
    }

    let catalog = sifr_sql_postgresql::PostgresCatalog::from_schema(
        &schema_ir,
        PostgresTypeRegistry::new(LibpgQueryParser.server_major()),
    )
    .unwrap();
    let analyzer = PostgresAnalyzer::new(LibpgQueryParser, catalog);
    let error = analyzer
        .analyze(&schema_fingerprint(&schema_ir), "SELECT missing FROM users")
        .unwrap_err();
    let ProviderAnalysisError::Diagnostic(diagnostic) = error else {
        panic!("provider diagnostic must survive the contract boundary");
    };
    assert_eq!(
        diagnostic.code,
        PostgresDiagnosticCode::UnknownColumn.as_str()
    );
    assert!(diagnostic.primary.end > diagnostic.primary.start);
}

#[test]
fn checked_live_server_facts_match_provider_analysis_for_the_selected_major() {
    let evidence: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../verification/areas/sql_platform/data/postgresql_server_matrix.json"
    )))
    .unwrap();
    let major = u64::from(LibpgQueryParser.server_major());
    let row = evidence["servers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["major"].as_u64() == Some(major))
        .expect("live row for selected major");
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let schema_ir = schema(&component);
    let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema_ir.clone(),
        source: "SELECT id, name, nickname FROM users WHERE id = $1 AND name = $2".to_string(),
        sifr_document: "src/differential.sifr".to_string(),
        sifr_start: 0,
        sifr_end: 66,
    });
    let PostgresComponentResponse::Query(analysis) = response else {
        panic!("differential query must analyze: {response:?}");
    };
    assert_eq!(
        analysis
            .parameters
            .iter()
            .map(|parameter| postgres_type_name(&parameter.database_type))
            .collect::<Vec<_>>()
            .join(","),
        row["parameter_types"]
            .as_str()
            .unwrap()
            .trim_matches(['{', '}'])
    );
    assert_eq!(
        analysis
            .result_fields
            .iter()
            .map(|field| postgres_type_name(&field.database_type))
            .collect::<Vec<_>>()
            .join("|"),
        row["result_types"].as_str().unwrap()
    );
    assert_eq!(
        analysis
            .result_fields
            .iter()
            .map(|field| format!(
                "{}:{}",
                field.name,
                if field.nullability == Nullability::NonNull {
                    "t"
                } else {
                    "f"
                }
            ))
            .collect::<Vec<_>>()
            .join("|"),
        row["nullability"].as_str().unwrap()
    );

    let write = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema_ir.clone(),
        source: "INSERT INTO users(id, name) VALUES (1, 'second') ON CONFLICT(id) DO UPDATE SET name = excluded.name RETURNING id::text || '|' || name AS written".to_string(),
        sifr_document: "src/differential.sifr".to_string(),
        sifr_start: 70,
        sifr_end: 220,
    });
    let PostgresComponentResponse::Query(write) = write else {
        panic!("differential write must analyze: {write:?}");
    };
    assert_eq!(write.effects.effect, sifr_sql_contract::QueryEffect::Write);
    assert_eq!(write.result_fields.len(), 1);
    assert_eq!(
        postgres_type_name(&write.result_fields[0].database_type),
        "text"
    );
    assert!(!row["write_result"].as_str().unwrap().is_empty());

    let diagnostic = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema_ir,
        source: "SELECT missing_column FROM users".to_string(),
        sifr_document: "src/differential.sifr".to_string(),
        sifr_start: 230,
        sifr_end: 262,
    });
    let PostgresComponentResponse::Diagnostic(diagnostic) = diagnostic else {
        panic!("differential error must produce a diagnostic");
    };
    assert_eq!(row["diagnostic_sqlstate"].as_str(), Some("42703"));
    assert_eq!(diagnostic.code, PostgresDiagnosticCode::UnknownColumn);
}

fn schema(component: &PostgresCompilerComponent<LibpgQueryParser>) -> sifr_sql_contract::SchemaIr {
    let response = component.execute(PostgresComponentRequest::NormalizeSchema {
        provider: provider(),
        server_major: LibpgQueryParser.server_major(),
        documents: vec![(
            "db/schema.sql".to_string(),
            "CREATE TABLE users (id bigint PRIMARY KEY, name text NOT NULL, nickname text);"
                .to_string(),
        )],
    });
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("schema must normalize: {response:?}");
    };
    normalize_schema(provider(), output.dialect, output.documents).unwrap()
}

fn type_name(path: &[&str], modifiers: &[i64]) -> PostgresTypeName {
    PostgresTypeName {
        path: path.iter().map(|value| (*value).to_string()).collect(),
        modifiers: modifiers.to_vec(),
        array_dimensions: 0,
    }
}

fn postgres_type_name(ty: &DatabaseType) -> &'static str {
    match ty {
        DatabaseType::Integer {
            width: sifr_sql_contract::IntegerWidth::Bits64,
            ..
        } => "bigint",
        DatabaseType::Text { .. } => "text",
        DatabaseType::Named { identity, .. } if identity.as_str() == "pg_catalog.varchar" => {
            "character varying"
        }
        _ => "unsupported",
    }
}

fn schema_fingerprint(schema: &sifr_sql_contract::SchemaIr) -> String {
    sifr_sql_contract::schema_fingerprint(schema)
        .unwrap()
        .as_str()
        .to_string()
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-postgresql@0.0.0#workspace".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "workspace:crates/sifr_sql_postgresql".to_string(),
        package_graph_digest: "b".repeat(64),
        compiler_components: BTreeMap::from([(
            "sifr.sql.postgresql.sql".to_string(),
            "c".repeat(64),
        )]),
    }
}
