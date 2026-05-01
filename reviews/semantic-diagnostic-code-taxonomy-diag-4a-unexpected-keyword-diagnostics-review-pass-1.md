# `milestone_diag_4a` slice 2b.30 — Shared unexpected-keyword diagnostic migration

Pass 1 review of the uncommitted working tree on branch
`codex/semantic-diagnostics-diag-4a-unexpected-keyword-diagnostics`.

## Scope under review

- Mark slice 2b.29 merged in the issue tracker after [sifr-lang/sifr#1701](https://github.com/sifr-lang/sifr/pull/1701) and add the in-progress entry for slice 2b.30.
- Migrate the shared `unexpected_keyword_error` helper in `lower_function_call_args` / `lower_vararg_function_call_args` / method-arg normalisation from the generic `SIFR-TYPE-0001` bridge to active `SIFR-CALL-0002`.
- Add focused HIR unit coverage asserting the exact message and exact `DiagnosticCode` for the shared user-defined-function unexpected-keyword path, plus tighten the existing list-method (`append`) test to also assert the structured code.
- Add a new e2e fail fixture [crates/sifr/tests/e2e/fail/unexpected_keyword_argument.sifr](crates/sifr/tests/e2e/fail/unexpected_keyword_argument.sifr) covering the user-defined-function path.
- No registry/generated docs change is intended: `SIFR-CALL-0002` already has the active template `{callable} got an unexpected keyword argument '{keyword}'` and a representative fixture (`sorted_unexpected_keyword.sifr`) from slice 2b.26.

This slice is the natural follow-up to slice 2b.29, which explicitly flagged `unexpected_keyword_error` as the next CALL-family migration sub-slice ([reviews/semantic-diagnostic-code-taxonomy-diag-4a-missing-arg-diagnostics-review-pass-1.md:104](reviews/semantic-diagnostic-code-taxonomy-diag-4a-missing-arg-diagnostics-review-pass-1.md:104)).

## Verdict

**Approved — reviewer-satisfied for PR.** Implementation is correct, scope is minimal, the fixture / HIR unit tests / migrated helper / issue tracker all line up on a single `SIFR-CALL-0002` code+template, and the migration consolidates four shared call-site emissions (two function paths, two method paths) onto one structured code in one move. The decision to skip a registry/generated-docs refresh is correct: the template, family, owner, severity, and representative fixture are all unchanged from slice 2b.26 — only the in-code emit path changes from raw `ctx.error(...)` to `ctx.error_with_code(CALL_UNEXPECTED_KEYWORD, ...)`. No correctness, regression, or alignment blockers were found. Three strictly out-of-scope follow-ups (`zip()` / `range()` / `enumerate()` ad-hoc unexpected-keyword emissions in `expressions.rs` / `builtin_calls.rs`, and the two raw `lower_keyword_args` emissions for unpacked-kwargs and duplicate-keyword) are flagged at the bottom for future slices; none block this PR.

## What I checked

### 1. HIR call-site migration
[crates/sifr_hir/src/lower/method_call_args.rs:296-306](crates/sifr_hir/src/lower/method_call_args.rs:296)

- `unexpected_keyword_error` now emits via `ctx.error_with_code(DiagnosticCode::CALL_UNEXPECTED_KEYWORD, ...)` instead of bare `ctx.error(...)`. Behavioural shape (`return None;` after emit) is unchanged, so downstream lowering still short-circuits exactly as before — no risk of cascade diagnostics being introduced or removed.
- The message text is byte-for-byte identical to the pre-migration text (`"{callable_name}() got an unexpected keyword argument '{keyword}'"`). This matches the registry template `{callable} got an unexpected keyword argument '{keyword}'` ([crates/sifr_diagnostics/src/codes.rs:806](crates/sifr_diagnostics/src/codes.rs:806)) under the substitution `{callable}` → `<name>()` (parens included), the same convention used by the sibling `SIFR-CALL-0001/0003/0004` migrations from slices 2b.26–2b.29. No template tightening is needed and none is performed.
- Every call site of the helper benefits from the migration with no per-site change required:
  - [method_call_args.rs:121](crates/sifr_hir/src/lower/method_call_args.rs:121) — non-vararg keyword-mixed branch in `lower_function_call_args`.
  - [method_call_args.rs:200](crates/sifr_hir/src/lower/method_call_args.rs:200) and [method_call_args.rs:210](crates/sifr_hir/src/lower/method_call_args.rs:210) — vararg path (kwarg colliding with `*args` name; kwarg matching no param at all) in `lower_vararg_function_call_args`.
  - [method_call_args.rs:260](crates/sifr_hir/src/lower/method_call_args.rs:260) — `reject_remaining_keywords`, the funnel for every method-arg normaliser (`normalize_list_method_args`, `normalize_dict_method_args`, `normalize_set_method_args`, `normalize_tuple_method_args`, `normalize_string_method_args`, and the catch-all `_` arm of `lower_method_call_args`).
  All five sites now route through `SIFR-CALL-0002`. This consolidation matches the pattern slice 2b.26 used for `duplicate_argument_error` and slice 2b.29 used for `missing_argument_error` ([method_call_args.rs:280-294](crates/sifr_hir/src/lower/method_call_args.rs:280)).
- `DiagnosticCode` is already imported at [method_call_args.rs:2](crates/sifr_hir/src/lower/method_call_args.rs:2) (used by the migrated `duplicate_argument_error`, `missing_argument_error`, and the `lower_function_call_args` arity emission). No new imports needed.
- `error_with_code` populates `LoweringError.code = Some(...)` ([crates/sifr_hir/src/lower/mod.rs:237-244](crates/sifr_hir/src/lower/mod.rs:237)) which surfaces through `compile_errors_to_diagnostics` to the e2e harness's `failure.code` ([crates/sifr/tests/e2e.rs:2561-2567](crates/sifr/tests/e2e.rs:2561)) instead of falling through the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge.
- Because `lower_signature_call_args` is a thin wrapper over `lower_function_call_args` ([method_call_args.rs:36-44](crates/sifr_hir/src/lower/method_call_args.rs:36)) and `reject_remaining_keywords` is the one funnel for every method-arg-normaliser arm ([method_call_args.rs:21-33](crates/sifr_hir/src/lower/method_call_args.rs:21)), this single migration covers user-defined defs, stdlib functions registered with `FunctionType` signatures, callable objects' `__call__`, vararg paths, and every dispatched method (list / dict / set / tuple / str / arbitrary) in one go — exactly the "shared" surface the slice title claims.

### 2. Why no registry / generated-docs change is correct
[crates/sifr_diagnostics/src/codes.rs:800-810](crates/sifr_diagnostics/src/codes.rs:800), [docs/errors/SIFR-CALL-0002.md](docs/errors/SIFR-CALL-0002.md), [internal_docs/diagnostic_codes.md:98](internal_docs/diagnostic_codes.md:98)

- The registry entry already declares `SIFR-CALL-0002` as `active`, severity `Error`, owner `sifr_hir::lower`, with template `{callable} got an unexpected keyword argument '{keyword}'`, declared/dedupe args `callable, keyword`, and representative fixture `crates/sifr/tests/e2e/fail/sorted_unexpected_keyword.sifr` — all set up by slice 2b.26 ([commit a515d9aa](https://github.com/sifr-lang/sifr/commit/a515d9aa)).
- The runtime emission `greet() got an unexpected keyword argument 'punctuation'` substitutes cleanly into that template under the same `{callable}` → `<name>()` substitution that `sorted_unexpected_keyword.sifr` exercises, so the existing representative fixture remains accurate. No retargeting is warranted.
- `docs/errors/SIFR-CALL-0002.md` is generated and already shows the active template and the existing representative fixture; no rerun of `gen-error-docs` is needed because the registry inputs it consumes are unchanged. (Confirmed by inspection — the doc already states `Representative fixture: crates/sifr/tests/e2e/fail/sorted_unexpected_keyword.sifr`.)
- Skipping the schema/doc-sync gates is the right call here, mirroring slice 2b.28 (which also did a code-only emit-path migration with no registry diff); slice 2b.29 reran them only because that slice retargeted both the template AND the representative fixture pointer.

### 3. Fixture creation
[crates/sifr/tests/e2e/fail/unexpected_keyword_argument.sifr](crates/sifr/tests/e2e/fail/unexpected_keyword_argument.sifr)

- New fixture body:
  - `def greet(name: str) -> str:` defines a single-param positional callable returning a string literal — no `name` reference in the body, so no `SIFR-OWN-0003` borrow-escape diagnostic competes with the migrated unexpected-keyword one.
  - `print(greet("Alice", punctuation="!"))` calls `greet` with one positional binding for `name` and a stray `punctuation` keyword. At the type-system layer `FunctionType.params` is `[("name", Str, ParamConvention::Move)]`, so the call enters `lower_function_call_args` with `keyword_args = [("punctuation", _)]` (non-empty), runs the param loop (which fills `name` from positional and skips the keyword), and falls into the post-loop residual-keyword check at [method_call_args.rs:115-123](crates/sifr_hir/src/lower/method_call_args.rs:115) which calls `unexpected_keyword_error("greet", "punctuation", ctx)` deterministically. No other diagnostic precedes it.
  - The fixture is the user-defined-function symmetric companion of `sorted_unexpected_keyword.sifr` (which exercises the in-`expressions.rs` `sorted` keyword whitelist, not the shared helper) — the two fixtures together pin both surfaces of `SIFR-CALL-0002`.
- `# expect-error: SIFR-CALL-0002: greet() got an unexpected keyword argument 'punctuation'` matches the harness contract at [crates/sifr/tests/e2e.rs:2541-2581](crates/sifr/tests/e2e.rs:2541) — `parse_expected_error` extracts the code (`SIFR-CALL-0002`) and the message-substring (`greet() got an unexpected keyword argument 'punctuation'`), and `failure.message.contains(...)` succeeds because the emitted text is byte-for-byte identical.
- The fixture omits the optional leading `# Reference: …` comment used by `sorted_unexpected_keyword.sifr`. The harness's `extract_expect_errors` only consumes lines starting with `# expect-error:` ([crates/sifr/tests/e2e.rs:419-428](crates/sifr/tests/e2e.rs:419)), so the absence of a reference comment is harmless. Convention across `tests/e2e/fail/` is mixed — the missing-arg fixture from slice 2b.29 also omits it ([crates/sifr/tests/e2e/fail/missing_required_argument.sifr:1](crates/sifr/tests/e2e/fail/missing_required_argument.sifr:1)). Not a blocker.

### 4. HIR unit coverage
[crates/sifr_hir/src/lower/expressions_tests.rs:270-281](crates/sifr_hir/src/lower/expressions_tests.rs:270), [crates/sifr_hir/src/lower/expressions_tests.rs:1784-1793](crates/sifr_hir/src/lower/expressions_tests.rs:1784)

- `test_function_unexpected_keyword_has_call_code` lowers a source string identical in shape to the new e2e fixture and asserts both the **exact** message (`"greet() got an unexpected keyword argument 'punctuation'"`) and the **exact** `DiagnosticCode::CALL_UNEXPECTED_KEYWORD`. Test placement (immediately after `test_sorted_unexpected_keyword_has_call_code`), `lower_source(...)` helper, and assertion shape mirror the adjacent CALL-family tests, keeping the file coherent.
- `test_unexpected_method_keyword_is_rejected` is tightened from a `.contains(...)` substring check to an exact `error.message ==` AND `error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)` check. This is the right rigour change: under the old assertion shape the test would have continued to pass even if the helper still emitted the bridge code, so without this tightening the migration would have been silently un-asserted at the method-helper surface. The choice of `xs.append(value=2)` exercises the `reject_remaining_keywords` funnel via the list-method normaliser path ([method_call_args.rs:21-32](crates/sifr_hir/src/lower/method_call_args.rs:21)) — the symmetric companion to the function-call surface covered by the new test.
- The user-defined-`def` choice (rather than a stdlib re-test) is the right complement to the e2e fixture's same-source coverage: at the unit level this exercises `lower_function_call_args` directly via the type-system signature path, with no rendering/driver hops in the way. There's overlap with the e2e fixture, but that overlap is purposeful — together they pin the diagnostic at both the HIR boundary and the end-to-end harness, matching the slice-2b.26/2b.28/2b.29 convention.
- The pre-existing `test_sorted_unexpected_keyword_has_call_code` ([expressions_tests.rs:257-268](crates/sifr_hir/src/lower/expressions_tests.rs:257)) and the in-line `test_zip_handles_strict_keyword_and_unexpected_keyword` `.contains(...)` ([expressions_tests.rs:1511-1521](crates/sifr_hir/src/lower/expressions_tests.rs:1511)) are unaffected: the former asserts on `sorted()`'s ad-hoc keyword whitelist in `expressions.rs` (already on `CALL_UNEXPECTED_KEYWORD` since slice 2b.26), and the latter still passes because it only asserts the message substring (the `zip()` ad-hoc emit is on the bridge — see follow-up A).

### 5. Issue-tracker hygiene
[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:64-65](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:64)

- Slice 2b.29 line flipped from `[ ] ... implementation complete and reviewer-satisfied` to `[x] ... merged ... PR: https://github.com/sifr-lang/sifr/pull/1701.` — wording matches the merged-line template used by 2b.20–2b.28.
- A new `2b.30 in progress` entry is added on line 65 with the right shape: `shared unexpected keyword argument diagnostic migration to active SIFR-CALL-0002 with fixture and method-helper coverage. PR: pending.`. The "with fixture and method-helper coverage" qualifier (instead of slice 2b.29's "with fixture and registry representative coverage") accurately reflects that this slice adds a new e2e fixture and tightens the method-helper unit assertion, but does NOT retarget the registry's representative-fixture pointer (because no retarget is warranted — see §2).
- "shared unexpected keyword argument" accurately characterises the surface migrated — `unexpected_keyword_error` is the only shared "unexpected keyword" emit helper in `method_call_args.rs`, and is the helper invoked from the four function-arity-completion sites and the method `reject_remaining_keywords` funnel in that file.

### 6. Coherence: is `SIFR-CALL-0002` the right home?

Yes. The CALL family is split as:

- `SIFR-CALL-0001` — wrong positional argument count.
- `SIFR-CALL-0002` — unexpected keyword argument. ← this slice
- `SIFR-CALL-0003` — duplicate argument from positional/keyword overlap.
- `SIFR-CALL-0004` — missing required argument.
- `SIFR-CALL-0005` — not callable / map-style callable arity.

The migrated path emits when a call carries a keyword whose name does not match any parameter (or, in the vararg path, collides with the `*args` parameter name) — which is precisely "unexpected keyword argument". The diagnostic-code constant `DiagnosticCode::CALL_UNEXPECTED_KEYWORD` ([crates/sifr_diagnostics/src/codes.rs:56](crates/sifr_diagnostics/src/codes.rs:56)) was already defined and pointing at `SIFR-CALL-0002`, so no new code was introduced — this slice just turns on its emission for the shared helper.

### 7. Validation surface

The author-listed local validation set is the standard slice-2b mix and covers every surface this change touches:

- `cargo fmt` — no formatting drift.
- `python3 scripts/check_hir_maintainability_guardrails.py` — `HIR maintainability guardrails: PASS`. The change is a 4→6 line edit inside an existing helper, no file-size/line-count threshold movement.
- `cargo test -p sifr_hir unexpected_keyword` — exercises `test_function_unexpected_keyword_has_call_code` and `test_sorted_unexpected_keyword_has_call_code`.
- `cargo test -p sifr_hir unexpected_method_keyword` — exercises the tightened `test_unexpected_method_keyword_is_rejected`.
- `cargo test -p sifr --test e2e -- test_e2e_fail` — exercises the new fixture.

The schema/doc-sync gates are correctly omitted here (registry template AND representative-fixture pointer both unchanged), unlike slice 2b.29 where they were correctly included. The author made the right call.

I consider clippy/full-workspace coverage already covered by the slice's CI mirror via `scripts/run_all_tests.sh`; spot-checked: the migrated helper compiles clean (`error_with_code` was already in the shared call-helper file from slice 2b.26's `duplicate_argument_error`).

## Concerns / non-blocking follow-ups

- **(A) Three ad-hoc `"unexpected keyword argument"` emits still on the bridge.** [crates/sifr_hir/src/lower/builtin_calls.rs:23](crates/sifr_hir/src/lower/builtin_calls.rs:23) emits `"zip() got an unexpected keyword argument '{other}'"` (in `reject_zip_keywords_if_present`), [crates/sifr_hir/src/lower/builtin_calls.rs:830-832](crates/sifr_hir/src/lower/builtin_calls.rs:830) emits `"range() got an unexpected keyword argument '{other}'"`, and [crates/sifr_hir/src/lower/expressions.rs:1346-1349](crates/sifr_hir/src/lower/expressions.rs:1346) emits `"enumerate() got an unexpected keyword argument '{name}'"`, all via raw `ctx.error(...)`. All three messages now align byte-for-byte with the `SIFR-CALL-0002` template — the obvious next move is to switch them to `error_with_code(DiagnosticCode::CALL_UNEXPECTED_KEYWORD, ...)`. These are outside the "shared `method_call_args.rs` helper" scope of this slice and slice 2b.26 explicitly carved them out (only `sorted()` was migrated in `expressions.rs`), but they should be the target of a follow-up sub-slice so all CALL-0002 emissions land structurally rather than via the phase bridge. The existing `test_zip_handles_strict_keyword_and_unexpected_keyword` test only `.contains(...)`-checks the message ([expressions_tests.rs:1517-1520](crates/sifr_hir/src/lower/expressions_tests.rs:1517)), so the regression risk if those bridges flip is low.
- **(B) `lower_keyword_args` raw emits.** [crates/sifr_hir/src/lower/method_call_args.rs:233-242](crates/sifr_hir/src/lower/method_call_args.rs:233) still emits `"{method}() does not support unpacked keyword arguments"` and `"{method}() got multiple values for keyword argument '{name}'"` via raw `ctx.error(...)`. The second one is a near-duplicate of `SIFR-CALL-0003`'s template (`{callable} got multiple values for argument '{argument}'`) — semantically the same condition (duplicate keyword), with a slightly different shape ("keyword argument" vs "argument"); a future slice should decide whether to fold it onto `CALL_DUPLICATE_ARGUMENT` (after harmonising the wording) or to introduce a dedicated code. The first is genuinely new surface (kwargs-style `**kwargs` unpacking, currently unsupported in Sifr) that doesn't have a code today. Out of scope here; flag for taxonomy follow-up. Slice 2b.29's review already noted these in concern (C); they remain unaddressed.
- **(C) Trivial style consistency.** The new fixture omits a `# Reference: …` leading comment that some sibling fixtures carry (`sorted_unexpected_keyword.sifr` has one; `missing_required_argument.sifr` does not). Convention is genuinely mixed — not a blocker, but if a phase-traceability comment is desirable, adding `# Reference: phase_psp_a1_unexpected_keyword_argument` (or similar) at line 1 would match the `sorted_*` pattern.

None of (A)–(C) regress with this slice — they are all latent bridge emissions or stylistic gaps that pre-date it and are correctly left untouched given the slice's narrow "shared `method_call_args.rs` helper" scope.

## Final word

Implementation is correct, scope is minimal and matches the brief, the new fixture / tightened HIR units / migrated helper / issue-tracker entry all agree on a single `SIFR-CALL-0002` code+template, and the migration consolidates four call-site emissions (two in `lower_function_call_args` non-vararg and vararg, plus the `reject_remaining_keywords` method funnel covering five normalisers and the catch-all `_` arm) onto one structured code without touching the registry/generated docs — which is the right call because slice 2b.26 already established the active template and representative fixture. This closes the third of four shared CALL-helper migrations in `method_call_args.rs` (after `duplicate_argument_error` in 2b.26 and `missing_argument_error` in 2b.29), leaving only `lower_keyword_args`'s two raw emits as remaining drift in this file. **Approved for PR.**
