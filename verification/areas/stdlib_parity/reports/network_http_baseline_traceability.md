# Network HTTP network/HTTP baseline capability Traceability

Capability: `network/HTTP baseline`

| Requirement | Evidence |
| --- | --- |
| Public/internal/deferred/rejected surface classification | `verification/areas/stdlib_parity/reports/network_http_substrate_inventory.md` and `verification/areas/stdlib_parity/data/network_http_substrate_inventory.json` |
| CPython evidence scan and state assignment | `verification/areas/stdlib_parity/reports/network_http_cpython_evidence_matrix.md` |
| Workload classification and pending async diagnostics | `verification/areas/stdlib_parity/reports/network_http_workload_database.md` |
| Rust ecosystem and Ring 5 production absence proof | `verification/areas/stdlib_parity/data/network_http_dependency_snapshots.json` |
| Per-crate dependency audit fields | `verification/areas/stdlib_parity/reports/network_http_dependency_audit.md` |
| Shared platform rules | `verification/areas/runtime_platform/platform_rules.md` and `verification/areas/runtime_platform/platform_rules.json` |
| Supported-host baseline rows | `verification/areas/runtime_platform/supported_host_matrix.md` |
| Cross-capability golden fixtures | `verification/areas/runtime_platform/golden/unsupported_cpython_network_imports.sifr`, `unsupported_cpython_tls_imports.sifr`, `unsupported_cpython_url_imports.sifr`, `unsupported_cpython_http_imports.sifr`, `unsupported_cpython_readiness_imports.sifr`, and manifest entries |
| Multi-core serving capability | `network-http-serving-scale-capability record` (`network-http-serving-scale-capability`) |
| Unsupported CPython-shaped diagnostics | `crates/sifr_stdlib_imports/src/lib.rs`, network/HTTP baseline capability e2e fail fixtures under `crates/sifr/tests/e2e/fail/` |

## Network/HTTP Baseline Readiness Gate

Network/HTTP baseline readiness is complete only after:

- the inventory has no `open` terminal states;
- every network/HTTP implementation capability has a traceability document and concrete evidence records;
- the validation loop returns `PASS`;
- local validation for the network/HTTP baseline PR is recorded in `issues/production-network-http-platform-substrate-execution.md`.
