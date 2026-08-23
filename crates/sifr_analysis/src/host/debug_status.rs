use super::implementation::AnalysisHost;
use crate::AnalysisRevision;
use crate::symbols::{SymbolBucketKind, SymbolBucketReadinessState};
use sifr_frontend::{WorkspaceDebugSnapshot, WorkspaceIndexReadinessStatus};

impl AnalysisHost {
    pub fn debug_snapshot(&mut self) -> WorkspaceDebugSnapshot {
        let mut debug = self.session.snapshot().debug.as_ref().clone();
        debug.status.index_readiness = self.symbol_index.as_ref().map_or_else(
            || unavailable_readiness(self.current_revision),
            |index| {
                index
                    .bucket_readiness()
                    .into_iter()
                    .map(|readiness| WorkspaceIndexReadinessStatus {
                        bucket: format!(
                            "{:?}:{}",
                            readiness.id.kind,
                            readiness.id.module.map_or_else(
                                || "-".to_string(),
                                |module| { module.as_u32().to_string() }
                            )
                        ),
                        readiness: format!("{:?}", readiness.state),
                    })
                    .collect()
            },
        );
        debug
    }
}

fn unavailable_readiness(_revision: AnalysisRevision) -> Vec<WorkspaceIndexReadinessStatus> {
    [
        SymbolBucketKind::Workspace,
        SymbolBucketKind::Package,
        SymbolBucketKind::Stdlib,
    ]
    .into_iter()
    .map(|kind| WorkspaceIndexReadinessStatus {
        bucket: format!("{kind:?}:-"),
        readiness: format!("{:?}", SymbolBucketReadinessState::Unavailable),
    })
    .collect()
}
