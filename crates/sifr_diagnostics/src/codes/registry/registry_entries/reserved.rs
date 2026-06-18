//! Reserved family bases and retired diagnostic-code slots.

use super::super::DiagnosticRegistryEntry;
use super::super::{reserved_code, reserved_family_base};

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    reserved_family_base("SIFR-PARSE-0000", "PARSE"),
    reserved_family_base("SIFR-NAME-0000", "NAME"),
    reserved_family_base("SIFR-IMPORT-0000", "IMPORT"),
    reserved_family_base("SIFR-TYPE-0000", "TYPE"),
    reserved_family_base("SIFR-ASYNC-0000", "ASYNC"),
    reserved_family_base("SIFR-PYENV-0000", "PYENV"),
    reserved_family_base("SIFR-PYIMP-0000", "PYIMP"),
    reserved_family_base("SIFR-PYCALL-0000", "PYCALL"),
    reserved_family_base("SIFR-PYCONV-0000", "PYCONV"),
    reserved_family_base("SIFR-PYRES-0000", "PYRES"),
    reserved_family_base("SIFR-PYZC-0000", "PYZC"),
    reserved_family_base("SIFR-PYCB-0000", "PYCB"),
    reserved_family_base("SIFR-PYTRUST-0000", "PYTRUST"),
    reserved_family_base("SIFR-DECIMAL-0000", "DECIMAL"),
    reserved_family_base("SIFR-INT-0000", "INT"),
    reserved_family_base("SIFR-IO-0000", "IO"),
    reserved_family_base("SIFR-ENCODING-0000", "ENCODING"),
    reserved_code(
            "SIFR-INT-0002",
            "INT",
            "Reserved for implicit narrowing from exact or fixed-width integer sources to narrower fixed-width targets.",
        ),
    reserved_code(
            "SIFR-INT-0008",
            "INT",
            "Reserved for fixed-width array, tensor, or dataframe arithmetic without an explicit overflow policy.",
        ),
    reserved_code(
            "SIFR-INT-0009",
            "INT",
            "Reserved for JSON or web-safe integer serialization policy failures.",
        ),
    reserved_code(
            "SIFR-INT-0010",
            "INT",
            "Reserved for bytes or bytearray construction and mutation values that do not fit uint8.",
        ),
    reserved_code(
            "SIFR-TYPE-0903",
            "TYPE",
            "Retired: direct annotated workload calls from async code are now ASYNC-family errors.",
        ),
    reserved_family_base("SIFR-CALL-0000", "CALL"),
    reserved_family_base("SIFR-OWN-0000", "OWN"),
    reserved_family_base("SIFR-FLOW-0000", "FLOW"),
    reserved_family_base("SIFR-FMT-0000", "FMT"),
    reserved_family_base("SIFR-LINT-0000", "LINT"),
    reserved_family_base("SIFR-MATCH-0000", "MATCH"),
    reserved_family_base("SIFR-PROTO-0000", "PROTO"),
    reserved_family_base("SIFR-CLASS-0000", "CLASS"),
    reserved_family_base("SIFR-RESULT-0000", "RESULT"),
    reserved_family_base("SIFR-STDLIB-0000", "STDLIB"),
    reserved_family_base("SIFR-WORKSPACE-0000", "WORKSPACE"),
    reserved_family_base("SIFR-PACKAGE-0000", "PACKAGE"),
    reserved_code(
            "SIFR-PACKAGE-0105",
            "PACKAGE",
            "Retired: Cargo credential failures are wrapped by SIFR-PACKAGE-0101.",
        ),
    reserved_code(
            "SIFR-PACKAGE-0302",
            "PACKAGE",
            "Reserved for future backend trust diagnostics.",
        ),
    reserved_code(
            "SIFR-PACKAGE-0306",
            "PACKAGE",
            "Reserved for future backend trust and feature diagnostics.",
        ),
    reserved_code(
            "SIFR-PACKAGE-0307",
            "PACKAGE",
            "Reserved for future backend trust and feature diagnostics.",
        ),
    reserved_code(
            "SIFR-PACKAGE-0308",
            "PACKAGE",
            "Reserved for future backend trust and feature diagnostics.",
        ),
    reserved_code(
            "SIFR-PACKAGE-0309",
            "PACKAGE",
            "Reserved for future backend trust and feature diagnostics.",
        ),
    reserved_family_base("SIFR-CODEGEN-0000", "CODEGEN"),
    reserved_family_base("SIFR-BUILD-0000", "BUILD"),
    reserved_family_base("SIFR-INTERNAL-0000", "INTERNAL"),
];
