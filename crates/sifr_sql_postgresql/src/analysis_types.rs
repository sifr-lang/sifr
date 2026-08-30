use crate::ast::Expression;
use crate::catalog::{CatalogColumn, PostgresCatalog};
use crate::diagnostic::{PostgresDiagnostic, PostgresDiagnosticCode};
use crate::raw_adapter::PostgresParseError;
use sifr_sql_contract::{Cardinality, DatabaseType, ObjectId, QueryEffect};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresAnalysisError {
    pub diagnostic: PostgresDiagnostic,
}

impl PostgresAnalysisError {
    pub(crate) fn new(
        code: PostgresDiagnosticCode,
        message: impl Into<String>,
        expression: &Expression,
    ) -> Self {
        Self {
            diagnostic: PostgresDiagnostic::at_sql(
                code,
                message,
                expression.span.start,
                expression.span.end,
            ),
        }
    }

    pub(crate) fn at_start(code: PostgresDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            diagnostic: PostgresDiagnostic::at_sql(code, message, 0, 1),
        }
    }

    pub(crate) fn with_sifr_span(mut self, document: &str, start: u32, end: u32) -> Self {
        self.diagnostic = self
            .diagnostic
            .with_sifr_span(document.to_string(), start, end);
        self
    }
}

impl fmt::Display for PostgresAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.diagnostic.message)
    }
}

impl std::error::Error for PostgresAnalysisError {}

impl From<PostgresParseError> for PostgresAnalysisError {
    fn from(value: PostgresParseError) -> Self {
        Self {
            diagnostic: value.diagnostic,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ScopeBinding {
    pub(crate) alias: String,
    pub(crate) relation: Option<ObjectId>,
    pub(crate) columns: BTreeMap<String, CatalogColumn>,
    pub(crate) column_order: Vec<String>,
}

impl ScopeBinding {
    pub(crate) fn derived(alias: &str, names: Vec<String>, fields: Vec<ResultFact>) -> Self {
        Self {
            alias: alias.to_string(),
            relation: None,
            column_order: names.clone(),
            columns: names
                .into_iter()
                .zip(fields)
                .enumerate()
                .map(|(index, (name, field))| {
                    let identity = ObjectId::new(format!("derived.{alias}.{index}"));
                    (
                        name.clone(),
                        CatalogColumn {
                            identity,
                            name,
                            database_type: field.database_type,
                            nullable: field.nullable,
                            has_default: false,
                            generated: false,
                            source: None,
                        },
                    )
                })
                .collect(),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct ScopeFrame {
    pub(crate) bindings: Vec<ScopeBinding>,
}

#[derive(Clone)]
pub(crate) struct TypeFact {
    pub(crate) database_type: DatabaseType,
    pub(crate) nullable: bool,
    pub(crate) source_object: Option<ObjectId>,
    pub(crate) name_hint: Option<String>,
}

#[derive(Clone)]
pub(crate) struct ResultFact {
    pub(crate) name: String,
    pub(crate) database_type: DatabaseType,
    pub(crate) nullable: bool,
    pub(crate) source_object: Option<ObjectId>,
}

pub(crate) struct AnalyzedStatement {
    pub(crate) fields: Vec<ResultFact>,
    pub(crate) cardinality: Cardinality,
    pub(crate) effect: QueryEffect,
    pub(crate) referenced: BTreeSet<ObjectId>,
    pub(crate) affected: BTreeSet<ObjectId>,
    pub(crate) flags: BTreeSet<String>,
}

pub(crate) struct AnalysisContext<'a> {
    pub(crate) catalog: &'a PostgresCatalog,
    pub(crate) parameters: BTreeMap<u32, DatabaseType>,
    pub(crate) referenced: BTreeSet<ObjectId>,
}
