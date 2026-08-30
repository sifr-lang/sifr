use crate::ast::{
    ColumnDefinition, ConflictAction, ConflictClause, CreateDomainStatement, CreateEnumStatement,
    CreateFunctionStatement, CreateIndexStatement, CreateSequenceStatement, CreateTableStatement,
    CreateViewStatement, DeleteStatement, Expression, ExpressionKind, FromItem, InsertStatement,
    JoinKind, OrderDirection, OrderItem, PostgresStatement, PostgresTypeName, SelectItem,
    SelectStatement, SetOperation, SetOperator, SqlSpan, StatementKind, TableConstraint,
    UpdateStatement,
};
use crate::diagnostic::{PostgresDiagnostic, PostgresDiagnosticCode};
use crate::ffi;
use crate::raw_helpers::{
    alias, array, bool_field, name_list, object, object_field, optional_array,
    optional_object_field, relation_name, string_field, string_node, tagged, type_name, u32_field,
};
use crate::raw_writes::{assignments, returning_items};
use serde_json::{Map, Value};
use std::fmt;

pub trait PostgresParser {
    fn server_major(&self) -> u16;
    fn parse(&self, source: &str) -> Result<Vec<PostgresStatement>, PostgresParseError>;
    fn normalize(&self, source: &str) -> Result<String, PostgresParseError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LibpgQueryParser;

impl PostgresParser for LibpgQueryParser {
    fn server_major(&self) -> u16 {
        env!("SIFR_POSTGRESQL_MAJOR").parse().unwrap_or(18)
    }

    fn parse(&self, source: &str) -> Result<Vec<PostgresStatement>, PostgresParseError> {
        let raw = ffi::parse_json(source).map_err(PostgresParseError::from_raw)?;
        RawAdapter::new(source).statements(&raw)
    }

    fn normalize(&self, source: &str) -> Result<String, PostgresParseError> {
        ffi::normalize(source).map_err(PostgresParseError::from_raw)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresParseError {
    pub diagnostic: PostgresDiagnostic,
}

impl PostgresParseError {
    fn from_raw(error: ffi::RawParserError) -> Self {
        Self {
            diagnostic: PostgresDiagnostic::at_sql(
                PostgresDiagnosticCode::Parse,
                error.message,
                error.cursor,
                error.cursor.saturating_add(1),
            ),
        }
    }

    pub(crate) fn unsupported(message: impl Into<String>, span: SqlSpan) -> Self {
        Self {
            diagnostic: PostgresDiagnostic::at_sql(
                PostgresDiagnosticCode::UnsupportedCoreSyntax,
                message,
                span.start,
                span.end,
            ),
        }
    }
}

impl fmt::Display for PostgresParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.diagnostic.message)
    }
}

impl std::error::Error for PostgresParseError {}

pub(crate) struct RawAdapter<'a> {
    pub(crate) source: &'a str,
}

impl<'a> RawAdapter<'a> {
    const fn new(source: &'a str) -> Self {
        Self { source }
    }

    fn statements(&self, root: &Value) -> Result<Vec<PostgresStatement>, PostgresParseError> {
        let parsed_major = root
            .get("version")
            .and_then(Value::as_u64)
            .map(|version| version / 10_000);
        if parsed_major != Some(u64::from(LibpgQueryParser.server_major())) {
            return Err(PostgresParseError::unsupported(
                "libpg_query parse-tree version does not match the selected PostgreSQL component",
                SqlSpan::default(),
            ));
        }
        let statements = array(root, "stmts")?;
        statements
            .iter()
            .map(|entry| {
                let wrapper = object(entry, "statement")?;
                let raw = object_field(wrapper, "stmt")?;
                let (name, body) = tagged(raw, "statement")?;
                let location = u32_field(wrapper, "stmt_location").unwrap_or(0);
                let length = u32_field(wrapper, "stmt_len")
                    .filter(|value| *value > 0)
                    .unwrap_or_else(|| {
                        u32::try_from(self.source.len())
                            .unwrap_or(u32::MAX)
                            .saturating_sub(location)
                    });
                let span = SqlSpan {
                    start: location,
                    end: location.saturating_add(length),
                };
                let kind = match name {
                    "SelectStmt" => StatementKind::Select(self.select(body)?),
                    "InsertStmt" => StatementKind::Insert(self.insert(body)?),
                    "UpdateStmt" => StatementKind::Update(self.update(body)?),
                    "DeleteStmt" => StatementKind::Delete(self.delete(body)?),
                    "CreateStmt" => StatementKind::CreateTable(self.create_table(body)?),
                    "CreateEnumStmt" => StatementKind::CreateEnum(Self::create_enum(body)),
                    "CreateDomainStmt" => StatementKind::CreateDomain(Self::create_domain(body)?),
                    "CompositeTypeStmt" => {
                        StatementKind::CreateComposite(self.create_composite(body)?)
                    }
                    "CreateRangeStmt" => StatementKind::CreateRange(self.create_range(body)?),
                    "ViewStmt" => StatementKind::CreateView(self.create_view(body)?),
                    "CreateTableAsStmt"
                        if [string_field(body, "objtype"), string_field(body, "relkind")]
                            .into_iter()
                            .flatten()
                            .any(|kind| kind == "OBJECT_MATVIEW") =>
                    {
                        StatementKind::CreateView(self.create_materialized_view(body)?)
                    }
                    "IndexStmt" => StatementKind::CreateIndex(Self::create_index(body)?),
                    "CreateSeqStmt" => StatementKind::CreateSequence(Self::create_sequence(body)?),
                    "CreateFunctionStmt" => {
                        StatementKind::CreateFunction(Self::create_function(body)?)
                    }
                    other => {
                        return Err(PostgresParseError::unsupported(
                            format!("PostgreSQL core compiler does not support {other}"),
                            span,
                        ));
                    }
                };
                Ok(PostgresStatement { kind, span })
            })
            .collect()
    }

    pub(crate) fn select(
        &self,
        value: &Map<String, Value>,
    ) -> Result<SelectStatement, PostgresParseError> {
        let operation = string_field(value, "op").unwrap_or("SETOP_NONE");
        let set_operation = if operation == "SETOP_NONE" {
            None
        } else {
            let operator = match operation {
                "SETOP_UNION" => SetOperator::Union,
                "SETOP_INTERSECT" => SetOperator::Intersect,
                "SETOP_EXCEPT" => SetOperator::Except,
                _ => {
                    return Err(self.invalid("unknown PostgreSQL set operator", value));
                }
            };
            Some(SetOperation {
                operator,
                all: bool_field(value, "all").unwrap_or(false),
                left: Box::new(self.select(object_field(value, "larg")?)?),
                right: Box::new(self.select(object_field(value, "rarg")?)?),
            })
        };
        Ok(SelectStatement {
            common_tables: self.common_tables(value)?,
            recursive: Self::with_recursive(value)?,
            targets: optional_array(value, "targetList")
                .iter()
                .map(|target| self.select_item(target))
                .collect::<Result<_, _>>()?,
            from: optional_array(value, "fromClause")
                .iter()
                .map(|item| self.parse_from_item(item))
                .collect::<Result<_, _>>()?,
            predicate: optional_object_field(value, "whereClause")
                .map(|expression| self.expression_object(expression))
                .transpose()?,
            group_by: optional_array(value, "groupClause")
                .iter()
                .map(|expression| self.expression(expression))
                .collect::<Result<_, _>>()?,
            having: optional_object_field(value, "havingClause")
                .map(|expression| self.expression_object(expression))
                .transpose()?,
            order_by: optional_array(value, "sortClause")
                .iter()
                .map(|order| self.order_item(order))
                .collect::<Result<_, _>>()?,
            windows: self.named_windows(value)?,
            limit: optional_object_field(value, "limitCount")
                .map(|expression| self.expression_object(expression))
                .transpose()?,
            offset: optional_object_field(value, "limitOffset")
                .map(|expression| self.expression_object(expression))
                .transpose()?,
            locking: Self::locking_clauses(value)?,
            values: optional_array(value, "valuesLists")
                .iter()
                .map(|row| {
                    let (_, list) = tagged(object(row, "values row")?, "values row")?;
                    optional_array(list, "items")
                        .iter()
                        .map(|item| self.expression(item))
                        .collect()
                })
                .collect::<Result<_, _>>()?,
            set_operation,
        })
    }

    pub(crate) fn select_item(&self, value: &Value) -> Result<SelectItem, PostgresParseError> {
        let (_, target) = tagged(object(value, "select target")?, "select target")?;
        let expression = self.expression_object(object_field(target, "val")?)?;
        Ok(SelectItem {
            span: self.span(target),
            expression,
            alias: string_field(target, "name").map(str::to_string),
        })
    }

    fn parse_from_item(&self, value: &Value) -> Result<FromItem, PostgresParseError> {
        let (name, body) = tagged(object(value, "FROM item")?, "FROM item")?;
        match name {
            "RangeVar" => Ok(FromItem::Relation {
                name: relation_name(body),
                alias: alias(body),
                span: self.span(body),
            }),
            "RangeSubselect" => {
                let subquery = object_field(body, "subquery")?;
                let (_, select) = tagged(subquery, "subquery")?;
                Ok(FromItem::Subquery {
                    query: Box::new(self.select(select)?),
                    alias: alias(body).ok_or_else(|| {
                        self.invalid("derived PostgreSQL relation needs an alias", body)
                    })?,
                    lateral: bool_field(body, "lateral").unwrap_or(false),
                    span: self.span(body),
                })
            }
            "JoinExpr" => {
                let join = match string_field(body, "jointype").unwrap_or("JOIN_INNER") {
                    "JOIN_INNER" => JoinKind::Inner,
                    "JOIN_LEFT" => JoinKind::Left,
                    "JOIN_RIGHT" => JoinKind::Right,
                    "JOIN_FULL" => JoinKind::Full,
                    "JOIN_CROSS" => JoinKind::Cross,
                    _ => return Err(self.invalid("unknown PostgreSQL join kind", body)),
                };
                Ok(FromItem::Join {
                    join,
                    left: Box::new(self.parse_from_object(object_field(body, "larg")?)?),
                    right: Box::new(self.parse_from_object(object_field(body, "rarg")?)?),
                    condition: optional_object_field(body, "quals")
                        .map(|value| self.expression_object(value))
                        .transpose()?,
                    using_columns: optional_array(body, "usingClause")
                        .iter()
                        .filter_map(string_node)
                        .collect(),
                    span: self.span(body),
                })
            }
            other => Err(PostgresParseError::unsupported(
                format!("unsupported PostgreSQL FROM node {other}"),
                self.span(body),
            )),
        }
    }

    fn parse_from_object(
        &self,
        value: &Map<String, Value>,
    ) -> Result<FromItem, PostgresParseError> {
        self.parse_from_item(&Value::Object(value.clone()))
    }

    pub(crate) fn order_item(&self, value: &Value) -> Result<OrderItem, PostgresParseError> {
        let (_, body) = tagged(object(value, "ORDER BY item")?, "ORDER BY item")?;
        Ok(OrderItem {
            expression: self.expression_object(object_field(body, "node")?)?,
            direction: match string_field(body, "sortby_dir").unwrap_or("SORTBY_DEFAULT") {
                "SORTBY_ASC" => OrderDirection::Ascending,
                "SORTBY_DESC" => OrderDirection::Descending,
                _ => OrderDirection::Default,
            },
        })
    }

    pub(crate) fn expression(&self, value: &Value) -> Result<Expression, PostgresParseError> {
        let object = object(value, "expression")?;
        self.expression_object(object)
    }

    pub(crate) fn expression_object(
        &self,
        value: &Map<String, Value>,
    ) -> Result<Expression, PostgresParseError> {
        let (name, body) = tagged(value, "expression")?;
        let span = self.span(body);
        let kind = match name {
            "ColumnRef" if Self::star_qualifier(body).is_some() => ExpressionKind::Star {
                qualifier: Self::star_qualifier(body).unwrap_or_default(),
            },
            "ColumnRef" => ExpressionKind::Column {
                path: optional_array(body, "fields")
                    .iter()
                    .filter_map(string_node)
                    .collect(),
            },
            "ParamRef" => ExpressionKind::Parameter {
                number: u32_field(body, "number")
                    .ok_or_else(|| self.invalid("parameter has no number", body))?,
            },
            "A_Const" => self.constant(body)?,
            "TypeCast" => ExpressionKind::Cast {
                expression: Box::new(self.expression_object(object_field(body, "arg")?)?),
                ty: type_name(object_field(body, "typeName")?),
            },
            "A_Expr" => self.operator_expression(body)?,
            "BoolExpr" => {
                let mut expressions = optional_array(body, "args")
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                match string_field(body, "boolop").unwrap_or("AND_EXPR") {
                    "AND_EXPR" => ExpressionKind::BooleanList {
                        and: true,
                        expressions,
                    },
                    "OR_EXPR" => ExpressionKind::BooleanList {
                        and: false,
                        expressions,
                    },
                    "NOT_EXPR" if expressions.len() == 1 => ExpressionKind::Unary {
                        operator: "NOT".to_string(),
                        expression: Box::new(expressions.remove(0)),
                    },
                    other => {
                        return Err(self.invalid(
                            format!("unsupported PostgreSQL boolean expression {other}"),
                            body,
                        ));
                    }
                }
            }
            "FuncCall" => ExpressionKind::Function {
                name: name_list(body, "funcname"),
                arguments: optional_array(body, "args")
                    .iter()
                    .map(|argument| self.expression(argument))
                    .collect::<Result<_, _>>()?,
                aggregate_star: bool_field(body, "agg_star").unwrap_or(false),
                distinct: bool_field(body, "agg_distinct").unwrap_or(false),
                filter: optional_object_field(body, "agg_filter")
                    .map(|value| self.expression_object(value).map(Box::new))
                    .transpose()?,
                window: optional_object_field(body, "over")
                    .map(|value| self.window_specification(value, true))
                    .transpose()?,
            },
            "A_ArrayExpr" => self.array_expression(body)?,
            "CaseExpr" => self.case_expression(body)?,
            "CoalesceExpr" => self.coalesce_expression(body)?,
            "NullTest" => ExpressionKind::NullTest {
                expression: Box::new(self.expression_object(object_field(body, "arg")?)?),
                is_not: string_field(body, "nulltesttype").unwrap_or("IS_NULL") == "IS_NOT_NULL",
            },
            "SubLink" => self.subquery_expression(body)?,
            "SetToDefault" => ExpressionKind::Default,
            other => {
                return Err(PostgresParseError::unsupported(
                    format!("unsupported PostgreSQL expression {other}"),
                    span,
                ));
            }
        };
        Ok(Expression { kind, span })
    }

    fn operator_expression(
        &self,
        body: &Map<String, Value>,
    ) -> Result<ExpressionKind, PostgresParseError> {
        let kind = string_field(body, "kind").unwrap_or("AEXPR_OP");
        let operator = name_list(body, "name").join(".");
        match kind {
            "AEXPR_OP" | "AEXPR_LIKE" | "AEXPR_DISTINCT" | "AEXPR_NOT_DISTINCT" => {
                let operator = match (kind, operator.as_str()) {
                    ("AEXPR_LIKE", "~~") => "LIKE".to_string(),
                    ("AEXPR_LIKE", "!~~") => "NOT LIKE".to_string(),
                    ("AEXPR_DISTINCT", _) => "IS DISTINCT FROM".to_string(),
                    ("AEXPR_NOT_DISTINCT", _) => "IS NOT DISTINCT FROM".to_string(),
                    _ => operator,
                };
                match (
                    optional_object_field(body, "lexpr"),
                    optional_object_field(body, "rexpr"),
                ) {
                    (None, Some(right)) if kind == "AEXPR_OP" => Ok(ExpressionKind::Unary {
                        operator,
                        expression: Box::new(self.expression_object(right)?),
                    }),
                    (Some(left), Some(right)) => Ok(ExpressionKind::Binary {
                        operator,
                        left: Box::new(self.expression_object(left)?),
                        right: Box::new(self.expression_object(right)?),
                    }),
                    _ => Err(self.invalid("PostgreSQL operator has missing operands", body)),
                }
            }
            "AEXPR_IN" => {
                let list = object_field(body, "rexpr")?;
                let (tag, list) = tagged(list, "IN expression list")?;
                if tag != "List" {
                    return Err(self.invalid("PostgreSQL IN right operand is not a list", body));
                }
                Ok(ExpressionKind::InList {
                    expression: Box::new(self.expression_object(object_field(body, "lexpr")?)?),
                    values: optional_array(list, "items")
                        .iter()
                        .map(|item| self.expression(item))
                        .collect::<Result<_, _>>()?,
                    negated: operator == "<>",
                })
            }
            other => Err(self.invalid(
                format!("unsupported PostgreSQL operator expression {other}"),
                body,
            )),
        }
    }

    fn subquery_expression(
        &self,
        body: &Map<String, Value>,
    ) -> Result<ExpressionKind, PostgresParseError> {
        let subselect = object_field(body, "subselect")?;
        let (_, select) = tagged(subselect, "subquery")?;
        let query = Box::new(self.select(select)?);
        match string_field(body, "subLinkType").unwrap_or("EXPR_SUBLINK") {
            "EXPR_SUBLINK" => Ok(ExpressionKind::Subquery { query }),
            "EXISTS_SUBLINK" => Ok(ExpressionKind::Exists { query }),
            kind @ ("ANY_SUBLINK" | "ALL_SUBLINK" | "ROWCOMPARE_SUBLINK") => {
                let left = self.expression_object(object_field(body, "testexpr")?)?;
                Ok(ExpressionKind::SubqueryComparison {
                    operator: name_list(body, "operName")
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "=".to_string()),
                    left: Box::new(left),
                    query,
                    quantifier: if kind == "ALL_SUBLINK" {
                        crate::ast::SubqueryQuantifier::All
                    } else {
                        crate::ast::SubqueryQuantifier::Any
                    },
                })
            }
            other => Err(self.invalid(
                format!("unsupported PostgreSQL subquery expression {other}"),
                body,
            )),
        }
    }

    fn constant(&self, body: &Map<String, Value>) -> Result<ExpressionKind, PostgresParseError> {
        if bool_field(body, "isnull").unwrap_or(false) {
            return Ok(ExpressionKind::Null);
        }
        if let Some(value) = body.get("val").and_then(Value::as_object) {
            let (kind, value) = tagged(value, "constant value")?;
            return match kind {
                "Integer" => Ok(ExpressionKind::Integer {
                    value: value
                        .get("ival")
                        .map(Value::to_string)
                        .unwrap_or_else(|| "0".to_string()),
                }),
                "Float" => Ok(ExpressionKind::Float {
                    value: string_field(value, "str").unwrap_or("0").to_string(),
                }),
                "String" | "BitString" => Ok(ExpressionKind::String {
                    value: string_field(value, "str").unwrap_or("").to_string(),
                }),
                "Null" => Ok(ExpressionKind::Null),
                _ => Err(self.invalid("unknown PostgreSQL constant", body)),
            };
        }
        if let Some(value) = body.get("ival").and_then(Value::as_object) {
            return Ok(ExpressionKind::Integer {
                value: value
                    .get("ival")
                    .map(Value::to_string)
                    .unwrap_or_else(|| "0".to_string()),
            });
        }
        if let Some(value) = body.get("fval").and_then(Value::as_object) {
            return Ok(ExpressionKind::Float {
                value: string_field(value, "fval").unwrap_or("0").to_string(),
            });
        }
        if let Some(value) = body.get("sval").and_then(Value::as_object) {
            return Ok(ExpressionKind::String {
                value: string_field(value, "sval").unwrap_or("").to_string(),
            });
        }
        if let Some(value) = body.get("boolval").and_then(Value::as_object) {
            return Ok(ExpressionKind::Boolean {
                value: bool_field(value, "boolval").unwrap_or(false),
            });
        }
        Err(self.invalid("unknown PostgreSQL constant", body))
    }

    fn insert(&self, body: &Map<String, Value>) -> Result<InsertStatement, PostgresParseError> {
        let relation = relation_name(object_field(body, "relation")?);
        let columns = optional_array(body, "cols")
            .iter()
            .map(|column| {
                tagged(object(column, "insert column")?, "insert column")?
                    .1
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .ok_or_else(|| self.invalid("insert column has no name", body))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let select = optional_object_field(body, "selectStmt")
            .map(|value| tagged(value, "insert source"))
            .transpose()?
            .map(|(_, value)| self.select(value))
            .transpose()?;
        let (rows, source) = match select {
            Some(select) if !select.values.is_empty() => (select.values, None),
            Some(select) => (Vec::new(), Some(Box::new(select))),
            None => (Vec::new(), None),
        };
        Ok(InsertStatement {
            relation,
            columns,
            rows,
            source,
            conflict: optional_object_field(body, "onConflictClause")
                .map(|value| self.conflict(value))
                .transpose()?,
            returning: returning_items(body, self)?,
        })
    }

    fn conflict(&self, body: &Map<String, Value>) -> Result<ConflictClause, PostgresParseError> {
        let action = match string_field(body, "action").unwrap_or("ONCONFLICT_NONE") {
            "ONCONFLICT_NOTHING" => ConflictAction::Nothing,
            "ONCONFLICT_UPDATE" => ConflictAction::Update,
            _ => return Err(self.invalid("unknown ON CONFLICT action", body)),
        };
        let infer = optional_object_field(body, "infer");
        let target_columns = infer
            .map(|value| optional_array(value, "indexElems"))
            .unwrap_or_default()
            .iter()
            .filter_map(|item| {
                object(item, "conflict target")
                    .ok()
                    .and_then(|item| tagged(item, "conflict target").ok())
                    .and_then(|(_, item)| string_field(item, "name"))
                    .map(str::to_string)
            })
            .collect();
        Ok(ConflictClause {
            action,
            target_columns,
            target_predicate: infer
                .and_then(|value| optional_object_field(value, "whereClause"))
                .map(|value| self.expression_object(value))
                .transpose()?,
            assignments: assignments(body, "targetList", self)?,
            update_predicate: optional_object_field(body, "whereClause")
                .map(|value| self.expression_object(value))
                .transpose()?,
        })
    }

    fn update(&self, body: &Map<String, Value>) -> Result<UpdateStatement, PostgresParseError> {
        let relation = object_field(body, "relation")?;
        Ok(UpdateStatement {
            relation: relation_name(relation),
            alias: alias(relation),
            assignments: assignments(body, "targetList", self)?,
            from: optional_array(body, "fromClause")
                .iter()
                .map(|item| self.parse_from_item(item))
                .collect::<Result<_, _>>()?,
            predicate: optional_object_field(body, "whereClause")
                .map(|value| self.expression_object(value))
                .transpose()?,
            returning: returning_items(body, self)?,
        })
    }

    fn delete(&self, body: &Map<String, Value>) -> Result<DeleteStatement, PostgresParseError> {
        let relation = object_field(body, "relation")?;
        Ok(DeleteStatement {
            relation: relation_name(relation),
            alias: alias(relation),
            using_relations: optional_array(body, "usingClause")
                .iter()
                .map(|item| self.parse_from_item(item))
                .collect::<Result<_, _>>()?,
            predicate: optional_object_field(body, "whereClause")
                .map(|value| self.expression_object(value))
                .transpose()?,
            returning: returning_items(body, self)?,
        })
    }

    fn create_table(
        &self,
        body: &Map<String, Value>,
    ) -> Result<CreateTableStatement, PostgresParseError> {
        let mut columns = Vec::new();
        let mut constraints = Vec::new();
        for element in optional_array(body, "tableElts") {
            let (name, value) = tagged(object(element, "table element")?, "table element")?;
            if name == "ColumnDef" {
                columns.push(self.column_definition(value)?);
            } else if name == "Constraint" {
                constraints.push(self.table_constraint(value)?);
            } else {
                return Err(PostgresParseError::unsupported(
                    format!("unsupported CREATE TABLE element {name}"),
                    self.span(value),
                ));
            }
        }
        Ok(CreateTableStatement {
            name: relation_name(object_field(body, "relation")?),
            columns,
            constraints,
        })
    }

    pub(crate) fn column_definition(
        &self,
        body: &Map<String, Value>,
    ) -> Result<ColumnDefinition, PostgresParseError> {
        let mut nullable = true;
        let mut has_default = false;
        let mut generated = false;
        let mut primary_key = false;
        let mut unique = false;
        let mut references = None;
        for constraint in optional_array(body, "constraints") {
            let (_, constraint) = tagged(object(constraint, "column constraint")?, "constraint")?;
            match string_field(constraint, "contype").unwrap_or("") {
                "CONSTR_NOTNULL" => nullable = false,
                "CONSTR_DEFAULT" => has_default = true,
                "CONSTR_GENERATED" | "CONSTR_IDENTITY" => generated = true,
                "CONSTR_PRIMARY" => {
                    primary_key = true;
                    nullable = false;
                }
                "CONSTR_UNIQUE" => unique = true,
                "CONSTR_FOREIGN" => {
                    references = Some((
                        relation_name(object_field(constraint, "pktable")?),
                        name_list(constraint, "pk_attrs"),
                    ));
                }
                _ => {}
            }
        }
        Ok(ColumnDefinition {
            name: string_field(body, "colname")
                .ok_or_else(|| self.invalid("column has no name", body))?
                .to_string(),
            ty: type_name(object_field(body, "typeName")?),
            nullable,
            has_default,
            generated,
            primary_key,
            unique,
            references,
            span: self.span(body),
        })
    }

    fn table_constraint(
        &self,
        body: &Map<String, Value>,
    ) -> Result<TableConstraint, PostgresParseError> {
        match string_field(body, "contype").unwrap_or("") {
            "CONSTR_PRIMARY" => Ok(TableConstraint::PrimaryKey {
                columns: name_list(body, "keys"),
            }),
            "CONSTR_UNIQUE" => Ok(TableConstraint::Unique {
                columns: name_list(body, "keys"),
            }),
            "CONSTR_FOREIGN" => Ok(TableConstraint::ForeignKey {
                columns: name_list(body, "fk_attrs"),
                relation: relation_name(object_field(body, "pktable")?),
                referenced: name_list(body, "pk_attrs"),
            }),
            "CONSTR_CHECK" => Ok(TableConstraint::Check {
                expression: self.expression_object(object_field(body, "raw_expr")?)?,
            }),
            _ => Err(self.invalid("unsupported table constraint", body)),
        }
    }

    fn create_enum(body: &Map<String, Value>) -> CreateEnumStatement {
        CreateEnumStatement {
            name: name_list(body, "typeName"),
            values: name_list(body, "vals"),
        }
    }

    fn create_domain(
        body: &Map<String, Value>,
    ) -> Result<CreateDomainStatement, PostgresParseError> {
        let nullable = !optional_array(body, "constraints")
            .iter()
            .any(|constraint| {
                object(constraint, "domain constraint")
                    .ok()
                    .and_then(|value| tagged(value, "domain constraint").ok())
                    .and_then(|(_, value)| string_field(value, "contype"))
                    == Some("CONSTR_NOTNULL")
            });
        Ok(CreateDomainStatement {
            name: name_list(body, "domainname"),
            base_type: type_name(object_field(body, "typeName")?),
            nullable,
        })
    }

    fn create_view(
        &self,
        body: &Map<String, Value>,
    ) -> Result<CreateViewStatement, PostgresParseError> {
        let query = object_field(body, "query")?;
        let (_, select) = tagged(query, "view query")?;
        Ok(CreateViewStatement {
            name: relation_name(object_field(body, "view")?),
            query: self.select(select)?,
            materialized: false,
        })
    }

    fn create_materialized_view(
        &self,
        body: &Map<String, Value>,
    ) -> Result<CreateViewStatement, PostgresParseError> {
        let query = object_field(body, "query")?;
        let (_, select) = tagged(query, "materialized view query")?;
        let into = object_field(body, "into")?;
        Ok(CreateViewStatement {
            name: relation_name(object_field(into, "rel")?),
            query: self.select(select)?,
            materialized: true,
        })
    }

    fn create_index(body: &Map<String, Value>) -> Result<CreateIndexStatement, PostgresParseError> {
        Ok(CreateIndexStatement {
            name: string_field(body, "idxname").unwrap_or("").to_string(),
            relation: relation_name(object_field(body, "relation")?),
            columns: optional_array(body, "indexParams")
                .iter()
                .filter_map(|item| {
                    object(item, "index element")
                        .ok()
                        .and_then(|value| tagged(value, "index element").ok())
                        .and_then(|(_, value)| string_field(value, "name"))
                        .map(str::to_string)
                })
                .collect(),
            unique: bool_field(body, "unique").unwrap_or(false),
        })
    }

    fn create_sequence(
        body: &Map<String, Value>,
    ) -> Result<CreateSequenceStatement, PostgresParseError> {
        Ok(CreateSequenceStatement {
            name: relation_name(object_field(body, "sequence")?),
        })
    }

    fn create_function(
        body: &Map<String, Value>,
    ) -> Result<CreateFunctionStatement, PostgresParseError> {
        let mut arguments = Vec::new();
        for parameter in optional_array(body, "parameters") {
            let (_, parameter) = tagged(object(parameter, "function parameter")?, "parameter")?;
            arguments.push(type_name(object_field(parameter, "argType")?));
        }
        let strict = optional_array(body, "options").iter().any(|option| {
            object(option, "function option")
                .ok()
                .and_then(|value| tagged(value, "function option").ok())
                .and_then(|(_, value)| string_field(value, "defname"))
                == Some("strict")
        });
        Ok(CreateFunctionStatement {
            name: name_list(body, "funcname"),
            arguments,
            result: object_field(body, "returnType")
                .map(type_name)
                .unwrap_or_else(|_| PostgresTypeName {
                    path: vec!["void".to_string()],
                    modifiers: Vec::new(),
                    array_dimensions: 0,
                }),
            strict,
            aggregate: false,
        })
    }

    pub(crate) fn span(&self, body: &Map<String, Value>) -> SqlSpan {
        let source_len = u32::try_from(self.source.len()).unwrap_or(u32::MAX);
        let start = body
            .get("location")
            .and_then(Value::as_i64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0)
            .min(source_len);
        let mut end = usize::try_from(start).unwrap_or(self.source.len());
        let bytes = self.source.as_bytes();
        if end < bytes.len() {
            end += self.source[end..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);
            while end < bytes.len()
                && !bytes[end].is_ascii_whitespace()
                && !matches!(bytes[end], b',' | b')' | b'(' | b';')
            {
                end += 1;
            }
        }
        SqlSpan {
            start,
            end: u32::try_from(end).unwrap_or(source_len).min(source_len),
        }
    }

    pub(crate) fn invalid(
        &self,
        message: impl Into<String>,
        body: &Map<String, Value>,
    ) -> PostgresParseError {
        PostgresParseError::unsupported(message, self.span(body))
    }
}
