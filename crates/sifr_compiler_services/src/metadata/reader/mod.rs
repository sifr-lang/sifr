//! Bounded projections from one immutable indexed metadata owner.
pub use decode::Decode;
use sifr_sysroot::metadata as wire;
use wire::Result;
mod decode;
mod navigation;
pub mod nominal;
mod provider;
mod qualification;
pub mod selection;
mod semantic;
mod support;
mod support_projection;
mod templates;
mod types;
pub use navigation::{StdlibNavigation, StdlibNavigationSymbol};
pub use provider::Provider;
pub use selection::select;
mod hir_nodes;
mod python_interop;
mod rust_interop;
mod specialization_metadata;
mod sql_migrations;
mod sql_queries;
mod template_strings;
mod type_records;
pub struct Decoder<'a> {
    store: &'a wire::MetadataStore,
    active: std::collections::BTreeSet<wire::RecordId>,
    work: usize,
    nominal_cache: Option<&'a nominal::Cache>,
    local_nominals: std::collections::BTreeMap<wire::RecordId, std::sync::Arc<nominal::Projection>>,
}
impl<'a> Decoder<'a> {
    pub fn new(store: &'a wire::MetadataStore) -> Self {
        Self {
            store,
            active: Default::default(),
            work: 0,
            nominal_cache: None,
            local_nominals: Default::default(),
        }
    }
    pub fn with_nominals(store: &'a wire::MetadataStore, cache: &'a nominal::Cache) -> Self {
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

pub use qualification::qualify_development_metadata;

/// Restore only explicitly complete typed families; caller owns input and
/// completeness validation. No syntax, editor index or executable is implied.
pub fn decode_project_results(
    artifact: &super::ProjectTypedArtifact,
    compatibility: wire::Compatibility,
) -> Result<std::collections::BTreeMap<String, (sifr_ir::HirModule, sifr_lowering::ExternalDefs)>> {
    let store = wire::MetadataStore::open_bytes(
        artifact.bytes.clone(),
        compatibility,
        wire::Limits::default(),
    )?;
    store.validate_complete()?;
    for (reference, captured) in &artifact.sources {
        use sha2::Digest;
        let source = store.get(*reference)?;
        let digest: [u8; 32] = sha2::Sha256::digest(captured.text.as_bytes()).into();
        if source.relative_path != format!("project/{}.sifr", captured.identity())
            || source.content_digest != digest
            || usize::try_from(source.byte_length).ok() != Some(captured.text.len())
        {
            return Err(wire::MetadataError("project source table mismatch".into()));
        }
    }
    let mut cx = Decoder::new(&store);
    artifact
        .modules
        .iter()
        .map(|(name, references)| {
            let module = store.get(references.module)?;
            let stored_name: String = Decode::decode(&module.name, &mut cx)?;
            if !artifact.sources.contains_key(&module.source)
                || stored_name != *name
                || module.hir_inventory != references.checked.hir
                || module.semantic != references.interface.exports.semantic
                || module.hir_inventory != references.interface.exports.bodies
                || references.checked.module_identity != references.interface.module_identity
                || references.checked.input_identity != references.interface.input_identity
                || references.checked.input_identity.is_empty()
            {
                return Err(wire::MetadataError(
                    "inconsistent project family references".into(),
                ));
            }
            Ok((
                name.clone(),
                (
                    Decode::decode(&references.checked.hir, &mut cx)?,
                    semantic::project(name, references.interface.exports.semantic, &mut cx)?,
                ),
            ))
        })
        .collect()
}
