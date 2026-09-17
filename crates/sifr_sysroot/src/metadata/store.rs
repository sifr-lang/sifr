use super::container::{ENTRY_SIZE, Entry, HEADER_SIZE, MAGIC, VERSION};
use super::{
    BTreeMap, Compatibility, Digest, KIND_COUNT, Limits, MetadataError, Record, RecordId, Ref,
    Result, Sha256, err,
};
use std::any::Any;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

pub trait MetadataRead: Read + Seek + Send {}
impl<T: Read + Seek + Send> MetadataRead for T {}
struct Retained {
    bytes: u64,
    records: BTreeMap<RecordId, Arc<dyn Any + Send + Sync>>,
}
/// One immutable input owner and its shared decoded records. No global cache or
/// source-bootstrap fallback. Separate stores never share decoded nominal owners.
pub struct MetadataStore {
    input: Mutex<Box<dyn MetadataRead>>,
    pub(super) directory: BTreeMap<RecordId, Entry>,
    retained: Mutex<Retained>,
    pub(super) limits: Limits,
    compatibility: Compatibility,
}
impl MetadataStore {
    pub fn open(
        mut input: impl MetadataRead + 'static,
        expected: Compatibility,
        limits: Limits,
    ) -> Result<Self> {
        let size = input.seek(SeekFrom::End(0)).map_err(|e| io_error(&e))?;
        if size < HEADER_SIZE as u64 || size > limits.file_bytes {
            return Err(err("file size outside bounded container limits"));
        }
        input.seek(SeekFrom::Start(0)).map_err(|e| io_error(&e))?;
        let mut header = [0; HEADER_SIZE];
        input.read_exact(&mut header).map_err(|e| io_error(&e))?;
        if &header[..8] != MAGIC || u32::from_le_bytes(array(&header, 8)?) != VERSION {
            return Err(err("unknown magic or incompatible schema version"));
        }
        let actual = Compatibility {
            compiler: array(&header, 16)?,
            semantic_target: array(&header, 48)?,
            stdlib_inputs: array(&header, 80)?,
        };
        if actual != expected {
            return Err(err(
                "compiler, semantic target or stdlib input identity mismatch",
            ));
        }
        if u64::from_le_bytes(array(&header, 112)?) != size {
            return Err(err("declared file length differs from input"));
        }
        let count = u32::from_le_bytes(array(&header, 12)?);
        if count > limits.records {
            return Err(err("directory record count exceeds limit"));
        }
        let start = (HEADER_SIZE as u64)
            .checked_add(u64::from(count) * ENTRY_SIZE as u64)
            .ok_or_else(|| err("directory length overflow"))?;
        if start > size {
            return Err(err("truncated directory"));
        }
        let mut directory = BTreeMap::new();
        let mut next = start;
        let mut previous = None;
        for _ in 0..count {
            let mut raw = [0; ENTRY_SIZE];
            input.read_exact(&mut raw).map_err(|e| io_error(&e))?;
            let id: RecordId = array(&raw, 0)?;
            let kind = u16::from_le_bytes(array(&raw, 32)?);
            if kind == 0 || kind > KIND_COUNT || raw[34..36] != [0, 0] {
                return Err(err("unknown required record kind or directory flags"));
            }
            if previous.is_some_and(|p| p >= id) {
                return Err(err("noncanonical or duplicate record ID"));
            }
            previous = Some(id);
            let entry = Entry {
                kind,
                offset: u64::from_le_bytes(array(&raw, 36)?),
                len: u64::from_le_bytes(array(&raw, 44)?),
                decoded_bound: u64::from_le_bytes(array(&raw, 52)?),
                digest: array(&raw, 60)?,
            };
            if entry.offset != next || entry.len == 0 || entry.len > limits.record_bytes {
                return Err(err("invalid payload offset or bounded length"));
            }
            if entry.decoded_bound
                != entry
                    .len
                    .checked_mul(32)
                    .and_then(|n| n.checked_add(super::container::RECORD_FIXED_BOUND))
                    .ok_or_else(|| err("decoded bound overflow"))?
                || entry.decoded_bound > limits.retained_bytes
            {
                return Err(err("invalid decoded allocation bound"));
            }
            next = next
                .checked_add(entry.len)
                .ok_or_else(|| err("payload offset overflow"))?;
            if next > size {
                return Err(err("payload outside file bounds"));
            }
            directory.insert(id, entry);
        }
        if next != size {
            return Err(err("unindexed trailing bytes"));
        }
        Ok(Self {
            input: Mutex::new(Box::new(input)),
            directory,
            retained: Mutex::new(Retained {
                bytes: 0,
                records: BTreeMap::new(),
            }),
            limits,
            compatibility: actual,
        })
    }
    /// Enumerate the small typed directory without reading record payloads.
    pub fn record_refs<T: Record>(&self) -> impl Iterator<Item = Ref<T>> + '_ {
        self.directory
            .iter()
            .filter(|(_, entry)| entry.kind == T::KIND)
            .map(|(id, _)| Ref {
                id: *id,
                marker: std::marker::PhantomData,
            })
    }
    #[must_use]
    pub fn compatibility(&self) -> Compatibility {
        self.compatibility
    }
    pub fn retained_records(&self) -> Result<usize> {
        Ok(self
            .retained
            .lock()
            .map_err(|_| err("retention lock poisoned"))?
            .records
            .len())
    }
    pub fn retained_bound(&self) -> Result<u64> {
        Ok(self
            .retained
            .lock()
            .map_err(|_| err("retention lock poisoned"))?
            .bytes)
    }
    pub fn get<T: Record>(&self, reference: Ref<T>) -> Result<Arc<T>> {
        let entry = self
            .directory
            .get(&reference.id)
            .ok_or_else(|| err("record ID is absent"))?;
        if entry.kind != T::KIND {
            return Err(err("record reference has incompatible kind"));
        }
        let mut retained = self
            .retained
            .lock()
            .map_err(|_| err("retention lock poisoned"))?;
        if let Some(value) = retained.records.get(&reference.id) {
            return Arc::clone(value)
                .downcast::<T>()
                .map_err(|_| err("decoded record kind mismatch"));
        }
        if retained
            .bytes
            .checked_add(entry.decoded_bound)
            .is_none_or(|n| n > self.limits.retained_bytes)
        {
            // A live caller's Arc pins its record. Only cache-exclusive owners
            // may be evicted, under the same lock used to issue new handles.
            retained
                .records
                .retain(|_, value| Arc::strong_count(value) > 1);
            let mut retained_bytes = 0_u64;
            for id in retained.records.keys() {
                let bound = self
                    .directory
                    .get(id)
                    .ok_or_else(|| err("retained record has no directory entry"))?
                    .decoded_bound;
                retained_bytes = retained_bytes
                    .checked_add(bound)
                    .ok_or_else(|| err("retained byte count overflow"))?;
            }
            retained.bytes = retained_bytes;
            if retained
                .bytes
                .checked_add(entry.decoded_bound)
                .is_none_or(|n| n > self.limits.retained_bytes)
            {
                return Err(err(
                    "active decoded handles exhaust retention budget; release completed handles or raise the configured limit",
                ));
            }
        }
        let payload = self.read_payload(entry)?;
        let value: T = serde_json::from_slice(&payload)
            .map_err(|e| err(format!("record {}: {e}", T::KIND)))?;
        if serde_json::to_vec(&value).map_err(|e| err(e.to_string()))? != payload {
            return Err(err("noncanonical record encoding"));
        }
        self.validate_fields(&value)?;
        self.validate_graph(reference.id, &payload)?;
        let mut references = Vec::new();
        value.references(&mut references);
        for (id, kind) in references {
            if self.directory.get(&id).is_none_or(|e| e.kind != kind) {
                return Err(err("payload contains absent or wrong-kind reference"));
            }
        }
        let value = Arc::new(value);
        retained.records.insert(reference.id, value.clone());
        retained.bytes += entry.decoded_bound;
        Ok(value)
    }
    pub(super) fn read_payload(&self, entry: &Entry) -> Result<Vec<u8>> {
        let len =
            usize::try_from(entry.len).map_err(|_| err("record size exceeds address space"))?;
        let mut bytes = vec![0; len];
        let mut input = self.input.lock().map_err(|_| err("input lock poisoned"))?;
        input
            .seek(SeekFrom::Start(entry.offset))
            .map_err(|e| io_error(&e))?;
        input.read_exact(&mut bytes).map_err(|e| io_error(&e))?;
        let digest: RecordId = Sha256::digest(&bytes).into();
        if digest != entry.digest {
            return Err(err("payload digest mismatch"));
        }
        Ok(bytes)
    }
}
fn array<const N: usize>(bytes: &[u8], start: usize) -> Result<[u8; N]> {
    bytes
        .get(start..start + N)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| err("truncated fixed-width field"))
}
fn io_error(e: &std::io::Error) -> MetadataError {
    err(format!("indexed input read: {e}"))
}
