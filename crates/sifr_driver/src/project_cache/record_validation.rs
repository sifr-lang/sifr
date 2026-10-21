//! Successful immutable-record facts, bounded by one storage owner's lifetime.
//! Every lookup follows a fresh owned-file read and hashes the observed bytes.
//! Entries retain only identities; payloads, decoded records and failures are not cached.
use super::storage::{RECORD_COUNT, invalid, stamp};
use sifr_frontend::persistence::CompletedCheck;
use sifr_identity::IdentityEncoder;
use std::{collections::VecDeque, io, sync::Mutex};

struct Fact {
    stored: String,
    observed: String,
}
#[derive(Default)]
struct Entries {
    facts: VecDeque<Fact>,
    #[cfg(test)]
    hits: usize,
    #[cfg(test)]
    validations: usize,
}
pub(super) struct RecordValidation {
    context: String,
    capacity: usize,
    entries: Mutex<Entries>,
}
impl RecordValidation {
    pub(super) fn new(context: String) -> Self {
        Self::with_capacity(context, RECORD_COUNT)
    }
    fn with_capacity(context: String, capacity: usize) -> Self {
        Self {
            context,
            capacity,
            entries: Mutex::new(Entries::default()),
        }
    }
    pub(super) fn validate(&self, stored: &str, bytes: &[u8], mismatch: &str) -> io::Result<()> {
        let mut digest = IdentityEncoder::new("project-record-observed-bytes-v1");
        digest.field("record", bytes);
        let observed = digest.finish();
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| io::Error::other("project record validation owner is poisoned"))?;
        if let Some(index) = entries
            .facts
            .iter()
            .position(|fact| fact.stored == stored && fact.observed == observed)
            && let Some(fact) = entries.facts.remove(index)
        {
            entries.facts.push_back(fact);
            #[cfg(test)]
            {
                entries.hits += 1;
            }
            return Ok(());
        }
        #[cfg(test)]
        {
            entries.validations += 1;
        }
        // Preserve the existing on-disk digest, including its byte-vector encoding.
        if stamp("project-record-v1", &bytes)? != stored {
            return Err(invalid(mismatch));
        }
        let record: CompletedCheck = serde_json::from_slice(bytes)?;
        if record.schema != 1
            || record
                .result
                .inputs
                .semantic_inputs
                .identity()
                .ok()
                .as_deref()
                != Some(self.context.as_str())
            || record.diagnostics().is_err()
        {
            return Err(invalid("incomplete project record"));
        }
        if self.capacity != 0 {
            while entries.facts.len() >= self.capacity {
                entries.facts.pop_front();
            }
            entries.facts.push_back(Fact {
                stored: stored.to_owned(),
                observed,
            });
        }
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn stats(&self) -> (usize, usize, usize) {
        let entries = self.entries.lock().expect("test validation owner");
        (entries.hits, entries.validations, entries.facts.len())
    }
}

#[cfg(test)]
mod tests;
