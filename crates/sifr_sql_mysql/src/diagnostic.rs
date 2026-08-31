use serde::{Deserialize, Serialize};
use sifr_compiler_component::{
    DiagnosticCodeDeclaration, DiagnosticLifecycle, DiagnosticRegistry, DiagnosticRegistryOwner,
};
use sifr_sql_contract::{ProviderDiagnosticSpan, ProviderSemanticDiagnostic};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MysqlDiagnosticCode {
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

impl MysqlDiagnosticCode {
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::Syntax => "SIFR-MYSQL-0001",
            Self::UnsupportedVersion => "SIFR-MYSQL-0002",
            Self::UnsupportedMode => "SIFR-MYSQL-0003",
            Self::UnknownObject => "SIFR-MYSQL-0004",
            Self::UnknownColumn => "SIFR-MYSQL-0005",
            Self::AmbiguousColumn => "SIFR-MYSQL-0006",
            Self::TypeMismatch => "SIFR-MYSQL-0007",
            Self::CollationMismatch => "SIFR-MYSQL-0008",
            Self::UnsupportedFeature => "SIFR-MYSQL-0009",
            Self::InvalidSchema => "SIFR-MYSQL-0010",
            Self::ProviderContract => "SIFR-MYSQL-0011",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlDiagnostic {
    pub code: MysqlDiagnosticCode,
    pub message: String,
    pub primary: Box<ProviderDiagnosticSpan>,
    pub related: Vec<ProviderDiagnosticSpan>,
}

impl MysqlDiagnostic {
    #[must_use]
    pub fn at_sql(
        code: MysqlDiagnosticCode,
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
                label: "MySQL analysis failed here".to_string(),
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
        MysqlDiagnosticCode::Syntax,
        MysqlDiagnosticCode::UnsupportedVersion,
        MysqlDiagnosticCode::UnsupportedMode,
        MysqlDiagnosticCode::UnknownObject,
        MysqlDiagnosticCode::UnknownColumn,
        MysqlDiagnosticCode::AmbiguousColumn,
        MysqlDiagnosticCode::TypeMismatch,
        MysqlDiagnosticCode::CollationMismatch,
        MysqlDiagnosticCode::UnsupportedFeature,
        MysqlDiagnosticCode::InvalidSchema,
        MysqlDiagnosticCode::ProviderContract,
    ]
    .into_iter()
    .map(|code| DiagnosticCodeDeclaration {
        code: code.code().to_string(),
        lifecycle: DiagnosticLifecycle::Active,
    })
    .collect();
    DiagnosticRegistry {
        owner: DiagnosticRegistryOwner::Provider {
            namespace: "MYSQL".to_string(),
        },
        declarations: codes,
    }
}
