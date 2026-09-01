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
    }
}

fn schema_output_fingerprint(schema: &sifr_sql_contract::SchemaIr) -> String {
    schema_fingerprint(schema).unwrap().as_str().to_string()
}
