use crate::catalog::PostgresCatalogError;
use sha2::{Digest, Sha256};
use sifr_sql_contract::{DatabaseType, ObjectId, SchemaObject, SchemaObjectKind, SemanticValue};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresParser, PostgresStatement, PostgresTypeName, PostgresTypeRegistry,
    StatementKind, canonical_postgres_ast_json, generated_sifr_type,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn normalize_catalog_objects(
    server_major: u16,
    objects: &mut [SchemaObject],
) -> Result<(), PostgresCatalogError> {
    let mut registry = PostgresTypeRegistry::new(server_major);
    register_simple_nominals(&mut registry, objects);
    register_dependent_nominals(&mut registry, objects)?;
    normalize_constraint_dependencies(objects)?;
    for object in objects.iter_mut() {
        normalize_object(&registry, object)?;
    }
    canonicalize_constraint_identities(objects)?;
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

fn register_dependent_nominals(
    registry: &mut PostgresTypeRegistry,
    objects: &[SchemaObject],
) -> Result<(), PostgresCatalogError> {
    let mut remaining = objects
        .iter()
        .filter(|object| {
            matches!(
                object.kind,
                SchemaObjectKind::Domain | SchemaObjectKind::Range
            )
        })
        .collect::<Vec<_>>();
    while !remaining.is_empty() {
        let before = remaining.len();
        remaining.retain(|object| {
            let name = match object.kind {
                SchemaObjectKind::Domain => optional_text(object, "base-database-type-name"),
                SchemaObjectKind::Range => optional_text(object, "subtype-database-type-name"),
                _ => None,
            };
            let Some(name) = name else { return true };
            let Ok(base) = resolve_type(registry, name) else {
                return true;
            };
            let ty = match object.kind {
                SchemaObjectKind::Domain => DatabaseType::Domain {
                    identity: object.identity.clone(),
                    base: Box::new(base),
                },
                SchemaObjectKind::Range => DatabaseType::Named {
                    identity: object.identity.clone(),
                    parameters: Vec::new(),
                    canonical: Box::new(DatabaseType::Range {
                        element: Box::new(base),
                        multirange: optional_bool(object, "multirange").unwrap_or(false),
                    }),
                },
                _ => return true,
            };
            registry.add_nominal(&identity_path(&object.identity), ty);
            false
        });
        if remaining.len() == before {
            return Err(incomplete("domain or range base type"));
        }
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
        SchemaObjectKind::IdentityColumn => BTreeMap::from([
            (
                "column".to_string(),
                SemanticValue::Text(required_text(object, "column")?.to_string()),
            ),
            (
                "generation".to_string(),
                SemanticValue::Text(
                    match required_text(object, "generation")? {
                        "a" | "always" => "always",
                        "d" | "by-default" => "by-default",
                        _ => return Err(incomplete("identity generation mode")),
                    }
                    .to_string(),
                ),
            ),
        ]),
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
            SemanticValue::Text(canonical_expression(required_text(object, "expression")?)?),
        )]),
        SchemaObjectKind::Index => BTreeMap::from([
            ("columns".to_string(), required_list(object, "columns")?),
            (
                "unique".to_string(),
                SemanticValue::Bool(required_bool(object, "unique")?),
            ),
        ]),
        SchemaObjectKind::Sequence => sequence_semantics(object)?,
        _ => object.semantic.clone(),
    };
    Ok(())
}

fn sequence_semantics(
    object: &SchemaObject,
) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    let mut semantic = BTreeMap::from([
        (
            "name".to_string(),
            SemanticValue::Text(required_text(object, "name")?.to_string()),
        ),
        (
            "data-type".to_string(),
            SemanticValue::Text(required_text(object, "data-type")?.to_string()),
        ),
        (
            "start".to_string(),
            SemanticValue::Signed(required_signed(object, "start")?),
        ),
        (
            "increment".to_string(),
            SemanticValue::Signed(required_signed(object, "increment")?),
        ),
        (
            "minimum".to_string(),
            SemanticValue::Signed(required_signed(object, "minimum")?),
        ),
        (
            "maximum".to_string(),
            SemanticValue::Signed(required_signed(object, "maximum")?),
        ),
        (
            "cache".to_string(),
            SemanticValue::Signed(required_signed(object, "cache")?),
        ),
        (
            "cycle".to_string(),
            SemanticValue::Bool(required_bool(object, "cycle")?),
        ),
    ]);
    if let Some(owner) = optional_text(object, "owned-by") {
        semantic.insert(
            "owned-by".to_string(),
            SemanticValue::Text(owner.to_string()),
        );
    }
    Ok(semantic)
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
    let mut semantic = BTreeMap::from([
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
        ("result-nullable".to_string(), SemanticValue::Bool(true)),
    ]);
    for key in ["overload_namespace", "overload_name"] {
        if let Some(value) = optional_text(object, key) {
            semantic.insert(key.to_string(), SemanticValue::Text(value.to_string()));
        }
    }
    Ok(semantic)
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
                SemanticValue::Text(canonical_select(required_text(relation, "definition")?)?),
            );
        }
        relation.semantic = semantic;
    }
    Ok(())
}

fn canonical_select(source: &str) -> Result<String, PostgresCatalogError> {
    let statements = LibpgQueryParser
        .parse(source)
        .map_err(|_| incomplete("parseable view definition"))?;
    let [
        PostgresStatement {
            kind: StatementKind::Select(query),
            ..
        },
    ] = statements.as_slice()
    else {
        return Err(incomplete("single SELECT view definition"));
    };
    canonical_postgres_ast_json(query).map_err(|_| incomplete("canonical view definition"))
}

fn canonical_expression(source: &str) -> Result<String, PostgresCatalogError> {
    let statements = LibpgQueryParser
        .parse(&format!("SELECT {source}"))
        .map_err(|_| incomplete("parseable check expression"))?;
    let [
        PostgresStatement {
            kind: StatementKind::Select(query),
            ..
        },
    ] = statements.as_slice()
    else {
        return Err(incomplete("single check expression"));
    };
    let [target] = query.targets.as_slice() else {
        return Err(incomplete("single check expression target"));
    };
    canonical_postgres_ast_json(&target.expression)
        .map_err(|_| incomplete("canonical check expression"))
}

fn canonicalize_constraint_identities(
    objects: &mut [SchemaObject],
) -> Result<(), PostgresCatalogError> {
    let relation_ids = objects
        .iter()
        .filter(|object| object.kind == SchemaObjectKind::Table)
        .map(|object| object.identity.clone())
        .collect::<BTreeSet<_>>();
    let mut replacements = BTreeMap::new();
    for object in objects.iter().filter(|object| is_constraint(object.kind)) {
        let relation = object
            .dependencies
            .iter()
            .find(|identity| relation_ids.contains(*identity))
            .ok_or_else(|| incomplete("constraint relation dependency"))?;
        let identity = match object.kind {
            SchemaObjectKind::PrimaryKey => constraint_identity(relation, "pkey", &[]),
            SchemaObjectKind::UniqueConstraint => constraint_identity(
                relation,
                "unique",
                &text_list(&object.semantic, "columns")
                    .ok_or_else(|| incomplete("constraint columns"))?,
            ),
            SchemaObjectKind::ForeignKey => {
                let mut signature = text_list(&object.semantic, "columns")
                    .ok_or_else(|| incomplete("foreign-key columns"))?;
                signature.extend(
                    required_text(object, "referenced-relation")?
                        .split('.')
                        .map(str::to_string),
                );
                signature.extend(
                    text_list(&object.semantic, "referenced-columns")
                        .ok_or_else(|| incomplete("foreign-key referenced columns"))?,
                );
                constraint_identity(relation, "fkey", &signature)
            }
            SchemaObjectKind::CheckConstraint => constraint_identity(
                relation,
                "check",
                &[required_text(object, "provider-expression")?.to_string()],
            ),
            _ => continue,
        };
        replacements.insert(object.identity.clone(), identity);
    }
    let mut seen = BTreeSet::new();
    for object in objects.iter_mut() {
        if let Some(identity) = replacements.get(&object.identity) {
            object.identity = identity.clone();
        }
        object.dependencies = object
            .dependencies
            .iter()
            .map(|identity| replacements.get(identity).unwrap_or(identity).clone())
            .collect();
        if !seen.insert(object.identity.clone()) {
            return Err(PostgresCatalogError {
                message: format!(
                    "PostgreSQL catalog normalizes more than one object to '{}'",
                    object.identity
                ),
            });
        }
    }
    Ok(())
}

fn constraint_identity(relation: &ObjectId, kind: &str, signature: &[String]) -> ObjectId {
    let mut segments = relation.as_str().rsplitn(2, '.');
    let table = segments.next().unwrap_or("relation");
    let namespace = segments.next().unwrap_or("public");
    if kind == "pkey" {
        return ObjectId::new(format!("{namespace}.{table}_pkey"));
    }
    let mut digest = Sha256::new();
    digest.update(kind.as_bytes());
    for value in signature {
        digest.update([0]);
        digest.update(value.as_bytes());
    }
    let digest = lower_hex(&digest.finalize());
    ObjectId::new(format!("{namespace}.{table}_{kind}_{}", &digest[..16]))
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
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

fn required_signed(object: &SchemaObject, key: &str) -> Result<i64, PostgresCatalogError> {
    match object.semantic.get(key) {
        Some(SemanticValue::Signed(value)) => Ok(*value),
        _ => Err(incomplete(key)),
    }
}

fn optional_bool(object: &SchemaObject, key: &str) -> Option<bool> {
    match object.semantic.get(key) {
        Some(SemanticValue::Bool(value)) => Some(*value),
        _ => None,
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
