use super::Session;
use std::path::Path;

impl Session {
    /// Revalidate a root on demand. E01b uses this while watcher authority is absent.
    pub(crate) fn observe_external_inputs_for_path(&mut self, path: &Path) -> u64 {
        let mut provider = sifr_analysis::DiskSourceProvider::new();
        let current = crate::python_declarations::package_root_for(path, &mut provider);
        let tracked = self
            .external_inputs
            .owning_root(path)
            .map(Path::to_path_buf);
        if let (Some(current), Some(tracked)) = (&current, &tracked) {
            if tracked.components().count() > current.components().count() {
                // A nested manifest may have disappeared. Retire its old owner,
                // then establish the ancestor that now owns the open document.
                self.observe_external_root(tracked);
            }
        }
        current
            .or(tracked)
            .map_or(0, |root| self.observe_external_root(&root))
    }

    fn observe_external_root(&mut self, root: &Path) -> u64 {
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
