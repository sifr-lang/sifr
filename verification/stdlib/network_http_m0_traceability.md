# Network HTTP M0 Traceability

Milestone: `milestone_network_http_0`

| Requirement | Evidence |
| --- | --- |
| Public/internal/deferred/rejected surface classification | `verification/stdlib/network_http_substrate_inventory.md` and `.json` |
| CPython evidence scan and state assignment | `verification/stdlib/network_http_cpython_evidence_matrix.md` |
| Workload classification and async diagnostics backlog | `verification/stdlib/network_http_workload_database.md` |
| Rust ecosystem and Ring 5 production absence proof | `verification/stdlib/network_http_dependency_snapshots.json` |
| Per-crate dependency audit fields | `verification/stdlib/network_http_dependency_audit.md` |
| Shared platform contract | `verification/areas/runtime_platform/platform_contract.md` and `.json` |
| Supported-host baseline rows | `verification/areas/runtime_platform/supported_host_matrix.md` |
| Cross-phase golden fixtures | `verification/areas/runtime_platform/golden/unsupported_cpython_network_imports.sifr`, `unsupported_cpython_tls_imports.sifr`, `unsupported_cpython_url_imports.sifr`, `unsupported_cpython_http_imports.sifr`, `unsupported_cpython_readiness_imports.sifr`, and manifest entries |
| Multi-core serving follow-up | `issues/ad-hoc-network-http-serving-scale-follow-up.md` (`ad-hoc-network-http-serving-scale-follow-up`) |
| Unsupported CPython-shaped diagnostics | `crates/sifr_stdlib/src/lib.rs`, M0 e2e fail fixtures under `crates/sifr/tests/e2e/fail/` |

## M0 Closure Gate

M0 is complete only after:

- the inventory has no `open` terminal states;
- every M1-M5 implementation milestone has a traceability document and concrete backlog entries;
- the reviewer loop returns `PASS`;
- local validation for the M0 PR is recorded in `issues/ad-hoc-production-network-http-platform-substrate-execution.md`.
