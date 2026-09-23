#![allow(clippy::expect_used, clippy::unwrap_used)]

mod support;

use sifr_compiler_component::{
    AnalysisContext, COMPONENT_PROTOCOL_MAJOR, ComponentHost, ComponentHostLimits, ContextArtifact,
    EmbeddedAnalysisRequest, PlanKind, SourceSpan, TemplatePart,
};
use sifr_sql_contract::schema_fingerprint;
use sifr_sql_postgresql::{
    LibpgQueryParser, POSTGRESQL_QUERY_OPERATION, POSTGRESQL_SCHEMA_ARTIFACT_KIND,
    PostgresCompilerComponent, PostgresParser, SUPPORTED_POSTGRESQL_MAJORS, component_registration,
    embedded_sources,
};
use std::collections::BTreeMap;
use support::schema_for_semantics;

#[test]
fn exact_libpg_query_sources_and_registration_cover_every_supported_major() {
    let sources = embedded_sources().expect("checked-in source manifest");
    assert_eq!(
        sources
            .iter()
            .map(|source| source.server_major)
            .collect::<Vec<_>>(),
        SUPPORTED_POSTGRESQL_MAJORS
    );
    for source in sources {
        assert_eq!(source.commit.len(), 40);
        assert_eq!(source.source_content_sha256.len(), 64);
        assert!(source.tag.starts_with(&source.server_major.to_string()));
        let registration = component_registration(source.server_major).unwrap();
        assert!(
            registration
                .identity
                .processor
                .starts_with(POSTGRESQL_QUERY_OPERATION)
        );
        assert!(
            registration
                .artifact
                .ends_with(&format!("postgresql-{}.wasm", source.server_major))
        );
        assert_eq!(registration.diagnostics.declarations.len(), 11);
    }
}

#[test]
fn every_checked_in_component_executes_in_the_capability_free_host() {
    let native = PostgresCompilerComponent::new(LibpgQueryParser);
    let mut schema = schema_for_semantics(&native, LibpgQueryParser.server_major());
    for major in SUPPORTED_POSTGRESQL_MAJORS {
        schema.dialect.server_version = major.to_string();
        let fingerprint = schema_output_fingerprint(&schema);
        let registration = component_registration(major).expect("checked component registration");
        let bytes = std::fs::read(sifr_sql_postgresql::component_artifact_path(major))
            .expect("checked component artifact");
        let span = SourceSpan {
            document: "src/component.sifr".to_string(),
            start: 10,
            end: 55,
        };
        let request = EmbeddedAnalysisRequest {
            protocol_major: COMPONENT_PROTOCOL_MAJOR,
            component: registration.identity.clone(),
            provider_diagnostics: registration.diagnostics.clone(),
            compiler_semantic_version: "0.0.0".to_string(),
            parts: vec![TemplatePart::Static {
                text: "SELECT id FROM users ORDER BY id LIMIT 1".to_string(),
                span,
            }],
            holes: Vec::new(),
            context: AnalysisContext {
                schema_profile: Some("app.Schema".to_string()),
                schema_fingerprint: Some(fingerprint.clone()),
                semantic_profile: BTreeMap::new(),
                imported_signatures: Vec::new(),
                artifacts: vec![ContextArtifact {
                    kind: POSTGRESQL_SCHEMA_ARTIFACT_KIND.to_string(),
                    identity: "app.Schema".to_string(),
                    format_version: 1,
                    fingerprint,
                    payload: serde_json::to_vec(&schema).expect("SchemaIR serialization"),
                }],
            },
            plan_kind: PlanKind::Expression,
        };
        let limits = ComponentHostLimits {
            fuel: 100_000_000,
            ..ComponentHostLimits::default()
        };
        let mut host = ComponentHost::new(limits, None).expect("component host");
        let run = host
            .analyze(&registration, &bytes, &request)
            .unwrap_or_else(|error| panic!("PostgreSQL {major} component failed: {error}"));
        assert_eq!(
            run.response.plan.provider_identity,
            registration.identity.processor
        );
        assert!(run.response.plan.diagnostics.is_empty());
        assert!(!run.response.plan.operations.is_empty());
        exercise_sequence_schema_component(&mut host, major, &bytes);
    }
}

fn exercise_sequence_schema_component(host: &mut ComponentHost, major: u16, bytes: &[u8]) {
    use sifr_sql_contract::{
        ObjectId, SchemaDocumentKind, SchemaSourceInput, SemanticValue, SessionContract,
        schema_normalization_from_response, schema_normalization_request,
        schema_source_fingerprint,
    };
    let mut registration = component_registration(major).unwrap();
    registration.identity.processor = format!("sifr.sql.postgresql.v{major}.schema");
    let sources = [
        ("01.sql", "CREATE TABLE owners (first bigint, second bigint); CREATE SEQUENCE seq AS integer INCREMENT -2 MINVALUE -1000 MAXVALUE -1 START -1 OWNED BY owners.first; CREATE SEQUENCE detached;"),
        ("02.sql", "ALTER SEQUENCE seq OWNED BY owners.second; ALTER SEQUENCE detached OWNED BY owners.first;"),
        ("03.sql", "ALTER SEQUENCE detached OWNED BY NONE;"),
    ].into_iter().map(|(document, sql)| SchemaSourceInput {
        document: document.to_string(), kind: SchemaDocumentKind::SqlDdl,
        fingerprint: schema_source_fingerprint(sql.as_bytes()), contents: sql.as_bytes().to_vec(),
    }).collect::<Vec<_>>();
    let request = schema_normalization_request(
        &registration,
        "0.0.0",
        "app.Schema",
        &major.to_string(),
        &SessionContract::default(),
        &Default::default(),
        &sources,
    )
    .unwrap();
    let run = host
        .analyze(&registration, bytes, &request)
        .unwrap_or_else(|error| panic!("PostgreSQL {major} schema component failed: {error}"));
    assert!(run.response.plan.diagnostics.is_empty());
    let output =
        schema_normalization_from_response(support::provider(), &sources, &run.response).unwrap();
    let sequence = &output.schema.objects[&ObjectId::new("public.seq")];
    assert_eq!(
        sequence.semantic.get("owned-by"),
        Some(&SemanticValue::Text("public.owners.second".to_string()))
    );
    assert!(
        !sequence
            .dependencies
            .contains(&ObjectId::new("public.owners.first"))
    );
    assert_eq!(
        sequence.semantic.get("increment"),
        Some(&SemanticValue::Signed(-2))
    );
    let detached = &output.schema.objects[&ObjectId::new("public.detached")];
    assert!(!detached.semantic.contains_key("owned-by"));
    assert_eq!(
        detached.dependencies,
        [ObjectId::new("public")].into_iter().collect()
    );
}

fn schema_output_fingerprint(schema: &sifr_sql_contract::SchemaIr) -> String {
    schema_fingerprint(schema).unwrap().as_str().to_string()
}
