use mysql_common::{Row, Value};
use sifr_sql_runtime::{OwnedSqlValue, RuntimeLimits, SqlError, SqlErrorKind};
use std::sync::Arc;

pub(crate) fn encode_parameters(
    parameters: sifr_sql_runtime::BoundParameters,
) -> Result<Vec<Value>, SqlError> {
    parameters
        .into_values()
        .into_iter()
        .map(|parameter| encode_value(parameter.value))
        .collect()
}

fn encode_value(value: OwnedSqlValue) -> Result<Value, SqlError> {
    match value {
        OwnedSqlValue::Null => Ok(Value::NULL),
        OwnedSqlValue::Bool(value) => Ok(Value::Int(i64::from(value))),
        OwnedSqlValue::Signed(value) => Ok(Value::Int(value)),
        OwnedSqlValue::Unsigned(value) => Ok(Value::UInt(value)),
        OwnedSqlValue::Float(value) if value.is_finite() => Ok(Value::Double(value)),
        OwnedSqlValue::Float(_) => Err(codec_error()),
        OwnedSqlValue::ExactInteger(value) | OwnedSqlValue::Text(value) => {
            Ok(Value::Bytes(value.into_bytes()))
        }
        OwnedSqlValue::Bytes(value) => Ok(Value::Bytes(value.to_vec())),
        OwnedSqlValue::Encoded { payload, .. } => Ok(Value::Bytes(payload.to_vec())),
        OwnedSqlValue::Sequence(_) => Err(codec_error()),
    }
}

pub(crate) fn decode_row(
    row: &Row,
    limits: RuntimeLimits,
) -> Result<(Vec<OwnedSqlValue>, u64), SqlError> {
    let mut values = Vec::with_capacity(row.len());
    let mut decoded_bytes = 0_u64;
    for index in 0..row.len() {
        let value = row.as_ref(index).ok_or_else(codec_error)?;
        let bytes = value_size(value);
        decoded_bytes = decoded_bytes.checked_add(bytes).ok_or_else(codec_error)?;
        if decoded_bytes > limits.max_decoded_row_bytes {
            return Err(SqlError::new(SqlErrorKind::ResourceLimit));
        }
        values.push(decode_value(value)?);
    }
    Ok((values, decoded_bytes))
}

fn decode_value(value: &Value) -> Result<OwnedSqlValue, SqlError> {
    match value {
        Value::NULL => Ok(OwnedSqlValue::Null),
        Value::Bytes(value) => Ok(String::from_utf8(value.clone()).map_or_else(
            |error| OwnedSqlValue::Bytes(Arc::from(error.into_bytes())),
            OwnedSqlValue::Text,
        )),
        Value::Int(value) => Ok(OwnedSqlValue::Signed(*value)),
        Value::UInt(value) => Ok(OwnedSqlValue::Unsigned(*value)),
        Value::Float(value) if value.is_finite() => Ok(OwnedSqlValue::Float(f64::from(*value))),
        Value::Double(value) if value.is_finite() => Ok(OwnedSqlValue::Float(*value)),
        Value::Float(_) | Value::Double(_) => Err(codec_error()),
        Value::Date(year, month, day, hour, minute, second, micros) => Ok(OwnedSqlValue::Encoded {
            type_identity: "mysql.datetime.binary.v1".to_string(),
            payload: Arc::from(
                format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{micros:06}"
                )
                .into_bytes(),
            ),
        }),
        Value::Time(negative, days, hours, minutes, seconds, micros) => {
            Ok(OwnedSqlValue::Encoded {
                type_identity: "mysql.time.binary.v1".to_string(),
                payload: Arc::from(
                    format!(
                        "{}{days}:{hours:02}:{minutes:02}:{seconds:02}.{micros:06}",
                        if *negative { "-" } else { "" }
                    )
                    .into_bytes(),
                ),
            })
        }
    }
}

fn value_size(value: &Value) -> u64 {
    match value {
        Value::NULL => 0,
        Value::Bytes(value) => u64::try_from(value.len()).unwrap_or(u64::MAX),
        Value::Int(_) | Value::UInt(_) | Value::Double(_) => 8,
        Value::Float(_) => 4,
        Value::Date(..) | Value::Time(..) => 16,
    }
}

fn codec_error() -> SqlError {
    SqlError::new(SqlErrorKind::Decode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_nonfinite_values_fail_without_panicking() {
        assert!(decode_value(&Value::Double(f64::NAN)).is_err());
        assert!(decode_value(&Value::Float(f32::INFINITY)).is_err());
    }
}
