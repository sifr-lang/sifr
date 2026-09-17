use super::{
    HirSqlMigrationGraph, HirSqlMigrationStep, HirSqlMigrationStepKind, Record, RecordId,
    References, sealed,
};
impl References for HirSqlMigrationStepKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Ddl => {}
            Self::SqlData => {}
            Self::SifrData => {}
            Self::Assertion => {}
            Self::Backfill => {}
            Self::Transaction => {}
            Self::RecoveryPoint => {}
        }
    }
}
impl sealed::Sealed for HirSqlMigrationStepKind {}
impl Record for HirSqlMigrationStepKind {
    const KIND: u16 = 50;
}
impl References for HirSqlMigrationStep {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.migration_identity.references(out);
        self.parent_identity.references(out);
        self.step_identity.references(out);
        self.input_state_identity.references(out);
        self.output_state_identity.references(out);
        self.input_plan_type.references(out);
        self.output_plan_type.references(out);
        self.callback_db_type.references(out);
        self.referenced_objects.references(out);
        self.affected_objects.references(out);
        self.kind.references(out);
    }
}
impl sealed::Sealed for HirSqlMigrationStep {}
impl Record for HirSqlMigrationStep {
    const KIND: u16 = 49;
}
impl References for HirSqlMigrationGraph {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.provider_family.references(out);
        self.head.references(out);
        self.target_fingerprint.references(out);
        self.steps.references(out);
    }
}
impl sealed::Sealed for HirSqlMigrationGraph {}
impl Record for HirSqlMigrationGraph {
    const KIND: u16 = 48;
}
