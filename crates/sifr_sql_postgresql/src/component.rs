use crate::analysis::{PostgresAnalysisError, PostgresAnalyzer};
use crate::catalog::{PostgresCatalog, ddl_document};
use crate::diagnostic::{PostgresDiagnostic, PostgresDiagnosticCode};
use crate::raw_adapter::PostgresParser;
use crate::types::PostgresTypeRegistry;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_compiler_component::{
    COMPONENT_PROTOCOL_MAJOR, ClosedType, ComponentIdentity, ComponentRegistration,
    DependencyDescriptor, DiagnosticCodeDeclaration, DiagnosticLifecycle, DiagnosticRegistry,
    DiagnosticRegistryOwner, EmbeddedAnalysisResponse, EmbeddedPlan, PlanKind, ProtocolRange,
    RecordField, RuntimeLowering, SemanticOperation,
};
use sifr_sql_contract::{
    DialectIdentity, ProviderAnalysis, ProviderIdentity, SchemaIr, SchemaNormalizationOutput,
    SifrType, normalize_schema,
};
use std::collections::BTreeSet;

pub const POSTGRESQL_QUERY_OPERATION: &str = "sifr.sql.postgresql.sql";
pub(crate) const POSTGRESQL_QUERY_PAYLOAD_TAG: &str = "sifr.sql.postgresql.analysis";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PostgresComponentRequest {
    NormalizeSchema {
        provider: ProviderIdentity,
        server_major: u16,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PostgresComponentResponse {
    Schema(SchemaNormalizationOutput),
    Query(ProviderAnalysis),
    Diagnostic(PostgresDiagnostic),
}

pub struct PostgresCompilerComponent<P> {
    parser: P,
}

impl<P: PostgresParser> PostgresCompilerComponent<P> {
    #[must_use]
    pub fn new(parser: P) -> Self {
        Self { parser }
    }

    pub fn execute(&self, request: PostgresComponentRequest) -> PostgresComponentResponse {
        match self.execute_checked(request) {
            Ok(response) => response,
            Err(error) => PostgresComponentResponse::Diagnostic(error.diagnostic),
        }
    }

    fn execute_checked(
        &self,
        request: PostgresComponentRequest,
    ) -> Result<PostgresComponentResponse, PostgresAnalysisError> {
        match request {
            PostgresComponentRequest::NormalizeSchema {
                provider,
                server_major,
                documents,
            } => {
                if server_major != self.parser.server_major() {
                    return Err(component_error(format!(
                        "component parser major {} cannot normalize PostgreSQL {server_major}",
                        self.parser.server_major()
                    )));
                }
                let types = PostgresTypeRegistry::new(server_major);
                let mut normalized = Vec::with_capacity(documents.len());
                for (document, source) in documents {
                    let statements = self.parser.parse(&source)?;
                    normalized.push(
                        ddl_document(document, &statements, &types)
                            .map_err(|diagnostic| PostgresAnalysisError { diagnostic })?,
                    );
                }
                let dialect = dialect(server_major);
                normalize_schema(provider, dialect.clone(), normalized.clone())
                    .map_err(|error| component_error(error.to_string()))?;
                Ok(PostgresComponentResponse::Schema(
                    SchemaNormalizationOutput {
                        dialect,
                        documents: normalized,
                    },
                ))
            }
            PostgresComponentRequest::AnalyzeQuery {
                schema,
                source,
                sifr_document,
                sifr_start,
                sifr_end,
            } => {
                let server_major = schema
                    .dialect
                    .server_version
                    .split('.')
                    .next()
                    .and_then(|major| major.parse::<u16>().ok())
                    .ok_or_else(|| component_error("schema has no PostgreSQL server major"))?;
                if server_major != self.parser.server_major() {
                    return Err(component_error("schema and component parser majors differ"));
                }
                let catalog =
                    PostgresCatalog::from_schema(&schema, PostgresTypeRegistry::new(server_major))
                        .map_err(|diagnostic| PostgresAnalysisError { diagnostic })?;
                let analysis = PostgresAnalyzer::new(&self.parser, catalog)
                    .analyze_query_with_sifr_span(&source, &sifr_document, sifr_start, sifr_end)?;
                Ok(PostgresComponentResponse::Query(analysis))
            }
        }
    }
}

impl<T: PostgresParser + ?Sized> PostgresParser for &T {
    fn server_major(&self) -> u16 {
        (**self).server_major()
    }

    fn parse(
        &self,
        source: &str,
    ) -> Result<Vec<crate::PostgresStatement>, crate::PostgresParseError> {
        (**self).parse(source)
    }

    fn normalize(&self, source: &str) -> Result<String, crate::PostgresParseError> {
        (**self).normalize(source)
    }
}

pub fn component_registration(
    server_major: u16,
    artifact_sha256: impl Into<String>,
) -> Result<ComponentRegistration, PostgresDiagnostic> {
    if !crate::SUPPORTED_POSTGRESQL_MAJORS.contains(&server_major) {
        return Err(PostgresDiagnostic::at_sql(
            PostgresDiagnosticCode::UnsupportedCoreSyntax,
            format!("PostgreSQL {server_major} is not supported"),
            0,
            1,
        ));
    }
    Ok(ComponentRegistration {
        identity: ComponentIdentity {
            package: "sifr-sql-postgresql".to_string(),
            processor: format!("{POSTGRESQL_QUERY_OPERATION}.v{server_major}"),
            version: Version::new(0, 0, 0),
            sha256: artifact_sha256.into(),
        },
        protocol: ProtocolRange {
            minimum: COMPONENT_PROTOCOL_MAJOR,
            maximum: COMPONENT_PROTOCOL_MAJOR,
        },
        artifact: format!("components/postgresql-{server_major}.wasm"),
        diagnostics: provider_diagnostics(),
    })
}

#[must_use]
pub fn provider_diagnostics() -> DiagnosticRegistry {
    DiagnosticRegistry {
        owner: DiagnosticRegistryOwner::Provider {
            namespace: "SQL-POSTGRESQL".to_string(),
        },
        declarations: [
            PostgresDiagnosticCode::Parse,
            PostgresDiagnosticCode::UnknownRelation,
            PostgresDiagnosticCode::UnknownColumn,
            PostgresDiagnosticCode::AmbiguousColumn,
            PostgresDiagnosticCode::TypeMismatch,
            PostgresDiagnosticCode::UnknownFunction,
            PostgresDiagnosticCode::UnknownOperator,
            PostgresDiagnosticCode::InvalidParameter,
            PostgresDiagnosticCode::InvalidWrite,
            PostgresDiagnosticCode::InvalidResult,
            PostgresDiagnosticCode::UnsupportedCoreSyntax,
        ]
        .into_iter()
        .map(|code| DiagnosticCodeDeclaration {
            code: code.as_str().to_string(),
            lifecycle: DiagnosticLifecycle::Active,
        })
        .collect(),
    }
}

pub fn into_embedded_response(
    server_major: u16,
    schema_identity: Option<String>,
    schema_fingerprint: &str,
    response: &PostgresComponentResponse,
) -> Result<EmbeddedAnalysisResponse, PostgresDiagnostic> {
    let payload = serde_json::to_vec(response).map_err(|_| {
        PostgresDiagnostic::at_sql(
            PostgresDiagnosticCode::UnsupportedCoreSyntax,
            "cannot serialize PostgreSQL component response",
            0,
            1,
        )
    })?;
    let fingerprint = lower_hex(&Sha256::digest(
        [schema_fingerprint.as_bytes(), payload.as_slice()].concat(),
    ));
    let (plan_kind, result_type, runtime, dependencies, diagnostics) = match response {
        PostgresComponentResponse::Query(analysis) => (
            PlanKind::Expression,
            if analysis.result_fields.is_empty() {
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
            },
            RuntimeLowering::ProviderCall {
                declaration: "sifr.sql.postgresql.execute".to_string(),
                payload: payload.clone(),
                parameter_order: analysis
                    .parameters
                    .iter()
                    .map(|parameter| parameter.slot)
                    .collect(),
            },
            analysis
                .effects
                .referenced_objects
                .union(&analysis.effects.affected_objects)
                .map(|identity| DependencyDescriptor {
                    identity: identity.as_str().to_string(),
                    fingerprint: schema_fingerprint.to_string(),
                })
                .collect(),
            Vec::new(),
        ),
        PostgresComponentResponse::Schema(_) => (
            PlanKind::Document,
            ClosedType::None,
            RuntimeLowering::NoRuntime,
            Vec::new(),
            Vec::new(),
        ),
        PostgresComponentResponse::Diagnostic(diagnostic) => {
            let embedded = sifr_compiler_component::EmbeddedDiagnostic {
                code: diagnostic.code.as_str().to_string(),
                severity: sifr_compiler_component::DiagnosticSeverity::Error,
                lifecycle: DiagnosticLifecycle::Active,
                message: diagnostic.message.clone(),
                primary: sifr_compiler_component::SourceSpan {
                    document: diagnostic.primary.document.clone(),
                    start: diagnostic.primary.start,
                    end: diagnostic.primary.end,
                },
                related: diagnostic
                    .related
                    .iter()
                    .map(|span| sifr_compiler_component::SourceSpan {
                        document: span.document.clone(),
                        start: span.start,
                        end: span.end,
                    })
                    .collect(),
            };
            (
                PlanKind::Expression,
                ClosedType::None,
                RuntimeLowering::NoRuntime,
                Vec::new(),
                vec![embedded],
            )
        }
    };
    Ok(EmbeddedAnalysisResponse {
        protocol_major: COMPONENT_PROTOCOL_MAJOR,
        plan: EmbeddedPlan {
            provider_identity: format!("sifr.sql.postgresql.{server_major}"),
            protocol_major: COMPONENT_PROTOCOL_MAJOR,
            plan_kind,
            schema_identity,
            result_type,
            operations: vec![SemanticOperation::ProviderNode {
                tag: POSTGRESQL_QUERY_PAYLOAD_TAG.to_string(),
                payload,
            }],
            runtime,
            dependencies,
            diagnostics,
            source_map: Vec::new(),
            stable_fingerprint: fingerprint,
        },
    })
}

fn closed_type(ty: &SifrType) -> ClosedType {
    match ty {
        SifrType::Bool => ClosedType::Bool,
        SifrType::FixedInteger { .. } | SifrType::ExactInteger => ClosedType::Int,
        SifrType::Float => ClosedType::Float,
        SifrType::Bytes => ClosedType::Bytes,
        SifrType::None => ClosedType::None,
        SifrType::List { element } | SifrType::SqlArray { element } => ClosedType::List {
            item: Box::new(closed_type(element)),
        },
        SifrType::Union { members } if members.len() == 2 && members.contains(&SifrType::None) => {
            let item = members
                .iter()
                .find(|member| **member != SifrType::None)
                .map(closed_type)
                .unwrap_or(ClosedType::None);
            ClosedType::Optional {
                item: Box::new(item),
            }
        }
        SifrType::Decimal
        | SifrType::BigDecimal
        | SifrType::Numeric
        | SifrType::Str
        | SifrType::Date
        | SifrType::LocalTime
        | SifrType::OffsetTime
        | SifrType::LocalDateTime
        | SifrType::Instant
        | SifrType::CalendarInterval
        | SifrType::Uuid
        | SifrType::JsonValue
        | SifrType::Nominal { .. }
        | SifrType::Range { .. }
        | SifrType::IpAddress
        | SifrType::IpNetwork
        | SifrType::MacAddress
        | SifrType::Custom { .. }
        | SifrType::Union { .. } => ClosedType::Str,
    }
}

fn dialect(server_major: u16) -> DialectIdentity {
    DialectIdentity {
        family: "postgresql".to_string(),
        server_version: server_major.to_string(),
        modes: BTreeSet::new(),
        features: BTreeSet::from(["libpg-query".to_string(), "core-semantics".to_string()]),
    }
}

fn component_error(message: impl Into<String>) -> PostgresAnalysisError {
    PostgresAnalysisError {
        diagnostic: PostgresDiagnostic::at_sql(
            PostgresDiagnosticCode::UnsupportedCoreSyntax,
            message,
            0,
            1,
        ),
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
