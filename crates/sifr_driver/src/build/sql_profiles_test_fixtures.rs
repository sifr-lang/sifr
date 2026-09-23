use sifr_sql_contract::{
    ObjectId, SchemaObject, SchemaObjectKind, SchemaSourceLocation, SemanticValue,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn schema_object(
    identity: &str,
    kind: SchemaObjectKind,
    dependencies: BTreeSet<ObjectId>,
) -> SchemaObject {
    SchemaObject {
        identity: ObjectId::new(identity),
        kind,
        semantic: BTreeMap::new(),
        dependencies,
        source: Some(SchemaSourceLocation {
            document: "db/schema.sql".to_string(),
            start: 0,
            end: 21,
        }),
    }
}

pub(super) fn generated_scalar_object() -> SchemaObject {
    let mut object = schema_object(
        "public.scalar_samples",
        SchemaObjectKind::Composite,
        BTreeSet::from([ObjectId::new("public")]),
    );
    object.semantic.insert(
        "fields".to_string(),
        SemanticValue::Map(
            BTreeMap::from([
                ("created_on".to_string(), "date".to_string()),
                ("local_time".to_string(), "time".to_string()),
                ("offset_time".to_string(), "OffsetTime".to_string()),
                ("local_timestamp".to_string(), "datetime".to_string()),
                ("instant".to_string(), "Instant".to_string()),
                ("identifier".to_string(), "UUID".to_string()),
                ("document".to_string(), "JsonValue".to_string()),
                ("address".to_string(), "IPAddress".to_string()),
                ("network".to_string(), "IPNetwork".to_string()),
                ("hardware_address".to_string(), "MacAddress".to_string()),
            ])
            .into_iter()
            .map(|(name, ty)| (name, SemanticValue::Text(ty)))
            .collect(),
        ),
    );
    object
}

pub(super) fn generated_enum_object() -> SchemaObject {
    let mut object = schema_object(
        "public.mood",
        SchemaObjectKind::Enum,
        BTreeSet::from([ObjectId::new("public")]),
    );
    object.semantic.insert(
        "variants".to_string(),
        SemanticValue::List(vec![
            SemanticValue::Text("happy".to_string()),
            SemanticValue::Text("sad".to_string()),
        ]),
    );
    object
}
