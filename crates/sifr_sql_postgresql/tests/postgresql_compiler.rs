#![allow(clippy::expect_used, clippy::unwrap_used)]

mod support;

use sifr_sql_contract::{
    Cardinality, DatabaseType, ObjectId, PoolingMode, QueryEffect, SchemaDocument,
    SchemaDocumentKind, SchemaEvidence, SchemaObject, SchemaObjectKind, SchemaProfile,
    SchemaRequirement, SchemaRequirementIdentity, SchemaStrictness, SemanticValue, SessionContract,
    build_profile_authority, build_provider_schema_requirement, normalize_schema,
    provider_analysis_from_response, schema_source_fingerprint,
};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresCompilerComponent, PostgresComponentRequest,
    PostgresComponentResponse, PostgresDiagnosticCode, PostgresParser, StatementKind,
    into_embedded_response, postgresql_capabilities, rewrite_parameter_slots,
};
use std::collections::{BTreeMap, BTreeSet};
use support::{provider, schema_for_semantics, schema_for_writes};

#[test]
fn fragment_parameter_rewrite_preserves_postgresql_lexical_boundaries() {
    let source = "id = $1 AND note = '$2' AND body = $tag$ $3 $tag$ /* $4 */ -- $5\nOR x = $2";
    assert_eq!(
        rewrite_parameter_slots(source, 3).unwrap(),
        "id = $4 AND note = '$2' AND body = $tag$ $3 $tag$ /* $4 */ -- $5\nOR x = $5"
    );
    assert!(rewrite_parameter_slots("x = $0", 1).is_err());
}

#[test]
fn libpg_query_maps_raw_nodes_to_owned_select_write_and_ddl_nodes() {
    let parser = LibpgQueryParser;
    let select = parser
        .parse(
            "SELECT u.id, lower(u.name) AS normalized FROM public.users AS u \
             WHERE u.id = $1 UNION ALL SELECT id, name FROM archived_users",
        )
        .unwrap();
    assert!(matches!(select[0].kind, StatementKind::Select(_)));

    let insert = parser
        .parse(
            "INSERT INTO users(id, name) VALUES ($1, $2) \
             ON CONFLICT(id) DO UPDATE SET name = excluded.name RETURNING id",
        )
        .unwrap();
    assert!(matches!(insert[0].kind, StatementKind::Insert(_)));

    let ddl = parser
        .parse(
            "CREATE TABLE public.users (\
               id bigint PRIMARY KEY,\
               name text NOT NULL,\
               generated text GENERATED ALWAYS AS (name || id::text) STORED\
             )",
        )
        .unwrap();
    let StatementKind::CreateTable(table) = &ddl[0].kind else {
        panic!("expected provider-owned CREATE TABLE node");
    };
    assert_eq!(table.columns.len(), 3);
    assert!(table.columns[2].generated);
}

#[test]
fn ddl_normalization_and_query_analysis_share_one_schema_authority() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let response = component.execute(PostgresComponentRequest::NormalizeSchema {
        provider: provider(),
        server_major,
        documents: vec![(
            "db/schema.sql".to_string(),
            "CREATE TABLE public.teams (id integer PRIMARY KEY, name text NOT NULL);\
             CREATE TABLE public.users (\
               id bigint PRIMARY KEY,\
               name text NOT NULL UNIQUE CHECK (name <> ''),\
               team_id integer REFERENCES public.teams(id),\
               nickname text,\
               generated text GENERATED ALWAYS AS (name || id::text) STORED\
             );\
             CREATE UNIQUE INDEX users_name_idx ON public.users(name);\
             CREATE VIEW public.user_names AS SELECT id, name FROM public.users;\
             CREATE MATERIALIZED VIEW public.team_names AS SELECT id, name FROM public.teams;\
             CREATE SEQUENCE public.audit_sequence;\
             CREATE SEQUENCE public.owned_users_sequence;\
             ALTER SEQUENCE public.owned_users_sequence OWNED BY public.users.id;\
             CREATE TYPE public.mood AS ENUM ('ok', 'sad');\
             CREATE DOMAIN public.label AS text NOT NULL;"
                .to_string(),
        )],
    });
    let PostgresComponentResponse::Schema(schema_output) = response else {
        panic!("DDL must normalize: {response:?}");
    };
    let schema = normalize_schema(provider(), schema_output.dialect, schema_output.documents)
        .expect("normalized SchemaIR");
    assert_eq!(
        schema
            .objects
            .values()
            .filter(|object| object.kind == SchemaObjectKind::Table)
            .count(),
        2
    );
    let kinds = schema
        .objects
        .values()
        .map(|object| object.kind)
        .collect::<BTreeSet<_>>();
    for kind in [
        SchemaObjectKind::CheckConstraint,
        SchemaObjectKind::Column,
        SchemaObjectKind::Domain,
        SchemaObjectKind::Enum,
        SchemaObjectKind::ForeignKey,
        SchemaObjectKind::Index,
        SchemaObjectKind::MaterializedView,
        SchemaObjectKind::Namespace,
        SchemaObjectKind::PrimaryKey,
        SchemaObjectKind::Sequence,
        SchemaObjectKind::Table,
        SchemaObjectKind::UniqueConstraint,
        SchemaObjectKind::View,
    ] {
        assert!(kinds.contains(&kind), "missing normalized object {kind:?}");
    }
    let owned_sequence = schema
        .objects
        .get(&ObjectId::new("public.owned_users_sequence"))
        .expect("owned sequence");
    assert_eq!(
        owned_sequence.semantic.get("owned-by"),
        Some(&SemanticValue::Text("public.users.id".to_string()))
    );
    assert!(
        owned_sequence
            .dependencies
            .contains(&ObjectId::new("public.users.id"))
    );
    let check = schema
        .objects
        .values()
        .find(|object| object.kind == SchemaObjectKind::CheckConstraint)
        .expect("column CHECK constraint");
    assert!(
        check
            .dependencies
            .contains(&sifr_sql_contract::ObjectId::new("public.users.name"))
    );

    let query = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: "SELECT u.id, lower(u.name) AS normalized, u.nickname \
                 FROM public.users AS u WHERE u.id = $1 ORDER BY u.id LIMIT 1"
            .to_string(),
        sifr_document: "src/queries.sifr".to_string(),
        sifr_start: 40,
        sifr_end: 180,
    });
    let PostgresComponentResponse::Query(analysis) = query else {
        panic!("query must analyze");
    };
    assert_eq!(analysis.parameters.len(), 1);
    assert!(matches!(
        analysis.parameters[0].database_type,
        DatabaseType::Integer { .. }
    ));
    assert_eq!(analysis.result_fields.len(), 3);
    assert!(matches!(
        analysis.result_fields[0].sifr_type,
        sifr_sql_contract::SifrType::FixedInteger { .. }
    ));
    assert_eq!(
        analysis.result_fields[2].nullability,
        sifr_sql_contract::Nullability::Nullable
    );
    assert_eq!(analysis.cardinality, Cardinality::AT_MOST_ONE);
    assert_eq!(analysis.effects.effect, QueryEffect::Read);
    assert!(analysis.semantic_flags.contains("deterministic-order"));
    assert_eq!(
        analysis.required_capabilities,
        BTreeSet::from([
            "sql.bind.parameters".to_string(),
            "sql.expression.equality".to_string(),
            "sql.expression.function".to_string(),
            "sql.query.select".to_string(),
        ])
    );

    let embedded = into_embedded_response(
        server_major,
        Some("app.Schema".to_string()),
        &schema,
        &PostgresComponentResponse::Query(analysis.clone()),
    )
    .unwrap();
    assert_eq!(embedded.plan.operations.len(), 1);
    assert_eq!(embedded.plan.schema_identity.as_deref(), Some("app.Schema"));
    assert_eq!(
        provider_analysis_from_response(&embedded).expect("common provider envelope"),
        analysis
    );
}

#[test]
fn postgresql_normalizes_portable_requirement_ddl_with_explicit_capabilities() {
    let source = "CREATE TABLE public.users (id bigint PRIMARY KEY, email text NOT NULL UNIQUE);";
    let response = PostgresCompilerComponent::new(LibpgQueryParser).execute(
        PostgresComponentRequest::NormalizeSchema {
            provider: provider(),
            server_major: LibpgQueryParser.server_major(),
            documents: vec![(
                "db/requirements/has_users.postgresql.sql".to_string(),
                source.to_string(),
            )],
        },
    );
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("requirement DDL must normalize")
    };
    assert_eq!(output.capabilities, postgresql_capabilities());
    let schema = normalize_schema(provider(), output.dialect, output.documents).unwrap();
    let required = BTreeSet::from([
        "sql.bind.parameters".to_string(),
        "sql.query.select".to_string(),
    ]);
    let identity = SchemaRequirementIdentity::new("library", "has_users").unwrap();
    let artifact = build_provider_schema_requirement(
        identity.clone(),
        "db/requirements/has_users.postgresql.sql",
        schema_source_fingerprint(source.as_bytes()),
        &schema,
        required,
        &postgresql_capabilities(),
    )
    .unwrap();
    assert!(
        artifact
            .schema
            .objects
            .contains_key(&sifr_sql_contract::ObjectId::new("public.users.email"))
    );

    let application_source = "CREATE TABLE public.users (\
        id bigint PRIMARY KEY,\
        email text NOT NULL UNIQUE,\
        display_name text,\
        CHECK (display_name <> '')\
    );";
    let application = PostgresCompilerComponent::new(LibpgQueryParser).execute(
        PostgresComponentRequest::NormalizeSchema {
            provider: provider(),
            server_major: LibpgQueryParser.server_major(),
            documents: vec![("db/schema.sql".to_string(), application_source.to_string())],
        },
    );
    let PostgresComponentResponse::Schema(application) = application else {
        panic!("application DDL must normalize")
    };
    let application =
        normalize_schema(provider(), application.dialect, application.documents).unwrap();
    let authority = build_profile_authority(SchemaProfile {
        package_id: "app@1.0.0#registry".to_string(),
        name: "app".to_string(),
        source_files: BTreeSet::from(["db/schema.sql".to_string()]),
        source_fingerprints: BTreeMap::from([(
            "db/schema.sql".to_string(),
            schema_source_fingerprint(application_source.as_bytes()),
        )]),
        evidence: SchemaEvidence::MigrationHead,
        strictness: SchemaStrictness::Compatible,
        pooling: PoolingMode::Session,
        session: SessionContract::default(),
        accepted_signers: BTreeSet::new(),
        capabilities: postgresql_capabilities(),
        schema: application,
    })
    .unwrap();
    SchemaRequirement::new(identity, [artifact])
        .unwrap()
        .prove(&authority)
        .expect("application-only columns and constraints must preserve the structural proof");
}

#[test]
fn aliases_correlations_set_operations_and_parameter_codecs_are_exact() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_semantics(&component, server_major);
    let correlated = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: "SELECT u.id AS user_id, (SELECT t.name FROM teams AS t \
                 WHERE t.id = u.team_id LIMIT 1) AS team_name \
                 FROM users AS u WHERE u.id = $1 ORDER BY user_id LIMIT 1"
            .to_string(),
        sifr_document: "src/semantic.sifr".to_string(),
        sifr_start: 20,
        sifr_end: 170,
    });
    let PostgresComponentResponse::Query(correlated) = correlated else {
        panic!("correlated query must analyze: {correlated:?}");
    };
    assert_eq!(correlated.parameters.len(), 1);
    assert!(matches!(
        correlated.parameters[0].database_type,
        DatabaseType::Integer {
            sign: sifr_sql_contract::IntegerSign::Signed,
            width: sifr_sql_contract::IntegerWidth::Bits64,
        }
    ));
    assert!(
        correlated.parameters[0]
            .codec
            .as_str()
            .starts_with("postgresql.int.")
    );
    assert_eq!(correlated.result_fields[0].name, "user_id");
    assert_eq!(
        correlated.result_fields[1].nullability,
        sifr_sql_contract::Nullability::Nullable
    );
    assert_eq!(correlated.cardinality, Cardinality::AT_MOST_ONE);
    for object in [
        "public.users",
        "public.users.id",
        "public.users.team_id",
        "public.teams",
        "public.teams.id",
        "public.teams.name",
    ] {
        assert!(correlated.accessed_objects.contains(&ObjectId::new(object)));
    }
    assert_eq!(
        correlated.required_capabilities,
        BTreeSet::from([
            "sql.bind.parameters".to_string(),
            "sql.expression.equality".to_string(),
            "sql.query.select".to_string(),
            "sql.query.subquery".to_string(),
        ])
    );

    let set = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema,
        source: "SELECT id AS value FROM users UNION ALL \
                 SELECT id AS value FROM users ORDER BY value LIMIT 1"
            .to_string(),
        sifr_document: "src/semantic.sifr".to_string(),
        sifr_start: 180,
        sifr_end: 290,
    });
    let PostgresComponentResponse::Query(set) = set else {
        panic!("set operation must analyze");
    };
    assert_eq!(set.result_fields[0].name, "value");
    assert_eq!(set.cardinality, Cardinality::AT_MOST_ONE);
    assert!(set.semantic_flags.contains("set-union"));
    assert!(set.semantic_flags.contains("deterministic-order"));
    assert_eq!(
        set.required_capabilities,
        BTreeSet::from([
            "sql.query.select".to_string(),
            "sql.query.set-operation".to_string(),
        ])
    );
}

#[test]
fn portability_capabilities_and_predicate_object_account_are_closed() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_semantics(&component, server_major);
    let cases = [
        (
            "SELECT lower(name) AS value FROM users",
            "sql.expression.function",
        ),
        ("SELECT id::text AS value FROM users", "sql.type.cast"),
        ("SELECT ARRAY[id] AS value FROM users", "sql.type.array"),
        (
            "SELECT users.id FROM users, LATERAL (SELECT users.team_id AS team_id) AS selected",
            "sql.query.lateral",
        ),
        (
            "SELECT count(*) FILTER (WHERE nickname IS NULL) AS value FROM users",
            "sql.query.aggregate-filter",
        ),
    ];
    for (source, capability) in cases {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/portable-capabilities.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        let PostgresComponentResponse::Query(analysis) = response else {
            panic!("portable capability query must analyze: {response:?}");
        };
        assert!(
            analysis.required_capabilities.contains(capability),
            "{source}"
        );
        let mut incomplete = postgresql_capabilities();
        incomplete.remove(capability);
        assert!(
            !analysis.required_capabilities.is_subset(&incomplete),
            "portable specialization must reject a profile missing {capability}"
        );
    }

    let source = "SELECT users.name FROM users JOIN teams ON teams.id = users.team_id \
                  WHERE users.nickname IS NULL AND teams.name = $1";
    let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema,
        source: source.to_string(),
        sifr_document: "src/object-account.sifr".to_string(),
        sifr_start: 0,
        sifr_end: u32::try_from(source.len()).unwrap(),
    });
    let PostgresComponentResponse::Query(analysis) = response else {
        panic!("object-account query must analyze: {response:?}");
    };
    for object in [
        "public.users",
        "public.users.name",
        "public.users.team_id",
        "public.users.nickname",
        "public.teams",
        "public.teams.id",
        "public.teams.name",
    ] {
        assert!(
            analysis.accessed_objects.contains(&ObjectId::new(object)),
            "{object}"
        );
    }
}

#[test]
fn semantic_failures_keep_virtual_sifr_and_schema_spans() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_semantics(&component, server_major);
    let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: "SELECT u.missing FROM users AS u".to_string(),
        sifr_document: "src/errors.sifr".to_string(),
        sifr_start: 30,
        sifr_end: 66,
    });
    let PostgresComponentResponse::Diagnostic(diagnostic) = response else {
        panic!("unknown column must fail");
    };
    assert_eq!(
        diagnostic.primary.kind,
        sifr_sql_postgresql::PostgresSpanKind::VirtualSql
    );
    assert!(
        diagnostic
            .related
            .iter()
            .any(|span| span.kind == sifr_sql_postgresql::PostgresSpanKind::Sifr)
    );
    assert!(
        diagnostic
            .related
            .iter()
            .any(|span| span.kind == sifr_sql_postgresql::PostgresSpanKind::Schema)
    );

    for source in [
        "SELECT id AS duplicate, name AS duplicate FROM users",
        "SELECT id FROM users WHERE id = $1 AND name = $1",
        "SELECT id FROM users WHERE id = $2",
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/errors.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        assert!(matches!(response, PostgresComponentResponse::Diagnostic(_)));
    }
}

#[test]
fn update_and_delete_returning_preserve_write_effects() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_writes(&component, server_major);
    for source in [
        "UPDATE users SET name = $1 WHERE id = $2 RETURNING id, nickname",
        "DELETE FROM users WHERE id = $1 RETURNING id",
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/write.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        let PostgresComponentResponse::Query(analysis) = response else {
            panic!("write must analyze: {response:?}");
        };
        assert_eq!(analysis.effects.effect, QueryEffect::Write);
        assert_eq!(analysis.effects.affected_objects.len(), 1);
        assert!(!analysis.result_fields.is_empty());
        assert!(
            analysis
                .required_capabilities
                .contains("sql.write.returning")
        );
        if source.starts_with("UPDATE") {
            assert!(
                analysis
                    .accessed_objects
                    .contains(&ObjectId::new("public.users.name"))
            );
        }
    }
}

#[test]
fn writes_enforce_required_generated_conflict_and_returning_contracts() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_writes(&component, server_major);

    let valid = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: "INSERT INTO users(id, name) VALUES ($1, $2) \
                 ON CONFLICT(id) DO UPDATE SET name = excluded.name RETURNING id"
            .to_string(),
        sifr_document: "src/write.sifr".to_string(),
        sifr_start: 10,
        sifr_end: 120,
    });
    let PostgresComponentResponse::Query(valid) = valid else {
        panic!("valid write must analyze: {valid:?}");
    };
    assert_eq!(valid.parameters.len(), 2);
    assert_eq!(valid.effects.effect, QueryEffect::Write);
    assert_eq!(valid.result_fields.len(), 1);
    assert_eq!(
        valid.accessed_objects,
        BTreeSet::from([
            ObjectId::new("public.users"),
            ObjectId::new("public.users.id"),
            ObjectId::new("public.users.name"),
        ])
    );
    assert_eq!(
        valid.required_capabilities,
        BTreeSet::from([
            "sql.bind.parameters".to_string(),
            "sql.query.insert".to_string(),
            "sql.write.conflict".to_string(),
            "sql.write.returning".to_string(),
        ])
    );

    for source in [
        "INSERT INTO users(id) VALUES ($1)",
        "INSERT INTO users(id, name, generated) VALUES ($1, $2, $3)",
        "INSERT INTO users(id, name) VALUES ($1, $2) ON CONFLICT(name) DO NOTHING",
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/write.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        let PostgresComponentResponse::Diagnostic(diagnostic) = response else {
            panic!("invalid write must produce a diagnostic");
        };
        assert_eq!(diagnostic.code, PostgresDiagnosticCode::InvalidWrite);
        assert!(
            diagnostic
                .related
                .iter()
                .any(|span| { span.kind == sifr_sql_postgresql::PostgresSpanKind::Sifr })
        );
    }
}

#[test]
fn metadata_catalog_resolves_functions_operators_casts_and_schema_spans() {
    let server_major = LibpgQueryParser.server_major();
    let base = DatabaseType::Text {
        fixed: false,
        max_characters: None,
    };
    let document = SchemaDocument {
        kind: SchemaDocumentKind::ProviderMetadata,
        document: "catalog.json".to_string(),
        objects: vec![SchemaObject {
            identity: sifr_sql_contract::ObjectId::new("public.custom_lower"),
            kind: SchemaObjectKind::Function,
            semantic: BTreeMap::from([
                (
                    "arguments".to_string(),
                    SemanticValue::List(vec![SemanticValue::Text(
                        serde_json::to_string(&base).unwrap(),
                    )]),
                ),
                (
                    "result".to_string(),
                    SemanticValue::Text(serde_json::to_string(&base).unwrap()),
                ),
                ("strict".to_string(), SemanticValue::Bool(true)),
                ("aggregate".to_string(), SemanticValue::Bool(false)),
                ("result-nullable".to_string(), SemanticValue::Bool(false)),
            ]),
            dependencies: BTreeSet::new(),
            source: Some(sifr_sql_contract::SchemaSourceLocation {
                document: "catalog.json".to_string(),
                start: 12,
                end: 45,
            }),
        }],
    };
    let schema = normalize_schema(
        provider(),
        sifr_sql_contract::DialectIdentity {
            family: "postgresql".to_string(),
            server_version: server_major.to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        [document],
    )
    .unwrap();
    let catalog = sifr_sql_postgresql::PostgresCatalog::from_schema(
        &schema,
        sifr_sql_postgresql::PostgresTypeRegistry::new(server_major),
    )
    .unwrap();
    assert_eq!(catalog.functions(&["custom_lower".to_string()]).len(), 1);
}

#[test]
fn advanced_postgresql_semantics_are_owned_and_exact() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_semantics(&component, server_major);
    let source = "WITH named AS (SELECT id, team_id, nickname FROM users) \
                  SELECT named.id, \
                    CASE WHEN teams.id IS NULL THEN 'none' ELSE teams.name END AS team_name, \
                    COALESCE(named.nickname, 'anonymous') AS display_name, \
                    count(*) OVER peers_window AS peers, \
                    row_number() OVER peers_window AS team_position \
                  FROM named LEFT JOIN teams ON teams.id = named.team_id \
                  WHERE named.id = $1 \
                  WINDOW peers_window AS (PARTITION BY named.team_id ORDER BY named.id)";
    let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema,
        source: source.to_string(),
        sifr_document: "src/advanced.sifr".to_string(),
        sifr_start: 0,
        sifr_end: u32::try_from(source.len()).unwrap(),
    });
    let PostgresComponentResponse::Query(analysis) = response else {
        panic!("advanced query must analyze: {response:?}");
    };
    assert_eq!(analysis.result_fields.len(), 5);
    assert_eq!(analysis.cardinality, Cardinality::MANY);
    assert!(analysis.semantic_flags.contains("common-table-expression"));
    assert!(analysis.semantic_flags.contains("window-function"));
    assert_eq!(
        analysis.required_capabilities,
        BTreeSet::from([
            "sql.bind.parameters".to_string(),
            "sql.expression.case".to_string(),
            "sql.expression.equality".to_string(),
            "sql.expression.function".to_string(),
            "sql.query.aggregate".to_string(),
            "sql.query.common-table-expression".to_string(),
            "sql.query.join".to_string(),
            "sql.query.select".to_string(),
            "sql.query.window".to_string(),
        ])
    );
    assert_eq!(
        analysis.result_fields[1].nullability,
        sifr_sql_contract::Nullability::NonNull
    );
    assert_eq!(
        analysis.result_fields[2].nullability,
        sifr_sql_contract::Nullability::NonNull
    );

    let source = "WITH selected AS (SELECT id FROM users WHERE id = $1) \
                  SELECT id FROM selected UNION ALL SELECT id FROM selected";
    let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema_for_semantics(&component, server_major),
        source: source.to_string(),
        sifr_document: "src/advanced-set.sifr".to_string(),
        sifr_start: 0,
        sifr_end: u32::try_from(source.len()).unwrap(),
    });
    let PostgresComponentResponse::Query(analysis) = response else {
        panic!("a CTE must remain in scope across a set operation: {response:?}");
    };
    assert!(analysis.semantic_flags.contains("common-table-expression"));
    assert!(analysis.semantic_flags.contains("set-union"));

    let source = "WITH first AS (SELECT id FROM users), \
                  second AS (SELECT id FROM first) SELECT id FROM second";
    assert!(matches!(
        component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema_for_semantics(&component, server_major),
            source: source.to_string(),
            sifr_document: "src/advanced-cte-scope.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        }),
        PostgresComponentResponse::Query(_)
    ));

    for source in [
        "WITH repeated AS (SELECT id FROM users), repeated AS (SELECT id FROM users) SELECT id FROM repeated",
        "SELECT row_number() OVER repeated FROM users WINDOW repeated AS (), repeated AS ()",
        "SELECT row_number() OVER first_window FROM users WINDOW first_window AS (second_window), second_window AS (first_window)",
    ] {
        assert!(
            matches!(
                component.execute(PostgresComponentRequest::AnalyzeQuery {
                    schema: schema_for_semantics(&component, server_major),
                    source: source.to_string(),
                    sifr_document: "src/advanced-names.sifr".to_string(),
                    sifr_start: 0,
                    sifr_end: u32::try_from(source.len()).unwrap(),
                }),
                PostgresComponentResponse::Diagnostic(_)
            ),
            "duplicate or cyclic names must be rejected: {source}"
        );
    }

    let source = "SELECT users.id FROM users LEFT JOIN teams ON teams.id = users.team_id FOR UPDATE OF users NOWAIT";
    let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema_for_semantics(&component, server_major),
        source: source.to_string(),
        sifr_document: "src/advanced-lock.sifr".to_string(),
        sifr_start: 0,
        sifr_end: u32::try_from(source.len()).unwrap(),
    });
    let PostgresComponentResponse::Query(analysis) = response else {
        panic!("locking a non-nullable outer-join side must analyze: {response:?}");
    };
    assert!(analysis.semantic_flags.contains("row-locking"));

    let source =
        "SELECT users.id FROM users LEFT JOIN teams ON teams.id = users.team_id FOR UPDATE";
    assert!(matches!(
        component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema_for_semantics(&component, server_major),
            source: source.to_string(),
            sifr_document: "src/advanced-lock.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        }),
        PostgresComponentResponse::Diagnostic(_)
    ));
}

#[test]
fn cardinality_star_and_default_policies_close_regressions() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_writes(&component, server_major);
    for (source, expected) in [
        (
            "SELECT * FROM users WHERE id = $1",
            Cardinality::AT_MOST_ONE,
        ),
        (
            "SELECT (SELECT count(*) FROM users) AS total FROM users",
            Cardinality::MANY,
        ),
        (
            "SELECT id FROM users LIMIT 5",
            Cardinality::new(0, Some(5)).unwrap(),
        ),
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/cardinality.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        let PostgresComponentResponse::Query(analysis) = response else {
            panic!("cardinality query must analyze: {response:?}");
        };
        assert_eq!(analysis.cardinality, expected);
        if source.contains('*') && source.starts_with("SELECT *") {
            assert!(analysis.semantic_flags.contains("expanded-select-star"));
            assert_eq!(analysis.result_fields[0].name, "id");
            assert!(!analysis.normalized_statement.contains('*'));
        }
    }
    for source in [
        "SELECT * FROM users UNION ALL SELECT * FROM users",
        "WITH selected AS (SELECT * FROM users) SELECT id FROM selected",
        "SELECT nested.id FROM (SELECT * FROM users) AS nested",
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/nested-star.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        let PostgresComponentResponse::Query(analysis) = response else {
            panic!("nested private star must analyze: {response:?}");
        };
        assert!(analysis.semantic_flags.contains("expanded-select-star"));
        assert!(!analysis.normalized_statement.contains('*'));
    }

    let source = "VALUES ('1'::bigint), ('1'::bigint) UNION (SELECT id FROM users LIMIT 0)";
    let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema: schema.clone(),
        source: source.to_string(),
        sifr_document: "src/distinct-cardinality.sifr".to_string(),
        sifr_start: 0,
        sifr_end: u32::try_from(source.len()).unwrap(),
    });
    let PostgresComponentResponse::Query(analysis) = response else {
        panic!("distinct set cardinality must analyze: {response:?}");
    };
    assert_eq!(analysis.cardinality, Cardinality::new(1, Some(2)).unwrap());

    for source in [
        "INSERT INTO users(id, name) VALUES (1, DEFAULT)",
        "UPDATE users SET name = DEFAULT WHERE id = 1",
    ] {
        assert!(
            matches!(
                component.execute(PostgresComponentRequest::AnalyzeQuery {
                    schema: schema.clone(),
                    source: source.to_string(),
                    sifr_document: "src/default.sifr".to_string(),
                    sifr_start: 0,
                    sifr_end: u32::try_from(source.len()).unwrap(),
                }),
                PostgresComponentResponse::Diagnostic(_)
            ),
            "{source}"
        );
    }
}
