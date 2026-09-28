use super::LspAnalysisWorkspace;
use std::path::Path;

impl LspAnalysisWorkspace {
    pub(crate) fn record_watcher_events_for_root(&mut self, root: &Path, event_count: usize) {
        if let Some(analysis) = self.projects.get_mut(root) {
            if let Some(host) = analysis.host.as_mut() {
                host.record_watcher_events(event_count, Self::WATCHER_STORM_THRESHOLD);
            }
            analysis.refresh_external_state();
        }
        for (uri, analysis) in &mut self.documents {
            if url::Url::parse(uri)
                .ok()
                .and_then(|uri| uri.to_file_path().ok())
                .is_some_and(|path| path.starts_with(root))
            {
                if let Some(host) = analysis.host.as_mut() {
                    host.record_watcher_events(event_count, Self::WATCHER_STORM_THRESHOLD);
                }
            }
        }
    }
}
