use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqlSpan {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresStatement {
    pub kind: StatementKind,
    pub span: SqlSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StatementKind {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    CreateEnum(CreateEnumStatement),
    CreateDomain(CreateDomainStatement),
    CreateComposite(CreateCompositeStatement),
    CreateRange(CreateRangeStatement),
    CreateView(CreateViewStatement),
    CreateIndex(CreateIndexStatement),
    CreateSequence(CreateSequenceStatement),
    AlterSequence(AlterSequenceStatement),
    CreateFunction(CreateFunctionStatement),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectStatement {
    pub common_tables: Vec<CommonTableExpression>,
    pub recursive: bool,
    pub targets: Vec<SelectItem>,
    pub from: Vec<FromItem>,
    pub predicate: Option<Expression>,
    pub group_by: Vec<Expression>,
    pub having: Option<Expression>,
    pub order_by: Vec<OrderItem>,
    pub windows: Vec<NamedWindowDefinition>,
    pub limit: Option<Expression>,
    pub offset: Option<Expression>,
    pub locking: Vec<LockingClause>,
    pub values: Vec<Vec<Expression>>,
    pub set_operation: Option<SetOperation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedWindowDefinition {
    pub name: String,
    pub specification: WindowSpecification,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommonTableExpression {
    pub name: String,
    pub columns: Vec<String>,
    pub query: Box<SelectStatement>,
    pub materialization: CteMaterialization,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CteMaterialization {
    Default,
    Materialized,
    NotMaterialized,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockingClause {
    pub strength: LockStrength,
    pub relations: Vec<String>,
    pub wait: LockWait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockStrength {
    KeyShare,
    Share,
    NoKeyUpdate,
    Update,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockWait {
    Block,
    SkipLocked,
    NoWait,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectItem {
    pub expression: Expression,
    pub alias: Option<String>,
    pub span: SqlSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FromItem {
    Relation {
        name: Vec<String>,
        alias: Option<String>,
        span: SqlSpan,
    },
    Subquery {
        query: Box<SelectStatement>,
        alias: String,
        lateral: bool,
        span: SqlSpan,
    },
    Join {
        join: JoinKind,
        left: Box<FromItem>,
        right: Box<FromItem>,
        condition: Option<Expression>,
        using_columns: Vec<String>,
        span: SqlSpan,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetOperation {
    pub operator: SetOperator,
    pub all: bool,
    pub left: Box<SelectStatement>,
    pub right: Box<SelectStatement>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetOperator {
    Union,
    Intersect,
    Except,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderItem {
    pub expression: Expression,
    pub direction: OrderDirection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderDirection {
    Default,
    Ascending,
    Descending,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: SqlSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresTypeName {
    pub path: Vec<String>,
    pub modifiers: Vec<i64>,
    pub array_dimensions: u8,
}

impl PostgresTypeName {
    #[must_use]
    pub fn display(&self) -> String {
        let mut value = self.path.join(".");
        if !self.modifiers.is_empty() {
            value.push('(');
            value.push_str(
                &self
                    .modifiers
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
            value.push(')');
        }
        for _ in 0..self.array_dimensions {
            value.push_str("[]");
        }
        value
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExpressionKind {
    Star {
        qualifier: Vec<String>,
    },
    Column {
        path: Vec<String>,
    },
    Parameter {
        number: u32,
    },
    Integer {
        value: String,
    },
    Float {
        value: String,
    },
    String {
        value: String,
    },
    Boolean {
        value: bool,
    },
    Null,
    Cast {
        expression: Box<Expression>,
        ty: PostgresTypeName,
    },
    Binary {
        operator: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    InList {
        expression: Box<Expression>,
        values: Vec<Expression>,
        negated: bool,
    },
    Unary {
        operator: String,
        expression: Box<Expression>,
    },
    BooleanList {
        and: bool,
        expressions: Vec<Expression>,
    },
    Function {
        name: Vec<String>,
        arguments: Vec<Expression>,
        aggregate_star: bool,
        distinct: bool,
        filter: Option<Box<Expression>>,
        window: Option<WindowSpecification>,
    },
    Array {
        elements: Vec<Expression>,
    },
    Case {
        operand: Option<Box<Expression>>,
        branches: Vec<CaseBranch>,
        fallback: Option<Box<Expression>>,
    },
    Coalesce {
        arguments: Vec<Expression>,
    },
    NullTest {
        expression: Box<Expression>,
        is_not: bool,
    },
    Subquery {
        query: Box<SelectStatement>,
    },
    Exists {
        query: Box<SelectStatement>,
    },
    SubqueryComparison {
        operator: String,
        left: Box<Expression>,
        query: Box<SelectStatement>,
        quantifier: SubqueryQuantifier,
    },
    Default,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseBranch {
    pub condition: Expression,
    pub result: Expression,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowSpecification {
    pub reference: Option<String>,
    pub partition_by: Vec<Expression>,
    pub order_by: Vec<OrderItem>,
    pub frame_options: u32,
    pub start_offset: Option<Box<Expression>>,
    pub end_offset: Option<Box<Expression>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubqueryQuantifier {
    Any,
    All,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    pub column: String,
    pub value: Expression,
    pub span: SqlSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsertStatement {
    pub relation: Vec<String>,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Expression>>,
    pub source: Option<Box<SelectStatement>>,
    pub conflict: Option<ConflictClause>,
    pub returning: Vec<SelectItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConflictClause {
    pub action: ConflictAction,
    pub target_columns: Vec<String>,
    pub target_predicate: Option<Expression>,
    pub assignments: Vec<Assignment>,
    pub update_predicate: Option<Expression>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictAction {
    Nothing,
    Update,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateStatement {
    pub relation: Vec<String>,
    pub alias: Option<String>,
    pub assignments: Vec<Assignment>,
    pub from: Vec<FromItem>,
    pub predicate: Option<Expression>,
    pub returning: Vec<SelectItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteStatement {
    pub relation: Vec<String>,
    pub alias: Option<String>,
    pub using_relations: Vec<FromItem>,
    pub predicate: Option<Expression>,
    pub returning: Vec<SelectItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTableStatement {
    pub name: Vec<String>,
    pub columns: Vec<ColumnDefinition>,
    pub constraints: Vec<TableConstraint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColumnDefinition {
    pub name: String,
    pub ty: PostgresTypeName,
    pub nullable: bool,
    pub has_default: bool,
    pub generated: bool,
    pub identity_generation: Option<String>,
    pub primary_key: bool,
    pub unique: bool,
    pub references: Option<(Vec<String>, Vec<String>)>,
    pub checks: Vec<Expression>,
    pub span: SqlSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TableConstraint {
    PrimaryKey {
        columns: Vec<String>,
    },
    Unique {
        columns: Vec<String>,
    },
    ForeignKey {
        columns: Vec<String>,
        relation: Vec<String>,
        referenced: Vec<String>,
    },
    Check {
        expression: Expression,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateEnumStatement {
    pub name: Vec<String>,
    pub values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDomainStatement {
    pub name: Vec<String>,
    pub base_type: PostgresTypeName,
    pub nullable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateCompositeStatement {
    pub name: Vec<String>,
    pub attributes: Vec<ColumnDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRangeStatement {
    pub name: Vec<String>,
    pub subtype: PostgresTypeName,
    pub multirange_name: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateViewStatement {
    pub name: Vec<String>,
    pub query: SelectStatement,
    pub materialized: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateIndexStatement {
    pub name: String,
    pub relation: Vec<String>,
    pub columns: Vec<String>,
    pub unique: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSequenceStatement {
    pub name: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlterSequenceStatement {
    pub name: Vec<String>,
    pub owned_by: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateFunctionStatement {
    pub name: Vec<String>,
    pub arguments: Vec<PostgresTypeName>,
    pub result: PostgresTypeName,
    pub strict: bool,
    pub aggregate: bool,
}
