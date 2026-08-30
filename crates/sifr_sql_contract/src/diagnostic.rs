use crate::{BindRejection, ProviderAnalysisError, SchemaContractErrorKind};
use serde::{Deserialize, Serialize};

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
        match self {
            Self::DatabaseType => "SIFR-SQL-0001",
            Self::BindCompatibility => "SIFR-SQL-0002",
            Self::Nullability => "SIFR-SQL-0003",
            Self::CodecContract => "SIFR-SQL-0004",
            Self::Cardinality => "SIFR-SQL-0005",
            Self::Effect => "SIFR-SQL-0006",
            Self::ProviderContract => "SIFR-SQL-0007",
            Self::Ownership => "SIFR-SQL-0008",
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
