use crate::{ObjectId, SchemaIr, SchemaObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectChangeKind {
    Added,
    Removed,
    Changed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectChange {
    pub identity: ObjectId,
    pub kind: ObjectChangeKind,
    pub before: Option<SchemaObject>,
    pub after: Option<SchemaObject>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaDiff {
    pub provider_changed: bool,
    pub dialect_changed: bool,
    pub objects: Vec<ObjectChange>,
}

impl SchemaDiff {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.provider_changed && !self.dialect_changed && self.objects.is_empty()
    }
}

#[must_use]
pub fn semantic_diff(before: &SchemaIr, after: &SchemaIr) -> SchemaDiff {
    let mut identities = before
        .objects
        .keys()
        .chain(after.objects.keys())
        .collect::<Vec<_>>();
    identities.sort();
    identities.dedup();
    let objects = identities
        .into_iter()
        .filter_map(
            |identity| match (before.objects.get(identity), after.objects.get(identity)) {
                (None, Some(after)) => Some(ObjectChange {
                    identity: identity.clone(),
                    kind: ObjectChangeKind::Added,
                    before: None,
                    after: Some(after.clone()),
                }),
                (Some(before), None) => Some(ObjectChange {
                    identity: identity.clone(),
                    kind: ObjectChangeKind::Removed,
                    before: Some(before.clone()),
                    after: None,
                }),
                (Some(before), Some(after)) if !same_semantics(before, after) => {
                    Some(ObjectChange {
                        identity: identity.clone(),
                        kind: ObjectChangeKind::Changed,
                        before: Some(before.clone()),
                        after: Some(after.clone()),
                    })
                }
                _ => None,
            },
        )
        .collect();
    SchemaDiff {
        provider_changed: before.provider != after.provider,
        dialect_changed: before.dialect != after.dialect,
        objects,
    }
}

fn same_semantics(before: &SchemaObject, after: &SchemaObject) -> bool {
    before.identity == after.identity
        && before.kind == after.kind
        && before.semantic == after.semantic
        && before.dependencies == after.dependencies
}
