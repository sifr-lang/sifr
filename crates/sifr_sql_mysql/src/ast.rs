use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqlSpan {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlStatement {
    pub kind: MysqlStatementKind,
    pub span: SqlSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MysqlStatementKind {
    Query(MysqlQuery),
    Insert(MysqlWrite),
    Update(MysqlWrite),
    Delete(MysqlWrite),
    CreateTable(MysqlCreateTable),
    CreateView(MysqlNamedDdl),
    CreateIndex(MysqlNamedDdl),
    AlterTable(MysqlNamedDdl),
    Drop(MysqlNamedDdl),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlQuery {
    pub common_tables: Vec<String>,
    pub projections: Vec<MysqlProjection>,
    pub relations: Vec<Vec<String>>,
    pub joins: Vec<Vec<String>>,
    pub predicate: Option<MysqlExpression>,
    pub group_by: Vec<MysqlExpression>,
    pub having: Option<MysqlExpression>,
    pub order_by: Vec<MysqlExpression>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub distinct: bool,
    pub windowed: bool,
    pub for_update: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlProjection {
    pub expression: MysqlExpression,
    pub alias: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MysqlExpression {
    Star {
        qualifier: Vec<String>,
    },
    Column {
        path: Vec<String>,
    },
    Parameter,
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
        parameters: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlWrite {
    pub relation: Vec<String>,
    pub columns: Vec<String>,
    pub assignments: Vec<String>,
    pub expressions: Vec<MysqlExpression>,
    pub conflict: MysqlConflictForm,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MysqlConflictForm {
    #[default]
    None,
    Ignore,
    Replace,
    OnDuplicateKeyUpdate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlCreateTable {
    pub name: Vec<String>,
    pub columns: Vec<MysqlColumnDefinition>,
    pub primary_key: Vec<String>,
    pub unique_keys: Vec<MysqlKeyDefinition>,
    pub foreign_keys: Vec<MysqlForeignKeyDefinition>,
    pub checks: Vec<String>,
    pub indexes: Vec<MysqlKeyDefinition>,
    pub default_character_set: Option<String>,
    pub default_collation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlColumnDefinition {
    pub name: String,
    pub ty: MysqlTypeName,
    pub nullable: bool,
    pub auto_increment: bool,
    pub generated: Option<MysqlGeneratedColumn>,
    pub default: Option<String>,
    pub character_set: Option<String>,
    pub collation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlGeneratedColumn {
    pub expression: String,
    pub stored: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlKeyDefinition {
    pub name: Option<String>,
    pub columns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlForeignKeyDefinition {
    pub name: Option<String>,
    pub columns: Vec<String>,
    pub referenced_table: Vec<String>,
    pub referenced_columns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlNamedDdl {
    pub name: Vec<String>,
    pub relation: Option<Vec<String>>,
    pub definition: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlTypeName {
    pub name: String,
    pub parameters: Vec<String>,
    pub unsigned: bool,
    pub zerofill: bool,
}
