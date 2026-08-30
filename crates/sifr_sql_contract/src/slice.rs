use crate::{
    ObjectId, SchemaContractError, SchemaContractErrorKind, SchemaIr, SchemaObjectKind,
    SemanticValue,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AbsenceFact {
    MissingObject {
        identity: ObjectId,
    },
    ExactOverloadSet {
        namespace: String,
        name: String,
        candidates: BTreeSet<ObjectId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaDependencyRequest {
    pub identity: ObjectId,
    pub properties: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectRequirement {
    pub identity: ObjectId,
    pub kind: SchemaObjectKind,
    pub properties: BTreeMap<String, SemanticValue>,
    pub dependencies: BTreeSet<ObjectId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaSlice {
    pub objects: BTreeMap<ObjectId, ObjectRequirement>,
    pub absence_facts: BTreeSet<AbsenceFact>,
}

pub fn minimum_schema_slice(
    schema: &SchemaIr,
    requests: impl IntoIterator<Item = SchemaDependencyRequest>,
    absence_facts: impl IntoIterator<Item = AbsenceFact>,
) -> Result<SchemaSlice, SchemaContractError> {
    let mut requested = BTreeMap::<ObjectId, BTreeSet<String>>::new();
    for request in requests {
        requested
            .entry(request.identity)
            .or_default()
            .extend(request.properties);
    }
    let mut queue = requested.keys().cloned().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    while let Some(identity) = queue.pop_front() {
        if !visited.insert(identity.clone()) {
            continue;
        }
        let object = schema.objects.get(&identity).ok_or_else(|| {
            SchemaContractError::new(
                SchemaContractErrorKind::UnknownSymbol,
                format!("schema dependency '{identity}' does not exist"),
            )
        })?;
        for dependency in &object.dependencies {
            let dependency_object = schema.objects.get(dependency).ok_or_else(|| {
                SchemaContractError::new(
                    SchemaContractErrorKind::MissingDependency,
                    format!("schema object '{identity}' depends on missing object '{dependency}'"),
                )
            })?;
            requested
                .entry(dependency.clone())
                .or_insert_with(|| dependency_object.semantic.keys().cloned().collect());
            queue.push_back(dependency.clone());
        }
    }
    let mut objects = BTreeMap::new();
    for (identity, properties) in requested {
        let object = schema.objects.get(&identity).ok_or_else(|| {
            SchemaContractError::new(
                SchemaContractErrorKind::UnknownSymbol,
                format!("schema dependency '{identity}' does not exist"),
            )
        })?;
        let selected = properties
            .into_iter()
            .map(|name| {
                object
                    .semantic
                    .get(&name)
                    .cloned()
                    .map(|value| (name.clone(), value))
                    .ok_or_else(|| {
                        SchemaContractError::new(
                            SchemaContractErrorKind::UnknownSymbol,
                            format!("schema object '{identity}' has no property '{name}'"),
                        )
                    })
            })
            .collect::<Result<_, _>>()?;
        objects.insert(
            identity.clone(),
            ObjectRequirement {
                identity,
                kind: object.kind,
                properties: selected,
                dependencies: object.dependencies.clone(),
            },
        );
    }
    let absence_facts = absence_facts.into_iter().collect();
    Ok(SchemaSlice {
        objects,
        absence_facts,
    })
}

pub fn verify_compatible_slice(
    observed: &SchemaIr,
    required: &SchemaSlice,
) -> Result<(), SchemaContractError> {
    for requirement in required.objects.values() {
        let Some(object) = observed.objects.get(&requirement.identity) else {
            return Err(incompatible(format!(
                "required schema object '{}' is missing",
                requirement.identity
            )));
        };
        if object.kind != requirement.kind || object.dependencies != requirement.dependencies {
            return Err(incompatible(format!(
                "required schema object '{}' changed kind or dependencies",
                requirement.identity
            )));
        }
        for (name, value) in &requirement.properties {
            if object.semantic.get(name) != Some(value) {
                return Err(incompatible(format!(
                    "required schema property '{}.{name}' changed",
                    requirement.identity
                )));
            }
        }
    }
    for fact in &required.absence_facts {
        match fact {
            AbsenceFact::MissingObject { identity } if observed.objects.contains_key(identity) => {
                return Err(incompatible(format!(
                    "schema object '{identity}' now exists but its absence was required"
                )));
            }
            AbsenceFact::ExactOverloadSet {
                namespace,
                name,
                candidates,
            } => {
                let prefix = format!("{namespace}.{name}.");
                let exact = format!("{namespace}.{name}");
                let observed_candidates = observed
                    .objects
                    .iter()
                    .filter(|(identity, object)| {
                        object.kind == SchemaObjectKind::Function
                            && (identity.as_str() == exact
                                || identity.as_str().starts_with(&prefix))
                    })
                    .map(|(identity, _)| identity.clone())
                    .collect::<BTreeSet<_>>();
                if observed_candidates != *candidates {
                    return Err(incompatible(format!(
                        "overload set '{namespace}.{name}' changed"
                    )));
                }
            }
            AbsenceFact::MissingObject { .. } => {}
        }
    }
    Ok(())
}

fn incompatible(message: impl Into<String>) -> SchemaContractError {
    SchemaContractError::new(SchemaContractErrorKind::IncompatibleSchema, message)
}
