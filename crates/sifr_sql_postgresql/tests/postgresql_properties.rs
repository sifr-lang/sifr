#![allow(clippy::expect_used, clippy::unwrap_used)]

#[allow(dead_code)]
mod support;

use sifr_sql_contract::{Cardinality, DatabaseType, SchemaObjectKind, normalize_schema};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresCompilerComponent, PostgresComponentRequest,
    PostgresComponentResponse, PostgresParser,
};
use std::collections::BTreeSet;
use support::{provider, schema_for_semantics};

#[test]
fn cardinality_and_nullability_properties_hold_across_query_families() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let schema = schema_for_semantics(&component, server_major);
    for limit in 0..=8 {
        for offset in 0..=3 {
            let source = format!("SELECT id FROM users LIMIT {limit} OFFSET {offset}");
            let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
                schema: schema.clone(),
                source: source.clone(),
                sifr_document: "src/cardinality-property.sifr".to_string(),
                sifr_start: 0,
                sifr_end: u32::try_from(source.len()).unwrap(),
            });
            let PostgresComponentResponse::Query(analysis) = response else {
                panic!("bounded query must analyze: {response:?}");
            };
            assert_eq!(
                analysis.cardinality,
                Cardinality::new(0, Some(limit)).unwrap()
            );
        }
    }

    for source in [
        "SELECT 1 AS value WHERE false",
        "SELECT count(*) AS value FROM users HAVING false",
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/cardinality-property.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        let PostgresComponentResponse::Query(analysis) = response else {
            panic!("filtered singleton query must analyze: {response:?}");
        };
        assert_eq!(analysis.cardinality, Cardinality::AT_MOST_ONE);
    }
    let source = "SELECT id, name, count(*) AS total FROM users GROUP BY id";
    assert!(matches!(
        component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/grouping-property.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        }),
        PostgresComponentResponse::Query(_)
    ));
    let source = "SELECT team_id, name, count(*) AS total FROM users GROUP BY team_id";
    assert!(matches!(
        component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/grouping-property.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        }),
        PostgresComponentResponse::Diagnostic(_)
    ));

    for (source, expected) in [
        (
            "SELECT teams.name AS value FROM users LEFT JOIN teams ON teams.id = users.team_id",
            sifr_sql_contract::Nullability::Nullable,
        ),
        (
            "SELECT CASE WHEN teams.id IS NULL THEN 'none' ELSE teams.name END AS value FROM users LEFT JOIN teams ON teams.id = users.team_id",
            sifr_sql_contract::Nullability::NonNull,
        ),
        (
            "SELECT COALESCE(users.nickname, 'none') AS value FROM users",
            sifr_sql_contract::Nullability::NonNull,
        ),
        (
            "SELECT (SELECT name FROM teams WHERE id = users.team_id) AS value FROM users",
            sifr_sql_contract::Nullability::Nullable,
        ),
    ] {
        let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source: source.to_string(),
            sifr_document: "src/nullability-property.sifr".to_string(),
            sifr_start: 0,
            sifr_end: u32::try_from(source.len()).unwrap(),
        });
        let PostgresComponentResponse::Query(analysis) = response else {
            panic!("nullability query must analyze: {response:?}");
        };
        assert_eq!(analysis.result_fields[0].nullability, expected, "{source}");
    }

    for source in [
        "SELECT row_number() AS value FROM users",
        "SELECT id FROM users WHERE row_number() OVER () = 1",
        "SELECT lower(name) OVER () AS value FROM users",
        "SELECT row_number() OVER missing AS value FROM users",
    ] {
        assert!(
            matches!(
                component.execute(PostgresComponentRequest::AnalyzeQuery {
                    schema: schema.clone(),
                    source: source.to_string(),
                    sifr_document: "src/window-property.sifr".to_string(),
                    sifr_start: 0,
                    sifr_end: u32::try_from(source.len()).unwrap(),
                }),
                PostgresComponentResponse::Diagnostic(_)
            ),
            "window query unexpectedly passed: {source}"
        );
    }
}

#[test]
fn arrays_ranges_composites_and_json_are_schema_checked() {
    let component = PostgresCompilerComponent::new(LibpgQueryParser);
    let server_major = LibpgQueryParser.server_major();
    let normalized = component.execute(PostgresComponentRequest::NormalizeSchema {
        provider: provider(),
        server_major,
        documents: vec![(
            "db/advanced.sql".to_string(),
            "CREATE TYPE public.address AS (street text, zip integer); \
             CREATE TYPE public.score_range AS RANGE (subtype = integer); \
             CREATE TABLE public.items (id bigint PRIMARY KEY, tags text[], payload jsonb, score score_range, address address);"
                .to_string(),
        )],
    });
    let PostgresComponentResponse::Schema(output) = normalized else {
        panic!("advanced DDL must normalize: {normalized:?}");
    };
    let schema = normalize_schema(provider(), output.dialect, output.documents).unwrap();
    let kinds = schema
        .objects
        .values()
        .map(|object| object.kind)
        .collect::<BTreeSet<_>>();
    assert!(kinds.contains(&SchemaObjectKind::Composite));
    assert!(kinds.contains(&SchemaObjectKind::Range));
    let source = "SELECT payload ->> 'name' AS name, tags || ARRAY['checked'] AS tags FROM items WHERE id = $1";
    let response = component.execute(PostgresComponentRequest::AnalyzeQuery {
        schema,
        source: source.to_string(),
        sifr_document: "src/types.sifr".to_string(),
        sifr_start: 0,
        sifr_end: u32::try_from(source.len()).unwrap(),
    });
    let PostgresComponentResponse::Query(analysis) = response else {
        panic!("advanced types must analyze: {response:?}");
    };
    assert_eq!(analysis.cardinality, Cardinality::AT_MOST_ONE);
    assert_eq!(analysis.result_fields.len(), 2);
    assert!(matches!(
        analysis.result_fields[1].database_type,
        DatabaseType::Array {
            dimensions: None,
            ..
        }
    ));
}
