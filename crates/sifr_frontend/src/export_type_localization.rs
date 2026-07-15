use sifr_lowering::{ExternalDefs, HirModule};
use std::collections::HashMap;

pub(crate) fn reexport_class_aliases(
    module: &HirModule,
    external_defs: &ExternalDefs,
) -> HashMap<String, HashMap<String, String>> {
    let mut aliases = HashMap::new();
    for import in &module.imports {
        let Some(classes) = external_defs.classes.get(&import.module) else {
            continue;
        };
        for name in &import.names {
            if !classes.contains_key(name) {
                continue;
            }
            let local = import
                .aliases
                .iter()
                .find(|(source, _)| source == name)
                .map_or_else(|| name.clone(), |(_, local)| local.clone());
            aliases
                .entry(import.module.clone())
                .or_insert_with(HashMap::new)
                .insert(name.clone(), local);
        }
    }
    aliases
}
