# Text/I18n M0 Traceability

Milestone: `milestone_text_i18n_0`

| Requirement | Artifact |
| --- | --- |
| Shared platform contract | `verification/platform/platform_contract.md`, `verification/platform/platform_contract.json` |
| Supported host matrix | `verification/platform/supported_host_matrix.md` |
| Golden manifest and runner | `verification/platform/golden/manifest.json`, `scripts/run_platform_golden.sh` |
| Text/i18n substrate inventory | `verification/stdlib/text_i18n_substrate_inventory.md`, `.json` |
| CPython reference classification | `verification/stdlib/text_i18n_reference_matrix.md` |
| Dependency decision records | `verification/stdlib/text_i18n_dependency_decisions.md` |
| Negative bare CPython import fixtures | `crates/sifr/tests/e2e/fail/bare_cpython_text_i18n_imports.sifr`, `bare_cpython_encodings_import.sifr`, `bare_cpython_dotted_codecs_import.sifr`, `bare_cpython_unicodedata_import.sifr`, `bare_cpython_locale_import.sifr`, `bare_cpython_gettext_import.sifr` |
| Binary I/O prerequisite | `demos/binary_files/main.sifr`, `demos/bytes_file_io/main.sifr`, `verification/platform/golden/binary_file_io_contract.sifr` |
| External review PASS | `reviews/ad-hoc-production-text-i18n-m0-implementation-review-pass-2.md` |

M0 review must verify that no implementation milestone opens before the external review approves these artifacts.
