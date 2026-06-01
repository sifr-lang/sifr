use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NamespaceModuleFile {
    pub(crate) path: PathBuf,
    pub(crate) declarations: Vec<String>,
}

pub(crate) fn rust_module_file_path(module_id: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for component in module_id.split('.') {
        path.push(component);
    }
    path.set_extension("rs");
    path
}

pub(crate) fn top_level_module_declarations(module_ids: &[String]) -> Vec<String> {
    module_ids
        .iter()
        .filter_map(|module_id| module_id.split('.').next())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub(crate) fn namespace_module_files(module_ids: &[String]) -> Vec<NamespaceModuleFile> {
    let mut namespace_children: std::collections::BTreeMap<String, BTreeSet<String>> =
        std::collections::BTreeMap::new();
    for module_id in module_ids {
        let parts: Vec<&str> = module_id.split('.').collect();
        if parts.len() < 2 {
            continue;
        }
        for depth in 1..parts.len() {
            let namespace = parts[..depth].join(".");
            namespace_children
                .entry(namespace)
                .or_default()
                .insert(parts[depth].to_string());
        }
    }

    namespace_children
        .into_iter()
        .map(|(namespace, children)| {
            let mut path = PathBuf::new();
            for component in namespace.split('.') {
                path.push(component);
            }
            path.push("mod.rs");
            NamespaceModuleFile {
                path,
                declarations: children.into_iter().collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dotted_module_id_maps_to_nested_rust_file() {
        assert_eq!(
            rust_module_file_path("helpers.nodes"),
            PathBuf::from("helpers").join("nodes.rs")
        );
    }

    #[test]
    fn test_top_level_declarations_deduplicate_dotted_namespaces() {
        let module_ids = vec![
            "helpers.nodes".to_string(),
            "helpers.tree_node".to_string(),
            "math".to_string(),
        ];

        assert_eq!(
            top_level_module_declarations(&module_ids),
            vec!["helpers".to_string(), "math".to_string()]
        );
    }

    #[test]
    fn test_namespace_files_declare_direct_children() {
        let module_ids = vec![
            "helpers.nodes".to_string(),
            "helpers.nested.value".to_string(),
        ];

        let namespace_files = namespace_module_files(&module_ids);

        assert_eq!(
            namespace_files,
            vec![
                NamespaceModuleFile {
                    path: PathBuf::from("helpers").join("mod.rs"),
                    declarations: vec!["nested".to_string(), "nodes".to_string()],
                },
                NamespaceModuleFile {
                    path: PathBuf::from("helpers").join("nested").join("mod.rs"),
                    declarations: vec!["value".to_string()],
                },
            ]
        );
    }
}
