use crate::{BindRejection, ProviderAnalysisError, SchemaContractErrorKind};
use serde::{Deserialize, Serialize};
use sifr_diagnostics::DiagnosticCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommonSqlDiagnostic {
    DatabaseType,
    BindCompatibility,
    Nullability,
    CodecContract,
    Cardinality,
    Effect,
    ProviderContract,
    Ownership,
}

impl CommonSqlDiagnostic {
    #[must_use]
    pub const fn code(self) -> &'static str {
        self.diagnostic_code().code()
    }

    #[must_use]
    pub const fn diagnostic_code(self) -> DiagnosticCode {
        match self {
            Self::DatabaseType => DiagnosticCode::SQL_DATABASE_TYPE,
            Self::BindCompatibility => DiagnosticCode::SQL_BIND_COMPATIBILITY,
            Self::Nullability => DiagnosticCode::SQL_NULLABILITY,
            Self::CodecContract => DiagnosticCode::SQL_CODEC_CONTRACT,
            Self::Cardinality => DiagnosticCode::SQL_CARDINALITY,
            Self::Effect => DiagnosticCode::SQL_EFFECT,
            Self::ProviderContract => DiagnosticCode::SQL_PROVIDER_CONTRACT,
            Self::Ownership => DiagnosticCode::SQL_OWNERSHIP,
        }
    }

    #[must_use]
    pub const fn for_bind_rejection(rejection: BindRejection) -> Self {
        match rejection {
            BindRejection::Nullability => Self::Nullability,
            BindRejection::MissingCodec => Self::CodecContract,
            BindRejection::IntegerWidth
            | BindRejection::IntegerSign
            | BindRejection::ArrayElement
            | BindRejection::NominalIdentity
            | BindRejection::UnsupportedPair => Self::BindCompatibility,
        }
    }

    #[must_use]
    pub const fn for_schema_error(kind: SchemaContractErrorKind) -> Self {
        match kind {
            SchemaContractErrorKind::InvalidProfile | SchemaContractErrorKind::InvalidProvider => {
                Self::ProviderContract
            }
            SchemaContractErrorKind::InvalidSchema
            | SchemaContractErrorKind::DuplicateObject
            | SchemaContractErrorKind::MissingDependency
            | SchemaContractErrorKind::AmbiguousSymbol
            | SchemaContractErrorKind::UnknownSymbol
            | SchemaContractErrorKind::ReservedExport
            | SchemaContractErrorKind::IncompatibleSchema
            | SchemaContractErrorKind::Serialization => Self::DatabaseType,
        }
    }

    #[must_use]
    pub const fn for_provider_error(error: &ProviderAnalysisError) -> Self {
        match error {
            ProviderAnalysisError::UnsupportedDatabaseType(_) => Self::DatabaseType,
            ProviderAnalysisError::InvalidBind { .. } => Self::BindCompatibility,
            ProviderAnalysisError::InvalidResultField { .. }
            | ProviderAnalysisError::InvalidDialectSemantics => Self::ProviderContract,
        }
    }
}
