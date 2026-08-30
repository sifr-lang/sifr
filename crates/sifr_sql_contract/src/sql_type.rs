use crate::{CodecIdentity, CodecRegistry, ObjectId, SchemaContractError, SchemaContractErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegerWidth {
    Bits8,
    Bits16,
    Bits32,
    Bits64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegerSign {
    Signed,
    Unsigned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecimalRepresentation {
    Decimal,
    BigDecimal,
    Numeric,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nullability {
    NonNull,
    Nullable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqliteStorageClass {
    Integer,
    Real,
    Text,
    Blob,
    Null,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DatabaseType {
    Boolean,
    Integer {
        sign: IntegerSign,
        width: IntegerWidth,
    },
    Decimal {
        precision: Option<u16>,
        scale: Option<i16>,
        representation: DecimalRepresentation,
    },
    Float32,
    Float64,
    Text {
        fixed: bool,
        max_characters: Option<u32>,
    },
    Binary {
        max_bytes: Option<u64>,
    },
    Date,
    LocalTime {
        precision: u8,
    },
    OffsetTime {
        precision: u8,
    },
    LocalDateTime {
        precision: u8,
    },
    Instant {
        precision: u8,
    },
    CalendarInterval,
    Uuid,
    Json {
        binary: bool,
    },
    Array {
        element: Box<DatabaseType>,
        dimensions: Option<u8>,
        element_nullability: Nullability,
        preserves_lower_bounds: bool,
    },
    Enum {
        identity: ObjectId,
    },
    Domain {
        identity: ObjectId,
        base: Box<DatabaseType>,
    },
    Composite {
        identity: ObjectId,
    },
    Range {
        element: Box<DatabaseType>,
        multirange: bool,
    },
    IpAddress,
    IpNetwork,
    MacAddress,
    Named {
        identity: ObjectId,
        parameters: Vec<i64>,
        canonical: Box<DatabaseType>,
    },
    Custom {
        identity: ObjectId,
        codec: CodecIdentity,
    },
    SqliteDynamic {
        storage_classes: BTreeSet<SqliteStorageClass>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SifrType {
    Bool,
    FixedInteger {
        sign: IntegerSign,
        width: IntegerWidth,
    },
    ExactInteger,
    Decimal,
    BigDecimal,
    Numeric,
    Float,
    Str,
    Bytes,
    Date,
    LocalTime,
    OffsetTime,
    LocalDateTime,
    Instant,
    CalendarInterval,
    Uuid,
    JsonValue,
    None,
    List {
        element: Box<SifrType>,
    },
    SqlArray {
        element: Box<SifrType>,
    },
    Nominal {
        identity: ObjectId,
    },
    Range {
        element: Box<SifrType>,
        multirange: bool,
    },
    IpAddress,
    IpNetwork,
    MacAddress,
    Custom {
        identity: String,
    },
    Union {
        members: BTreeSet<SifrType>,
    },
}

pub fn canonical_read_type(database: &DatabaseType) -> Result<SifrType, SchemaContractError> {
    canonical_read_type_with_registry(database, None)
}

fn canonical_read_type_with_registry(
    database: &DatabaseType,
    codecs: Option<&CodecRegistry>,
) -> Result<SifrType, SchemaContractError> {
    validate_database_type(database)?;
    let mapped = match database {
        DatabaseType::Boolean => SifrType::Bool,
        DatabaseType::Integer { sign, width } => SifrType::FixedInteger {
            sign: *sign,
            width: *width,
        },
        DatabaseType::Decimal { representation, .. } => match representation {
            DecimalRepresentation::Decimal => SifrType::Decimal,
            DecimalRepresentation::BigDecimal => SifrType::BigDecimal,
            DecimalRepresentation::Numeric => SifrType::Numeric,
        },
        DatabaseType::Float32 | DatabaseType::Float64 => SifrType::Float,
        DatabaseType::Text { .. } => SifrType::Str,
        DatabaseType::Binary { .. } => SifrType::Bytes,
        DatabaseType::Date => SifrType::Date,
        DatabaseType::LocalTime { .. } => SifrType::LocalTime,
        DatabaseType::OffsetTime { .. } => SifrType::OffsetTime,
        DatabaseType::LocalDateTime { .. } => SifrType::LocalDateTime,
        DatabaseType::Instant { .. } => SifrType::Instant,
        DatabaseType::CalendarInterval => SifrType::CalendarInterval,
        DatabaseType::Uuid => SifrType::Uuid,
        DatabaseType::Json { .. } => SifrType::JsonValue,
        DatabaseType::Array {
            element,
            element_nullability,
            ..
        } => SifrType::SqlArray {
            element: Box::new(with_nullability(
                canonical_read_type_with_registry(element, codecs)?,
                *element_nullability,
            )),
        },
        DatabaseType::Enum { identity }
        | DatabaseType::Composite { identity }
        | DatabaseType::Domain { identity, .. } => SifrType::Nominal {
            identity: identity.clone(),
        },
        DatabaseType::Range {
            element,
            multirange,
        } => SifrType::Range {
            element: Box::new(canonical_read_type_with_registry(element, codecs)?),
            multirange: *multirange,
        },
        DatabaseType::IpAddress => SifrType::IpAddress,
        DatabaseType::IpNetwork => SifrType::IpNetwork,
        DatabaseType::MacAddress => SifrType::MacAddress,
        DatabaseType::Named { canonical, .. } => {
            canonical_read_type_with_registry(canonical, codecs)?
        }
        DatabaseType::Custom { .. } => {
            let Some(codecs) = codecs else {
                return Err(SchemaContractError::new(
                    SchemaContractErrorKind::InvalidProvider,
                    "custom database type requires a profile codec registry",
                ));
            };
            codecs
                .codec_for_database_type(database)
                .map(|contract| contract.sifr_type.clone())
                .ok_or_else(|| {
                    SchemaContractError::new(
                        SchemaContractErrorKind::InvalidProvider,
                        "custom database type has no codec in the selected server profile",
                    )
                })?
        }
        DatabaseType::SqliteDynamic { storage_classes } => {
            if storage_classes.is_empty() {
                return Err(SchemaContractError::new(
                    SchemaContractErrorKind::InvalidSchema,
                    "SQLite dynamic type must permit at least one storage class",
                ));
            }
            SifrType::Union {
                members: storage_classes
                    .iter()
                    .copied()
                    .map(sqlite_storage_type)
                    .collect(),
            }
        }
    };
    Ok(mapped)
}

pub fn canonical_read_type_in(
    database: &DatabaseType,
    codecs: &CodecRegistry,
) -> Result<SifrType, SchemaContractError> {
    canonical_read_type_with_registry(database, Some(codecs))
}

pub fn canonical_read_type_with_nullability(
    database: &DatabaseType,
    nullability: Nullability,
) -> Result<SifrType, SchemaContractError> {
    Ok(with_nullability(
        canonical_read_type(database)?,
        nullability,
    ))
}

pub fn canonical_read_type_with_nullability_in(
    database: &DatabaseType,
    nullability: Nullability,
    codecs: &CodecRegistry,
) -> Result<SifrType, SchemaContractError> {
    Ok(with_nullability(
        canonical_read_type_in(database, codecs)?,
        nullability,
    ))
}

fn with_nullability(value: SifrType, nullability: Nullability) -> SifrType {
    if nullability == Nullability::NonNull || value == SifrType::None {
        return value;
    }
    match value {
        SifrType::Union { mut members } => {
            members.insert(SifrType::None);
            SifrType::Union { members }
        }
        value => SifrType::Union {
            members: BTreeSet::from([value, SifrType::None]),
        },
    }
}

fn validate_database_type(database: &DatabaseType) -> Result<(), SchemaContractError> {
    match database {
        DatabaseType::Decimal {
            precision: Some(0), ..
        } => return Err(invalid("decimal precision must be positive")),
        DatabaseType::Text {
            max_characters: Some(0),
            ..
        } => return Err(invalid("text length must be positive")),
        DatabaseType::LocalTime { precision }
        | DatabaseType::OffsetTime { precision }
        | DatabaseType::LocalDateTime { precision }
        | DatabaseType::Instant { precision }
            if *precision > 9 =>
        {
            return Err(invalid("temporal precision cannot exceed nanoseconds"));
        }
        DatabaseType::Array {
            element,
            dimensions,
            ..
        } => {
            if dimensions == &Some(0) {
                return Err(invalid("SQL array dimensions must be positive"));
            }
            validate_database_type(element)?;
        }
        DatabaseType::Domain { base, .. } => validate_database_type(base)?,
        DatabaseType::Named {
            identity,
            canonical,
            ..
        } => {
            if identity.as_str().trim().is_empty() {
                return Err(invalid("named database type identity cannot be empty"));
            }
            validate_database_type(canonical)?;
        }
        DatabaseType::Range { element, .. } => validate_database_type(element)?,
        DatabaseType::SqliteDynamic { storage_classes } if storage_classes.is_empty() => {
            return Err(invalid(
                "SQLite dynamic type must permit at least one storage class",
            ));
        }
        _ => {}
    }
    Ok(())
}

fn sqlite_storage_type(storage: SqliteStorageClass) -> SifrType {
    match storage {
        SqliteStorageClass::Integer => SifrType::FixedInteger {
            sign: IntegerSign::Signed,
            width: IntegerWidth::Bits64,
        },
        SqliteStorageClass::Real => SifrType::Float,
        SqliteStorageClass::Text => SifrType::Str,
        SqliteStorageClass::Blob => SifrType::Bytes,
        SqliteStorageClass::Null => SifrType::None,
    }
}

fn invalid(message: impl Into<String>) -> SchemaContractError {
    SchemaContractError::new(SchemaContractErrorKind::InvalidSchema, message)
}
