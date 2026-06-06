**PASS** — manifest cleanup is sound, no blockers.

Verified:
- `verification/platform/golden/locale_host_limited_formatting.sifr` imports only `sifr.i18n` symbols (`Collator`, `FormatError`, `LocaleId`, `NumberFormatter`, `host_locale`); no `text-open`, no `sifr.encoding`, no implicit default encoding paths.
- `depends_on: ["milestone_text_i18n_3"]` is correct — M3 is the locale-formatting milestone (`verification/stdlib/text_i18n_substrate_inventory.json:42,258,271,284,297,310` all own M3-scoped locale APIs).
- `must_not_depend_on: ["milestone_text_i18n_1"]` correctly asserts orthogonality from the encoding-IO substrate (M1). Matches the contract spelled out in `verification/stdlib/text_i18n_m3_traceability.md:9`: "never feeds text I/O defaults."
- `checks` entry "no implicit text-open default encoding" is consistent with the source file and `must_not_depend_on`.
- Field shape matches other entries (`binary_file_io_contract.sifr` uses the same `must_not_depend_on: ["milestone_text_i18n_1"]` form, schema_version 1 unchanged, all keys present).
- `must_not_depend_on` is contract/documentation (no validator parses it across the repo), so the addition cannot introduce a regression.

M3 platform golden remains sound.
