use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScheduledDiagnosticJob {
    pub(crate) uri: String,
    pub(crate) version: Option<i32>,
    sequence: u64,
}

#[derive(Default)]
pub(crate) struct DiagnosticJobs {
    jobs: BTreeMap<String, ScheduledDiagnosticJob>,
    next_sequence: u64,
}

impl DiagnosticJobs {
    pub(crate) fn clear(&mut self) {
        self.jobs.clear();
    }

    pub(crate) fn remove(&mut self, uri: &str) {
        self.jobs.remove(uri);
    }

    pub(crate) fn schedule(&mut self, uri: &str, version: Option<i32>) -> ScheduledDiagnosticJob {
        let sequence = if let Some(existing) = self.jobs.get(uri) {
            existing.sequence
        } else {
            let sequence = self.next_sequence;
            self.next_sequence = self.next_sequence.saturating_add(1);
            sequence
        };
        let job = ScheduledDiagnosticJob {
            uri: uri.to_string(),
            version,
            sequence,
        };
        self.jobs.insert(uri.to_string(), job.clone());
        job
    }

    pub(crate) fn take_next(&mut self) -> Option<ScheduledDiagnosticJob> {
        let uri = self
            .jobs
            .iter()
            .min_by_key(|(_, job)| job.sequence)
            .map(|(uri, _)| uri.clone())?;
        self.jobs.remove(&uri)
    }
}
