use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectId(String);

impl ObjectId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderIdentity {
    pub package_id: String,
    pub package_version: Version,
    pub package_source: String,
    pub package_graph_digest: String,
    pub compiler_components: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialectIdentity {
    pub family: String,
    pub server_version: String,
    pub modes: BTreeMap<String, String>,
    pub features: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaObjectKind {
    Catalog,
    Namespace,
    Table,
    Column,
    PrimaryKey,
    UniqueConstraint,
    ForeignKey,
    CheckConstraint,
    Index,
    Sequence,
    IdentityColumn,
    View,
    MaterializedView,
    Enum,
    Domain,
    Composite,
    Array,
    Range,
    Function,
    Operator,
    Cast,
    Collation,
    CharacterSet,
    Extension,
    Trigger,
    ServerCapability,
    DialectMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SemanticValue {
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Text(String),
    BytesHex(String),
    List(Vec<SemanticValue>),
    Set(BTreeSet<SemanticValue>),
    Map(BTreeMap<String, SemanticValue>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaSourceLocation {
    pub document: String,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaObject {
    pub identity: ObjectId,
    pub kind: SchemaObjectKind,
    pub semantic: BTreeMap<String, SemanticValue>,
    pub dependencies: BTreeSet<ObjectId>,
    pub source: Option<SchemaSourceLocation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaIr {
    pub format_version: u32,
    pub provider: ProviderIdentity,
    pub dialect: DialectIdentity,
    pub objects: BTreeMap<ObjectId, SchemaObject>,
}

impl SchemaIr {
    #[must_use]
    pub fn object(&self, identity: &ObjectId) -> Option<&SchemaObject> {
        self.objects.get(identity)
    }

    pub fn resolve_symbol(&self, name: &str) -> Result<&SchemaObject, crate::SchemaContractError> {
        use crate::{SchemaContractError, SchemaContractErrorKind};
        if name.contains('.') {
            return self.objects.get(&ObjectId::new(name)).ok_or_else(|| {
                SchemaContractError::new(
                    SchemaContractErrorKind::UnknownSymbol,
                    format!("schema symbol '{name}' does not exist"),
                )
            });
        }
        let suffix = format!(".{name}");
        let mut matches = self.objects.iter().filter(|(identity, _)| {
            identity.as_str() == name || identity.as_str().ends_with(&suffix)
        });
        let Some((_, first)) = matches.next() else {
            return Err(SchemaContractError::new(
                SchemaContractErrorKind::UnknownSymbol,
                format!("schema symbol '{name}' does not exist"),
            ));
        };
        if matches.next().is_some() {
            return Err(SchemaContractError::new(
                SchemaContractErrorKind::AmbiguousSymbol,
                format!("schema symbol '{name}' is ambiguous; use a qualified name"),
            ));
        }
        Ok(first)
    }
}
