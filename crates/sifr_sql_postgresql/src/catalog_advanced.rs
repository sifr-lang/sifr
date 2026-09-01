use crate::ast::{CreateCompositeStatement, CreateRangeStatement, PostgresStatement};
use crate::catalog::{add_namespace, namespace_dependency, qualified_name, source_location};
use crate::catalog_semantics::{database_value, schema_error_message};
use crate::diagnostic::PostgresDiagnostic;
use crate::types::PostgresTypeRegistry;
use sifr_sql_contract::{DatabaseType, ObjectId, SchemaObject, SchemaObjectKind, SemanticValue};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn add_composite(
    document: &str,
    statement: &PostgresStatement,
    value: &CreateCompositeStatement,
    types: &mut PostgresTypeRegistry,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), PostgresDiagnostic> {
    add_namespace(document, &value.name, objects);
    let identity = ObjectId::new(qualified_name(&value.name));
    let mut attributes = Vec::new();
    let mut fields = BTreeMap::new();
    for attribute in &value.attributes {
        let ty = types.resolve(&attribute.ty).ok_or_else(|| {
            schema_error_message(format!(
                "unknown PostgreSQL composite attribute type '{}'",
                attribute.ty.display()
            ))
        })?;
        attributes.push(SemanticValue::Map(BTreeMap::from([
            (
                "name".to_string(),
                SemanticValue::Text(attribute.name.clone()),
            ),
            (
                "database-type".to_string(),
                database_value(&ty.database_type)?,
            ),
            (
                "nullable".to_string(),
                SemanticValue::Bool(attribute.nullable),
            ),
        ])));
        fields.insert(
            attribute.name.clone(),
            SemanticValue::Text(
                crate::generated_sifr_type(&ty.database_type)
                    .map_err(|error| schema_error_message(error.to_string()))?,
            ),
        );
    }
    objects.insert(
        identity.clone(),
        SchemaObject {
            identity: identity.clone(),
            kind: SchemaObjectKind::Composite,
            semantic: BTreeMap::from([
                ("attributes".to_string(), SemanticValue::List(attributes)),
                ("fields".to_string(), SemanticValue::Map(fields)),
            ]),
            dependencies: namespace_dependency(&value.name),
            source: Some(source_location(document, statement)),
        },
    );
    types.add_nominal(&value.name, DatabaseType::Composite { identity });
    Ok(())
}

pub(crate) fn add_range(
    document: &str,
    statement: &PostgresStatement,
    value: &CreateRangeStatement,
    types: &mut PostgresTypeRegistry,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), PostgresDiagnostic> {
    add_namespace(document, &value.name, objects);
    let subtype = types.resolve(&value.subtype).ok_or_else(|| {
        schema_error_message(format!(
            "unknown PostgreSQL range subtype '{}'",
            value.subtype.display()
        ))
    })?;
    let identity = ObjectId::new(qualified_name(&value.name));
    objects.insert(
        identity.clone(),
        range_object(
            identity.clone(),
            &subtype.database_type,
            false,
            namespace_dependency(&value.name),
            source_location(document, statement),
        )?,
    );
    types.add_nominal(
        &value.name,
        DatabaseType::Named {
            identity: identity.clone(),
            parameters: Vec::new(),
            canonical: Box::new(DatabaseType::Range {
                element: Box::new(subtype.database_type.clone()),
                multirange: false,
            }),
        },
    );
    if let Some(multirange_name) = &value.multirange_name {
        let multirange_identity = ObjectId::new(qualified_name(multirange_name));
        objects.insert(
            multirange_identity.clone(),
            range_object(
                multirange_identity,
                &subtype.database_type,
                true,
                BTreeSet::from([identity]),
                source_location(document, statement),
            )?,
        );
    }
    Ok(())
}

fn range_object(
    identity: ObjectId,
    subtype: &DatabaseType,
    multirange: bool,
    dependencies: BTreeSet<ObjectId>,
    source: sifr_sql_contract::SchemaSourceLocation,
) -> Result<SchemaObject, PostgresDiagnostic> {
    Ok(SchemaObject {
        identity,
        kind: SchemaObjectKind::Range,
        semantic: BTreeMap::from([
            (
                "subtype-database-type".to_string(),
                database_value(subtype)?,
            ),
            ("multirange".to_string(), SemanticValue::Bool(multirange)),
        ]),
        dependencies,
        source: Some(source),
    })
}
