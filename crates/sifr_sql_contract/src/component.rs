use crate::{
    DialectIdentity, ProviderAnalysis, ProviderIdentity, SchemaContractError,
    SchemaContractErrorKind, SchemaDocument, SchemaDocumentKind, SchemaIr, normalize_schema,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_compiler_component::{
    AnalysisContext, COMPONENT_PROTOCOL_MAJOR, ClosedType, ComponentRegistration, ContextArtifact,
    EmbeddedAnalysisRequest, EmbeddedAnalysisResponse, PlanKind, RuntimeLowering,
    SemanticOperation,
};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_NORMALIZATION_OPERATION: &str = "sifr.sql.normalize-schema";
pub const SCHEMA_NORMALIZATION_PAYLOAD_TAG: &str = "sifr.sql.normalized-schema";
pub const PROVIDER_ANALYSIS_PAYLOAD_TAG: &str = "sifr.sql.provider-analysis";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaSourceInput {
    pub document: String,
    pub kind: SchemaDocumentKind,
    pub fingerprint: String,
    pub contents: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaSourceArtifact {
    pub contents: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaNormalizationOutput {
    pub dialect: DialectIdentity,
    pub capabilities: BTreeSet<String>,
    pub documents: Vec<SchemaDocument>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaNormalizationResult {
    pub schema: SchemaIr,
    pub capabilities: BTreeSet<String>,
}

pub fn schema_source_fingerprint(contents: &[u8]) -> String {
    lower_hex(&Sha256::digest(contents))
}

pub fn schema_normalization_request(
    registration: &ComponentRegistration,
    compiler_semantic_version: &str,
    profile_identity: &str,
    server_version: &str,
    sql_modes: &BTreeSet<String>,
    extensions: &BTreeSet<String>,
    sources: &[SchemaSourceInput],
) -> Result<EmbeddedAnalysisRequest, SchemaContractError> {
    if sources.is_empty() {
        return Err(invalid("schema normalization needs at least one source"));
    }
    let mut artifacts = sources
        .iter()
        .map(|source| {
            if source.document.is_empty()
                || source.fingerprint != schema_source_fingerprint(&source.contents)
            {
                return Err(invalid(format!(
                    "schema source '{}' has invalid identity, contents, or fingerprint",
                    source.document
                )));
            }
            Ok(ContextArtifact {
                kind: source_artifact_kind(source.kind).to_string(),
                identity: source.document.clone(),
                format_version: 1,
                fingerprint: source.fingerprint.clone(),
                payload: serde_json::to_vec(&SchemaSourceArtifact {
                    contents: source.contents.clone(),
                })
                .map_err(|error| serialization_error(&error))?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    artifacts
        .sort_by(|left, right| (&left.kind, &left.identity).cmp(&(&right.kind, &right.identity)));
    if artifacts
        .iter()
        .map(|artifact| artifact.identity.as_str())
        .collect::<BTreeSet<_>>()
        .len()
        != artifacts.len()
    {
        return Err(invalid("schema source identities must be unique"));
    }
    let semantic_profile = BTreeMap::from([
        (
            "operation".to_string(),
            SCHEMA_NORMALIZATION_OPERATION.to_string(),
        ),
        ("server-version".to_string(), server_version.to_string()),
        (
            "sql-modes".to_string(),
            serde_json::to_string(sql_modes).map_err(|error| serialization_error(&error))?,
        ),
        (
            "extensions".to_string(),
            serde_json::to_string(extensions).map_err(|error| serialization_error(&error))?,
        ),
    ]);
    Ok(EmbeddedAnalysisRequest {
        protocol_major: COMPONENT_PROTOCOL_MAJOR,
        component: registration.identity.clone(),
        provider_diagnostics: registration.diagnostics.clone(),
        compiler_semantic_version: compiler_semantic_version.to_string(),
        parts: Vec::new(),
        holes: Vec::new(),
        context: AnalysisContext {
            schema_profile: Some(profile_identity.to_string()),
            schema_fingerprint: None,
            semantic_profile,
            imported_signatures: Vec::new(),
            artifacts,
        },
        plan_kind: PlanKind::Document,
    })
}

pub fn normalized_schema_from_response(
    provider: ProviderIdentity,
    sources: &[SchemaSourceInput],
    response: &EmbeddedAnalysisResponse,
) -> Result<SchemaIr, SchemaContractError> {
    Ok(schema_normalization_from_response(provider, sources, response)?.schema)
}

pub fn schema_normalization_from_response(
    provider: ProviderIdentity,
    sources: &[SchemaSourceInput],
    response: &EmbeddedAnalysisResponse,
) -> Result<SchemaNormalizationResult, SchemaContractError> {
    if response.plan.result_type != ClosedType::None
        || response.plan.runtime != RuntimeLowering::NoRuntime
        || response.plan.operations.len() != 1
    {
        return Err(invalid(
            "schema normalizer must return one no-runtime provider payload",
        ));
    }
    let SemanticOperation::ProviderNode { tag, payload } = &response.plan.operations[0] else {
        return Err(invalid(
            "schema normalizer returned a non-provider operation",
        ));
    };
    if tag != SCHEMA_NORMALIZATION_PAYLOAD_TAG {
        return Err(invalid("schema normalizer returned an unknown payload tag"));
    }
    let output: SchemaNormalizationOutput =
        serde_json::from_slice(payload).map_err(|error| serialization_error(&error))?;
    if output.capabilities.is_empty()
        || output.capabilities.iter().any(|capability| {
            !capability.starts_with("sql.")
                || capability.len() > 96
                || capability.bytes().any(|byte| {
                    !(byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'-'))
                })
        })
    {
        return Err(invalid(
            "schema normalizer returned an invalid provider capability set",
        ));
    }
    let expected = sources
        .iter()
        .map(|source| (source.document.clone(), source.kind))
        .collect::<BTreeMap<_, _>>();
    let observed = output
        .documents
        .iter()
        .map(|document| (document.document.clone(), document.kind))
        .collect::<BTreeMap<_, _>>();
    if expected.len() != sources.len()
        || observed.len() != output.documents.len()
        || expected != observed
    {
        return Err(invalid(
            "schema normalizer changed, omitted, or duplicated a source identity or kind",
        ));
    }
    let schema = normalize_schema(provider, output.dialect, output.documents)?;
    Ok(SchemaNormalizationResult {
        schema,
        capabilities: output.capabilities,
    })
}

pub fn provider_analysis_from_response(
    response: &EmbeddedAnalysisResponse,
) -> Result<ProviderAnalysis, SchemaContractError> {
    if !response.plan.diagnostics.is_empty() {
        return Err(invalid("provider analysis returned diagnostics"));
    }
    let mut payloads = response.plan.operations.iter().filter_map(|operation| {
        if let SemanticOperation::ProviderNode { tag, payload } = operation
            && tag == PROVIDER_ANALYSIS_PAYLOAD_TAG
        {
            Some(payload)
        } else {
            None
        }
    });
    let payload = payloads
        .next()
        .ok_or_else(|| invalid("provider analysis payload is missing"))?;
    if payloads.next().is_some() {
        return Err(invalid("provider analysis payload is duplicated"));
    }
    serde_json::from_slice(payload).map_err(|error| serialization_error(&error))
}

const fn source_artifact_kind(kind: SchemaDocumentKind) -> &'static str {
    match kind {
        SchemaDocumentKind::SqlDdl => "sifr.sql.schema-source.sql-ddl",
        SchemaDocumentKind::ProviderMetadata => "sifr.sql.schema-source.provider-metadata",
        SchemaDocumentKind::GeneratedDefinitions => "sifr.sql.schema-source.generated-definitions",
    }
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn invalid(message: impl Into<String>) -> SchemaContractError {
    SchemaContractError::new(SchemaContractErrorKind::InvalidSchema, message)
}

fn serialization_error(error: &serde_json::Error) -> SchemaContractError {
    SchemaContractError::new(
        SchemaContractErrorKind::Serialization,
        format!("cannot serialize schema component payload: {error}"),
    )
}
