//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::{Ref, Text, Type};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirSqlMigrationStepKind {
    Ddl,
    SqlData,
    SifrData,
    Assertion,
    Backfill,
    Transaction,
    RecoveryPoint,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirSqlMigrationStep {
    pub migration_identity: Ref<Text>,
    pub parent_identity: Ref<Text>,
    pub step_identity: Ref<Text>,
    pub input_state_identity: Ref<Text>,
    pub output_state_identity: Ref<Text>,
    pub input_plan_type: Ref<Type>,
    pub output_plan_type: Ref<Type>,
    pub callback_db_type: Option<Ref<Type>>,
    pub referenced_objects: Vec<Ref<Text>>,
    pub affected_objects: Vec<Ref<Text>>,
    pub kind: Ref<HirSqlMigrationStepKind>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirSqlMigrationGraph {
    pub provider_family: Ref<Text>,
    pub head: Ref<Text>,
    pub target_fingerprint: Ref<Text>,
    pub steps: Vec<Ref<HirSqlMigrationStep>>,
}
