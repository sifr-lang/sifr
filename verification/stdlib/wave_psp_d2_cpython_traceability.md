# `wave_psp_d2` CPython Traceability

Status: partially superseded by `milestone_concurrency_runtime_0a`.

The OS, environment, sys, logging, platform, time, and timeit anchors from this wave remain valid through `crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr`. The old `sifr.subprocess` parity anchors were removed when M0a rejected public CPython-shaped runtime/process adapters.

Current subprocess/process traceability lives in:

- `verification/stdlib/concurrency_runtime_substrate_inventory.*`
- `verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md`
- `verification/areas/runtime_platform/golden/legacy_sifr_runtime_surfaces_removed.sifr`

Future production process APIs are owned by `milestone_concurrency_runtime_4` under the native `sifr.process` namespace.
