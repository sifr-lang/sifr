use super::SHARED_STDLIB_NOMINAL_MODULE;
use crate::builtin_errors::BuiltinError;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct ProjectNominalRegistry {
    pub(crate) shared_rust_names: HashSet<String>,
    pub(crate) crate_root_rust_names: HashSet<String>,
    pub(crate) rust_paths: HashMap<String, String>,
}

impl ProjectNominalRegistry {
    pub(super) fn register_shared(&mut self, identity: String, rust_name: String) {
        self.rust_paths.insert(
            identity,
            format!("crate::{SHARED_STDLIB_NOMINAL_MODULE}::{rust_name}"),
        );
        self.shared_rust_names.insert(rust_name);
    }

    pub(super) fn register_crate_root(&mut self, identity: String, rust_name: String) {
        self.rust_paths
            .insert(identity, format!("crate::{rust_name}"));
        self.shared_rust_names.remove(&rust_name);
        self.crate_root_rust_names.insert(rust_name);
    }

    pub(super) fn register_builtin(&mut self, builtin: BuiltinError, rust_name: String) {
        self.register_shared(builtin.identity(), rust_name);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/project_stdlib_nominals/registry/tests.rs"]
mod tests;
