use sifr_sql_contract::{
    CodecContract, CodecIdentity, CodecRegistry, DatabaseType, DecimalRepresentation, IntegerSign,
    IntegerWidth, NullCodecBehavior, PanicContainment, SchemaContractError, WireFormatIdentity,
    canonical_read_type,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresType {
    pub canonical_name: String,
    pub database_type: DatabaseType,
}

#[derive(Clone, Debug)]
pub struct PostgresTypeRegistry {
    server_profile: String,
    types: BTreeMap<String, PostgresType>,
}

impl PostgresTypeRegistry {
    #[must_use]
    pub fn new(server_major: u16) -> Self {
        let mut registry = Self {
            server_profile: format!("postgresql-{server_major}"),
            types: BTreeMap::new(),
        };
        for (names, ty) in builtin_types() {
            let canonical_name = names[0].to_string();
            for name in names {
                registry.types.insert(
                    normalize_name(name),
                    PostgresType {
                        canonical_name: canonical_name.clone(),
                        database_type: ty.clone(),
                    },
                );
            }
        }
        registry
    }

    #[must_use]
    pub fn server_profile(&self) -> &str {
        &self.server_profile
    }

    #[must_use]
    pub fn resolve(&self, name: &[String]) -> Option<&PostgresType> {
        self.types.get(&normalize_path(name))
    }

    pub fn add_nominal(&mut self, names: &[String], ty: DatabaseType) {
        let canonical_name = names.join(".");
        self.types.insert(
            normalize_path(names),
            PostgresType {
                canonical_name,
                database_type: ty,
            },
        );
    }

    pub fn codec_registry(&self) -> Result<CodecRegistry, SchemaContractError> {
        let mut contracts = BTreeMap::new();
        for ty in self.types.values() {
            contracts
                .entry(ty.database_type.clone())
                .or_insert_with(|| ty.clone());
        }
        CodecRegistry::for_profile(
            self.server_profile.clone(),
            contracts
                .into_iter()
                .map(|(database_type, ty)| codec_contract(&self.server_profile, &ty, database_type))
                .collect::<Result<Vec<_>, _>>()?,
        )
    }

    pub fn codec_identity(
        &self,
        database_type: &DatabaseType,
    ) -> Result<CodecIdentity, SchemaContractError> {
        self.codec_registry()?
            .codec_for_database_type(database_type)
            .map(|codec| codec.identity.clone())
            .ok_or_else(|| {
                SchemaContractError::new(
                    sifr_sql_contract::SchemaContractErrorKind::InvalidProvider,
                    "PostgreSQL type has no exact codec",
                )
            })
    }
}

fn codec_contract(
    server_profile: &str,
    ty: &PostgresType,
    database_type: DatabaseType,
) -> Result<CodecContract, SchemaContractError> {
    let identity = CodecIdentity::new(format!(
        "postgresql.{}.v1",
        ty.canonical_name.replace('.', "_")
    ))?;
    Ok(CodecContract {
        sifr_type: canonical_read_type(&database_type)?,
        database_type,
        identity: identity.clone(),
        server_profiles: BTreeSet::from([server_profile.to_string()]),
        encode_error: "sifr.sql.EncodeError".to_string(),
        decode_error: "sifr.sql.DecodeError".to_string(),
        null_behavior: NullCodecBehavior::PassThrough,
        wire_format: WireFormatIdentity::new(format!(
            "postgresql.binary.{}.v1",
            ty.canonical_name.replace('.', "_")
        ))?,
        panic_containment: PanicContainment::CatchAndRedact,
    })
}

fn normalize_path(name: &[String]) -> String {
    let last = name.last().map(String::as_str).unwrap_or_default();
    normalize_name(last)
}

fn normalize_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

#[allow(clippy::too_many_lines)]
fn builtin_types() -> Vec<(&'static [&'static str], DatabaseType)> {
    vec![
        (&["bool", "boolean"], DatabaseType::Boolean),
        (
            &["int2", "smallint", "pg_catalog.int2"],
            DatabaseType::Integer {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits16,
            },
        ),
        (
            &["int4", "integer", "int", "pg_catalog.int4"],
            DatabaseType::Integer {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits32,
            },
        ),
        (
            &["int8", "bigint", "pg_catalog.int8"],
            DatabaseType::Integer {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits64,
            },
        ),
        (
            &["numeric", "decimal", "pg_catalog.numeric"],
            DatabaseType::Decimal {
                precision: None,
                scale: None,
                representation: DecimalRepresentation::Numeric,
            },
        ),
        (&["float4", "real"], DatabaseType::Float32),
        (&["float8", "double precision"], DatabaseType::Float64),
        (
            &["text", "varchar", "character varying", "char", "bpchar"],
            DatabaseType::Text {
                fixed: false,
                max_characters: None,
            },
        ),
        (&["bytea"], DatabaseType::Binary { max_bytes: None }),
        (&["date"], DatabaseType::Date),
        (&["time"], DatabaseType::LocalTime { precision: 6 }),
        (&["timetz"], DatabaseType::OffsetTime { precision: 6 }),
        (&["timestamp"], DatabaseType::LocalDateTime { precision: 6 }),
        (&["timestamptz"], DatabaseType::Instant { precision: 6 }),
        (&["interval"], DatabaseType::CalendarInterval),
        (&["uuid"], DatabaseType::Uuid),
        (&["json"], DatabaseType::Json { binary: false }),
        (&["jsonb"], DatabaseType::Json { binary: true }),
        (&["inet"], DatabaseType::IpNetwork),
        (&["cidr"], DatabaseType::IpNetwork),
        (&["macaddr", "macaddr8"], DatabaseType::MacAddress),
    ]
}
