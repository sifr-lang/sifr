use mysql_async::{Column, Row, Value, consts::ColumnType};
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
        OwnedSqlValue::Float(_) => Err(encode_error()),
        OwnedSqlValue::ExactInteger(value) | OwnedSqlValue::Text(value) => {
            Ok(Value::Bytes(value.into_bytes()))
        }
        OwnedSqlValue::Bytes(value) => Ok(Value::Bytes(value.to_vec())),
        OwnedSqlValue::Encoded {
            type_identity,
            payload,
        } => match type_identity.as_str() {
            "mysql.date.binary.v1" | "mysql.datetime.binary.v1" | "mysql.timestamp.binary.v1" => {
                let bytes: &[u8; 11] = payload.as_ref().try_into().map_err(|_| encode_error())?;
                let value = Value::Date(
                    u16::from_be_bytes([bytes[0], bytes[1]]),
                    bytes[2],
                    bytes[3],
                    bytes[4],
                    bytes[5],
                    bytes[6],
                    u32::from_be_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]),
                );
                if type_identity == "mysql.date.binary.v1"
                    && bytes[4..].iter().any(|byte| *byte != 0)
                {
                    return Err(encode_error());
                }
                Ok(value)
            }
            "mysql.time.binary.v1" => {
                let bytes: &[u8; 12] = payload.as_ref().try_into().map_err(|_| encode_error())?;
                if bytes[0] > 1 {
                    return Err(encode_error());
                }
                Ok(Value::Time(
                    bytes[0] == 1,
                    u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
                    bytes[5],
                    bytes[6],
                    bytes[7],
                    u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
                ))
            }
            "mysql.json.binary.v1" => Ok(Value::Bytes(payload.to_vec())),
            _ => Ok(Value::Bytes(payload.to_vec())),
        },
        OwnedSqlValue::Sequence(_) => Err(encode_error()),
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
        let column = row.columns_ref().get(index);
        values.push(decode_value(value, column)?);
    }
    Ok((values, decoded_bytes))
}

fn decode_value(value: &Value, column: Option<&Column>) -> Result<OwnedSqlValue, SqlError> {
    let column_type = column.map(Column::column_type);
    match value {
        Value::NULL => Ok(OwnedSqlValue::Null),
        Value::Bytes(value) if column_type == Some(ColumnType::MYSQL_TYPE_JSON) => {
            Ok(OwnedSqlValue::Encoded {
                type_identity: "mysql.json.binary.v1".to_string(),
                payload: Arc::from(value.clone()),
            })
        }
        Value::Bytes(value) => {
            let column = column.ok_or_else(codec_error)?;
            if binary_column(column) {
                Ok(OwnedSqlValue::Bytes(Arc::from(value.clone())))
            } else {
                String::from_utf8(value.clone())
                    .map(OwnedSqlValue::Text)
                    .map_err(|_| codec_error())
            }
        }
        Value::Int(value) => Ok(OwnedSqlValue::Signed(*value)),
        Value::UInt(value) => Ok(OwnedSqlValue::Unsigned(*value)),
        Value::Float(value) if value.is_finite() => Ok(OwnedSqlValue::Float(f64::from(*value))),
        Value::Double(value) if value.is_finite() => Ok(OwnedSqlValue::Float(*value)),
        Value::Float(_) | Value::Double(_) => Err(codec_error()),
        Value::Date(year, month, day, hour, minute, second, micros) => {
            let type_identity = match column_type {
                Some(ColumnType::MYSQL_TYPE_DATE | ColumnType::MYSQL_TYPE_NEWDATE) => {
                    "mysql.date.binary.v1"
                }
                Some(ColumnType::MYSQL_TYPE_TIMESTAMP | ColumnType::MYSQL_TYPE_TIMESTAMP2) => {
                    "mysql.timestamp.binary.v1"
                }
                _ => "mysql.datetime.binary.v1",
            };
            let mut payload = Vec::with_capacity(11);
            payload.extend_from_slice(&year.to_be_bytes());
            payload.extend_from_slice(&[*month, *day, *hour, *minute, *second]);
            payload.extend_from_slice(&micros.to_be_bytes());
            Ok(OwnedSqlValue::Encoded {
                type_identity: type_identity.to_string(),
                payload: Arc::from(payload),
            })
        }
        Value::Time(negative, days, hours, minutes, seconds, micros) => {
            let mut payload = Vec::with_capacity(12);
            payload.push(u8::from(*negative));
            payload.extend_from_slice(&days.to_be_bytes());
            payload.extend_from_slice(&[*hours, *minutes, *seconds]);
            payload.extend_from_slice(&micros.to_be_bytes());
            Ok(OwnedSqlValue::Encoded {
                type_identity: "mysql.time.binary.v1".to_string(),
                payload: Arc::from(payload),
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

fn binary_column(column: &Column) -> bool {
    let ty = column.column_type();
    matches!(
        ty,
        ColumnType::MYSQL_TYPE_BIT
            | ColumnType::MYSQL_TYPE_GEOMETRY
            | ColumnType::MYSQL_TYPE_VECTOR
    ) || (column.character_set() == 63
        && matches!(
            ty,
            ColumnType::MYSQL_TYPE_STRING
                | ColumnType::MYSQL_TYPE_VAR_STRING
                | ColumnType::MYSQL_TYPE_VARCHAR
                | ColumnType::MYSQL_TYPE_TINY_BLOB
                | ColumnType::MYSQL_TYPE_MEDIUM_BLOB
                | ColumnType::MYSQL_TYPE_LONG_BLOB
                | ColumnType::MYSQL_TYPE_BLOB
                | ColumnType::MYSQL_TYPE_ENUM
                | ColumnType::MYSQL_TYPE_SET
        ))
}

fn encode_error() -> SqlError {
    SqlError::new(SqlErrorKind::Encode)
}

fn codec_error() -> SqlError {
    SqlError::new(SqlErrorKind::Decode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_nonfinite_values_fail_without_panicking() {
        assert!(decode_value(&Value::Double(f64::NAN), None).is_err());
        assert!(decode_value(&Value::Float(f32::INFINITY), None).is_err());
    }

    #[test]
    fn codecs_use_the_raw_drivers_exact_parameter_and_row_types() {
        let encode: fn(
            sifr_sql_runtime::BoundParameters,
        ) -> Result<Vec<mysql_async::Value>, SqlError> = encode_parameters;
        let _: fn(&mysql_async::Row, RuntimeLimits) -> Result<(Vec<OwnedSqlValue>, u64), SqlError> =
            decode_row;
        let values = encode(sifr_sql_runtime::BoundParameters::default())
            .unwrap_or_else(|error| panic!("empty parameters must encode: {error:?}"));
        assert_eq!(
            mysql_async::Params::Positional(values),
            mysql_async::Params::Positional(vec![])
        );
    }

    #[test]
    fn driver_values_preserve_integer_limits_and_binary_payloads() {
        assert!(matches!(
            encode_value(OwnedSqlValue::Unsigned(u64::MAX)),
            Ok(Value::UInt(u64::MAX))
        ));
        assert!(matches!(
            decode_value(&Value::Int(i64::MIN), None),
            Ok(OwnedSqlValue::Signed(i64::MIN))
        ));
        let bytes = vec![0xff, 0, 0x80];
        assert!(
            matches!(decode_value(&Value::Bytes(bytes.clone()), Some(&Column::new(ColumnType::MYSQL_TYPE_BLOB).with_character_set(63))), Ok(OwnedSqlValue::Bytes(value)) if value.as_ref() == bytes)
        );
        assert_eq!(
            encode_value(OwnedSqlValue::Bytes(Arc::from(bytes.clone()))).ok(),
            Some(Value::Bytes(bytes))
        );
        assert!(matches!(
            decode_value(&Value::NULL, None),
            Ok(OwnedSqlValue::Null)
        ));
        assert!(
            matches!(decode_value(&Value::Bytes(b"text".to_vec()), Some(&Column::new(ColumnType::MYSQL_TYPE_VAR_STRING).with_character_set(45))), Ok(OwnedSqlValue::Text(value)) if value == "text")
        );
    }

    #[test]
    fn standard_mysql_values_round_trip_with_column_identity() {
        for (column_type, identity) in [
            (ColumnType::MYSQL_TYPE_DATE, "mysql.date.binary.v1"),
            (ColumnType::MYSQL_TYPE_DATETIME, "mysql.datetime.binary.v1"),
            (
                ColumnType::MYSQL_TYPE_TIMESTAMP,
                "mysql.timestamp.binary.v1",
            ),
        ] {
            let value = Value::Date(2026, 9, 23, 0, 0, 0, 0);
            let decoded =
                decode_value(&value, Some(&Column::new(column_type))).expect("date family decode");
            let OwnedSqlValue::Encoded { type_identity, .. } = &decoded else {
                panic!("date family must keep its SQL identity");
            };
            assert_eq!(type_identity, identity);
            assert_eq!(encode_value(decoded).ok(), Some(value));
        }
        let value = Value::Time(false, 0, 12, 34, 56, 789);
        let decoded = decode_value(&value, Some(&Column::new(ColumnType::MYSQL_TYPE_TIME)))
            .expect("time decode");
        assert_eq!(encode_value(decoded).ok(), Some(value));
        let json = Value::Bytes(br#"{"ready":true}"#.to_vec());
        let decoded = decode_value(&json, Some(&Column::new(ColumnType::MYSQL_TYPE_JSON)))
            .expect("JSON decode");
        assert!(
            matches!(&decoded, OwnedSqlValue::Encoded { type_identity, .. } if type_identity == "mysql.json.binary.v1")
        );
        assert_eq!(encode_value(decoded).ok(), Some(json));
    }

    #[test]
    fn malformed_mysql_standard_values_reject_encoding() {
        for value in [
            OwnedSqlValue::Encoded {
                type_identity: "mysql.date.binary.v1".to_string(),
                payload: Arc::from([0_u8; 3]),
            },
            OwnedSqlValue::Encoded {
                type_identity: "mysql.time.binary.v1".to_string(),
                payload: Arc::from([2_u8; 12]),
            },
        ] {
            assert_eq!(
                encode_value(value).expect_err("encoding must fail").kind(),
                SqlErrorKind::Encode
            );
        }
    }

    #[test]
    fn unsupported_parameters_remain_typed_errors() {
        for value in [
            OwnedSqlValue::Float(f64::NAN),
            OwnedSqlValue::Float(f64::INFINITY),
            OwnedSqlValue::Sequence(vec![]),
        ] {
            assert_eq!(
                encode_value(value).expect_err("encoding must fail").kind(),
                SqlErrorKind::Encode
            );
        }
    }

    #[test]
    fn binary_and_text_identity_comes_from_column_metadata() {
        for ty in [
            ColumnType::MYSQL_TYPE_STRING,
            ColumnType::MYSQL_TYPE_VAR_STRING,
            ColumnType::MYSQL_TYPE_VARCHAR,
            ColumnType::MYSQL_TYPE_TINY_BLOB,
            ColumnType::MYSQL_TYPE_MEDIUM_BLOB,
            ColumnType::MYSQL_TYPE_LONG_BLOB,
            ColumnType::MYSQL_TYPE_BLOB,
        ] {
            let binary = Column::new(ty).with_character_set(63);
            let text = Column::new(ty).with_character_set(45);
            for bytes in [
                Vec::new(),
                b"valid UTF-8".to_vec(),
                b"a\0b".to_vec(),
                vec![0xff, 0, 0x80],
            ] {
                let native = encode_value(OwnedSqlValue::Bytes(Arc::from(bytes.clone())))
                    .expect("binary encoding");
                assert_eq!(
                    decode_value(&native, Some(&binary)).expect("binary decoding"),
                    OwnedSqlValue::Bytes(Arc::from(bytes.clone()))
                );
                let textual = decode_value(&native, Some(&text));
                match std::str::from_utf8(&bytes) {
                    Ok(value) => assert_eq!(
                        textual.expect("text decoding"),
                        OwnedSqlValue::Text(value.into())
                    ),
                    Err(_) => assert_eq!(
                        textual.expect_err("invalid declared text").kind(),
                        SqlErrorKind::Decode
                    ),
                }
                assert_eq!(
                    decode_value(&native, None)
                        .expect_err("missing column metadata")
                        .kind(),
                    SqlErrorKind::Decode
                );
            }
        }
        // MySQL decimal values also use byte packets and binary character set,
        // but their existing wire representation is textual, not a binary string.
        let decimal = Column::new(ColumnType::MYSQL_TYPE_NEWDECIMAL).with_character_set(63);
        assert_eq!(
            decode_value(&Value::Bytes(b"123.45".to_vec()), Some(&decimal)).expect("decimal"),
            OwnedSqlValue::Text("123.45".into())
        );
        let binary_collation = Column::new(ColumnType::MYSQL_TYPE_VAR_STRING)
            .with_character_set(46)
            .with_flags(mysql_async::consts::ColumnFlags::BINARY_FLAG);
        assert_eq!(
            decode_value(&Value::Bytes(b"text".to_vec()), Some(&binary_collation))
                .expect("binary text collation"),
            OwnedSqlValue::Text("text".into())
        );
    }
}
