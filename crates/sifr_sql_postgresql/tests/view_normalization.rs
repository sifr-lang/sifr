#![allow(clippy::expect_used, clippy::panic)]

use semver::Version;
use sifr_sql_contract::{ProviderIdentity, normalize_schema};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresCatalog, PostgresCompilerComponent, PostgresComponentRequest,
    PostgresComponentResponse, PostgresDiagnosticCode, PostgresParser, PostgresTypeRegistry,
    StatementKind, canonical_postgres_view_json,
};
use std::collections::BTreeMap;

fn catalog() -> PostgresCatalog {
    let provider = ProviderIdentity {
        package_id: "sifr-sql-postgresql@0.0.0#workspace".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "workspace:crates/sifr_sql_postgresql".to_string(),
        package_graph_digest: "b".repeat(64),
        compiler_components: BTreeMap::from([(
            "sifr.sql.postgresql.sql".to_string(),
            "c".repeat(64),
        )]),
    };
    let response = PostgresCompilerComponent::new(LibpgQueryParser).execute(
        PostgresComponentRequest::NormalizeSchema {
            provider: provider.clone(),
            server_major: LibpgQueryParser.server_major(),
            documents: vec![(
                "views.sql".to_string(),
                "CREATE TABLE users (id bigint PRIMARY KEY, name text); \
                 CREATE TABLE other.users (id bigint PRIMARY KEY, name text); \
                 CREATE TABLE projects (id bigint PRIMARY KEY);"
                    .to_string(),
            )],
        },
    );
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("real DDL producer: {response:?}");
    };
    let schema = normalize_schema(provider, output.dialect, output.documents).expect("schema");
    PostgresCatalog::from_schema(
        &schema,
        PostgresTypeRegistry::new(LibpgQueryParser.server_major()),
    )
    .expect("catalog")
}

fn canonical(
    sql: &str,
    catalog: &PostgresCatalog,
) -> Result<String, sifr_sql_postgresql::PostgresAnalysisError> {
    let statements = LibpgQueryParser.parse(sql).expect("parse SELECT");
    let StatementKind::Select(query) = &statements[0].kind else {
        panic!("SELECT");
    };
    canonical_postgres_view_json(query, catalog)
}

#[test]
fn view_equivalence_uses_resolved_columns_and_ordered_star_expansion() {
    let catalog = catalog();
    for (ddl, live) in [
        (
            "SELECT id, name FROM users",
            "SELECT users.id, users.name FROM public.users",
        ),
        (
            "SELECT public.users.id FROM public.users",
            "SELECT id FROM users",
        ),
        ("SELECT u.id FROM users u", "SELECT id FROM public.users u"),
        (
            "SELECT u.id FROM users u",
            "SELECT renamed.id FROM public.users renamed",
        ),
        ("SELECT users.id FROM users users", "SELECT id FROM users"),
        (
            "SELECT * FROM users",
            "SELECT users.id, users.name FROM users",
        ),
        ("SELECT users.* FROM users", "SELECT id, name FROM users"),
        ("SELECT id AS id FROM users", "SELECT users.id FROM users"),
        (
            "SELECT id FROM users ORDER BY id ASC",
            "SELECT users.id FROM users ORDER BY id",
        ),
        (
            "SELECT id FROM users ORDER BY id",
            "SELECT id FROM users ORDER BY users.id NULLS LAST",
        ),
        (
            "WITH u AS (SELECT id FROM users) SELECT id FROM u",
            "WITH u AS (SELECT users.id FROM users) SELECT u.id FROM u",
        ),
        (
            "SELECT u.id FROM (SELECT id FROM users) u",
            "SELECT id FROM (SELECT users.id FROM users) u",
        ),
    ] {
        assert_eq!(
            canonical(ddl, &catalog).expect(ddl),
            canonical(live, &catalog).expect(live),
            "{ddl} / {live}"
        );
    }
}

#[test]
fn view_identity_preserves_schema_binding_output_and_ordering_distinctions() {
    let catalog = catalog();
    for (left, right) in [
        ("SELECT id FROM public.users", "SELECT id FROM other.users"),
        ("SELECT id FROM users", "SELECT name FROM users"),
        ("SELECT id, name FROM users", "SELECT name, id FROM users"),
        (
            "SELECT id FROM users ORDER BY id ASC",
            "SELECT id FROM users ORDER BY id DESC",
        ),
        (
            "SELECT id FROM users ORDER BY id NULLS FIRST",
            "SELECT id FROM users ORDER BY id NULLS LAST",
        ),
        ("SELECT id FROM users", "SELECT id AS user_id FROM users"),
        (
            "SELECT a.id FROM users a JOIN users b ON a.id = b.id",
            "SELECT b.id FROM users a JOIN users b ON a.id = b.id",
        ),
        (
            "SELECT id FROM users WHERE id > 1",
            "SELECT id FROM users WHERE id > 2",
        ),
    ] {
        assert_ne!(
            canonical(left, &catalog).expect(left),
            canonical(right, &catalog).expect(right),
            "{left} / {right}"
        );
    }
    assert_eq!(
        canonical(
            "SELECT id FROM users a JOIN projects b ON a.id = b.id",
            &catalog
        )
        .expect_err("ambiguous")
        .diagnostic
        .code,
        PostgresDiagnosticCode::AmbiguousColumn
    );
    assert_eq!(
        canonical("SELECT other.users.id FROM public.users", &catalog)
            .expect_err("wrong schema")
            .diagnostic
            .code,
        PostgresDiagnosticCode::UnknownColumn
    );
}

#[test]
fn recursive_view_capture_ignores_the_type_only_anchor_prepass() {
    let catalog = catalog();
    let ddl = "WITH RECURSIVE r(id) AS (SELECT id FROM users UNION ALL SELECT id + 1 AS id FROM r WHERE id < 3) SELECT id FROM r";
    let live = "WITH RECURSIVE r(id) AS (SELECT users.id FROM public.users UNION ALL SELECT r_1.id + 1 AS id FROM r r_1 WHERE r_1.id < 3) SELECT r.id FROM r";
    assert_eq!(
        canonical(ddl, &catalog).expect("recursive DDL"),
        canonical(live, &catalog).expect("recursive live")
    );
    let different = ddl.replace("id < 3", "id < 4");
    assert_ne!(
        canonical(ddl, &catalog).expect("recursive DDL"),
        canonical(&different, &catalog).expect("different recursive bound")
    );
}

#[test]
fn set_view_ordering_retains_positional_output_identity() {
    let catalog = catalog();
    let ddl = "SELECT id, name FROM users UNION SELECT id, name FROM other.users ORDER BY id";
    let live = "SELECT users.id, users.name FROM public.users UNION SELECT users.id, users.name FROM other.users ORDER BY 1";
    assert_eq!(
        canonical(ddl, &catalog).expect("set DDL"),
        canonical(live, &catalog).expect("set live ordinal")
    );
    for different in [
        ddl.replace("ORDER BY id", "ORDER BY name"),
        ddl.replace("ORDER BY id", "ORDER BY id DESC"),
        ddl.replace(" UNION ", " UNION ALL "),
    ] {
        assert_ne!(
            canonical(ddl, &catalog).expect("set DDL"),
            canonical(&different, &catalog).expect("different set view")
        );
    }
    let values = "VALUES ('1'::bigint), ('2'::bigint) UNION SELECT id FROM users ORDER BY 1";
    assert_eq!(
        canonical(values, &catalog).expect("VALUES set"),
        canonical(
            &values.replace("ORDER BY 1", "ORDER BY 1 ASC NULLS LAST"),
            &catalog
        )
        .expect("VALUES ordinal")
    );
}
