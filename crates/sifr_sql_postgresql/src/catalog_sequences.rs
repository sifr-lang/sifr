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
    if objects.contains_key(&identity) {
        if sequence.if_not_exists {
            return Ok(());
        }
        return Err(schema_error_message(format!(
            "CREATE SEQUENCE names existing object '{identity}'"
        )));
    }
    let mut semantic = sequence_semantics(sequence)?;
    let mut dependencies = namespace_dependency(&sequence.name);
    if let Some(owner) = &sequence.owned_by {
        let owner = validate_owner(&sequence.name, owner, objects)?;
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
    document: &str,
    statement: &PostgresStatement,
    change: &AlterSequenceStatement,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), PostgresDiagnostic> {
    let identity = ObjectId::new(qualified_name(&change.name));
    if !objects.contains_key(&identity) {
        if change.if_exists {
            return Ok(());
        }
        return Err(schema_error_message(format!(
            "ALTER SEQUENCE names unknown sequence '{identity}'"
        )));
    }
    let owned_by = change
        .owned_by
        .as_ref()
        .map(|owner| validate_owner(&change.name, owner, objects))
        .transpose()?;
    let sequence = objects.get_mut(&identity).ok_or_else(|| {
        schema_error_message(format!(
            "ALTER SEQUENCE names unknown sequence '{identity}'"
        ))
    })?;
    if sequence.kind != SchemaObjectKind::Sequence {
        return Err(schema_error_message(format!(
            "'{identity}' is not a sequence"
        )));
    }
    if let Some(SemanticValue::Text(previous)) = sequence.semantic.remove("owned-by") {
        sequence.dependencies.remove(&ObjectId::new(previous));
    }
    if let Some(owned_by) = owned_by {
        sequence.semantic.insert(
            "owned-by".to_string(),
            SemanticValue::Text(owned_by.clone()),
        );
        sequence.dependencies.insert(ObjectId::new(owned_by));
    }
    sequence.source = Some(source_location(document, statement));
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

fn validate_owner(
    sequence: &[String],
    owner: &[String],
    objects: &BTreeMap<ObjectId, SchemaObject>,
) -> Result<String, PostgresDiagnostic> {
    let (namespace, relation, column) = match owner {
        [relation, column] => ("public", relation, column),
        [namespace, relation, column] => (namespace.as_str(), relation, column),
        _ => {
            return Err(schema_error_message(
                "sequence owner must name a table column",
            ));
        }
    };
    let sequence_namespace = if sequence.len() == 1 {
        "public"
    } else {
        &sequence[0]
    };
    if namespace != sequence_namespace {
        return Err(schema_error_message(
            "sequence and owning table must be in the same schema",
        ));
    }
    let identity = format!("{namespace}.{relation}.{column}");
    if !objects
        .get(&ObjectId::new(&identity))
        .is_some_and(|object| object.kind == SchemaObjectKind::Column)
        || !objects
            .get(&ObjectId::new(format!("{namespace}.{relation}")))
            .is_some_and(|object| object.kind == SchemaObjectKind::Table)
    {
        return Err(schema_error_message(format!(
            "sequence owner '{identity}' is not a table column"
        )));
    }
    Ok(identity)
}

/// Resolve the sequence named by a direct `nextval` column default.
pub fn sequence_default_reference(
    expression: &crate::ast::Expression,
) -> Result<Option<String>, &'static str> {
    use crate::ast::ExpressionKind;
    let ExpressionKind::Function {
        name, arguments, ..
    } = &expression.kind
    else {
        return Ok(None);
    };
    if !matches!(name.as_slice(), [function] if function == "nextval")
        && !matches!(name.as_slice(), [schema, function] if schema == "pg_catalog" && function == "nextval")
    {
        return Ok(None);
    }
    let [argument] = arguments.as_slice() else {
        return Err("nextval default must have one sequence argument");
    };
    let value = match &argument.kind {
        ExpressionKind::String { value } => value,
        ExpressionKind::Cast { expression, ty }
            if ty.path.last().is_some_and(|name| name == "regclass") =>
        {
            let ExpressionKind::String { value } = &expression.kind else {
                return Err("nextval default must name a fixed sequence");
            };
            value
        }
        _ => return Err("nextval default must name a fixed sequence"),
    };
    let parts = value.split('.').collect::<Vec<_>>();
    if !matches!(parts.as_slice(), [_] | [_, _])
        || parts.iter().any(|part| {
            let mut bytes = part.bytes();
            !bytes
                .next()
                .is_some_and(|byte| byte.is_ascii_lowercase() || byte == b'_')
                || !bytes
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        })
    {
        return Err(
            "nextval default needs an unquoted sequence name in public or an explicit schema",
        );
    }
    Ok(Some(if parts.len() == 1 {
        format!("public.{}", parts[0])
    } else {
        value.clone()
    }))
}
