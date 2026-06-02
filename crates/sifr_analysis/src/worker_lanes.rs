#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovedWorkerLane {
    Parse,
    SourceMapCreation,
    IndependentHirLower,
    LintFileRules,
    FormatterChecks,
    SelectedDiagnostics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SingleOwnerCompilerPhase {
    TypeIdentityCreation,
    OwnershipMutation,
    PackageGraphMutation,
    CodegenState,
}

pub const APPROVED_WORKER_LANES: &[ApprovedWorkerLane] = &[
    ApprovedWorkerLane::Parse,
    ApprovedWorkerLane::SourceMapCreation,
    ApprovedWorkerLane::IndependentHirLower,
    ApprovedWorkerLane::LintFileRules,
    ApprovedWorkerLane::FormatterChecks,
    ApprovedWorkerLane::SelectedDiagnostics,
];

pub const SINGLE_OWNER_PHASES: &[SingleOwnerCompilerPhase] = &[
    SingleOwnerCompilerPhase::TypeIdentityCreation,
    SingleOwnerCompilerPhase::OwnershipMutation,
    SingleOwnerCompilerPhase::PackageGraphMutation,
    SingleOwnerCompilerPhase::CodegenState,
];

#[cfg(test)]
mod tests {
    use super::{
        ApprovedWorkerLane, SingleOwnerCompilerPhase, APPROVED_WORKER_LANES, SINGLE_OWNER_PHASES,
    };
    use std::collections::BTreeSet;

    #[test]
    fn approved_lanes_exclude_single_owner_compiler_state() {
        assert!(APPROVED_WORKER_LANES.contains(&ApprovedWorkerLane::Parse));
        assert!(APPROVED_WORKER_LANES.contains(&ApprovedWorkerLane::SourceMapCreation));
        assert!(APPROVED_WORKER_LANES.contains(&ApprovedWorkerLane::IndependentHirLower));
        assert!(APPROVED_WORKER_LANES.contains(&ApprovedWorkerLane::LintFileRules));
        assert!(APPROVED_WORKER_LANES.contains(&ApprovedWorkerLane::FormatterChecks));
        assert!(APPROVED_WORKER_LANES.contains(&ApprovedWorkerLane::SelectedDiagnostics));
        assert!(SINGLE_OWNER_PHASES.contains(&SingleOwnerCompilerPhase::TypeIdentityCreation));
        assert!(SINGLE_OWNER_PHASES.contains(&SingleOwnerCompilerPhase::OwnershipMutation));
        assert!(SINGLE_OWNER_PHASES.contains(&SingleOwnerCompilerPhase::PackageGraphMutation));
        assert!(SINGLE_OWNER_PHASES.contains(&SingleOwnerCompilerPhase::CodegenState));

        let lane_names = APPROVED_WORKER_LANES
            .iter()
            .map(|lane| format!("{lane:?}"))
            .collect::<BTreeSet<_>>();
        let single_owner_names = SINGLE_OWNER_PHASES
            .iter()
            .map(|phase| format!("{phase:?}"))
            .collect::<BTreeSet<_>>();
        assert!(lane_names.is_disjoint(&single_owner_names));
    }
}
