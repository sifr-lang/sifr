use crate::ast::{CreateTableStatement, PostgresStatement, StatementKind, TableConstraint};
use crate::ddl_constraints::{TableConstraintInput, add_table_constraints};
use crate::diagnostic::{PostgresDiagnostic, PostgresDiagnosticCode};
use crate::types::PostgresTypeRegistry;
use serde::{Deserialize, Serialize};
use sifr_sql_contract::{
    DatabaseType, ObjectId, SchemaDocument, SchemaDocumentKind, SchemaIr, SchemaObject,
    SchemaObjectKind, SchemaSourceLocation, SemanticValue,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogColumn {
    pub identity: ObjectId,
    pub name: String,
    pub database_type: DatabaseType,
    pub nullable: bool,
    pub has_default: bool,
    pub generated: bool,
    pub source: Option<SchemaSourceLocation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogRelation {
    pub identity: ObjectId,
    pub columns: BTreeMap<String, CatalogColumn>,
    pub primary_key: BTreeSet<String>,
    pub unique_sets: BTreeSet<Vec<String>>,
    pub source: Option<SchemaSourceLocation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogFunction {
    pub identity: ObjectId,
    pub arguments: Vec<DatabaseType>,
    pub result: DatabaseType,
    pub strict: bool,
    pub aggregate: bool,
    pub result_nullable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogOperator {
    pub identity: ObjectId,
    pub name: String,
    pub left: DatabaseType,
    pub right: DatabaseType,
    pub result: DatabaseType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogCast {
    pub source: DatabaseType,
    pub target: DatabaseType,
    pub implicit: bool,
}

#[derive(Clone, Debug)]
pub struct PostgresCatalog {
    pub schema_fingerprint: String,
    pub types: PostgresTypeRegistry,
    relations: BTreeMap<ObjectId, CatalogRelation>,
    functions: BTreeMap<String, Vec<CatalogFunction>>,
    operators: BTreeMap<String, Vec<CatalogOperator>>,
    casts: Vec<CatalogCast>,
    objects: BTreeMap<ObjectId, SchemaObject>,
}

impl PostgresCatalog {
    pub fn from_schema(
        schema: &SchemaIr,
        mut types: PostgresTypeRegistry,
    ) -> Result<Self, PostgresDiagnostic> {
        let mut relations = BTreeMap::new();
        let mut columns = BTreeMap::<ObjectId, CatalogColumn>::new();
        let mut functions = BTreeMap::<String, Vec<CatalogFunction>>::new();
        let mut operators = BTreeMap::<String, Vec<CatalogOperator>>::new();
        let mut casts = Vec::new();
        for object in schema.objects.values() {
            match object.kind {
                SchemaObjectKind::Enum => types.add_nominal(
                    &split_identity(&object.identity),
                    DatabaseType::Enum {
                        identity: object.identity.clone(),
                    },
                ),
                SchemaObjectKind::Domain => {
                    let base = database_type_property(object, "base-database-type")?;
                    types.add_nominal(
                        &split_identity(&object.identity),
                        DatabaseType::Domain {
                            identity: object.identity.clone(),
                            base: Box::new(base),
                        },
                    );
                }
                SchemaObjectKind::Composite => types.add_nominal(
                    &split_identity(&object.identity),
                    DatabaseType::Composite {
                        identity: object.identity.clone(),
                    },
                ),
                SchemaObjectKind::Column => {
                    let name = text_property(object, "name")?.to_string();
                    columns.insert(
                        object.identity.clone(),
                        CatalogColumn {
                            identity: object.identity.clone(),
                            name,
                            database_type: database_type_property(object, "database-type")?,
                            nullable: bool_property(object, "nullable")?,
                            has_default: optional_bool_property(object, "has-default")
                                .unwrap_or(false),
                            generated: optional_bool_property(object, "generated").unwrap_or(false),
                            source: object.source.clone(),
                        },
                    );
                }
                SchemaObjectKind::Function => {
                    let function = function_from_object(object)?;
                    functions
                        .entry(last_segment(&object.identity).to_ascii_lowercase())
                        .or_default()
                        .push(function);
                }
                SchemaObjectKind::Operator => {
                    let operator = operator_from_object(object)?;
                    operators
                        .entry(operator.name.clone())
                        .or_default()
                        .push(operator);
                }
                SchemaObjectKind::Cast => casts.push(CatalogCast {
                    source: database_type_property(object, "source-database-type")?,
                    target: database_type_property(object, "target-database-type")?,
                    implicit: optional_bool_property(object, "implicit").unwrap_or(false),
                }),
                _ => {}
            }
        }
        for object in schema.objects.values().filter(|object| {
            matches!(
                object.kind,
                SchemaObjectKind::Table
                    | SchemaObjectKind::View
                    | SchemaObjectKind::MaterializedView
            )
        }) {
            let column_ids = object_id_list_property(object, "columns")?;
            let relation_columns = column_ids
                .into_iter()
                .map(|identity| {
                    columns
                        .get(&identity)
                        .cloned()
                        .map(|column| (column.name.clone(), column))
                        .ok_or_else(|| {
                            schema_error(object, "relation references an unknown column")
                        })
                })
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            relations.insert(
                object.identity.clone(),
                CatalogRelation {
                    identity: object.identity.clone(),
                    columns: relation_columns,
                    primary_key: string_set_property(object, "primary-key").unwrap_or_default(),
                    unique_sets: nested_string_set_property(object, "unique-sets")
                        .unwrap_or_default(),
                    source: object.source.clone(),
                },
            );
        }
        Ok(Self {
            schema_fingerprint: sifr_sql_contract::schema_fingerprint(schema)
                .map_err(|error| schema_error_message(error.to_string()))?
                .as_str()
                .to_string(),
            types,
            relations,
            functions,
            operators,
            casts,
            objects: schema.objects.clone(),
        })
    }

    pub fn relation(&self, name: &[String]) -> Result<&CatalogRelation, PostgresDiagnostic> {
        let requested = name.join(".");
        if name.len() > 1 {
            return self
                .relations
                .get(&ObjectId::new(requested.clone()))
                .ok_or_else(|| unknown_relation(&requested));
        }
        if let Some(public) = self
            .relations
            .get(&ObjectId::new(format!("public.{requested}")))
        {
            return Ok(public);
        }
        let suffix = format!(".{requested}");
        let mut matches = self.relations.values().filter(|relation| {
            relation.identity.as_str() == requested || relation.identity.as_str().ends_with(&suffix)
        });
        let Some(first) = matches.next() else {
            return Err(unknown_relation(&requested));
        };
        if matches.next().is_some() {
            return Err(PostgresDiagnostic::at_sql(
                PostgresDiagnosticCode::UnknownRelation,
                format!("PostgreSQL relation '{requested}' is ambiguous"),
                0,
                1,
            ));
        }
        Ok(first)
    }

    #[must_use]
    pub fn functions(&self, name: &[String]) -> &[CatalogFunction] {
        self.functions
            .get(
                &name
                    .last()
                    .map(|name| name.to_ascii_lowercase())
                    .unwrap_or_default(),
            )
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn operators(&self, name: &str) -> &[CatalogOperator] {
        self.operators
            .get(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn can_cast(&self, source: &DatabaseType, target: &DatabaseType, implicit: bool) -> bool {
        source == target
            || self.casts.iter().any(|cast| {
                &cast.source == source && &cast.target == target && (!implicit || cast.implicit)
            })
            || builtin_cast(source, target, implicit)
    }

    #[must_use]
    pub fn object(&self, identity: &ObjectId) -> Option<&SchemaObject> {
        self.objects.get(identity)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogSnapshot {
    pub document: String,
    pub objects: Vec<SchemaObject>,
}

impl CatalogSnapshot {
    #[must_use]
    pub fn into_document(self) -> SchemaDocument {
        SchemaDocument {
            kind: SchemaDocumentKind::ProviderMetadata,
            document: self.document,
            objects: self.objects,
        }
    }
}

pub(crate) fn ddl_document(
    document: impl Into<String>,
    statements: &[PostgresStatement],
    types: &PostgresTypeRegistry,
) -> Result<SchemaDocument, PostgresDiagnostic> {
    let document = document.into();
    let mut objects = BTreeMap::new();
    for statement in statements {
        match &statement.kind {
            StatementKind::CreateTable(table) => {
                add_table(&document, statement, table, types, &mut objects)?;
            }
            StatementKind::CreateEnum(value) => {
                add_namespace(&document, &value.name, &mut objects);
                let identity = ObjectId::new(qualified_name(&value.name));
                objects.insert(
                    identity.clone(),
                    SchemaObject {
                        identity,
                        kind: SchemaObjectKind::Enum,
                        semantic: BTreeMap::from([(
                            "values".to_string(),
                            SemanticValue::List(
                                value
                                    .values
                                    .iter()
                                    .cloned()
                                    .map(SemanticValue::Text)
                                    .collect(),
                            ),
                        )]),
                        dependencies: namespace_dependency(&value.name),
                        source: Some(source_location(&document, statement)),
                    },
                );
            }
            StatementKind::CreateDomain(value) => {
                add_namespace(&document, &value.name, &mut objects);
                let base = types.resolve(&value.base_type).ok_or_else(|| {
                    schema_error_message(format!(
                        "unknown PostgreSQL domain base type '{}'",
                        value.base_type.join(".")
                    ))
                })?;
                let identity = ObjectId::new(qualified_name(&value.name));
                objects.insert(
                    identity.clone(),
                    SchemaObject {
                        identity,
                        kind: SchemaObjectKind::Domain,
                        semantic: BTreeMap::from([
                            (
                                "base-database-type".to_string(),
                                database_value(&base.database_type)?,
                            ),
                            ("nullable".to_string(), SemanticValue::Bool(value.nullable)),
                        ]),
                        dependencies: namespace_dependency(&value.name),
                        source: Some(source_location(&document, statement)),
                    },
                );
            }
            StatementKind::CreateSequence(value) => {
                add_namespace(&document, &value.name, &mut objects);
                let identity = ObjectId::new(qualified_name(&value.name));
                objects.insert(
                    identity.clone(),
                    SchemaObject {
                        identity,
                        kind: SchemaObjectKind::Sequence,
                        semantic: BTreeMap::new(),
                        dependencies: namespace_dependency(&value.name),
                        source: Some(source_location(&document, statement)),
                    },
                );
            }
            StatementKind::CreateIndex(value) => {
                let relation = ObjectId::new(qualified_name(&value.relation));
                let identity = ObjectId::new(format!(
                    "{}.{}",
                    value
                        .relation
                        .first()
                        .map(String::as_str)
                        .unwrap_or("public"),
                    value.name
                ));
                objects.insert(
                    identity.clone(),
                    SchemaObject {
                        identity,
                        kind: SchemaObjectKind::Index,
                        semantic: BTreeMap::from([
                            ("columns".to_string(), string_list(&value.columns)),
                            ("unique".to_string(), SemanticValue::Bool(value.unique)),
                        ]),
                        dependencies: BTreeSet::from([relation]),
                        source: Some(source_location(&document, statement)),
                    },
                );
            }
            StatementKind::CreateView(value) => {
                add_namespace(&document, &value.name, &mut objects);
                let identity = ObjectId::new(qualified_name(&value.name));
                objects.insert(
                    identity.clone(),
                    SchemaObject {
                        identity,
                        kind: if value.materialized {
                            SchemaObjectKind::MaterializedView
                        } else {
                            SchemaObjectKind::View
                        },
                        semantic: BTreeMap::from([
                            ("columns".to_string(), SemanticValue::List(Vec::new())),
                            (
                                "provider-query".to_string(),
                                SemanticValue::Text("libpg-query-ast".to_string()),
                            ),
                        ]),
                        dependencies: namespace_dependency(&value.name),
                        source: Some(source_location(&document, statement)),
                    },
                );
            }
            StatementKind::CreateFunction(value) => {
                add_namespace(&document, &value.name, &mut objects);
                let arguments = value
                    .arguments
                    .iter()
                    .map(|name| {
                        types
                            .resolve(name)
                            .map(|ty| database_value(&ty.database_type))
                            .transpose()
                            .and_then(|ty| {
                                ty.ok_or_else(|| {
                                    schema_error_message("unknown function argument type")
                                })
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let result = types
                    .resolve(&value.result)
                    .ok_or_else(|| schema_error_message("unknown function result type"))?;
                let identity = ObjectId::new(qualified_name(&value.name));
                objects.insert(
                    identity.clone(),
                    SchemaObject {
                        identity,
                        kind: SchemaObjectKind::Function,
                        semantic: BTreeMap::from([
                            ("arguments".to_string(), SemanticValue::List(arguments)),
                            ("result".to_string(), database_value(&result.database_type)?),
                            ("strict".to_string(), SemanticValue::Bool(value.strict)),
                            (
                                "aggregate".to_string(),
                                SemanticValue::Bool(value.aggregate),
                            ),
                            ("result-nullable".to_string(), SemanticValue::Bool(true)),
                        ]),
                        dependencies: namespace_dependency(&value.name),
                        source: Some(source_location(&document, statement)),
                    },
                );
            }
            _ => {}
        }
    }
    Ok(SchemaDocument {
        kind: SchemaDocumentKind::SqlDdl,
        document,
        objects: objects.into_values().collect(),
    })
}

fn add_table(
    document: &str,
    statement: &PostgresStatement,
    table: &CreateTableStatement,
    types: &PostgresTypeRegistry,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), PostgresDiagnostic> {
    add_namespace(document, &table.name, objects);
    let table_identity = ObjectId::new(qualified_name(&table.name));
    let mut column_ids = Vec::new();
    let mut primary_key = BTreeSet::new();
    let mut unique_sets = BTreeSet::new();
    for column in &table.columns {
        let ty = types.resolve(&column.ty).ok_or_else(|| {
            schema_error_message(format!("unknown PostgreSQL type '{}'", column.ty.join(".")))
        })?;
        let identity = ObjectId::new(format!("{}.{}", table_identity, column.name));
        column_ids.push(identity.clone());
        if column.primary_key {
            primary_key.insert(column.name.clone());
        }
        if column.unique {
            unique_sets.insert(vec![column.name.clone()]);
        }
        let mut dependencies = BTreeSet::from([table_identity.clone()]);
        if let Some((relation, _)) = &column.references {
            dependencies.insert(ObjectId::new(qualified_name(relation)));
        }
        objects.insert(
            identity.clone(),
            SchemaObject {
                identity,
                kind: SchemaObjectKind::Column,
                semantic: BTreeMap::from([
                    ("name".to_string(), SemanticValue::Text(column.name.clone())),
                    (
                        "database-type".to_string(),
                        database_value(&ty.database_type)?,
                    ),
                    ("nullable".to_string(), SemanticValue::Bool(column.nullable)),
                    (
                        "has-default".to_string(),
                        SemanticValue::Bool(column.has_default),
                    ),
                    (
                        "generated".to_string(),
                        SemanticValue::Bool(column.generated),
                    ),
                ]),
                dependencies,
                source: Some(SchemaSourceLocation {
                    document: document.to_string(),
                    start: column.span.start,
                    end: column.span.end,
                }),
            },
        );
    }
    for constraint in &table.constraints {
        match constraint {
            TableConstraint::PrimaryKey { columns } => primary_key.extend(columns.iter().cloned()),
            TableConstraint::Unique { columns } => {
                unique_sets.insert(columns.clone());
            }
            TableConstraint::ForeignKey { .. } | TableConstraint::Check { .. } => {}
        }
    }
    let constraint_ids = add_table_constraints(
        &TableConstraintInput {
            document,
            statement,
            table,
            table_identity: &table_identity,
            column_ids: &column_ids,
            primary_key: &primary_key,
            unique_sets: &unique_sets,
        },
        objects,
    )?;
    objects.insert(
        table_identity.clone(),
        SchemaObject {
            identity: table_identity,
            kind: SchemaObjectKind::Table,
            semantic: BTreeMap::from([
                (
                    "columns".to_string(),
                    SemanticValue::List(
                        column_ids
                            .iter()
                            .map(|value| SemanticValue::Text(value.as_str().to_string()))
                            .collect(),
                    ),
                ),
                (
                    "primary-key".to_string(),
                    SemanticValue::Set(
                        primary_key
                            .iter()
                            .cloned()
                            .map(SemanticValue::Text)
                            .collect(),
                    ),
                ),
                (
                    "unique-sets".to_string(),
                    SemanticValue::List(unique_sets.iter().map(|set| string_list(set)).collect()),
                ),
                (
                    "constraints".to_string(),
                    SemanticValue::List(
                        constraint_ids
                            .iter()
                            .map(|identity| SemanticValue::Text(identity.as_str().to_string()))
                            .collect(),
                    ),
                ),
            ]),
            dependencies: namespace_dependency(&table.name),
            source: Some(source_location(document, statement)),
        },
    );
    Ok(())
}

fn add_namespace(document: &str, name: &[String], objects: &mut BTreeMap<ObjectId, SchemaObject>) {
    let namespace = if name.len() > 1 { &name[0] } else { "public" };
    let identity = ObjectId::new(namespace);
    objects.entry(identity.clone()).or_insert(SchemaObject {
        identity,
        kind: SchemaObjectKind::Namespace,
        semantic: BTreeMap::new(),
        dependencies: BTreeSet::new(),
        source: Some(SchemaSourceLocation {
            document: document.to_string(),
            start: 0,
            end: 0,
        }),
    });
}

fn namespace_dependency(name: &[String]) -> BTreeSet<ObjectId> {
    BTreeSet::from([ObjectId::new(if name.len() > 1 {
        name[0].clone()
    } else {
        "public".to_string()
    })])
}

fn qualified_name(name: &[String]) -> String {
    if name.len() > 1 {
        name.join(".")
    } else {
        format!(
            "public.{}",
            name.first().map(String::as_str).unwrap_or("unknown")
        )
    }
}

fn source_location(document: &str, statement: &PostgresStatement) -> SchemaSourceLocation {
    SchemaSourceLocation {
        document: document.to_string(),
        start: statement.span.start,
        end: statement.span.end,
    }
}

fn database_value(database_type: &DatabaseType) -> Result<SemanticValue, PostgresDiagnostic> {
    serde_json::to_string(database_type)
        .map(SemanticValue::Text)
        .map_err(|_| schema_error_message("cannot serialize PostgreSQL database type"))
}

fn database_type_property(
    object: &SchemaObject,
    key: &str,
) -> Result<DatabaseType, PostgresDiagnostic> {
    let value = text_property(object, key)?;
    serde_json::from_str(value).map_err(|_| schema_error(object, "invalid database type metadata"))
}

fn text_property<'a>(object: &'a SchemaObject, key: &str) -> Result<&'a str, PostgresDiagnostic> {
    match object.semantic.get(key) {
        Some(SemanticValue::Text(value)) => Ok(value),
        _ => Err(schema_error(
            object,
            format!("missing text property '{key}'"),
        )),
    }
}

fn bool_property(object: &SchemaObject, key: &str) -> Result<bool, PostgresDiagnostic> {
    optional_bool_property(object, key)
        .ok_or_else(|| schema_error(object, format!("missing boolean property '{key}'")))
}

fn optional_bool_property(object: &SchemaObject, key: &str) -> Option<bool> {
    match object.semantic.get(key) {
        Some(SemanticValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn object_id_list_property(
    object: &SchemaObject,
    key: &str,
) -> Result<Vec<ObjectId>, PostgresDiagnostic> {
    match object.semantic.get(key) {
        Some(SemanticValue::List(values)) => values
            .iter()
            .map(|value| match value {
                SemanticValue::Text(value) => Ok(ObjectId::new(value)),
                _ => Err(schema_error(object, "invalid object identity list")),
            })
            .collect(),
        _ => Err(schema_error(
            object,
            format!("missing list property '{key}'"),
        )),
    }
}

fn string_set_property(object: &SchemaObject, key: &str) -> Option<BTreeSet<String>> {
    match object.semantic.get(key) {
        Some(SemanticValue::Set(values)) => values
            .iter()
            .map(|value| match value {
                SemanticValue::Text(value) => Some(value.clone()),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

fn nested_string_set_property(object: &SchemaObject, key: &str) -> Option<BTreeSet<Vec<String>>> {
    match object.semantic.get(key) {
        Some(SemanticValue::List(values)) => values
            .iter()
            .map(|value| match value {
                SemanticValue::List(values) => values
                    .iter()
                    .map(|value| match value {
                        SemanticValue::Text(value) => Some(value.clone()),
                        _ => None,
                    })
                    .collect(),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

fn function_from_object(object: &SchemaObject) -> Result<CatalogFunction, PostgresDiagnostic> {
    let arguments = match object.semantic.get("arguments") {
        Some(SemanticValue::List(values)) => values
            .iter()
            .map(|value| match value {
                SemanticValue::Text(value) => serde_json::from_str(value)
                    .map_err(|_| schema_error(object, "invalid function argument type")),
                _ => Err(schema_error(object, "invalid function argument metadata")),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(schema_error(object, "function has no argument list")),
    };
    Ok(CatalogFunction {
        identity: object.identity.clone(),
        arguments,
        result: database_type_property(object, "result")?,
        strict: optional_bool_property(object, "strict").unwrap_or(false),
        aggregate: optional_bool_property(object, "aggregate").unwrap_or(false),
        result_nullable: optional_bool_property(object, "result-nullable").unwrap_or(true),
    })
}

fn operator_from_object(object: &SchemaObject) -> Result<CatalogOperator, PostgresDiagnostic> {
    Ok(CatalogOperator {
        identity: object.identity.clone(),
        name: text_property(object, "name")?.to_string(),
        left: database_type_property(object, "left")?,
        right: database_type_property(object, "right")?,
        result: database_type_property(object, "result")?,
    })
}

fn builtin_cast(source: &DatabaseType, target: &DatabaseType, implicit: bool) -> bool {
    if source == target {
        return true;
    }
    matches!(
        (source, target, implicit),
        (
            DatabaseType::Integer { .. },
            DatabaseType::Decimal { .. } | DatabaseType::Float32 | DatabaseType::Float64,
            true
        ) | (
            DatabaseType::Integer { .. }
                | DatabaseType::Decimal { .. }
                | DatabaseType::Float32
                | DatabaseType::Float64,
            DatabaseType::Text { .. },
            false
        )
    )
}

fn split_identity(identity: &ObjectId) -> Vec<String> {
    identity.as_str().split('.').map(str::to_string).collect()
}

fn last_segment(identity: &ObjectId) -> &str {
    identity
        .as_str()
        .rsplit('.')
        .next()
        .unwrap_or(identity.as_str())
}

fn string_list(values: &[String]) -> SemanticValue {
    SemanticValue::List(values.iter().cloned().map(SemanticValue::Text).collect())
}

fn unknown_relation(name: &str) -> PostgresDiagnostic {
    PostgresDiagnostic::at_sql(
        PostgresDiagnosticCode::UnknownRelation,
        format!("unknown PostgreSQL relation '{name}'"),
        0,
        1,
    )
}

fn schema_error(object: &SchemaObject, message: impl Into<String>) -> PostgresDiagnostic {
    let mut diagnostic = schema_error_message(format!("{}: {}", object.identity, message.into()));
    if let Some(source) = &object.source {
        diagnostic = diagnostic.with_schema_span(source.document.clone(), source.start, source.end);
    }
    diagnostic
}

fn schema_error_message(message: impl Into<String>) -> PostgresDiagnostic {
    PostgresDiagnostic::at_sql(PostgresDiagnosticCode::TypeMismatch, message, 0, 1)
}
