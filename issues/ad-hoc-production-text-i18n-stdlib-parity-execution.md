# Ad Hoc Phase Execution: Production Text, Unicode, Encoding, And I18n Runtime

Phase contract: [ad-hoc-production-text-i18n-stdlib-parity.md](./ad-hoc-production-text-i18n-stdlib-parity.md)

Status: draft

## Scope Split

This ledger tracks:

- `sifr.encoding`
- `sifr.unicode`
- `sifr.io` explicit text I/O
- `sifr.i18n`
- CPython-shaped text/i18n surfaces only as reference material, waiver evidence, or deferred adapters

Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate-execution.md](./ad-hoc-production-network-http-platform-substrate-execution.md). Concurrency/runtime parity remains in [ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md](./ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md).

Execution order: this is the first phase in the split production-stdlib sequence. Concurrency/runtime starts after this phase and consumes its text-dependent M1/M3 outputs; network/HTTP starts third and consumes this phase's M1/M2/M2.5/M3 outputs. Later phases must not invent local text, encoding, Unicode, locale, or fallback-decoder behavior.

## Milestone Checklist

- [ ] `milestone_text_i18n_0`: Product Boundary And Rust Lowering Contract
- [ ] `milestone_text_i18n_1`: Encoding And Explicit Text I/O
- [ ] `milestone_text_i18n_2`: Unicode Core
- [ ] `milestone_text_i18n_2_5`: Unicode Segmentation
- [ ] `milestone_text_i18n_3`: Locale Identifiers And Locale-Sensitive Formatting
- [ ] `milestone_text_i18n_4`: Translation Bundles
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
- Reviewer-driven substrate pivot:
  - Result: accepted; the phase is now defined as production text/Unicode/encoding/i18n runtime substrate, not CPython stdlib parity.
  - Public production API center: `sifr.encoding`, `sifr.unicode`, `sifr.io`, and `sifr.i18n`.
  - CPython-shaped surfaces (`sifr.codecs`, `sifr.encodings`, `sifr.unicodedata`, `sifr.locale`, `sifr.gettext`) are reference/deferred-adapter surfaces unless a later review explicitly accepts them over the native substrate.
- Text/i18n substrate review:
  - `reviews/ad-hoc-production-text-i18n-substrate-review-pass-1.md`
  - Result: `PASS`; two editorial observations were remediated by aligning the public `sifr.unicode` property list with M2 scope and adding per-crate/e2e validation commands to the ledger baseline.
- Phase-order clarification:
  - Result: accepted; this phase is first in the split production-stdlib sequence because both network/HTTP and concurrency/runtime depend on its text/encoding substrate.
- Cross-phase implementation-readiness review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-21-phase-order-readiness.md`
  - Result: `FAIL`; network/runtime dependency matrix, network cancellation/shutdown provider wording, and legacy filename naming-note gaps were remediated across the split phase docs.
- Cross-phase implementation-readiness follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-22-phase-order-readiness.md`
  - Result: `PASS`; pass 21 remediations were verified, with one minor network state-vocabulary inconsistency remediated.
- Final cross-phase implementation-readiness verification:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-23-final-readiness.md`
  - Result: `PASS`; no material blockers, stale labels, or implementation-blocking contradictions remained.
- Rust ecosystem-first clarification:
  - Result: accepted; this phase now requires wrapping mature Rust text/Unicode/i18n crates where suitable, records preferred crate families, defers any required surface that the selected ecosystem stack cannot satisfy, and makes dependency decision records an M0 gate.
- Cross-phase decision-closure review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-24-decision-closure.md`
  - Result: `PASS`; all material product/API/dependency decisions across text/i18n, concurrency/runtime, and network/HTTP were clear enough for implementation.
- Final cross-phase decision delta review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-25-final-delta.md`
  - Result: `PASS`; final clarifications introduced no unmade or contradictory implementation decisions.

## Planning Review Remediation Retained In This Phase

- [x] Add text/i18n CPython source, docs, tests, and native backing scope as reference material rather than product API scope.
- [x] Add shared text/i18n typed error mapping requirement.
- [x] Add static encoding registry and unsupported mutation policy as an M0 decision.
- [x] Add Unicode data version and generated-table strategy as M0 decisions.
- [x] Reject process-global locale mutation as a production API; use typed locale IDs and object-based i18n formatters.
- [x] Add gettext/global-installation policy as unsupported; explicit translation bundles are the supported path.
- [x] Require `str.encode(...)`, `bytes.decode(...)`, and `open(..., encoding=...)` to use the same encoding substrate.
- [x] Require global-state mutation to be rejected, waived, or host-limited with tests rather than synchronized into the recommended model.
- [x] Add explicit provider contract for network/web and concurrency/runtime text consumers.
- [x] Clarify implementation order: text/i18n runs first, and network/runtime text-dependent surfaces consume its M1/M3 gates rather than shipping local encoding or locale fallbacks.
- [x] Clarify that binary-mode file I/O is prior infrastructure and this phase owns only text-mode `open(..., encoding=..., errors=...)` integration.
- [x] Clarify Python-style locale state is not the production model; threaded/process-pool code must use explicit locale values and formatter objects, while host locale discovery remains read-only and host-limited.
- [x] Name binary file I/O as prior runtime/file-object parity and current `sifr.io` infrastructure, with M0 verification before text-mode `open(..., encoding=..., errors=...)`.
- [x] Make binary file I/O verification a hard prerequisite: failures block `milestone_text_i18n_1` and are fixed in the existing `sifr.io`/runtime file-object surface rather than worked around in text-mode code.
- [x] Define `open(path)`/`open(path, mode="r")` without explicit `encoding=` as permanently `unsupported`/`intentional-diff` from CPython's locale-derived default; M3 documents this as the final intentional difference and does not unblock these forms. Static omissions get compile-time diagnostics; dynamic cases get typed unsupported-default-encoding errors.
- [x] Make explicit text encodings permanently required for text-mode `open(...)`; locale preferred encoding APIs do not make implicit text opens legal.
- [x] Require literal/static `open(...)` modes so the compiler can choose binary versus text handle types; dynamic/nonliteral mode strings get a compile-time diagnostic unless routed through a future typed helper API.
- [x] Record Python-shaped text stream wrappers such as `io.TextIOWrapper` as unsupported in this phase with CPython evidence; the production surface is `sifr.io.open_text(...)` and Sifr-native typed text readers/writers.
- [x] Carve out `io.StringIO` as encoding-free native-string I/O; it must not inherit the codec-backed wrapper encoding requirement.
- [x] Define typed/statically known encoding `errors=` handling for `str.encode`, `bytes.decode`, `open`, and text wrappers, with dynamic error-handler names unsupported because synchronized runtime lookup is not adopted in this phase.
- [x] Require `io.StringIO(newline=...)` to either implement CPython-compatible newline semantics or reject unsupported newline parameters with diagnostics.
- [x] Require encode/decode context validation for codec error handlers, including rejecting encode-only handlers such as `xmlcharrefreplace` on decode call sites.
- [x] Add explicit codec error-handler applicability classes: encode-only, bidirectional, and codec-limited bidirectional, preserving `backslashreplace` as valid for both encode and decode.
- [x] Define encode/decode error-handler enforcement through separate typed handler parameters and compile-time diagnostics for invalid static literals; synchronized dynamic handler lookup is not adopted in this phase.
- [x] Document the static-versus-dynamic codec handler diagnostic timing difference as intentional.
- [x] Define incremental encoder/decoder finalization as transitioning to exhausted state, with typed exhausted errors on subsequent encode/decode calls.
- [x] Define incremental strict errors as typed failures with no partial success value, and recoverable non-strict errors as typed success outcomes carrying both produced output and recovery diagnostics.
- [x] Resolve the registry mutation fork to a static registry in this phase, with `codecs.register`/`codecs.unregister` unsupported or intentional-diff.
- [x] Resolve locale mutation by rejecting `setlocale`-style process-global behavior as the production path; use explicit locale values and formatter objects.
- [x] Resolve `gettext.install` global mutation to unsupported/waived in this phase, with explicit translation bundles and translators as the supported path.
- [x] Define incremental encoder/decoder ownership as unique mutable linear state, not hidden shared mutation.
- [x] Add pre-M0 binary file I/O smoke criteria and `sifr.io` owner for prerequisite fixes.
- [x] Add external-review owner and five-working-day fallback rule.
- [x] Define `milestone_text_i18n_1` as the reciprocal unblock point for network/web non-UTF-8 URL/HTTP text behavior and concurrency/runtime subprocess/warning text behavior, with locale-sensitive formatting still additionally blocked on M3.
- [x] Add no-backward-compatibility policy: no bare CPython stdlib aliases, no legacy aliases, no deprecated behavior, no implicit locale-default text behavior, and no compatibility shims; only separately reviewed adapters over Sifr-native APIs are allowed.
- [x] Align the phase with the stdlib namespace cleanup: `sifr.*` remains the permanent public stdlib namespace and bare CPython stdlib import attempts get namespace-contract diagnostics.
- [x] Remove stale dynamic error-handler lookup wording; dynamic handler names remain unsupported in this phase.
- [x] Clarify no-encoding text `open(...)` remains permanently unsupported after M3; M3 documents the intentional difference rather than unblocking locale-derived defaults.
- [x] Add Rust `String`/`str`-compatible text invariants: normal Sifr strings are valid Unicode, arbitrary bytes stay bytes, and invalid Unicode recovery cannot be hidden inside ordinary strings.
- [x] Replace CPython parity definition with support tiers: production substrate, production API, import/compatibility backend, host-limited, and rejected/deferred.
- [x] Add demand-tiered encoding scope: Tier 0 required encodings, Tier 1 web/file compatibility, Tier 2 CJK deferred to a separate issue with dependency/data-size/workload review, and Tier 3 rejected/deferred CPython-only/pseudo-codec/module-zoo parity.
- [x] Add Unicode segmentation as a dedicated M2.5 milestone for grapheme and word boundaries.
- [x] Reframe gettext as translation-bundle/backend import support, not the strategic global i18n API.

## Implementation PRs

- M0: pending.
- M1: pending.
- M2: pending.
- M2.5: pending.
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
cargo test -p sifr_stdlib
cargo test -p sifr -- stdlib
scripts/run_e2e_pass.sh
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
- Standards and Rust crate sources reviewed.
- Production APIs adopted.
- Python-shaped surfaces adapted, waived, rejected, or deferred as adapters.
- Unsupported/intentional-diff/host-limited/deferred-adapter surfaces.
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
