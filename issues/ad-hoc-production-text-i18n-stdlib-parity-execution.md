# Ad Hoc Phase Execution: Production Text And Internationalization Stdlib Parity

Phase contract: [ad-hoc-production-text-i18n-stdlib-parity.md](./ad-hoc-production-text-i18n-stdlib-parity.md)

Status: draft

## Scope Split

This ledger tracks:

- `codecs`, `encodings`
- `unicodedata`
- `locale`
- `gettext`

Network/web parity remains in [ad-hoc-production-stdlib-platform-parity-execution.md](./ad-hoc-production-stdlib-platform-parity-execution.md). Concurrency/runtime parity remains in [ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md](./ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md).

## Milestone Checklist

- [ ] `milestone_text_i18n_0`: CPython Inventory, Error Mapping, And Registry Design
- [ ] `milestone_text_i18n_1`: Codecs Registry, Encodings, And Text I/O Integration
- [ ] `milestone_text_i18n_2`: Unicode Data And Normalization
- [ ] `milestone_text_i18n_3`: Locale
- [ ] `milestone_text_i18n_4`: Gettext
- [ ] `milestone_text_i18n_5`: Integration, Documentation, And Production Gate

## Planning Reviews

- Inherited from the original combined stdlib parity planning review:
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-1d.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-2.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-3.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-4.md`
- Final combined review result before split: `PASS`.
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
- No-legacy readiness scan:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-18-no-legacy-readiness.md`
  - Result: `FAIL`; stale dynamic handler lookup wording and misleading M3 implicit-open wording were remediated.
- No-legacy follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-19-no-legacy-readiness.md`
  - Result: `PASS`; no remaining backward-compatibility, legacy-support, deprecated-behavior, shim, bridge-alias, or fallback decisions remained.
- Namespace consistency scan:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-20-sifr-namespace-readiness.md`
  - Result: `PASS`; all three phase docs consistently use canonical `sifr.*` stdlib imports and reject bare CPython stdlib aliases.
- Required follow-up: run a dedicated external review after M0 inventory and before M1 implementation, because this phase is now independently scoped.

## Planning Review Remediation Retained In This Phase

- [x] Add text/i18n CPython source, docs, tests, and native backing scope.
- [x] Add shared text/i18n error mapping requirement.
- [x] Add codec registry mutation policy as an M0 decision.
- [x] Add Unicode data version and generated-table strategy as M0 decisions.
- [x] Add locale process-global synchronization as an M0 decision.
- [x] Add gettext global-installation policy as an M0 decision.
- [x] Require `str.encode(...)`, `bytes.decode(...)`, and `open(..., encoding=...)` to use the same codec registry.
- [x] Require global-state mutation to be synchronized, waived, or host-limited with tests.
- [x] Add explicit provider contract for network/web and concurrency/runtime text consumers.
- [x] Clarify that binary-mode file I/O is prior infrastructure and this phase owns only text-mode `open(..., encoding=..., errors=...)` integration.
- [x] Clarify locale state is process-scoped and threaded/process-pool code must serialize locale-sensitive operations through this phase's locale lock or record host-limited/intentional-diff behavior.
- [x] Name binary file I/O as prior runtime/file-object parity and current `sifr.io` infrastructure, with M0 verification before text-mode `open(..., encoding=..., errors=...)`.
- [x] Make binary file I/O verification a hard prerequisite: failures block `milestone_text_i18n_1` and are fixed in the existing `sifr.io`/runtime file-object surface rather than worked around in text-mode code.
- [x] Define `open(path)`/`open(path, mode="r")` without explicit `encoding=` as permanently `unsupported`/`intentional-diff` from CPython's locale-derived default; M3 documents this as the final intentional difference and does not unblock these forms. Static omissions get compile-time diagnostics; dynamic cases get typed unsupported-default-encoding errors.
- [x] Make explicit text encodings permanently required for text-mode `open(...)`; locale preferred encoding APIs do not make implicit text opens legal.
- [x] Require literal/static `open(...)` modes so the compiler can choose binary versus text handle types; dynamic/nonliteral mode strings get a compile-time diagnostic unless routed through a future typed helper API.
- [x] Extend the explicit-encoding policy to text stream wrappers such as `io.TextIOWrapper`, or record `io.TextIOWrapper` as unsupported with CPython evidence.
- [x] Carve out `io.StringIO` as encoding-free native-string I/O; it must not inherit the codec-backed wrapper encoding requirement.
- [x] Define typed/statically known codec `errors=` handling for `str.encode`, `bytes.decode`, `open`, and text wrappers, with dynamic error-handler names unsupported because synchronized runtime lookup is not adopted in this phase.
- [x] Require `io.StringIO(newline=...)` to either implement CPython-compatible newline semantics or reject unsupported newline parameters with diagnostics.
- [x] Require encode/decode context validation for codec error handlers, including rejecting encode-only handlers such as `xmlcharrefreplace` on decode call sites.
- [x] Add explicit codec error-handler applicability classes: encode-only, bidirectional, and codec-limited bidirectional, preserving `backslashreplace` as valid for both encode and decode.
- [x] Define encode/decode error-handler enforcement through separate typed handler parameters and compile-time diagnostics for invalid static literals; synchronized dynamic handler lookup is not adopted in this phase.
- [x] Document the static-versus-dynamic codec handler diagnostic timing difference as intentional.
- [x] Define incremental encoder/decoder finalization as transitioning to exhausted state, with typed exhausted errors on subsequent encode/decode calls.
- [x] Define incremental strict errors as typed failures with no partial success value, and recoverable non-strict errors as typed success outcomes carrying both produced output and recovery diagnostics.
- [x] Resolve the registry mutation fork to a static registry in this phase, with `codecs.register`/`codecs.unregister` unsupported or intentional-diff.
- [x] Resolve locale mutation to a process-global lock for adopted APIs, with locale names/host behavior recorded as host-limited where needed.
- [x] Resolve `gettext.install` global mutation to unsupported/waived in this phase, with explicit translation objects as the supported path.
- [x] Define incremental encoder/decoder ownership as unique mutable linear state, not hidden shared mutation.
- [x] Add pre-M0 binary file I/O smoke criteria and `sifr.io` owner for prerequisite fixes.
- [x] Add external-review owner and five-working-day fallback rule.
- [x] Define `milestone_text_i18n_1` as the reciprocal unblock point for network/web non-UTF-8 URL/HTTP text behavior and concurrency/runtime subprocess/warning text behavior, with locale-sensitive formatting still additionally blocked on M3.
- [x] Add no-backward-compatibility policy: current-CPython API shape under canonical `sifr.*` imports only, no bare CPython stdlib aliases, no legacy aliases, no deprecated behavior, no implicit locale-default text behavior, and no compatibility shims; only inventory-recorded current adapters with Sifr-safe semantics are allowed.
- [x] Align the phase with the stdlib namespace cleanup: `sifr.*` remains the permanent public stdlib namespace and bare CPython stdlib import attempts get namespace-contract diagnostics.
- [x] Remove stale dynamic error-handler lookup wording; dynamic handler names remain unsupported in this phase.
- [x] Clarify no-encoding text `open(...)` remains permanently unsupported after M3; M3 documents the intentional difference rather than unblocking locale-derived defaults.

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
