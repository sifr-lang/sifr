use super::Session;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExternalInputIdentity {
    pub(crate) package_root: Option<PathBuf>,
    pub(crate) generation: u64,
    pub(crate) fingerprint: u64,
    pub(crate) stable: bool,
}

impl Session {
    #[cfg(test)]
    pub(crate) fn external_input_generation(&self, root: &Path) -> Option<u64> {
        self.external_inputs.generation(root)
    }

    fn external_input_roots_for_path(&self, path: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
        let mut provider = sifr_analysis::DiskSourceProvider::new();
        let current = crate::python_declarations::package_root_for(path, &mut provider);
        let tracked = self
            .external_inputs
            .owning_root(path)
            .map(Path::to_path_buf);
        (current, tracked)
    }

    pub(crate) fn external_input_identity_for_path(&self, path: &Path) -> ExternalInputIdentity {
        let (package_root, tracked) = self.external_input_roots_for_path(path);
        let (generation, fingerprint, stable) = package_root
            .as_deref()
            .or(tracked.as_deref())
            .and_then(|root| self.external_inputs.state(root))
            .unwrap_or((0, 0, true));
        ExternalInputIdentity {
            package_root,
            generation,
            fingerprint,
            stable,
        }
    }

    /// Revalidate a root on demand. E01b uses this while watcher authority is absent.
    pub(crate) fn observe_external_inputs_for_path(&mut self, path: &Path) -> u64 {
        self.observe_external_input_identity_for_path(path)
            .generation
    }

    pub(crate) fn observe_external_input_identity_for_path(
        &mut self,
        path: &Path,
    ) -> ExternalInputIdentity {
        let (package_root, tracked) = self.external_input_roots_for_path(path);
        let owner_changed = if let (Some(current), Some(previous)) = (&package_root, &tracked) {
            if previous.components().count() > current.components().count() {
                // The nested manifest disappeared. Remove its old authority so
                // later captures and watcher events use the ancestor owner.
                self.external_inputs.retire_root(previous);
                self.python_declarations.invalidate_external_root(previous);
                self.analysis.refresh_external_root(previous, &self.store);
                true
            } else {
                false
            }
        } else {
            false
        };
        let root = package_root.as_deref().or(tracked.as_deref());
        if let Some(root) = root {
            self.observe_external_root(root);
        }
        let (generation, fingerprint, stable) = root
            .and_then(|root| self.external_inputs.state(root))
            .unwrap_or((0, 0, true));
        if owner_changed {
            if let Some(current) = package_root.as_deref() {
                // Rebuild the ancestor project with the reassigned open source.
                // A stable external fingerprint alone cannot update its file map.
                self.python_declarations.invalidate_external_root(current);
                self.analysis.refresh_external_root(current, &self.store);
            }
        }
        ExternalInputIdentity {
            package_root,
            generation,
            fingerprint,
            stable,
        }
    }

    pub(super) fn observe_external_root(&mut self, root: &Path) -> u64 {
        let new_root = !self.external_inputs.contains_root(root);
        let (generation, changed) = self.external_inputs.observe(root);
        let new_owner = new_root
            && self.store.documents().any(|document| {
                document.path().starts_with(root) && self.analysis.owns_document(document.uri())
            });
        if changed || new_owner {
            self.python_declarations.invalidate_external_root(root);
            self.analysis.refresh_external_root(root, &self.store);
        }
        generation
    }
}
