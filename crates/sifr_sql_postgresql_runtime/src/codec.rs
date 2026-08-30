use bytes::BytesMut;
use postgres_types::{FromSql, IsNull, Kind, ToSql, Type, to_sql_checked};
use sifr_sql_runtime::{OwnedSqlValue, SqlError, SqlErrorKind};
use std::error::Error;
use std::sync::Arc;

pub(crate) struct PostgresParameter(pub OwnedSqlValue);

impl ToSql for PostgresParameter {
    fn to_sql(
        &self,
        ty: &Type,
        output: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        match &self.0 {
            OwnedSqlValue::Null => Ok(IsNull::Yes),
            OwnedSqlValue::Bool(value) => value.to_sql(ty, output),
            OwnedSqlValue::Signed(value) => encode_signed(*value, ty, output),
            OwnedSqlValue::Unsigned(value) => encode_unsigned(*value, ty, output),
            OwnedSqlValue::Float(value) => encode_float(*value, ty, output),
            OwnedSqlValue::ExactInteger(value) => encode_exact_integer(value, ty, output),
            OwnedSqlValue::Text(value) => encode_text(value, ty, output),
            OwnedSqlValue::Bytes(value) if *ty == Type::BYTEA => {
                output.extend_from_slice(value);
                Ok(IsNull::No)
            }
            OwnedSqlValue::Encoded { payload, .. } => {
                output.extend_from_slice(payload);
                Ok(IsNull::No)
            }
            OwnedSqlValue::Sequence(values) if matches!(ty.kind(), Kind::Array(_)) => values
                .iter()
                .cloned()
                .map(PostgresParameter)
                .collect::<Vec<_>>()
                .to_sql(ty, output),
            OwnedSqlValue::Bytes(_) | OwnedSqlValue::Sequence(_) => Err(codec_error()),
        }
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }

    to_sql_checked!();
}

impl std::fmt::Debug for PostgresParameter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PostgresParameter(<redacted>)")
    }
}

fn encode_signed(
    value: i64,
    ty: &Type,
    output: &mut BytesMut,
) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
    match *ty {
        Type::INT2 => i16::try_from(value)
            .map_err(|_| codec_error())?
            .to_sql(ty, output),
        Type::INT4 => i32::try_from(value)
            .map_err(|_| codec_error())?
            .to_sql(ty, output),
        Type::INT8 => value.to_sql(ty, output),
        Type::NUMERIC => encode_numeric_integer(&value.to_string(), output),
        _ => Err(codec_error()),
    }
}

fn encode_unsigned(
    value: u64,
    ty: &Type,
    output: &mut BytesMut,
) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
    if *ty == Type::NUMERIC {
        return encode_numeric_integer(&value.to_string(), output);
    }
    let signed = i64::try_from(value).map_err(|_| codec_error())?;
    encode_signed(signed, ty, output)
}

#[allow(clippy::cast_possible_truncation, clippy::float_cmp)]
fn encode_float(
    value: f64,
    ty: &Type,
    output: &mut BytesMut,
) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
    if !value.is_finite() {
        return Err(codec_error());
    }
    match *ty {
        Type::FLOAT4 => {
            let narrowed = value as f32;
            if f64::from(narrowed) != value {
                return Err(codec_error());
            }
            narrowed.to_sql(ty, output)
        }
        Type::FLOAT8 => value.to_sql(ty, output),
        _ => Err(codec_error()),
    }
}

fn encode_exact_integer(
    value: &str,
    ty: &Type,
    output: &mut BytesMut,
) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
    if *ty == Type::NUMERIC {
        return encode_numeric_integer(value, output);
    }
    let signed = value.parse::<i64>().map_err(|_| codec_error())?;
    encode_signed(signed, ty, output)
}

fn encode_text(
    value: &str,
    ty: &Type,
    output: &mut BytesMut,
) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
    match *ty {
        Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME | Type::JSON => {
            output.extend_from_slice(value.as_bytes());
            Ok(IsNull::No)
        }
        Type::JSONB => {
            output.extend_from_slice(&[1]);
            output.extend_from_slice(value.as_bytes());
            Ok(IsNull::No)
        }
        _ if matches!(ty.kind(), Kind::Enum(_) | Kind::Domain(_)) => {
            output.extend_from_slice(value.as_bytes());
            Ok(IsNull::No)
        }
        _ => Err(codec_error()),
    }
}

fn encode_numeric_integer(
    value: &str,
    output: &mut BytesMut,
) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
    let (negative, digits) = value
        .strip_prefix('-')
        .map_or((false, value), |digits| (true, digits));
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(codec_error());
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        write_i16(output, 0);
        write_i16(output, 0);
        write_u16(output, 0);
        write_u16(output, 0);
        return Ok(IsNull::No);
    }
    let first_group = digits.len() % 4;
    let mut groups = Vec::new();
    let mut offset = 0;
    if first_group != 0 {
        groups.push(parse_group(&digits[..first_group])?);
        offset = first_group;
    }
    while offset < digits.len() {
        groups.push(parse_group(&digits[offset..offset + 4])?);
        offset += 4;
    }
    let count = i16::try_from(groups.len()).map_err(|_| codec_error())?;
    write_i16(output, count);
    write_i16(output, count.saturating_sub(1));
    write_u16(output, if negative { 0x4000 } else { 0 });
    write_u16(output, 0);
    for group in groups {
        write_i16(output, group);
    }
    Ok(IsNull::No)
}

fn parse_group(value: &str) -> Result<i16, Box<dyn Error + Sync + Send>> {
    value.parse::<i16>().map_err(|_| codec_error())
}

fn write_i16(output: &mut BytesMut, value: i16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn write_u16(output: &mut BytesMut, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn codec_error() -> Box<dyn Error + Sync + Send> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "PostgreSQL value does not match its verified codec",
    ))
}

pub(crate) struct RawPostgresValue(pub Vec<u8>);

impl<'a> FromSql<'a> for RawPostgresValue {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Ok(Self(raw.to_vec()))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

pub(crate) fn decode_value(
    ty: &Type,
    raw: Option<RawPostgresValue>,
) -> Result<OwnedSqlValue, SqlError> {
    let Some(RawPostgresValue(bytes)) = raw else {
        return Ok(OwnedSqlValue::Null);
    };
    match *ty {
        Type::BOOL if bytes.len() == 1 => Ok(OwnedSqlValue::Bool(bytes[0] != 0)),
        Type::INT2 => read_i16(&bytes).map(|value| OwnedSqlValue::Signed(i64::from(value))),
        Type::INT4 => read_i32(&bytes).map(|value| OwnedSqlValue::Signed(i64::from(value))),
        Type::INT8 => read_i64(&bytes).map(OwnedSqlValue::Signed),
        Type::FLOAT4 => read_u32(&bytes)
            .map(f32::from_bits)
            .map(|value| OwnedSqlValue::Float(f64::from(value))),
        Type::FLOAT8 => read_u64(&bytes)
            .map(f64::from_bits)
            .map(OwnedSqlValue::Float),
        Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME | Type::JSON => {
            String::from_utf8(bytes)
                .map(OwnedSqlValue::Text)
                .map_err(|_| SqlError::new(SqlErrorKind::Decode))
        }
        Type::BYTEA => Ok(OwnedSqlValue::Bytes(Arc::from(bytes))),
        _ if matches!(ty.kind(), Kind::Enum(_)) => String::from_utf8(bytes)
            .map(OwnedSqlValue::Text)
            .map_err(|_| SqlError::new(SqlErrorKind::Decode)),
        _ => Ok(OwnedSqlValue::Encoded {
            type_identity: format!("postgresql.oid.{}", ty.oid()),
            payload: Arc::from(bytes),
        }),
    }
}

fn read_i16(bytes: &[u8]) -> Result<i16, SqlError> {
    bytes
        .try_into()
        .map(i16::from_be_bytes)
        .map_err(|_| SqlError::new(SqlErrorKind::Decode))
}

fn read_i32(bytes: &[u8]) -> Result<i32, SqlError> {
    bytes
        .try_into()
        .map(i32::from_be_bytes)
        .map_err(|_| SqlError::new(SqlErrorKind::Decode))
}

fn read_i64(bytes: &[u8]) -> Result<i64, SqlError> {
    bytes
        .try_into()
        .map(i64::from_be_bytes)
        .map_err(|_| SqlError::new(SqlErrorKind::Decode))
}

fn read_u32(bytes: &[u8]) -> Result<u32, SqlError> {
    bytes
        .try_into()
        .map(u32::from_be_bytes)
        .map_err(|_| SqlError::new(SqlErrorKind::Decode))
}

fn read_u64(bytes: &[u8]) -> Result<u64, SqlError> {
    bytes
        .try_into()
        .map(u64::from_be_bytes)
        .map_err(|_| SqlError::new(SqlErrorKind::Decode))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn malformed_wire_values_are_typed_errors_without_panics() {
        let result =
            std::panic::catch_unwind(|| decode_value(&Type::INT8, Some(RawPostgresValue(vec![1]))));
        let decoded = result.expect("decoder must not panic");
        assert_eq!(
            decoded.expect_err("wrong width must fail").kind(),
            SqlErrorKind::Decode
        );

        let result = std::panic::catch_unwind(|| {
            decode_value(&Type::TEXT, Some(RawPostgresValue(vec![0xff, 0xfe])))
        });
        let decoded = result.expect("text decoder must not panic");
        assert_eq!(
            decoded.expect_err("invalid UTF-8 must fail").kind(),
            SqlErrorKind::Decode
        );
    }

    #[test]
    fn bounded_malformed_wire_corpus_never_panics() {
        let types = [
            Type::BOOL,
            Type::INT2,
            Type::INT4,
            Type::INT8,
            Type::FLOAT4,
            Type::FLOAT8,
            Type::TEXT,
        ];
        for ty in &types {
            for length in 0..=32_u8 {
                let bytes = (0..length)
                    .map(|index| index.wrapping_mul(31).wrapping_add(length))
                    .collect::<Vec<_>>();
                let result =
                    std::panic::catch_unwind(|| decode_value(ty, Some(RawPostgresValue(bytes))));
                assert!(
                    result.is_ok(),
                    "wire decoder panicked for {ty} length {length}"
                );
            }
        }
    }
}
