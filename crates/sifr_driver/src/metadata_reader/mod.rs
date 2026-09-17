//! Bounded projections from one immutable indexed metadata owner.
use decode::Decode;
use sifr_sysroot::metadata as wire;
use wire::Result;
mod decode;
mod navigation;
mod nominal;
mod provider;
mod qualification;
mod selection;
mod semantic;
mod support;
mod support_projection;
mod templates;
mod types;
pub use navigation::{StdlibNavigation, StdlibNavigationSymbol};
pub(crate) use provider::Provider;
pub(crate) use selection::select;
mod hir_nodes;
mod python_interop;
mod rust_interop;
mod specialization_metadata;
mod sql_migrations;
mod sql_queries;
mod template_strings;
mod type_records;
struct Decoder<'a> {
    store: &'a wire::MetadataStore,
    active: std::collections::BTreeSet<wire::RecordId>,
    work: usize,
    nominal_cache: Option<&'a nominal::Cache>,
    local_nominals: std::collections::BTreeMap<wire::RecordId, std::sync::Arc<nominal::Projection>>,
}
impl<'a> Decoder<'a> {
    fn new(store: &'a wire::MetadataStore) -> Self {
        Self {
            store,
            active: Default::default(),
            work: 0,
            nominal_cache: None,
            local_nominals: Default::default(),
        }
    }
    fn with_nominals(store: &'a wire::MetadataStore, cache: &'a nominal::Cache) -> Self {
        Self {
            nominal_cache: Some(cache),
            ..Self::new(store)
        }
    }
    fn record<R: wire::Record, T>(
        &mut self,
        reference: wire::Ref<R>,
        f: impl FnOnce(&R, &mut Self) -> Result<T>,
    ) -> Result<T> {
        self.work += 1;
        if self.work > 1_000_000 || self.active.len() >= 128 || !self.active.insert(reference.id())
        {
            return Err(wire::MetadataError("demanded metadata projection exceeds depth/work or repeats an active structural record".into()));
        }
        let result = self.store.get(reference).and_then(|value| f(&value, self));
        self.active.remove(&reference.id());
        result
    }
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod corpus_tests;

#[cfg(all(test, unix))]
mod generation_tests;

pub use qualification::qualify_development_metadata;
