use super::{ExternalDefs, LowerCtx};

impl LowerCtx {
    pub(in crate::lower) fn with_current_module(mut self, module_name: &str) -> Self {
        self.current_module_name = Some(module_name.to_string());
        self
    }

    pub(in crate::lower) fn effective_import_module_name(
        &self,
        raw_module_name: &str,
        level: u32,
        externals: &ExternalDefs,
    ) -> String {
        if level != 1 {
            return raw_module_name.to_string();
        }
        let Some(current_module) = self.current_module_name.as_deref() else {
            return raw_module_name.to_string();
        };
        for absolute_module in relative_import_module_names(current_module, raw_module_name) {
            if external_module_exists(externals, &absolute_module) {
                return absolute_module;
            }
        }
        raw_module_name.to_string()
    }
}

pub(in crate::lower) fn external_module_exists(
    externals: &ExternalDefs,
    module_name: &str,
) -> bool {
    externals.functions.contains_key(module_name)
        || externals.classes.contains_key(module_name)
        || externals.constants.contains_key(module_name)
}

fn relative_import_module_names(current_module: &str, raw_module_name: &str) -> Vec<String> {
    if current_module == "main" {
        return vec![raw_module_name.to_string()];
    }
    let mut candidates = Vec::new();
    if !current_module.is_empty() {
        candidates.push(format!("{current_module}.{raw_module_name}"));
    }
    if let Some((parent, _)) = current_module.rsplit_once('.') {
        if !parent.is_empty() {
            candidates.push(format!("{parent}.{raw_module_name}"));
        }
    }
    candidates.push(raw_module_name.to_string());
    candidates.dedup();
    candidates
}
