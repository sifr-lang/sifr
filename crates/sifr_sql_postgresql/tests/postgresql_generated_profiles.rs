#![allow(clippy::unwrap_used)]

use sifr_sql_contract::{
    DatabaseType, DecimalRepresentation, IntegerSign, IntegerWidth, Nullability,
};
use sifr_sql_postgresql::generated_sifr_type;

#[test]
fn generated_postgresql_types_use_the_closed_sifr_annotation_vocabulary() {
    let cases = [
        (DatabaseType::Date, "date"),
        (DatabaseType::LocalTime { precision: 6 }, "time"),
        (DatabaseType::OffsetTime { precision: 6 }, "OffsetTime"),
        (DatabaseType::LocalDateTime { precision: 6 }, "datetime"),
        (DatabaseType::Instant { precision: 6 }, "Instant"),
        (DatabaseType::Uuid, "UUID"),
        (DatabaseType::Json { binary: true }, "JsonValue"),
        (DatabaseType::IpAddress, "IPAddress"),
        (DatabaseType::IpNetwork, "IPNetwork"),
        (DatabaseType::MacAddress, "MacAddress"),
        (
            DatabaseType::Integer {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits64,
            },
            "int64",
        ),
        (
            DatabaseType::Decimal {
                precision: Some(12),
                scale: Some(2),
                representation: DecimalRepresentation::Decimal,
            },
            "decimal",
        ),
        (
            DatabaseType::Array {
                element: Box::new(DatabaseType::Uuid),
                dimensions: Some(1),
                element_nullability: Nullability::Nullable,
                preserves_lower_bounds: true,
            },
            "SqlArray[UUID | None]",
        ),
    ];
    for (database_type, expected) in cases {
        assert_eq!(generated_sifr_type(&database_type).unwrap(), expected);
    }
}

#[test]
fn claimed_postgresql_sql_values_keep_canonical_bind_contracts() {
    use sifr_sql_contract::{
        BindCompatibility, InputType, ParameterType, bind_compatibility, canonical_read_type,
    };
    let databases = [
        DatabaseType::Date,
        DatabaseType::LocalTime { precision: 6 },
        DatabaseType::OffsetTime { precision: 6 },
        DatabaseType::LocalDateTime { precision: 6 },
        DatabaseType::Instant { precision: 6 },
        DatabaseType::Uuid,
        DatabaseType::Json { binary: true },
        DatabaseType::IpAddress,
        DatabaseType::IpNetwork,
        DatabaseType::MacAddress,
    ];
    let registry = sifr_sql_postgresql::PostgresTypeRegistry::new(18);
    let codecs = registry
        .codec_registry_for(databases.iter())
        .expect("PostgreSQL codec registry");
    for database in databases {
        let value = canonical_read_type(&database).expect("canonical PostgreSQL value");
        let compatibility = bind_compatibility(
            &InputType {
                value,
                nullability: Nullability::NonNull,
            },
            &ParameterType {
                database: database.clone(),
                nullability: Nullability::NonNull,
            },
            &codecs,
        );
        assert!(
            !matches!(compatibility, BindCompatibility::Rejected(_)),
            "{database:?}: {compatibility:?}"
        );
    }
}
