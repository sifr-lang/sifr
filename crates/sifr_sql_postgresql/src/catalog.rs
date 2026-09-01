use crate::analysis::AnalysisContext;
use crate::ast::{CreateTableStatement, PostgresStatement, StatementKind, TableConstraint};
use crate::catalog_metadata::{
    enum_semantics, function_from_object, nested_string_set_property, operator_from_object,
    string_set_property,
};
use crate::catalog_semantics::{
    bool_property, database_type_property, database_value, last_segment, object_id_list_property,
    optional_bool_property, schema_error, schema_error_message, split_identity, string_list,
    text_property, unknown_relation,
};
use crate::ddl_constraints::{TableConstraintInput, add_table_constraints};
use crate::diagnostic::{PostgresDiagnostic, PostgresDiagnosticCode};
use crate::types::PostgresTypeRegistry;
use semver::Version;
use sifr_sql_contract::{
    DatabaseType, DialectIdentity, ObjectId, ProviderIdentity, SchemaDocument, SchemaDocumentKind,
    SchemaIr, SchemaObject, SchemaObjectKind, SchemaSourceLocation, SemanticValue,
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
    pub column_order: Vec<String>,
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
                SchemaObjectKind::Range => {
                    let element = database_type_property(object, "subtype-database-type")?;
                    types.add_nominal(
                        &split_identity(&object.identity),
                        DatabaseType::Named {
                            identity: object.identity.clone(),
                            parameters: Vec::new(),
                            canonical: Box::new(DatabaseType::Range {
                                element: Box::new(element),
                                multirange: optional_bool_property(object, "multirange")
                                    .unwrap_or(false),
                            }),
                        },
                    );
                }
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
                    let lookup_name = match object.semantic.get("overload_name") {
                        Some(SemanticValue::Text(name)) => name.as_str(),
                        _ => last_segment(&object.identity),
                    };
                    functions
                        .entry(lookup_name.to_ascii_lowercase())
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
            let mut relation_columns = BTreeMap::new();
            let mut column_order = Vec::with_capacity(column_ids.len());
            for identity in column_ids {
                let column = columns
                    .get(&identity)
                    .cloned()
                    .ok_or_else(|| schema_error(object, "relation references an unknown column"))?;
                column_order.push(column.name.clone());
                relation_columns.insert(column.name.clone(), column);
            }
            relations.insert(
                object.identity.clone(),
                CatalogRelation {
                    identity: object.identity.clone(),
                    columns: relation_columns,
                    column_order,
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
    pub(crate) fn relation_by_id(&self, identity: &ObjectId) -> Option<&CatalogRelation> {
        self.relations.get(identity)
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
            || crate::semantic_helpers::can_builtin_cast(source, target, implicit)
    }

    #[must_use]
    pub fn object(&self, identity: &ObjectId) -> Option<&SchemaObject> {
        self.objects.get(identity)
    }
}

pub(crate) fn ddl_document(
    document: impl Into<String>,
    statements: &[PostgresStatement],
    types: &PostgresTypeRegistry,
    prior_objects: &BTreeMap<ObjectId, SchemaObject>,
) -> Result<SchemaDocument, PostgresDiagnostic> {
    let document = document.into();
    let prior_identities = prior_objects.keys().cloned().collect::<BTreeSet<_>>();
    let mut objects = prior_objects.clone();
    let mut working_types = types.clone();
    for object in prior_objects.values() {
        match object.kind {
            SchemaObjectKind::Enum => working_types.add_nominal(
                &split_identity(&object.identity),
                DatabaseType::Enum {
                    identity: object.identity.clone(),
                },
            ),
            SchemaObjectKind::Domain => working_types.add_nominal(
                &split_identity(&object.identity),
                DatabaseType::Domain {
                    identity: object.identity.clone(),
                    base: Box::new(database_type_property(object, "base-database-type")?),
                },
            ),
            SchemaObjectKind::Composite => working_types.add_nominal(
                &split_identity(&object.identity),
                DatabaseType::Composite {
                    identity: object.identity.clone(),
                },
            ),
            SchemaObjectKind::Range => working_types.add_nominal(
                &split_identity(&object.identity),
                DatabaseType::Named {
                    identity: object.identity.clone(),
                    parameters: Vec::new(),
                    canonical: Box::new(DatabaseType::Range {
                        element: Box::new(database_type_property(object, "subtype-database-type")?),
                        multirange: optional_bool_property(object, "multirange").unwrap_or(false),
                    }),
                },
            ),
            _ => {}
        }
    }
    for statement in statements {
        match &statement.kind {
            StatementKind::CreateTable(table) => {
                add_table(&document, statement, table, &working_types, &mut objects)?;
            }
            StatementKind::CreateEnum(value) => {
                add_namespace(&document, &value.name, &mut objects);
                let identity = ObjectId::new(qualified_name(&value.name));
                objects.insert(
                    identity.clone(),
                    SchemaObject {
                        identity,
                        kind: SchemaObjectKind::Enum,
                        semantic: enum_semantics(&value.values),
                        dependencies: namespace_dependency(&value.name),
                        source: Some(source_location(&document, statement)),
                    },
                );
                working_types.add_nominal(
                    &value.name,
                    DatabaseType::Enum {
                        identity: ObjectId::new(qualified_name(&value.name)),
                    },
                );
            }
            StatementKind::CreateDomain(value) => {
                add_namespace(&document, &value.name, &mut objects);
                let base = types.resolve(&value.base_type).ok_or_else(|| {
                    schema_error_message(format!(
                        "unknown PostgreSQL domain base type '{}'",
                        value.base_type.display()
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
                            (
                                "sifr_type".to_string(),
                                SemanticValue::Text(
                                    crate::generated_sifr_type(&base.database_type)
                                        .map_err(|error| schema_error_message(error.to_string()))?,
                                ),
                            ),
                        ]),
                        dependencies: namespace_dependency(&value.name),
                        source: Some(source_location(&document, statement)),
                    },
                );
                working_types.add_nominal(
                    &value.name,
                    DatabaseType::Domain {
                        identity: ObjectId::new(qualified_name(&value.name)),
                        base: Box::new(base.database_type),
                    },
                );
            }
            StatementKind::CreateComposite(value) => {
                crate::catalog_advanced::add_composite(
                    &document,
                    statement,
                    value,
                    &mut working_types,
                    &mut objects,
                )?;
            }
            StatementKind::CreateRange(value) => {
                crate::catalog_advanced::add_range(
                    &document,
                    statement,
                    value,
                    &mut working_types,
                    &mut objects,
                )?;
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
                add_view(&document, statement, value, &working_types, &mut objects)?;
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
        objects: objects
            .into_iter()
            .filter_map(|(identity, object)| {
                (!prior_identities.contains(&identity)).then_some(object)
            })
            .collect(),
    })
}

fn add_view(
    document: &str,
    statement: &PostgresStatement,
    view: &crate::ast::CreateViewStatement,
    types: &PostgresTypeRegistry,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), PostgresDiagnostic> {
    add_namespace(document, &view.name, objects);
    let identity = ObjectId::new(qualified_name(&view.name));
    let schema = SchemaIr {
        format_version: 1,
        provider: ProviderIdentity {
            package_id: "sifr-sql-postgresql@0.0.0#ddl".to_string(),
            package_version: Version::new(0, 0, 0),
            package_source: "embedded-ddl".to_string(),
            package_graph_digest: "0".repeat(64),
            compiler_components: BTreeMap::new(),
        },
        dialect: DialectIdentity {
            family: "postgresql".to_string(),
            server_version: types
                .server_profile()
                .strip_prefix("postgresql-")
                .unwrap_or_default()
                .to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        objects: objects.clone(),
    };
    let catalog = PostgresCatalog::from_schema(&schema, types.clone())?;
    let mut context = AnalysisContext::new(&catalog);
    let analyzed = context
        .analyze_select(&view.query, Vec::new())
        .map_err(|error| error.diagnostic)?;
    let mut column_ids = Vec::with_capacity(analyzed.fields.len());
    for field in &analyzed.fields {
        let column_identity = ObjectId::new(format!("{identity}.{}", field.name));
        column_ids.push(column_identity.clone());
        objects.insert(
            column_identity.clone(),
            SchemaObject {
                identity: column_identity,
                kind: SchemaObjectKind::Column,
                semantic: BTreeMap::from([
                    ("name".to_string(), SemanticValue::Text(field.name.clone())),
                    (
                        "database-type".to_string(),
                        database_value(&field.database_type)?,
                    ),
                    ("nullable".to_string(), SemanticValue::Bool(field.nullable)),
                    ("has-default".to_string(), SemanticValue::Bool(false)),
                    ("generated".to_string(), SemanticValue::Bool(false)),
                ]),
                dependencies: BTreeSet::from([identity.clone()]),
                source: Some(source_location(document, statement)),
            },
        );
    }
    let provider_query = crate::canonical_postgres_ast_json(&view.query)
        .map_err(|_| schema_error_message("cannot serialize PostgreSQL view query"))?;
    let mut dependencies = namespace_dependency(&view.name);
    dependencies.extend(analyzed.referenced);
    objects.insert(
        identity.clone(),
        SchemaObject {
            identity,
            kind: if view.materialized {
                SchemaObjectKind::MaterializedView
            } else {
                SchemaObjectKind::View
            },
            semantic: BTreeMap::from([
                (
                    "columns".to_string(),
                    SemanticValue::List(
                        column_ids
                            .iter()
                            .map(|column| SemanticValue::Text(column.as_str().to_string()))
                            .collect(),
                    ),
                ),
                (
                    "provider-query".to_string(),
                    SemanticValue::Text(provider_query),
                ),
                (
                    "primary-key".to_string(),
                    SemanticValue::Set(BTreeSet::new()),
                ),
                ("unique-sets".to_string(), SemanticValue::List(Vec::new())),
            ]),
            dependencies,
            source: Some(source_location(document, statement)),
        },
    );
    Ok(())
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
            schema_error_message(format!("unknown PostgreSQL type '{}'", column.ty.display()))
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
        if let Some(generation) = &column.identity_generation {
            let identity_object =
                ObjectId::new(format!("{}._identity_{}", table_identity, column.name));
            objects.insert(
                identity_object.clone(),
                SchemaObject {
                    identity: identity_object,
                    kind: SchemaObjectKind::IdentityColumn,
                    semantic: BTreeMap::from([
                        (
                            "column".to_string(),
                            SemanticValue::Text(column.name.clone()),
                        ),
                        (
                            "generation".to_string(),
                            SemanticValue::Text(generation.clone()),
                        ),
                    ]),
                    dependencies: BTreeSet::from([ObjectId::new(format!(
                        "{}.{}",
                        table_identity, column.name
                    ))]),
                    source: Some(SchemaSourceLocation {
                        document: document.to_string(),
                        start: column.span.start,
                        end: column.span.end,
                    }),
                },
            );
        }
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

pub(crate) fn add_namespace(
    document: &str,
    name: &[String],
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) {
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

pub(crate) fn namespace_dependency(name: &[String]) -> BTreeSet<ObjectId> {
    BTreeSet::from([ObjectId::new(if name.len() > 1 {
        name[0].clone()
    } else {
        "public".to_string()
    })])
}

pub(crate) fn qualified_name(name: &[String]) -> String {
    if name.len() > 1 {
        name.join(".")
    } else {
        format!(
            "public.{}",
            name.first().map(String::as_str).unwrap_or("unknown")
        )
    }
}

pub(crate) fn source_location(
    document: &str,
    statement: &PostgresStatement,
) -> SchemaSourceLocation {
    SchemaSourceLocation {
        document: document.to_string(),
        start: statement.span.start,
        end: statement.span.end,
    }
}
