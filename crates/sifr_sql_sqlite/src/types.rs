use crate::ast::SqliteTypeName;
use serde::{Deserialize, Serialize};
use sifr_sql_contract::{DatabaseType, IntegerSign, IntegerWidth, ObjectId, SqliteStorageClass};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteServerSeries {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SqliteServerSeries {
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    #[must_use]
    pub fn profile(self) -> String {
        format!("sqlite-{}.{}.{}", self.major, self.minor, self.patch)
    }

    #[must_use]
    pub fn version(self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub const SUPPORTED_SQLITE_SERIES: [SqliteServerSeries; 1] = [SqliteServerSeries::new(3, 53, 2)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqliteAffinity {
    Integer,
    Real,
    Text,
    Blob,
    Numeric,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqliteType {
    pub database: DatabaseType,
    pub declared_name: String,
    pub affinity: SqliteAffinity,
    pub strict: bool,
}

pub(crate) fn sqlite_type(
    name: &SqliteTypeName,
    _object_identity: &ObjectId,
    strict: bool,
) -> Result<SqliteType, String> {
    let declared = name.name.trim().to_ascii_lowercase();
    if declared.is_empty() {
        return Ok(SqliteType {
            database: DatabaseType::Binary { max_bytes: None },
            declared_name: declared,
            affinity: SqliteAffinity::Blob,
            strict,
        });
    }
    let affinity = affinity(&declared);
    if strict
        && !matches!(
            declared.as_str(),
            "any" | "blob" | "integer" | "int" | "real" | "text"
        )
    {
        return Err(format!(
            "SQLite STRICT table type '{}' is not one of ANY, BLOB, INT, INTEGER, REAL, or TEXT",
            name.name
        ));
    }
    let database = match (strict, declared.as_str(), affinity) {
        (true, "any", _) => dynamic_type(),
        (_, _, SqliteAffinity::Integer) => DatabaseType::Integer {
            sign: IntegerSign::Signed,
            width: IntegerWidth::Bits64,
        },
        (_, _, SqliteAffinity::Real) => DatabaseType::Float64,
        (_, _, SqliteAffinity::Text) => DatabaseType::Text {
            fixed: false,
            max_characters: None,
        },
        (_, _, SqliteAffinity::Blob) => DatabaseType::Binary { max_bytes: None },
        (_, _, SqliteAffinity::Numeric) => dynamic_type(),
    };
    Ok(SqliteType {
        database,
        declared_name: declared,
        affinity,
        strict,
    })
}

fn dynamic_type() -> DatabaseType {
    DatabaseType::SqliteDynamic {
        storage_classes: BTreeSet::from([
            SqliteStorageClass::Integer,
            SqliteStorageClass::Real,
            SqliteStorageClass::Text,
            SqliteStorageClass::Blob,
            SqliteStorageClass::Null,
        ]),
    }
}

#[must_use]
pub fn affinity(declared_type: &str) -> SqliteAffinity {
    let upper = declared_type.to_ascii_uppercase();
    if upper.contains("INT") {
        SqliteAffinity::Integer
    } else if upper.contains("CHAR") || upper.contains("CLOB") || upper.contains("TEXT") {
        SqliteAffinity::Text
    } else if upper.contains("BLOB") || upper.is_empty() {
        SqliteAffinity::Blob
    } else if upper.contains("REAL") || upper.contains("FLOA") || upper.contains("DOUB") {
        SqliteAffinity::Real
    } else {
        SqliteAffinity::Numeric
    }
}
