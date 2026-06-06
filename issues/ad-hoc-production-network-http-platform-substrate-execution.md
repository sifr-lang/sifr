# Ad Hoc Phase Execution: Production Network and HTTP Platform Substrate

Phase contract: [ad-hoc-production-network-http-platform-substrate.md](./ad-hoc-production-network-http-platform-substrate.md)

Status: draft

## Scope Split

The original broad planning scan was split into three implementation phases:

- This ledger tracks the production network/TLS/URL/HTTP substrate: `sifr.net`, `sifr.tls`, `sifr.url`, accepted `sifr.http` protocol/runtime primitives, typed errors, async suspension points, resource limits, observability hooks, and internal loopback harnesses.
- [ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md](./ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md) tracks queues, subprocess/process pools/multiprocessing, and runtime ergonomics.
- [ad-hoc-production-text-i18n-stdlib-parity-execution.md](./ad-hoc-production-text-i18n-stdlib-parity-execution.md) tracks text and internationalization.

CPython-shaped public networking/web modules are no longer this phase's objective. `sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib.request`, `sifr.http.client`, `sifr.http.server`, and `sifr.socketserver` are deferred or rejected unless a later product phase proves migration demand and delegates to the Sifr-native substrate.

## Milestone Checklist

- [ ] `milestone_network_http_0`: Product Boundary And Architecture
- [ ] `milestone_network_http_1`: Async Network Runtime
- [ ] `milestone_network_http_2`: TLS Runtime
- [ ] `milestone_network_http_3`: URL, Header, And Cookie Primitives
- [ ] `milestone_network_http_4`: HTTP Core Transport
- [ ] `milestone_network_http_5`: Integration, Documentation, And Production Handoff

## Planning Reviews

- Initial Claude planning review covered the original combined stdlib scan.
  - Full-file Claude attempts stalled before output and produced no retained review content.
  - Embedded-summary review completed:
    - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-1d.md`
  - Result: `FAIL`; blockers folded into the original phase draft.
- Follow-up Claude planning reviews on the original parity plan:
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-2.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-3.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-4.md`
  - Result: original split-phase parity plan reached `PASS`, then was superseded by the cleaner production-substrate scope.
- Split-phase Claude reviews:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-1-constrained.md` through `reviews/ad-hoc-production-split-stdlib-phases-review-pass-20-sifr-namespace-readiness.md`
  - Result: previous parity-oriented phase docs reached readiness, then were superseded for this network/web phase only by the product-boundary reset recorded below.
- Reviewer-driven scope reset:
  - Source: user-provided reviewer notes on avoiding toy compatibility modules and keeping only production substrate / production developer experience.
  - Result: `sifr.net`, `sifr.tls`, `sifr.url`, accepted `sifr.http` substrate, typed errors, async streams, blocking-I/O diagnostics, observability hooks, and internal loopback harnesses remain in scope.
  - Result: CPython-shaped public modules are no longer accepted as the phase's success criteria.

## Planning Review Remediation Retained In This Phase

- [x] Keep native runtime primitives as the implementation root.
- [x] Keep Tokio as the backing async runtime, with concrete feature expansion required in M0.
- [x] Keep async TLS stream boundary and reject public event-loop retry model.
- [x] Keep HTTP/HTTPS dependency on M2 TLS substrate.
- [x] Keep cross-phase dependency contract for text/i18n and concurrency/runtime consumers.
- [x] Keep non-UTF-8 URL/HTTP text behavior blocked on text/i18n `milestone_text_i18n_1`.
- [x] Keep concrete network/TLS/HTTP typed error hierarchy and cross-module nesting requirements.
- [x] Keep workload classification and async-context diagnostics for network/TLS/HTTP.
- [x] Keep external-review owner and five-working-day fallback rule.
- [x] Keep namespace cleanup alignment: public imports remain under `sifr.*`, and bare CPython stdlib names are not aliases.
- [x] Replace CPython stdlib parity with production substrate as the completion goal.
- [x] Add No-Toy-Module Gate and Maintenance Burden Test.
- [x] Reject/defer public `sifr.http.server`, `sifr.socketserver`, `sifr.urllib.request`, `sifr.http.client`, `sifr.select`, `sifr.selectors`, and CPython descriptor-shaped socket/TLS APIs in this phase.
- [x] Add explicit Phase 41 handoff and separate production HTTP client handoff.
- [x] Keep CPython scans as evidence mining, not parity backlog.
- [x] Classify HTTP/2 and HTTP/3 as deferred future protocol work with revisit rules.
- [x] Make stream I/O buffer ownership/lifetime semantics an M0 gate before M1 implementation.
- [x] Add mTLS/client certificate authentication as an M0/M2 TLS classification item.

## Implementation PRs

- M0: pending.
- M1: pending.
- M2: pending.
- M3: pending.
- M4: pending.
- M5: pending.

## Validation Evidence

Record local validation for each milestone before opening its PR.

Required baseline commands:

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
python3 scripts/check_hir_maintainability_guardrails.py
scripts/run_all_tests.sh --profile create-pr
```

Required merge-gate command before milestone closure:

```bash
scripts/run_all_tests.sh
```

## Required Tracking Artifacts

M0 must create and keep current:

- [ ] `verification/stdlib/network_http_substrate_inventory.md`
- [ ] `verification/stdlib/network_http_substrate_inventory.json`
- [ ] `verification/stdlib/network_http_cpython_evidence_matrix.md`
- [ ] one traceability document per milestone domain under `verification/stdlib/`

Opening the M0 implementation PR is blocked until the artifact locations and schemas are present in that PR.

## CPython Evidence Scan

Each milestone must record:

- CPython source files scanned.
- CPython docs files scanned.
- CPython tests scanned.
- Public APIs classified as production-public, production-substrate, internal-test, deferred, rejected, blocked, or host-limited.
- CPython behavior mined into Sifr-native tests.
- CPython behavior rejected as legacy, unsafe, toy, dynamic, descriptor-shaped, raw-event-loop, or non-product.
- Sifr e2e pass/fail fixtures added.

Evidence-family states:

- `mined`
- `blocked`
- `rejected`
- `external-signal`

## Decision Index

No implementation decisions recorded yet.

Every deferred/rejected/host-limited/internal-only decision must include:

- surface
- terminal state
- rationale
- revisit rule
- CPython evidence where relevant
- Sifr regression fixture or diagnostic test where relevant
