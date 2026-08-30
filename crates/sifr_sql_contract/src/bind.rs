use crate::{
    CodecRegistry, DatabaseType, Nullability, SifrType, SqliteStorageClass, canonical_read_type_in,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputType {
    pub value: SifrType,
    pub nullability: Nullability,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterType {
    pub database: DatabaseType,
    pub nullability: Nullability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncodeCheck {
    ExactIntegerRange,
    Float32RangeAndPrecision,
    DecimalPrecisionAndScale,
    TextLength,
    BinaryLength,
    ArrayShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindRejection {
    Nullability,
    IntegerWidth,
    IntegerSign,
    ArrayElement,
    NominalIdentity,
    MissingCodec,
    UnsupportedPair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", content = "reason", rename_all = "snake_case")]
pub enum BindCompatibility {
    Exact,
    Fallible(EncodeCheck),
    Rejected(BindRejection),
}

#[must_use]
pub fn bind_compatibility(
    input: &InputType,
    target: &ParameterType,
    codecs: &CodecRegistry,
) -> BindCompatibility {
    let (value, inline_nullability) = split_nested_nullability(&input.value);
    let nullability = if input.nullability == Nullability::Nullable
        || inline_nullability == Nullability::Nullable
    {
        Nullability::Nullable
    } else {
        Nullability::NonNull
    };
    if nullability == Nullability::Nullable && target.nullability == Nullability::NonNull {
        return BindCompatibility::Rejected(BindRejection::Nullability);
    }
    if nullability == Nullability::Nullable
        && codecs
            .codec_for_database_type(&target.database)
            .is_some_and(|codec| codec.null_behavior == crate::NullCodecBehavior::Reject)
    {
        return BindCompatibility::Rejected(BindRejection::Nullability);
    }
    if value == SifrType::None {
        return BindCompatibility::Exact;
    }

    if let Some(result) = special_compatibility(&value, &target.database, codecs) {
        return result;
    }

    match canonical_read_type_in(&target.database, codecs) {
        Ok(mapped) if mapped == value => constrained_exact(&target.database),
        Ok(_) | Err(_) => BindCompatibility::Rejected(BindRejection::UnsupportedPair),
    }
}

fn special_compatibility(
    input: &SifrType,
    target: &DatabaseType,
    codecs: &CodecRegistry,
) -> Option<BindCompatibility> {
    if let DatabaseType::Named { canonical, .. } = target {
        return special_compatibility(input, canonical, codecs);
    }
    match (input, target) {
        (SifrType::ExactInteger, DatabaseType::Integer { .. }) => {
            Some(BindCompatibility::Fallible(EncodeCheck::ExactIntegerRange))
        }
        (
            SifrType::FixedInteger {
                sign: input_sign,
                width: _,
            },
            DatabaseType::Integer {
                sign: target_sign,
                width: _,
            },
        ) if input_sign != target_sign => {
            Some(BindCompatibility::Rejected(BindRejection::IntegerSign))
        }
        (
            SifrType::FixedInteger {
                width: input_width, ..
            },
            DatabaseType::Integer {
                width: target_width,
                ..
            },
        ) if input_width != target_width => {
            Some(BindCompatibility::Rejected(BindRejection::IntegerWidth))
        }
        (SifrType::Float, DatabaseType::Float32) => Some(BindCompatibility::Fallible(
            EncodeCheck::Float32RangeAndPrecision,
        )),
        (SifrType::Float, DatabaseType::Float64) => Some(BindCompatibility::Exact),
        (
            SifrType::Str,
            DatabaseType::Text { fixed: true, .. }
            | DatabaseType::Text {
                max_characters: Some(_),
                ..
            },
        ) => Some(BindCompatibility::Fallible(EncodeCheck::TextLength)),
        (SifrType::Bytes, DatabaseType::Binary { max_bytes: Some(_) }) => {
            Some(BindCompatibility::Fallible(EncodeCheck::BinaryLength))
        }
        (
            SifrType::List { element },
            DatabaseType::Array {
                element: target_element,
                dimensions,
                element_nullability,
                ..
            },
        ) => Some(list_array_compatibility(
            element,
            target_element,
            *dimensions,
            *element_nullability,
            codecs,
        )),
        (
            SifrType::SqlArray { element },
            DatabaseType::Array {
                element: target_element,
                dimensions,
                element_nullability,
                ..
            },
        ) => Some(array_element_compatibility(
            element,
            target_element,
            *element_nullability,
            codecs,
            if dimensions.is_some() {
                BindCompatibility::Fallible(EncodeCheck::ArrayShape)
            } else {
                BindCompatibility::Exact
            },
        )),
        (
            SifrType::Nominal { identity: left },
            DatabaseType::Enum { identity: right }
            | DatabaseType::Domain {
                identity: right, ..
            }
            | DatabaseType::Composite { identity: right },
        ) => Some(if left == right {
            BindCompatibility::Exact
        } else {
            BindCompatibility::Rejected(BindRejection::NominalIdentity)
        }),
        (SifrType::Custom { identity }, database @ DatabaseType::Custom { codec, .. }) => {
            Some(match codecs.codec_for_database_type(database) {
                Some(contract)
                    if contract.identity == *codec
                        && contract.sifr_type
                            == (SifrType::Custom {
                                identity: identity.clone(),
                            }) =>
                {
                    BindCompatibility::Exact
                }
                Some(_) => BindCompatibility::Rejected(BindRejection::NominalIdentity),
                None => BindCompatibility::Rejected(BindRejection::MissingCodec),
            })
        }
        (input, DatabaseType::SqliteDynamic { storage_classes }) => {
            Some(sqlite_dynamic_compatibility(input, storage_classes))
        }
        _ => None,
    }
}

fn sqlite_dynamic_compatibility(
    input: &SifrType,
    allowed: &BTreeSet<SqliteStorageClass>,
) -> BindCompatibility {
    let Some(required) = sqlite_storage_classes(input) else {
        return BindCompatibility::Rejected(BindRejection::UnsupportedPair);
    };
    if !required.is_subset(allowed) {
        return BindCompatibility::Rejected(BindRejection::UnsupportedPair);
    }
    if matches!(input, SifrType::ExactInteger)
        || matches!(
            input,
            SifrType::FixedInteger {
                sign: crate::IntegerSign::Unsigned,
                width: crate::IntegerWidth::Bits64,
            }
        )
    {
        BindCompatibility::Fallible(EncodeCheck::ExactIntegerRange)
    } else {
        BindCompatibility::Exact
    }
}

fn sqlite_storage_classes(input: &SifrType) -> Option<BTreeSet<SqliteStorageClass>> {
    let storage = match input {
        SifrType::Bool | SifrType::FixedInteger { .. } | SifrType::ExactInteger => {
            SqliteStorageClass::Integer
        }
        SifrType::Float => SqliteStorageClass::Real,
        SifrType::Str => SqliteStorageClass::Text,
        SifrType::Bytes => SqliteStorageClass::Blob,
        SifrType::None => SqliteStorageClass::Null,
        SifrType::Union { members } => {
            let mut result = BTreeSet::new();
            for member in members {
                result.extend(sqlite_storage_classes(member)?);
            }
            return Some(result);
        }
        _ => return None,
    };
    Some(BTreeSet::from([storage]))
}

fn list_array_compatibility(
    input: &SifrType,
    target: &DatabaseType,
    dimensions: Option<u8>,
    element_nullability: Nullability,
    codecs: &CodecRegistry,
) -> BindCompatibility {
    if dimensions.is_some_and(|count| count != 1) {
        return BindCompatibility::Rejected(BindRejection::ArrayElement);
    }
    array_element_compatibility(
        input,
        target,
        element_nullability,
        codecs,
        BindCompatibility::Exact,
    )
}

fn array_element_compatibility(
    input: &SifrType,
    target: &DatabaseType,
    element_nullability: Nullability,
    codecs: &CodecRegistry,
    success: BindCompatibility,
) -> BindCompatibility {
    let (value, nullability) = split_nested_nullability(input);
    let nested = bind_compatibility(
        &InputType { value, nullability },
        &ParameterType {
            database: target.clone(),
            nullability: element_nullability,
        },
        codecs,
    );
    match nested {
        BindCompatibility::Exact => success,
        BindCompatibility::Fallible(check) => BindCompatibility::Fallible(check),
        BindCompatibility::Rejected(_) => BindCompatibility::Rejected(BindRejection::ArrayElement),
    }
}

fn split_nested_nullability(input: &SifrType) -> (SifrType, Nullability) {
    if input == &SifrType::None {
        return (SifrType::None, Nullability::Nullable);
    }
    let SifrType::Union { members } = input else {
        return (input.clone(), Nullability::NonNull);
    };
    if !members.contains(&SifrType::None) {
        return (input.clone(), Nullability::NonNull);
    }
    let mut values = members.clone();
    values.remove(&SifrType::None);
    let value = match values.len() {
        0 => SifrType::None,
        1 => values.into_iter().next().unwrap_or(SifrType::None),
        _ => SifrType::Union { members: values },
    };
    (value, Nullability::Nullable)
}

fn constrained_exact(database: &DatabaseType) -> BindCompatibility {
    if let DatabaseType::Named { canonical, .. } = database {
        return constrained_exact(canonical);
    }
    match database {
        DatabaseType::Decimal {
            precision: Some(_), ..
        }
        | DatabaseType::Decimal { scale: Some(_), .. } => {
            BindCompatibility::Fallible(EncodeCheck::DecimalPrecisionAndScale)
        }
        DatabaseType::Text {
            fixed: true,
            max_characters: _,
        }
        | DatabaseType::Text {
            max_characters: Some(_),
            ..
        } => BindCompatibility::Fallible(EncodeCheck::TextLength),
        DatabaseType::Binary { max_bytes: Some(_) } => {
            BindCompatibility::Fallible(EncodeCheck::BinaryLength)
        }
        _ => BindCompatibility::Exact,
    }
}
