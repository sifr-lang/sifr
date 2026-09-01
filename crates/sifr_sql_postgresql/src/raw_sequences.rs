use crate::ast::{AlterSequenceStatement, CreateSequenceStatement, SequenceDataType, SqlSpan};
use crate::raw_adapter::{PostgresParseError, RawAdapter};
use crate::raw_helpers::{
    bool_field, name_list, object, object_field, optional_array, optional_object_field,
    relation_name, string_field, tagged, type_name,
};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

impl RawAdapter<'_> {
    pub(crate) fn create_sequence(
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
                "increment" => statement.increment = sequence_integer(option, name)?,
                "minvalue" => statement.minimum = optional_sequence_integer(option, name)?,
                "maxvalue" => statement.maximum = optional_sequence_integer(option, name)?,
                "start" => statement.start = Some(sequence_integer(option, name)?),
                "cache" => statement.cache = sequence_integer(option, name)?,
                "cycle" => statement.cycle = sequence_boolean(option, name)?,
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
    option: &Map<String, Value>,
    name: &str,
) -> Result<Option<i64>, PostgresParseError> {
    optional_object_field(option, "arg")
        .map(|argument| sequence_integer_argument(argument, name))
        .transpose()
}

fn sequence_integer(option: &Map<String, Value>, name: &str) -> Result<i64, PostgresParseError> {
    sequence_integer_argument(object_field(option, "arg")?, name)
}

fn sequence_integer_argument(
    argument: &Map<String, Value>,
    name: &str,
) -> Result<i64, PostgresParseError> {
    let (tag, body) = tagged(argument, "sequence integer")?;
    let value = match tag {
        "Integer" => body.get("ival").and_then(Value::as_i64),
        "Float" => string_field(body, "fval").and_then(|value| value.parse().ok()),
        _ => None,
    };
    value.ok_or_else(|| sequence_error(format!("sequence {name} is not an i64 integer")))
}

fn sequence_boolean(option: &Map<String, Value>, name: &str) -> Result<bool, PostgresParseError> {
    let argument = object_field(option, "arg")?;
    let (tag, body) = tagged(argument, "sequence boolean")?;
    if tag != "Boolean" {
        return Err(sequence_error(format!("sequence {name} is not a boolean")));
    }
    bool_field(body, "boolval")
        .ok_or_else(|| sequence_error(format!("sequence {name} has no boolean value")))
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
