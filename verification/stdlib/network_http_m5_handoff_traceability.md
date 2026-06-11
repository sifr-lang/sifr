# Network HTTP M5 Traceability: Integration, Documentation, And Production Handoff

Status: backlog from M0.

| Work item | M0 decision | Acceptance evidence |
| --- | --- | --- |
| Public docs | Document `sifr.net`, `sifr.tls`, `sifr.url`, and `sifr.http` substrate types. | Docs updated with rejected CPython-shaped surfaces and provider dependency states. |
| Architecture docs | Record runtime/networking/TLS/HTTP boundaries and dependency manifests. | `internal_docs/architecture.md`, roadmap/phase docs, and verification artifacts updated. |
| Demos | TCP echo, TLS loopback, HTTP transport loopback. | Demos compile/run deterministically without external network. |
| Generated dependency snapshots | All feature combinations, including Ring 5 absence from production. M0 `network_http_dependency_snapshots.json` is a planning snapshot; M5 must replace or supplement it with resolver-backed generated snapshots from actual feature wiring. | Resolver-backed snapshots and tests. |
| Panic scan | Network/TLS/URL/HTTP emitted paths contain no user-triggerable panic. | Generated-code quality panic scan and targeted fixtures. |
| Inventory closure | No `open` state; every deferred/rejected/host-limited entry has rationale and revisit rule. | Final inventory diff, evidence matrix, and decision index closed. |
| Phase 41 handoff | Protocol/runtime ready; multi-core throughput deferred to serving-scale follow-up. | Handoff doc and linked issue identifier. |
| HTTP client phase handoff | Transport substrate ready; policy features deferred. | Handoff doc for pooling, retries, redirects, auth, cookies, proxies, streaming. |
| Final review | Reviewer loop reaches `PASS`. | Review artifacts in `reviews/` and execution ledger entry. |
