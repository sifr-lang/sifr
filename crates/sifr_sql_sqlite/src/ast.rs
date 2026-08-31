use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqlSpan {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteStatement {
    pub kind: SqliteStatementKind,
    pub span: SqlSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SqliteStatementKind {
    Query(SqliteQuery),
    Insert(SqliteWrite),
    Update(SqliteWrite),
    Delete(SqliteWrite),
    CreateTable(SqliteCreateTable),
    CreateView(SqliteNamedDdl),
    CreateIndex(SqliteNamedDdl),
    CreateTrigger(SqliteNamedDdl),
    AlterTable(SqliteNamedDdl),
    Drop(SqliteNamedDdl),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteQuery {
    pub common_tables: Vec<String>,
    pub projections: Vec<SqliteProjection>,
    pub relations: Vec<Vec<String>>,
    pub joins: Vec<Vec<String>>,
    pub predicate: Option<SqliteExpression>,
    pub group_by: Vec<SqliteExpression>,
    pub having: Option<SqliteExpression>,
    pub order_by: Vec<SqliteExpression>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub distinct: bool,
    pub windowed: bool,
    pub for_update: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteProjection {
    pub expression: SqliteExpression,
    pub alias: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SqliteExpression {
    Star {
        qualifier: Vec<String>,
    },
    Column {
        path: Vec<String>,
    },
    Parameter {
        marker: String,
    },
    Literal {
        value: String,
    },
    Function {
        name: String,
        arguments: Vec<Self>,
    },
    Binary {
        operator: String,
        left: Box<Self>,
        right: Box<Self>,
    },
    Raw {
        normalized: String,
        columns: Vec<Vec<String>>,
        parameters: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteWrite {
    pub relation: Vec<String>,
    pub columns: Vec<String>,
    pub assignments: Vec<String>,
    pub expressions: Vec<SqliteExpression>,
    pub conflict: SqliteConflictForm,
    pub returning: Vec<SqliteProjection>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqliteConflictForm {
    #[default]
    None,
    Ignore,
    Replace,
    Rollback,
    Abort,
    Fail,
    UpsertDoNothing,
    UpsertDoUpdate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteCreateTable {
    pub name: Vec<String>,
    pub columns: Vec<SqliteColumnDefinition>,
    pub primary_key: Vec<String>,
    pub unique_keys: Vec<SqliteKeyDefinition>,
    pub foreign_keys: Vec<SqliteForeignKeyDefinition>,
    pub checks: Vec<String>,
    pub indexes: Vec<SqliteKeyDefinition>,
    pub strict: bool,
    pub without_rowid: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteColumnDefinition {
    pub name: String,
    pub ty: SqliteTypeName,
    pub nullable: bool,
    pub primary_key: bool,
    pub primary_key_desc: bool,
    pub auto_increment: bool,
    pub generated: Option<SqliteGeneratedColumn>,
    pub default: Option<String>,
    pub collation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteGeneratedColumn {
    pub expression: String,
    pub stored: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteKeyDefinition {
    pub name: Option<String>,
    pub columns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteForeignKeyDefinition {
    pub name: Option<String>,
    pub columns: Vec<String>,
    pub referenced_table: Vec<String>,
    pub referenced_columns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteNamedDdl {
    pub name: Vec<String>,
    pub relation: Option<Vec<String>>,
    pub definition: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteTypeName {
    pub name: String,
    pub parameters: Vec<String>,
}
