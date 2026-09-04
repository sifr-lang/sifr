use super::{HashMap, HashSet, HirModule};
use sifr_type_system::source_class_rust_name;

pub(crate) fn project_nominal_type_paths(
    modules: &[(&str, &HirModule)],
    crate_root_modules: &HashSet<&str>,
) -> HashMap<String, String> {
    let mut paths = HashMap::new();
    let mut basename_counts = HashMap::new();
    for (_, module) in modules {
        for class in &module.classes {
            *basename_counts
                .entry(class.name.as_str())
                .or_insert(0_usize) += 1;
        }
    }
    for (module_name, module) in modules {
        for class in &module.classes {
            let rust_name = source_class_rust_name(&class.name);
            let path = if crate_root_modules.contains(module_name) {
                format!("crate::{rust_name}")
            } else {
                format!("crate::{}::{rust_name}", module_name.replace('.', "::"))
            };
            let canonical = class
                .identity
                .clone()
                .unwrap_or_else(|| format!("{module_name}.{}", class.name));
            paths.insert(canonical, path.clone());
            paths.insert(format!("{module_name}.{}", class.name), path.clone());
            if basename_counts.get(class.name.as_str()) == Some(&1) {
                paths.insert(class.name.clone(), path);
            }
        }
    }
    paths
}
