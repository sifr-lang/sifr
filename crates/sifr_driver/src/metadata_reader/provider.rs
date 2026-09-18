use super::{Decode, Decoder, Result, wire};
use crate::metadata_producer::PreparedMetadata;
use sifr_lowering::{ExternalDefs, ExternalProvider};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

pub(crate) struct Provider {
    pub(crate) metadata: Arc<PreparedMetadata>,
    pub(crate) modules: BTreeMap<String, wire::Ref<wire::Module>>,
    pub(super) nominals: super::nominal::Cache,
    semantics: Mutex<BTreeMap<String, Arc<ExternalDefs>>>,
    pub(super) navigation_cache:
        Mutex<BTreeMap<std::path::PathBuf, std::sync::Weak<super::StdlibNavigation>>>,
}
impl std::fmt::Debug for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataProvider")
            .field("identity", &self.metadata.metadata_id)
            .finish_non_exhaustive()
    }
}
impl Provider {
    pub(crate) fn new(metadata: Arc<PreparedMetadata>) -> Result<Arc<Self>> {
        let mut modules = BTreeMap::new();
        let mut cx = Decoder::new(&metadata.store);
        for reference in metadata.store.record_refs::<wire::Module>() {
            let module = metadata.store.get(reference)?;
            let name = String::decode(&module.name, &mut cx)?;
            if modules.insert(name, reference).is_some() {
                return Err(wire::MetadataError("duplicate metadata module name".into()));
            }
        }
        Ok(Arc::new(Self {
            metadata,
            modules,
            semantics: Mutex::default(),
            nominals: Mutex::default(),
            navigation_cache: Mutex::default(),
        }))
    }
    pub(crate) fn closure(&self, requested: &[String]) -> Result<BTreeSet<String>> {
        let mut pending = requested.to_vec();
        let mut found = BTreeSet::new();
        while let Some(name) = pending.pop() {
            let Some(reference) = self.modules.get(&name) else {
                continue;
            };
            if !found.insert(name) {
                continue;
            }
            let module = self.metadata.store.get(*reference)?;
            for dependency in &module.dependencies {
                let dependency = self.metadata.store.get(*dependency)?;
                pending.push(self.metadata.store.get(dependency.name)?.value.clone());
            }
        }
        Ok(found)
    }
    pub(crate) fn semantic(&self, name: &str) -> Result<Arc<ExternalDefs>> {
        let mut cached = self
            .semantics
            .lock()
            .map_err(|_| wire::MetadataError("metadata semantic owner poisoned".into()))?;
        if let Some(defs) = cached.get(name) {
            return Ok(defs.clone());
        }
        let reference = self
            .modules
            .get(name)
            .ok_or_else(|| wire::MetadataError(format!("unknown metadata module {name}")))?;
        let module = self.metadata.store.get(*reference)?;
        let mut cx = Decoder::with_nominals(&self.metadata.store, &self.nominals);
        let defs = Arc::new(super::semantic::project(name, module.semantic, &mut cx)?);
        cached.insert(name.to_owned(), defs.clone());
        Ok(defs)
    }
}
impl ExternalProvider for Provider {
    fn prepare(&self, modules: &[String]) -> std::result::Result<ExternalDefs, String> {
        let mut defs = ExternalDefs::default();
        for name in self.closure(modules).map_err(|e| e.to_string())? {
            let module = self.semantic(&name).map_err(|e| e.to_string())?;
            defs.extend_baseline(&module);
        }
        Ok(defs)
    }
}
impl Provider {
    pub(crate) fn loaded_semantic_modules(&self) -> Vec<String> {
        self.semantics
            .lock()
            .map(|modules| modules.keys().cloned().collect())
            .unwrap_or_default()
    }
}
impl Provider {
    pub(crate) fn projected_nominal_views(&self) -> Result<usize> {
        self.nominals
            .lock()
            .map(|values| values.len())
            .map_err(|_| wire::MetadataError("nominal projection owner poisoned".into()))
    }
}
