# Ad Hoc Phase Execution: Production Network and HTTP Platform Substrate

Phase contract: [ad-hoc-production-network-http-platform-substrate.md](./ad-hoc-production-network-http-platform-substrate.md)

Status: draft

## Scope Split

The original broad planning scan was split into three implementation phases:

- This ledger tracks the production network/TLS/URL/HTTP substrate: `sifr.net`, `sifr.tls`, `sifr.url`, accepted `sifr.http` protocol/runtime primitives, typed errors, async suspension points, resource limits, observability hooks, and internal loopback harnesses.
- [ad-hoc-production-concurrency-runtime-platform-substrate-execution.md](./ad-hoc-production-concurrency-runtime-platform-substrate-execution.md) tracks concurrency/process/runtime substrate.
- [ad-hoc-production-text-i18n-platform-substrate-execution.md](./ad-hoc-production-text-i18n-platform-substrate-execution.md) tracks text/Unicode/encoding/i18n runtime substrate.

Execution order: this is the third phase in the split production-stdlib sequence. Text/i18n runs first, concurrency/runtime runs second, and network/HTTP consumes both provider phases. Network/HTTP implementation must not start early or close text-dependent/runtime-dependent surfaces without the relevant provider milestones recorded as complete.

CPython-shaped public networking/web modules are no longer this phase's objective or a future adapter track. `sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib.request`, `sifr.urllib.parse`, `sifr.http.client`, `sifr.http.server`, and `sifr.socketserver` are evidence only and must resolve to `rejected`, `unsupported-with-diagnostic`, `internal-only`, or `test-only-harness`.

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
- Rust ecosystem strategy review:
  - `reviews/ad-hoc-production-network-http-substrate-review-pass-23-ecosystem.md`
  - Result: `PASS`; crate stack, from-scratch rejection policy, M0 dependency decision records, no-fallback policy, and execution ledger alignment were accepted.
- Rust ecosystem final review:
  - `reviews/ad-hoc-production-network-http-substrate-review-pass-24-ecosystem-final.md`
  - Result: `FAIL`; M0 dependency records were not in the milestone definition of done, and conditional crates such as `x509-parser` could bypass the audit requirement. Both findings were remediated.
- Text/i18n dependency discovery:
  - Result: complete; network/HTTP text-dependent surfaces were discovered and classified across URL parsing/building, percent encoding, query/form behavior, headers, bodies, cookies, TLS certificate display, diagnostics, observability, demos, Phase 41 handoff, and HTTP client handoff.
- Text/i18n dependency review:
  - `reviews/ad-hoc-production-network-http-substrate-review-pass-26-text-dependency.md`
  - Result: `PASS`; the dependency matrix, M1/M2/M2.5/M3 blockers, no-local-decoding rule, binary substrate readiness, and ledger alignment were accepted.
- Text/i18n dependency final reviews:
  - `reviews/ad-hoc-production-network-http-substrate-review-pass-27-text-dependency-final.md`
  - Result: `PASS`; final cross-phase decisions and ledger state were accepted.
  - `reviews/ad-hoc-production-network-http-substrate-review-pass-28-text-dependency-handoff.md`
  - Result: `PASS`; explicit Phase 41 and HTTP client handoff rows introduced no contradictions and completed the dependency matrix.
- Cross-phase implementation-readiness review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-21-phase-order-readiness.md`
  - Result: `FAIL`; missing concurrency/runtime dependency matrix, local cancellation/shutdown ownership ambiguity, and legacy filename naming-note gaps were remediated.
- Cross-phase implementation-readiness follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-22-phase-order-readiness.md`
  - Result: `PASS`; pass 21 remediations were verified, with one minor state-vocabulary inconsistency remediated.
- Final cross-phase implementation-readiness verification:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-23-final-readiness.md`
  - Result: `PASS`; no material blockers, stale labels, or implementation-blocking contradictions remained.
- Final implementation-readiness review:
  - `reviews/ad-hoc-production-network-http-substrate-review-pass-29-final-readiness.md`
  - Result: `PASS`; HTTP/2 conformance DoD coverage, concurrency/runtime dependency classification, M3/M4 header ownership, and reviewer tracking were tightened after review.
- Unmade-decision discovery review:
  - `reviews/ad-hoc-production-network-http-substrate-review-pass-31-unmade-decisions-discovery.md`
  - Result: `FAIL`; 14 unresolved decisions were found across rustls provider/roots, DNS, stream ownership, `sifr.http` path, UDP, Tower, OpenTelemetry, `x509-parser`, `socket2`, mTLS, multipart, upgrade hooks, and Tokio features. The phase doc now records fixed decisions for each.
- Decision remediation review:
  - `reviews/ad-hoc-production-network-http-substrate-review-pass-32-decisions-remediation.md`
  - Result: `FAIL`; all 14 decisions were resolved, but `tower-service` crate identity and the M2 mTLS definition-of-done gate needed tightening. Both findings were remediated.
- Final decision-readiness review:
  - `reviews/ad-hoc-production-network-http-substrate-review-pass-33-final-decision-readiness.md`
  - Result: `PASS`; no unmade decisions or contradictions remained. Cosmetic Tower/Tokio wording was tightened afterward.
- Cross-phase decision-closure review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-24-decision-closure.md`
  - Result: `PASS`; all material product/API/dependency decisions across text/i18n, concurrency/runtime, and network/HTTP were clear enough for implementation.
- Final cross-phase decision delta review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-25-final-delta.md`
  - Result: `PASS`; final cross-phase and no-bespoke-policy clarifications introduced no unmade or contradictory implementation decisions.
- Clean-language and dependency-ring review:
  - Source: local review after the concurrency/runtime substrate cleanup and dependency-policy creation.
  - Result: phase doc tightened to remove the future CPython-shaped adapter path, classify Python-shaped network/web modules as rejected or unsupported diagnostics, replace the flat Rust ecosystem table with dependency-ring decisions, reject implementation-time crate-family discovery, align Tokio features with the concurrency/runtime provider boundary, and add a network-owned security/resource model.
  - Claude review attempt: `reviews/ad-hoc-production-network-http-platform-substrate-review-pass-1.md` was started but produced no content and was removed; no external review result was retained for this pass.
- Reviewer dependency and contract tightening:
  - Source: user-provided review notes on Hyper tracing, UDP gating, URL/IDNA, TLS build/platform contracts, DNS semantics, byte buffers, HTTP substrate types, body streams, Hyper-Util conditionality, and M5 formatting.
  - Result: `PASS after contract fixes`; all required fixes were applied in the phase doc.
  - Remediations: removed Hyper's unstable `tracing` feature, made Hyper-Util conditional/internal-only, made UDP M0-gated, added URL/IDNA guard, added TLS generated-build and platform-verifier host requirements, added TLS write/flush/shutdown contract, added public byte-buffer and DNS semantics gates, defined `sifr.http` substrate type/body stream contracts, clarified crate patch versions as M0 lockfile pins, fixed M5 inventory indentation, and clarified Content-Encoding compression versus HPACK.
- Claude implementation-readiness review pass 1:
  - `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-1.md`
  - Result: `PASS`; Claude found no blocking implementation-readiness gaps.
  - Non-blocking polish applied afterward: clarified conditional `server-graceful`, assigned metrics schema ownership to the runtime/networking phase owner, and added direct/transitive `h2` lockfile coherence requirements.
- Claude implementation-readiness review pass 2:
  - `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-2.md`
  - Result: `PASS`; Claude confirmed the post-pass-1 polish introduced no contradictions and found no remaining CPython fallback path, dependency-ring gap, unmade provider/ecosystem decision, M0 contract hole, milestone-ordering conflict, security/resource omission, or text/runtime provider substitution risk.
- Reviewer serving-scale and stream-shape review:
  - Source: user-provided review on the unowned multi-core serving story, missing TCP full-duplex API shape, and missing TCP half-close semantics.
  - Result: `PASS after M0 contract fixes`; the phase now records a single-runtime-worker-per-process v1 serving boundary, requires M0 to create or link the multi-core serving follow-up, adds owned `TcpStream` split halves as the full-duplex model, and makes write-side TCP half-close an M1 substrate requirement.
- Claude Opus implementation-readiness review pass 1:
  - `reviews/ad-hoc-production-network-http-platform-substrate-opus-review-pass-1.md`
  - Result: `CONDITIONAL PASS`; body-stream contract ownership, `TcpStream.split()` failure shape, write-after-`shutdown_write`, and unsplit half-close handle disposition needed contract fixes.
  - Remediations: M0 now owns body-stream contract definition while M4 implements it, `split()` is infallible and affine-consuming, write-after-shutdown returns a stable typed error, unsplit `shutdown_write()` preserves read usability, split-half close/error evidence must come from underlying socket state, the serving-scale follow-up needs a stable recorded identifier, URL/IDNA backend approval requires text/i18n owner sign-off, M3 namespace ownership is explicit, Phase 41 deferred capability scope is explicit, and Hyper-Util graceful shutdown has a Sifr-owned baseline.
- Claude Opus implementation-readiness review pass 2:
  - `reviews/ad-hoc-production-network-http-platform-substrate-opus-review-pass-2.md`
  - Result: `PASS`; all pass-1 blockers and recommended edits were verified as remediated, no new blocking contradictions or decision-by-discovery holes remained, and the phase was judged implementation-ready for M0.
- Implementation-readiness merge ledger:
  - PR: https://github.com/sifr-lang/sifr/pull/2490
  - Merge commit: `f30e31f9e`
  - Scope: finalized network/HTTP implementation-readiness planning contracts, including UDP gating, URL/IDNA behavior, TLS build/platform semantics, DNS/address resolution, byte buffers, HTTP substrate/body stream types, Hyper/Hyper-Util boundaries, metrics ownership, direct/transitive `h2` lockfile coherence, and retained Claude implementation-readiness review artifacts.
  - Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` required before PR.

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
- [x] Reject or route to unsupported diagnostics public `sifr.http.server`, `sifr.socketserver`, `sifr.urllib.request`, `sifr.urllib.parse`, `sifr.http.client`, `sifr.select`, `sifr.selectors`, and CPython descriptor-shaped socket/TLS APIs in this phase.
- [x] Add explicit Phase 41 handoff and separate production HTTP client handoff.
- [x] Keep CPython scans as evidence mining, not parity backlog.
- [x] Bring HTTP/2 into the production substrate and keep HTTP/3 / QUIC deferred with a revisit rule.
- [x] Add dependency-ring decisions for network, DNS, TLS, URL, HTTP/1, HTTP/2, cookies, observability, and tests using `internal_docs/dependency_policy.md`.
- [x] Require M0 to verify the locked Rust Ecosystem Decisions table; if the ecosystem stack cannot satisfy a required surface, defer that surface with evidence instead of hand-rolling protocol/domain infrastructure in this phase.
- [x] Add M0 definition-of-done gate for dependency decision records across every Rust Ecosystem Decisions crate family.
- [x] Ensure conditional crates such as `x509-parser` receive the same dependency audit as baseline crates.
- [x] Add a text/i18n dependency matrix for binary/ASCII-safe substrate versus features blocked on text/i18n M1, M2, M2.5, or M3.
- [x] Add a concurrency/runtime dependency matrix for features blocked on task/cancellation M1, sync/backpressure M2, offload M3, shutdown/diagnostics M5, and IPC/process-worker M6.
- [x] Require network/HTTP consumers to call `sifr.encoding`, `sifr.unicode`, `sifr.io`, or `sifr.i18n` rather than adding local encoding, Unicode, locale, or fallback-decoder behavior.
- [x] Require network/HTTP consumers to call the concurrency/runtime provider substrate rather than adding local cancellation, timeout, shutdown, offload, executor, task-context, diagnostics, queue/channel, process/worker, or IPC substitutes.
- [x] Make stream I/O buffer ownership/lifetime semantics an M0 gate before M1 implementation.
- [x] Add mTLS/client certificate authentication as an M0/M2 TLS classification item.
- [x] Require HTTP/2 loopback coverage for SETTINGS negotiation, RST_STREAM cancellation, GOAWAY graceful shutdown, and HPACK correctness edge cases selected in the M0 conformance inventory.
- [x] Make M3 the owner of canonical URL/header/cookie primitives and M4 the consumer to prevent duplicate HTTP representations.
- [x] Resolve ecosystem and API decisions before M0: Tokio feature set, rustls `aws-lc-rs`, `rustls-platform-verifier`, `tokio::net::lookup_host`, owned-buffer stream I/O, M0-gated UDP, `socket2` option set, `sifr.http` path, `tower-service` handoff, conditional Hyper-Util, OTel deferral, mTLS inclusion, multipart deferral, internal-only upgrade hooks, and external CPython test handling.
- [x] Pin the service handoff crate to `tower-service` only and add an M2 DoD gate for mTLS loopback success/rejection.
- [x] Lock clean-language policy for networking: no backward-compatibility shim, migration path, bridge alias, fallback path, or CPython-shaped adapter track survives this phase.
- [x] Add network-owned security/resource rows for TLS defaults, root stores, request smuggling, header normalization, HTTP/2 abuse, size limits, URL authority security, cookie-header scope, compression deferral, redaction, and external-network test policy.
- [x] Add M0 contracts for public byte-buffer semantics, DNS/address resolution, TLS write/flush/shutdown, URL/IDNA guard behavior, `sifr.http` substrate types, and HTTP body streams.
- [x] Remove Hyper's unstable `tracing` feature from accepted dependencies; Sifr emits wrapper-level `tracing` spans/events instead.
- [x] Make Hyper-Util conditional/internal-only and prefer Hyper plus Sifr-owned adapters before adding it.
- [x] Assign network metrics schema ownership and require direct/transitive `h2` lockfile coherence verification in M0.
- [x] Add explicit v1 serving-scale boundary: this phase is single-runtime-worker per process, and M0 must create or link the follow-up that owns multi-core serving throughput.
- [x] Add TCP full-duplex ownership contract using owned split read/write halves instead of shared mutable stream aliasing.
- [x] Add TCP write-side half-close as an M1 substrate requirement with M0 semantics for repeated shutdown, split-half behavior, cancellation, and partial-progress evidence.
- [x] Resolve Claude Opus pass 1 blockers: M0 owns body-stream contract definition, `TcpStream.split()` is infallible, write-after-shutdown is typed, and unsplit `shutdown_write()` preserves the read side.
- [x] Record Claude Opus pass 2 `PASS` confirming no remaining implementation-readiness blockers.

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

## Review Ownership

- Phase owner: runtime/networking implementation owner.
- Designated compiler/runtime reviewer: assign in the M0 implementation PR before the first implementation milestone is marked complete.
- External/final review fallback: M5 may use the five-working-day fallback rule only after this ledger records the reviewer assignment, posted review artifact, attempted follow-ups, open questions, and conservative self-review.

## CPython Evidence Scan

Each milestone must record:

- CPython source files scanned.
- CPython docs files scanned.
- CPython tests scanned.
- Public APIs classified with shared terminal states and stability levels.
- Rust ecosystem crates accepted/rejected with feature flags, public API leak checks, typed error mapping, panic/unsafe audit notes, and conformance evidence.
- Any change to a resolved ecosystem decision must include a blocking implementation finding, replacement decision, and the same dependency audit fields.
- Text/i18n dependency states recorded for URL, percent encoding, query/form, header, body, cookie, content-type/charset, certificate-display, diagnostics, observability, demos, Phase 41 handoff, and HTTP client handoff surfaces.
- Concurrency/runtime dependency states recorded for async suspension points, server accept/dispatch/shutdown, blocking sync helpers, offload, signal-driven shutdown, and subprocess/process-pool-dependent demos.
- CPython behavior classified as `mined-as-substrate-fixture` when it becomes a Sifr-native test.
- CPython behavior rejected as legacy, unsafe, toy, dynamic, descriptor-shaped, raw-event-loop, or non-product.
- Sifr e2e pass/fail fixtures added.

Evidence-family states:

- `mined-as-substrate-fixture`
- `adapted-for-sifr-api`
- `compat-adapter-deferred` (shared vocabulary only; intentionally unused by this phase)
- `blocked-on-phase-X`
- `rejected`
- `external-signal`
- `waived-with-rationale`

## Decision Index

No implementation decisions recorded yet.

Every `deferred-to-phase-X`, `rejected`, `host-limited`, `internal-only`, or `unsupported-with-diagnostic` decision must include:

- surface
- terminal state
- rationale
- revisit rule
- CPython evidence where relevant
- Sifr regression fixture or diagnostic test where relevant
