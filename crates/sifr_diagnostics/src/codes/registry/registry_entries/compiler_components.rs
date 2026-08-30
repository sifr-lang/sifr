//! Sandboxed compiler-component diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-COMPONENT-0001",
        "COMPONENT",
        "Compiler component registration is invalid.",
        Severity::Error,
        "crates/sifr_compiler_component/src/tests.rs",
        "compiler component registration is invalid",
        "sifr_compiler_component",
        [],
        []
    ),
    active_entry!(
        "SIFR-COMPONENT-0002",
        "COMPONENT",
        "Compiler component integrity verification failed.",
        Severity::Error,
        "crates/sifr_compiler_component/src/tests.rs",
        "compiler component integrity verification failed",
        "sifr_compiler_component",
        [],
        []
    ),
    active_entry!(
        "SIFR-COMPONENT-0003",
        "COMPONENT",
        "Compiler component protocol version is incompatible.",
        Severity::Error,
        "crates/sifr_compiler_component/src/tests.rs",
        "compiler component protocol version is incompatible",
        "sifr_compiler_component",
        [],
        []
    ),
    active_entry!(
        "SIFR-COMPONENT-0004",
        "COMPONENT",
        "Compiler component protocol envelope is invalid.",
        Severity::Error,
        "crates/sifr_compiler_component/src/tests.rs",
        "compiler component protocol envelope is invalid",
        "sifr_compiler_component",
        [],
        []
    ),
    active_entry!(
        "SIFR-COMPONENT-0005",
        "COMPONENT",
        "Compiler component requested a forbidden capability.",
        Severity::Error,
        "crates/sifr_compiler_component/src/tests.rs",
        "compiler component requested a forbidden capability",
        "sifr_compiler_component",
        [],
        []
    ),
    active_entry!(
        "SIFR-COMPONENT-0006",
        "COMPONENT",
        "Compiler component exceeded a resource limit.",
        Severity::Error,
        "crates/sifr_compiler_component/src/tests.rs",
        "compiler component exceeded a resource limit",
        "sifr_compiler_component",
        [],
        []
    ),
    active_entry!(
        "SIFR-COMPONENT-0007",
        "COMPONENT",
        "Compiler component execution failed.",
        Severity::Error,
        "crates/sifr_compiler_component/src/tests.rs",
        "compiler component execution failed",
        "sifr_compiler_component",
        [],
        []
    ),
    active_entry!(
        "SIFR-COMPONENT-0008",
        "COMPONENT",
        "Compiler component cache operation failed.",
        Severity::Error,
        "crates/sifr_compiler_component/src/tests.rs",
        "compiler component cache operation failed",
        "sifr_compiler_component",
        [],
        []
    ),
    active_entry!(
        "SIFR-COMPONENT-0009",
        "COMPONENT",
        "Compiler component diagnostic registry is invalid.",
        Severity::Error,
        "crates/sifr_compiler_component/src/tests.rs",
        "compiler component diagnostic registry is invalid",
        "sifr_compiler_component",
        [],
        []
    ),
];
