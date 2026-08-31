use crate::analyzer::MysqlAnalyzer;
use crate::diagnostic::{MysqlDiagnostic, MysqlDiagnosticCode, provider_diagnostic_registry};
use crate::lower_hex;
use crate::parser::MysqlParser;
use crate::schema::{MysqlSchemaOptions, normalize_mysql_documents};
use crate::types::{MysqlServerSeries, SUPPORTED_MYSQL_SERIES};
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

pub const MYSQL_QUERY_OPERATION: &str = "sifr.sql.mysql.sql";
pub const MYSQL_SCHEMA_ARTIFACT_KIND: &str = "sifr.sql.schema-ir";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MysqlComponentRequest {
    NormalizeSchema {
        provider: ProviderIdentity,
        server_series: MysqlServerSeriesRecord,
        options: MysqlSchemaOptions,
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
pub struct MysqlServerSeriesRecord {
    pub major: u16,
    pub minor: u16,
}

impl From<MysqlServerSeries> for MysqlServerSeriesRecord {
    fn from(value: MysqlServerSeries) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
        }
    }
}

impl From<MysqlServerSeriesRecord> for MysqlServerSeries {
    fn from(value: MysqlServerSeriesRecord) -> Self {
        Self::new(value.major, value.minor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MysqlComponentResponse {
    Schema(SchemaNormalizationOutput),
    Query(ProviderAnalysis),
    Diagnostic(MysqlDiagnostic),
}

pub struct MysqlCompilerComponent {
    parser: MysqlParser,
}

impl MysqlCompilerComponent {
    #[must_use]
    pub fn new(parser: MysqlParser) -> Self {
        Self { parser }
    }

    #[must_use]
    pub fn parser(&self) -> &MysqlParser {
        &self.parser
    }

    pub fn execute(&self, request: MysqlComponentRequest) -> MysqlComponentResponse {
        match self.execute_checked(request) {
            Ok(response) => response,
            Err(diagnostic) => MysqlComponentResponse::Diagnostic(diagnostic),
        }
    }

    fn execute_checked(
        &self,
        request: MysqlComponentRequest,
    ) -> Result<MysqlComponentResponse, MysqlDiagnostic> {
        match request {
            MysqlComponentRequest::NormalizeSchema {
                provider,
                server_series,
                options,
                documents,
            } => {
                if MysqlServerSeries::from(server_series) != self.parser.series() {
                    return Err(component_diagnostic(
                        "MySQL component and requested server series differ",
                    ));
                }
                normalize_mysql_documents(provider, &self.parser, &options, documents)
                    .map(MysqlComponentResponse::Schema)
                    .map_err(|error| {
                        MysqlDiagnostic::at_sql(
                            MysqlDiagnosticCode::InvalidSchema,
                            error.message,
                            u32::try_from(error.offset).unwrap_or(u32::MAX),
                            u32::try_from(error.offset.saturating_add(1)).unwrap_or(u32::MAX),
                        )
                    })
            }
            MysqlComponentRequest::AnalyzeQuery {
                schema,
                source,
                sifr_document,
                sifr_start,
                sifr_end,
            } => {
                let mut response = MysqlAnalyzer::new(&self.parser, &schema)
                    .and_then(|analyzer| analyzer.analyze_query(&source))
                    .map(MysqlComponentResponse::Query);
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
pub fn mysql_capabilities() -> BTreeSet<String> {
    [
        "sql.mysql.collation",
        "sql.mysql.generated-columns",
        "sql.mysql.sql-mode",
        "sql.mysql.type.enum-set",
        "sql.mysql.type.unsigned",
        "sql.mysql.write.conflict",
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
        "sql.query.row-locking",
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
    series: MysqlServerSeries,
) -> Result<ComponentRegistration, MysqlDiagnostic> {
    if !SUPPORTED_MYSQL_SERIES.contains(&series) {
        return Err(MysqlDiagnostic::at_sql(
            MysqlDiagnosticCode::UnsupportedVersion,
            "unsupported MySQL server series",
            0,
            1,
        ));
    }
    let artifact_path = component_artifact_path(series);
    let artifact = fs::read(&artifact_path).map_err(|error| {
        component_diagnostic(format!(
            "cannot read MySQL compiler component '{}': {error}",
            artifact_path.display()
        ))
    })?;
    Ok(ComponentRegistration {
        identity: ComponentIdentity {
            package: "sifr-sql-mysql".to_string(),
            processor: processor(series),
            version: Version::new(0, 0, 0),
            sha256: lower_hex(&Sha256::digest(artifact)),
        },
        protocol: ProtocolRange {
            minimum: COMPONENT_PROTOCOL_MAJOR,
            maximum: COMPONENT_PROTOCOL_MAJOR,
        },
        artifact: format!("components/mysql-{}.{}.wasm", series.major, series.minor),
        diagnostics: provider_diagnostics(),
    })
}

#[must_use]
pub fn component_artifact_path(series: MysqlServerSeries) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("components")
        .join(format!("mysql-{}.{}.wasm", series.major, series.minor))
}

pub fn execute_embedded_request(
    request: EmbeddedAnalysisRequest,
) -> Result<EmbeddedAnalysisResponse, MysqlDiagnostic> {
    if request.protocol_major != COMPONENT_PROTOCOL_MAJOR {
        return Err(component_diagnostic(
            "MySQL component protocol major does not match the compiler",
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
        .filter(|artifact| artifact.kind == MYSQL_SCHEMA_ARTIFACT_KIND);
    let artifact = artifacts
        .next()
        .ok_or_else(|| component_diagnostic("MySQL analysis requires one SchemaIR artifact"))?;
    if artifacts.next().is_some() {
        return Err(component_diagnostic(
            "MySQL analysis accepts exactly one SchemaIR artifact",
        ));
    }
    let schema: SchemaIr = serde_json::from_slice(&artifact.payload)
        .map_err(|_| component_diagnostic("MySQL SchemaIR artifact is invalid"))?;
    let series = parse_series(&schema.dialect.server_version)?;
    if request.component.processor != MYSQL_QUERY_OPERATION
        && request.component.processor != processor(series)
    {
        return Err(component_diagnostic(
            "MySQL component identity and SchemaIR server series differ",
        ));
    }
    let parser = parser_from_schema(&schema, series)?;
    let (source, document, start, end) = template_source(&request.parts)?;
    let response =
        MysqlCompilerComponent::new(parser).execute(MysqlComponentRequest::AnalyzeQuery {
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
) -> Result<EmbeddedAnalysisResponse, MysqlDiagnostic> {
    if !request.component.processor.ends_with(".schema") {
        return Err(component_diagnostic(
            "MySQL schema normalization requires the schema processor",
        ));
    }
    let semantic = &request.context.semantic_profile;
    let version = semantic
        .get("server-version")
        .ok_or_else(|| component_diagnostic("MySQL schema profile has no server version"))?;
    let series = parse_series(version)?;
    if !SUPPORTED_MYSQL_SERIES.contains(&series) {
        return Err(component_diagnostic(
            "MySQL schema profile uses an unsupported series",
        ));
    }
    let sql_modes = semantic_json::<BTreeSet<String>>(semantic, "sql-modes")?;
    let extensions = semantic_json::<BTreeSet<String>>(semantic, "extensions")?;
    let search_path = semantic_json::<Vec<String>>(semantic, "search-path")?;
    let default_database = search_path
        .first()
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| component_diagnostic("MySQL schema profile needs one default database"))?;
    let default_character_set = semantic_json::<Option<String>>(semantic, "character-set")?
        .ok_or_else(|| component_diagnostic("MySQL schema profile needs a character set"))?;
    let default_collation = semantic_json::<Option<String>>(semantic, "collation")?
        .ok_or_else(|| component_diagnostic("MySQL schema profile needs a collation"))?;
    let mut documents = Vec::with_capacity(request.context.artifacts.len());
    for artifact in &request.context.artifacts {
        if artifact.kind != "sifr.sql.schema-source.sql-ddl" {
            return Err(component_diagnostic(
                "MySQL schema components accept SQL DDL sources only",
            ));
        }
        let source: SchemaSourceArtifact = serde_json::from_slice(&artifact.payload)
            .map_err(|_| component_diagnostic("MySQL schema source artifact is invalid"))?;
        let contents = String::from_utf8(source.contents)
            .map_err(|_| component_diagnostic("MySQL schema source must be UTF-8"))?;
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
    let parser = MysqlParser::new(series, sql_modes.clone(), default_collation.clone())
        .map_err(|error| component_diagnostic(error.message))?;
    let output = normalize_mysql_documents(
        provider,
        &parser,
        &MysqlSchemaOptions {
            default_database,
            default_character_set,
            default_collation,
            sql_modes,
            extensions,
        },
        documents,
    )
    .map_err(|error| component_diagnostic(error.message))?;
    let payload = serde_json::to_vec(&output)
        .map_err(|_| component_diagnostic("cannot serialize MySQL normalized schema"))?;
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
) -> Result<T, MysqlDiagnostic> {
    semantic
        .get(key)
        .ok_or_else(|| component_diagnostic(format!("MySQL schema profile has no {key}")))
        .and_then(|value| {
            serde_json::from_str(value)
                .map_err(|_| component_diagnostic(format!("MySQL schema profile {key} is invalid")))
        })
}

fn into_embedded_response(
    plan_kind: PlanKind,
    schema_identity: String,
    schema: &SchemaIr,
    response: MysqlComponentResponse,
) -> Result<EmbeddedAnalysisResponse, MysqlDiagnostic> {
    let MysqlComponentResponse::Query(analysis) = response else {
        return Err(component_diagnostic(
            "MySQL embedded query analysis did not return query facts",
        ));
    };
    let payload = serde_json::to_vec(&analysis)
        .map_err(|_| component_diagnostic("cannot serialize MySQL analysis"))?;
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
        declaration: "sifr.sql.mysql.runtime.execute".to_string(),
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

fn template_source(parts: &[TemplatePart]) -> Result<(String, String, u32, u32), MysqlDiagnostic> {
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
                "MySQL template parts must belong to one source document",
            ));
        }
        document.get_or_insert_with(|| span.document.clone());
        start = start.min(span.start);
        end = end.max(span.end);
    }
    if source.trim().is_empty() {
        return Err(component_diagnostic("MySQL template has no SQL source"));
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
    series: MysqlServerSeries,
) -> Result<MysqlParser, MysqlDiagnostic> {
    let sql_modes = schema
        .dialect
        .modes
        .iter()
        .filter(|mode| !mode.starts_with("character-set:") && !mode.starts_with("collation:"))
        .cloned();
    let collation = schema
        .dialect
        .modes
        .iter()
        .find_map(|mode| mode.strip_prefix("collation:"))
        .ok_or_else(|| component_diagnostic("MySQL SchemaIR has no default collation"))?;
    MysqlParser::new(series, sql_modes, collation)
        .map_err(|error| component_diagnostic(error.message))
}

fn parse_series(version: &str) -> Result<MysqlServerSeries, MysqlDiagnostic> {
    let mut parts = version.split('.');
    let major = parts.next().and_then(|value| value.parse().ok());
    let minor = parts.next().and_then(|value| value.parse().ok());
    match (major, minor) {
        (Some(major), Some(minor)) => Ok(MysqlServerSeries::new(major, minor)),
        _ => Err(component_diagnostic("MySQL SchemaIR has no server series")),
    }
}

fn processor(series: MysqlServerSeries) -> String {
    format!("{MYSQL_QUERY_OPERATION}.v{}-{}", series.major, series.minor)
}

fn stable_plan_fingerprint(
    provider_identity: &str,
    schema_identity: &str,
    analysis: &ProviderAnalysis,
    plan_kind: PlanKind,
) -> Result<String, MysqlDiagnostic> {
    let bytes = serde_json::to_vec(&(provider_identity, schema_identity, analysis, plan_kind))
        .map_err(|_| component_diagnostic("cannot fingerprint MySQL plan"))?;
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

fn component_diagnostic(message: impl Into<String>) -> MysqlDiagnostic {
    MysqlDiagnostic::at_sql(MysqlDiagnosticCode::ProviderContract, message, 0, 1)
}

#[allow(dead_code)]
fn schema_response_payload(
    response: &SchemaNormalizationOutput,
) -> Result<(String, Vec<u8>), MysqlDiagnostic> {
    serde_json::to_vec(response)
        .map(|payload| (SCHEMA_NORMALIZATION_PAYLOAD_TAG.to_string(), payload))
        .map_err(|_| component_diagnostic("cannot serialize MySQL schema response"))
}
