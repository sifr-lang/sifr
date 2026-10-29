//! Bind language-owned nominal identities to generated-Rust transformations.
pub(crate) use sifr_generated_rust::stdlib_filter::*;
use sifr_type_system::{is_global_rust_nominal_identity, stdlib_class_rust_name};
use std::collections::{HashMap, HashSet};

fn nominal_replacements(module: &str, nominal_types: &HashSet<String>) -> HashMap<String, String> {
    nominal_types
        .iter()
        .filter(|name| !is_global_rust_nominal_identity(&format!("{module}.{name}")))
        .map(|name| (name.clone(), stdlib_class_rust_name(module, name)))
        .collect()
}

pub(crate) fn seal_canonical_stdlib_names(
    rust_code: &str,
    module: &str,
    nominal_types: &HashSet<String>,
) -> String {
    sifr_generated_rust::stdlib_filter::seal_canonical_stdlib_names(
        rust_code,
        &nominal_replacements(module, nominal_types),
    )
}

pub(crate) fn filter_canonical_stdlib_ir_to_needed(
    rust_code: &str,
    imported_names: &HashSet<String>,
    module: &str,
    nominal_types: &HashSet<String>,
) -> String {
    let replacements = nominal_replacements(module, nominal_types)
        .into_iter()
        .map(|(source, canonical)| (canonical, source))
        .collect();
    sifr_generated_rust::stdlib_filter::filter_canonical_stdlib_ir_to_needed(
        rust_code,
        imported_names,
        &replacements,
    )
}

#[cfg(test)]
#[path = "../tests/unit/stdlib_filter/tests.rs"]
mod tests;
