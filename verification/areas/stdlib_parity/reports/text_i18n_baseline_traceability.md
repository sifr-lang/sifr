# Text/I18n Baseline Traceability

Capability: `text/i18n baseline`

| Requirement | Artifact |
| --- | --- |
| Shared platform rules | `verification/areas/runtime_platform/platform_rules.md`, `verification/areas/runtime_platform/platform_rules.json` |
| Supported host matrix | `verification/areas/runtime_platform/supported_host_matrix.md` |
| Golden manifest and runner | `verification/areas/runtime_platform/golden/manifest.json`, `uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite platform-golden` |
| Text/i18n substrate inventory | `verification/areas/stdlib_parity/reports/text_i18n_substrate_inventory.md`, `verification/areas/stdlib_parity/data/text_i18n_substrate_inventory.json` |
| CPython reference classification | `verification/areas/stdlib_parity/reports/text_i18n_reference_matrix.md` |
| Dependency decision records | `verification/areas/stdlib_parity/reports/text_i18n_dependency_decisions.md` |
| Negative bare CPython import fixtures | `crates/sifr/tests/e2e/fail/bare_cpython_text_i18n_imports.sifr`, `bare_cpython_encodings_import.sifr`, `bare_cpython_dotted_codecs_import.sifr`, `bare_cpython_unicodedata_import.sifr`, `bare_cpython_locale_import.sifr`, `bare_cpython_gettext_import.sifr` |
| Binary I/O prerequisite | `demos/binary_files/main.sifr`, `demos/bytes_file_io/main.sifr`, `verification/areas/runtime_platform/golden/binary_file_io_capability.sifr` |
| External review PASS | `reviews/production-text-i18n-baseline-implementation-review-pass-2.md` |

Text/i18n baseline review must verify that no implementation capability opens before the external review approves these artifacts.
