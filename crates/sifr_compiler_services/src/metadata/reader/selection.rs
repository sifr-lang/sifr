use super::{Provider, Result};
use sifr_identity::CompilerIdentity;
pub use sifr_semantic_services::metadata::installed::target;
use sifr_sysroot::{ResolvedSysroot, SysrootMode};
use std::{path::Path, sync::Arc};
pub fn select(
    identity: &CompilerIdentity,
    root: &ResolvedSysroot,
    override_path: Option<&Path>,
    cache_root: &Path,
) -> Result<Arc<Provider>> {
    let prepared = if root.mode() == SysrootMode::SourceTreeDevelopment {
        let prepared = match override_path {
            Some(path) => crate::metadata::validate_development_metadata(
                identity,
                &root.root,
                target(),
                path,
            )?,
            None => crate::metadata::ensure_development_metadata(
                identity,
                &root.root,
                target(),
                cache_root,
                &|| false,
            )?,
        };
        crate::metadata::open_consumer(
            &prepared.path,
            prepared.compatibility,
            Some(&prepared.metadata_id),
        )?
    } else {
        return sifr_semantic_services::metadata::installed::select(identity, root, override_path)
            .map(Provider::from_semantic);
    };
    Provider::new(prepared)
}
