use crate::ast::{
    CaseBranch, CommonTableExpression, CreateCompositeStatement, CreateRangeStatement,
    CteMaterialization, ExpressionKind, LockStrength, LockWait, LockingClause,
    NamedWindowDefinition, WindowSpecification,
};
use crate::raw_adapter::{PostgresParseError, RawAdapter};
use crate::raw_helpers::{
    bool_field, name_list, object, object_field, optional_array, optional_object_field,
    relation_name, string_field, string_node, tagged, type_name, u32_field,
};
use serde_json::{Map, Value};

impl RawAdapter<'_> {
    pub(crate) fn create_composite(
        &self,
        body: &Map<String, Value>,
    ) -> Result<CreateCompositeStatement, PostgresParseError> {
        Ok(CreateCompositeStatement {
            name: relation_name(object_field(body, "typevar")?),
            attributes: optional_array(body, "coldeflist")
                .iter()
                .map(|value| {
                    let (kind, column) =
                        tagged(object(value, "composite attribute")?, "composite attribute")?;
                    if kind != "ColumnDef" {
                        return Err(
                            self.invalid("composite attribute is not a column definition", column)
                        );
                    }
                    self.column_definition(column)
                })
                .collect::<Result<_, _>>()?,
        })
    }

    pub(crate) fn create_range(
        &self,
        body: &Map<String, Value>,
    ) -> Result<CreateRangeStatement, PostgresParseError> {
        let mut subtype = None;
        let mut multirange_name = None;
        for value in optional_array(body, "params") {
            let (_, parameter) = tagged(object(value, "range parameter")?, "range parameter")?;
            match string_field(parameter, "defname").unwrap_or_default() {
                "subtype" => {
                    let argument = object_field(parameter, "arg")?;
                    let (_, argument) = tagged(argument, "range subtype")?;
                    subtype = Some(type_name(argument));
                }
                "multirange_type_name" => {
                    let argument = object_field(parameter, "arg")?;
                    let (_, argument) = tagged(argument, "multirange name")?;
                    multirange_name = Some(name_list(argument, "names"));
                }
                _ => {}
            }
        }
        Ok(CreateRangeStatement {
            name: name_list(body, "typeName"),
            subtype: subtype.ok_or_else(|| self.invalid("range type has no subtype", body))?,
            multirange_name,
        })
    }

    pub(crate) fn star_qualifier(body: &Map<String, Value>) -> Option<Vec<String>> {
        let fields = optional_array(body, "fields");
        let (_, last) = tagged(fields.last()?.as_object()?, "star").ok()?;
        let last_tag = fields.last()?.as_object()?.keys().next()?;
        if last_tag != "A_Star" || !last.is_empty() {
            return None;
        }
        Some(
            fields[..fields.len() - 1]
                .iter()
                .filter_map(string_node)
                .collect(),
        )
    }

    pub(crate) fn common_tables(
        &self,
        select: &Map<String, Value>,
    ) -> Result<Vec<CommonTableExpression>, PostgresParseError> {
        let Some(wrapper) = optional_object_field(select, "withClause") else {
            return Ok(Vec::new());
        };
        let clause = if wrapper.contains_key("ctes") {
            wrapper
        } else {
            tagged(wrapper, "WITH clause")?.1
        };
        optional_array(clause, "ctes")
            .iter()
            .map(|value| {
                let (_, cte) = tagged(
                    object(value, "common table expression")?,
                    "common table expression",
                )?;
                let query = object_field(cte, "ctequery")?;
                let (kind, query) = tagged(query, "common table query")?;
                if kind != "SelectStmt" {
                    return Err(self.invalid("a reusable CTE must contain a SELECT", cte));
                }
                Ok(CommonTableExpression {
                    name: string_field(cte, "ctename")
                        .ok_or_else(|| self.invalid("CTE has no name", cte))?
                        .to_string(),
                    columns: name_list(cte, "aliascolnames"),
                    query: Box::new(self.select(query)?),
                    materialization: match string_field(cte, "ctematerialized")
                        .unwrap_or("CTEMaterializeDefault")
                    {
                        "CTEMaterializeAlways" => CteMaterialization::Materialized,
                        "CTEMaterializeNever" => CteMaterialization::NotMaterialized,
                        _ => CteMaterialization::Default,
                    },
                })
            })
            .collect()
    }

    pub(crate) fn with_recursive(select: &Map<String, Value>) -> Result<bool, PostgresParseError> {
        let Some(wrapper) = optional_object_field(select, "withClause") else {
            return Ok(false);
        };
        let clause = if wrapper.contains_key("ctes") {
            wrapper
        } else {
            tagged(wrapper, "WITH clause")?.1
        };
        Ok(bool_field(clause, "recursive").unwrap_or(false))
    }

    pub(crate) fn locking_clauses(
        select: &Map<String, Value>,
    ) -> Result<Vec<LockingClause>, PostgresParseError> {
        optional_array(select, "lockingClause")
            .iter()
            .map(|value| {
                let (_, lock) = tagged(object(value, "locking clause")?, "locking clause")?;
                Ok(LockingClause {
                    strength: match string_field(lock, "strength").unwrap_or("LCS_FORUPDATE") {
                        "LCS_FORKEYSHARE" => LockStrength::KeyShare,
                        "LCS_FORSHARE" => LockStrength::Share,
                        "LCS_FORNOKEYUPDATE" => LockStrength::NoKeyUpdate,
                        _ => LockStrength::Update,
                    },
                    relations: optional_array(lock, "lockedRels")
                        .iter()
                        .filter_map(|value| object(value, "locked relation").ok())
                        .filter_map(|value| tagged(value, "locked relation").ok())
                        .map(|(_, value)| relation_name(value).join("."))
                        .collect(),
                    wait: match string_field(lock, "waitPolicy").unwrap_or("LockWaitBlock") {
                        "LockWaitSkip" => LockWait::SkipLocked,
                        "LockWaitError" => LockWait::NoWait,
                        _ => LockWait::Block,
                    },
                })
            })
            .collect()
    }

    pub(crate) fn window_specification(
        &self,
        wrapper: &Map<String, Value>,
        over_clause: bool,
    ) -> Result<WindowSpecification, PostgresParseError> {
        let window = if wrapper.contains_key("partitionClause")
            || wrapper.contains_key("orderClause")
            || wrapper.contains_key("frameOptions")
        {
            wrapper
        } else {
            tagged(wrapper, "window specification")?.1
        };
        Ok(WindowSpecification {
            reference: string_field(window, "refname")
                .or_else(|| over_clause.then(|| string_field(window, "name")).flatten())
                .map(str::to_string),
            partition_by: optional_array(window, "partitionClause")
                .iter()
                .map(|value| self.expression(value))
                .collect::<Result<_, _>>()?,
            order_by: optional_array(window, "orderClause")
                .iter()
                .map(|value| self.order_item(value))
                .collect::<Result<_, _>>()?,
            frame_options: u32_field(window, "frameOptions").unwrap_or(0),
            start_offset: optional_object_field(window, "startOffset")
                .map(|value| self.expression_object(value).map(Box::new))
                .transpose()?,
            end_offset: optional_object_field(window, "endOffset")
                .map(|value| self.expression_object(value).map(Box::new))
                .transpose()?,
        })
    }

    pub(crate) fn named_windows(
        &self,
        select: &Map<String, Value>,
    ) -> Result<Vec<NamedWindowDefinition>, PostgresParseError> {
        optional_array(select, "windowClause")
            .iter()
            .map(|value| {
                let (_, window) = tagged(object(value, "named window")?, "named window")?;
                Ok(NamedWindowDefinition {
                    name: string_field(window, "name")
                        .ok_or_else(|| self.invalid("named window has no name", window))?
                        .to_string(),
                    specification: self.window_specification(window, false)?,
                })
            })
            .collect()
    }

    pub(crate) fn array_expression(
        &self,
        body: &Map<String, Value>,
    ) -> Result<ExpressionKind, PostgresParseError> {
        Ok(ExpressionKind::Array {
            elements: optional_array(body, "elements")
                .iter()
                .map(|value| self.expression(value))
                .collect::<Result<_, _>>()?,
        })
    }

    pub(crate) fn case_expression(
        &self,
        body: &Map<String, Value>,
    ) -> Result<ExpressionKind, PostgresParseError> {
        let operand = optional_object_field(body, "arg")
            .map(|value| self.expression_object(value).map(Box::new))
            .transpose()?;
        let branches = optional_array(body, "args")
            .iter()
            .map(|value| {
                let (_, branch) = tagged(object(value, "CASE branch")?, "CASE branch")?;
                Ok(CaseBranch {
                    condition: self.expression_object(object_field(branch, "expr")?)?,
                    result: self.expression_object(object_field(branch, "result")?)?,
                })
            })
            .collect::<Result<_, _>>()?;
        let fallback = optional_object_field(body, "defresult")
            .map(|value| self.expression_object(value).map(Box::new))
            .transpose()?;
        Ok(ExpressionKind::Case {
            operand,
            branches,
            fallback,
        })
    }

    pub(crate) fn coalesce_expression(
        &self,
        body: &Map<String, Value>,
    ) -> Result<ExpressionKind, PostgresParseError> {
        Ok(ExpressionKind::Coalesce {
            arguments: optional_array(body, "args")
                .iter()
                .map(|value| self.expression(value))
                .collect::<Result<_, _>>()?,
        })
    }
}
