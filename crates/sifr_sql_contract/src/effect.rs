use crate::{ObjectId, SchemaContractError, SchemaContractErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryEffect {
    Read,
    Write,
    ReadWrite,
    SchemaChange,
    SessionChange,
    TransactionControl,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectContract {
    pub effect: QueryEffect,
    pub referenced_objects: BTreeSet<ObjectId>,
    pub affected_objects: BTreeSet<ObjectId>,
}

impl EffectContract {
    pub fn new(
        effect: QueryEffect,
        referenced_objects: BTreeSet<ObjectId>,
        affected_objects: BTreeSet<ObjectId>,
    ) -> Result<Self, SchemaContractError> {
        let contract = Self {
            effect,
            referenced_objects,
            affected_objects,
        };
        contract.validate()?;
        Ok(contract)
    }

    pub fn validate(&self) -> Result<(), SchemaContractError> {
        if matches!(self.effect, QueryEffect::Read) && !self.affected_objects.is_empty() {
            return Err(invalid("read effect cannot affect schema objects"));
        }
        if matches!(
            self.effect,
            QueryEffect::Write | QueryEffect::ReadWrite | QueryEffect::SchemaChange
        ) && self.affected_objects.is_empty()
        {
            return Err(invalid("effect must identify an affected schema object"));
        }
        if matches!(
            self.effect,
            QueryEffect::SessionChange | QueryEffect::TransactionControl
        ) && (!self.referenced_objects.is_empty() || !self.affected_objects.is_empty())
        {
            return Err(invalid(
                "control effects cannot claim common read or write object sets",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn application_safe(&self) -> bool {
        matches!(
            self.effect,
            QueryEffect::Read | QueryEffect::Write | QueryEffect::ReadWrite
        )
    }
}

fn invalid(message: impl Into<String>) -> SchemaContractError {
    SchemaContractError::new(SchemaContractErrorKind::InvalidSchema, message)
}
