# Review: milestone_diag_3 — Diagnostic Emission Inventory (Pass 2)

Branch: `codex/semantic-diagnostics-diag-3`
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Pass-1 review: [reviews/semantic-diagnostic-code-taxonomy-diag-3-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-3-review-pass-1.md)
Validation evidence reported: `scripts/run_all_tests.sh --profile quick` (signature `e1bf653aaa770517`).

## Scope reviewed

- Updated [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md).
- Phase tracker delta in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) (status flip `2a → 3`, three new checked DoD bullets, two unchecked, validation note).

milestone_diag_3 remains paper-only; this pass re-checks the four pass-1 blockers (B1–B4), the non-blocking accuracy items (N1–N6, M1–M4), and walks through the new sections to look for net-new gaps.

## Verdict

All four pass-1 blockers are resolved. The inventory now contains a Verification Baseline Surface section (B1), a Parser Surface section that maps Ruff-fork variants to eight proposed `SIFR-PARSE-000x` codes (B2), an "Unannotated fail fixtures" subsection that groups all 88 unannotated fixtures with target families plus the full filename list (B3), and a Recovery Expectations By Category table covering 14 family groups (B4).

All six non-blocking accuracy items (N1–N6) and all four minor items (M1–M4) from pass 1 are also addressed: the `CompileError` count is corrected to 47 (43 production + 4 test) with the specific four file rows fixed, the four HIR family rows are widened to match what their files actually emit, ctx.warn / reveal_type / catch_unwind are inventoried as a new "Non-Error Emission Paths" section, the CLI test `CompilerDiagnostic` constructions are enumerated, the marker-count heading is renamed to fail-fixture-and-harness-sample, and the two stale "current code mechanism" details are surfaced as a separate "Current public-code mechanisms to remove" table.

I found no new blocking issues. There are four small residual nits that are worth recording but do not gate the milestone — they're listed under "Residual nits" below and can be folded into `milestone_diag_2b`/`milestone_diag_7` when those milestones touch the same rows.

Recommendation: proceed to merge.

## Definition-of-done coverage

| DoD bullet | Status | Evidence / notes |
| --- | --- | --- |
| Inventory covers all raw HIR `ctx.error(...)` call sites | ✅ | Per-file counts in HIR Lowering Surface table sum to 489 across 22 files; reproduces under `rg "ctx\.error\(" crates/sifr_hir/src -g '*.rs' -c`. |
| Inventory covers all `CompileError` construction paths | ✅ | All 16 emitting files appear; headline corrected to 47 actual constructions (43 production + 4 test); the four pass-1 inflated rows (`diagnostics.rs` 4→1, `materialize.rs` 2→1, `workspace/mod.rs` 4→2, `discovery.rs` 7→6) are now accurate. |
| Inventory covers all `sifr_type_system::TypeError` and `TypeErrorKind` variants | ✅ | All 8 `TypeErrorKind` variants present; 24 construction sites in `sifr_type_system/src/check.rs` reproduce. |
| Inventory covers e2e expectation parsing **and verification baselines** | ✅ | New Verification Baseline Surface subsection enumerates all nine `crates/sifr/tests/verification/` cases (decimal_invalid_literal, missing_import_reports_error, multi_module_run, workspace_ambiguous_import, workspace_dotted_helper_run, workspace_malformed_manifest, workspace_unresolved_import, CR-0001, CR-0002) with current markers and migration owners. |
| No diagnostic category migrated without a known target code and fixture plan | ✅ | Target Code And Fixture Plan covers all migration families with representative fixtures or explicit "fixture pending in `milestone_diag_2b`/`milestone_diag_7`" notes; the 88 unannotated fail-fixture set is grouped by family with target codes; parser categories are no longer collapsed into one line. |

## Pass-1 finding follow-up

### Blocking (B1–B4) — all resolved

#### B1. Verification baselines now inventoried — RESOLVED

The new "Verification Baseline Surface" section (lines 257–269 of the inventory) enumerates each verification subdirectory, its current baseline markers, and the target/owner. Confirmed against the working tree:

```bash
ls crates/sifr/tests/verification/{crashes,diagnostics,project}
# crashes: CR-0001_cfg_invariant_minimized.sifr  CR-0002_parser_invariant_minimized.sifr
# diagnostics: decimal_invalid_literal
# project: missing_import_reports_error  multi_module_run  workspace_ambiguous_import
#          workspace_dotted_helper_run  workspace_malformed_manifest  workspace_unresolved_import
```

All seven `project/*` cases plus `diagnostics/decimal_invalid_literal` plus the two crash cases are listed. The migration-owner column correctly distinguishes "regenerate in decimal migration" (decimal_invalid_literal) from "renderer integration regenerates schema shape only" (the workspace cases that keep their codes) from "no diagnostic migration, but rerun with renderer tests" (multi_module_run, workspace_dotted_helper_run).

#### B2. Parser surface decomposed — RESOLVED

The new "Parser Surface" section (lines 47–62) and the parser block in the Target Code And Fixture Plan (lines 289–296) jointly propose eight active `SIFR-PARSE-000x` codes (`0002..0009`) plus retired `0001`, mapped to specific Ruff-fork error variants.

Spot-check against `third_party/ruff/crates/ruff_python_parser/src/error.rs`:

- `0003` lexical/interpolated: `Lexical(LexicalErrorType)` (line 202), `FStringError(InterpolatedStringErrorType)` (line 198), `TStringError(InterpolatedStringErrorType)` (line 200) — all present. ✓
- `0004` indentation/layout: `UnexpectedIndentation` (line 189), `SimpleStatementsOnSameLine` (line 170), `SimpleAndCompoundStatementOnSameLine` (line 172) — all present. ✓
- `0005` invalid targets: `InvalidAssignmentTarget` (150), `InvalidNamedAssignmentTarget` (152), `InvalidAnnotatedAssignmentTarget` (154), `InvalidAugmentedAssignmentTarget` (156), `InvalidDeleteTarget` (158), `InvalidStarredExpressionUsage` (135) — all present. ✓
- `0006` invalid call argument order: `PositionalAfterKeywordArgument` (161), `PositionalAfterKeywordUnpacking` (163), `InvalidArgumentUnpackingOrder` (165), `DuplicateKeywordArgumentError(String)` (147) — all present. ✓
- `0007` empty/malformed declaration lists: `EmptyImportNames` (119), `EmptyGlobalNames` (113), `EmptyNonlocalNames` (115), `EmptyTypeParams` (121), parameter-order = `ParamAfterVarKeywordParam`/`NonDefaultParamAfterDefaultParam`/`VarParameterWithDefault` (140/142/144) — all present. ✓
- `0008` invalid match/pattern: `InvalidStarPatternUsage` (137); other mapping/class pattern errors flow through `OtherError`/`ExpectedToken` recovery, which is acceptable as documented. ✓
- `0009` unsupported syntax: `UnsupportedSyntaxErrorKind`, `UnexpectedIpythonEscapeCommand` (193), `UnexpectedTokenAfterAsync(TokenKind)` (191) — present. ✓

The decomposition is sufficient for `milestone_diag_2b` registry population and gives `milestone_diag_7` a concrete parser-bucket worklist instead of a single catch-all.

#### B3. 88 unannotated fail fixtures inventoried — RESOLVED

The new "Unannotated fail fixtures" block (lines 154–255) groups fixtures into five family categories (stdlib unsupported APIs; bytes and binary/text I/O; ownership/mutability; collection/container shape; type alias/recursive typing) with target family/code plans, followed by the full 88-file enumeration as a code block.

I verified by reproducing the set:

```bash
comm -23 <(ls crates/sifr/tests/e2e/fail/ | sort) \
         <(rg -l "# expect-error" crates/sifr/tests/e2e/fail/ -g '*.sifr' | xargs -n1 basename | sort) | wc -l
# 88
```

The 88 filenames in the inventory match the diff output exactly (verified line-by-line). The grouping is internally consistent — each filename pattern in the table prefix list (`argparse_*`, `bytes_*`, `borrowed_*`, `dict_*`, `recursive_*`, etc.) maps to a covering family code in the Target Code And Fixture Plan.

#### B4. Per-category recovery expectations recorded — RESOLVED

The new "Recovery Expectations By Category" table (lines 357–374) covers 14 category groups: `PARSE-*`, `NAME-0001..0004`, `IMPORT-*/WORKSPACE-*`, `TYPE-0002..0009`, `DECIMAL-0001..0008`, `CALL-0001..0005`, `OWN-0001..0004`, `FLOW-*`, `MATCH-*`, `PROTO-*`, `CLASS-*`, `RESULT-*`, `STDLIB-*`, `CODEGEN-*/BUILD-*/INTERNAL-*`, plus the warning/note codes `TYPE-0901`/`FLOW-0901`/`TYPE-0902`. Each row carries a recovery expectation and a dedupe-key sketch. The expectations are concrete enough to validate against `milestone_diag_10`'s implementation (e.g. `OWN-*` taints binding state; `MATCH-*` non-exhaustiveness emits once per match expression; `CALL-*` is non-tainting for the callable but taints the call result; reveal-type notes participate in the 50 top-level cap).

### Non-blocking (N1–N6) — all resolved

- **N1.** `CompileError` headline is now `47 sites: 43 production CompileError { ... } literals plus 4 test-only` (line 8), and the per-file row counts for the four affected files (`diagnostics.rs:1`, `materialize.rs:1`, `workspace/mod.rs:2`, `discovery.rs:6`) are now actual-construction counts, not raw regex matches. Sum verifies: `1+2+1+6+1+1+3+1+7+4+1+2+8+2+3 = 43` production. ✓
- **N2.** HIR family assignments are widened: `decimal_methods.rs` now lists `DECIMAL, CALL, NAME` (covers "no method", "takes no arguments", method arity); `classes.rs` now lists `CLASS, TYPE, NAME, PROTO` (covers iter/next/reversed protocol shape); `mod.rs` row WORKSPACE entry is annotated "wrong-layer `WORKSPACE` moves to driver"; `typing_and_functions.rs` now includes `CALL`. ✓
- **N3.** A new "Non-Error Emission Paths" section (lines 271–281) covers ctx.warn arithmetic (5 sites in `arithmetic_warnings.rs` → `SIFR-TYPE-0901`), ctx.warn unreachable (1 site in `statements.rs` → `SIFR-FLOW-0901`), ctx.warn exhaustive-return panic recovery (1 in `typing_and_functions.rs` → wrong-layer `SIFR-INTERNAL-0001`), `ctx.reveal_types` (→ `SIFR-TYPE-0902` note), and HIR-internal `catch_unwind` (wrong-layer). ✓
- **N4.** A new CLI Test Surface table (lines 109–112) lists `crates/sifr/src/main.rs` as 9 sites and `crates/sifr_driver/src/tests/diagnostics.rs` as 2 sites, both owned by `milestone_diag_5`. ✓
- **N5.** Marker-count table heading is now "Current fail-fixture and harness-sample code markers:" (line 139). ✓
- **N6.** A new "Current public-code mechanisms to remove" table (lines 114–122) calls out phase-derived `CompilePhase` mapping, the workspace prefix classifier, type-error string forwarding, message-embedded pseudo-code, and test-only hard-coded diagnostics with their replacement strategy. ✓

### Minor (M1–M4) — all resolved

- **M1.** `typing_and_functions.rs` row now includes `CALL`. ✓
- **M2.** Fixture-pending phrasing is now consistent: most rows use "fixture pending in `milestone_diag_2b`" or "fixture pending in `milestone_diag_7`" with a target milestone explicit. ✓
- **M3.** Resolved together with N5. ✓
- **M4.** `SIFR-CLASS-0001` fixture is now pinned to `crates/sifr/tests/e2e/fail/auto_init_inheritance_missing_super.sifr` and `SIFR-CLASS-0002` to `auto_init_required_after_default.sifr`. ✓

## Number reproductions

All headline numbers in the inventory reproduce against the working tree:

```bash
rg "ctx\.error\(" crates/sifr_hir/src -g '*.rs' | wc -l                        # 489 ✓
rg "ctx\.error\(" crates/sifr_hir/src -g '*.rs' -c                              # per-file matches HIR table exactly ✓
rg "TypeErrorKind::" crates/sifr_type_system/src crates/sifr_hir/src -g '*.rs' -c
                                                                                # check.rs: 24 ✓
rg "CompileError \{" crates/sifr_driver/src crates/sifr/src -g '*.rs' | wc -l   # 54 raw; inventory says 47 actual constructions
rg "# expect-error" crates/sifr/tests/e2e/fail crates/sifr/tests/e2e.rs -g '*.sifr' -g '*.rs' | wc -l
                                                                                # 100 = 92 fixture + 8 harness ✓
ls crates/sifr/tests/e2e/fail/ | wc -l                                          # 179 total fixtures
# 179 - 91 annotated = 88 unannotated, matches inventory ✓
```

Marker totals also reproduce: `[E2501] 1`, `[E2502] 2`, `[E2503] 1`, `[E2504] 2`, `[E2505] 3`, `[E2506] 2`, `[E2507] 5`, `[E2508] 2` against grep across `crates/sifr/tests/e2e/fail/` plus `crates/sifr/tests/e2e.rs`. ✓

## Residual nits — non-blocking, optional follow-up

These are small accuracy points I noticed while re-reading the new sections. None gates the milestone, but each is a one-line edit that could be folded into a `milestone_diag_2b` or `milestone_diag_7` PR when those milestones touch the same row.

1. **`reveal_type` propagation row is slightly off-source.** The Non-Error Emission Paths row says `reveal_type(...)` is in `lower/builtin_calls.rs`; guarded-index reveal propagation in `lower/guarded_index.rs`. The only emission site is [crates/sifr_hir/src/lower/builtin_calls.rs:744](../crates/sifr_hir/src/lower/builtin_calls.rs:744) (`lower_reveal_type_call` → `ctx.reveal_types.push(...)`). The references in [crates/sifr_hir/src/lower/guarded_index.rs](../crates/sifr_hir/src/lower/guarded_index.rs) are tests of reveal-type behavior, not separate emission paths — `guarded_index.rs` does narrow types so reveal_type shows narrowed forms, but it does not emit reveal_type diagnostics itself. Either drop the `guarded_index.rs` clause or rephrase as "type narrowing in `lower/guarded_index.rs` affects what `reveal_type` displays".

2. **`expressions.rs` "unsupported expression/operator features" not pinned to a code.** The description column for the `expressions.rs` HIR row now mentions "unsupported expression/operator features", which captures the four leftover sites — `ctx.error("unsupported expression type")` ([expressions.rs:95](../crates/sifr_hir/src/lower/expressions.rs:95)), `"matrix multiplication operator (@) is not supported"` ([expressions.rs:337](../crates/sifr_hir/src/lower/expressions.rs:337) and [aug_assign_lowering.rs:28](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:28)), `"unsupported comparison operator"` ([expressions.rs:476](../crates/sifr_hir/src/lower/expressions.rs:476)), and `"walrus operator target must be a simple name"` ([expressions.rs:3780](../crates/sifr_hir/src/lower/expressions.rs:3780)). The Target Code And Fixture Plan lists `SIFR-TYPE-0005` for "unsupported operator or operand types", which seems to cover the operator cases. The "unsupported expression type" fallback at [expressions.rs:95](../crates/sifr_hir/src/lower/expressions.rs:95) is more wrong-layer-shaped (it fires on any AST kind the lowerer hasn't implemented, e.g. lambdas, dict comprehensions if unsupported) and might belong under `SIFR-INTERNAL-*`. Pinning each of the four sites to a target code in `milestone_diag_2b` would remove the only ambiguity in the HIR target-family columns.

3. **Several Ruff parser variants are not slotted.** The eight proposed `SIFR-PARSE-000x` buckets accept the column header "Ruff category / examples", but a non-trivial set of variants in [third_party/ruff/crates/ruff_python_parser/src/error.rs](../third_party/ruff/crates/ruff_python_parser/src/error.rs) is not explicitly mentioned in any bucket: `EmptySlice`, `EmptyDeleteTargets`, `IterableUnpackingInComprehension`, `UnparenthesizedNamedExpression`, `UnparenthesizedTupleExpression`, `UnparenthesizedGeneratorExpression`, `InvalidLambdaExpressionUsage`, `InvalidYieldExpressionUsage`, `ExpectedKeywordParam`, `ExpectedRealNumber`, `ExpectedImaginaryNumber`. These plausibly land in `0002` (generic recovery) or `0005` (invalid target) but `milestone_diag_7` will need a slotting decision. A one-line "remaining variants slot into `0002` by default" caveat in the parser table would prevent that decision from getting deferred forever.

4. **Pass-1 said "8 ctx.warn sites including `lower/mod.rs`", actual is 7.** The inventory's per-row counts (`5 + 1 + 1 = 7`) are accurate; pass-1 was slightly off. No edit needed in the inventory; recording here for the record.

## What is solid

- HIR `ctx.error(...)` per-file count table is exact, complete, and reproducible (489 across 22 files).
- All 8 `TypeErrorKind` variants are inventoried with target codes.
- Family assignments at the code level (target codes in the plan) match the issue's identity policy: `SIFR-NAME-0001..0004`, `SIFR-CALL-0001..0005`, `SIFR-OWN-0001..0004`, `SIFR-DECIMAL-0001..0008`, `SIFR-FLOW-0001..0003`, `SIFR-MATCH-0001..0003`, `SIFR-PROTO-0001..0004`, `SIFR-CLASS-0001..0004`, `SIFR-RESULT-0001..0003`.
- All inventoried fixture paths spot-checked exist on disk.
- Wrong-Layer notes cover the three real wrong-layer cases (import resolution, decimal pseudo-codes, builtin/stdlib helper split) and now also flag the HIR `catch_unwind` boundary.
- The new "Current public-code mechanisms to remove" table is the right level of decisiveness for `milestone_diag_4a` — each row names a concrete deletion target.
- The `SIFR-PARSE-0001` row is correctly recorded as "retired legacy phase bucket" with no active fixture, matching the issue's identity policy.
- The Workspace code review preserves `0001..0004` and `0101..0103` and proposes `0104` for project import-cycle, consistent with `milestone_diag_2b`'s mandate.

## Recommendation

Merge as-is. The four pass-1 blockers are resolved with new, content-bearing inventory sections; all six pass-1 non-blocking items are resolved; all four pass-1 minors are resolved. The four residual nits above are optional follow-up that fit naturally into `milestone_diag_2b` or `milestone_diag_7` PRs.

The remaining unchecked DoD bullets in the issue ("agent review for `milestone_diag_3` completed and all actionable findings addressed", "`milestone_diag_3` PR opened and merged") can be flipped once this review is filed and the PR lands.

## Validation

Local validation evidence accepted: `scripts/run_all_tests.sh --profile quick` (signature `e1bf653aaa770517`). The signature matches the `milestone_diag_2a` branch baseline, which is consistent with this branch being inventory-only (no source changes).

Inventory numbers re-verified against the working tree at this snapshot (April 29, 2026):

- `rg "ctx\.error\(" crates/sifr_hir/src -g '*.rs'` → 489 across 22 files ✓
- Per-file HIR `ctx.error` counts reproduce the HIR Lowering Surface table exactly ✓
- `rg "TypeErrorKind::" crates/sifr_type_system/src crates/sifr_hir/src -g '*.rs'` → 24 ✓
- `rg "CompileError \{" crates/sifr_driver/src crates/sifr/src -g '*.rs'` → 54 raw matches; actual constructions = 47 (43 production + 4 test) ✓
- `rg "# expect-error" crates/sifr/tests/e2e/fail crates/sifr/tests/e2e.rs -g '*.sifr' -g '*.rs'` → 100 (92 fixture + 8 harness) ✓
- 179 fail-fixture files; 91 annotated; 88 unannotated set matches the enumerated list in the inventory exactly ✓
- Verification baselines: 7 project subdirs, 1 diagnostics subdir, 2 crash inputs all enumerated ✓
- `ctx.warn(...)`: 7 sites (5 in `arithmetic_warnings.rs`, 1 in `statements.rs`, 1 in `typing_and_functions.rs`); inventory's per-row counts of 5/1/1 are correct ✓
