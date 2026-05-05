# `milestone_diag_4a` slice 2b.28 — Shared call-arity diagnostic migration

Pass 1 review of the uncommitted working tree on branch
`codex/semantic-diagnostics-diag-4a-call-arity-diagnostics`.

## Scope under review

- Mark slice 2b.27 merged in the issue tracker after [sifr-lang/sifr#1699](https://github.com/sifr-lang/sifr/pull/1699) and add the in-progress entry for slice 2b.28.
- Migrate the shared user-defined / stdlib wrong positional argument count path in `lower_function_call_args` from the generic `SIFR-TYPE-0001` bridge to active `SIFR-CALL-0001`.
- Re-key [crates/sifr/tests/e2e/fail/stdlib_wrong_arg_count.sifr](crates/sifr/tests/e2e/fail/stdlib_wrong_arg_count.sifr).
- Add focused HIR unit coverage asserting the exact message and exact `DiagnosticCode` for the shared function-call arity path.

The author's note that no registry/generated-docs change is needed because `SIFR-CALL-0001`'s template was already widened in slice 2b.26 to `{callable} takes {quantifier} {expected_count} argument(s), got {actual_count}` is correct — see verification below.

## Verdict

**Approved — reviewer-satisfied for PR.** Behaviour, fixture re-keying, HIR unit coverage, issue-tracker hygiene, and code/family alignment all line up. No correctness, regression, or alignment blockers were found. Two strictly out-of-scope follow-ups are flagged at the bottom for future slices; neither blocks this PR.

## What I checked

### 1. HIR call-site migration
[crates/sifr_hir/src/lower/method_call_args.rs:69-80](crates/sifr_hir/src/lower/method_call_args.rs:69)

- `lower_function_call_args`'s "too many positional args, no keywords" branch now emits via `ctx.error_with_code(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT, ...)` instead of bare `ctx.error(...)`. Refactor binds `expected_count = ft.params.len()` and `actual_count = positional_args.len()` before the format and uses inline format args, keeping clippy's `uninlined_format_args` happy and matching the slice-2b.26 sum-arity migration style at [crates/sifr_hir/src/lower/expressions.rs:1132-1140](crates/sifr_hir/src/lower/expressions.rs:1132).
- Message text is preserved verbatim — `"{callable_name}() takes at most {expected_count} argument(s), got {actual_count}"`. Behavioural shape (`return None;` after emit) is unchanged, so downstream lowering still short-circuits exactly as before; no risk of cascade diagnostics being introduced or removed.
- The `DiagnosticCode` import is already present at [crates/sifr_hir/src/lower/method_call_args.rs:2](crates/sifr_hir/src/lower/method_call_args.rs:2) (used by the existing `duplicate_argument_error` helper migrated in slice 2b.26). No new imports needed.
- `error_with_code` populates `LoweringError.code = Some(...)` ([crates/sifr_hir/src/lower/mod.rs:237](crates/sifr_hir/src/lower/mod.rs:237)), which surfaces through `compile_errors_to_diagnostics` to the e2e harness's `failure.code` ([crates/sifr/tests/e2e.rs:2561-2567](crates/sifr/tests/e2e.rs:2561)) instead of falling through the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge at [crates/sifr_driver/src/diagnostics.rs:137](crates/sifr_driver/src/diagnostics.rs:137).
- `lower_signature_call_args` is a thin wrapper that calls `lower_function_call_args` ([method_call_args.rs:36-44](crates/sifr_hir/src/lower/method_call_args.rs:36)), so all signature-driven call paths — user-defined defs, stdlib functions like `sqrt`, callable-object `__call__` ([expressions.rs:1719](crates/sifr_hir/src/lower/expressions.rs:1719)), and protocol/class method calls ([expressions.rs:2412, 2429](crates/sifr_hir/src/lower/expressions.rs:2412)) — now emit `SIFR-CALL-0001` for this exact arity violation.

### 2. Registry / generated-docs alignment
[crates/sifr_diagnostics/src/codes.rs:784-799](crates/sifr_diagnostics/src/codes.rs:784), [docs/errors/SIFR-CALL-0001.md](docs/errors/SIFR-CALL-0001.md), [internal_docs/diagnostic_codes.md:97](internal_docs/diagnostic_codes.md:97), [internal_docs/diagnostic_emission_inventory.md:75](internal_docs/diagnostic_emission_inventory.md:75)

- Template: `{callable} takes {quantifier} {expected_count} argument(s), got {actual_count}`. Both messages routed through this code interpolate into a coherent fill: slice 2b.26's `sum() takes exactly 1 argument(s), got 2` and slice 2b.28's `sqrt() takes at most 1 argument(s), got 2` only differ in `{quantifier}` (`exactly` vs `at most`). The widened template adopted in slice 2b.26 was specifically intended to accommodate this slice — confirmed by [reviews/semantic-diagnostic-code-taxonomy-diag-4a-call-shape-diagnostics-review-pass-1.md:56](reviews/semantic-diagnostic-code-taxonomy-diag-4a-call-shape-diagnostics-review-pass-1.md:56). No drift introduced.
- Representative fixture is already `crates/sifr/tests/e2e/fail/stdlib_wrong_arg_count.sifr` (from slice 2b.26), so no fixture-pointer change is required for this slice. The fixture remains live after the re-key (see §4).
- Owner (`sifr_hir::lower`), declared/dedupe arg lists (`callable, quantifier, expected_count, actual_count`), severity, family code, and family description are all unchanged. Generated-docs sync (`docs/errors/SIFR-CALL-0001.md`, `internal_docs/diagnostic_codes.md`) therefore remains in lock-step without re-running `gen-error-docs`. The emission inventory entry at [internal_docs/diagnostic_emission_inventory.md:75](internal_docs/diagnostic_emission_inventory.md:75) (`TypeErrorKind::WrongArgumentCount` → `SIFR-CALL-0001` → `stdlib_wrong_arg_count.sifr`) is now actually true at runtime, where it was aspirational before this slice.

### 3. Fixture re-keying
[crates/sifr/tests/e2e/fail/stdlib_wrong_arg_count.sifr:1](crates/sifr/tests/e2e/fail/stdlib_wrong_arg_count.sifr:1)

- `# expect-error` marker changed from `SIFR-TYPE-0001:` to `SIFR-CALL-0001:` with the message tail (`sqrt() takes at most 1 argument(s), got 2`) unchanged. No body changes.
- Fixture body still drives the same code path: `from sifr.math import sqrt` then `sqrt(1.0, 2.0)`. `sqrt` is registered as a 1-param stdlib function so `lower_function_call_args` enters the `keyword_args.is_empty() && positional_args.len() > ft.params.len()` branch and emits the migrated diagnostic deterministically.
- I verified no other `# expect-error` fixture relies on a `... takes at most ... argument(s), got ...` shape under `SIFR-TYPE-0001` — `grep -rn "argument(s), got" crates/sifr/tests/e2e/` returns only the two SIFR-CALL-0001 fixtures (`stdlib_wrong_arg_count.sifr` and `builtin_sum_wrong_arity.sifr`) plus three unrelated `SIFR-TYPE-0007`/`SIFR-CALL-0005` fixtures with different message shapes. No additional fixture re-keying is required.

### 4. HIR unit coverage
[crates/sifr_hir/src/lower/expressions_tests.rs:320-331](crates/sifr_hir/src/lower/expressions_tests.rs:320)

- `test_function_wrong_arg_count_has_call_code` constructs a single-param user-defined `def takes_one(x: int) -> int` and calls `takes_one(1, 2)` (wrapped in `print(...)` purely so the outer expression type-checks). The inner two-arg call drives the migrated branch in `lower_function_call_args`.
- Asserts both the **exact** message (`"takes_one() takes at most 1 argument(s), got 2"`) and the **exact** `DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT` — matching the brief.
- Test placement, `lower_source(...)` helper, and assertion shape mirror the adjacent slice-2b.26/2b.27 tests (`test_builtin_sum_wrong_arity_has_call_code` at line 246, `test_hash_unhashable_argument_has_proto_code` at line 308), keeping the file coherent.
- Choosing a user-defined function (rather than a stdlib re-test) is the right complement to the existing fixture: the e2e fixture covers the stdlib-imported path, and the HIR unit test covers the user-defined path. Together they pin both call-resolution branches to the same diagnostic.

I re-ran `cargo test -p sifr_hir test_function_wrong_arg_count_has_call_code` — passes. I also re-ran `cargo test -p sifr --test e2e test_e2e_fail` — passes (1 test, 25 filtered).

### 5. Issue-tracker hygiene
[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:62-63](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:62)

- Slice 2b.27 line flipped from `[ ] ... implementation complete and reviewer-satisfied` to `[x] ... merged ... PR: https://github.com/sifr-lang/sifr/pull/1699.` — wording matches the established merged-line template used by 2b.20–2b.26 (same pattern: `merged: ... migration to active <code(s)> with fixture coverage. PR: ...`).
- A new `2b.28 in progress` entry is added on line 63 with the right shape (`shared wrong positional argument count diagnostic migration to active SIFR-CALL-0001 with stdlib fixture coverage. PR: pending.`), consistent with how prior in-progress entries were formatted before being flipped to merged.
- "shared wrong positional argument count" accurately characterises the surface migrated — the same `lower_function_call_args` path serves user-defined, stdlib-imported, callable-object, and class-/protocol-method calls.

### 6. Coherence: is `SIFR-CALL-0001` the right home?

Yes. The CALL family is split as:

- `SIFR-CALL-0001` — wrong positional argument count (template parameterised by `{quantifier}` to cover both `exactly` and `at most`).
- `SIFR-CALL-0002` — unexpected keyword argument.
- `SIFR-CALL-0003` — duplicate argument from positional/keyword overlap.
- `SIFR-CALL-0004` — missing required argument.
- `SIFR-CALL-0005` — not callable / map-style callable arity.

The migrated path emits `"{name}() takes at most {N} argument(s), got {M}"` when there are no keyword args and `M > N`. That is purely a positional-arity violation, so `SIFR-CALL-0001` is the correct bucket. The existing slice-2b.26 sum-arity emission already lives there, so this slice consolidates both ad-hoc spots onto a single shared code, exactly as the family taxonomy intends.

### 7. Validation surface

The author-listed local validation set (`cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir "function_wrong_arg_count"`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`) is the standard slice-2b mix and covers every surface this change touches. Because no registry/generated-docs change is involved, the `gen-error-docs` / `check_diagnostic_docs_sync.py` / `check_diagnostic_schema_sync.py` / `cargo test -p sifr_diagnostics` gates are correctly omitted — they would be no-ops here.

I additionally re-ran:

- `python3 scripts/check_hir_maintainability_guardrails.py` → `HIR maintainability guardrails: PASS`.
- `cargo test -p sifr_hir test_function_wrong_arg_count_has_call_code` → 1 passed.
- `cargo test -p sifr --test e2e test_e2e_fail` → 1 passed (25 filtered).

## Concerns / non-blocking follow-ups

- **(A) `missing_argument_error` still on the bridge.** The companion path at [crates/sifr_hir/src/lower/method_call_args.rs:288-293](crates/sifr_hir/src/lower/method_call_args.rs:288) — `"{callable_name}(): missing argument '{arg}' with no default value"` — still uses bare `ctx.error(...)` and so flows through `SIFR-TYPE-0001`. Its natural target is `SIFR-CALL-0004` (`CALL_MISSING_REQUIRED_ARGUMENT`), whose registry entry already exists at [crates/sifr_diagnostics/src/codes.rs:822-828](crates/sifr_diagnostics/src/codes.rs:822) with template `"{callable} missing required argument {argument}"` and representative fixture `missing_keyword_only_arg.sifr`. The current emitted text doesn't perfectly match that template (`(): missing argument '...' with no default value` vs `missing required argument ...`), so a future slice will need either to reword the emission or widen the template. Out of scope for slice 2b.28 (scope is specifically the shared *too-many*-positional path); flag it as the next CALL-family migration sub-slice.
- **(B) Latent gap: extra positional args when keyword args are present.** In `lower_function_call_args`, the keyword-non-empty branch ([method_call_args.rs:95-113](crates/sifr_hir/src/lower/method_call_args.rs:95)) iterates over `ft.params.iter().enumerate()` and silently ignores any positional argument at index ≥ `ft.params.len()`. So a call like `f(1, 2, kw=3)` against `def f(x: int, *, kw: int)` would not raise the migrated `SIFR-CALL-0001` — the trailing positional is dropped. This is a pre-existing behaviour gap that pre-dates this slice and is not introduced by it; the diagnostic-code migration is still correct for every input that already reached the migrated emit. Worth a separate ticket so a future slice can add a symmetric guard before the param-zip loop and route it through the same `CALL-0001` code.

Neither follow-up alters the verdict.

## Final word

Implementation is correct, scope is minimal and matches the brief, the fixture / HIR unit test / issue-tracker / registry / generated-docs all agree on a single `SIFR-CALL-0001` code+template, and the migration is coherent with the rest of the CALL family established in slice 2b.26. **Approved for PR.**
