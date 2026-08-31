use crate::ast::{
    SqliteExpression, SqliteProjection, SqliteQuery, SqliteStatementKind, SqliteWrite,
};
use crate::codec::sqlite_codec_registry_for_types;
use crate::diagnostic::{SqliteDiagnostic, SqliteDiagnosticCode};
use crate::parser::SqliteParser;
use sifr_sql_contract::{
    Cardinality, CodecRegistry, DatabaseType, EffectContract, Nullability, ObjectId,
    ProviderAnalysis, ProviderParameter, ProviderResultField, QueryEffect, SchemaIr,
    SchemaObjectKind, SemanticValue, canonical_read_type_with_nullability_in,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
struct SqliteColumn {
    name: String,
    identity: ObjectId,
    database_type: DatabaseType,
    nullable: bool,
}

#[derive(Clone, Debug)]
struct SqliteRelation {
    identity: ObjectId,
    columns: BTreeMap<String, SqliteColumn>,
}

#[derive(Clone, Debug)]
struct SqliteCatalog {
    relations: BTreeMap<ObjectId, SqliteRelation>,
    codecs: CodecRegistry,
}

pub struct SqliteAnalyzer<'a> {
    parser: &'a SqliteParser,
    catalog: SqliteCatalog,
}

impl<'a> SqliteAnalyzer<'a> {
    pub fn new(parser: &'a SqliteParser, schema: &SchemaIr) -> Result<Self, SqliteDiagnostic> {
        if schema.dialect.family != "sqlite"
            || schema.dialect.server_version != parser.series().version()
            || schema.dialect.modes != *parser.compile_flags()
        {
            return Err(diagnostic(
                SqliteDiagnosticCode::UnsupportedMode,
                "SQLite schema version, compile flags, and parser identity differ",
            ));
        }
        let mut relations = BTreeMap::new();
        let mut database_types = Vec::new();
        for (identity, object) in &schema.objects {
            if object.kind != SchemaObjectKind::Table && object.kind != SchemaObjectKind::View {
                continue;
            }
            let prefix = format!("{}.", identity.as_str());
            let mut columns = BTreeMap::new();
            for (column_identity, column) in &schema.objects {
                if !column_identity.as_str().starts_with(&prefix)
                    || !matches!(
                        column.kind,
                        SchemaObjectKind::Column | SchemaObjectKind::IdentityColumn
                    )
                {
                    continue;
                }
                let database_type = text_property(column, "database-type")
                    .and_then(|value| serde_json::from_str::<DatabaseType>(value).ok())
                    .ok_or_else(|| {
                        diagnostic(
                            SqliteDiagnosticCode::InvalidSchema,
                            format!("SQLite column '{column_identity}' has no canonical type"),
                        )
                    })?;
                let name = text_property(column, "name")
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        column_identity
                            .as_str()
                            .rsplit('.')
                            .next()
                            .unwrap_or("unknown")
                            .to_string()
                    });
                let nullable = bool_property(column, "nullable").unwrap_or(true);
                database_types.push(database_type.clone());
                columns.insert(
                    name.clone(),
                    SqliteColumn {
                        name,
                        identity: column_identity.clone(),
                        database_type,
                        nullable,
                    },
                );
            }
            relations.insert(
                identity.clone(),
                SqliteRelation {
                    identity: identity.clone(),
                    columns,
                },
            );
        }
        database_types.push(default_parameter_type());
        let codecs =
            sqlite_codec_registry_for_types(parser.series(), database_types).map_err(|error| {
                diagnostic(SqliteDiagnosticCode::ProviderContract, error.to_string())
            })?;
        Ok(Self {
            parser,
            catalog: SqliteCatalog { relations, codecs },
        })
    }

    pub fn analyze_query(&self, source: &str) -> Result<ProviderAnalysis, SqliteDiagnostic> {
        let statements = self.parser.parse(source).map_err(|error| {
            SqliteDiagnostic::at_sql(
                SqliteDiagnosticCode::Syntax,
                error.message,
                u32::try_from(error.offset).unwrap_or(u32::MAX),
                u32::try_from(error.offset.saturating_add(1)).unwrap_or(u32::MAX),
            )
        })?;
        if statements.len() != 1 {
            return Err(diagnostic(
                SqliteDiagnosticCode::Syntax,
                "a Sifr SQLite template must contain exactly one statement",
            ));
        }
        let statement = statements.into_iter().next().ok_or_else(|| {
            diagnostic(SqliteDiagnosticCode::Syntax, "SQLite statement is missing")
        })?;
        let normalized_statement = self
            .parser
            .normalize(source)
            .map_err(|error| diagnostic(SqliteDiagnosticCode::Syntax, error.message))?;
        let mut analysis = match statement.kind {
            SqliteStatementKind::Query(query) => self.analyze_select(&query)?,
            SqliteStatementKind::Insert(write) => {
                self.analyze_write(&write, QueryEffect::Write, "sql.query.insert")?
            }
            SqliteStatementKind::Update(write) => {
                self.analyze_write(&write, QueryEffect::Write, "sql.query.update")?
            }
            SqliteStatementKind::Delete(write) => {
                self.analyze_write(&write, QueryEffect::Write, "sql.query.delete")?
            }
            _ => {
                return Err(diagnostic(
                    SqliteDiagnosticCode::UnsupportedFeature,
                    "application query templates cannot execute SQLite DDL",
                ));
            }
        };
        analysis.normalized_statement = normalized_statement;
        analysis.validate(&self.catalog.codecs).map_err(|error| {
            diagnostic(SqliteDiagnosticCode::ProviderContract, format!("{error:?}"))
        })?;
        Ok(analysis)
    }

    fn analyze_select(&self, query: &SqliteQuery) -> Result<ProviderAnalysis, SqliteDiagnostic> {
        let mut scope = Vec::new();
        for relation in query.relations.iter().chain(&query.joins) {
            scope.push(self.resolve_relation(relation)?);
        }
        let referenced_objects = scope
            .iter()
            .map(|relation| relation.identity.clone())
            .collect::<BTreeSet<_>>();
        let mut accessed_objects = referenced_objects.clone();
        let mut result_fields = Vec::new();
        let mut parameter_types = Vec::new();
        for (index, projection) in query.projections.iter().enumerate() {
            self.analyze_projection(
                projection,
                index,
                &scope,
                &mut accessed_objects,
                &mut parameter_types,
                &mut result_fields,
            )?;
        }
        for expression in query
            .predicate
            .iter()
            .chain(&query.group_by)
            .chain(query.having.iter())
            .chain(&query.order_by)
        {
            Self::account_expression(
                expression,
                &scope,
                &mut accessed_objects,
                &mut parameter_types,
            )?;
        }
        let mut required_capabilities = BTreeSet::from(["sql.query.select".to_string()]);
        if !parameter_types.is_empty() {
            required_capabilities.insert("sql.bind.parameters".to_string());
        }
        if !query.joins.is_empty() {
            required_capabilities.insert("sql.query.join".to_string());
        }
        if !query.common_tables.is_empty() {
            required_capabilities.insert("sql.query.common-table-expression".to_string());
        }
        if query.windowed {
            required_capabilities.insert("sql.query.window".to_string());
        }
        if query.for_update {
            required_capabilities.insert("sql.query.row-locking".to_string());
        }
        if !query.group_by.is_empty() || query.having.is_some() {
            required_capabilities.insert("sql.query.aggregate".to_string());
        }
        let cardinality = if query.limit == Some(1) {
            Cardinality::AT_MOST_ONE
        } else {
            Cardinality::MANY
        };
        Ok(ProviderAnalysis {
            server_profile: self.parser.series().profile(),
            normalized_statement: "SELECT".to_string(),
            parameters: self.parameters(parameter_types)?,
            result_fields,
            cardinality,
            effects: EffectContract::new(QueryEffect::Read, referenced_objects, BTreeSet::new())
                .map_err(|error| {
                    diagnostic(SqliteDiagnosticCode::ProviderContract, error.to_string())
                })?,
            accessed_objects,
            semantic_flags: BTreeSet::from([
                "sqlite-dynamic-storage-class".to_string(),
                "provider-owned-object-account".to_string(),
            ]),
            required_capabilities,
        })
    }

    fn analyze_write(
        &self,
        write: &SqliteWrite,
        effect: QueryEffect,
        capability: &str,
    ) -> Result<ProviderAnalysis, SqliteDiagnostic> {
        let relation = self.resolve_relation(&write.relation)?;
        let mut accessed_objects = BTreeSet::from([relation.identity.clone()]);
        for column in write.columns.iter().chain(&write.assignments) {
            let column = relation.columns.get(column).ok_or_else(|| {
                diagnostic(
                    SqliteDiagnosticCode::UnknownColumn,
                    format!("unknown SQLite write column '{column}'"),
                )
            })?;
            accessed_objects.insert(column.identity.clone());
        }
        let mut parameter_types = Vec::new();
        for expression in &write.expressions {
            Self::account_expression(
                expression,
                &[relation],
                &mut accessed_objects,
                &mut parameter_types,
            )?;
        }
        let mut required_capabilities = BTreeSet::from([capability.to_string()]);
        if !parameter_types.is_empty() {
            required_capabilities.insert("sql.bind.parameters".to_string());
        }
        if !matches!(write.conflict, crate::ast::SqliteConflictForm::None) {
            required_capabilities.insert("sql.sqlite.write.conflict".to_string());
        }
        Ok(ProviderAnalysis {
            server_profile: self.parser.series().profile(),
            normalized_statement: capability.to_string(),
            parameters: self.parameters(parameter_types)?,
            result_fields: Vec::new(),
            cardinality: Cardinality::ZERO,
            effects: EffectContract::new(
                effect,
                BTreeSet::from([relation.identity.clone()]),
                BTreeSet::from([relation.identity.clone()]),
            )
            .map_err(|error| {
                diagnostic(SqliteDiagnosticCode::ProviderContract, error.to_string())
            })?,
            accessed_objects,
            semantic_flags: BTreeSet::from(["provider-owned-object-account".to_string()]),
            required_capabilities,
        })
    }

    fn analyze_projection(
        &self,
        projection: &SqliteProjection,
        index: usize,
        scope: &[&SqliteRelation],
        accessed: &mut BTreeSet<ObjectId>,
        parameters: &mut Vec<DatabaseType>,
        fields: &mut Vec<ProviderResultField>,
    ) -> Result<(), SqliteDiagnostic> {
        match &projection.expression {
            SqliteExpression::Star { .. } => {
                for relation in scope {
                    for column in relation.columns.values() {
                        fields.push(self.result_field(column.name.clone(), column)?);
                        accessed.insert(column.identity.clone());
                    }
                }
            }
            SqliteExpression::Column { path } => {
                let column = Self::resolve_column(scope, path)?;
                fields.push(
                    self.result_field(
                        projection
                            .alias
                            .clone()
                            .unwrap_or_else(|| column.name.clone()),
                        column,
                    )?,
                );
                accessed.insert(column.identity.clone());
            }
            expression => {
                Self::account_expression(expression, scope, accessed, parameters)?;
                let database_type = expression_type(expression, scope)?;
                let nullability = Nullability::Nullable;
                let codec = self.codec(&database_type)?;
                let sifr_type = canonical_read_type_with_nullability_in(
                    &database_type,
                    nullability,
                    &self.catalog.codecs,
                )
                .map_err(|error| {
                    diagnostic(SqliteDiagnosticCode::TypeMismatch, error.to_string())
                })?;
                fields.push(ProviderResultField {
                    name: projection
                        .alias
                        .clone()
                        .unwrap_or_else(|| format!("field_{}", index + 1)),
                    sifr_type,
                    database_type,
                    nullability,
                    codec,
                    source_object: None,
                });
            }
        }
        Ok(())
    }

    fn account_expression(
        expression: &SqliteExpression,
        scope: &[&SqliteRelation],
        accessed: &mut BTreeSet<ObjectId>,
        parameters: &mut Vec<DatabaseType>,
    ) -> Result<(), SqliteDiagnostic> {
        match expression {
            SqliteExpression::Column { path } => {
                accessed.insert(Self::resolve_column(scope, path)?.identity.clone());
            }
            SqliteExpression::Parameter => parameters.push(default_parameter_type()),
            SqliteExpression::Function { arguments, .. } => {
                for argument in arguments {
                    Self::account_expression(argument, scope, accessed, parameters)?;
                }
            }
            SqliteExpression::Binary { left, right, .. } => {
                Self::account_expression(left, scope, accessed, parameters)?;
                Self::account_expression(right, scope, accessed, parameters)?;
            }
            SqliteExpression::Raw {
                columns,
                parameters: count,
                ..
            } => {
                let inferred = columns
                    .first()
                    .and_then(|path| Self::resolve_column(scope, path).ok())
                    .map_or_else(default_parameter_type, |column| {
                        column.database_type.clone()
                    });
                for path in columns {
                    accessed.insert(Self::resolve_column(scope, path)?.identity.clone());
                }
                parameters.extend(std::iter::repeat_n(inferred, *count as usize));
            }
            SqliteExpression::Star { .. } | SqliteExpression::Literal { .. } => {}
        }
        Ok(())
    }

    fn parameters(
        &self,
        database_types: Vec<DatabaseType>,
    ) -> Result<Vec<ProviderParameter>, SqliteDiagnostic> {
        database_types
            .into_iter()
            .enumerate()
            .map(|(slot, database_type)| {
                Ok(ProviderParameter {
                    slot: u32::try_from(slot).map_err(|_| {
                        diagnostic(
                            SqliteDiagnosticCode::TypeMismatch,
                            "too many SQLite parameters",
                        )
                    })?,
                    codec: self.codec(&database_type)?,
                    database_type,
                    nullability: Nullability::Nullable,
                })
            })
            .collect()
    }

    fn result_field(
        &self,
        name: String,
        column: &SqliteColumn,
    ) -> Result<ProviderResultField, SqliteDiagnostic> {
        let nullability = if column.nullable {
            Nullability::Nullable
        } else {
            Nullability::NonNull
        };
        Ok(ProviderResultField {
            name,
            sifr_type: canonical_read_type_with_nullability_in(
                &column.database_type,
                nullability,
                &self.catalog.codecs,
            )
            .map_err(|error| diagnostic(SqliteDiagnosticCode::TypeMismatch, error.to_string()))?,
            database_type: column.database_type.clone(),
            nullability,
            codec: self.codec(&column.database_type)?,
            source_object: Some(column.identity.clone()),
        })
    }

    fn codec(
        &self,
        database_type: &DatabaseType,
    ) -> Result<sifr_sql_contract::CodecIdentity, SqliteDiagnostic> {
        self.catalog
            .codecs
            .codec_for_database_type(database_type)
            .map(|codec| codec.identity.clone())
            .ok_or_else(|| {
                diagnostic(
                    SqliteDiagnosticCode::TypeMismatch,
                    "SQLite type has no qualified codec",
                )
            })
    }

    fn resolve_relation(&self, path: &[String]) -> Result<&SqliteRelation, SqliteDiagnostic> {
        let joined = path.join(".");
        if let Some(relation) = self.catalog.relations.get(&ObjectId::new(&joined)) {
            return Ok(relation);
        }
        let suffix = format!(".{joined}");
        let matches = self
            .catalog
            .relations
            .iter()
            .filter(|(identity, _)| identity.as_str().ends_with(&suffix))
            .map(|(_, relation)| relation)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [relation] => Ok(*relation),
            [] => Err(diagnostic(
                SqliteDiagnosticCode::UnknownObject,
                format!("unknown SQLite relation '{joined}'"),
            )),
            _ => Err(diagnostic(
                SqliteDiagnosticCode::UnknownObject,
                format!("ambiguous SQLite relation '{joined}'"),
            )),
        }
    }

    fn resolve_column<'b>(
        scope: &'b [&SqliteRelation],
        path: &[String],
    ) -> Result<&'b SqliteColumn, SqliteDiagnostic> {
        let Some(name) = path.last() else {
            return Err(diagnostic(
                SqliteDiagnosticCode::UnknownColumn,
                "empty column path",
            ));
        };
        let qualifier = (path.len() > 1).then(|| path[..path.len() - 1].join("."));
        let matches = scope
            .iter()
            .filter(|relation| {
                qualifier.as_ref().is_none_or(|qualifier| {
                    relation.identity.as_str() == qualifier
                        || relation
                            .identity
                            .as_str()
                            .ends_with(&format!(".{qualifier}"))
                })
            })
            .filter_map(|relation| relation.columns.get(name))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [column] => Ok(*column),
            [] => Err(diagnostic(
                SqliteDiagnosticCode::UnknownColumn,
                format!("unknown SQLite column '{}'", path.join(".")),
            )),
            _ => Err(diagnostic(
                SqliteDiagnosticCode::AmbiguousColumn,
                format!("ambiguous SQLite column '{name}'"),
            )),
        }
    }
}

fn expression_type(
    expression: &SqliteExpression,
    scope: &[&SqliteRelation],
) -> Result<DatabaseType, SqliteDiagnostic> {
    match expression {
        SqliteExpression::Column { path } => Ok(SqliteAnalyzer::resolve_column(scope, path)?
            .database_type
            .clone()),
        SqliteExpression::Parameter
        | SqliteExpression::Literal { .. }
        | SqliteExpression::Raw { .. } => Ok(default_parameter_type()),
        SqliteExpression::Function { .. } | SqliteExpression::Binary { .. } => {
            Ok(default_parameter_type())
        }
        SqliteExpression::Star { .. } => Err(diagnostic(
            SqliteDiagnosticCode::TypeMismatch,
            "star has no scalar SQLite type",
        )),
    }
}

fn default_parameter_type() -> DatabaseType {
    DatabaseType::Text {
        fixed: false,
        max_characters: None,
    }
}

fn text_property<'a>(object: &'a sifr_sql_contract::SchemaObject, name: &str) -> Option<&'a str> {
    match object.semantic.get(name) {
        Some(SemanticValue::Text(value)) => Some(value),
        _ => None,
    }
}

fn bool_property(object: &sifr_sql_contract::SchemaObject, name: &str) -> Option<bool> {
    match object.semantic.get(name) {
        Some(SemanticValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn diagnostic(code: SqliteDiagnosticCode, message: impl Into<String>) -> SqliteDiagnostic {
    SqliteDiagnostic::at_sql(code, message, 0, 1)
}
