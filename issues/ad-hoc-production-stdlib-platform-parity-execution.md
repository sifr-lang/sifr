# Ad Hoc Phase Execution: Production Network And Web Stdlib Parity

Phase contract: [ad-hoc-production-stdlib-platform-parity.md](./ad-hoc-production-stdlib-platform-parity.md)

Status: draft

## Scope Split

The original broad planning scan was split into three implementation phases:

- This ledger tracks network/web stdlib parity: `socket`, `select`, `selectors`, `ssl`, `urllib.*`, `http.*`, `socketserver`.
- [ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md](./ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md) tracks queues, subprocess/process pools/multiprocessing, and runtime ergonomics.
- [ad-hoc-production-text-i18n-stdlib-parity-execution.md](./ad-hoc-production-text-i18n-stdlib-parity-execution.md) tracks text and internationalization.

## Milestone Checklist

- [ ] `milestone_network_web_0`: CPython Inventory And Harness Lock
- [ ] `milestone_network_web_1`: Socket, Select, Selectors, And Async Network Streams
- [ ] `milestone_network_web_2`: TLS And SSL
- [ ] `milestone_network_web_3`: URL Parsing, HTTP Client, HTTP Server, Cookies, And Robots
- [ ] `milestone_network_web_4`: Integration, Documentation, And Production Gate

## Planning Reviews

- Initial Claude planning review covered the original combined phase.
  - Full-file Claude attempts stalled before output and produced no retained review content.
  - Embedded-summary review completed:
    - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-1d.md`
  - Result: `FAIL`; blockers folded into the original phase draft.
- Second Claude planning review:
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-2.md`
  - Result: `FAIL`; remaining conditional gates folded into the original phase draft.
- Final Claude planning reviews:
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-3.md`
  - Result: `FAIL`; one conditional `contextmanager` branch remained.
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-4.md`
  - Result: `PASS`; no blocking gaps remained before the split.
- Split-phase Claude review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-1-constrained.md`
  - Result: `FAIL`; cross-phase dependency and ownership gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-2-constrained.md`
  - Result: `FAIL`; remaining ownership/disposition gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-3-constrained.md`
  - Result: `FAIL`; remaining sequencing/error-surface gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-4-constrained.md`
  - Result: `FAIL`; remaining async-context/file/default-encoding/thread-error gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-5-constrained.md`
  - Result: `FAIL`; remaining contextvars/future-cancellation/open-policy/worker-typing gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-6-constrained.md`
  - Result: `FAIL`; remaining executor map/timeout/cancellation/heterogeneous-future gaps and text-wrapper gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-7-constrained.md`
  - Result: `FAIL`; remaining executor state-machine, `StringIO`, `threading.local`, and codec error-handler gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-8-constrained.md`
  - Result: `FAIL`; remaining Future.cancel, wait partition, executor.map timeout, and text handler gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-9-constrained.md`
  - Result: `FAIL`; remaining executor deadline/cancellation/wait fallback and codec handler classification gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-10-constrained.md`
  - Result: `FAIL`; remaining handler enforcement, partial iteration, FIRST_EXCEPTION trigger, and shutdown pending/running gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-11-constrained.md`
  - Result: `FAIL`; remaining future ownership/lifecycle and shutdown observability gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-12-constrained.md`
  - Result: `FAIL`; remaining `wait()` ownership, cancelled result typing, and incremental codec finalization gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-13-constrained.md`
  - Result: `FAIL`; remaining `gather()` ownership/result typing, `as_completed()` timeout signaling, codec recoverable-error, and `TaskGroup` aggregation gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-14-constrained.md`
  - Result: `FAIL`; remaining network error hierarchy, TLS socket ownership, workload classification, handler model, concurrency gate, text decision, and review-gate gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-15-constrained.md`
  - Result: `FAIL`; remaining TLS wrap failure-state, `signal.pause`, and text-i18n dependency milestone gaps were remediated.
- Final split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-16-constrained.md`
  - Result: `PASS`; no material implementation-blocking gaps remained.
- Final implementation-readiness scan:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-17-final-readiness.md`
  - Result: `PASS`; all three phases were implementation-ready, with one editorial ledger-title mismatch remediated in the concurrency/runtime ledger.
- No-legacy readiness scans:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-18-no-legacy-readiness.md`
  - Result: `FAIL`; text/i18n stale dynamic-handler and implicit-open wording were remediated.
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-19-no-legacy-readiness.md`
  - Result: `PASS`; no remaining backward-compatibility, legacy-support, deprecated-behavior, shim, bridge-alias, or fallback decisions remained.
- Namespace consistency scan:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-20-sifr-namespace-readiness.md`
  - Result: `PASS`; all three phase docs consistently use canonical `sifr.*` stdlib imports and reject bare CPython stdlib aliases.

## Planning Review Remediation Retained In This Phase

- [x] Add `socketserver` source, docs, tests, and milestone scope.
- [x] Add C-backed `_socket` reexport inventory requirements.
- [x] Define async TLS stream boundary and reject public event-loop retry model.
- [x] Define concrete async HTTP deliverables.
- [x] Add milestone dependency graph.
- [x] Add shared network/web error mapping requirement.
- [x] Add M4 closeout acceptance criteria.
- [x] Name Tokio as the backing async runtime for this phase and require concrete feature expansion in M0.
- [x] Make HTTP/HTTPS dependency on M2 `AsyncTlsStream` explicit.
- [x] Make `SSLContext.wrap_socket` sync-only for this phase.
- [x] Mark `socketserver.ThreadingMixIn`, `socketserver.ForkingMixIn`, and `ThreadingHTTPServer` unsupported for this phase.
- [x] Add explicit cross-phase dependency contract for text/i18n and concurrency/runtime consumers.
- [x] Clarify that core `asyncio` scheduler/task helpers are prior async-model infrastructure and this phase owns only network stream compatibility additions.
- [x] Clarify `urllib.parse` byte/ASCII/UTF-8 ownership and block non-UTF-8 codec lookup on the text/i18n phase.
- [x] Mark non-ASCII/non-UTF-8 `urllib.parse` encoding behavior as `blocked-on-text-i18n` until `milestone_text_i18n_1`.
- [x] Define pre-text-i18n user-visible behavior for non-UTF-8 `urllib.parse` encoding arguments: compile-time diagnostic for static values, typed `UnsupportedEncodingError`/`URLError` for dynamic values.
- [x] Add concrete network/TLS/HTTP typed error hierarchy and cross-module nesting requirements.
- [x] Add required workload classification table for socket/select/TLS/HTTP and async-context diagnostics.
- [x] Specify `SSLContext.wrap_socket` consumes the plain socket and `SSLSocket.unwrap()` consumes TLS before returning a plain socket.
- [x] Specify failed TLS wrapping returns a typed `TlsWrapError` carrying recovered-or-closed socket state and nested transport evidence.
- [x] Define the static handler abstraction requirement for `socketserver` and `http.server` instead of dynamic Python inheritance.
- [x] Add external-review owner and five-working-day fallback rule.
- [x] Pin non-UTF-8 URL/HTTP text behavior to text/i18n `milestone_text_i18n_1` completion.
- [x] Add no-backward-compatibility policy: current-CPython API shape under canonical `sifr.*` imports only, no bare CPython stdlib aliases, no legacy aliases, no deprecated behavior, and no compatibility shims; only inventory-recorded current adapters with Sifr-safe semantics are allowed.
- [x] Align the phase with the stdlib namespace cleanup: `sifr.*` remains the permanent public stdlib namespace and bare CPython stdlib import attempts get namespace-contract diagnostics.

## Implementation PRs

- M0: pending.
- M1: pending.
- M2: pending.
- M3: pending.
- M4: pending.

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

## CPython Scan Evidence

Each milestone must record:

- CPython source files scanned.
- CPython docs files scanned.
- CPython tests scanned.
- Public APIs adopted, adapted, waived.
- Unsupported/intentional-diff/host-limited surfaces.
- Sifr e2e pass/fail fixtures added.

## Waiver Index

No waivers recorded yet.

Every waiver must include:

- surface
- terminal state: `intentional-diff`, `unsupported`, or `host-limited`
- rationale
- revisit rule
- CPython evidence
- Sifr regression fixture
