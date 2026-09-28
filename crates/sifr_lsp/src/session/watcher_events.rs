use super::Session;
use sifr_analysis::WorkspaceTracePhase;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WatcherEventKind {
    Created,
    Changed,
    Deleted,
}

#[derive(Default)]
struct WatcherCounts {
    created: usize,
    changed: usize,
    deleted: usize,
}

impl WatcherCounts {
    fn add(&mut self, kind: WatcherEventKind) {
        let count = match kind {
            WatcherEventKind::Created => &mut self.created,
            WatcherEventKind::Changed => &mut self.changed,
            WatcherEventKind::Deleted => &mut self.deleted,
        };
        *count = count.saturating_add(1);
    }

    fn total(&self) -> usize {
        self.created
            .saturating_add(self.changed)
            .saturating_add(self.deleted)
    }
}

pub(crate) struct WatcherEvent {
    pub(crate) path: PathBuf,
    pub(crate) kind: WatcherEventKind,
}

impl WatcherEvent {
    pub(crate) fn from_protocol(uri: &str, kind: u64) -> Option<Self> {
        let path = Url::parse(uri).ok()?.to_file_path().ok()?;
        let kind = match kind {
            1 => WatcherEventKind::Created,
            2 => WatcherEventKind::Changed,
            3 => WatcherEventKind::Deleted,
            _ => return None,
        };
        Some(Self { path, kind })
    }
}

impl Session {
    pub(crate) fn revalidate_open_external_inputs(&mut self) {
        self.revalidate_open_external_inputs_where(|_| true);
    }

    pub(crate) fn revalidate_open_external_inputs_where(
        &mut self,
        mut needs_revalidation: impl FnMut(&Path) -> bool,
    ) {
        let paths: Vec<_> = self
            .store()
            .documents()
            .map(|document| document.path().to_path_buf())
            .filter(|path| needs_revalidation(path))
            .collect();
        for path in paths {
            self.observe_external_inputs_for_path(&path);
        }
    }

    pub(crate) fn record_watcher_file_events(&mut self, events: &[WatcherEvent]) {
        let mut counts = BTreeMap::<PathBuf, WatcherCounts>::new();
        for event in events {
            // A delete can remove the manifest that established the previous owner.
            let previous = self
                .external_inputs
                .owning_root(&event.path)
                .map(Path::to_path_buf);
            let mut provider = sifr_analysis::DiskSourceProvider::new();
            let current = crate::python_declarations::package_root_for(&event.path, &mut provider);
            for root in [previous, current].into_iter().flatten() {
                if !self
                    .store()
                    .documents()
                    .any(|document| document.path().starts_with(&root))
                {
                    continue;
                }
                counts.entry(root).or_default().add(event.kind);
            }
        }
        for (root, counts) in counts {
            let count = counts.total();
            self.observe_external_root(&root);
            self.python_declarations.invalidate_external_root(&root);
            self.analysis.record_watcher_events_for_root(&root, count);
            self.trace(
                WorkspaceTracePhase::SourceUpdate,
                format!(
                    "workspace_watcher_events root={} count={count} create={} change={} delete={}",
                    root.display(),
                    counts.created,
                    counts.changed,
                    counts.deleted
                ),
            );
        }
    }
}
