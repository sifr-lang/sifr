mod compiler;
mod graph;
mod plan;
mod types;

pub use compiler::{MigrationCompiler, MigrationDialect};
pub use graph::topological_order;
pub use plan::{MigrationDb, MigrationPlan, MigrationState};
pub use types::{
    BackfillContract, CompiledMigration, CompiledMigrationGraph, CompiledMigrationPath,
    CompiledMigrationStep, CompiledStepKind, DataCallbackContract, DdlReflection, DdlRisk,
    MigrationBaseline, MigrationCompileError, MigrationCompileErrorKind, MigrationDefinition,
    MigrationGraphDefinition, MigrationImpact, MigrationNodeId, MigrationProviderConstraint,
    MigrationSourceDeclaration, MigrationSourceStep, MigrationSourceStepKind,
    MigrationStateIdentity, MigrationStepDefinition, MigrationStepKind, ReplayPolicy,
    TransactionBoundary, TransactionRequirement,
};

pub const MIGRATION_GRAPH_FORMAT_VERSION: u32 = 2;
