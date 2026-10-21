//! Native capability wrapper around the shared immutable metadata owner.
use super::{Result, wire};
use std::sync::Arc;
#[derive(Debug)]
pub struct Provider(Arc<sifr_semantic_services::metadata::reader::Provider>);
impl Provider {
    pub fn new(metadata: Arc<crate::metadata::PreparedMetadata>) -> Result<Arc<Self>> {
        sifr_semantic_services::metadata::reader::Provider::new(metadata).map(Self::from_semantic)
    }
    pub(super) fn from_semantic(
        provider: Arc<sifr_semantic_services::metadata::reader::Provider>,
    ) -> Arc<Self> {
        Arc::new(Self(provider))
    }
}
impl std::ops::Deref for Provider {
    type Target = sifr_semantic_services::metadata::reader::Provider;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl sifr_lowering::ExternalProvider for Provider {
    fn prepare(
        &self,
        modules: &[String],
    ) -> std::result::Result<sifr_lowering::ExternalDefs, String> {
        sifr_lowering::ExternalProvider::prepare(self.0.as_ref(), modules)
    }
}
