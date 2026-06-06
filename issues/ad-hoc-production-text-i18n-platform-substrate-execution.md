# Ad Hoc Phase Execution: Production Text, Unicode, Encoding, And I18n Runtime

Phase contract: [ad-hoc-production-text-i18n-platform-substrate.md](./ad-hoc-production-text-i18n-platform-substrate.md)

Status: draft

## Scope Split

This ledger tracks:

- `sifr.encoding`
- `sifr.unicode`
- `sifr.io` explicit text I/O
- `sifr.i18n`
- CPython-shaped text/i18n surfaces only as reference material, waiver evidence, or `deferred-to-adapter-phase`

Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate-execution.md](./ad-hoc-production-network-http-platform-substrate-execution.md). Concurrency/runtime platform substrate remains in [ad-hoc-production-concurrency-runtime-platform-substrate-execution.md](./ad-hoc-production-concurrency-runtime-platform-substrate-execution.md).

Execution order: this is the first phase in the split production-stdlib sequence. Concurrency/runtime starts after this phase and consumes its text-dependent M1/M3 outputs; network/HTTP starts third and consumes this phase's M1/M2/M2.5/M3 outputs. Later phases must not invent local text, encoding, Unicode, locale, or fallback-decoder behavior.

## Milestone Checklist

- [x] `milestone_text_i18n_0`: Product Boundary And Rust Lowering Contract
- [x] `milestone_text_i18n_1`: Encoding And Explicit Text I/O
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
  - CPython-shaped surfaces (`sifr.codecs`, `sifr.encodings`, `sifr.unicodedata`, `sifr.locale`, `sifr.gettext`) are reference or `deferred-to-adapter-phase` surfaces unless a later review explicitly accepts them over the native substrate.
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
- Updated text/i18n product-boundary review:
  - Source: `/Users/yaseralnajjar/.codex/attachments/d3be2da1-89ee-48a2-a601-d1d450934f21/pasted-text.txt`
  - Result: mostly already remediated by the substrate rewrite; remaining useful refinements were added: explicit No-Toy-Module Gate, `SIFR_CPYTHON_CHECKOUT` source-tree variable, generated Unicode table marker/regeneration requirements, and constrained safe `.mo` plural-expression parsing.
- M0 implementation review pass 1:
  - `reviews/ad-hoc-production-text-i18n-m0-implementation-review-pass-1.md`
  - Result: `FAIL`; blockers covered missing `encodings`/`locale`/dotted CPython import fixtures, missing itemized unsupported Python-shaped classifications, incomplete dependency decision fields, missing exact Tier 0/Tier 1 alias table, missing reserved `open(...)` diagnostic wording, missing no-global-state policy evidence, and missing platform-contract review PASS.
  - Remediation: added the missing e2e fail fixtures, dotted reserved-root diagnostic matching, itemized unsupported-surface rows in inventory, exact alias table, reserved diagnostic codes/messages, detailed dependency decision records, and shared no-global-state policy.
- M0 implementation review pass 2:
  - `reviews/ad-hoc-production-text-i18n-m0-implementation-review-pass-2.md`
  - Result: `PASS`; pass 1 blockers were verified as remediated. Non-blocking observations were recorded for future golden stub timing, diagnostic substring coupling, deferred-state naming, JSON first-party `.mo` parser shape, and review-artifact cross-linking.

## Planning Review Remediation Retained In This Phase

- [x] Add text/i18n CPython source, docs, tests, and native backing scope as reference material rather than product API scope.
- [x] Add shared text/i18n typed error mapping requirement.
- [x] Add static encoding registry and unsupported mutation policy as an M0 decision.
- [x] Add Unicode data version and generated-table strategy as M0 decisions.
- [x] Reject process-global locale mutation as a production API; use typed locale IDs and object-based i18n formatters.
- [x] Add gettext/global-installation policy as unsupported; explicit translation bundles are the supported path.
- [x] Require `str.encode(...)`, `bytes.decode(...)`, and `open(..., encoding=...)` to use the same encoding substrate.
- [x] Require global-state mutation to be classified as `rejected`, `waived-with-rationale`, or `host-limited` with tests rather than synchronized into the recommended model.
- [x] Add explicit provider contract for network/web and concurrency/runtime text consumers.
- [x] Clarify implementation order: text/i18n runs first, and network/runtime text-dependent surfaces consume its M1/M3 gates rather than shipping local encoding or locale fallbacks.
- [x] Clarify that binary-mode file I/O is prior infrastructure and this phase owns only text-mode `open(..., encoding=..., errors=...)` integration.
- [x] Clarify Python-style locale state is not the production model; threaded/process-pool code must use explicit locale values and formatter objects, while host locale discovery remains read-only and host-limited.
- [x] Name binary file I/O as prior runtime/file-object parity and current `sifr.io` infrastructure, with M0 verification before text-mode `open(..., encoding=..., errors=...)`.
- [x] Make binary file I/O verification a hard prerequisite: failures block `milestone_text_i18n_1` and are fixed in the existing `sifr.io`/runtime file-object surface rather than worked around in text-mode code.
- [x] Define `open(path)`/`open(path, mode="r")` without explicit `encoding=` as permanently `unsupported-with-diagnostic` from CPython's locale-derived default; M3 documents this as the final intentional difference and does not unblock these forms. Static omissions get compile-time diagnostics; dynamic cases get typed unsupported-default-encoding errors.
- [x] Make explicit text encodings permanently required for text-mode `open(...)`; locale preferred encoding APIs do not make implicit text opens legal.
- [x] Require literal/static `open(...)` modes so the compiler can choose binary versus text handle types; dynamic/nonliteral mode strings get a compile-time diagnostic unless routed through a future typed helper API.
- [x] Record Python-shaped text stream wrappers such as `io.TextIOWrapper` as `unsupported-with-diagnostic` in this phase with CPython evidence; the production surface is `sifr.io.open_text(...)` and Sifr-native typed text readers/writers.
- [x] Carve out `io.StringIO` as encoding-free native-string I/O; it must not inherit the codec-backed wrapper encoding requirement.
- [x] Define typed/statically known encoding `errors=` handling for `str.encode`, `bytes.decode`, `open`, and text wrappers, with dynamic error-handler names unsupported because synchronized runtime lookup is not accepted in this phase.
- [x] Require `io.StringIO(newline=...)` to either implement CPython-compatible newline semantics or reject unsupported newline parameters with diagnostics.
- [x] Require encode/decode context validation for codec error handlers, including rejecting encode-only handlers such as `xmlcharrefreplace` on decode call sites.
- [x] Add explicit codec error-handler applicability classes: encode-only, bidirectional, and codec-limited bidirectional, preserving `backslashreplace` as valid for both encode and decode.
- [x] Define encode/decode error-handler enforcement through separate typed handler parameters and compile-time diagnostics for invalid static literals; synchronized dynamic handler lookup is not accepted in this phase.
- [x] Document the static-versus-dynamic codec handler diagnostic timing difference as intentional.
- [x] Define incremental encoder/decoder finalization as transitioning to exhausted state, with typed exhausted errors on subsequent encode/decode calls.
- [x] Define incremental strict errors as typed failures with no partial success value, and recoverable non-strict errors as typed success outcomes carrying both produced output and recovery diagnostics.
- [x] Resolve the registry mutation fork to a static registry in this phase, with `codecs.register`/`codecs.unregister` `unsupported-with-diagnostic`.
- [x] Resolve locale mutation by rejecting `setlocale`-style process-global behavior as the production path; use explicit locale values and formatter objects.
- [x] Resolve `gettext.install` global mutation to `unsupported-with-diagnostic` or `waived-with-rationale` in this phase, with explicit translation bundles and translators as the supported path.
- [x] Define incremental encoder/decoder ownership as unique mutable linear state, not hidden shared mutation.
- [x] Add pre-M0 binary file I/O smoke criteria and `sifr.io` owner for prerequisite fixes.
- [x] Add external-review owner and five-working-day fallback rule.
- [x] Define `milestone_text_i18n_1` as the reciprocal unblock point for network/web non-UTF-8 URL/HTTP text behavior and concurrency/runtime subprocess/warning text behavior, with locale-sensitive formatting still additionally blocked on M3.
- [x] Add no-backward-compatibility policy: no bare CPython stdlib aliases, no legacy aliases, no deprecated behavior, no implicit locale-default text behavior, and no compatibility shims; only separately reviewed adapters over Sifr-native APIs are allowed.
- [x] Align the phase with the stdlib namespace cleanup: `sifr.*` remains the permanent public stdlib namespace and bare CPython stdlib import attempts get namespace-contract diagnostics.
- [x] Remove stale dynamic error-handler lookup wording; dynamic handler names remain unsupported in this phase.
- [x] Clarify no-encoding text `open(...)` remains permanently unsupported after M3; M3 documents the intentional difference rather than unblocking locale-derived defaults.
- [x] Add Rust `String`/`str`-compatible text invariants: normal Sifr strings are valid Unicode, arbitrary bytes stay bytes, and invalid Unicode recovery cannot be hidden inside ordinary strings.
- [x] Replace CPython parity definition with shared support states: production substrate, production API, import/compatibility backend, host-limited, `unsupported-with-diagnostic`, `rejected`, and `deferred-to-phase-X`.
- [x] Add No-Toy-Module Gate so public text/i18n modules cannot ship merely because CPython has them, a compatibility demo needs them, or a partial module is easy.
- [x] Add demand-tiered encoding scope: Tier 0 required encodings, Tier 1 web/file compatibility, Tier 2 CJK deferred to a separate issue with dependency/data-size/workload review, and Tier 3 rejected/deferred CPython-only/pseudo-codec/module-zoo parity.
- [x] Add Unicode segmentation as a dedicated M2.5 milestone for grapheme and word boundaries.
- [x] Reframe gettext as translation-bundle/backend import support, not the strategic global i18n API.
- [x] Require `.mo` plural expressions to use a constrained safe plural-expression parser rather than a general expression engine.
- [x] Make generated Unicode tables require a generated-file marker and checked-in regeneration command.
- [x] Use `SIFR_CPYTHON_CHECKOUT` as the portable CPython source-tree setting while recording the local planning checkout path.

## Implementation PRs

- M0: https://github.com/sifr-lang/sifr/pull/2297
- M1: https://github.com/sifr-lang/sifr/pull/2298
- M2: https://github.com/sifr-lang/sifr/pull/2299
- M2.5: pending.
- M3: pending.
- M4: pending.
- M5: pending.

## Implementation Reviews

- M1 pass 1: `reviews/ad-hoc-production-text-i18n-m1-implementation-review-pass-1.md`; result `FAIL`, blockers B1-B7 remediated.
- M1 pass 2: `reviews/ad-hoc-production-text-i18n-m1-implementation-review-pass-2.md`; result `PASS` with closure preconditions C1/C2 and non-blocking N2 followed up.
- M1 pass 3: `reviews/ad-hoc-production-text-i18n-m1-implementation-review-pass-3.md`; result `PASS`, no blockers and no re-review required.
- M2 pass 1: `reviews/ad-hoc-production-text-i18n-m2-implementation-review-pass-1.md`; result `PASS`, no blockers and no re-review required.
- M2 pass 2: `reviews/ad-hoc-production-text-i18n-m2-implementation-review-pass-2.md`; result `PASS` after runtime Unicode feature-gating remediation, no blockers and no re-review required.

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

M0 focused validation on branch `text-i18n-m0-platform-contract`:

- `cargo run -q -p sifr -- run demos/binary_files/main.sifr` passed.
- `cargo run -q -p sifr -- run demos/bytes_file_io/main.sifr` passed.
- `cargo test -p sifr_stdlib bare_stdlib_tail -- --nocapture` passed.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` passed for 372 fail fixtures; observed two existing fail-corpus internal-compiler-error panic messages printed by the harness.
- `scripts/run_platform_golden.sh` passed with 2 pass / 3 skipped blocked entries.
- `cargo fmt --check` passed.
- `python3 scripts/check_hir_maintainability_guardrails.py` passed.
- `python3 scripts/check_file_size_guardrails.py` passed.

M0 post-review-remediation focused validation:

- `cargo test -p sifr_stdlib bare_stdlib_tail -- --nocapture` passed.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` passed for 375 fail fixtures; observed the same two existing fail-corpus internal-compiler-error panic messages printed by the harness.
- `python3 -m json.tool` passed for `verification/platform/platform_contract.json`, `verification/platform/golden/manifest.json`, and `verification/stdlib/text_i18n_substrate_inventory.json`.
- `scripts/run_platform_golden.sh` passed with 2 pass / 3 skipped blocked entries.
- `cargo fmt --check` passed.
- `python3 scripts/check_hir_maintainability_guardrails.py` passed.
- `python3 scripts/check_file_size_guardrails.py` passed.
- `cargo clippy --workspace -- -D warnings` passed.
- `cargo test -p sifr_stdlib` passed.
- `cargo test -p sifr -- stdlib` passed.
- `scripts/run_e2e_pass.sh --profile create-pr` passed with 67/67 pass fixtures.
- `scripts/run_all_tests.sh --profile create-pr` passed; report `target/validation_lane_reports/create-pr.latest.json`, wall time 167.66s, 67/67 e2e pass fixtures, non-blocking warm wall-time/cache advisories.
- `scripts/run_all_tests.sh` passed; report `target/validation_lane_reports/merge.latest.json`, wall time 689.72s, 73/73 e2e pass fixtures, non-blocking group-skew advisory.

M1 focused validation on branch `text-i18n-m1-encoding-io`:

- `cargo check -p sifr_runtime` passed.
- `cargo test -p sifr_stdlib intrinsic -- --nocapture` passed.
- `cargo check -p sifr_codegen` passed.
- `cargo check -p sifr_lowering` passed.
- `cargo test -p sifr_lowering bytes -- --nocapture` passed.
- `cargo test -p sifr_stdlib features -- --nocapture` passed.
- `cargo test -p sifr_codegen registry_core -- --nocapture` passed.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr` passed.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` passed for 375 fail fixtures; observed the same two existing fail-corpus internal-compiler-error panic messages printed by the harness.
- `cargo fmt --check` passed.
- `python3 scripts/check_file_size_guardrails.py` passed.
- `python3 scripts/check_hir_maintainability_guardrails.py` passed.
- `scripts/run_all_tests.sh --profile create-pr` passed after runtime Unicode feature-gating remediation; report `target/validation_lane_reports/create-pr.latest.json`, wall time 256.41s, 67/67 e2e pass fixtures, non-blocking warm wall-time/cache advisories.
- `cargo test -p sifr_stdlib` passed.
- `cargo test -p sifr -- stdlib` passed.
- `cargo clippy --workspace -- -D warnings` passed.
- `scripts/run_e2e_pass.sh --profile create-pr` passed with 67/67 pass fixtures.
- `scripts/run_all_tests.sh --profile create-pr` passed; report `target/validation_lane_reports/create-pr.latest.json`, wall time 184.25s, 67/67 e2e pass fixtures, non-blocking warm wall-time/cache advisories.

M1 post-review-remediation focused validation:

- `cargo test -p sifr_diagnostics codes -- --nocapture` passed.
- `cargo check -p sifr_codegen` passed.
- `cargo check -p sifr_lowering` passed.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr` passed.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` passed for 381 fail fixtures; observed the same two existing fail-corpus internal-compiler-error panic messages printed by the harness.
- `scripts/run_e2e_pass.sh --profile create-pr` passed with 67/67 pass fixtures.
- `cargo fmt --check` passed.
- `python3 scripts/check_file_size_guardrails.py` passed.
- `python3 scripts/check_hir_maintainability_guardrails.py` passed.
- `cargo clippy --workspace -- -D warnings` passed.
- `cargo test -p sifr_stdlib` passed.
- `cargo test -p sifr -- stdlib` passed.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/open_readline.sifr` passed after fixing optional list-index narrowing in the updated explicit-I/O fixture.
- `cargo test -p sifr --test e2e test_emit_pass_fixtures_do_not_include_unwrap_or_expect -- --nocapture` passed.
- `cargo test -p sifr_codegen builtin_open_text_roots_text_handle_support -- --nocapture` passed after rooting compiler-special text-open stdlib dependencies.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/open_read.sifr` passed.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/open_write.sifr` passed.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_io_consolidated.sifr` passed.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_logging_consolidated.sifr` passed.
- `scripts/run_e2e_pass.sh --profile merge` passed with 73/73 pass fixtures.
- `scripts/run_all_tests.sh --profile create-pr` passed; report `target/validation_lane_reports/create-pr.latest.json`, wall time 183.44s, 67/67 e2e pass fixtures, non-blocking warm wall-time/cache advisories.
- `scripts/run_all_tests.sh` passed; report `target/validation_lane_reports/merge.latest.json`, wall time 589.16s, 73/73 e2e pass fixtures, hardening variants 34/34 with 0 failures, non-blocking warm-cache/group-skew advisories.

M2 focused validation on branch `text-i18n-m2-unicode-core`:

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/text_i18n_unicode_core.sifr` passed.
- `cargo test -p sifr_runtime --features unicode unicode -- --nocapture` passed.
- `cargo test -p sifr_stdlib features -- --nocapture` passed.
- `cargo test -p sifr_codegen unicode -- --nocapture` passed.
- `cargo fmt --check` passed.
- `python3 scripts/check_file_size_guardrails.py` passed.
- `python3 scripts/check_hir_maintainability_guardrails.py` passed.
- `scripts/run_all_tests.sh --profile create-pr` passed; report `target/validation_lane_reports/create-pr.latest.json`, wall time 242.95s, 67/67 e2e pass fixtures, non-blocking warm wall-time advisory.
- `scripts/run_all_tests.sh` initially failed in `performance_budget_checks` for `build-project-001-additional-modules` peak RSS (`390545408` bytes versus `342556672` threshold); root cause was unconditional compilation of generated Unicode data through `sifr_runtime`.
- Post-remediation targeted performance check passed after feature-gating `sifr_runtime` Unicode support: `python3 verification/performance/run_benchmarks.py --case build-project-001-additional-modules --json-out target/performance/m2-unicode-runtime-feature-gate.budget.json && python3 verification/performance/check_budgets.py --results target/performance/m2-unicode-runtime-feature-gate.budget.json --allow-subset`; peak RSS `313999360` bytes.
- `scripts/run_all_tests.sh` passed after remediation; report `target/validation_lane_reports/merge.latest.json`, wall time 605.19s, 73/73 e2e pass fixtures, hardening variants 34/34 with 0 failures, non-blocking group-skew advisory.

## CPython Scan Evidence

Each milestone must record:

- CPython source files scanned.
- CPython docs files scanned.
- CPython tests scanned.
- Standards and Rust crate sources reviewed.
- Production APIs classified with shared terminal states and stability levels.
- Python-shaped surfaces classified as `adapted-for-sifr-api`, `waived-with-rationale`, `rejected`, or `deferred-to-adapter-phase`.
- `unsupported-with-diagnostic`, `waived-with-rationale`, `host-limited`, and `deferred-to-adapter-phase` surfaces.
- Sifr e2e pass/fail fixtures added.

M1 scan evidence:

- CPython source/docs/tests scanned: `Lib/codecs.py`, `Lib/encodings/*.py`, `Lib/encodings/aliases.py`, `Doc/library/codecs.rst`, `Lib/test/test_codecs.py`, `Lib/test/test_capi/test_codecs.py`, `Modules/_codecsmodule.c`, and `Modules/cjkcodecs/*` from `SIFR_CPYTHON_CHECKOUT=/Users/yaseralnajjar/work/sifr/cpython`.
- Standards and Rust crate sources reviewed: WHATWG-compatible label behavior through `encoding_rs 0.8.35` crate metadata and docs; Rust `String`/`str` valid UTF-8 invariants; M0 Tier 0/Tier 1 alias table in `verification/stdlib/text_i18n_substrate_inventory.md`.
- Production APIs classified as `production-public` / `stable-public-api`: `sifr.encoding.Encoding`, `DecodeError`, `EncodeError`, typed handlers, `DecodeOutcome`, `EncodeOutcome`, `Decoder`, `Encoder`, `sifr.io.open_text`, and compiler-special `open(..., encoding=..., errors=...)` over the same substrate.
- Python-shaped surfaces classified as `unsupported-with-diagnostic` or `deferred-to-adapter-phase`: `io.TextIOWrapper`, public `codecs` registry mutation, dynamic codec/error-handler registration, public `encodings.*` module parity, Tier 2 CJK codecs, text-to-text codecs, and bytes-to-bytes pseudo-codecs.
- Sifr fixtures added: `crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr`, `crates/sifr/tests/e2e/fail/text_i18n_textiowrapper_unsupported.sifr`, `crates/sifr/tests/e2e/fail/text_i18n_open_without_encoding.sifr`, `crates/sifr/tests/e2e/fail/text_i18n_open_dynamic_mode.sifr`, `crates/sifr/tests/e2e/fail/text_i18n_dynamic_errors_handler.sifr`, `crates/sifr/tests/e2e/fail/text_i18n_open_dynamic_errors_handler.sifr`, `crates/sifr/tests/e2e/fail/text_i18n_decode_encode_only_handler.sifr`, and `crates/sifr/tests/e2e/fail/text_i18n_codecs_register_unsupported.sifr`. The prior UTF-8-only negative fixture was removed because `str.encode("latin-1")` is supported by M1.

M2 scan evidence:

- CPython source/docs/tests scanned: `Doc/library/unicodedata.rst`, `Lib/test/test_unicodedata.py`, and `Modules/unicodedata.c` from `SIFR_CPYTHON_CHECKOUT=/Users/yaseralnajjar/work/sifr/cpython`.
- Standards and Rust crate sources reviewed: Unicode 17.0.0 UCD files `UnicodeData.txt`, `EastAsianWidth.txt`, and `CaseFolding.txt` through `scripts/generate_unicode_tables.py`; `unicode-normalization 0.1.25` source exposing `UNICODE_VERSION = (17, 0, 0)`; `unicode_names2 3.1.0` source/docs for Unicode 17.0 names and lookup.
- Production APIs classified as `production-public` / `stable-public-api`: `sifr.unicode.data_version`, `normalize`, `is_normalized`, `name`, `lookup`, `category`, `bidirectional`, `combining`, `east_asian_width`, `mirrored`, `decomposition`, `decimal`, `digit`, `numeric_value`, and `case_fold`.
- Python-shaped surfaces classified as `deferred-to-adapter-phase`: `sifr.unicodedata` and bare `unicodedata`; the existing `crates/sifr/tests/e2e/fail/bare_cpython_unicodedata_import.sifr` fixture continues to enforce the namespace boundary.
- Sifr fixtures added: `crates/sifr/tests/e2e/pass/text_i18n_unicode_core.sifr`.

## Waiver Index

No waivers recorded yet.

Every waiver must include:

- surface
- terminal state: shared platform terminal state, usually `unsupported-with-diagnostic`, `waived-with-rationale`, or `host-limited`
- rationale
- revisit rule
- CPython evidence
- Sifr regression fixture
