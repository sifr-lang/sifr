//! One complete nominal projection per artifact-local view. Type occurrences
//! share these payloads; contextual substitutions detach through copy-on-write.
use super::{Decode, Decoder, Result, wire};
use sifr_type_system::{FunctionType, SharedVec, Type};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

pub type Cache = Mutex<BTreeMap<wire::RecordId, Arc<Projection>>>;
pub struct Projection {
    pub name: String,
    pub identity: Option<String>,
    pub fields: SharedVec<(String, Type)>,
    pub methods: SharedVec<(String, FunctionType)>,
    pub parent_class: Option<String>,
    pub enum_variants: SharedVec<(String, Option<i64>)>,
    pub newtype_inner: Option<Type>,
}
impl Decoder<'_> {
    pub(super) fn nominal(
        &mut self,
        reference: wire::Ref<wire::NominalView>,
    ) -> Result<Arc<Projection>> {
        let existing = match self.nominal_cache {
            Some(cache) => cache
                .lock()
                .map_err(|_| wire::MetadataError("nominal projection owner poisoned".into()))?
                .get(&reference.id())
                .cloned(),
            None => self.local_nominals.get(&reference.id()).cloned(),
        };
        if let Some(projected) = existing {
            return Ok(projected);
        }
        let projected = Arc::new(self.record(reference, |value, cx| {
            Ok(Projection {
                name: Decode::decode(&value.name, cx)?,
                identity: Decode::decode(&value.identity, cx)?,
                fields: Decode::decode(&value.fields, cx)?,
                methods: Decode::decode(&value.methods, cx)?,
                parent_class: Decode::decode(&value.parent_class, cx)?,
                enum_variants: Decode::decode(&value.enum_variants, cx)?,
                newtype_inner: Decode::decode(&value.newtype_inner, cx)?,
            })
        })?);
        // Never hold the cache lock while following nested type records.
        let retained = match self.nominal_cache {
            Some(cache) => cache
                .lock()
                .map_err(|_| wire::MetadataError("nominal projection owner poisoned".into()))?
                .entry(reference.id())
                .or_insert(projected)
                .clone(),
            None => self
                .local_nominals
                .entry(reference.id())
                .or_insert(projected)
                .clone(),
        };
        Ok(retained)
    }
}
