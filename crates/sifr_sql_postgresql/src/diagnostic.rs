use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PostgresDiagnosticCode {
    Parse,
    UnknownRelation,
    UnknownColumn,
    AmbiguousColumn,
    TypeMismatch,
    UnknownFunction,
    UnknownOperator,
    InvalidParameter,
    InvalidWrite,
    InvalidResult,
    UnsupportedCoreSyntax,
}

impl PostgresDiagnosticCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parse => "SIFR-SQL-POSTGRESQL-0001",
            Self::UnknownRelation => "SIFR-SQL-POSTGRESQL-0002",
            Self::UnknownColumn => "SIFR-SQL-POSTGRESQL-0003",
            Self::AmbiguousColumn => "SIFR-SQL-POSTGRESQL-0004",
            Self::TypeMismatch => "SIFR-SQL-POSTGRESQL-0005",
            Self::UnknownFunction => "SIFR-SQL-POSTGRESQL-0006",
            Self::UnknownOperator => "SIFR-SQL-POSTGRESQL-0007",
            Self::InvalidParameter => "SIFR-SQL-POSTGRESQL-0008",
            Self::InvalidWrite => "SIFR-SQL-POSTGRESQL-0009",
            Self::InvalidResult => "SIFR-SQL-POSTGRESQL-0010",
            Self::UnsupportedCoreSyntax => "SIFR-SQL-POSTGRESQL-0011",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresSpanKind {
    Sifr,
    VirtualSql,
    Schema,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresDiagnosticSpan {
    pub kind: PostgresSpanKind,
    pub document: String,
    pub start: u32,
    pub end: u32,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresDiagnostic {
    pub code: PostgresDiagnosticCode,
    pub message: String,
    pub primary: PostgresDiagnosticSpan,
    pub related: Vec<PostgresDiagnosticSpan>,
}

impl PostgresDiagnostic {
    #[must_use]
    pub fn at_sql(
        code: PostgresDiagnosticCode,
        message: impl Into<String>,
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            primary: PostgresDiagnosticSpan {
                kind: PostgresSpanKind::VirtualSql,
                document: "sifr://sql/query".to_string(),
                start,
                end,
                label: "PostgreSQL query".to_string(),
            },
            related: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_sifr_span(mut self, document: impl Into<String>, start: u32, end: u32) -> Self {
        self.related.push(PostgresDiagnosticSpan {
            kind: PostgresSpanKind::Sifr,
            document: document.into(),
            start,
            end,
            label: "Sifr template source".to_string(),
        });
        self
    }

    #[must_use]
    pub fn with_schema_span(mut self, document: impl Into<String>, start: u32, end: u32) -> Self {
        self.related.push(PostgresDiagnosticSpan {
            kind: PostgresSpanKind::Schema,
            document: document.into(),
            start,
            end,
            label: "schema declaration".to_string(),
        });
        self
    }
}
