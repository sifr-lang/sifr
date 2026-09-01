use crate::ast::{AlterSequenceStatement, CreateSequenceStatement, SqlSpan};
use crate::raw_adapter::{PostgresParseError, RawAdapter};
use crate::raw_helpers::{
    name_list, object, object_field, optional_array, relation_name, string_field, tagged,
};
use serde_json::{Map, Value};

impl RawAdapter<'_> {
    pub(crate) fn create_sequence(
        body: &Map<String, Value>,
    ) -> Result<CreateSequenceStatement, PostgresParseError> {
        Ok(CreateSequenceStatement {
            name: relation_name(object_field(body, "sequence")?),
        })
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
