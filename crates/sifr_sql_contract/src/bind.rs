use crate::{CodecRegistry, DatabaseType, Nullability, SifrType, canonical_read_type};
use serde::{Deserialize, Serialize};

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
    if input.nullability == Nullability::Nullable && target.nullability == Nullability::NonNull {
        return BindCompatibility::Rejected(BindRejection::Nullability);
    }
    if input.nullability == Nullability::Nullable
        && matches!(target.database, DatabaseType::Custom { .. })
        && codecs
            .codec_for_database_type(&target.database)
            .is_some_and(|codec| codec.null_behavior == crate::NullCodecBehavior::Reject)
    {
        return BindCompatibility::Rejected(BindRejection::Nullability);
    }

    if let Some(result) = special_compatibility(&input.value, &target.database, codecs) {
        return result;
    }

    match canonical_read_type(&target.database) {
        Ok(mapped) if mapped == input.value => constrained_exact(&target.database),
        Ok(_) | Err(_) => BindCompatibility::Rejected(BindRejection::UnsupportedPair),
    }
}

fn special_compatibility(
    input: &SifrType,
    target: &DatabaseType,
    codecs: &CodecRegistry,
) -> Option<BindCompatibility> {
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
        _ => None,
    }
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
    let SifrType::Union { members } = input else {
        return (input.clone(), Nullability::NonNull);
    };
    if !members.contains(&SifrType::None) {
        return (input.clone(), Nullability::NonNull);
    }
    let mut values = members.clone();
    values.remove(&SifrType::None);
    let value = if values.len() == 1 {
        values.into_iter().next().unwrap_or(SifrType::None)
    } else {
        SifrType::Union { members: values }
    };
    (value, Nullability::Nullable)
}

fn constrained_exact(database: &DatabaseType) -> BindCompatibility {
    match database {
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
