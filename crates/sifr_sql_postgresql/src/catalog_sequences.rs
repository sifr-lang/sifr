use crate::ast::{
    AlterSequenceStatement, CreateSequenceStatement, PostgresStatement, SequenceDataType,
};
use crate::catalog::{add_namespace, namespace_dependency, qualified_name, source_location};
use crate::catalog_semantics::schema_error_message;
use crate::diagnostic::PostgresDiagnostic;
use sifr_sql_contract::{ObjectId, SchemaObject, SchemaObjectKind, SemanticValue};
use std::collections::BTreeMap;

pub(crate) fn add_sequence(
    document: &str,
    statement: &PostgresStatement,
    sequence: &CreateSequenceStatement,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), PostgresDiagnostic> {
    add_namespace(document, &sequence.name, objects);
    let identity = ObjectId::new(qualified_name(&sequence.name));
    let mut semantic = sequence_semantics(sequence)?;
    let mut dependencies = namespace_dependency(&sequence.name);
    if let Some(owner) = &sequence.owned_by {
        let owner = owned_sequence_column_identity(owner);
        semantic.insert("owned-by".to_string(), SemanticValue::Text(owner.clone()));
        dependencies.insert(ObjectId::new(owner));
    }
    objects.insert(
        identity.clone(),
        SchemaObject {
            identity,
            kind: SchemaObjectKind::Sequence,
            semantic,
            dependencies,
            source: Some(source_location(document, statement)),
        },
    );
    Ok(())
}

pub(crate) fn alter_sequence(
    change: &AlterSequenceStatement,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), PostgresDiagnostic> {
    let identity = ObjectId::new(qualified_name(&change.name));
    let owned_by = owned_sequence_column_identity(&change.owned_by);
    let sequence = objects.get_mut(&identity).ok_or_else(|| {
        schema_error_message(format!(
            "ALTER SEQUENCE names unknown sequence '{identity}'"
        ))
    })?;
    sequence.semantic.insert(
        "owned-by".to_string(),
        SemanticValue::Text(owned_by.clone()),
    );
    sequence.dependencies.insert(ObjectId::new(owned_by));
    Ok(())
}

fn sequence_semantics(
    sequence: &CreateSequenceStatement,
) -> Result<BTreeMap<String, SemanticValue>, PostgresDiagnostic> {
    let (data_type, type_minimum, type_maximum) = match sequence.data_type {
        SequenceDataType::SmallInt => ("smallint", i64::from(i16::MIN), i64::from(i16::MAX)),
        SequenceDataType::Integer => ("integer", i64::from(i32::MIN), i64::from(i32::MAX)),
        SequenceDataType::BigInt => ("bigint", i64::MIN, i64::MAX),
    };
    let minimum = sequence.minimum.unwrap_or(if sequence.increment > 0 {
        1
    } else {
        type_minimum
    });
    let maximum = sequence.maximum.unwrap_or(if sequence.increment > 0 {
        type_maximum
    } else {
        -1
    });
    if minimum < type_minimum || minimum > type_maximum {
        return Err(schema_error_message(format!(
            "CREATE SEQUENCE MINVALUE {minimum} is outside {data_type}"
        )));
    }
    if maximum < type_minimum || maximum > type_maximum {
        return Err(schema_error_message(format!(
            "CREATE SEQUENCE MAXVALUE {maximum} is outside {data_type}"
        )));
    }
    if minimum >= maximum {
        return Err(schema_error_message(
            "CREATE SEQUENCE MINVALUE must be less than MAXVALUE",
        ));
    }
    let start = sequence.start.unwrap_or(if sequence.increment > 0 {
        minimum
    } else {
        maximum
    });
    if start < minimum || start > maximum {
        return Err(schema_error_message(format!(
            "CREATE SEQUENCE START {start} is outside its minimum and maximum"
        )));
    }
    Ok(BTreeMap::from([
        (
            "name".to_string(),
            SemanticValue::Text(sequence.name.last().cloned().unwrap_or_default()),
        ),
        (
            "data-type".to_string(),
            SemanticValue::Text(data_type.to_string()),
        ),
        ("start".to_string(), SemanticValue::Signed(start)),
        (
            "increment".to_string(),
            SemanticValue::Signed(sequence.increment),
        ),
        ("minimum".to_string(), SemanticValue::Signed(minimum)),
        ("maximum".to_string(), SemanticValue::Signed(maximum)),
        ("cache".to_string(), SemanticValue::Signed(sequence.cache)),
        ("cycle".to_string(), SemanticValue::Bool(sequence.cycle)),
    ]))
}

fn owned_sequence_column_identity(name: &[String]) -> String {
    if name.len() == 2 {
        format!("public.{}", name.join("."))
    } else {
        name.join(".")
    }
}
