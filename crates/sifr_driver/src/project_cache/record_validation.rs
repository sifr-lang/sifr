//! Successful immutable-record facts, bounded by one storage owner's lifetime.
//! Every lookup follows a fresh owned-file read. A bounded exact-byte proof or
//! observed digest ties that read to prior validation; no decoded result is reused.
use super::storage::{RECORD_COUNT, invalid, stamp};
use sifr_frontend::persistence::CompletedCheck;
use sifr_identity::IdentityEncoder;
use std::{collections::VecDeque, io, sync::Mutex};

const BYTE_BUDGET: usize = 2 * 1024 * 1024;

struct Fact {
    stored: String,
    observed: String,
    bytes: Option<Box<[u8]>>,
}
#[derive(Default)]
struct Entries {
    facts: VecDeque<Fact>,
    retained_bytes: usize,
    #[cfg(test)]
    hits: usize,
    #[cfg(test)]
    validations: usize,
}
pub(super) struct RecordValidation {
    context: String,
    capacity: usize,
    byte_budget: usize,
    entries: Mutex<Entries>,
}
impl RecordValidation {
    pub(super) fn new(context: String) -> Self {
        Self::with_capacity(context, RECORD_COUNT)
    }
    fn with_capacity(context: String, capacity: usize) -> Self {
        Self::with_limits(context, capacity, BYTE_BUDGET)
    }
    fn with_limits(context: String, capacity: usize, byte_budget: usize) -> Self {
        Self {
            context,
            capacity,
            byte_budget,
            entries: Mutex::new(Entries::default()),
        }
    }
    pub(super) fn validate(&self, stored: &str, bytes: &[u8], mismatch: &str) -> io::Result<()> {
        let mut observed = None;
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| io::Error::other("project record validation owner is poisoned"))?;
        if let Some(index) = entries.facts.iter().position(|fact| {
            fact.stored == stored
                && match &fact.bytes {
                    Some(validated) => validated.as_ref() == bytes,
                    None => {
                        fact.observed == *observed.get_or_insert_with(|| observed_digest(bytes))
                    }
                }
        }) && let Some(fact) = entries.facts.remove(index)
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
                if let Some(fact) = entries.facts.pop_front() {
                    entries.retained_bytes -= fact.bytes.as_ref().map_or(0, |bytes| bytes.len());
                }
            }
            // Keep digest facts even when the byte budget evicts their exact-byte
            // proofs. Large records retain the existing digest-only validation.
            let retained = if bytes.len() <= self.byte_budget {
                let mut retained_bytes = entries.retained_bytes;
                for fact in &mut entries.facts {
                    if retained_bytes <= self.byte_budget - bytes.len() {
                        break;
                    }
                    if let Some(previous) = fact.bytes.take() {
                        retained_bytes -= previous.len();
                    }
                }
                entries.retained_bytes = retained_bytes + bytes.len();
                Some(bytes.into())
            } else {
                None
            };
            entries.facts.push_back(Fact {
                stored: stored.to_owned(),
                observed: observed.unwrap_or_else(|| observed_digest(bytes)),
                bytes: retained,
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

fn observed_digest(bytes: &[u8]) -> String {
    let mut digest = IdentityEncoder::new("project-record-observed-bytes-v1");
    digest.field("record", bytes);
    digest.finish()
}

#[cfg(test)]
mod tests;
