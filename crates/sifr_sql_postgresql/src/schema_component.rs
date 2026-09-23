use crate::component::{component_diagnostic, into_non_query_embedded_response};
use crate::{
    LibpgQueryParser, PostgresCompilerComponent, PostgresComponentRequest,
    PostgresComponentResponse, PostgresDiagnostic, PostgresParser,
};
use sifr_compiler_component::{EmbeddedAnalysisRequest, EmbeddedAnalysisResponse, PlanKind};
use sifr_sql_contract::{ProviderIdentity, SchemaSourceArtifact, schema_source_fingerprint};
use std::collections::BTreeMap;

pub(crate) fn execute_schema_normalization(
    request: EmbeddedAnalysisRequest,
) -> Result<EmbeddedAnalysisResponse, PostgresDiagnostic> {
    let major = LibpgQueryParser.server_major();
    let processor = &request.component.processor;
    if processor != "sifr.sql.postgresql.schema"
        && processor != &format!("sifr.sql.postgresql.v{major}.schema")
    {
        return Err(component_diagnostic(
            "PostgreSQL schema normalization requires the matching schema processor",
        ));
    }
    let version = request
        .context
        .semantic_profile
        .get("server-version")
        .and_then(|version| version.split('.').next())
        .and_then(|version| version.parse::<u16>().ok());
    if version != Some(major) {
        return Err(component_diagnostic(
            "PostgreSQL schema and component parser majors differ",
        ));
    }
    if request.plan_kind != PlanKind::Document || request.context.artifacts.is_empty() {
        return Err(component_diagnostic(
            "PostgreSQL schema normalization requires a document request with sources",
        ));
    }
    let mut documents = Vec::new();
    for artifact in &request.context.artifacts {
        if artifact.kind != "sifr.sql.schema-source.sql-ddl" || artifact.format_version != 1 {
            return Err(component_diagnostic(
                "PostgreSQL schema components accept version 1 SQL DDL sources only",
            ));
        }
        let source: SchemaSourceArtifact = serde_json::from_slice(&artifact.payload)
            .map_err(|_| component_diagnostic("PostgreSQL schema source artifact is invalid"))?;
        if artifact.fingerprint != schema_source_fingerprint(&source.contents) {
            return Err(component_diagnostic(
                "PostgreSQL schema source fingerprint is invalid",
            ));
        }
        let contents = String::from_utf8(source.contents)
            .map_err(|_| component_diagnostic("PostgreSQL schema source must be UTF-8"))?;
        documents.push((artifact.identity.clone(), contents));
    }
    let provider = ProviderIdentity {
        package_id: request.component.package.clone(),
        package_version: request.component.version.clone(),
        package_source: "compiler-component".to_string(),
        package_graph_digest: request.component.sha256.clone(),
        compiler_components: BTreeMap::from([(
            processor.clone(),
            request.component.sha256.clone(),
        )]),
    };
    let response = PostgresCompilerComponent::new(LibpgQueryParser).execute(
        PostgresComponentRequest::NormalizeSchema {
            provider,
            server_major: major,
            documents,
        },
    );
    if let PostgresComponentResponse::Diagnostic(diagnostic) = response {
        return Err(diagnostic);
    }
    let mut response =
        into_non_query_embedded_response(major, request.context.schema_profile, &response)?;
    response.plan.provider_identity.clone_from(processor);
    response.plan.stable_fingerprint =
        sifr_compiler_component::compute_plan_fingerprint(&response.plan)
            .map_err(|error| component_diagnostic(error.to_string()))?;
    Ok(response)
}
