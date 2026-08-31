use crate::analyzer::SqliteAnalyzer;
use crate::diagnostic::{SqliteDiagnostic, SqliteDiagnosticCode, provider_diagnostic_registry};
use crate::lower_hex;
use crate::parser::SqliteParser;
use crate::schema::{SqliteSchemaOptions, normalize_sqlite_documents};
use crate::types::{SUPPORTED_SQLITE_SERIES, SqliteServerSeries};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_compiler_component::{
    COMPONENT_PROTOCOL_MAJOR, ClosedType, ComponentIdentity, ComponentRegistration,
    DependencyDescriptor, EmbeddedAnalysisRequest, EmbeddedAnalysisResponse, EmbeddedPlan,
    PlanKind, ProtocolRange, RecordField, RuntimeLowering, SemanticOperation, TemplatePart,
};
use sifr_sql_contract::{
    PROVIDER_ANALYSIS_PAYLOAD_TAG, ProviderAnalysis, ProviderIdentity,
    SCHEMA_NORMALIZATION_OPERATION, SCHEMA_NORMALIZATION_PAYLOAD_TAG, SchemaIr,
    SchemaNormalizationOutput, SchemaSourceArtifact, SifrType, schema_object_fingerprint,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const SQLITE_QUERY_OPERATION: &str = "sifr.sql.sqlite.sql";
pub const SQLITE_SCHEMA_ARTIFACT_KIND: &str = "sifr.sql.schema-ir";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SqliteComponentRequest {
    NormalizeSchema {
        provider: ProviderIdentity,
        server_series: SqliteServerSeriesRecord,
        options: SqliteSchemaOptions,
        documents: Vec<(String, String)>,
    },
    AnalyzeQuery {
        schema: SchemaIr,
        source: String,
        sifr_document: String,
        sifr_start: u32,
        sifr_end: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteServerSeriesRecord {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl From<SqliteServerSeries> for SqliteServerSeriesRecord {
    fn from(value: SqliteServerSeries) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

impl From<SqliteServerSeriesRecord> for SqliteServerSeries {
    fn from(value: SqliteServerSeriesRecord) -> Self {
        Self::new(value.major, value.minor, value.patch)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SqliteComponentResponse {
    Schema(SchemaNormalizationOutput),
    Query(ProviderAnalysis),
    Diagnostic(SqliteDiagnostic),
}

pub struct SqliteCompilerComponent {
    parser: SqliteParser,
}

impl SqliteCompilerComponent {
    #[must_use]
    pub fn new(parser: SqliteParser) -> Self {
        Self { parser }
    }

    #[must_use]
    pub fn parser(&self) -> &SqliteParser {
        &self.parser
    }

    pub fn execute(&self, request: SqliteComponentRequest) -> SqliteComponentResponse {
        match self.execute_checked(request) {
            Ok(response) => response,
            Err(diagnostic) => SqliteComponentResponse::Diagnostic(diagnostic),
        }
    }

    fn execute_checked(
        &self,
        request: SqliteComponentRequest,
    ) -> Result<SqliteComponentResponse, SqliteDiagnostic> {
        match request {
            SqliteComponentRequest::NormalizeSchema {
                provider,
                server_series,
                options,
                documents,
            } => {
                if SqliteServerSeries::from(server_series) != self.parser.series() {
                    return Err(component_diagnostic(
                        "SQLite component and requested server series differ",
                    ));
                }
                normalize_sqlite_documents(provider, &self.parser, &options, documents)
                    .map(SqliteComponentResponse::Schema)
                    .map_err(|error| {
                        SqliteDiagnostic::at_sql(
                            SqliteDiagnosticCode::InvalidSchema,
                            error.message,
                            u32::try_from(error.offset).unwrap_or(u32::MAX),
                            u32::try_from(error.offset.saturating_add(1)).unwrap_or(u32::MAX),
                        )
                    })
            }
            SqliteComponentRequest::AnalyzeQuery {
                schema,
                source,
                sifr_document,
                sifr_start,
                sifr_end,
            } => {
                let mut response = SqliteAnalyzer::new(&self.parser, &schema)
                    .and_then(|analyzer| analyzer.analyze_query(&source))
                    .map(SqliteComponentResponse::Query);
                if let Err(diagnostic) = &mut response {
                    diagnostic
                        .related
                        .push(sifr_sql_contract::ProviderDiagnosticSpan {
                            kind: "sifr".to_string(),
                            document: sifr_document,
                            start: sifr_start,
                            end: sifr_end,
                            label: "Sifr template source".to_string(),
                        });
                }
                response
            }
        }
    }
}

#[must_use]
pub fn sqlite_capabilities() -> BTreeSet<String> {
    [
        "sql.sqlite.collation",
        "sql.sqlite.generated-columns",
        "sql.sqlite.affinity",
        "sql.sqlite.attached-schema",
        "sql.sqlite.strict-table",
        "sql.sqlite.without-rowid",
        "sql.sqlite.write.conflict",
        "sql.bind.parameters",
        "sql.expression.case",
        "sql.expression.cast",
        "sql.expression.equality",
        "sql.expression.function",
        "sql.query.aggregate",
        "sql.query.common-table-expression",
        "sql.query.delete",
        "sql.query.insert",
        "sql.query.join",
        "sql.write.returning",
        "sql.query.select",
        "sql.query.set-operation",
        "sql.query.subquery",
        "sql.query.update",
        "sql.query.window",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

#[must_use]
pub fn provider_diagnostics() -> sifr_compiler_component::DiagnosticRegistry {
    provider_diagnostic_registry()
}

pub fn component_registration(
    series: SqliteServerSeries,
) -> Result<ComponentRegistration, SqliteDiagnostic> {
    if !SUPPORTED_SQLITE_SERIES.contains(&series) {
        return Err(SqliteDiagnostic::at_sql(
            SqliteDiagnosticCode::UnsupportedVersion,
            "unsupported SQLite server series",
            0,
            1,
        ));
    }
    let artifact_path = component_artifact_path(series);
    let artifact = fs::read(&artifact_path).map_err(|error| {
        component_diagnostic(format!(
            "cannot read SQLite compiler component '{}': {error}",
            artifact_path.display()
        ))
    })?;
    Ok(ComponentRegistration {
        identity: ComponentIdentity {
            package: "sifr-sql-sqlite".to_string(),
            processor: processor(series),
            version: Version::new(0, 0, 0),
            sha256: lower_hex(&Sha256::digest(artifact)),
        },
        protocol: ProtocolRange {
            minimum: COMPONENT_PROTOCOL_MAJOR,
            maximum: COMPONENT_PROTOCOL_MAJOR,
        },
        artifact: format!(
            "components/sqlite-{}.{}.{}.wasm",
            series.major, series.minor, series.patch
        ),
        diagnostics: provider_diagnostics(),
    })
}

#[must_use]
pub fn component_artifact_path(series: SqliteServerSeries) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("components")
        .join(format!(
            "sqlite-{}.{}.{}.wasm",
            series.major, series.minor, series.patch
        ))
}

pub fn execute_embedded_request(
    request: EmbeddedAnalysisRequest,
) -> Result<EmbeddedAnalysisResponse, SqliteDiagnostic> {
    if request.protocol_major != COMPONENT_PROTOCOL_MAJOR {
        return Err(component_diagnostic(
            "SQLite component protocol major does not match the compiler",
        ));
    }
    if request
        .context
        .semantic_profile
        .get("operation")
        .is_some_and(|operation| operation == SCHEMA_NORMALIZATION_OPERATION)
    {
        return execute_schema_normalization(request);
    }
    let mut artifacts = request
        .context
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == SQLITE_SCHEMA_ARTIFACT_KIND);
    let artifact = artifacts
        .next()
        .ok_or_else(|| component_diagnostic("SQLite analysis requires one SchemaIR artifact"))?;
    if artifacts.next().is_some() {
        return Err(component_diagnostic(
            "SQLite analysis accepts exactly one SchemaIR artifact",
        ));
    }
    let schema: SchemaIr = serde_json::from_slice(&artifact.payload)
        .map_err(|_| component_diagnostic("SQLite SchemaIR artifact is invalid"))?;
    let series = parse_series(&schema.dialect.server_version)?;
    if request.component.processor != SQLITE_QUERY_OPERATION
        && request.component.processor != processor(series)
    {
        return Err(component_diagnostic(
            "SQLite component identity and SchemaIR server series differ",
        ));
    }
    let parser = parser_from_schema(&schema, series)?;
    let (source, document, start, end) = template_source(&request.parts)?;
    let response =
        SqliteCompilerComponent::new(parser).execute(SqliteComponentRequest::AnalyzeQuery {
            schema: schema.clone(),
            source,
            sifr_document: document,
            sifr_start: start,
            sifr_end: end,
        });
    into_embedded_response(
        request.plan_kind,
        artifact.identity.clone(),
        &schema,
        response,
    )
}

fn execute_schema_normalization(
    request: EmbeddedAnalysisRequest,
) -> Result<EmbeddedAnalysisResponse, SqliteDiagnostic> {
    if !request.component.processor.ends_with(".schema") {
        return Err(component_diagnostic(
            "SQLite schema normalization requires the schema processor",
        ));
    }
    let semantic = &request.context.semantic_profile;
    let version = semantic
        .get("server-version")
        .ok_or_else(|| component_diagnostic("SQLite schema profile has no server version"))?;
    let series = parse_series(version)?;
    if !SUPPORTED_SQLITE_SERIES.contains(&series) {
        return Err(component_diagnostic(
            "SQLite schema profile uses an unsupported series",
        ));
    }
    let compile_flags = semantic_json::<BTreeSet<String>>(semantic, "sql-modes")?;
    let extensions = semantic_json::<BTreeSet<String>>(semantic, "extensions")?;
    let required_features = extensions.clone();
    let search_path = semantic_json::<Vec<String>>(semantic, "search-path")?;
    let default_schema = search_path
        .first()
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| component_diagnostic("SQLite schema profile needs the main schema"))?;
    let attached_schemas = search_path.into_iter().skip(1).collect();
    let mut documents = Vec::with_capacity(request.context.artifacts.len());
    for artifact in &request.context.artifacts {
        if artifact.kind != "sifr.sql.schema-source.sql-ddl" {
            return Err(component_diagnostic(
                "SQLite schema components accept SQL DDL sources only",
            ));
        }
        let source: SchemaSourceArtifact = serde_json::from_slice(&artifact.payload)
            .map_err(|_| component_diagnostic("SQLite schema source artifact is invalid"))?;
        let contents = String::from_utf8(source.contents)
            .map_err(|_| component_diagnostic("SQLite schema source must be UTF-8"))?;
        documents.push((artifact.identity.clone(), contents));
    }
    let provider = ProviderIdentity {
        package_id: request.component.package.clone(),
        package_version: request.component.version.clone(),
        package_source: "compiler-component".to_string(),
        package_graph_digest: request.component.sha256.clone(),
        compiler_components: BTreeMap::from([(
            request.component.processor.clone(),
            request.component.sha256.clone(),
        )]),
    };
    let parser = SqliteParser::new(series, compile_flags.clone())
        .map_err(|error| component_diagnostic(error.message))?;
    let output = normalize_sqlite_documents(
        provider,
        &parser,
        &SqliteSchemaOptions {
            default_schema,
            compile_flags,
            attached_schemas,
            required_features,
            extensions,
        },
        documents,
    )
    .map_err(|error| component_diagnostic(error.message))?;
    let payload = serde_json::to_vec(&output)
        .map_err(|_| component_diagnostic("cannot serialize SQLite normalized schema"))?;
    let provider_identity = format!(
        "{}@{}#{}",
        request.component.package, request.component.version, request.component.sha256
    );
    let mut response = EmbeddedAnalysisResponse {
        protocol_major: COMPONENT_PROTOCOL_MAJOR,
        plan: EmbeddedPlan {
            provider_identity,
            protocol_major: COMPONENT_PROTOCOL_MAJOR,
            plan_kind: PlanKind::Document,
            schema_identity: request.context.schema_profile,
            result_type: ClosedType::None,
            operations: vec![SemanticOperation::ProviderNode {
                tag: SCHEMA_NORMALIZATION_PAYLOAD_TAG.to_string(),
                payload,
            }],
            runtime: RuntimeLowering::NoRuntime,
            dependencies: Vec::new(),
            diagnostics: Vec::new(),
            source_map: Vec::new(),
            stable_fingerprint: String::new(),
        },
    };
    response.plan.stable_fingerprint =
        sifr_compiler_component::compute_plan_fingerprint(&response.plan)
            .map_err(|error| component_diagnostic(error.to_string()))?;
    Ok(response)
}

fn semantic_json<T: serde::de::DeserializeOwned>(
    semantic: &BTreeMap<String, String>,
    key: &str,
) -> Result<T, SqliteDiagnostic> {
    semantic
        .get(key)
        .ok_or_else(|| component_diagnostic(format!("SQLite schema profile has no {key}")))
        .and_then(|value| {
            serde_json::from_str(value).map_err(|_| {
                component_diagnostic(format!("SQLite schema profile {key} is invalid"))
            })
        })
}

fn into_embedded_response(
    plan_kind: PlanKind,
    schema_identity: String,
    schema: &SchemaIr,
    response: SqliteComponentResponse,
) -> Result<EmbeddedAnalysisResponse, SqliteDiagnostic> {
    let SqliteComponentResponse::Query(analysis) = response else {
        return Err(component_diagnostic(
            "SQLite embedded query analysis did not return query facts",
        ));
    };
    let payload = serde_json::to_vec(&analysis)
        .map_err(|_| component_diagnostic("cannot serialize SQLite analysis"))?;
    let result_type = if analysis.result_fields.is_empty() {
        ClosedType::None
    } else {
        ClosedType::List {
            item: Box::new(ClosedType::Record {
                fields: analysis
                    .result_fields
                    .iter()
                    .map(|field| RecordField {
                        name: field.name.clone(),
                        ty: closed_type(&field.sifr_type),
                    })
                    .collect(),
            }),
        }
    };
    let dependencies = analysis
        .accessed_objects
        .iter()
        .filter_map(|identity| {
            schema.objects.get(identity).and_then(|object| {
                schema_object_fingerprint(object)
                    .ok()
                    .map(|fingerprint| DependencyDescriptor {
                        identity: identity.to_string(),
                        fingerprint,
                    })
            })
        })
        .collect();
    let parameter_order = analysis
        .parameters
        .iter()
        .map(|parameter| parameter.slot)
        .collect();
    let provider_identity = format!(
        "{}@{}#{}",
        schema.provider.package_id,
        schema.provider.package_version,
        schema.provider.package_graph_digest
    );
    let operations = vec![SemanticOperation::ProviderNode {
        tag: PROVIDER_ANALYSIS_PAYLOAD_TAG.to_string(),
        payload: payload.clone(),
    }];
    let runtime = RuntimeLowering::ProviderCall {
        declaration: "sifr.sql.sqlite.runtime.execute".to_string(),
        payload,
        parameter_order,
    };
    let stable_fingerprint =
        stable_plan_fingerprint(&provider_identity, &schema_identity, &analysis, plan_kind)?;
    Ok(EmbeddedAnalysisResponse {
        protocol_major: COMPONENT_PROTOCOL_MAJOR,
        plan: EmbeddedPlan {
            provider_identity,
            protocol_major: COMPONENT_PROTOCOL_MAJOR,
            plan_kind,
            schema_identity: Some(schema_identity),
            result_type,
            operations,
            runtime,
            dependencies,
            diagnostics: Vec::new(),
            source_map: Vec::new(),
            stable_fingerprint,
        },
    })
}

fn template_source(parts: &[TemplatePart]) -> Result<(String, String, u32, u32), SqliteDiagnostic> {
    let mut source = String::new();
    let mut document = None;
    let mut start = u32::MAX;
    let mut end = 0;
    for part in parts {
        let span = match part {
            TemplatePart::Static { text, span } => {
                source.push_str(text);
                span
            }
            TemplatePart::Hole { span, .. } => {
                source.push('?');
                span
            }
        };
        if document
            .as_ref()
            .is_some_and(|value| value != &span.document)
        {
            return Err(component_diagnostic(
                "SQLite template parts must belong to one source document",
            ));
        }
        document.get_or_insert_with(|| span.document.clone());
        start = start.min(span.start);
        end = end.max(span.end);
    }
    if source.trim().is_empty() {
        return Err(component_diagnostic("SQLite template has no SQL source"));
    }
    Ok((
        source,
        document.unwrap_or_else(|| "sifr://sql/query".to_string()),
        start,
        end,
    ))
}

fn parser_from_schema(
    schema: &SchemaIr,
    series: SqliteServerSeries,
) -> Result<SqliteParser, SqliteDiagnostic> {
    SqliteParser::new(series, schema.dialect.modes.iter().cloned())
        .map_err(|error| component_diagnostic(error.message))
}

fn parse_series(version: &str) -> Result<SqliteServerSeries, SqliteDiagnostic> {
    let mut parts = version.split('.');
    let major = parts.next().and_then(|value| value.parse().ok());
    let minor = parts.next().and_then(|value| value.parse().ok());
    let patch = parts.next().and_then(|value| value.parse().ok());
    match (major, minor, patch, parts.next()) {
        (Some(major), Some(minor), Some(patch), None) => {
            Ok(SqliteServerSeries::new(major, minor, patch))
        }
        _ => Err(component_diagnostic("SQLite SchemaIR has no server series")),
    }
}

fn processor(series: SqliteServerSeries) -> String {
    format!(
        "{SQLITE_QUERY_OPERATION}.v{}-{}-{}",
        series.major, series.minor, series.patch
    )
}

fn stable_plan_fingerprint(
    provider_identity: &str,
    schema_identity: &str,
    analysis: &ProviderAnalysis,
    plan_kind: PlanKind,
) -> Result<String, SqliteDiagnostic> {
    let bytes = serde_json::to_vec(&(provider_identity, schema_identity, analysis, plan_kind))
        .map_err(|_| component_diagnostic("cannot fingerprint SQLite plan"))?;
    Ok(lower_hex(&Sha256::digest(bytes)))
}

fn closed_type(ty: &SifrType) -> ClosedType {
    match ty {
        SifrType::Bool => ClosedType::Bool,
        SifrType::FixedInteger { .. } | SifrType::ExactInteger => ClosedType::Int,
        SifrType::Float | SifrType::Decimal | SifrType::BigDecimal | SifrType::Numeric => {
            ClosedType::Float
        }
        SifrType::Str => ClosedType::Str,
        SifrType::Bytes => ClosedType::Bytes,
        SifrType::None => ClosedType::None,
        SifrType::Union { members } if members.contains(&SifrType::None) && members.len() == 2 => {
            let item = members
                .iter()
                .find(|member| **member != SifrType::None)
                .map(closed_type)
                .unwrap_or(ClosedType::None);
            ClosedType::Optional {
                item: Box::new(item),
            }
        }
        SifrType::List { element } | SifrType::SqlArray { element } => ClosedType::List {
            item: Box::new(closed_type(element)),
        },
        _ => ClosedType::Str,
    }
}

fn component_diagnostic(message: impl Into<String>) -> SqliteDiagnostic {
    SqliteDiagnostic::at_sql(SqliteDiagnosticCode::ProviderContract, message, 0, 1)
}

#[allow(dead_code)]
fn schema_response_payload(
    response: &SchemaNormalizationOutput,
) -> Result<(String, Vec<u8>), SqliteDiagnostic> {
    serde_json::to_vec(response)
        .map(|payload| (SCHEMA_NORMALIZATION_PAYLOAD_TAG.to_string(), payload))
        .map_err(|_| component_diagnostic("cannot serialize SQLite schema response"))
}
