use crate::ast::{Assignment, SelectItem};
use crate::raw_adapter::{PostgresParseError, RawAdapter};
use crate::raw_helpers::{
    object, object_field, optional_array, optional_object_field, string_field, tagged,
};
use serde_json::{Map, Value};

pub(crate) fn assignments(
    body: &Map<String, Value>,
    key: &str,
    adapter: &RawAdapter<'_>,
) -> Result<Vec<Assignment>, PostgresParseError> {
    optional_array(body, key)
        .iter()
        .map(|assignment| {
            let (_, assignment) = tagged(object(assignment, "assignment")?, "assignment")?;
            Ok(Assignment {
                column: string_field(assignment, "name")
                    .ok_or_else(|| adapter.invalid("assignment has no column", assignment))?
                    .to_string(),
                value: adapter.expression_object(object_field(assignment, "val")?)?,
                span: adapter.span(assignment),
            })
        })
        .collect()
}

pub(crate) fn returning_items(
    body: &Map<String, Value>,
    adapter: &RawAdapter<'_>,
) -> Result<Vec<SelectItem>, PostgresParseError> {
    let clause_items = optional_object_field(body, "returningClause")
        .map(|clause| optional_array(clause, "exprs"))
        .unwrap_or_default();
    let items = if clause_items.is_empty() {
        optional_array(body, "returningList")
    } else {
        clause_items
    };
    items.iter().map(|item| adapter.select_item(item)).collect()
}
