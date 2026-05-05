# `milestone_diag_4a` slice 2b.31 — Builtin `zip()` / `range()` / `enumerate()` unexpected-keyword diagnostic migration

Pass 1 review of the uncommitted working tree on branch
`codex/semantic-diagnostics-diag-4a-builtin-unexpected-keyword-diagnostics`.

## Scope under review

- Mark slice 2b.30 merged in the issue tracker after [sifr-lang/sifr#1702](https://github.com/sifr-lang/sifr/pull/1702) and add the in-progress entry for slice 2b.31.
- Migrate the three remaining ad-hoc unexpected-keyword emissions for builtins flagged as out-of-scope in slice 2b.30 (the `zip()` / `range()` / `enumerate()` follow-up A) from raw `ctx.error(...)` to `ctx.error_with_code(DiagnosticCode::CALL_UNEXPECTED_KEYWORD, ...)`:
  - [`reject_zip_keywords_if_present`](crates/sifr_hir/src/lower/builtin_calls.rs:13) — `zip()` non-`strict` keyword arm.
  - [`lower_range_call`](crates/sifr_hir/src/lower/builtin_calls.rs:835) — `range()` keyword loop default arm.
  - [`enumerate` block in `lower_call`](crates/sifr_hir/src/lower/expressions.rs:1338) — `enumerate()` post-`start` keyword validation loop.
- Keep `zip(strict=True)` "not supported" wording strictly unchanged and uncoded (explicitly out of scope).
- Add focused HIR unit coverage for the three migrated builtins:
  - Tighten `test_zip_keyword_diagnostics_are_stable` to assert exact message AND `DiagnosticCode::CALL_UNEXPECTED_KEYWORD` for the `bogus` arm (strict assertion left as `.contains(...)`).
  - Add `test_range_and_enumerate_unexpected_keywords_have_call_code` covering both `range()` and `enumerate()`.
- Add an e2e fail fixture [crates/sifr/tests/e2e/fail/zip_unexpected_keyword.sifr](crates/sifr/tests/e2e/fail/zip_unexpected_keyword.sifr) for the `zip()` non-`strict` keyword surface.
- No registry/generated docs change is intended: `SIFR-CALL-0002` already has the active template `{callable} got an unexpected keyword argument '{keyword}'` ([codes.rs:800-810](crates/sifr_diagnostics/src/codes.rs:800)) and the representative fixture `sorted_unexpected_keyword.sifr` from slice 2b.26.

This slice closes follow-up A from the slice 2b.30 review ([reviews/semantic-diagnostic-code-taxonomy-diag-4a-unexpected-keyword-diagnostics-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-4a-unexpected-keyword-diagnostics-review-pass-1.md)), which explicitly carved these three builtin sites out for a follow-up sub-slice.

## Verdict

**Approved — reviewer-satisfied for PR.** Implementation is correct, scope is minimal, message text is byte-for-byte preserved across all three migrated sites, and the HIR unit + e2e fixture coverage line up on a single `SIFR-CALL-0002` code+template. The decision to skip a registry/generated-docs refresh is correct: template, family, owner, severity, declared/dedupe args, and representative fixture are all unchanged from slice 2b.26 — only the in-code emit path changes from raw `ctx.error(...)` to `ctx.error_with_code(CALL_UNEXPECTED_KEYWORD, ...)`. The `zip(strict=True)` carve-out is preserved exactly as the task scope demands. No correctness, regression, or alignment blockers were found. Two strictly out-of-scope follow-ups (the `zip(strict=True)` "not supported" wording, which is the natural seed for a future `SIFR-CALL-00xx` "unsupported keyword" code; and a stylistic shape cleanup of `reject_zip_keywords_if_present`) are flagged at the bottom for future slices; neither blocks this PR.

## What I checked

### 1. HIR call-site migration — `zip()`
[crates/sifr_hir/src/lower/builtin_calls.rs:13-33](crates/sifr_hir/src/lower/builtin_calls.rs:13)

- `reject_zip_keywords_if_present` previously built a single `message` string in a `match` and emitted it via bare `ctx.error(message)`. The migration restructures so the `other` (non-`strict`, non-`None`) arm now calls `ctx.error_with_code(DiagnosticCode::CALL_UNEXPECTED_KEYWORD, format!("zip() got an unexpected keyword argument '{other}'"))` and `return true;` directly, while the `strict` arm continues to construct its message and fall through to the trailing `ctx.error(message); true` (uncoded, as required).
- The emitted message text for the unexpected-keyword path is byte-for-byte unchanged (`"zip() got an unexpected keyword argument '{other}'"`), matching the registry template `{callable} got an unexpected keyword argument '{keyword}'` ([codes.rs:806](crates/sifr_diagnostics/src/codes.rs:806)) under the same `{callable}` → `<name>()` substitution convention used by sorted/list-method/user-def call sites in slices 2b.26 and 2b.30.
- The `None` arm (`zip()` does not support unpacked keyword arguments) remains uncoded — correct, as that is a different diagnostic ("unpacked kwargs not supported") that is not in scope here and was also explicitly carved out by the slice 2b.30 review.
- The control-flow change is semantically a no-op for the existing `strict` and unpacked-kwargs paths: the function still returns `true` after emitting exactly one error, so callers (`lower_call`'s `func_name == "zip"` branch at [expressions.rs:1372-1375](crates/sifr_hir/src/lower/expressions.rs:1372)) still short-circuit identically. Confirmed by the unchanged strict assertion in `test_zip_keyword_diagnostics_are_stable`.
- `DiagnosticCode` is already imported at [builtin_calls.rs:2](crates/sifr_hir/src/lower/builtin_calls.rs:2) (used by the existing `range()` duplicate-argument migration). No new imports needed.

### 2. HIR call-site migration — `range()`
[crates/sifr_hir/src/lower/builtin_calls.rs:835-841](crates/sifr_hir/src/lower/builtin_calls.rs:835)

- The `other =>` arm of the keyword-name `match` inside `lower_range_call`'s loop now emits via `ctx.error_with_code(DiagnosticCode::CALL_UNEXPECTED_KEYWORD, format!("range() got an unexpected keyword argument '{other}'")); return None;`. The wording is byte-for-byte identical to the pre-migration text, and the `return None` short-circuits exactly as before — no risk of cascade diagnostics from later `lower_expr` calls on `start_expr`/`stop_expr`/`step_expr` being introduced.
- The siblings in the same `match` (`start`, `stop`, `step`) already use `error_with_code` with `DiagnosticCode::CALL_DUPLICATE_ARGUMENT` (slice 2b.26), so this migration brings the default arm into structural alignment with the rest of the loop. The earlier unpacked-kwargs guard at [builtin_calls.rs:801](crates/sifr_hir/src/lower/builtin_calls.rs:801) and the post-loop missing-stop emission at [builtin_calls.rs:846](crates/sifr_hir/src/lower/builtin_calls.rs:846) remain uncoded — both are out of scope for this slice (they map to "unsupported feature" / `CALL-0004` respectively and would be picked up by separate follow-ups).
- Behavior under simultaneous valid + invalid keywords: a call like `range(stop=3, bogus=1)` iterates the keywords in source order; `stop=3` populates `stop_expr` and falls through, then `bogus` triggers the migrated error. This is the path the new HIR test exercises, and it correctly produces a single coded diagnostic with no follow-up cascade.

### 3. HIR call-site migration — `enumerate()`
[crates/sifr_hir/src/lower/expressions.rs:1338-1356](crates/sifr_hir/src/lower/expressions.rs:1338)

- The "for keyword in &call.arguments.keywords" validation loop's non-`start` arm now calls `ctx.error_with_code(DiagnosticCode::CALL_UNEXPECTED_KEYWORD, format!("enumerate() got an unexpected keyword argument '{name}'")); return None;`. Wording byte-for-byte preserved; `return None` short-circuit unchanged.
- Diagnostic ordering trace under the new test input `enumerate(nums, bogus=1)`:
  1. `args.len() == 1` → arity guard at [expressions.rs:1298-1301](crates/sifr_hir/src/lower/expressions.rs:1298) passes.
  2. `lower_expr(&call.arguments.args[0], ctx)` lowers `nums` successfully (typed `list[int]`).
  3. `callable_builtin_element_type` resolves `int`.
  4. `args.len() != 2` and the `start` keyword lookup at [expressions.rs:1320-1325](crates/sifr_hir/src/lower/expressions.rs:1320) misses (only `bogus` is present), so `start` defaults to `IntLiteral(0)` without re-reading `bogus`.
  5. The validation loop iterates keywords; `bogus.arg.as_ref() == Some("bogus")`, the `name.as_str() != "start"` branch fires, emitting exactly one coded diagnostic, and the function returns `None`. Confirmed: no other diagnostic precedes it.
- Two pre-existing ordering subtleties of this block are intentionally left alone (out of scope and unchanged):
  - When `start=...` exists with a non-`int` value AND `bogus=...` is also present, the type-mismatch error at [expressions.rs:1328-1332](crates/sifr_hir/src/lower/expressions.rs:1328) fires first and the unexpected-keyword path is never reached. Pre-existing behavior; no regression.
  - When the call passes both `args.len() == 2` (start as positional) AND a `bogus=...` keyword, the migrated unexpected-keyword diagnostic fires before the duplicate-`start` check at [expressions.rs:1352-1355](crates/sifr_hir/src/lower/expressions.rs:1352), because the `name.as_str() != "start"` arm is checked first inside the loop. Pre-existing behavior; consistent with how `range()` orders its checks.
- `DiagnosticCode` is already imported at [expressions.rs:56](crates/sifr_hir/src/lower/expressions.rs:56) and used at multiple sites including the `sorted` unexpected-keyword arm at [expressions.rs:1202](crates/sifr_hir/src/lower/expressions.rs:1202). No new imports needed.

### 4. Strict-keyword carve-out preserved
[crates/sifr_hir/src/lower/builtin_calls.rs:21-32](crates/sifr_hir/src/lower/builtin_calls.rs:21)

- The `"strict" => "zip() keyword argument 'strict' is not supported"` arm still flows into bare `ctx.error(message)` — uncoded, exactly as the task scope demands. The strict path's emitted text is unchanged byte-for-byte, and the existing `test_zip_keyword_diagnostics_are_stable` strict-arm assertion (still using `.contains(...)`) continues to pass without modification.
- The `None`-arm (unpacked kwargs) is also untouched — also correct, as that's a separate diagnostic family and out of scope.
- This carve-out is the right choice for this slice: `strict=True` semantically belongs to a future "unsupported builtin keyword" / "feature-not-supported" code rather than to `CALL_UNEXPECTED_KEYWORD`, since `strict` is a *recognized* CPython 3.10+ keyword that Sifr deliberately does not implement (not a typo). Folding it into `CALL-0002` would conflate "bad name" with "known but unsupported", which would be a template/audience drift.

### 5. Why no registry / generated-docs change is correct
[crates/sifr_diagnostics/src/codes.rs:800-810](crates/sifr_diagnostics/src/codes.rs:800), [docs/errors/SIFR-CALL-0002.md](docs/errors/SIFR-CALL-0002.md)

- The registry entry already declares `SIFR-CALL-0002` as `active`, severity `Error`, owner `sifr_hir::lower`, with template `{callable} got an unexpected keyword argument '{keyword}'`, declared/dedupe args `callable, keyword`, and representative fixture `sorted_unexpected_keyword.sifr` — all set up by slice 2b.26 ([commit a515d9aa](https://github.com/sifr-lang/sifr/commit/a515d9aa)).
- Each of the three new runtime emissions (`zip() got an unexpected keyword argument 'bogus'`, `range() got an unexpected keyword argument 'bogus'`, `enumerate() got an unexpected keyword argument 'bogus'`) substitutes cleanly into that template under the same `{callable}` → `<name>()` substitution that `sorted_unexpected_keyword.sifr` already exercises, so the existing representative fixture remains accurate. No retargeting is warranted.
- This mirrors slice 2b.30's decision to skip schema/doc-sync gates: it is also a code-only emit-path migration with no registry diff.

### 6. E2E fixture
[crates/sifr/tests/e2e/fail/zip_unexpected_keyword.sifr](crates/sifr/tests/e2e/fail/zip_unexpected_keyword.sifr)

- Fixture body:
  - `nums: list[int] = [1, 2]` then `_paired = zip(nums, nums, bogus=True)`.
  - `_paired` underscore-prefix avoids the unused-binding warning that would otherwise fire on a successful lowering — correct and consistent with the `_paired = zip(...)` shape used in `test_zip_keyword_diagnostics_are_stable`.
- Trace through `lower_call`: `func_name == "zip"` → `reject_zip_keywords_if_present` → first keyword's `arg = Some("bogus")`, not "strict" → `error_with_code(CALL_UNEXPECTED_KEYWORD, "zip() got an unexpected keyword argument 'bogus'")`, `return true` → `lower_call` returns `None`. Single coded diagnostic, no cascade. The two `nums` positional args are never lowered, so no follow-up diagnostic competes with the migrated one.
- `# expect-error: SIFR-CALL-0002: zip() got an unexpected keyword argument 'bogus'` matches the harness contract at [crates/sifr/tests/e2e.rs:2541-2581](crates/sifr/tests/e2e.rs:2541): `parse_expected_error` extracts code `SIFR-CALL-0002` and message-substring `zip() got an unexpected keyword argument 'bogus'`, both of which match the propagated `CompileError::with_code(message, CompilePhase::TypeCheck, CALL_UNEXPECTED_KEYWORD)` produced by `lowering_error_to_compile_error` ([crates/sifr_driver/src/frontend/module_lowering.rs:36-52](crates/sifr_driver/src/frontend/module_lowering.rs:36)). The `failure.code == expected.code && failure.message.contains(expected.message_contains)` check at [e2e.rs:2561-2567](crates/sifr/tests/e2e.rs:2561) succeeds.
- The fixture omits the optional leading `# Reference: …` comment used by `sorted_unexpected_keyword.sifr`. The harness's `extract_expect_errors` only consumes `# expect-error:` lines, so this is harmless — and the missing-arg fixture from slice 2b.29 ([crates/sifr/tests/e2e/fail/missing_required_argument.sifr:1](crates/sifr/tests/e2e/fail/missing_required_argument.sifr:1)) and the function-unexpected-keyword fixture from slice 2b.30 ([crates/sifr/tests/e2e/fail/unexpected_keyword_argument.sifr:1](crates/sifr/tests/e2e/fail/unexpected_keyword_argument.sifr:1)) also omit it. Convention across `tests/e2e/fail/` is mixed; not a blocker.

### 7. Why one e2e fixture is sufficient (no `range`/`enumerate` fixtures)
[crates/sifr/tests/e2e/fail/sorted_unexpected_keyword.sifr](crates/sifr/tests/e2e/fail/sorted_unexpected_keyword.sifr), [crates/sifr/tests/e2e/fail/unexpected_keyword_argument.sifr](crates/sifr/tests/e2e/fail/unexpected_keyword_argument.sifr), [crates/sifr/tests/e2e/fail/zip_unexpected_keyword.sifr](crates/sifr/tests/e2e/fail/zip_unexpected_keyword.sifr)

- After this slice, three e2e fixtures pin distinct *call-shape surfaces* of `SIFR-CALL-0002`:
  - `sorted_unexpected_keyword.sifr` — single-positional builtin with whitelist match in `expressions.rs`.
  - `unexpected_keyword_argument.sifr` — user-defined `def` via the shared `unexpected_keyword_error` helper in `method_call_args.rs`.
  - `zip_unexpected_keyword.sifr` (new) — variadic builtin with eager keyword rejection in `builtin_calls.rs`.
- `range()` and `enumerate()` route through structurally similar keyword loops to the patterns already pinned — there is no semantically novel surface they would add at the e2e level beyond what the HIR unit test already asserts. Adding fixtures for them would duplicate the work of `test_range_and_enumerate_unexpected_keywords_have_call_code` without providing additional regression coverage. The slice's choice (one e2e for the variadic `zip` surface; HIR units for the other two) is well-calibrated.
- Confirmed: the slice does not regress fixture coverage. The pre-existing `range_duplicate_stop_keyword.sifr` continues to pin `SIFR-CALL-0003` for `range()`, and no `enumerate()` fixture existed before this slice (and none is needed now).

### 8. HIR unit coverage
[crates/sifr_hir/src/lower/expressions_tests.rs:1498-1541](crates/sifr_hir/src/lower/expressions_tests.rs:1498)

- `test_zip_keyword_diagnostics_are_stable` — the strict assertion is unchanged (`.contains("zip() keyword argument 'strict' is not supported")`) which correctly preserves the out-of-scope contract. The `bogus` assertion is tightened from `.message.contains(...)` to `error.message == "zip() got an unexpected keyword argument 'bogus'" && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)`. This is the right rigour change: under the old assertion shape the test would have continued to pass even after the helper still emitted the bridge code, so without this tightening the migration would have been silently un-asserted at the `zip` surface.
- `test_range_and_enumerate_unexpected_keywords_have_call_code` is a new test that lowers two source strings and asserts both the **exact** message and the **exact** `DiagnosticCode::CALL_UNEXPECTED_KEYWORD` for each:
  - `range(stop=3, bogus=1)` — exercises the keyword loop's default arm in `builtin_calls.rs` after a valid `stop` keyword has populated `stop_expr`. Wrapping in `print(list(range(...)))` is harmless (the diagnostic fires before the outer `list`/`print` ever try to lower their arguments) and ensures the source string is well-formed at the parser level.
  - `enumerate(nums, bogus=1)` — single positional iterable, single bad keyword. Avoids `start=` so the lookup at [expressions.rs:1320-1325](crates/sifr_hir/src/lower/expressions.rs:1320) misses and execution flows directly into the validation loop where the migrated error fires.
- Test placement is coherent: the new test is colocated with `test_zip_keyword_diagnostics_are_stable` in the expressions-tests file (the same file where the existing `test_sorted_unexpected_keyword_has_call_code` and `test_function_unexpected_keyword_has_call_code` live). This matches slice 2b.30's convention of grouping CALL-family unit tests together.
- The exact-message assertion shape (`==` rather than `.contains(...)`) is consistent with the slice 2b.30 tightening of `test_unexpected_method_keyword_is_rejected` and with the original `test_sorted_unexpected_keyword_has_call_code` from slice 2b.26. Slightly more brittle to wording changes, but the wording is the registry contract; pinning it exactly is the right rigour bar.
- `DiagnosticCode` is already in scope in this test file at [expressions_tests.rs:2](crates/sifr_hir/src/lower/expressions_tests.rs:2). No new imports.

### 9. Issue tracker update
[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:65-66](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:65)

- Slice 2b.30 transitions from `[ ] … implementation complete and reviewer-satisfied … PR: https://github.com/sifr-lang/sifr/pull/1702` to `[x] … merged … PR: https://github.com/sifr-lang/sifr/pull/1702`. Status is now consistent with PR #1702 having landed; PR URL preserved for traceability. Wording aligns with the prior slices 2b.27/2b.28/2b.29 (all use the "merged" terminology).
- Slice 2b.31 is added as `[ ] … in progress: builtin zip(), range(), and enumerate() unexpected keyword diagnostics migration to active SIFR-CALL-0002 with fixture and HIR coverage. PR: pending.` Phrasing mirrors the slice 2b.30 in-progress entry it replaced. The "fixture and HIR coverage" wording is faithful to what the slice actually delivers (one e2e fixture for `zip`, HIR units for `range` and `enumerate`).
- The append is single-line additive; no other entries were reordered or modified, and the deferred `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge entry at line 67 is left intact (still correctly tracking the eventual bridge deletion that will land after all CALL-family migrations complete).

### 10. Local validation correspondence
- The user-reported local validation matches what this slice could plausibly affect:
  - `cargo fmt` — formatting-only; passes.
  - `python3 scripts/check_hir_maintainability_guardrails.py` — `expressions.rs` is at 3710 lines vs. the 3800-line cap ([scripts/check_hir_maintainability_guardrails.py:18](scripts/check_hir_maintainability_guardrails.py:18)); the slice's net `+1` line in `expressions.rs` (-3 + 4 for the `error` → `error_with_code` reshape) keeps it well within the limit. `builtin_calls.rs` is not in the size-capped list, so its `+5` lines are unconstrained. No new banned monoliths.
  - `cargo test -p sifr_hir test_zip_keyword_diagnostics_are_stable` — the tightened test asserting `error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)` fails without this slice's `builtin_calls.rs` change, so its passing is positive evidence the migration took effect.
  - `cargo test -p sifr_hir test_range_and_enumerate_unexpected_keywords_have_call_code` — same shape, covering the other two builtins.
  - `cargo test -p sifr --test e2e -- test_e2e_fail` — covers the new fixture along with the existing `sorted_unexpected_keyword.sifr` and `unexpected_keyword_argument.sifr` fixtures; passing it confirms code+message propagation through the driver to the e2e harness.
- Note for the PR description: per `AGENTS.md`'s "Local validation (authoritative gate)" section, `scripts/run_all_tests.sh --profile quick` should also be run before the PR is opened. Recommend including its `report_signature` and `wall_time` in the issue tracker's eventual "merged" entry, matching the pattern from slices 2b.3–2b.5.

## Out-of-scope follow-ups (do not block this PR)

- **A. `zip(strict=True)` "not supported" wording** ([builtin_calls.rs:22](crates/sifr_hir/src/lower/builtin_calls.rs:22)). Currently uncoded by design. The natural home is a new `SIFR-CALL-00xx` "unsupported but recognized keyword" code (or potentially folding into a broader `SIFR-FEATURE-00xx` family), since `strict` is a real CPython 3.10+ keyword that Sifr deliberately doesn't implement. Slice scope correctly excludes this — it requires registry work (new code, new template, new representative fixture) rather than just an emit-path swap. The strict assertion in `test_zip_keyword_diagnostics_are_stable` continues to use `.contains(...)`, which is fine for the current uncoded path but should be tightened to exact `==` and `error.code == Some(...)` once a code is assigned.
- **B. Stylistic shape of `reject_zip_keywords_if_present`** ([builtin_calls.rs:13-33](crates/sifr_hir/src/lower/builtin_calls.rs:13)). After this slice, the function has a slightly asymmetric shape: the `strict` arm builds a `message` string and falls through to a trailing `ctx.error(message); true`, while the `other` arm calls `error_with_code` inline and `return true;`. Both halves are correct, but a future cleanup could either (i) hoist the `strict` emit into the same `match` arm style (`ctx.error(...); return true;` inline) so the trailing statement is no longer needed, or (ii) wait for follow-up A to migrate the `strict` arm to its own coded helper, which would naturally regularize the shape. Not worth churning in this slice given the imminence of follow-up A.
- **C. Other ad-hoc unexpected-keyword surfaces.** A grep for `unexpected keyword` across `crates/sifr_hir/src/` (excluding tests) confirms this slice plus slice 2b.30 cover every production unexpected-keyword emission site: `builtin_calls.rs:26` (zip), `builtin_calls.rs:838` (range), `expressions.rs:1203` (sorted, already on `CALL-0002` since 2b.26), `expressions.rs:1348` (enumerate), and `method_call_args.rs:303` (shared helper, on `CALL-0002` since 2b.30). No further surfaces remain. With this slice, the `CALL-0002` migration is complete from the HIR side; the deferred `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge can advance toward removal as scheduled.

## Summary

The slice correctly closes follow-up A from slice 2b.30 by routing the three remaining ad-hoc builtin unexpected-keyword diagnostics through the active `SIFR-CALL-0002` code, with byte-identical message text, no template/registry/doc churn, focused HIR unit coverage for all three builtins, and a single new e2e fixture for the variadic `zip` surface. The `zip(strict=True)` carve-out is preserved exactly. Issue tracker is updated cleanly. No correctness, regression, or alignment blockers; ready for PR.
