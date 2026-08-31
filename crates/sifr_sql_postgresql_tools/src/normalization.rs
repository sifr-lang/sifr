use crate::catalog::PostgresCatalogError;
use sifr_sql_contract::{DatabaseType, ObjectId, SchemaObject, SchemaObjectKind, SemanticValue};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresParser, PostgresStatement, PostgresTypeName, PostgresTypeRegistry,
    StatementKind, generated_sifr_type,
};
use std::collections::BTreeMap;

pub(crate) fn normalize_catalog_objects(
    server_major: u16,
    objects: &mut [SchemaObject],
) -> Result<(), PostgresCatalogError> {
    let mut registry = PostgresTypeRegistry::new(server_major);
    register_simple_nominals(&mut registry, objects);
    register_domains(&mut registry, objects)?;
    register_ranges(&mut registry, objects)?;
    normalize_constraint_dependencies(objects)?;
    for object in objects.iter_mut() {
        normalize_object(&registry, object)?;
    }
    normalize_relations(objects)?;
    Ok(())
}

fn register_simple_nominals(registry: &mut PostgresTypeRegistry, objects: &[SchemaObject]) {
    for object in objects {
        let identity = object.identity.clone();
        match object.kind {
            SchemaObjectKind::Enum => {
                registry.add_nominal(&identity_path(&identity), DatabaseType::Enum { identity });
            }
            SchemaObjectKind::Composite => {
                registry.add_nominal(
                    &identity_path(&identity),
                    DatabaseType::Composite { identity },
                );
            }
            _ => {}
        }
    }
}

fn register_domains(
    registry: &mut PostgresTypeRegistry,
    objects: &[SchemaObject],
) -> Result<(), PostgresCatalogError> {
    let mut remaining = objects
        .iter()
        .filter(|object| object.kind == SchemaObjectKind::Domain)
        .collect::<Vec<_>>();
    while !remaining.is_empty() {
        let before = remaining.len();
        remaining.retain(|object| {
            let Some(name) = optional_text(object, "base-database-type-name") else {
                return true;
            };
            let Ok(base) = resolve_type(registry, name) else {
                return true;
            };
            registry.add_nominal(
                &identity_path(&object.identity),
                DatabaseType::Domain {
                    identity: object.identity.clone(),
                    base: Box::new(base),
                },
            );
            false
        });
        if remaining.len() == before {
            return Err(incomplete("domain base type"));
        }
    }
    Ok(())
}

fn register_ranges(
    registry: &mut PostgresTypeRegistry,
    objects: &[SchemaObject],
) -> Result<(), PostgresCatalogError> {
    for object in objects
        .iter()
        .filter(|object| object.kind == SchemaObjectKind::Range)
    {
        let subtype = resolve_type(
            registry,
            required_text(object, "subtype-database-type-name")?,
        )?;
        let multirange = required_bool(object, "multirange")?;
        registry.add_nominal(
            &identity_path(&object.identity),
            DatabaseType::Named {
                identity: object.identity.clone(),
                parameters: Vec::new(),
                canonical: Box::new(DatabaseType::Range {
                    element: Box::new(subtype),
                    multirange,
                }),
            },
        );
    }
    Ok(())
}

fn normalize_object(
    registry: &PostgresTypeRegistry,
    object: &mut SchemaObject,
) -> Result<(), PostgresCatalogError> {
    object.semantic = match object.kind {
        SchemaObjectKind::Namespace => BTreeMap::new(),
        SchemaObjectKind::Column => column_semantics(registry, object)?,
        SchemaObjectKind::Enum => enum_semantics(object)?,
        SchemaObjectKind::Domain => domain_semantics(registry, object)?,
        SchemaObjectKind::Composite => composite_semantics(registry, object)?,
        SchemaObjectKind::Range => range_semantics(registry, object)?,
        SchemaObjectKind::Function => function_semantics(registry, object)?,
        SchemaObjectKind::Operator => operator_semantics(registry, object)?,
        SchemaObjectKind::Cast => cast_semantics(registry, object)?,
        SchemaObjectKind::PrimaryKey | SchemaObjectKind::UniqueConstraint => {
            BTreeMap::from([("columns".to_string(), required_list(object, "columns")?)])
        }
        SchemaObjectKind::ForeignKey => BTreeMap::from([
            ("columns".to_string(), required_list(object, "columns")?),
            (
                "referenced-relation".to_string(),
                SemanticValue::Text(required_text(object, "referenced-relation")?.to_string()),
            ),
            (
                "referenced-columns".to_string(),
                required_list(object, "referenced-columns")?,
            ),
        ]),
        SchemaObjectKind::CheckConstraint => BTreeMap::from([(
            "provider-expression".to_string(),
            SemanticValue::Text(required_text(object, "definition")?.to_string()),
        )]),
        SchemaObjectKind::Index => BTreeMap::from([
            ("columns".to_string(), required_list(object, "columns")?),
            (
                "unique".to_string(),
                SemanticValue::Bool(required_bool(object, "unique")?),
            ),
        ]),
        SchemaObjectKind::Sequence => BTreeMap::new(),
        _ => object.semantic.clone(),
    };
    Ok(())
}

fn column_semantics(
    registry: &PostgresTypeRegistry,
    object: &SchemaObject,
) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    let database_type = resolve_type(registry, required_text(object, "database-type-name")?)?;
    Ok(BTreeMap::from([
        (
            "name".to_string(),
            SemanticValue::Text(required_text(object, "name")?.to_string()),
        ),
        ("database-type".to_string(), database_value(&database_type)?),
        (
            "nullable".to_string(),
            SemanticValue::Bool(required_bool(object, "nullable")?),
        ),
        (
            "has-default".to_string(),
            SemanticValue::Bool(required_bool(object, "has-default")?),
        ),
        (
            "generated".to_string(),
            SemanticValue::Bool(required_bool(object, "generated")?),
        ),
    ]))
}

fn enum_semantics(
    object: &SchemaObject,
) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    let values = required_list(object, "values")?;
    Ok(BTreeMap::from([
        ("values".to_string(), values.clone()),
        ("variants".to_string(), values),
    ]))
}

fn domain_semantics(
    registry: &PostgresTypeRegistry,
    object: &SchemaObject,
) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    let base = resolve_type(registry, required_text(object, "base-database-type-name")?)?;
    Ok(BTreeMap::from([
        ("base-database-type".to_string(), database_value(&base)?),
        (
            "nullable".to_string(),
            SemanticValue::Bool(required_bool(object, "nullable")?),
        ),
        (
            "sifr_type".to_string(),
            SemanticValue::Text(generated_type(&base)?),
        ),
    ]))
}

fn composite_semantics(
    registry: &PostgresTypeRegistry,
    object: &SchemaObject,
) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    let SemanticValue::List(raw_attributes) = required_list(object, "attributes")? else {
        return Err(incomplete("composite attributes"));
    };
    let mut attributes = Vec::new();
    let mut fields = BTreeMap::new();
    for raw in raw_attributes {
        let SemanticValue::Map(raw) = raw else {
            return Err(incomplete("composite attribute map"));
        };
        let name = map_text(&raw, "name")?;
        let database_type = resolve_type(registry, map_text(&raw, "database-type-name")?)?;
        let nullable = match raw.get("nullable") {
            Some(SemanticValue::Bool(value)) => *value,
            _ => return Err(incomplete("composite attribute nullability")),
        };
        attributes.push(SemanticValue::Map(BTreeMap::from([
            ("name".to_string(), SemanticValue::Text(name.to_string())),
            ("database-type".to_string(), database_value(&database_type)?),
            ("nullable".to_string(), SemanticValue::Bool(nullable)),
        ])));
        fields.insert(
            name.to_string(),
            SemanticValue::Text(generated_type(&database_type)?),
        );
    }
    Ok(BTreeMap::from([
        ("attributes".to_string(), SemanticValue::List(attributes)),
        ("fields".to_string(), SemanticValue::Map(fields)),
    ]))
}

fn range_semantics(
    registry: &PostgresTypeRegistry,
    object: &SchemaObject,
) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    let subtype = resolve_type(
        registry,
        required_text(object, "subtype-database-type-name")?,
    )?;
    Ok(BTreeMap::from([
        (
            "subtype-database-type".to_string(),
            database_value(&subtype)?,
        ),
        (
            "multirange".to_string(),
            SemanticValue::Bool(required_bool(object, "multirange")?),
        ),
    ]))
}

fn function_semantics(
    registry: &PostgresTypeRegistry,
    object: &SchemaObject,
) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    let SemanticValue::List(arguments) = required_list(object, "arguments")? else {
        return Err(incomplete("function arguments"));
    };
    let arguments = arguments
        .into_iter()
        .map(|value| match value {
            SemanticValue::Text(name) => {
                resolve_type(registry, &name).and_then(|ty| database_value(&ty))
            }
            _ => Err(incomplete("function argument type")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result = resolve_type(registry, required_text(object, "result-type-name")?)?;
    Ok(BTreeMap::from([
        ("arguments".to_string(), SemanticValue::List(arguments)),
        ("result".to_string(), database_value(&result)?),
        (
            "strict".to_string(),
            SemanticValue::Bool(required_bool(object, "strict")?),
        ),
        (
            "aggregate".to_string(),
            SemanticValue::Bool(required_text(object, "kind")? == "a"),
        ),
        (
            "result-nullable".to_string(),
            SemanticValue::Bool(!required_bool(object, "strict")?),
        ),
    ]))
}

fn operator_semantics(
    registry: &PostgresTypeRegistry,
    object: &SchemaObject,
) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    Ok(BTreeMap::from([
        (
            "name".to_string(),
            SemanticValue::Text(required_text(object, "name")?.to_string()),
        ),
        (
            "left".to_string(),
            database_value(&resolve_type(
                registry,
                required_text(object, "left-type-name")?,
            )?)?,
        ),
        (
            "right".to_string(),
            database_value(&resolve_type(
                registry,
                required_text(object, "right-type-name")?,
            )?)?,
        ),
        (
            "result".to_string(),
            database_value(&resolve_type(
                registry,
                required_text(object, "result-type-name")?,
            )?)?,
        ),
    ]))
}

fn cast_semantics(
    registry: &PostgresTypeRegistry,
    object: &SchemaObject,
) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    Ok(BTreeMap::from([
        (
            "source-database-type".to_string(),
            database_value(&resolve_type(
                registry,
                required_text(object, "source-type-name")?,
            )?)?,
        ),
        (
            "target-database-type".to_string(),
            database_value(&resolve_type(
                registry,
                required_text(object, "target-type-name")?,
            )?)?,
        ),
        (
            "implicit".to_string(),
            SemanticValue::Bool(required_text(object, "context")? == "i"),
        ),
    ]))
}

fn normalize_relations(objects: &mut [SchemaObject]) -> Result<(), PostgresCatalogError> {
    let snapshot = objects.to_vec();
    for relation in objects.iter_mut().filter(|object| {
        matches!(
            object.kind,
            SchemaObjectKind::Table | SchemaObjectKind::View | SchemaObjectKind::MaterializedView
        )
    }) {
        let columns = required_list(relation, "columns")?;
        let constraints = snapshot
            .iter()
            .filter(|candidate| {
                is_constraint(candidate.kind) && candidate.dependencies.contains(&relation.identity)
            })
            .collect::<Vec<_>>();
        let primary_key = constraints
            .iter()
            .filter(|constraint| constraint.kind == SchemaObjectKind::PrimaryKey)
            .flat_map(|constraint| text_list(&constraint.semantic, "columns").unwrap_or_default())
            .map(SemanticValue::Text)
            .collect();
        let unique_sets = constraints
            .iter()
            .filter(|constraint| constraint.kind == SchemaObjectKind::UniqueConstraint)
            .map(|constraint| required_list(constraint, "columns"))
            .collect::<Result<Vec<_>, _>>()?;
        let mut semantic = BTreeMap::from([
            ("columns".to_string(), columns),
            ("primary-key".to_string(), SemanticValue::Set(primary_key)),
            ("unique-sets".to_string(), SemanticValue::List(unique_sets)),
        ]);
        if relation.kind == SchemaObjectKind::Table {
            semantic.insert(
                "constraints".to_string(),
                SemanticValue::List(
                    constraints
                        .into_iter()
                        .map(|value| SemanticValue::Text(value.identity.as_str().to_string()))
                        .collect(),
                ),
            );
        } else {
            semantic.insert(
                "provider-query".to_string(),
                SemanticValue::Text(required_text(relation, "definition")?.to_string()),
            );
        }
        relation.semantic = semantic;
    }
    Ok(())
}

fn normalize_constraint_dependencies(
    objects: &mut [SchemaObject],
) -> Result<(), PostgresCatalogError> {
    let snapshot = objects.to_vec();
    let mut foreign_updates = Vec::new();
    for constraint in objects
        .iter_mut()
        .filter(|object| is_constraint(object.kind))
    {
        let relation = constraint
            .dependencies
            .iter()
            .find(|identity| {
                snapshot.iter().any(|object| {
                    object.identity == **identity && object.kind == SchemaObjectKind::Table
                })
            })
            .cloned()
            .ok_or_else(|| incomplete("constraint relation dependency"))?;
        let columns = text_list(&constraint.semantic, "columns")
            .ok_or_else(|| incomplete("constraint columns"))?;
        for column in snapshot.iter().filter(|object| {
            object.kind == SchemaObjectKind::Column
                && object.dependencies.contains(&relation)
                && optional_text(object, "name")
                    .is_some_and(|name| columns.contains(&name.to_string()))
        }) {
            constraint.dependencies.insert(column.identity.clone());
        }
        if constraint.kind == SchemaObjectKind::ForeignKey {
            let target = required_text(constraint, "referenced-relation")?.to_string();
            foreign_updates.push((relation, columns, ObjectId::new(target)));
        }
    }
    for (relation, columns, target) in foreign_updates {
        for column in objects.iter_mut().filter(|object| {
            object.kind == SchemaObjectKind::Column
                && object.dependencies.contains(&relation)
                && optional_text(object, "name")
                    .is_some_and(|name| columns.contains(&name.to_string()))
        }) {
            column.dependencies.insert(target.clone());
        }
    }
    Ok(())
}

fn resolve_type(
    registry: &PostgresTypeRegistry,
    source: &str,
) -> Result<DatabaseType, PostgresCatalogError> {
    let name = parse_type_name(source)?;
    registry
        .resolve(&name)
        .map(|ty| ty.database_type)
        .ok_or_else(|| PostgresCatalogError {
            message: format!("PostgreSQL catalog contains unsupported database type '{source}'"),
        })
}

fn parse_type_name(source: &str) -> Result<PostgresTypeName, PostgresCatalogError> {
    let sql = format!("CREATE TABLE __sifr_catalog_type (value {source})");
    let statements = LibpgQueryParser
        .parse(&sql)
        .map_err(|_| incomplete("parseable PostgreSQL database type"))?;
    let [
        PostgresStatement {
            kind: StatementKind::CreateTable(table),
            ..
        },
    ] = statements.as_slice()
    else {
        return Err(incomplete("single PostgreSQL database type"));
    };
    let [column] = table.columns.as_slice() else {
        return Err(incomplete("single PostgreSQL database type column"));
    };
    Ok(column.ty.clone())
}

fn database_value(database_type: &DatabaseType) -> Result<SemanticValue, PostgresCatalogError> {
    serde_json::to_string(database_type)
        .map(SemanticValue::Text)
        .map_err(|_| incomplete("serializable PostgreSQL database type"))
}

fn generated_type(database_type: &DatabaseType) -> Result<String, PostgresCatalogError> {
    generated_sifr_type(database_type).map_err(|_| incomplete("representable generated Sifr type"))
}

fn required_text<'a>(object: &'a SchemaObject, key: &str) -> Result<&'a str, PostgresCatalogError> {
    optional_text(object, key).ok_or_else(|| incomplete(key))
}

fn optional_text<'a>(object: &'a SchemaObject, key: &str) -> Option<&'a str> {
    match object.semantic.get(key) {
        Some(SemanticValue::Text(value)) => Some(value),
        _ => None,
    }
}

fn required_bool(object: &SchemaObject, key: &str) -> Result<bool, PostgresCatalogError> {
    match object.semantic.get(key) {
        Some(SemanticValue::Bool(value)) => Ok(*value),
        _ => Err(incomplete(key)),
    }
}

fn required_list(object: &SchemaObject, key: &str) -> Result<SemanticValue, PostgresCatalogError> {
    match object.semantic.get(key) {
        Some(value @ SemanticValue::List(_)) => Ok(value.clone()),
        _ => Err(incomplete(key)),
    }
}

fn map_text<'a>(
    values: &'a BTreeMap<String, SemanticValue>,
    key: &str,
) -> Result<&'a str, PostgresCatalogError> {
    match values.get(key) {
        Some(SemanticValue::Text(value)) => Ok(value),
        _ => Err(incomplete(key)),
    }
}

fn text_list(values: &BTreeMap<String, SemanticValue>, key: &str) -> Option<Vec<String>> {
    match values.get(key) {
        Some(SemanticValue::List(values)) => values
            .iter()
            .map(|value| match value {
                SemanticValue::Text(value) => Some(value.clone()),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

fn identity_path(identity: &ObjectId) -> Vec<String> {
    identity.as_str().split('.').map(str::to_string).collect()
}

fn is_constraint(kind: SchemaObjectKind) -> bool {
    matches!(
        kind,
        SchemaObjectKind::PrimaryKey
            | SchemaObjectKind::UniqueConstraint
            | SchemaObjectKind::ForeignKey
            | SchemaObjectKind::CheckConstraint
    )
}

fn incomplete(field: &str) -> PostgresCatalogError {
    PostgresCatalogError {
        message: format!("PostgreSQL catalog is missing required {field}"),
    }
}
