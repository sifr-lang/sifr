# Review: milestone_diag_4a — Slice 2 Pre-Implementation (Pass 1)

Branch: `codex/semantic-diagnostics-diag-4a-slice2` (working tree on top of `10129970`, the slice 1 merge)
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Prior reviews:
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-3.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-3.md)

Slice scope as planned by the user prompt:

1. Add structured `DiagnosticCode` transport to HIR `LoweringError` so user-facing TypeCheck diagnostics no longer rely on `CompileError`'s legacy `TypeCheck => "SIFR-TYPE-0001"` fallback.
2. Update driver frontend / test-runner / stdlib forwarding to preserve the HIR code via `CompileError::with_code`.
3. Delete or neutralize `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` once no production user-facing TypeCheck path depends on it.
4. Re-key affected verification + e2e baselines from `SIFR-TYPE-0001` to active codes.
5. Treat this as a *mechanical transport* slice — defer category-specific helper refinement, related-span work, and dedupe-arg design to `milestone_diag_7` and `milestone_diag_8` unless required for correctness.

## Verdict

**The slice as scoped is internally consistent with the issue's `milestone_diag_4a` contract, but cannot ship as one PR without becoming the largest patch in this milestone by an order of magnitude.** Achieving "no production HIR call site falls back to `SIFR-TYPE-0001`" is a precondition for deleting the bridge arm, and that precondition currently requires touching **489 `LowerCtx::error(...)` call sites across 25 HIR files**, plus re-keying ~90 e2e fixtures, 2 verification baselines, and 23 driver/CLI unit-test occurrences. Bundling the transport plumbing, the call-site migration, the bridge deletion, and the fixture re-keying into a single PR is a reviewability and bisect-bisectability risk that the prior three slice-1 review passes already flagged when they recommended scope-splitting.

I recommend splitting slice 2 into three sub-slices (2a/2b/2c described below). If the implementer prefers a single PR, the safe ordering inside it is fixed and is also documented below.

A centralized message-prefix dispatcher *must not* be reintroduced — the issue explicitly forbids it as a non-goal, and slice 1 just removed an analogous workspace classifier. Every emitting site needs a deliberate code; the typesystem-enforced way to guarantee that is a non-defaulting `LowerCtx::error_with_code(code, message)` plus a one-shot deletion of the codeless `LowerCtx::error(message)` once migration is complete.

## Inventory of the surface this slice must cover

These numbers are derived from `crates/sifr_hir/src/`, `crates/sifr_driver/src/`, the registry in `crates/sifr_diagnostics/src/codes.rs`, and the e2e/verification trees. They are the load-bearing facts for the sequencing recommendation.

| Surface | Count | Location |
| --- | --- | --- |
| `LowerCtx::error(...)` call sites across HIR | **489** | `crates/sifr_hir/src/lower/**` (top concentration: `expressions.rs` 205, `statements.rs` 64, `builtin_calls.rs` 55) |
| `LowerCtx` emission helpers today | **2** | `error(&mut self, message: String)` and `warn(&mut self, message: String)` at [lower/mod.rs:205-215](../crates/sifr_hir/src/lower/mod.rs:205) — neither carries a `DiagnosticCode` |
| HIR → driver `CompileError` conversion sites | **3** | [frontend/module_lowering.rs:25-39](../crates/sifr_driver/src/frontend/module_lowering.rs:25), [test_runner/orchestrator.rs:107-118](../crates/sifr_driver/src/test_runner/orchestrator.rs:107), [stdlib/bootstrap.rs:58-73](../crates/sifr_driver/src/stdlib/bootstrap.rs:58) |
| Fixtures with `expect-error: SIFR-TYPE-0001` (bare) | **76 `.sifr` files** | `crates/sifr/tests/e2e/fail/*.sifr` |
| Fixtures with hybrid `[SIFR-TYPE-0001] [E25xx]` (decimal) | **~14 `.sifr` files** | same directory; e.g. `decimal_forbidden_mixed_arithmetic_seeded.sifr`, `bigdecimal_quantize_negative_scale_context.sifr` |
| Verification baselines pinning `SIFR-TYPE-0001` | **2** | `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-{json,compact}.stderr.txt` |
| Driver/CLI hard-coded `"SIFR-TYPE-0001"` | **23 occurrences** | [crates/sifr/src/main.rs](../crates/sifr/src/main.rs) (20 lines, all unit tests for the legacy renderer), [crates/sifr_driver/src/tests/diagnostics.rs](../crates/sifr_driver/src/tests/diagnostics.rs) (3 lines, recovery-cap synthetic tests) |
| Bridge arms still emitting **retired** codes | **all 4** | [diagnostics.rs:135-140](../crates/sifr_driver/src/diagnostics.rs:135) — `Parse → SIFR-PARSE-0001`, `TypeCheck → SIFR-TYPE-0001`, `Codegen → SIFR-CODEGEN-0001`, `Build → SIFR-BUILD-0001`. Slice 1 did not repoint any of them; only the explicit `with_code` callers carry active codes. |
| Decimal `[E25xx]` pseudo-codes still embedded in HIR messages | **~36 sites** | `crates/sifr_hir/src/lower/decimal_methods.rs`, `expressions.rs`, `crates/sifr_type_system/src/check.rs`. **Owned by milestone_diag_6**, not this slice. |

## Active codes available for HIR migration (in-scope code budget)

`crates/sifr_diagnostics/src/codes.rs` already defines the inventory codes that slice 2 must thread through. Counted by family, currently active:

- `SIFR-TYPE-0002..0009` (8 errors) + `0901` (warning) + `0902` (note) — type mismatch, branch mismatch, missing/invalid annotation, unsupported operator, int/bigint mixed, container element conflict, tuple unpack shape mismatch, arithmetic-overflow risk warning, reveal-type note.
- `SIFR-NAME-0001..0004` — undefined variable, undefined callable, unknown type, missing module/class member.
- `SIFR-IMPORT-0001..0002` — forbidden intrinsic, unknown source module.
- `SIFR-DECIMAL-0001..0008` — invalid literals (Decimal/BigDecimal), float-mixed, Decimal/BigDecimal mixed, float-construction forbidden (×2), scale invalid (×2).
- `SIFR-CALL-0001..0005` — wrong positional count, unexpected keyword, duplicate argument, missing required argument, callable arity / not-callable.
- `SIFR-OWN-0001..0004` — use-after-move, double mutable borrow, borrowed parameter escape, moved-across-loop.
- `SIFR-FLOW-0001..0003` (errors) + `0901` (warning) — break/continue/nonlocal misuse, unreachable.
- `SIFR-MATCH-0001..0003` — non-exhaustive match, guard-not-bool, invalid class-pattern field.
- `SIFR-PROTO-0001..0004` — protocol bound, iterator/reversible signature, context-manager missing, hashable/comparable required.
- `SIFR-CLASS-0001..0004` — missing initializer, required-after-default, duplicate/invalid value, missing member.
- `SIFR-RESULT-0001..0003` — unused Result, invalid error type, invalid raise.
- `SIFR-INTERNAL-0001` — internal compiler panic / invariant.

Total: **52 active inventory codes**, which `milestone_diag_3` already mapped against the call-site categories. The mechanical migration of the 489 sites should consume nearly all of them.

There is no active "TYPE catch-all" — and the issue is explicit that one must not be invented. If a call site does not fit any of the codes above, the correct response is to add a new active registry entry (with docs page, fixture plan, and registry constant), not to invent a transitional bucket. The issue also specifies that the registry can be extended in the migration milestone that emits the new code.

## Question 1: is a centralized transitional HIR code mapping acceptable?

**No, not as a runtime classifier — and the type system is the right place to enforce that.**

Three concrete framings:

1. **Message-prefix or message-substring dispatcher** — explicitly a non-goal of the phase ([issue line 165](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:165) "Add a string-prefix-to-code classifier"). Slice 1 just removed `CompileError::workspace_diagnostic_code`, the previous incarnation of this pattern. Reintroducing one in HIR would directly contradict the slice 1 verdict and the registered slice 1 regression test at [tests/diagnostics.rs:71-81](../crates/sifr_driver/src/tests/diagnostics.rs:71). Hard "no".

2. **A single transitional "HIR catch-all" code (e.g. a new `TYPE_GENERIC_LOWERING_FAILURE`)** — also wrong. It would re-create the `SIFR-TYPE-0001` bucket under a new name, regress every fidelity gain the registry population in `milestone_diag_2b` made, and violate the [diagnostic identity policy](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:134) ("a diagnostic code identifies the *kind* of user-facing compiler error"). Don't.

3. **A type-enforced transition where every emission must carry a code** — *yes*, this is the structurally safe path. Concretely:

   - Add `code: DiagnosticCode` (non-`Option`) to `LoweringError`, **or** keep it `Option<DiagnosticCode>` for the duration of slice 2a and tighten to non-optional in slice 2c.
   - Add `LowerCtx::error_with_code(&mut self, code: DiagnosticCode, message: String)`; have it set the `code` field on the new `LoweringError`.
   - Once every call site is migrated, **delete `LowerCtx::error(message: String)` outright** so any unmigrated site becomes a compile error rather than a silent fallback. This matches the issue's "must use an inventory-assigned canonical code through SifrDiagnostic transport or fail to compile" wording at [issue line 863](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:863).

   The "fail to compile" guarantee is the exact opposite of a centralized transitional dispatcher; it forces every author to make the code choice locally, which is the slice's actual goal.

Practical answer to the user's question: **every `LowerCtx::error` call must be touched in this slice (or a sub-slice of it)**. There is no inventory-faithful shortcut that lands the bridge deletion early.

## Question 2: which active codes to use for broad HIR categories without replacing one bad bucket with another

The risk of mass migration is "label-laundering" — pulling 489 sites into a small handful of codes (e.g. dumping anything ambiguous into `TYPE_MISMATCH`) and ending up with a new fat bucket. The defenses are:

1. **Use the registry's per-domain active codes by category, not by file.** The category breakdown for the 489 sites:

   | Category (issue's diag_3 inventory bucketing) | Representative call sites | Target active code(s) |
   | --- | --- | --- |
   | Type mismatch (assignment, return, generic application) | [expressions.rs:410-420](../crates/sifr_hir/src/lower/expressions.rs:410), [statements.rs](../crates/sifr_hir/src/lower/statements.rs), [aug_assign_lowering.rs](../crates/sifr_hir/src/lower/aug_assign_lowering.rs) | `TYPE_MISMATCH`, `TYPE_IF_BRANCH_MISMATCH`, `TYPE_UNSUPPORTED_OPERATOR` |
   | Undefined name | [expressions.rs:245](../crates/sifr_hir/src/lower/expressions.rs:245), [statements.rs:1534](../crates/sifr_hir/src/lower/statements.rs:1534) | `NAME_UNDEFINED_VARIABLE`, `NAME_UNDEFINED_CALLABLE`, `NAME_UNKNOWN_TYPE`, `NAME_MISSING_MODULE_MEMBER` |
   | Ownership / move | [expressions.rs:224](../crates/sifr_hir/src/lower/expressions.rs:224) | `OWN_USE_AFTER_MOVE`, `OWN_DOUBLE_MUTABLE_BORROW`, `OWN_BORROWED_PARAMETER_ESCAPES`, `OWN_MOVED_ACROSS_LOOP` |
   | Control flow | [statements.rs:201](../crates/sifr_hir/src/lower/statements.rs:201) | `FLOW_BREAK_OUTSIDE_LOOP`, `FLOW_CONTINUE_OUTSIDE_LOOP`, `FLOW_INVALID_NONLOCAL` |
   | Match exhaustiveness | [statements.rs:799,837](../crates/sifr_hir/src/lower/statements.rs:799) | `MATCH_NON_EXHAUSTIVE`, `MATCH_GUARD_NOT_BOOL`, `MATCH_INVALID_CLASS_PATTERN_FIELD` |
   | Decimal literal / arithmetic | [decimal_methods.rs](../crates/sifr_hir/src/lower/decimal_methods.rs), [expressions.rs:875-1103](../crates/sifr_hir/src/lower/expressions.rs:875), [type_system/check.rs:31,43,361,373](../crates/sifr_type_system/src/check.rs:31) | `SIFR-DECIMAL-0001..0008` (all 8) |
   | Class init / definition | [classes.rs:145,234](../crates/sifr_hir/src/lower/classes.rs:145) | `CLASS_MISSING_INITIALIZER`, `CLASS_REQUIRED_FIELD_AFTER_DEFAULT`, `CLASS_DUPLICATE_OR_INVALID_VALUE`, `CLASS_MISSING_MEMBER` |
   | Iterator / protocol | [classes.rs:145](../crates/sifr_hir/src/lower/classes.rs:145) | `PROTO_BOUND_NOT_SATISFIED`, `PROTO_INVALID_ITERATOR_SIGNATURE`, `PROTO_CONTEXT_MANAGER_MISSING`, `PROTO_HASHABLE_OR_COMPARABLE_REQUIRED` |
   | Call args | [method_call_args.rs](../crates/sifr_hir/src/lower/method_call_args.rs), [builtin_calls.rs](../crates/sifr_hir/src/lower/builtin_calls.rs) | `CALL_WRONG_POSITIONAL_COUNT`, `CALL_UNEXPECTED_KEYWORD`, `CALL_DUPLICATE_ARGUMENT`, `CALL_MISSING_REQUIRED_ARGUMENT`, `CALL_NOT_CALLABLE_OR_ARITY` |
   | Result / raise | [statements.rs:189](../crates/sifr_hir/src/lower/statements.rs:189) | `RESULT_UNUSED_VALUE`, `RESULT_INVALID_ERROR_TYPE`, `RESULT_INVALID_RAISE` |
   | Imports (HIR-side) | [imports.rs](../crates/sifr_hir/src/lower/imports.rs) | `IMPORT_FORBIDDEN_INTRINSIC`, `IMPORT_UNKNOWN_SOURCE_MODULE` |
   | Tuple unpack shape | [tuple_unpack.rs:100](../crates/sifr_hir/src/lower/tuple_unpack.rs:100) | `TYPE_UNPACK_SHAPE_MISMATCH` |
   | Container element conflict | [container_literal_specialization.rs](../crates/sifr_hir/src/lower/container_literal_specialization.rs) | `TYPE_CONTAINER_ELEMENT_CONFLICT` |
   | Annotation shape errors | [typing_and_functions.rs](../crates/sifr_hir/src/lower/typing_and_functions.rs) | `TYPE_INVALID_ANNOTATION`, `TYPE_MISSING_ANNOTATION` |

2. **Never use `INTERNAL_COMPILER_PANIC` (`SIFR-INTERNAL-0001`) as the transitional bucket for user-facing semantic errors.** That code maps to `EXIT_INTERNAL_COMPILER_FAILURE` (3) at [main.rs:260-262](../crates/sifr/src/main.rs:260) — using it for, say, a type-mismatch site would change the user-visible exit code from 1 (user diagnostic) to 3 (ICE). The categorical mapping above never needs `INTERNAL_COMPILER_PANIC`; reserve that code for true compiler-invariant failures already handled in slice 1 (e.g. [build/entrypoint.rs:215-221](../crates/sifr_driver/src/build/entrypoint.rs:215)).

3. **When a site is genuinely ambiguous, prefer a deliberate registry extension over a label-laundering choice.** E.g., if a generic-bound site does not match `PROTO_BOUND_NOT_SATISFIED`, `TYPE_MISMATCH`, or any existing code, slice 2 should add a new registry entry with template, declared args, owner module, fixture plan, and docs page. Adding an entry is cheap; mis-bucketing is not.

4. **Decimal pseudo-codes `[E25xx]` stay in the message templates for this slice.** Their removal is owned by `milestone_diag_6` ([issue lines 919-935](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:919)). Slice 2's decimal migration only attaches `SIFR-DECIMAL-000x` as the top-level identity; the embedded `[E25xx]` stays inside the rendered message and the hybrid `[SIFR-TYPE-0001] [E25xx]` fixtures become `SIFR-DECIMAL-0001` (etc.) `+ [E25xx] still in the message text`. Deleting `[E25xx]` here would conflate two milestones.

## Question 3: which tests/baselines must be updated to prove the bridge is gone

Three categories must change inside this slice (regardless of how many sub-slices it is split into):

### a) Driver / CLI unit tests that hard-code `"SIFR-TYPE-0001"` — 23 occurrences

| File | Lines | Purpose |
| --- | --- | --- |
| [crates/sifr/src/main.rs](../crates/sifr/src/main.rs) | 1273, 1276, 1295, 1300, 1312, 1315, 1323, 1326, 1364, 1367, 1376, 1379, 1389-1392, 1401, 1404, 1439, 1441 (≈ 20) | Renderer/help/compact/severity tests construct synthetic `CompilerDiagnostic` literals with `code: "SIFR-TYPE-0001".to_string()` |
| [crates/sifr_driver/src/tests/diagnostics.rs](../crates/sifr_driver/src/tests/diagnostics.rs) | 88, 91, 119 | `apply_diagnostic_recovery_limits` synthetic input |

These are not exercising the bridge — they are exercising the renderer and recovery limiter on synthetic `CompilerDiagnostic` values. Re-keying them to an active code (the natural choice is `TYPE_MISMATCH` / `SIFR-TYPE-0002`) preserves intent. **One of them, however, must remain a regression guard for "the bridge is gone"** — see the new test in (d) below.

### b) Verification baselines — 2 files

- [crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt) — JSON `"code": "SIFR-TYPE-0001"` and `"url": ".../SIFR-TYPE-0001"` lines.
- [crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-compact.stderr.txt](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-compact.stderr.txt) — `error [SIFR-TYPE-0001] [main] [E2501] ...` line and the `url:` line.

Both should re-key to `SIFR-DECIMAL-0001` (`DECIMAL_INVALID_LITERAL`). The `[E2501]` substring stays in the message until `milestone_diag_6`.

### c) E2E `expect-error` annotations — ~90 fixtures

`grep -c "expect-error: SIFR-TYPE-0001" crates/sifr/tests/e2e/fail/*.sifr` returns **76 files** for the bare form, with another ~14 fixtures using the hybrid `[SIFR-TYPE-0001] [E25xx]` form, for ~90 total. Each must re-key to its category-correct active code. Re-keying is mechanical *only when the call site has been migrated and now emits the right code*; otherwise the fixture will fail with a code-mismatch.

A practical sequencing constraint: a fixture must not be re-keyed *before* the corresponding call site is migrated, and a call site must not be migrated *after* its fixture has been re-keyed (in either order, the fixture briefly fails). This is the single biggest argument for sub-slicing by domain (decimal first, then ownership, then match, etc.) so each sub-PR's call-site migration and fixture re-key land together.

### d) New regression test — the "bridge is gone" proof

To pin the slice's headline outcome, add one of:

- **Compile-time proof (preferred):** delete the `CompilePhase::TypeCheck` arm from `diagnostic_code()` and either delete the `CompilePhase::TypeCheck` enum variant or make `diagnostic_code()` infallible by removing the `Option<DiagnosticCode>` field. After slice 2c, no production path can construct a `CompileError` with `code: None` for the TypeCheck phase because the enum/variant no longer exists or because the field is no longer optional. This is a structural guarantee, not a runtime one. The slice 2c PR description should call out the type-system enforcement explicitly.
- **Runtime regression test (acceptable fallback):** if the structural change is deferred to slice 4b, add a unit test in `crates/sifr_driver/src/tests/diagnostics.rs` that asserts `compile_errors_to_diagnostics` never produces `"SIFR-TYPE-0001"` from any input — e.g., by enumerating `CompilePhase` variants and a few representative messages and asserting the produced JSON `code` is not the retired string. Pair with a registry probe that asserts `DiagnosticCode::TYPE_MISMATCH.code() == "SIFR-TYPE-0002"` to anchor the canonical replacement.

The issue's [diag_5 line 919](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:919) wants harness-level code-validation to reject retired codes at fixture-load time. That guardrail is **owned by `milestone_diag_5`**, not slice 2. Slice 2 should not re-implement it; the existing `is_diagnostic_code` validator at [crates/sifr/tests/e2e.rs:637-652](../crates/sifr/tests/e2e.rs:637) accepts any `SIFR-X-NNNN` form, so a typo in a re-keyed fixture would be silently accepted today. Mitigation for slice 2: re-key fixtures by exact code-substitution scripted with `git grep` → `sed`, then run the full e2e suite locally; CI failure on a wrong code is the regression guard until `milestone_diag_5` lands.

## Recommended sequencing — split slice 2 into three sub-slices

Each sub-slice is an independently reviewable PR with a clear before/after.

### Slice 2a — Transport plumbing (small, pure additive)

Goal: HIR can carry codes; nothing else changes.

1. Add `code: Option<DiagnosticCode>` field to `LoweringError` at [lower/mod.rs:81-87](../crates/sifr_hir/src/lower/mod.rs:81). Default-initialize to `None`.
2. Add `LowerCtx::error_with_code(&mut self, code: DiagnosticCode, message: String)` at [lower/mod.rs:209](../crates/sifr_hir/src/lower/mod.rs:209) alongside the existing `error(message)`. The legacy method stays for now and produces `LoweringError { code: None, .. }`.
3. Update the three conversion sites to forward `error.code` faithfully:
   - [frontend/module_lowering.rs:25-39](../crates/sifr_driver/src/frontend/module_lowering.rs:25): branch on `e.code` — call `CompileError::with_code` when present, otherwise the existing `CompileError::new` path.
   - [test_runner/orchestrator.rs:107-118](../crates/sifr_driver/src/test_runner/orchestrator.rs:107): same branch — the `code: error.code` forward becomes live for whichever HIR sites already migrated.
   - [stdlib/bootstrap.rs:58-73](../crates/sifr_driver/src/stdlib/bootstrap.rs:58): keep the `STDLIB_BOOTSTRAP_FAILURE` override (intentional: a stdlib parse/lower failure is a bootstrap failure, not a user-facing semantic error). If `e.code` is set, prefer it over `STDLIB_BOOTSTRAP_FAILURE`? Decide explicitly in the PR — the safer default is to **keep** `STDLIB_BOOTSTRAP_FAILURE` because stdlib lowering errors are compiler bugs from a user's perspective, but the choice should be documented in a one-line comment.
4. Add a unit test in `sifr_hir` proving `LoweringError { code: Some(SIFR-TYPE-0002), .. }` round-trips to a `CompileError::with_code`, and a sibling test in `sifr_driver` proving `compile_errors_to_diagnostics` emits the active code (not `SIFR-TYPE-0001`) when `error.code` is `Some(_)`.
5. Keep the `TypeCheck => SIFR-TYPE-0001` arm in `diagnostic_code()` untouched. Keep all 90 fixtures untouched. Keep the 23 driver/CLI unit tests untouched. *Nothing else changes.*

Validation: `cargo test -p sifr_hir`, `cargo test -p sifr_driver`, `scripts/run_all_tests.sh --profile quick`. Quick-profile signature should still be `e1bf653aaa770517` because no production behavior changed.

### Slice 2b — Mechanical call-site migration, by domain (multiple PRs)

Goal: every `LowerCtx::error` site migrates to `LowerCtx::error_with_code`. Migrate by category so each sub-PR's call sites and fixtures land together.

Ordering recommendation (smallest blast radius first, hardest categorical decisions last):

1. **Decimal** — 18 + ~12 sites, ~14 fixtures, 2 baselines. Cleanest because the hybrid `[SIFR-TYPE-0001] [E25xx]` annotations already gesture at the right `SIFR-DECIMAL-000x` code. The mapping `[E2501]→0001`, `[E2502]→0002`, `[E2503]→0003`, `[E2504]→0004`, `[E2505]→0005`, `[E2506]→0006`, `[E2507]→0007`, `[E2508]→0008` is essentially documented in the inventory. **Do not** strip `[E25xx]` from message templates — that's `milestone_diag_6`.
2. **Ownership / Flow / Result / Match** — ~20-30 sites total, ~15 fixtures. These categories are clean: `OWN_*` codes are precise, `FLOW_*` are precise, `RESULT_*` are precise, `MATCH_*` are precise.
3. **Class / Protocol / Import** — ~25 sites, ~15 fixtures. Slightly more nuance because some class errors also look like protocol errors; the registry distinguishes them by the rule, not by the file.
4. **Call args / Tuple unpack / Container literal / Annotation shape** — ~40 sites, ~15 fixtures. Mostly clean; `CALL_*` codes already split the rule space.
5. **Type / Name** — the largest residual: ~250 sites in `expressions.rs` + `statements.rs` + `aug_assign_lowering.rs` + `typing_and_functions.rs` + `mod.rs`, ~30 fixtures. This is where label-laundering risk peaks. Consider a paired exploratory pass that reads each call site and selects between `TYPE_MISMATCH`, `TYPE_IF_BRANCH_MISMATCH`, `TYPE_UNSUPPORTED_OPERATOR`, `TYPE_INT_BIGINT_MIXED`, `TYPE_CONTAINER_ELEMENT_CONFLICT`, `TYPE_UNPACK_SHAPE_MISMATCH`, `NAME_*`, etc., before the migration PR opens.

Each 2b sub-PR's validation includes the full e2e suite (`scripts/run_e2e_pass.sh`) because re-keyed fixtures shift the JSON/compact baselines. Quick-profile alone is insufficient — the slice 1 quick-profile signature was stable precisely because no fixture was re-keyed.

### Slice 2c — Bridge deletion + structural cleanup (small, type-driven)

After every 2b sub-PR has merged and `git grep "ctx.error(" crates/sifr_hir/src/` returns zero hits:

1. **Delete `LowerCtx::error(message: String)`** at [lower/mod.rs:209-215](../crates/sifr_hir/src/lower/mod.rs:209). Any forgotten call site becomes a compile error. This is the type-system enforcement of "must use an inventory-assigned canonical code … or fail to compile" from [issue line 863](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:863).
2. **Delete `CompilePhase::TypeCheck => "SIFR-TYPE-0001"`** at [diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137). Decide between the two structural cleanups:
   - Option A: keep the `Option<DiagnosticCode>` field but make `diagnostic_code()` panic on `None` for `TypeCheck` (or for any phase) — runtime enforcement.
   - Option B: tighten `CompileError.code` to non-`Option`, force every constructor to take a `DiagnosticCode`, and delete `CompileError::new` — type-system enforcement. This is the issue's preferred direction (see [issue line 492-497](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:492) for the eventual `CompileError` retirement plan in `milestone_diag_4b`).
   - Option B is strictly better. Slice 2c should pick it unless the parser/codegen/build paths still emit `CompileError::new(...)` (some do — see [frontend/api.rs:21-44](../crates/sifr_driver/src/frontend/api.rs:21) and the parse-failure markers from slice 1's R5). If those parse paths are not yet coded, slice 2c should code them too (one-line per site, all parse failures → `PARSE_EXPECTED_TOKEN_OR_RECOVERY` until `milestone_diag_7` splits them) and proceed with Option B.
3. **Update the 23 driver/CLI unit-test occurrences** to use active codes. The natural substitution is `"SIFR-TYPE-0001"` → `"SIFR-TYPE-0002"` (`TYPE_MISMATCH`); the tests are exercising the renderer/recovery-limiter on synthetic input and don't care which active code they use.
4. **Re-key the 2 verification baselines** (decimal fixtures) — these were already re-keyed in slice 2b.1 if the sub-slicing landed; if not, do them here.
5. **Add a regression test** asserting `CompileError::new(...).to_diagnostic()` either fails to compile (Option B) or produces a non-retired code (Option A). Worded as a unit test in `crates/sifr_driver/src/tests/diagnostics.rs`, e.g., enumerate all `DiagnosticCode` constants and assert none equal `"SIFR-TYPE-0001"` AND no production path produces it.
6. **Update the slice-1 transitional comment** at [diagnostics.rs:129-134](../crates/sifr_driver/src/diagnostics.rs:129) — either delete it or rewrite it to describe the remaining `Parse → SIFR-PARSE-0001`, `Codegen → SIFR-CODEGEN-0001`, `Build → SIFR-BUILD-0001` arms (which are explicit `milestone_diag_4b` targets). Slice 1 review pass-3 already noted these arms should be repointed to active codes; slice 2c can repoint them as a free side-effect.

The slice 2c PR is small (≈ 10-20 file diff) but is the only one that visibly satisfies the issue's milestone exit gate at [issue line 882](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:882): "`CompilePhase::TypeCheck` no longer assigns `SIFR-TYPE-0001` to any diagnostic path."

## Single-PR fallback ordering (if the implementer rejects sub-slicing)

If slice 2 must ship as one PR, the safe ordering inside the PR is:

1. (2a steps 1-3) Add `code` to `LoweringError`, add `error_with_code`, update the 3 conversion sites.
2. (2b) Migrate all 489 call sites in one batch, ordered by domain (decimal → ownership/flow/match/result → class/proto/import → call/tuple/container/annotation → type/name).
3. Re-key all ~90 fixtures and 2 baselines to active codes in the same commit as the corresponding call-site migration to keep the build green per-commit.
4. Update the 23 driver/CLI unit-test occurrences.
5. Delete `LowerCtx::error` (the codeless overload).
6. Delete `CompilePhase::TypeCheck => "SIFR-TYPE-0001"`. Decide between Option A/B from slice 2c step 2.
7. Add the regression test from slice 2c step 5.
8. Run `scripts/run_all_tests.sh --profile quick` AND `scripts/run_e2e_pass.sh` for full coverage.

A single PR with this ordering is *correct* but is ~100 files of diff, which is reviewable only with a strong commit decomposition. I would still recommend sub-slicing.

## Scope traps to defer

These will be tempting to bundle into slice 2 and should be pushed out instead.

1. **Decimal `[E25xx]` removal from message templates** — 36 sites in HIR + type system. Owned by `milestone_diag_6` ([issue line 925-935](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:925)). Slice 2 attaches `SIFR-DECIMAL-000x` as the *top-level* identity but keeps `[E25xx]` in the message body so the human-readable output is unchanged. Do not pre-empt diag_6.
2. **`sifr_type_system::TypeError` deletion** — owned by `milestone_diag_7` ([issue line 958](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:958)). Type-system errors that flow through the HIR pipeline currently land as untyped strings in `LoweringError`; slice 2 must attach codes at the HIR boundary (probably `TYPE_MISMATCH` for the broad cases), but must not delete `TypeError`/`TypeErrorKind` symbols or restructure the type-system error model. The short-lived `From<TypeError> for SifrDiagnostic`-style adapter that diag_7 will delete is **not** in scope.
3. **`LoweringError → LoweringOutcome/DiagnosticSink` migration** — also part of `milestone_diag_4a` ([issue line 866](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:866)) but is a separate transport replacement that the issue lists as PR #2 in the milestone's expected PR sequence ([issue line 871](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:871)). Slice 2 only adds a `code` field to the *existing* `LoweringError`. The full sink/outcome rework should be a separate sub-slice (call it 2d) after 2c lands. Do not bundle.
4. **Parser bucket splitting (`SIFR-PARSE-0002..0009`)** — Ruff parse failures all funnel into `PARSE_EXPECTED_TOKEN_OR_RECOVERY` today (slice 1 R5, deferred). The issue's [milestone_diag_7 line 941-943](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:941) is the right home. Slice 2c may opportunistically repoint the `Parse → SIFR-PARSE-0001` bridge arm (one line) but should *not* attempt to split parse buckets.
5. **`is_internal_compile_error` simplification** — slice 1 review pass-3 O4 suggested generalizing the equality check to `code.starts_with("SIFR-INTERNAL-")`. Out of scope for slice 2; it's a `milestone_diag_4b` concern.
6. **Legacy CLI renderer (`compile_errors_to_diagnostics` + `render_compile_errors`)** — unchanged in slice 1. The renderer wiring slice should consume `SifrDiagnostic` directly; slice 2 should not entangle with it. The slice 1 review pass-3 N6 already flagged that the legacy renderer's `code.starts_with("SIFR-PARSE-")` severity-class sniffer ([main.rs:374-385](../crates/sifr/src/main.rs:374)) should not gain new code-prefix arms. Honor that.
7. **`milestone_diag_5` harness validation of retired/unknown expectation codes** — owned by diag_5 ([issue line 895](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:895)). Slice 2 fixture re-keying will silently succeed on typos until then, so re-key carefully and run the full e2e suite locally.
8. **Bounded multi-error recovery cap activation** — `milestone_diag_10`. The fixtures `bounded_multi_error_recovery.sifr` and `bounded_multi_error_recovery_repeated_type_errors.sifr` re-key to specific active codes in slice 2 but the cap-omission summary code (`SIFR-INTERNAL-0002` reserved) is not activated here.
9. **Any non-`SIFR-TYPE` bridge arm repoint** — the `Parse`, `Codegen`, `Build` arms in [diagnostics.rs:135-140](../crates/sifr_driver/src/diagnostics.rs:135) all still emit retired codes. Slice 2c may repoint them opportunistically because the implementer is in the file anyway, but the *milestone* exit gate ([issue line 999](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:999) "delete the remaining phase-derived public diagnostic-code mapping") is owned by `milestone_diag_4b`. If slice 2c repoints them, it should not also delete the mapping function.

## Concrete pre-implementation actions before slice 2a opens

1. **Confirm the `LoweringError.code` decision** — `Option<DiagnosticCode>` for slice 2a (so the existing 489 `ctx.error(...)` sites still type-check) and tighten to non-optional in slice 2c. The alternative — non-optional from day 1 — is a cleaner end state but turns slice 2a into a 489-file PR by itself.
2. **Decide stdlib bootstrap forwarding policy** — does `STDLIB_BOOTSTRAP_FAILURE` always win at [stdlib/bootstrap.rs:58-73](../crates/sifr_driver/src/stdlib/bootstrap.rs:58), or should an HIR-emitted code override it for stdlib parse/lower errors? Default to "always wins" (stdlib parse/lower failures are bootstrap failures); document the choice in a one-line comment.
3. **Pre-categorize the 489 sites into the 12 buckets above** before opening the slice 2b PRs. A single audit pass over `crates/sifr_hir/src/lower/` produces the (file, line, suggested code) list and avoids a second pass during code review. The result is an artifact (e.g. `internal_docs/diagnostic_call_site_categorization.md`) the diag_7/diag_8 milestones will reuse to add helpers.
4. **Prepare the registry extension list** — any call site whose intent doesn't fit the 52 active codes needs a registry entry added in `crates/sifr_diagnostics/src/codes.rs` (with template, declared args, owner, fixture plan, docs page). Bundle the registry extensions into slice 2a or a dedicated slice 2-prep PR rather than scattering them across the 2b sub-PRs.
5. **Audit the 90 fixtures' message bodies** — re-keying will only succeed if the new active code's message template matches the fixture's expected message. Slice 2 must not change message templates (that's diag_7 helper refinement), so any fixture whose message body doesn't match the active code's template needs either (a) the active code's template to be a superset of the message, or (b) the fixture's message to be left as the rendered string (the registry's `message_template` is not asserted by `expect-error`, only the code is). The message-body audit may surface fixtures that currently embed `[E25xx]` decimal pseudo-codes — those stay as-is.
6. **Plan the regression test for slice 2c** — pick Option A (runtime) or Option B (type-system) from §"Question 3 (d)" before slice 2c opens, so the test fixture is ready when the bridge arm is deleted.

## Validation budget per sub-slice

| Sub-slice | Required validation | Typical signature |
| --- | --- | --- |
| 2a (transport plumbing only) | `cargo test -p sifr_hir`, `cargo test -p sifr_driver`, `cargo clippy --workspace -- -D warnings`, `scripts/run_all_tests.sh --profile quick` | Quick-profile should match `e1bf653aaa770517` (no behavior change) |
| 2b.* (per-domain migration + fixture re-key) | Full e2e suite: `scripts/run_e2e_pass.sh`, `scripts/run_all_tests.sh` (full profile), schema/docs sync scripts | Quick-profile signature **will change** because re-keyed fixtures shift JSON/compact output |
| 2c (bridge deletion + cleanup) | Full e2e suite, `scripts/run_all_tests.sh`, plus a manual `git grep "SIFR-TYPE-0001" crates/ verification/ docs/` returning zero hits in production paths (registry retirement entry remains) | Final signature is the slice 2 baseline carried into `milestone_diag_4b` |

## Summary

Slice 2 as scoped is the right next step in `milestone_diag_4a`, but its actual surface — 489 HIR call sites, 90 fixtures, 23 unit-test occurrences, 2 baselines — makes it the single largest patch in the entire diag phase if landed as one PR. The user's framing as a "mechanical transport slice" is accurate per call site but not per slice: the *transport* is mechanical, the *call-site coverage* is not.

The recommended path is the three-sub-slice split (2a transport plumbing, 2b per-domain migration in 4-5 PRs, 2c structural cleanup + bridge deletion). A centralized message-prefix dispatcher must not be reintroduced; the type-system enforcement (delete the codeless `LowerCtx::error`, optionally tighten `CompileError.code` to non-`Option`) is the inventory-faithful way to land the bridge deletion. Active codes per category are listed in §"Question 2"; `INTERNAL_COMPILER_PANIC` is **not** an acceptable transitional bucket for user-facing semantic errors. The fixture re-keying is the highest-risk subtask because the harness's `is_diagnostic_code` validator silently accepts retired codes (slice 5 territory); careful per-domain landing with full e2e validation per sub-PR is the mitigation.

Defer in this slice: decimal `[E25xx]` message-template cleanup (diag_6), `sifr_type_system::TypeError` deletion (diag_7), `LoweringError → LoweringOutcome/DiagnosticSink` migration (separate slice 2d), parser-bucket splitting (diag_7), `is_internal_compile_error` generalization (diag_4b), legacy CLI renderer migration (later renderer-wiring slice), and the harness expectation-grammar validation (diag_5).
