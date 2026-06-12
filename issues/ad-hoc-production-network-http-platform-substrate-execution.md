# Ad Hoc Phase Execution: Production Network and HTTP Platform Substrate

Phase contract: [ad-hoc-production-network-http-platform-substrate.md](./ad-hoc-production-network-http-platform-substrate.md)

Status: in progress; M0, M1, and M2 merged; M3 URL, Header, And Cookie Primitives PR is merge-validated

## Scope Split

The original broad planning scan was split into three implementation phases:

- This ledger tracks the production network/TLS/URL/HTTP substrate: `sifr.net`, `sifr.tls`, `sifr.url`, accepted `sifr.http` protocol/runtime primitives, typed errors, async suspension points, resource limits, observability hooks, and internal loopback harnesses.
- [ad-hoc-production-concurrency-runtime-platform-substrate-execution.md](./ad-hoc-production-concurrency-runtime-platform-substrate-execution.md) tracks concurrency/process/runtime substrate.
- [ad-hoc-production-text-i18n-platform-substrate-execution.md](./ad-hoc-production-text-i18n-platform-substrate-execution.md) tracks text/Unicode/encoding/i18n runtime substrate.

Execution order: this is the third phase in the split production-stdlib sequence. Text/i18n runs first, concurrency/runtime runs second, and network/HTTP consumes both provider phases. Network/HTTP implementation must not start early or close text-dependent/runtime-dependent surfaces without the relevant provider milestones recorded as complete.

CPython-shaped public networking/web modules are no longer this phase's objective or a future adapter track. `sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib.request`, `sifr.urllib.parse`, `sifr.http.client`, `sifr.http.server`, and `sifr.socketserver` are evidence only and must resolve to `rejected`, `unsupported-with-diagnostic`, `internal-only`, or `test-only-harness`.

## Milestone Checklist

- [x] `milestone_network_http_0`: Product Boundary And Architecture
- [x] `milestone_network_http_1`: Async Network Runtime
- [x] `milestone_network_http_2`: TLS Runtime
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
- Reviewer TLS full-duplex disposition:
  - Source: user-provided follow-up review after the serving-scale, TCP split, and TCP half-close fixes.
  - Result: accepted; the phase now explicitly accepts owned `TlsStream.split()` into `TlsReadHalf` / `TlsWriteHalf`, defines TLS `close_notify()` as the write-side close operation instead of TCP-style `shutdown_write()`, requires write-after-close-notify typed errors, and adds M0 definition gates plus M2 implementation and loopback coverage for TLS full-duplex behavior.
- Claude Fable TLS full-duplex review pass 1:
  - `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-1.md`
  - Result: `CONDITIONAL PASS`; TLS full-duplex was substantively correct, but TLS stream contract ownership still said M2 could define the contract, and the TLS close/flush/split feasibility details needed M0 clarifications.
  - Remediations: M0 now owns TLS stream and TLS full-duplex contract definition before M2 starts; M2 implements that contract. The TLS contract now records close-notify/TCP half-close disposition as an M0 decision, requires successful `close_notify()` to flush accepted plaintext and the close alert or return typed partial-progress evidence, records TLS version coverage in loopback fixtures, and documents the lock-backed/synchronized implementation expectation through accepted Tokio/tokio-rustls utilities rather than bespoke TLS session sharing.
- Claude Fable TLS full-duplex review pass 2:
  - `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-2.md`
  - Result: `PASS`; the pass-1 TLS contract ownership blocker and all four recommended M0-gate edits were verified as remediated, no new blocking contradictions or decision-by-discovery holes were introduced, and the phase was judged implementation-ready for M0.
  - Follow-up polish: the phase now explicitly records deterministic `flush` behavior after successful `close_notify()` with no pending application data, and M2 DoD echoes the TLS-version fixture requirement.
- Claude Fable TLS full-duplex review pass 3:
  - `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-3.md`
  - Result: `PASS`; the post-pass-2 polish was verified as non-contradictory, the TLS M0 definition to M2 implementation ownership chain remained intact, and no new implementation-readiness blockers were found.
  - Follow-up polish: M2 DoD now also echoes repeated `close_notify` and empty-flush-after-close-notify fixture coverage.
- Claude Fable broad implementation-readiness review pass 4:
  - `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-4.md`
  - Result: `CONDITIONAL PASS`; HTTP/2 abuse limits were incorrectly worded as M4-defined instead of M0-defined/M4-implemented, and the terminal-state list omitted text/i18n M2, M2.5, and M3 labels.
  - Remediations: M0 now owns HTTP/2 abuse limits before M4 starts, the terminal-state vocabulary includes all text/i18n provider labels used by M0 DoD, process-runtime provider state is explicit, `UrlError` is named, stale `sifr.asyncio` baseline wording was removed, request-smuggling/header-normalization ownership is M0-defined, Phase 41 names this substrate dependency, and the shared platform contract status is updated to approved shared baseline.
- Claude Fable broad implementation-readiness review pass 5:
  - `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-5.md`
  - Result: `PASS`; pass-4 blockers were verified as fixed, all seven polish edits were checked against the repo, and no remaining implementation-readiness blockers were found.
  - Follow-up polish: Phase 41 dependency wording now includes `sifr.url`, the shared platform contract status cites review passes 3a-3d, and the historical dependency checklist includes process runtime M4.
- Claude Fable final follow-up verification pass 6:
  - `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-6.md`
  - Result: `PASS`; the pass-5 follow-ups were verified as non-contradictory, the main substrate doc remained unchanged from the pass-5-verified state, and the phase remained implementation-ready for M0.
- Final reviewer cleanup pass:
  - Source: user-provided final review attached in Codex.
  - Result: `PASS with small cleanup edits`; the phase was judged ready for M0 execution planning, with cleanup requested for byte-buffer placeholder naming, Hyper-Util proof artifacts, Ring 5 absence proof, TLS `close()` disposition, HTTP/2 priority/extension behavior, and the UDP production-consumer burden.
  - Remediations: API examples now use `ByteBuffer` as an explicit M0 placeholder instead of lowercase `bytes`; `hyper_util_necessity.md` is required if Hyper-Util is enabled; M0 generated release snapshots must prove Ring 5 dev/test/demo crates are absent from production feature combinations; M0 must define `TlsStream.close()` / `TlsWriteHalf.close()` disposition; HTTP/2 priority and extension-frame behavior is an explicit M0 decision; and UDP acceptance now requires both a named production consumer and a reason TCP/TLS/HTTP loopback fixtures are insufficient.
- Claude Fable final cleanup verification pass 7:
  - `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-7.md`
  - Result: `PASS`; all six final reviewer cleanup edits were verified as coherent, the ledger matched the diff, and no new implementation-readiness blockers were found.
- Claude Opus M0 implementation review pass 1:
  - `reviews/ad-hoc-production-network-http-m0-opus-review-pass-1.md`
  - Result: `FAIL`; M0 artifacts existed but several M0-owned decisions were still deferred to M1/M2/M4.
  - Remediations: M0 now resolves `ByteBuffer` to built-in `bytes`; defines TLS `close()`/`close_notify` disposition; defines HTTP/2 limits, priority/extension handling, header normalization, request-smuggling rules, `sifr.http` type table including trailers, body stream contract, body/header size limits, URL authority rules, and redaction rules; records public `SO_REUSEPORT` deferral; adds `network_http_dependency_audit.md`; expands golden/e2e unsupported import coverage; and expands the Decision Index.
- Claude Opus M0 implementation review pass 2:
  - `reviews/ad-hoc-production-network-http-m0-opus-review-pass-2.md`
  - Result: `PASS`; all pass-1 blocking findings B1-B11 and non-blocking follow-ups were verified as remediated. Reviewer stated the M0 PR is safe to open and merge, and M1 can safely start after validation evidence is recorded.
- Claude Opus M1 implementation review pass 1:
  - `reviews/ad-hoc-production-network-http-m1-opus-review-pass-1.md`
  - Result: `FAIL`; reviewer blocked on global `+ Send` validation, infallible affine `TcpStream.split()`, and cancellation fixture coverage.
  - Remediations: reran create-pr validation after the global sendability change, made `TcpStream.split()` infallible end-to-end with public `TcpStream.close(own self)`, added in-flight `accept()` cancellation coverage, removed unused M1 dependency emission, and documented M1 runtime behavior.
- Claude Opus M1 implementation review pass 2:
  - `reviews/ad-hoc-production-network-http-m1-opus-review-pass-2.md`
  - Result: `PASS`; reviewer accepted the M1 PR for opening after remediation, with the full merge gate required before closure.
- Claude Opus M2 implementation review pass 1:
  - `reviews/ad-hoc-production-network-http-m2-opus-review-pass-1.md`
  - Result: `FAIL`; reviewer blocked on a non-deterministic public TLS fixture certificate that expired after 24 hours.
  - Remediations: replaced the embedded localhost fixture certificate with a long-lived `CA:FALSE` localhost/127.0.0.1 SAN certificate, lifted repeated `close_notify()` and post-close-notify `flush()` into the public e2e fixture, tightened runtime mTLS/invalid-root assertions to expected failing sides, and made the e2e generated Tokio dependency explicitly include `net`.
- Claude Opus M2 implementation review pass 3:
  - `reviews/ad-hoc-production-network-http-m2-opus-review-pass-3.md`
  - Result: `PASS`; reviewer verified the fixture certificate time-bomb was fixed, the public close-notify fixture coverage was expanded, the runtime rejection tests were tightened, no new blockers were introduced, and the implementation is ready for PR opening after the required validation gates.
- Claude Opus M2 final branch-tip review pass 4:
  - `reviews/ad-hoc-production-network-http-m2-opus-review-pass-4.md`
  - Result: `PASS`; reviewer verified the final branch tip after the validation-contract follow-up commit, found no blocking issues, accepted the full merge-gate evidence, and stated PR #2496 is acceptable to merge now.
- Claude Opus M3 implementation review pass 2:
  - `reviews/ad-hoc-production-network-http-m3-opus-review-pass-2.md`
  - Result: `FAIL`; reviewer blocked on percent-encoded non-ASCII URL host bytes bypassing the IDNA guard and stale path-normalization traceability language.
  - Remediations: URL authority guard now rejects non-ASCII percent-decoded host bytes before `url` crate parsing; fixtures cover `%C3%A9.example` rejection, `%61.example` acceptance, IPv4 parsing, IPv6 building, and `%2F` path preservation; M3 traceability now records WHATWG dot-segment behavior instead of claiming raw path preservation; generated helpers enforce inventory hard caps for URL/query/header primitives; header canonicalization and embedded `=` cookie values are fixture-covered.
- Claude Opus M3 implementation review pass 3:
  - `reviews/ad-hoc-production-network-http-m3-opus-review-pass-3.md`
  - Result: `PASS`; reviewer verified the IDNA bypass and path-normalization traceability blockers are fully remediated, found no new implementation blockers, and stated M3 is acceptable to open as a PR after the standard create-pr validation gates pass.
- M0 implementation merge ledger:
  - PR: https://github.com/sifr-lang/sifr/pull/2494
  - Merge commit: `c426d01e26257c5b72e3ecd50e6884c86292a14b`
  - Scope: added the M0 substrate inventory, dependency audit and snapshots, CPython evidence matrix, workload database, platform golden fixtures, unsupported network/HTTP import diagnostics, negative e2e coverage, per-milestone traceability files, and the serving-scale follow-up issue.
  - Merge-gate validation: `scripts/run_all_tests.sh` passed on rerun for head `30b098eedeec52af8d6234d5af990b86a611ec67`; first merge-gate run had one transient `check-project-004-project-graph` performance p95 outlier, targeted representative performance rerun passed, and the full merge-gate rerun passed with wall-time/batching advisories only.
- M1 implementation merge ledger:
  - PR: https://github.com/sifr-lang/sifr/pull/2495
  - Merge commit: `ce5a411f4284404a1a374f77c0176351771e7cb9`
  - Scope: added public `sifr.net`, private `_sifr.net` intrinsics, optional `sifr_runtime/net`, network codegen helpers, deterministic TCP loopback/split/half-close/cancellation fixtures, UDP deferral coverage, and M1 Opus review artifacts.
  - Merge-gate validation: `scripts/run_all_tests.sh` passed for head `6c88bbd5f56035b488c4ad85a18061ab2b804fd2`; report `target/validation_lane_reports/merge.latest.json`; advisories were warm wall-time budget exceeded and high group skew only.
- M2 implementation merge ledger:
  - PR: https://github.com/sifr-lang/sifr/pull/2496
  - Merge commit: `742ea9f33dcac821d5abb644156d97dd2d7876cc`
  - Scope: added public `sifr.tls`, private `_sifr.tls` intrinsics, optional `sifr_runtime/tls`, TLS codegen helpers, Rustls/Tokio-Rustls runtime integration, deterministic TLS loopback/split/close-notify/config-error fixtures, dependency snapshots, and M2 Opus review artifacts.
  - Merge-gate validation: `scripts/run_all_tests.sh` passed for head `28d845c86b94cceb84bf3e29872498f18fdd7980`; report `target/validation_lane_reports/merge.latest.json`; advisory was high e2e group skew only.
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
- [x] Add a concurrency/runtime dependency matrix for features blocked on task/cancellation M1, sync/backpressure M2, offload M3, process runtime M4, shutdown/diagnostics M5, and IPC/process-worker M6.
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
- [x] Add explicit TLS full-duplex disposition with owned split halves and `close_notify` write-side close semantics.
- [x] Resolve Claude Fable pass 1 blocker: M0 owns TLS stream/full-duplex contract definition and M2 implements it.
- [x] Record Claude Fable pass 2 `PASS` confirming the TLS contract ownership fix and no remaining implementation-readiness blockers.
- [x] Add Fable pass 2 polish for post-`close_notify` flush behavior and TLS-version fixture coverage in M2 DoD.
- [x] Record Claude Fable pass 3 `PASS` confirming the final TLS polish did not introduce gaps.
- [x] Resolve Claude Fable pass 4 blockers: M0 owns HTTP/2 abuse limits and terminal-state vocabulary includes all text/i18n provider labels used by M0.
- [x] Apply Claude Fable pass 4 polish: explicit process-runtime state, named `UrlError`, current baseline wording, request-smuggling ownership, Phase 41 backlink, and platform-contract status refresh.
- [x] Record Claude Fable pass 5 `PASS` confirming no remaining implementation-readiness blockers.
- [x] Record Claude Fable pass 6 `PASS` confirming the pass-5 follow-up edits did not introduce gaps.
- [x] Apply final reviewer cleanup edits for byte-buffer placeholder naming, Hyper-Util proof, Ring 5 absence proof, TLS close disposition, HTTP/2 priority/extensions, and UDP acceptance burden.
- [x] Record Claude Fable pass 7 `PASS` confirming the final reviewer cleanup edits did not introduce gaps.

## Implementation PRs

- M0: https://github.com/sifr-lang/sifr/pull/2494 merged at `c426d01e26257c5b72e3ecd50e6884c86292a14b`.
- M1: https://github.com/sifr-lang/sifr/pull/2495 merged at `ce5a411f4284404a1a374f77c0176351771e7cb9`.
- M2: https://github.com/sifr-lang/sifr/pull/2496 merged at `742ea9f33dcac821d5abb644156d97dd2d7876cc`.
- M3: https://github.com/sifr-lang/sifr/pull/2497 ready for merge after local merge-gate validation.
- M4: pending.
- M5: pending.

## Validation Evidence

Record local validation for each milestone before opening its PR.

M0 validation:

| Command | Result | Notes |
| --- | --- | --- |
| `python3 -m json.tool` on network/platform JSON artifacts | PASS | Parsed `network_http_substrate_inventory.json`, `network_http_dependency_snapshots.json`, `platform_contract.json`, and platform golden manifest. |
| `cargo fmt --check` | PASS | Clean after M0 remediation. |
| `cargo test -p sifr_stdlib network_http -- --nocapture` | PASS | Covers reserved network/web import mappings and Ring 5 dependency snapshot absence check. |
| `cargo test -p sifr --test e2e e2e_fail -- --nocapture` | PASS | 479 fail fixtures completed; existing CFG panic messages appear inside negative harness but test exits successfully. |
| `SIFR_PLATFORM_CLOSED_MILESTONES=milestone_network_http_0 scripts/run_platform_golden.sh` | PASS | 11 pass, 1 expected skip. |
| `python3 scripts/check_file_size_guardrails.py` | PASS | 2293 files, limit 900 lines. |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | Lowering maintainability guardrails passed. |
| `cargo clippy --workspace -- -D warnings` | PASS | Finished dev profile in 2m16s. |
| `scripts/run_all_tests.sh --profile create-pr` | PASS | Report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded, no test failure. |
| `scripts/run_all_tests.sh` | PASS | Report `target/validation_lane_reports/merge.latest.json`; first run failed on one transient performance p95 outlier, targeted representative performance rerun and full merge-gate rerun passed. Advisories: warm wall-time budget exceeded and high group skew. |

M1 validation:

| Command | Result | Notes |
| --- | --- | --- |
| `cargo build -p sifr --bin sifr` | PASS | Rebuilt CLI after `sifr.net` stdlib/runtime/codegen changes. |
| `cargo check -p sifr_runtime --features net` | PASS | Runtime crate builds with optional `net` feature enabled. |
| `cargo test -p sifr_runtime --features net --lib net -- --nocapture` | PASS | Runtime crate builds and test harness completes; `sifr_runtime::net` currently has no unit tests, so behavior is covered by generated Sifr fixtures. |
| `cargo test -p sifr_stdlib --test concurrency_runtime_dependency_snapshots -- --nocapture` | PASS | Verifies existing concurrency/Tokio dependency snapshot remains unchanged without `sifr.net`. |
| `cargo test -p sifr_stdlib --test network_http_dependency_snapshots -- --nocapture` | PASS | Covers M0 Ring 5 dependency absence and the M1 `sifr.net` generated dependency snapshot with `sifr_runtime/net`, Tokio `net`, tracing, and no unused `bytes`/`socket2`/`tokio-util` emission. |
| `cargo test -p sifr_stdlib --test text_i18n_dependency_snapshots -- --nocapture` | PASS | Verifies moved text/i18n dependency snapshot coverage remains intact after feature-registry edits. |
| `target/debug/sifr check crates/sifr/tests/e2e/pass/network_http_m1_tcp_loopback_split.sifr` | PASS | Public `sifr.net` TCP loopback fixture type-checks. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m1_tcp_loopback_split.sifr` | PASS | Deterministic loopback connect/listen/accept/split/half-close fixture runs without external network dependency. |
| `target/debug/sifr check crates/sifr/tests/e2e/pass/network_http_m1_tcp_errors.sifr` | PASS | Deterministic typed error fixture type-checks. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m1_tcp_errors.sifr` | PASS | Invalid timeout and invalid backlog return typed `NetError` results without external network dependency. |
| `target/debug/sifr check crates/sifr/tests/e2e/pass/network_http_m1_tcp_cancel_accept.sifr` | PASS | Provider cancellation fixture type-checks. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m1_tcp_cancel_accept.sifr` | PASS | Cancels an in-flight listener `accept()` through a scoped task handle and observes `Cancelled` evidence. |
| `SIFR_E2E_FIXTURE_MANIFEST=<tmp manifest> SIFR_E2E_DISABLE_CACHE=1 cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` | PASS | Selected fixtures `network_http_m1_tcp_cancel_accept`, `network_http_m1_tcp_errors`, and `network_http_m1_tcp_loopback_split`; 3 pass tests completed. |
| `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` | PASS | Full fail corpus completed; validates `network_http_m1_udp_deferred.sifr` expected `SIFR-NAME-0004` and existing unsupported import diagnostics. Existing fail-harness internal CFG panic messages are caught by the negative harness and test exits successfully. |
| `cargo fmt --check` | PASS | Clean after M1 edits. |
| `python3 scripts/check_file_size_guardrails.py` | PASS | 2302 files, limit 900 lines. |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | Lowering maintainability guardrails passed. |
| `scripts/run_all_tests.sh --profile create-pr` | PASS | Report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. |
| `scripts/run_e2e_pass.sh` | PASS | Merge-manifest e2e pass suite completed 138 pass tests, 0 failed. |
| `scripts/run_all_tests.sh` | PASS | Report `target/validation_lane_reports/merge.latest.json`; all merge-lane steps passed. Advisories: warm wall-time budget exceeded and high group skew. |

M1 broad pass-suite note:

- An exploratory `cargo test -p sifr --test e2e e2e_pass -- network_http_m1 --nocapture` invocation ran the full pass corpus rather than filtering only M1 fixtures. It completed 636 pass fixtures and exposed pre-existing non-network failures: IO context-manager generated mutability in `cpython_io_subset`/`stdlib_io_consolidated` and `open_*` fixtures, plus `bytes_conversion_errors` expecting `latin-1` encode/decode rejection. The M1-specific fixtures pass through the selected manifest above.
- The clean `scripts/run_all_tests.sh --profile create-pr` rerun passed after clearing detached stale validation jobs from earlier interrupted runs. The only advisory was warm wall-time budget exceeded.
- The full `scripts/run_all_tests.sh` merge gate passed before M1 PR prep. Advisory-only output reported warm wall-time budget exceeded and high group skew.

M2 focused validation:

| Command | Result | Notes |
| --- | --- | --- |
| `cargo check -p sifr_runtime --features tls` | PASS | Verifies optional runtime TLS feature compilation with Rustls, Tokio-Rustls, PEM parsing, and platform verifier dependencies. |
| `cargo check -p sifr_stdlib -p sifr_codegen` | PASS | Verifies stdlib source embedding, intrinsic signatures, generated dependency features, and TLS lowerer/preamble compilation. |
| `cargo build -p sifr --bin sifr` | PASS | Rebuilt the CLI after `sifr.tls` wrapper edits so direct fixture runs used the current embedded stdlib. |
| `cargo test -p sifr_runtime --features tls --lib tls -- --nocapture` | PASS | Covers TLS loopback split/ALPN/close-notify, mTLS missing-client rejection, and invalid-root certificate rejection. |
| `cargo test -p sifr_stdlib --test network_http_dependency_snapshots -- --nocapture` | PASS | Verifies M0/M1/M2 generated dependency snapshots, TLS feature gating, and absence of `rcgen`, `webpki-roots`, and `x509-parser` from production dependency output. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m2_tls_loopback_split.sifr` | PASS | Public loopback fixture covers explicit roots, real TCP/TLS handshakes, SNI, ALPN, protocol version evidence, owned split halves, `flush`, `close_notify`, and write-after-close-notify typed error. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m2_tls_config_errors.sifr` | PASS | Public malformed-PEM fixture maps TLS config failure into typed `CertificateError`. |
| `SIFR_E2E_FIXTURE_MANIFEST=/tmp/sifr_m2_tls_fixtures_$$.json cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` | PASS | Selected fixtures `network_http_m2_tls_config_errors` and `network_http_m2_tls_loopback_split`; 2 pass tests completed. |
| `cargo fmt --check` | PASS | Clean after M2 edits. |
| `CARGO_TARGET_DIR=target/codex-clippy cargo clippy --workspace -- -D warnings` | PASS | Workspace clippy baseline passed; isolated target directory avoided stale default-target Cargo locks. |
| `python3 scripts/check_file_size_guardrails.py` | PASS | 2309 files checked; touched hand-maintained files remain under the 900-line cap. |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | Lowering maintainability guardrails passed after TLS intrinsic/codegen additions. |
| `scripts/run_all_tests.sh --profile create-pr` | PASS | Clean PTY run passed after clearing stale interrupted validation jobs; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. |
| `scripts/run_all_tests.sh` | PASS | Full merge-gate validation passed for head `d4e2feb1feef13c7fd037d14301531915ed75b2a`; report `target/validation_lane_reports/merge.latest.json`; wall time 799.89s, hardening failures 0, advisory only: high e2e group skew. |

M2 broad pass-suite note:

- An accidental full `cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` run launched after a temp-manifest wrapper mistake. It completed the M2 TLS fixture groups successfully, then failed in unrelated pre-existing pass fixtures: IO generated-build groups (`cpython_io_subset`, `stdlib_io_consolidated`, `open_*`) and `bytes_conversion_errors`. The targeted M2 manifest above is the authoritative M2 e2e signal for this candidate.
- Earlier interrupted `create-pr` validation attempts left stale Cargo child processes holding the default target lock. After terminating those stale validation jobs, a clean PTY `scripts/run_all_tests.sh --profile create-pr` run passed.
- Full merge-gate validation initially exposed stale contract assumptions around the e2e Tokio `net` feature and validation helper paths under `CARGO_TARGET_DIR`; the follow-up commit corrected those validation contracts, and the final merge-gate rerun passed.

M3 focused validation:

| Command | Result | Notes |
| --- | --- | --- |
| `cargo check -p sifr_stdlib -p sifr_codegen` | PASS | Verifies M3 stdlib intrinsic signatures, generated dependency features, and URL/HTTP lowerer compilation. |
| `cargo build -p sifr --bin sifr` | PASS | Rebuilt the CLI after adding public `sifr.url`, public `sifr.http`, generated helper preambles, and dependency feature wiring. |
| `target/debug/sifr check crates/sifr/tests/e2e/pass/network_http_m3_url_query_percent.sifr` | PASS | Public URL/query/percent fixture type-checks. |
| `target/debug/sifr check crates/sifr/tests/e2e/pass/network_http_m3_header_cookie.sifr` | PASS | Public header/cookie fixture type-checks. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m3_url_query_percent.sifr` | PASS | Fresh generated build after reviewer remediation; covers URL parse/build, IPv4 parsing, IPv6 building, already-punycode host, percent-encoded ASCII host acceptance, literal and percent-encoded non-ASCII host blocked states, `%2F` path preservation, invalid port, percent helpers, path normalization, and query parse/build. |
| `target/debug/sifr run crates/sifr/tests/e2e/pass/network_http_m3_header_cookie.sifr` | PASS | Fresh generated build after reviewer remediation; covers header name/value validation, lowercase canonicalization, OWS trim, duplicate order preservation, obs-fold rejection, cookie header parse/build, and embedded `=` cookie values. |
| `cargo test -p sifr network_http_dependency_contract_tests -- --nocapture` | PASS | Verifies locked URL/header/cookie dependency specs and generated-Rust dependency inference for `url`, `percent-encoding`, `http`, and `cookie`. |
| `SIFR_E2E_FIXTURE_MANIFEST=<M3 fixtures> SIFR_E2E_CACHE_DIR=target/sifr_e2e_cache/m3-focused SIFR_E2E_DISABLE_CACHE=0 cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` | PASS | Selected batch e2e run for `network_http_m3_header_cookie` and `network_http_m3_url_query_percent`; 2 passed, 0 failed, cache hits 0/2 after the IPv6 fixture change. |
| `cargo test -p sifr_stdlib --test network_http_dependency_snapshots -- --nocapture` | PASS | Verifies M0-M3 generated dependency snapshots, exact URL/header/cookie crate specs, and Ring 5 absence from M3 production dependencies. |
| `cargo fmt --check` | PASS | Rust formatting check. |
| `cargo clippy --workspace -- -D warnings` | PASS | Workspace clippy gate passed after Opus pass-3 remediation. |
| `scripts/run_e2e_pass.sh` | PASS | Full e2e pass suite completed 138 pass fixtures with 0 failures; report signature `4ede7c71d86f381c`. |
| `scripts/run_all_tests.sh --profile create-pr` | PASS | Authoritative create-pr validation passed; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. |
| `scripts/run_all_tests.sh` | PASS | Full merge-gate validation passed; report `target/validation_lane_reports/merge.latest.json`; advisory: high e2e group skew only. |
| `python3 scripts/check_file_size_guardrails.py` | PASS | File-size guardrail passed with 2319 files under the 900-line limit. |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | HIR maintainability guardrails passed. |

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

- [x] `verification/stdlib/network_http_substrate_inventory.md`
- [x] `verification/stdlib/network_http_substrate_inventory.json`
- [x] `verification/stdlib/network_http_cpython_evidence_matrix.md`
- [x] `verification/stdlib/network_http_dependency_audit.md`
- [x] one traceability document per milestone domain under `verification/stdlib/`

Opening the M0 implementation PR is blocked until the artifact locations and schemas are present in that PR.

## Review Ownership

- Phase owner: runtime/networking implementation owner.
- Designated compiler/runtime reviewer for M0: Claude Opus via `.cursor/skills/talk-to-claude-opus`; human compiler/runtime reviewer request remains required on the GitHub PR before merge.
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

M0 implementation decisions recorded:

| Surface | Terminal state | Rationale | Revisit rule | Evidence |
| --- | --- | --- | --- | --- |
| `sifr.net.UdpSocket` | `deferred-to-phase-X` | No named near-term production consumer was recorded with a reason TCP/TLS/HTTP loopback fixtures are insufficient. | A future issue must name the production consumer and fixture gap before any public datagram API is added. | `verification/stdlib/network_http_substrate_inventory.md` |
| `SO_REUSEPORT` public API | `deferred-to-serving-scale-follow-up` | Serving scale is explicitly outside this substrate phase; `reuse_addr` must not imply reuse-port. | `issues/ad-hoc-network-http-serving-scale-follow-up.md` must close before any public reuse-port listener option or constructor ships. | `verification/platform/supported_host_matrix.md` |
| internal readiness primitives | `internal-only` | Manual selectors/raw readiness are implementation details behind async streams. | Requires separate low-level readiness architecture issue. | `verification/stdlib/network_http_substrate_inventory.md` |
| internal HTTP transport harness | `test-only-harness` | Loopback client/server helpers validate substrate and are not product client/server APIs. | Phase 41 and the future HTTP client phase own product APIs. | `verification/stdlib/network_http_substrate_inventory.md` |
| `sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib.*`, `sifr.http.client`, `sifr.http.server`, `sifr.socketserver` | `unsupported-with-diagnostic` or `rejected` | CPython-shaped network/web APIs conflict with the Sifr-native substrate boundary. | Future APIs must be Sifr-native and owned by `sifr.net`, `sifr.tls`, `sifr.url`, `sifr.http`, Phase 41, or the future HTTP client phase. | `crates/sifr_stdlib/src/lib.rs`, M0 e2e fail fixtures |
| multi-core serving throughput | `deferred-to-serving-scale-follow-up` | This phase provides production-correct serving substrate for one current-thread runtime worker per process. | `issues/ad-hoc-network-http-serving-scale-follow-up.md` owns the scale strategy. | `verification/platform/supported_host_matrix.md` |
| HTTP/3 / QUIC | `deferred-to-phase-X` | QUIC transport strategy needs a separate runtime/security phase. | Open a transport phase after HTTP/2 substrate closes. | `verification/stdlib/network_http_substrate_inventory.md` |
| WebSocket and CONNECT public APIs | `deferred-to-phase-X` | Upgrade products need separate backpressure and security decisions. | Future product phase must define Sifr-native APIs and fixtures. | `verification/stdlib/network_http_substrate_inventory.md` |
| Multipart/form parsing | `deferred-to-phase-41` | Product-level body parsing and bomb limits belong to Phase 41 or the HTTP client phase. | Revisit with accepted framework/client requirements. | `verification/stdlib/network_http_substrate_inventory.md` |
| Content-Encoding compression | `deferred-to-phase-X` | Compression and decompression bomb policy are outside substrate. | Future compression issue must own limits and hooks. | `verification/stdlib/network_http_substrate_inventory.md` |
| `metrics` facade | `deferred-to-phase-X` | Optional metrics schema needs M5 approval before production dependency activation. | Add only after metric names, labels, redaction, and deterministic tests are approved. | `verification/stdlib/network_http_dependency_audit.md` |
| `hickory-resolver` | `deferred-to-phase-X` | TCP connect/address resolution uses `tokio::net::lookup_host`; custom resolver policy is outside substrate. | Future resolver issue must define record APIs and host behavior. | `verification/stdlib/network_http_substrate_inventory.md` |
| `x509-parser` | `deferred-to-phase-X` | Public certificate display parsing is outside M2; TLS errors carry typed verification evidence. | Future certificate-inspection issue must define text/i18n display behavior. | `verification/stdlib/network_http_dependency_audit.md` |
| Ring 5 dev/test/demo dependencies | `test-only-harness` | `tokio-test`, `proptest`, `rcgen`, and `tracing-subscriber` must not appear in production dependency combinations. | M5 must re-prove resolver-backed all-feature snapshots after implementation. | `verification/stdlib/network_http_dependency_snapshots.json`, `crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs` |

Every `deferred-to-phase-X`, `rejected`, `host-limited`, `internal-only`, or `unsupported-with-diagnostic` decision must include:

- surface
- terminal state
- rationale
- revisit rule
- CPython evidence where relevant
- Sifr regression fixture or diagnostic test where relevant
