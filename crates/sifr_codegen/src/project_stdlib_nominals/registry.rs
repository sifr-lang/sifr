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
    use crate::builtin_errors::builtin_error_identity;
    use crate::project_stdlib_nominals::{compiler_builtin_error, project_stdlib_nominal_plan};
    use sifr_type_system::Type;

    #[test]
    fn corpus_repair_builtin_registration_preserves_identity() {
        let mut registry = ProjectNominalRegistry::default();
        registry.register_crate_root("shadow.ValueError".to_string(), "ShadowError".to_string());
        for builtin in BuiltinError::all() {
            let name = builtin.name();
            assert_eq!(BuiltinError::from_name(name), Some(builtin));
            assert_eq!(builtin.identity(), format!("sifr.builtin.{name}"));
            assert_eq!(builtin_error_identity(name), Some(builtin.identity()));
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

        let paths = registry.rust_paths.clone();
        let shared_names = registry.shared_rust_names.clone();
        let root_names = registry.crate_root_rust_names.clone();
        for builtin in BuiltinError::all() {
            registry.register_builtin(builtin, builtin.name().to_string());
        }
        assert_eq!(registry.rust_paths, paths);
        assert_eq!(registry.shared_rust_names, shared_names);
        assert_eq!(registry.crate_root_rust_names, root_names);

        for name in ["WorkerError", "WorkerRuntimeError"] {
            let identity = format!("sifr.parallel.{name}");
            let builtin = compiler_builtin_error(Some(&identity), name)
                .expect("supported global worker error identity");
            assert_eq!(Some(builtin), BuiltinError::from_name(name));
            assert_eq!(builtin.identity(), format!("sifr.builtin.{name}"));
            let worker = Type::Class {
                identity: Some(identity.clone()),
                type_args: Vec::new(),
                name: name.to_string(),
                fields: vec![("message".to_string(), Type::Str)],
                methods: Vec::new(),
                parent_class: Some("Error".to_string()),
            };
            let unions = HashMap::from([("WorkerErrors".to_string(), vec![worker])]);
            let plan = project_stdlib_nominal_plan(&unions, &[]);
            let path = format!("crate::__sifr_project_nominals::{name}");
            assert_eq!(plan.registry.rust_paths.get(&identity), Some(&path));
            assert_eq!(
                plan.registry.rust_paths.get(&builtin.identity()),
                Some(&path)
            );
            assert!(plan.registry.shared_rust_names.contains(name));
            assert!(!plan.registry.rust_paths.contains_key(name));
        }
    }

    #[test]
    fn corpus_repair_builtin_registration_rejects_nonbuiltin_names() {
        for name in [
            "CustomError",
            "shadow.ValueError",
            "sifr.builtin.ValueError",
            "ValueErrorExtra",
            "shadow.WorkerError",
            "shadow.WorkerRuntimeError",
            "sifr.parallel.WorkerError",
            "sifr.parallel.WorkerRuntimeError",
            "",
        ] {
            assert!(BuiltinError::from_name(name).is_none(), "{name}");
            assert!(builtin_error_identity(name).is_none(), "{name}");
        }
        for name in ["ValueError", "WorkerError", "WorkerRuntimeError"] {
            let identity = format!("shadow.{name}");
            assert!(compiler_builtin_error(Some(&identity), name).is_none());
            let shadow = Type::Class {
                identity: Some(identity.clone()),
                type_args: Vec::new(),
                name: name.to_string(),
                fields: vec![("detail".to_string(), Type::Str)],
                methods: Vec::new(),
                parent_class: Some("Error".to_string()),
            };
            let unions = HashMap::from([("ShadowErrors".to_string(), vec![shadow])]);
            let plan = project_stdlib_nominal_plan(&unions, &[]);
            assert!(!plan.registry.rust_paths.contains_key(&identity));
            assert!(
                !plan
                    .registry
                    .rust_paths
                    .contains_key(&format!("sifr.builtin.{name}"))
            );
            assert!(!plan.registry.shared_rust_names.contains(name));
        }
        for identity in [None, Some("sifr.builtin.ValueError")] {
            let builtin = compiler_builtin_error(identity, "ValueError");
            assert_eq!(
                builtin.map(BuiltinError::identity),
                Some("sifr.builtin.ValueError".to_string())
            );
        }
    }
}
