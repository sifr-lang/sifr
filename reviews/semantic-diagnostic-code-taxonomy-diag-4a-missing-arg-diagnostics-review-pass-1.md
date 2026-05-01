# `milestone_diag_4a` slice 2b.29 — Shared missing-required-argument diagnostic migration

Pass 1 review of the uncommitted working tree on branch
`codex/semantic-diagnostics-diag-4a-missing-arg-diagnostics`.

## Scope under review

- Mark slice 2b.28 merged in the issue tracker after [sifr-lang/sifr#1700](https://github.com/sifr-lang/sifr/pull/1700) and add the in-progress entry for slice 2b.29.
- Migrate the shared `missing_argument_error` helper in `lower_function_call_args` / `lower_vararg_function_call_args` from the generic `SIFR-TYPE-0001` bridge to active `SIFR-CALL-0004`.
- Reword the emitted text to align with the registry template, and tighten the registry template/representative-fixture to match the live diagnostic.
- Add a new representative fixture [crates/sifr/tests/e2e/fail/missing_required_argument.sifr](crates/sifr/tests/e2e/fail/missing_required_argument.sifr).
- Add focused HIR unit coverage asserting the exact message and exact `DiagnosticCode` for the shared user-defined-call missing-required-argument path.
- Refresh the registry, generated docs (`docs/errors/SIFR-CALL-0004.md`), `internal_docs/diagnostic_codes.md` and `internal_docs/diagnostic_emission_inventory.md` to point at the new fixture/template.

This slice is the natural follow-up to slice 2b.28, which explicitly flagged `missing_argument_error` as the next CALL-family migration sub-slice ([reviews/semantic-diagnostic-code-taxonomy-diag-4a-call-arity-diagnostics-review-pass-1.md:85](reviews/semantic-diagnostic-code-taxonomy-diag-4a-call-arity-diagnostics-review-pass-1.md:85)).

## Verdict

**Approved — reviewer-satisfied for PR.** Implementation is correct, scope is minimal, fixture/HIR test/registry/generated docs all line up on a single `SIFR-CALL-0004` code+template, and the migration consolidates four shared call-site emissions onto one structured code in one move. No correctness, regression, or alignment blockers were found. Two strictly out-of-scope follow-ups (`unexpected_keyword_error` still on the bridge inside `method_call_args.rs`, and two ad-hoc `"missing required argument"` emissions in `expressions.rs` / `builtin_calls.rs`) are flagged at the bottom for future slices; neither blocks this PR.

## What I checked

### 1. HIR call-site migration
[crates/sifr_hir/src/lower/method_call_args.rs:288-294](crates/sifr_hir/src/lower/method_call_args.rs:288)

- `missing_argument_error` now emits via `ctx.error_with_code(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT, ...)` instead of bare `ctx.error(...)`. Behavioural shape (`return None;` after emit) is unchanged, so downstream lowering still short-circuits exactly as before — no risk of cascade diagnostics being introduced or removed.
- Every call site of the helper benefits from the migration with no per-site change required:
  - [method_call_args.rs:87](crates/sifr_hir/src/lower/method_call_args.rs:87) — no-keyword positional fill branch.
  - [method_call_args.rs:112](crates/sifr_hir/src/lower/method_call_args.rs:112) — keyword-mixed branch.
  - [method_call_args.rs:157](crates/sifr_hir/src/lower/method_call_args.rs:157) — vararg path, params before `*args`.
  - [method_call_args.rs:194](crates/sifr_hir/src/lower/method_call_args.rs:194) — vararg path, kw-only-style params after `*args`.
  All four sites now route through `SIFR-CALL-0004`. This consolidation matches the pattern slice 2b.26 used for `duplicate_argument_error` ([method_call_args.rs:280-286](crates/sifr_hir/src/lower/method_call_args.rs:280)).
- Message text is reworded from `"{callable_name}(): missing argument '{arg}' with no default value"` to `"{callable_name}() missing required argument '{arg}'"`. The new wording (a) drops the colon-after-`()` quirk that none of the sibling CALL diagnostics use, (b) replaces "missing argument … with no default value" with the simpler "missing required argument …" — semantically identical for the path that emits it (a parameter is missing AND has no default ⇔ it's required), and (c) matches sibling CALL templates in style (`'{argument}'` quoting, no internal punctuation).
- The `DiagnosticCode` import is already present at [method_call_args.rs:2](crates/sifr_hir/src/lower/method_call_args.rs:2) (used by the existing migrated `duplicate_argument_error` and slice-2b.28's `lower_function_call_args` site). No new imports needed.
- `error_with_code` populates `LoweringError.code = Some(...)` ([crates/sifr_hir/src/lower/mod.rs:237-244](crates/sifr_hir/src/lower/mod.rs:237)) which surfaces through `compile_errors_to_diagnostics` to the e2e harness's `failure.code` ([crates/sifr/tests/e2e.rs:2561-2567](crates/sifr/tests/e2e.rs:2561)) instead of falling through the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge.
- Because `lower_signature_call_args` is a thin wrapper over `lower_function_call_args` ([method_call_args.rs:36-44](crates/sifr_hir/src/lower/method_call_args.rs:36)), this single migration covers user-defined defs, stdlib functions registered with `FunctionType` signatures, callable objects' `__call__`, protocol/class method calls, and vararg paths in one go — exactly the "shared" surface the slice title claims.

### 2. Registry / generated-docs alignment
[crates/sifr_diagnostics/src/codes.rs:822-832](crates/sifr_diagnostics/src/codes.rs:822), [docs/errors/SIFR-CALL-0004.md](docs/errors/SIFR-CALL-0004.md), [internal_docs/diagnostic_codes.md:100](internal_docs/diagnostic_codes.md:100), [internal_docs/diagnostic_emission_inventory.md:323](internal_docs/diagnostic_emission_inventory.md:323)

- Template tightened from `"{callable} missing required argument {argument}"` to `"{callable} missing required argument '{argument}'"` — the `'{argument}'` quoting now matches sibling CALL templates (`'{keyword}'` in `SIFR-CALL-0002`, `'{argument}'` in `SIFR-CALL-0003`). The runtime emission `display() missing required argument 'verbose'` substitutes cleanly into this shape (`{callable}` → `display()`, `{argument}` → `verbose`).
- Representative fixture pointer flipped from `crates/sifr/tests/e2e/fail/missing_keyword_only_arg.sifr` to `crates/sifr/tests/e2e/fail/missing_required_argument.sifr`. The old pointer was stale: `missing_keyword_only_arg.sifr` actually carries `# expect-error: SIFR-OWN-0003: cannot return borrowed parameter 'name': borrowed parameters cannot escape …` ([crates/sifr/tests/e2e/fail/missing_keyword_only_arg.sifr:1](crates/sifr/tests/e2e/fail/missing_keyword_only_arg.sifr:1)) and was correctly re-keyed to `SIFR-OWN-0003` in slice 2b.15 ([reviews/semantic-diagnostic-code-taxonomy-diag-4a-ownership-diagnostics-review-pass-1.md:29](reviews/semantic-diagnostic-code-taxonomy-diag-4a-ownership-diagnostics-review-pass-1.md:29)). The CALL-0004 registry entry was the only place still naming the old fixture as a representative; this slice closes that drift.
- Owner (`sifr_hir::lower`), declared/dedupe arg lists (`callable, argument`), severity, family code, and family description are all unchanged.
- I re-ran `cargo run -q -p sifr_diagnostics --bin gen-error-docs --check` (via `python3 scripts/check_diagnostic_docs_sync.py`) → `pass`, and `python3 scripts/check_diagnostic_schema_sync.py` → `pass`. The generated `docs/errors/SIFR-CALL-0004.md` rows match the registry exactly:
  - `Message template`: `{callable} missing required argument '{argument}'`
  - `Representative fixture`: `crates/sifr/tests/e2e/fail/missing_required_argument.sifr`
  No drift between Rust source, generated docs, and `internal_docs/diagnostic_codes.md`.
- `internal_docs/diagnostic_emission_inventory.md` updates:
  - The `SIFR-CALL-0004` row at line 323 now points at the new fixture, matching the registry.
  - The `method_call_args.rs` summary row at line 27 (`13 emit sites`, families `CALL`, `TYPE`) is unchanged. That's still accurate: the file still has TYPE-bridge raw `ctx.error(...)` emissions in `lower_keyword_args` ([method_call_args.rs:233, 240](crates/sifr_hir/src/lower/method_call_args.rs:233)), `unexpected_keyword_error` ([method_call_args.rs:296](crates/sifr_hir/src/lower/method_call_args.rs:296)), and the `validate_*` helpers ([method_call_args.rs:474-554](crates/sifr_hir/src/lower/method_call_args.rs:474)). No row update is warranted here.

### 3. Fixture creation
[crates/sifr/tests/e2e/fail/missing_required_argument.sifr](crates/sifr/tests/e2e/fail/missing_required_argument.sifr)

- New fixture body:
  - `def display(name: str, *, verbose: bool) -> str:` defines a positional-only-callable (no `self`) signature with two required params, one of which is keyword-only after `*`.
  - The body returns string literals (`"verbose"` / `"quiet"`) and never references `name`, so the borrow-escape analysis that flagged `missing_keyword_only_arg.sifr` as `SIFR-OWN-0003` does not fire — the only diagnostic raised is the migrated missing-required-argument one. This is the right minimal fixture: it isolates the diagnostic under test without bleed-over from ownership analysis.
  - `display("Alice")` calls the function with one positional arg, missing the kw-only `verbose`. At the type-system layer `FunctionType.params` is a flat `Vec<(String, Type, ParamConvention)>` ([crates/sifr_type_system/src/types.rs:147](crates/sifr_type_system/src/types.rs:147)) — no kw-only marker — so the call enters `lower_function_call_args` with `keyword_args.is_empty() && positional_args.len() == 1 < ft.params.len() == 2`, hits the `default_arg_expr(defaults, 1)` branch (no default), and falls into `missing_argument_error("display", "verbose", ctx)` deterministically.
- `# expect-error: SIFR-CALL-0004: display() missing required argument 'verbose'` matches the harness contract at [crates/sifr/tests/e2e.rs:2541-2581](crates/sifr/tests/e2e.rs:2541) — `parse_expected_error` extracts the code (`SIFR-CALL-0004`) and the message-substring (`display() missing required argument 'verbose'`), and `failure.message.contains(...)` succeeds because the emitted text is byte-for-byte identical.
- `cargo test -p sifr --test e2e -- test_e2e_fail` → 1 passed (25 fixtures, +1 net new from this slice; the previous count in slice 2b.28's review was `1 passed (25 filtered)`, but `filtered` is the cargo-runtime selector display rather than the fixture count — the harness loops through every file in `tests/e2e/fail/` regardless).

### 4. HIR unit coverage
[crates/sifr_hir/src/lower/expressions_tests.rs:333-344](crates/sifr_hir/src/lower/expressions_tests.rs:333)

- `test_missing_required_argument_has_call_code` lowers a source string identical in shape to the e2e fixture, asserts both the **exact** message (`"display() missing required argument 'verbose'"`) and the **exact** `DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT` — matching the brief and the slice-2b.26/2b.28 convention.
- Test placement (immediately after `test_function_wrong_arg_count_has_call_code`), `lower_source(...)` helper, and assertion shape mirror the adjacent CALL-family tests, keeping the file coherent.
- The user-defined-`def` choice (rather than a stdlib re-test) is the right complement to the e2e fixture's same-source coverage: at the unit level this exercises `lower_function_call_args` directly via the type-system signature path, with no rendering/driver hops in the way. There's overlap with the e2e fixture, but that overlap is purposeful — together they pin the diagnostic at both the HIR boundary and the end-to-end harness.
- I re-ran `cargo test -p sifr_hir test_missing_required_argument_has_call_code` → 1 passed, 270 filtered.

### 5. Issue-tracker hygiene
[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:63-64](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:63)

- Slice 2b.28 line flipped from `[ ] ... implementation complete and reviewer-satisfied` to `[x] ... merged ... PR: https://github.com/sifr-lang/sifr/pull/1700.` — wording matches the merged-line template used by 2b.20–2b.27.
- A new `2b.29 in progress` entry is added on line 64 with the right shape: `shared missing required argument diagnostic migration to active SIFR-CALL-0004 with fixture and registry representative coverage. PR: pending.`. The "with fixture and registry representative coverage" qualifier (instead of slice 2b.28's "with stdlib fixture coverage") accurately reflects that this slice both adds a new e2e fixture AND retargets the registry's representative-fixture pointer.
- "shared missing required argument" accurately characterises the surface migrated — `missing_argument_error` is the only shared "required argument" emit helper in `method_call_args.rs`, and is the helper invoked from all four arity-completion paths in that file.

### 6. Coherence: is `SIFR-CALL-0004` the right home?

Yes. The CALL family is split as:

- `SIFR-CALL-0001` — wrong positional argument count.
- `SIFR-CALL-0002` — unexpected keyword argument.
- `SIFR-CALL-0003` — duplicate argument from positional/keyword overlap.
- `SIFR-CALL-0004` — missing required argument. ← this slice
- `SIFR-CALL-0005` — not callable / map-style callable arity.

The migrated path emits when a parameter has no positional-or-keyword binding and no default, which is precisely "missing required argument" — `SIFR-CALL-0004` is the correct bucket. The diagnostic-code constant `DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT` ([crates/sifr_diagnostics/src/codes.rs:58](crates/sifr_diagnostics/src/codes.rs:58)) was already defined and pointing at `SIFR-CALL-0004`, so no new code was introduced — this slice just turns on its emission.

### 7. Validation surface

The author-listed local validation set is the standard slice-2b mix and covers every surface this change touches:

- `cargo run -q -p sifr_diagnostics --bin gen-error-docs` — refreshed `docs/errors/SIFR-CALL-0004.md` against the new template/fixture in the registry.
- `python3 scripts/check_diagnostic_docs_sync.py` — verifies the generated docs match the registry. **Re-run by me → pass.**
- `python3 scripts/check_diagnostic_schema_sync.py` — verifies the JSON schema matches the Rust model. **Re-run by me → pass.**
- `python3 scripts/check_hir_maintainability_guardrails.py` — `HIR maintainability guardrails: PASS`. **Re-run by me → pass.**
- `cargo test -p sifr_hir missing_required_argument_has_call_code` — focused HIR unit. **Re-run by me → pass.**
- `cargo test -p sifr --test e2e -- test_e2e_fail` — exercises the new fixture. **Re-run by me → pass.**

The schema/docs gates are required here (registry template AND representative-fixture both changed), unlike slice 2b.28 where they were correctly omitted. The author included them, which is the right call.

## Concerns / non-blocking follow-ups

- **(A) `unexpected_keyword_error` still on the bridge inside `method_call_args.rs`.** [crates/sifr_hir/src/lower/method_call_args.rs:296-305](crates/sifr_hir/src/lower/method_call_args.rs:296) emits `"{callable_name}() got an unexpected keyword argument '{keyword}'"` via raw `ctx.error(...)`, despite the message being byte-identical to the migrated `SIFR-CALL-0002` template at [crates/sifr_diagnostics/src/codes.rs:806](crates/sifr_diagnostics/src/codes.rs:806). Slice 2b.26 migrated the *builtin*-call surface for unexpected keywords (in `expressions.rs` / `builtin_calls.rs`) but did not touch this shared helper. It is the symmetric companion of the `missing_argument_error` migration done in this slice and a `duplicate_argument_error` migration done in 2b.26 — wiring it onto `CALL_UNEXPECTED_KEYWORD` would close the last raw-`ctx.error` CALL-shape emit in `method_call_args.rs`. Out of scope for slice 2b.29 (scope is specifically the missing-required-argument helper); flag as the next CALL-family migration sub-slice.
- **(B) Two ad-hoc `"missing required argument"` emits in `expressions.rs` / `builtin_calls.rs` still on the bridge.** [crates/sifr_hir/src/lower/expressions.rs:1217](crates/sifr_hir/src/lower/expressions.rs:1217) emits `"sorted() missing required argument 'iterable'"` and [crates/sifr_hir/src/lower/builtin_calls.rs:839](crates/sifr_hir/src/lower/builtin_calls.rs:839) emits `"range() missing required argument 'stop'"`, both via raw `ctx.error(...)`. Both messages now align byte-for-byte with the `SIFR-CALL-0004` template — the obvious next move is to switch them to `error_with_code(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT, ...)` (or, even better, route them through the shared helper). These are outside the "shared `method_call_args.rs` helper" scope of this slice but should be the target of a follow-up sub-slice so all CALL-0004 emissions land structurally rather than via the phase bridge.
- **(C) `lower_keyword_args` raw emits.** [crates/sifr_hir/src/lower/method_call_args.rs:233-242](crates/sifr_hir/src/lower/method_call_args.rs:233) still emits `"{method}() does not support unpacked keyword arguments"` and `"{method}() got multiple values for keyword argument '{name}'"` via raw `ctx.error(...)`. The second one is a near-duplicate of `SIFR-CALL-0003`'s template. The first is genuinely new surface (kwargs unpacking) that doesn't have a code today. Out of scope here; flag for taxonomy follow-up.

None of (A)–(C) regress with this slice — they are all latent bridge emissions that pre-date it and are correctly left untouched given the slice's narrow "shared missing required argument" scope.

## Final word

Implementation is correct, scope is minimal and matches the brief, the fixture / HIR unit test / issue-tracker / registry / generated docs all agree on a single `SIFR-CALL-0004` code+template, and the migration consolidates four shared call-site emissions onto one structured code while closing a stale registry-fixture pointer. The message rewording cleanly aligns the live emission with the family style established in slices 2b.26–2b.28. **Approved for PR.**
