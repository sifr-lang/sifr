# wave_psp_runtime_4 CPython Traceability Matrix

Wave: `wave_psp_runtime_4`

Status: superseded by `milestone_concurrency_runtime_0a`.

The previous `sifr.subprocess` compatibility fixtures from this wave were removed when the production concurrency/runtime substrate rejected public CPython-shaped process adapters. Historical CPython subprocess evidence is retained in `verification/areas/stdlib_parity/data/concurrency_runtime_substrate_inventory.json` and `verification/areas/stdlib_parity/reports/concurrency_runtime_substrate_inventory.md`; public process work now belongs to the native `sifr.process` milestone.

Current anchors:

- Legacy public-module removal: `verification/areas/runtime_platform/golden/legacy_sifr_runtime_surfaces_removed.sifr`
- Negative import fixtures: `crates/sifr/tests/e2e/fail/legacy_sifr_subprocess_removed.sifr` and `crates/sifr/tests/e2e/fail/async_popen_unsupported.sifr`
- Native future owner: `milestone_concurrency_runtime_4`
