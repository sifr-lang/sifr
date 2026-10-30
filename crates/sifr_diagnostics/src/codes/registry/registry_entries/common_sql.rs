//! Provider-neutral SQL diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-SQL-0001",
        "SQL",
        "Database type has no common SQL mapping.",
        Severity::Error,
        "database type has no common SQL mapping",
        "sifr_sql_contract::sql_type",
        [],
        []
    ),
    active_entry!(
        "SIFR-SQL-0002",
        "SQL",
        "SQL parameter bind types are incompatible.",
        Severity::Error,
        "SQL parameter bind types are incompatible",
        "sifr_sql_contract::bind",
        [],
        []
    ),
    active_entry!(
        "SIFR-SQL-0003",
        "SQL",
        "Nullable value cannot bind to a non-null SQL parameter.",
        Severity::Error,
        "nullable value cannot bind to a non-null SQL parameter",
        "sifr_sql_contract::bind",
        [],
        []
    ),
    active_entry!(
        "SIFR-SQL-0004",
        "SQL",
        "SQL codec contract is invalid or missing.",
        Severity::Error,
        "SQL codec contract is invalid or missing",
        "sifr_sql_contract::codec",
        [],
        []
    ),
    active_entry!(
        "SIFR-SQL-0005",
        "SQL",
        "Execution method conflicts with query cardinality.",
        Severity::Error,
        "execution method conflicts with query cardinality",
        "sifr_sql_contract::cardinality",
        [],
        []
    ),
    active_entry!(
        "SIFR-SQL-0006",
        "SQL",
        "Query effect is not permitted by this SQL API.",
        Severity::Error,
        "query effect is not permitted by this SQL API",
        "sifr_sql_contract::effect",
        [],
        []
    ),
    active_entry!(
        "SIFR-SQL-0007",
        "SQL",
        "Provider analysis violates the common SQL contract.",
        Severity::Error,
        "provider analysis violates the common SQL contract",
        "sifr_sql_contract::provider",
        [],
        []
    ),
    active_entry!(
        "SIFR-SQL-0008",
        "SQL",
        "SQL handle ownership or lifetime is invalid.",
        Severity::Error,
        "SQL handle ownership or lifetime is invalid",
        "sifr_sql_runtime::handles",
        [],
        []
    ),
    active_entry!(
        "SIFR-SQL-0009",
        "SQL",
        "Configured SQL profile has no schema-profile import.",
        Severity::Error,
        "configured SQL profile has no schema-profile import",
        "sifr_driver::build::sql_profiles",
        [],
        []
    ),
];
