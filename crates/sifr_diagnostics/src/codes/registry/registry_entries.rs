//! Family-grouped diagnostic registry entries.

use std::sync::LazyLock;

use super::DiagnosticRegistryEntry;

mod calls_flow_and_protocols;
mod formatting_and_lint;
mod package;
mod parsing_names_and_types;
mod project_and_backend;
mod python_interop;
mod reserved;
mod rust_interop;

const REGISTRY_GROUPS: &[&[DiagnosticRegistryEntry]] = &[
    reserved::ENTRIES,
    parsing_names_and_types::ENTRIES,
    calls_flow_and_protocols::ENTRIES,
    project_and_backend::ENTRIES,
    formatting_and_lint::ENTRIES,
    package::ENTRIES,
    python_interop::ENTRIES,
    rust_interop::ENTRIES,
];

pub static DIAGNOSTIC_REGISTRY: LazyLock<Vec<DiagnosticRegistryEntry>> = LazyLock::new(|| {
    let entry_count = REGISTRY_GROUPS.iter().map(|entries| entries.len()).sum();
    let mut registry = Vec::with_capacity(entry_count);
    for entries in REGISTRY_GROUPS {
        registry.extend_from_slice(entries);
    }
    registry
});
