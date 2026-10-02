//! Native application boundary for compiler-approved SQLite queries.
use crate::{
    ExecutionOptions, SqliteEvidence, SqlitePool, SqliteProfile, VerificationProbe, Verified,
    open_pool,
};
use serde::Deserialize;
use sifr_sql_runtime::SqlErrorKind;
use sifr_sql_runtime::{
    BoundParameters, OwnedParameter, RuntimeCardinality, RuntimeCodecIdentity, RuntimeEffect,
    RuntimeEffectContract, RuntimeLimits, SchemaDependencySlice, SchemaStrictness,
};
pub use sifr_sql_runtime::{OwnedSqlValue, SqlError};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    statement: String,
    profile: String,
    schema: String,
    minimum: u64,
    maximum: Option<u64>,
    effect: String,
    referenced: Vec<String>,
    affected: Vec<String>,
    parameter_types: Vec<String>,
    result_types: serde_json::Value,
}

pub struct BoundQuery<Row> {
    descriptor: String,
    values: Vec<OwnedSqlValue>,
    decoder: fn(&[OwnedSqlValue]) -> Result<Row, SqlError>,
}
impl<Row> Clone for BoundQuery<Row> {
    fn clone(&self) -> Self {
        Self {
            descriptor: self.descriptor.clone(),
            values: self.values.clone(),
            decoder: self.decoder,
        }
    }
}
impl<Row> BoundQuery<Row> {
    #[must_use]
    pub fn new(
        descriptor: String,
        values: Vec<OwnedSqlValue>,
        decoder: fn(&[OwnedSqlValue]) -> Result<Row, SqlError>,
    ) -> Self {
        Self {
            descriptor,
            values,
            decoder,
        }
    }
}
#[derive(Clone)]
pub struct VerifiedPool {
    pool: SqlitePool<Verified>,
}

pub async fn connect(
    path: String,
    profile: String,
    schema: String,
) -> Result<VerifiedPool, SqlError> {
    let expected = SchemaDependencySlice::new(
        schema.clone(),
        [sifr_sql_runtime::SchemaProperty::new(
            "catalog.empty",
            Some("true".into()),
        )?],
    )?;
    // This initial application boundary accepts only the compiler's empty catalog.
    // Every acquired worker observes the actual SQLite catalog before verification.
    let observation = format!(
        "SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*') THEN '{schema}' ELSE 'catalog-drift' END"
    );
    let selected = SqliteProfile::new(
        path,
        profile,
        expected,
        SqliteEvidence::Introspection {
            fingerprint_statement: observation,
            probes: vec![VerificationProbe::new(
                "catalog.empty",
                "SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*') THEN 'true' ELSE 'false' END",
            )?],
        },
        SchemaStrictness::Exact,
        BTreeMap::new(),
        Vec::new(),
        (3, 53, 4),
        RuntimeLimits::default(),
        250,
    )?;
    Ok(VerifiedPool {
        pool: open_pool(selected)?.verify_schema().await?,
    })
}

impl VerifiedPool {
    fn request<Row>(
        &self,
        query: &BoundQuery<Row>,
        mode: sifr_sql_runtime::ExecutionMode,
    ) -> Result<sifr_sql_runtime::ExecutionRequest<SqliteProfile>, SqlError> {
        let descriptor: Descriptor =
            serde_json::from_str(&query.descriptor).map_err(|_| error(SqlErrorKind::Provider))?;
        let profile = self.pool.profile();
        if descriptor.profile != profile.profile_fingerprint()
            || descriptor.schema != profile.schema_fingerprint()
        {
            return Err(error(SqlErrorKind::SchemaContract));
        }
        let effect = match descriptor.effect.as_str() {
            "Read" => RuntimeEffect::Read,
            _ => return Err(error(SqlErrorKind::Provider)),
        };
        let values = query
            .values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                Ok(OwnedParameter {
                    slot: u32::try_from(index).map_err(|_| error(SqlErrorKind::Encode))?,
                    codec: RuntimeCodecIdentity::new("sifr.sql.native-scalar")
                        .map_err(|_| error(SqlErrorKind::Encode))?,
                    value: value.clone(),
                })
            })
            .collect::<Result<Vec<_>, SqlError>>()?;
        Ok(sifr_sql_runtime::ExecutionRequest {
            profile,
            statement: Arc::from(descriptor.statement.clone()),
            parameters: BoundParameters::new(values).map_err(|_| error(SqlErrorKind::Encode))?,
            mode,
            cardinality: RuntimeCardinality::new(descriptor.minimum, descriptor.maximum)?,
            effects: RuntimeEffectContract::new(
                effect,
                descriptor.referenced,
                descriptor.affected,
            )?,
            returns_rows: true,
            metadata: sifr_sql_runtime::ExecutionMetadata {
                normalized_statement_fingerprint: fingerprint(descriptor.statement.as_bytes()),
                parameter_type_fingerprint: fingerprint(
                    &serde_json::to_vec(&descriptor.parameter_types)
                        .map_err(|_| error(SqlErrorKind::Provider))?,
                ),
                result_type_fingerprint: fingerprint(
                    &serde_json::to_vec(&descriptor.result_types)
                        .map_err(|_| error(SqlErrorKind::Provider))?,
                ),
                schema_fingerprint: descriptor.schema,
            },
        })
    }
    pub async fn fetch_one<Row>(&self, query: BoundQuery<Row>) -> Result<Row, SqlError> {
        let request = self.request(&query, sifr_sql_runtime::ExecutionMode::FetchOne)?;
        let row = self
            .pool
            .fetch_one(request, ExecutionOptions::default())
            .await?;
        (query.decoder)(row.values())
    }
    pub async fn fetch_optional<Row>(
        &self,
        query: BoundQuery<Row>,
    ) -> Result<Option<Row>, SqlError> {
        let request = self.request(&query, sifr_sql_runtime::ExecutionMode::FetchOptional)?;
        self.pool
            .fetch_optional(request, ExecutionOptions::default())
            .await?
            .map(|row| (query.decoder)(row.values()))
            .transpose()
    }
    pub async fn fetch_all<Row>(
        &self,
        query: BoundQuery<Row>,
        maximum_rows: u64,
    ) -> Result<Vec<Row>, SqlError> {
        let request = self.request(
            &query,
            sifr_sql_runtime::ExecutionMode::FetchAll { maximum_rows },
        )?;
        self.pool
            .fetch_all(request, ExecutionOptions::default())
            .await?
            .iter()
            .map(|row| (query.decoder)(row.values()))
            .collect()
    }
}
fn error(kind: SqlErrorKind) -> SqlError {
    SqlError::new(kind)
}

pub trait Scalar: Sized {
    fn encode(self) -> OwnedSqlValue;
    fn decode(value: &OwnedSqlValue) -> Result<Self, SqlError>;
}
pub fn encode<T: Scalar>(value: T) -> OwnedSqlValue {
    value.encode()
}
pub fn decode<T: Scalar>(values: &[OwnedSqlValue], index: usize) -> Result<T, SqlError> {
    T::decode(
        values
            .get(index)
            .ok_or_else(|| error(SqlErrorKind::Decode))?,
    )
}
macro_rules! scalar {
    ($ty:ty, $variant:ident) => {
        impl Scalar for $ty {
            fn encode(self) -> OwnedSqlValue {
                OwnedSqlValue::$variant(self)
            }
            fn decode(value: &OwnedSqlValue) -> Result<Self, SqlError> {
                match value {
                    OwnedSqlValue::$variant(value) => Ok(value.clone()),
                    _ => Err(error(SqlErrorKind::Decode)),
                }
            }
        }
    };
}
scalar!(i64, Signed);
scalar!(f64, Float);
scalar!(String, Text);
impl Scalar for bool {
    fn encode(self) -> OwnedSqlValue {
        OwnedSqlValue::Bool(self)
    }
    fn decode(value: &OwnedSqlValue) -> Result<Self, SqlError> {
        match value {
            OwnedSqlValue::Bool(value) => Ok(*value),
            OwnedSqlValue::Signed(0) => Ok(false),
            OwnedSqlValue::Signed(1) => Ok(true),
            _ => Err(error(SqlErrorKind::Decode)),
        }
    }
}
impl Scalar for Vec<u8> {
    fn encode(self) -> OwnedSqlValue {
        OwnedSqlValue::Bytes(self.into())
    }
    fn decode(value: &OwnedSqlValue) -> Result<Self, SqlError> {
        match value {
            OwnedSqlValue::Bytes(value) => Ok(value.to_vec()),
            _ => Err(error(SqlErrorKind::Decode)),
        }
    }
}
impl<T: Scalar> Scalar for Option<T> {
    fn encode(self) -> OwnedSqlValue {
        self.map_or(OwnedSqlValue::Null, Scalar::encode)
    }
    fn decode(value: &OwnedSqlValue) -> Result<Self, SqlError> {
        if matches!(value, OwnedSqlValue::Null) {
            Ok(None)
        } else {
            T::decode(value).map(Some)
        }
    }
}

fn fingerprint(bytes: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut hex, byte| {
            const DIGITS: &[u8; 16] = b"0123456789abcdef";
            hex.push(char::from(DIGITS[usize::from(byte >> 4)]));
            hex.push(char::from(DIGITS[usize::from(byte & 15)]));
            hex
        })
}

impl Scalar for () {
    fn encode(self) -> OwnedSqlValue {
        OwnedSqlValue::Null
    }
    fn decode(value: &OwnedSqlValue) -> Result<Self, SqlError> {
        if matches!(value, OwnedSqlValue::Null) {
            Ok(())
        } else {
            Err(error(SqlErrorKind::Decode))
        }
    }
}
