use crate::model::Severity;

use super::DiagnosticCode;

impl DiagnosticCode {
    pub const SQL_DATABASE_TYPE: Self = Self::new("SIFR-SQL-0001", Severity::Error);
    pub const SQL_BIND_COMPATIBILITY: Self = Self::new("SIFR-SQL-0002", Severity::Error);
    pub const SQL_NULLABILITY: Self = Self::new("SIFR-SQL-0003", Severity::Error);
    pub const SQL_CODEC_CONTRACT: Self = Self::new("SIFR-SQL-0004", Severity::Error);
    pub const SQL_CARDINALITY: Self = Self::new("SIFR-SQL-0005", Severity::Error);
    pub const SQL_EFFECT: Self = Self::new("SIFR-SQL-0006", Severity::Error);
    pub const SQL_PROVIDER_CONTRACT: Self = Self::new("SIFR-SQL-0007", Severity::Error);
    pub const SQL_OWNERSHIP: Self = Self::new("SIFR-SQL-0008", Severity::Error);
}
