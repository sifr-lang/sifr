# `stdlib_parity_d2` CPython Traceability

Status: partially superseded by `concurrency-runtime legacy-subprocess rejection`.

The OS, environment, sys, logging, platform, time, and timeit anchors from this implementation pass remain valid through `crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr`. The old `sifr.subprocess` parity anchors were removed when legacy-subprocess rejection capability rejected public CPython-shaped runtime/process adapters.

Current subprocess/process traceability lives in:

- `verification/areas/stdlib_parity/data/concurrency_runtime_substrate_inventory.json`
- `verification/areas/stdlib_parity/reports/concurrency_runtime_substrate_inventory.md`
- `verification/areas/stdlib_parity/reports/concurrency_runtime_cpython_evidence_matrix.md`
- `verification/areas/runtime_platform/golden/legacy_sifr_runtime_surfaces_removed.sifr`

Future production process APIs are owned by `concurrency-runtime process supervision` under the native `sifr.process` namespace.
