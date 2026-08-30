use crate::ast::PostgresTypeName;
use sha2::{Digest, Sha256};
use sifr_sql_contract::{
    CodecContract, CodecIdentity, CodecRegistry, DatabaseType, DecimalRepresentation, IntegerSign,
    IntegerWidth, NullCodecBehavior, PanicContainment, SchemaContractError, SifrType,
    WireFormatIdentity, canonical_read_type,
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
    pub fn resolve(&self, name: &PostgresTypeName) -> Option<PostgresType> {
        let base = self.resolve_path(&name.path)?;
        apply_modifiers(base, &name.modifiers)
    }

    #[must_use]
    pub fn resolve_path(&self, name: &[String]) -> Option<PostgresType> {
        let key = normalize_path(name);
        if name.len() > 1 {
            return self.types.get(&key).cloned();
        }
        if let Some(builtin) = self.types.get(&key) {
            return Some(builtin.clone());
        }
        if let Some(public) = self.types.get(&format!("public.{key}")) {
            return Some(public.clone());
        }
        let suffix = format!(".{key}");
        let mut matches = self
            .types
            .iter()
            .filter(|(name, _)| name.ends_with(&suffix))
            .map(|(_, ty)| ty.clone());
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }

    pub fn add_nominal(&mut self, names: &[String], ty: DatabaseType) {
        let path = normalize_path(names);
        let canonical_name = if names.len() == 1 {
            format!("public.{path}")
        } else {
            path.clone()
        };
        let value = PostgresType {
            canonical_name,
            database_type: ty,
        };
        self.types.insert(path.clone(), value.clone());
        if names.len() == 1 {
            self.types.insert(format!("public.{path}"), value);
        }
    }

    pub fn codec_registry(&self) -> Result<CodecRegistry, SchemaContractError> {
        self.codec_registry_for(self.types.values().map(|ty| &ty.database_type))
    }

    pub fn codec_registry_for<'a>(
        &self,
        database_types: impl IntoIterator<Item = &'a DatabaseType>,
    ) -> Result<CodecRegistry, SchemaContractError> {
        let contracts = database_types.into_iter().cloned().collect::<BTreeSet<_>>();
        CodecRegistry::for_profile(
            self.server_profile.clone(),
            contracts
                .into_iter()
                .map(|database_type| codec_contract(&self.server_profile, database_type))
                .collect::<Result<Vec<_>, _>>()?,
        )
    }

    pub fn codec_identity(
        &self,
        database_type: &DatabaseType,
    ) -> Result<CodecIdentity, SchemaContractError> {
        Ok(codec_contract(&self.server_profile, database_type.clone())?.identity)
    }
}

fn codec_contract(
    server_profile: &str,
    database_type: DatabaseType,
) -> Result<CodecContract, SchemaContractError> {
    let family = codec_family(&database_type);
    let serialized = serde_json::to_vec(&database_type).map_err(|_| {
        SchemaContractError::new(
            sifr_sql_contract::SchemaContractErrorKind::InvalidProvider,
            "PostgreSQL database type cannot form a codec identity",
        )
    })?;
    let digest = Sha256::digest(serialized);
    let suffix = hex_bytes(&digest[..8]);
    let identity = match &database_type {
        DatabaseType::Custom { codec, .. } => codec.clone(),
        _ => CodecIdentity::new(format!("postgresql.{family}.{suffix}.v1"))?,
    };
    let sifr_type = match &database_type {
        DatabaseType::Custom { identity, .. } if identity.as_str() == "pg_catalog.macaddr8" => {
            SifrType::MacAddress
        }
        _ => canonical_read_type(&database_type)?,
    };
    Ok(CodecContract {
        sifr_type,
        database_type,
        identity: identity.clone(),
        server_profiles: BTreeSet::from([server_profile.to_string()]),
        encode_error: "sifr.sql.EncodeError".to_string(),
        decode_error: "sifr.sql.DecodeError".to_string(),
        null_behavior: NullCodecBehavior::PassThrough,
        wire_format: WireFormatIdentity::new(format!("postgresql.binary.{family}.v1"))?,
        panic_containment: PanicContainment::CatchAndRedact,
    })
}

fn hex_bytes(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn normalize_path(name: &[String]) -> String {
    name.iter()
        .map(|segment| normalize_name(segment))
        .collect::<Vec<_>>()
        .join(".")
}

fn normalize_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn apply_modifiers(mut ty: PostgresType, modifiers: &[i64]) -> Option<PostgresType> {
    let base = ty.canonical_name.rsplit('.').next().unwrap_or_default();
    ty.database_type = match (base, modifiers) {
        ("varchar" | "character varying", []) => named_type(
            "pg_catalog.varchar",
            modifiers,
            DatabaseType::Text {
                fixed: false,
                max_characters: None,
            },
        ),
        ("varchar" | "character varying", [length]) => named_type(
            "pg_catalog.varchar",
            modifiers,
            DatabaseType::Text {
                fixed: false,
                max_characters: Some(u32::try_from(*length).ok()?.max(1)),
            },
        ),
        ("char" | "bpchar" | "character", []) => named_type(
            "pg_catalog.bpchar",
            modifiers,
            DatabaseType::Text {
                fixed: true,
                max_characters: Some(1),
            },
        ),
        ("char" | "bpchar" | "character", [length]) => named_type(
            "pg_catalog.bpchar",
            modifiers,
            DatabaseType::Text {
                fixed: true,
                max_characters: Some(u32::try_from(*length).ok()?.max(1)),
            },
        ),
        ("numeric" | "decimal", []) => ty.database_type,
        ("numeric" | "decimal", [precision]) => DatabaseType::Decimal {
            precision: Some(u16::try_from(*precision).ok()?),
            scale: Some(0),
            representation: DecimalRepresentation::Numeric,
        },
        ("numeric" | "decimal", [precision, scale]) => DatabaseType::Decimal {
            precision: Some(u16::try_from(*precision).ok()?),
            scale: Some(i16::try_from(*scale).ok()?),
            representation: DecimalRepresentation::Numeric,
        },
        ("time", []) => DatabaseType::LocalTime { precision: 6 },
        ("time", [precision]) => DatabaseType::LocalTime {
            precision: time_precision(*precision)?,
        },
        ("timetz", []) => DatabaseType::OffsetTime { precision: 6 },
        ("timetz", [precision]) => DatabaseType::OffsetTime {
            precision: time_precision(*precision)?,
        },
        ("timestamp", []) => DatabaseType::LocalDateTime { precision: 6 },
        ("timestamp", [precision]) => DatabaseType::LocalDateTime {
            precision: time_precision(*precision)?,
        },
        ("timestamptz", []) => DatabaseType::Instant { precision: 6 },
        ("timestamptz", [precision]) => DatabaseType::Instant {
            precision: time_precision(*precision)?,
        },
        (_, []) => ty.database_type,
        _ => return None,
    };
    if !modifiers.is_empty() {
        ty.canonical_name = format!(
            "{}({})",
            ty.canonical_name,
            modifiers
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    Some(ty)
}

fn time_precision(value: i64) -> Option<u8> {
    let value = u8::try_from(value).ok()?;
    (value <= 6).then_some(value)
}

fn codec_family(database_type: &DatabaseType) -> &'static str {
    match database_type {
        DatabaseType::Boolean => "bool",
        DatabaseType::Integer { .. } => "int",
        DatabaseType::Decimal { .. } => "numeric",
        DatabaseType::Float32 => "float4",
        DatabaseType::Float64 => "float8",
        DatabaseType::Text { fixed: true, .. } => "bpchar",
        DatabaseType::Text { fixed: false, .. } => "text",
        DatabaseType::Binary { .. } => "bytea",
        DatabaseType::Date => "date",
        DatabaseType::LocalTime { .. } => "time",
        DatabaseType::OffsetTime { .. } => "timetz",
        DatabaseType::LocalDateTime { .. } => "timestamp",
        DatabaseType::Instant { .. } => "timestamptz",
        DatabaseType::CalendarInterval => "interval",
        DatabaseType::Uuid => "uuid",
        DatabaseType::Json { binary: false } => "json",
        DatabaseType::Json { binary: true } => "jsonb",
        DatabaseType::IpAddress => "inet",
        DatabaseType::IpNetwork => "cidr",
        DatabaseType::MacAddress => "macaddr",
        DatabaseType::Named { identity, .. } if identity.as_str() == "pg_catalog.varchar" => {
            "varchar"
        }
        DatabaseType::Named { identity, .. } if identity.as_str() == "pg_catalog.bpchar" => {
            "bpchar"
        }
        DatabaseType::Named { .. } => "named",
        DatabaseType::Custom { .. } => "custom",
        DatabaseType::Array { .. } => "array",
        DatabaseType::Enum { .. } => "enum",
        DatabaseType::Domain { .. } => "domain",
        DatabaseType::Composite { .. } => "composite",
        DatabaseType::Range { .. } => "range",
        DatabaseType::SqliteDynamic { .. } => "unsupported",
    }
}

fn named_type(identity: &str, parameters: &[i64], canonical: DatabaseType) -> DatabaseType {
    DatabaseType::Named {
        identity: sifr_sql_contract::ObjectId::new(identity),
        parameters: parameters.to_vec(),
        canonical: Box::new(canonical),
    }
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
            &["text", "pg_catalog.text"],
            DatabaseType::Text {
                fixed: false,
                max_characters: None,
            },
        ),
        (
            &["varchar", "character varying", "pg_catalog.varchar"],
            DatabaseType::Text {
                fixed: false,
                max_characters: None,
            },
        ),
        (
            &["char", "bpchar", "character", "pg_catalog.bpchar"],
            DatabaseType::Text {
                fixed: true,
                max_characters: Some(1),
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
        (&["inet", "pg_catalog.inet"], DatabaseType::IpAddress),
        (&["cidr"], DatabaseType::IpNetwork),
        (&["macaddr"], DatabaseType::MacAddress),
    ]
}
