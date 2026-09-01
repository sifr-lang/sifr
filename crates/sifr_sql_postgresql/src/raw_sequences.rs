use crate::ast::{AlterSequenceStatement, CreateSequenceStatement, SequenceDataType, SqlSpan};
use crate::raw_adapter::{PostgresParseError, RawAdapter};
use crate::raw_helpers::{
    bool_field, name_list, object, object_field, optional_array, optional_object_field,
    relation_name, string_field, tagged, type_name, u32_field,
};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

impl RawAdapter<'_> {
    pub(crate) fn create_sequence(
        &self,
        body: &Map<String, Value>,
    ) -> Result<CreateSequenceStatement, PostgresParseError> {
        let mut statement = CreateSequenceStatement {
            name: relation_name(object_field(body, "sequence")?),
            data_type: SequenceDataType::BigInt,
            increment: 1,
            minimum: None,
            maximum: None,
            start: None,
            cache: 1,
            cycle: false,
            owned_by: None,
        };
        let mut seen = BTreeSet::new();
        for option in optional_array(body, "options") {
            let option = sequence_option(option)?;
            let name = string_field(option, "defname")
                .ok_or_else(|| sequence_error("sequence option has no name"))?;
            if !seen.insert(name.to_string()) {
                return Err(sequence_error(format!(
                    "CREATE SEQUENCE repeats option '{name}'"
                )));
            }
            match name {
                "as" => statement.data_type = sequence_data_type(option)?,
                "increment" => statement.increment = sequence_integer(self.source, option, name)?,
                "minvalue" => {
                    statement.minimum = optional_sequence_integer(self.source, option, name)?;
                }
                "maxvalue" => {
                    statement.maximum = optional_sequence_integer(self.source, option, name)?;
                }
                "start" => statement.start = Some(sequence_integer(self.source, option, name)?),
                "cache" => statement.cache = sequence_integer(self.source, option, name)?,
                "cycle" => statement.cycle = sequence_boolean(self.source, option, name)?,
                "owned_by" => statement.owned_by = sequence_owner(option)?,
                other => {
                    return Err(sequence_error(format!(
                        "PostgreSQL DDL normalization does not support CREATE SEQUENCE option '{other}'"
                    )));
                }
            }
        }
        if statement.increment == 0 {
            return Err(sequence_error("CREATE SEQUENCE increment cannot be zero"));
        }
        if statement.cache <= 0 {
            return Err(sequence_error("CREATE SEQUENCE cache must be positive"));
        }
        Ok(statement)
    }

    pub(crate) fn alter_sequence(
        body: &Map<String, Value>,
    ) -> Result<AlterSequenceStatement, PostgresParseError> {
        let owned_by = optional_array(body, "options")
            .iter()
            .find_map(|option| {
                let option = object(option, "sequence option").ok()?;
                let (tag, option) = tagged(option, "sequence option").ok()?;
                if tag != "DefElem" || string_field(option, "defname") != Some("owned_by") {
                    return None;
                }
                let argument = object_field(option, "arg").ok()?;
                let (tag, list) = tagged(argument, "sequence owner").ok()?;
                (tag == "List").then(|| name_list(list, "items"))
            })
            .filter(|owner| owner.len() >= 2)
            .ok_or_else(|| {
                PostgresParseError::unsupported(
                    "PostgreSQL DDL normalization supports ALTER SEQUENCE only with OWNED BY a table column",
                    SqlSpan::default(),
                )
            })?;
        Ok(AlterSequenceStatement {
            name: relation_name(object_field(body, "sequence")?),
            owned_by,
        })
    }
}

fn sequence_option(value: &Value) -> Result<&Map<String, Value>, PostgresParseError> {
    let option = object(value, "sequence option")?;
    let (tag, option) = tagged(option, "sequence option")?;
    if tag != "DefElem" {
        return Err(sequence_error("sequence option is not a DefElem"));
    }
    Ok(option)
}

fn sequence_data_type(option: &Map<String, Value>) -> Result<SequenceDataType, PostgresParseError> {
    let argument = object_field(option, "arg")?;
    let (tag, body) = tagged(argument, "sequence data type")?;
    if tag != "TypeName" {
        return Err(sequence_error("sequence AS option is not a type name"));
    }
    match type_name(body).path.last().map(String::as_str) {
        Some("int2" | "smallint") => Ok(SequenceDataType::SmallInt),
        Some("int4" | "integer") => Ok(SequenceDataType::Integer),
        Some("int8" | "bigint") => Ok(SequenceDataType::BigInt),
        _ => Err(sequence_error(
            "CREATE SEQUENCE AS supports only smallint, integer, or bigint",
        )),
    }
}

fn optional_sequence_integer(
    source: &str,
    option: &Map<String, Value>,
    name: &str,
) -> Result<Option<i64>, PostgresParseError> {
    optional_object_field(option, "arg")
        .map(|argument| sequence_integer_argument(source, option, argument, name))
        .transpose()
}

fn sequence_integer(
    source: &str,
    option: &Map<String, Value>,
    name: &str,
) -> Result<i64, PostgresParseError> {
    sequence_integer_argument(source, option, object_field(option, "arg")?, name)
}

fn sequence_integer_argument(
    source: &str,
    option: &Map<String, Value>,
    argument: &Map<String, Value>,
    name: &str,
) -> Result<i64, PostgresParseError> {
    let (tag, body) = tagged(argument, "sequence integer")?;
    match tag {
        "Integer" => body
            .get("ival")
            .and_then(Value::as_i64)
            .or_else(|| sequence_integer_from_source(source, option))
            .ok_or_else(|| sequence_error(format!("sequence {name} is not an i64 integer"))),
        "Float" => string_field(body, "fval")
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| sequence_error(format!("sequence {name} is not an i64 integer"))),
        _ => Err(sequence_error(format!(
            "sequence {name} is not an i64 integer"
        ))),
    }
}

fn sequence_boolean(
    source: &str,
    option: &Map<String, Value>,
    name: &str,
) -> Result<bool, PostgresParseError> {
    let argument = object_field(option, "arg")?;
    let (tag, body) = tagged(argument, "sequence boolean")?;
    let value = match tag {
        "Boolean" => bool_field(body, "boolval"),
        "Integer" => body
            .get("ival")
            .and_then(Value::as_i64)
            .and_then(|value| match value {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            }),
        _ => None,
    };
    value
        .or_else(|| sequence_boolean_from_source(source, option))
        .ok_or_else(|| sequence_error(format!("sequence {name} has no boolean value")))
}

fn sequence_integer_from_source(source: &str, option: &Map<String, Value>) -> Option<i64> {
    let start = usize::try_from(u32_field(option, "location")?).ok()?;
    let bytes = source.as_bytes();
    let mut index = start;
    while index < bytes.len() {
        index = skip_sql_spacing(bytes, index)?;
        if index >= bytes.len() || bytes[index] == b';' {
            return None;
        }
        if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            continue;
        }
        let sign = if matches!(bytes[index], b'+' | b'-') {
            let sign = bytes[index];
            index += 1;
            index = skip_sql_spacing(bytes, index)?;
            Some(sign)
        } else {
            None
        };
        let digit_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index > digit_start {
            let digits = source.get(digit_start..index)?;
            return match sign {
                Some(b'-') => format!("-{digits}").parse().ok(),
                Some(b'+') | None => digits.parse().ok(),
                _ => None,
            };
        }
        if sign.is_some() {
            return None;
        }
        index += 1;
    }
    None
}

fn sequence_boolean_from_source(source: &str, option: &Map<String, Value>) -> Option<bool> {
    let start = usize::try_from(u32_field(option, "location")?).ok()?;
    let bytes = source.as_bytes();
    let index = skip_sql_spacing(bytes, start)?;
    let tail = bytes.get(index..)?;
    let end = tail
        .iter()
        .position(|byte| !byte.is_ascii_alphabetic())
        .map_or(bytes.len(), |offset| index + offset);
    let keyword = source.get(index..end)?;
    if keyword.eq_ignore_ascii_case("NO") {
        Some(false)
    } else if keyword.eq_ignore_ascii_case("CYCLE") {
        Some(true)
    } else {
        None
    }
}

fn skip_sql_spacing(bytes: &[u8], mut index: usize) -> Option<usize> {
    loop {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if bytes.get(index..index + 2) == Some(b"--") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            index += 2;
            let mut depth = 1_u32;
            while index < bytes.len() && depth > 0 {
                if bytes.get(index..index + 2) == Some(b"/*") {
                    depth = depth.checked_add(1)?;
                    index += 2;
                } else if bytes.get(index..index + 2) == Some(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            if depth > 0 {
                return None;
            }
            continue;
        }
        return Some(index);
    }
}

fn sequence_owner(option: &Map<String, Value>) -> Result<Option<Vec<String>>, PostgresParseError> {
    let argument = object_field(option, "arg")?;
    let (tag, list) = tagged(argument, "sequence owner")?;
    if tag != "List" {
        return Err(sequence_error("sequence owner is not a name list"));
    }
    let owner = name_list(list, "items");
    if owner.len() == 1 && owner[0].eq_ignore_ascii_case("none") {
        Ok(None)
    } else if owner.len() >= 2 {
        Ok(Some(owner))
    } else {
        Err(sequence_error(
            "sequence owner must name a table column or NONE",
        ))
    }
}

fn sequence_error(message: impl Into<String>) -> PostgresParseError {
    PostgresParseError::unsupported(message, SqlSpan::default())
}
