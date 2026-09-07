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
mod tests {
    use super::*;

    #[test]
    fn corpus_repair_builtin_registration_preserves_identity() {
        let mut registry = ProjectNominalRegistry::default();
        registry.register_crate_root("shadow.ValueError".to_string(), "ShadowError".to_string());
        for builtin in BuiltinError::all() {
            let name = builtin.name();
            registry.register_builtin(builtin, name.to_string());
            assert_eq!(
                registry.rust_paths.get(&builtin.identity()),
                Some(&format!("crate::__sifr_project_nominals::{name}"))
            );
            assert!(!registry.rust_paths.contains_key(name));
            assert!(registry.shared_rust_names.contains(name));
        }
        assert_eq!(
            registry.rust_paths.get("shadow.ValueError"),
            Some(&"crate::ShadowError".to_string())
        );
        assert!(registry.crate_root_rust_names.contains("ShadowError"));
        assert!(!registry.shared_rust_names.contains("ShadowError"));
    }

    #[test]
    fn corpus_repair_builtin_registration_rejects_nonbuiltin_names() {
        for name in ["CustomError", "shadow.ValueError", "ValueErrorExtra", ""] {
            assert!(BuiltinError::from_name(name).is_none(), "{name}");
        }
        assert!(
            super::super::compiler_builtin_error(Some("shadow.ValueError"), "ValueError").is_none()
        );
        for identity in [None, Some("sifr.builtin.ValueError")] {
            let builtin = super::super::compiler_builtin_error(identity, "ValueError");
            assert_eq!(
                builtin.map(BuiltinError::identity),
                Some("sifr.builtin.ValueError".to_string())
            );
        }
    }
}
