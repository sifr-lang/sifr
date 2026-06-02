use serde_json::{json, Value};

const DIAGNOSTICS_PROGRESS_DOCUMENT_THRESHOLD: usize = 2;
const REFERENCES_PROGRESS_LOCATION_THRESHOLD: usize = 8;
const INDEX_WARMING_PROGRESS_UNIT_THRESHOLD: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ProgressKind {
    FullDiagnostics,
    References,
    IndexWarming,
    WorkspaceLoad,
}

impl ProgressKind {
    #[cfg(test)]
    fn title(self) -> &'static str {
        match self {
            Self::FullDiagnostics => "Checking Sifr workspace",
            Self::References => "Finding Sifr references",
            Self::IndexWarming => "Warming Sifr index",
            Self::WorkspaceLoad => "Loading Sifr workspace",
        }
    }

    fn threshold(self) -> usize {
        match self {
            Self::FullDiagnostics | Self::WorkspaceLoad => DIAGNOSTICS_PROGRESS_DOCUMENT_THRESHOLD,
            Self::References => REFERENCES_PROGRESS_LOCATION_THRESHOLD,
            Self::IndexWarming => INDEX_WARMING_PROGRESS_UNIT_THRESHOLD,
        }
    }

    fn token_prefix(self) -> &'static str {
        match self {
            Self::FullDiagnostics => "sifr/full-diagnostics",
            Self::References => "sifr/references",
            Self::IndexWarming => "sifr/index-warming",
            Self::WorkspaceLoad => "sifr/workspace-load",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProgressHandle {
    token: String,
}

impl ProgressHandle {
    pub(crate) fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Default)]
pub(crate) struct ProgressState {
    enabled: bool,
    next_token: u64,
    #[cfg(test)]
    events: Vec<ProgressEvent>,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProgressEvent {
    token: String,
    phase: ProgressPhase,
    title: String,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProgressPhase {
    Begin,
    End,
}

impl ProgressState {
    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(crate) fn begin(
        &mut self,
        kind: ProgressKind,
        work_units: usize,
    ) -> Option<ProgressHandle> {
        if !self.enabled || work_units < kind.threshold() {
            return None;
        }
        let token = format!("{}-{}", kind.token_prefix(), self.next_token);
        self.next_token = self.next_token.saturating_add(1);
        #[cfg(test)]
        {
            self.events.push(ProgressEvent {
                token: token.clone(),
                phase: ProgressPhase::Begin,
                title: kind.title().to_string(),
            });
        }
        Some(ProgressHandle { token })
    }

    #[cfg_attr(not(test), allow(clippy::unused_self))]
    pub(crate) fn end(&mut self, handle: ProgressHandle, title: &str) {
        #[cfg(not(test))]
        {
            let _ = (&handle, title);
        }
        #[cfg(test)]
        {
            self.events.push(ProgressEvent {
                token: handle.token,
                phase: ProgressPhase::End,
                title: title.to_string(),
            });
        }
    }

    #[cfg(test)]
    pub(crate) fn events(&self) -> &[ProgressEvent] {
        &self.events
    }
}

pub(crate) fn begin_notification(handle: &ProgressHandle, title: &str) -> Value {
    json!({
        "token": handle.token(),
        "value": {
            "kind": "begin",
            "title": title,
        }
    })
}

pub(crate) fn end_notification(handle: &ProgressHandle, message: &str) -> Value {
    json!({
        "token": handle.token(),
        "value": {
            "kind": "end",
            "message": message,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{ProgressKind, ProgressState};

    #[test]
    fn progress_stays_quiet_for_fast_path_work() {
        let mut progress = ProgressState::default();
        progress.set_enabled(true);

        assert!(progress.begin(ProgressKind::FullDiagnostics, 1).is_none());
        assert!(progress.events().is_empty());
    }

    #[test]
    fn progress_records_begin_and_end_after_delay_gate() {
        let mut progress = ProgressState::default();
        progress.set_enabled(true);

        let handle = progress
            .begin(ProgressKind::FullDiagnostics, 2)
            .expect("threshold should start progress");
        progress.end(handle, "checked 2 document(s)");

        assert_eq!(progress.events().len(), 2);
    }

    #[test]
    fn disabled_progress_does_not_record_events() {
        let mut progress = ProgressState::default();

        assert!(progress.begin(ProgressKind::References, 100).is_none());
        assert!(progress.events().is_empty());
    }
}
