use crate::types::SqliteServerSeries;
use sifr_sql_contract::{
    CodecContract, CodecIdentity, CodecRegistry, DatabaseType, DecimalRepresentation, IntegerSign,
    IntegerWidth, NullCodecBehavior, PanicContainment, SchemaContractError, SifrType,
    WireFormatIdentity, canonical_read_type,
};
use std::collections::BTreeSet;

pub fn sqlite_codec_registry(
    series: SqliteServerSeries,
) -> Result<CodecRegistry, SchemaContractError> {
    let profile = series.profile();
    let types = [
        ("bool", DatabaseType::Boolean, SifrType::Bool),
        integer("i8", IntegerSign::Signed, IntegerWidth::Bits8),
        integer("u8", IntegerSign::Unsigned, IntegerWidth::Bits8),
        integer("i16", IntegerSign::Signed, IntegerWidth::Bits16),
        integer("u16", IntegerSign::Unsigned, IntegerWidth::Bits16),
        integer("i32", IntegerSign::Signed, IntegerWidth::Bits32),
        integer("u32", IntegerSign::Unsigned, IntegerWidth::Bits32),
        integer("i64", IntegerSign::Signed, IntegerWidth::Bits64),
        integer("u64", IntegerSign::Unsigned, IntegerWidth::Bits64),
        (
            "decimal",
            DatabaseType::Decimal {
                precision: None,
                scale: None,
                representation: DecimalRepresentation::Decimal,
            },
            SifrType::Decimal,
        ),
        ("f32", DatabaseType::Float32, SifrType::Float),
        ("f64", DatabaseType::Float64, SifrType::Float),
        (
            "text",
            DatabaseType::Text {
                fixed: false,
                max_characters: None,
            },
            SifrType::Str,
        ),
        (
            "bytes",
            DatabaseType::Binary { max_bytes: None },
            SifrType::Bytes,
        ),
        ("date", DatabaseType::Date, SifrType::Date),
        (
            "time",
            DatabaseType::LocalTime { precision: 0 },
            SifrType::LocalTime,
        ),
        (
            "datetime",
            DatabaseType::LocalDateTime { precision: 0 },
            SifrType::LocalDateTime,
        ),
        (
            "timestamp",
            DatabaseType::Instant { precision: 0 },
            SifrType::Instant,
        ),
        (
            "json",
            DatabaseType::Json { binary: true },
            SifrType::JsonValue,
        ),
    ];
    let mut contracts = Vec::with_capacity(types.len());
    for (name, database_type, sifr_type) in types {
        contracts.push(contract(&profile, name, database_type, sifr_type)?);
    }
    CodecRegistry::for_profile(profile, contracts)
}

pub(crate) fn sqlite_codec_registry_for_types(
    series: SqliteServerSeries,
    database_types: impl IntoIterator<Item = DatabaseType>,
) -> Result<CodecRegistry, SchemaContractError> {
    let profile = series.profile();
    let database_types = database_types.into_iter().collect::<BTreeSet<_>>();
    let mut contracts = Vec::with_capacity(database_types.len());
    for (index, database_type) in database_types.into_iter().enumerate() {
        let sifr_type = canonical_read_type(&database_type)?;
        contracts.push(contract(
            &profile,
            &format!("type_{index}"),
            database_type,
            sifr_type,
        )?);
    }
    CodecRegistry::for_profile(profile, contracts)
}

fn contract(
    profile: &str,
    name: &str,
    database_type: DatabaseType,
    sifr_type: SifrType,
) -> Result<CodecContract, SchemaContractError> {
    Ok(CodecContract {
        identity: CodecIdentity::new(format!("sqlite.{name}.binary.v1"))?,
        database_type,
        sifr_type,
        server_profiles: BTreeSet::from([profile.to_string()]),
        encode_error: "SqliteEncodeError".to_string(),
        decode_error: "SqliteDecodeError".to_string(),
        null_behavior: NullCodecBehavior::PassThrough,
        wire_format: WireFormatIdentity::new(format!("sqlite.binary.{name}.v1"))?,
        panic_containment: PanicContainment::CatchAndRedact,
    })
}

fn integer(
    name: &'static str,
    sign: IntegerSign,
    width: IntegerWidth,
) -> (&'static str, DatabaseType, SifrType) {
    (
        name,
        DatabaseType::Integer { sign, width },
        SifrType::FixedInteger { sign, width },
    )
}
