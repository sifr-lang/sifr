use sifr_ir::HirModule;
use sifr_type_system::source_class_rust_name;
use std::collections::HashSet;

pub(crate) fn project_module_binding_names(module: &HirModule) -> HashSet<String> {
    let mut names = module
        .classes
        .iter()
        .map(|class| source_class_rust_name(&class.name))
        .collect::<HashSet<_>>();
    // An imported nominal shadows a builtin just as a local declaration does.
    // Its basename is not a demand for the builtin's crate-root re-export.
    for import in &module.imports {
        if import.module.starts_with("sifr.") || import.module.starts_with("_sifr.") {
            continue;
        }
        for name in &import.names {
            let binding = import
                .aliases
                .iter()
                .find(|(original, _)| original == name)
                .map_or(name, |(_, alias)| alias);
            names.insert(source_class_rust_name(binding));
        }
    }
    names
}
