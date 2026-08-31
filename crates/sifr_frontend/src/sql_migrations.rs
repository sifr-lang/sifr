use sifr_ir::{HirSqlMigrationGraph, HirSqlMigrationStep, HirSqlMigrationStepKind};
use sifr_sql_contract::{
    CompiledMigrationGraph, CompiledStepKind, MigrationCompileError, MigrationCompileErrorKind,
};
use sifr_type_system::Type;

pub fn lower_migration_graph(
    graph: &CompiledMigrationGraph,
) -> Result<HirSqlMigrationGraph, MigrationCompileError> {
    let mut steps = Vec::new();
    for migration_id in &graph.topological_order {
        let migration = graph
            .migrations
            .get(migration_id)
            .ok_or_else(|| invalid("compiled migration is missing"))?;
        for path in migration.paths.values() {
            for step in &path.steps {
                if step.input_state.as_str().is_empty() || step.output_state.as_str().is_empty() {
                    return Err(invalid(
                        "compiled migration step has an empty nominal state",
                    ));
                }
                let input_state = nominal_type(step.input_state.as_str(), Vec::new());
                let output_state = nominal_type(step.output_state.as_str(), Vec::new());
                let callback_db_type =
                    matches!(step.kind, CompiledStepKind::SifrData { .. }).then(|| {
                        nominal_type("sifr.sql.migration.MigrationDb", vec![input_state.clone()])
                    });
                steps.push(HirSqlMigrationStep {
                    migration_identity: migration.id.to_string(),
                    parent_identity: path.parent.to_string(),
                    step_identity: step.id.to_string(),
                    input_state_identity: step.input_state.as_str().to_string(),
                    output_state_identity: step.output_state.as_str().to_string(),
                    input_plan_type: nominal_type(
                        "sifr.sql.migration.MigrationPlan",
                        vec![input_state],
                    ),
                    output_plan_type: nominal_type(
                        "sifr.sql.migration.MigrationPlan",
                        vec![output_state],
                    ),
                    callback_db_type,
                    referenced_objects: step
                        .referenced_objects
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    affected_objects: step
                        .affected_objects
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    kind: lower_kind(&step.kind),
                });
            }
        }
    }
    Ok(HirSqlMigrationGraph {
        provider_family: graph.provider_family.clone(),
        head: graph.head.to_string(),
        target_fingerprint: graph.target_fingerprint.clone(),
        steps,
    })
}

fn lower_kind(kind: &CompiledStepKind) -> HirSqlMigrationStepKind {
    match kind {
        CompiledStepKind::ReflectedDdl { .. } | CompiledStepKind::DeclaredDdl { .. } => {
            HirSqlMigrationStepKind::Ddl
        }
        CompiledStepKind::SqlData { .. } => HirSqlMigrationStepKind::SqlData,
        CompiledStepKind::SifrData { .. } => HirSqlMigrationStepKind::SifrData,
        CompiledStepKind::Assertion { .. } => HirSqlMigrationStepKind::Assertion,
        CompiledStepKind::Backfill { .. } => HirSqlMigrationStepKind::Backfill,
        CompiledStepKind::Transaction { .. } => HirSqlMigrationStepKind::Transaction,
        CompiledStepKind::RecoveryPoint { .. } => HirSqlMigrationStepKind::RecoveryPoint,
    }
}

fn nominal_type(identity: &str, type_args: Vec<Type>) -> Type {
    Type::Class {
        identity: Some(identity.to_string()),
        type_args,
        name: identity.rsplit('.').next().unwrap_or(identity).to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    }
}

fn invalid(message: impl Into<String>) -> MigrationCompileError {
    MigrationCompileError {
        kind: MigrationCompileErrorKind::InvalidStep,
        message: message.into(),
    }
}
