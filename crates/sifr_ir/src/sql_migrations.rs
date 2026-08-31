use sifr_type_system::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HirSqlMigrationStepKind {
    Ddl,
    SqlData,
    SifrData,
    Assertion,
    Backfill,
    Transaction,
    RecoveryPoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HirSqlMigrationStep {
    pub migration_identity: String,
    pub parent_identity: String,
    pub step_identity: String,
    pub input_state_identity: String,
    pub output_state_identity: String,
    pub input_plan_type: Type,
    pub output_plan_type: Type,
    pub callback_db_type: Option<Type>,
    pub referenced_objects: Vec<String>,
    pub affected_objects: Vec<String>,
    pub kind: HirSqlMigrationStepKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HirSqlMigrationGraph {
    pub provider_family: String,
    pub head: String,
    pub target_fingerprint: String,
    pub steps: Vec<HirSqlMigrationStep>,
}
