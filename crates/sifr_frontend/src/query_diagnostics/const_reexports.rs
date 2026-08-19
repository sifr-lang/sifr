use super::{ExternalDefs, HashMap};

pub(super) fn record_local_const_function(
    function: &sifr_lowering::HirFunction,
    functions: &mut HashMap<String, sifr_lowering::HirFunction>,
) {
    if function
        .decorators
        .iter()
        .any(|decorator| decorator == "const_eval")
    {
        functions.insert(function.name.clone(), function.clone());
    }
}

pub(super) fn copy_const_function_and_defaults(
    external_defs: &ExternalDefs,
    module: &str,
    name: &str,
    local_name: &str,
    functions: &mut HashMap<String, sifr_lowering::HirFunction>,
    defaults: &mut HashMap<String, Vec<(usize, sifr_lowering::HirExpr)>>,
) {
    if let Some(function) = external_defs
        .const_functions
        .get(module)
        .and_then(|items| items.get(name))
    {
        let mut function = function.clone();
        local_name.clone_into(&mut function.name);
        functions.insert(local_name.to_string(), function);
    }
    if let Some(values) = external_defs
        .function_defaults
        .get(module)
        .and_then(|items| items.get(name))
    {
        defaults.insert(local_name.to_string(), values.clone());
    }
}
