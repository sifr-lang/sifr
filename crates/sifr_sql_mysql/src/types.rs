use crate::ast::MysqlTypeName;
use sifr_sql_contract::{DatabaseType, DecimalRepresentation, IntegerSign, IntegerWidth, ObjectId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MysqlServerSeries {
    pub major: u16,
    pub minor: u16,
}

impl MysqlServerSeries {
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    #[must_use]
    pub fn profile(self) -> String {
        format!("mysql-{}.{}", self.major, self.minor)
    }

    #[must_use]
    pub fn version(self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

pub const SUPPORTED_MYSQL_SERIES: [MysqlServerSeries; 3] = [
    MysqlServerSeries::new(8, 4),
    MysqlServerSeries::new(9, 7),
    MysqlServerSeries::new(26, 7),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MysqlType {
    pub database: DatabaseType,
    pub canonical_name: String,
    pub character_set: Option<String>,
    pub collation: Option<String>,
}

pub(crate) fn mysql_type(
    name: &MysqlTypeName,
    object_identity: &ObjectId,
) -> Result<MysqlType, String> {
    let canonical = name.name.to_ascii_lowercase();
    let sign = if name.unsigned {
        IntegerSign::Unsigned
    } else {
        IntegerSign::Signed
    };
    let parameter = |index: usize| {
        name.parameters
            .get(index)
            .and_then(|value| value.parse::<u32>().ok())
    };
    let database = match canonical.as_str() {
        "bool" | "boolean" => DatabaseType::Boolean,
        "tinyint" if name.parameters.first().is_some_and(|value| value == "1") => {
            DatabaseType::Boolean
        }
        "tinyint" => DatabaseType::Integer {
            sign,
            width: IntegerWidth::Bits8,
        },
        "smallint" => DatabaseType::Integer {
            sign,
            width: IntegerWidth::Bits16,
        },
        "mediumint" | "int" | "integer" => DatabaseType::Integer {
            sign,
            width: IntegerWidth::Bits32,
        },
        "bigint" => DatabaseType::Integer {
            sign,
            width: IntegerWidth::Bits64,
        },
        "decimal" | "numeric" => DatabaseType::Decimal {
            precision: parameter(0).and_then(|value| u16::try_from(value).ok()),
            scale: parameter(1).and_then(|value| i16::try_from(value).ok()),
            representation: DecimalRepresentation::Decimal,
        },
        "float" => DatabaseType::Float32,
        "double" | "real" => DatabaseType::Float64,
        "char" => DatabaseType::Text {
            fixed: true,
            max_characters: parameter(0),
        },
        "varchar" | "text" | "tinytext" | "mediumtext" | "longtext" => DatabaseType::Text {
            fixed: false,
            max_characters: parameter(0),
        },
        "binary" => DatabaseType::Binary {
            max_bytes: parameter(0).map(u64::from),
        },
        "varbinary" | "blob" | "tinyblob" | "mediumblob" | "longblob" => DatabaseType::Binary {
            max_bytes: parameter(0).map(u64::from),
        },
        "date" => DatabaseType::Date,
        "time" => DatabaseType::LocalTime {
            precision: precision(name)?,
        },
        "datetime" => DatabaseType::LocalDateTime {
            precision: precision(name)?,
        },
        "timestamp" => DatabaseType::Instant {
            precision: precision(name)?,
        },
        "json" => DatabaseType::Json { binary: true },
        "enum" | "set" => DatabaseType::Enum {
            identity: object_identity.clone(),
        },
        "year" => DatabaseType::Integer {
            sign: IntegerSign::Unsigned,
            width: IntegerWidth::Bits16,
        },
        _ => return Err(format!("unsupported MySQL type '{}'", name.name)),
    };
    Ok(MysqlType {
        database,
        canonical_name: canonical,
        character_set: None,
        collation: None,
    })
}

fn precision(name: &MysqlTypeName) -> Result<u8, String> {
    let value = name
        .parameters
        .first()
        .map_or(Ok(0), |value| value.parse::<u8>())
        .map_err(|_| format!("invalid precision for MySQL type '{}'", name.name))?;
    if value > 6 {
        return Err(format!(
            "MySQL temporal precision for '{}' exceeds 6",
            name.name
        ));
    }
    Ok(value)
}
