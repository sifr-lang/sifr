use std::collections::HashMap;

pub(crate) fn replace_module_entry<T>(
    modules: &mut HashMap<String, T>,
    module_name: &str,
    value: T,
    is_empty: impl FnOnce(&T) -> bool,
) {
    if is_empty(&value) {
        modules.remove(module_name);
    } else {
        modules.insert(module_name.to_string(), value);
    }
}
