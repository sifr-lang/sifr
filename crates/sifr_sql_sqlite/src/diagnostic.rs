use serde::{Deserialize, Serialize};
use sifr_compiler_component::{
    DiagnosticCodeDeclaration, DiagnosticLifecycle, DiagnosticRegistry, DiagnosticRegistryOwner,
};
use sifr_sql_contract::{ProviderDiagnosticSpan, ProviderSemanticDiagnostic};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqliteDiagnosticCode {
    Syntax,
    UnsupportedVersion,
    UnsupportedMode,
    UnknownObject,
    UnknownColumn,
    AmbiguousColumn,
    TypeMismatch,
    CollationMismatch,
    UnsupportedFeature,
    InvalidSchema,
    ProviderContract,
}

impl SqliteDiagnosticCode {
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::Syntax => "SIFR-SQLITE-0001",
            Self::UnsupportedVersion => "SIFR-SQLITE-0002",
            Self::UnsupportedMode => "SIFR-SQLITE-0003",
            Self::UnknownObject => "SIFR-SQLITE-0004",
            Self::UnknownColumn => "SIFR-SQLITE-0005",
            Self::AmbiguousColumn => "SIFR-SQLITE-0006",
            Self::TypeMismatch => "SIFR-SQLITE-0007",
            Self::CollationMismatch => "SIFR-SQLITE-0008",
            Self::UnsupportedFeature => "SIFR-SQLITE-0009",
            Self::InvalidSchema => "SIFR-SQLITE-0010",
            Self::ProviderContract => "SIFR-SQLITE-0011",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteDiagnostic {
    pub code: SqliteDiagnosticCode,
    pub message: String,
    pub primary: Box<ProviderDiagnosticSpan>,
    pub related: Vec<ProviderDiagnosticSpan>,
}

impl SqliteDiagnostic {
    #[must_use]
    pub fn at_sql(
        code: SqliteDiagnosticCode,
        message: impl Into<String>,
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            primary: Box::new(ProviderDiagnosticSpan {
                kind: "sql-template".to_string(),
                document: "query.sql".to_string(),
                start,
                end,
                label: "SQLite analysis failed here".to_string(),
            }),
            related: Vec::new(),
        }
    }

    #[must_use]
    pub fn semantic(&self) -> ProviderSemanticDiagnostic {
        ProviderSemanticDiagnostic {
            code: self.code.code().to_string(),
            message: self.message.clone(),
            primary: (*self.primary).clone(),
            related: self.related.clone(),
        }
    }
}

#[must_use]
pub(crate) fn provider_diagnostic_registry() -> DiagnosticRegistry {
    let codes = [
        SqliteDiagnosticCode::Syntax,
        SqliteDiagnosticCode::UnsupportedVersion,
        SqliteDiagnosticCode::UnsupportedMode,
        SqliteDiagnosticCode::UnknownObject,
        SqliteDiagnosticCode::UnknownColumn,
        SqliteDiagnosticCode::AmbiguousColumn,
        SqliteDiagnosticCode::TypeMismatch,
        SqliteDiagnosticCode::CollationMismatch,
        SqliteDiagnosticCode::UnsupportedFeature,
        SqliteDiagnosticCode::InvalidSchema,
        SqliteDiagnosticCode::ProviderContract,
    ]
    .into_iter()
    .map(|code| DiagnosticCodeDeclaration {
        code: code.code().to_string(),
        lifecycle: DiagnosticLifecycle::Active,
    })
    .collect();
    DiagnosticRegistry {
        owner: DiagnosticRegistryOwner::Provider {
            namespace: "SQLITE".to_string(),
        },
        declarations: codes,
    }
}
