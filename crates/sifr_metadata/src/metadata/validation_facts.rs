//! Successful record-local facts for one complete validation invocation.
use super::container::Entry;
use super::{BTreeMap, MetadataStore, Result};
use std::collections::VecDeque;
use std::sync::Arc;

type References = Arc<[(Option<usize>, u16)]>;
const MAX_FACT_BYTES: u64 = 32 * 1024 * 1024;
// Covers the key, Arc, FIFO slot and conservative map/node allocation overhead.
const ENTRY_OVERHEAD: u64 = 256;

pub(super) struct ValidationFacts {
    limit: u64,
    bytes: u64,
    records: BTreeMap<usize, References>,
    order: VecDeque<usize>,
    #[cfg(test)]
    pub(super) hits: usize,
    #[cfg(test)]
    pub(super) evictions: usize,
}

impl ValidationFacts {
    pub(super) fn new(limit: u64) -> Self {
        Self {
            limit: limit.min(MAX_FACT_BYTES),
            bytes: 0,
            records: BTreeMap::new(),
            order: VecDeque::new(),
            #[cfg(test)]
            hits: 0,
            #[cfg(test)]
            evictions: 0,
        }
    }

    pub(super) fn references(
        &mut self,
        store: &MetadataStore,
        id: usize,
        entry: &Entry,
    ) -> Result<References> {
        if let Some(refs) = self.records.get(&id) {
            #[cfg(test)]
            {
                self.hits += 1;
            }
            return Ok(Arc::clone(refs));
        }
        let bytes = store.read_payload(entry)?;
        // This performs canonical decoding and all field validation. Failed facts
        // are never retained. Reference kinds and graph verdicts remain caller-owned.
        let refs: References = super::decode::references(store, entry.kind, &bytes)?
            .into_iter()
            .map(|(id, kind)| (store.directory.index(&id), kind))
            .collect();
        // Positions belong only to this captured directory. Missing references and
        // wrong kinds remain caller-owned errors, in the original traversal order.
        let retained_charge = charge(&refs);
        if retained_charge <= self.limit {
            while self.bytes > self.limit - retained_charge {
                let Some(oldest) = self.order.pop_front() else {
                    break;
                };
                if let Some(old) = self.records.remove(&oldest) {
                    self.bytes -= charge(&old);
                    #[cfg(test)]
                    {
                        self.evictions += 1;
                    }
                }
            }
            self.bytes += retained_charge;
            self.order.push_back(id);
            self.records.insert(id, Arc::clone(&refs));
        }
        Ok(refs)
    }
}

fn charge(refs: &[(Option<usize>, u16)]) -> u64 {
    ENTRY_OVERHEAD.saturating_add(
        (refs.len() as u64).saturating_mul(std::mem::size_of::<(Option<usize>, u16)>() as u64),
    )
}
