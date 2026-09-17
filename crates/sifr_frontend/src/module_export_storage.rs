use sifr_lowering::ModuleMap;

pub(crate) fn replace_module_entry<T: Clone>(
    modules: &mut ModuleMap<String, T>,
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
