use super::{BTreeMap, Digest, Record, RecordId, Ref, Result, Sha256, err};

pub(crate) const MAGIC: &[u8; 8] = b"SIFRMETA";
pub(crate) const VERSION: u32 = 3;
pub(crate) const HEADER_SIZE: usize = 120;
pub(crate) const ENTRY_SIZE: usize = 92;
pub(crate) const RECORD_FIXED_BOUND: u64 = 4096;
type EncodedRecord = (u16, Vec<u8>, Vec<(RecordId, u16)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Compatibility {
    pub compiler: RecordId,
    pub semantic_target: RecordId,
    pub stdlib_inputs: RecordId,
}
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub file_bytes: u64,
    pub records: u32,
    pub record_bytes: u64,
    pub retained_bytes: u64,
    pub graph_depth: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            file_bytes: 256 * 1024 * 1024,
            records: 200_000,
            record_bytes: 2 * 1024 * 1024,
            retained_bytes: 64 * 1024 * 1024,
            graph_depth: 256,
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub kind: u16,
    pub offset: u64,
    pub len: u64,
    pub decoded_bound: u64,
    pub digest: RecordId,
}

/// Canonical container assembly from explicit records. This is not the stdlib
/// producer: callers must supply already checked, complete record families.
pub struct MetadataEncoder {
    compatibility: Compatibility,
    limits: Limits,
    records: BTreeMap<RecordId, EncodedRecord>,
}
impl MetadataEncoder {
    #[must_use]
    pub fn new(compatibility: Compatibility, limits: Limits) -> Self {
        Self {
            compatibility,
            limits,
            records: BTreeMap::new(),
        }
    }
    pub fn insert<T: Record>(&mut self, reference: Ref<T>, record: &T) -> Result<()> {
        let bytes = serde_json::to_vec(record).map_err(|e| err(format!("encode record: {e}")))?;
        if bytes.len() as u64 > self.limits.record_bytes {
            return Err(err("record exceeds encoded byte limit"));
        }
        let mut refs = Vec::new();
        record.references(&mut refs);
        let value = (T::KIND, bytes, refs);
        if let Some(previous) = self.records.get(&reference.id) {
            if previous != &value {
                return Err(err("stable record anchor has conflicting definitions"));
            }
            return Ok(());
        }
        if self.records.len() >= self.limits.records as usize {
            return Err(err("record count exceeds limit"));
        }
        self.records.insert(reference.id, value);
        Ok(())
    }
    pub fn intern<T: Record>(&mut self, record: &T) -> Result<Ref<T>> {
        let bytes = serde_json::to_vec(record).map_err(|e| err(format!("encode record: {e}")))?;
        let reference = Ref::<T>::anchor(&[b"content", &bytes]);
        self.insert(reference, record)?;
        Ok(reference)
    }
    pub fn finish(self) -> Result<Vec<u8>> {
        let index_bytes = self
            .records
            .len()
            .checked_mul(ENTRY_SIZE)
            .ok_or_else(|| err("directory overflow"))?;
        let start = HEADER_SIZE
            .checked_add(index_bytes)
            .ok_or_else(|| err("directory overflow"))?;
        let mut total = start as u64;
        for (_, bytes, refs) in self.records.values() {
            total = total
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| err("container length overflow"))?;
            for (id, kind) in refs {
                if self.records.get(id).is_none_or(|entry| entry.0 != *kind) {
                    return Err(err("missing or wrong-kind record reference"));
                }
            }
        }
        if total > self.limits.file_bytes {
            return Err(err("container exceeds file byte limit"));
        }
        let size = usize::try_from(total).map_err(|_| err("container exceeds address space"))?;
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(
            &u32::try_from(self.records.len())
                .map_err(|_| err("record count overflow"))?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&self.compatibility.compiler);
        bytes.extend_from_slice(&self.compatibility.semantic_target);
        bytes.extend_from_slice(&self.compatibility.stdlib_inputs);
        bytes.extend_from_slice(&total.to_le_bytes());
        let mut offset = start as u64;
        for (id, (kind, payload, _)) in &self.records {
            bytes.extend_from_slice(id);
            bytes.extend_from_slice(&kind.to_le_bytes());
            bytes.extend_from_slice(&0_u16.to_le_bytes());
            let len = payload.len() as u64;
            bytes.extend_from_slice(&offset.to_le_bytes());
            bytes.extend_from_slice(&len.to_le_bytes());
            bytes.extend_from_slice(&(RECORD_FIXED_BOUND + len.saturating_mul(32)).to_le_bytes());
            bytes.extend_from_slice(&Sha256::digest(payload));
            offset += len;
        }
        for (_, payload, _) in self.records.into_values() {
            bytes.extend_from_slice(&payload);
        }
        super::physical::encode(&bytes, self.limits)
    }
}
