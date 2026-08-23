use super::{DIAGNOSTIC_REGISTRY, DiagnosticRegistryEntry, DiagnosticState, DiagnosticTooling};

#[must_use]
pub fn registry_entry(id: &str) -> Option<&'static DiagnosticRegistryEntry> {
    DIAGNOSTIC_REGISTRY.iter().find(|entry| entry.id == id)
}

pub fn active_registry_entries() -> impl Iterator<Item = &'static DiagnosticRegistryEntry> {
    DIAGNOSTIC_REGISTRY
        .iter()
        .filter(|entry| entry.state == DiagnosticState::Active)
}

pub(super) const fn reserved_family_base(
    id: &'static str,
    family: &'static str,
) -> DiagnosticRegistryEntry {
    reserved_code(
        id,
        family,
        "Reserved family base; not emitted as a diagnostic.",
    )
}

pub(super) const fn reserved_code(
    id: &'static str,
    family: &'static str,
    summary: &'static str,
) -> DiagnosticRegistryEntry {
    DiagnosticRegistryEntry {
        id,
        family,
        summary,
        state: DiagnosticState::Reserved,
        docs_path: "docs/errors/diagnostic-codes.md",
        representative_fixture_path: None,
        message_template: None,
        owner_module: None,
        declared_args: &[],
        dedupe_args: &[],
        declared_severity: None,
        tooling: DiagnosticTooling::DEFAULT,
    }
}
