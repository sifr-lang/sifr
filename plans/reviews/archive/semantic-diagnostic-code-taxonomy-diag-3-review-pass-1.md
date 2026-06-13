# Review: milestone_diag_3 — Diagnostic Emission Inventory (Pass 1)

Branch: `codex/semantic-diagnostics-diag-3`
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Validation evidence reported: `scripts/run_all_tests.sh --profile quick` (signature `e1bf653aaa770517`).

## Scope reviewed

- New file: [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md).
- Phase tracker delta in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) (status flip `2a → 3`, three new checked DoD bullets, two unchecked, validation note).

milestone_diag_3 is a paper-only milestone whose product is the inventory; the only correctness surface is whether the inventory faithfully and exhaustively describes today's emission state, and whether the family/code/fixture plan produced from it is internally consistent.

## Verdict

The inventory is in the right shape and most rows hold up. The HIR file/count breakdown reproduces exactly, the eight `TypeErrorKind` variants are all accounted for, and the `Target Code And Fixture Plan` table is the strongest section — every fixture path I spot-checked exists on disk and the family assignments largely match the issue's identity policy.

There are, however, **four blocking coverage gaps** against the milestone DoD: the "Verification baselines" inventory target listed in the issue is not enumerated at all; 88 of 179 fail-fixture files have no `# expect-error` annotation and the inventory does not address them; the parser-frontend surface is collapsed to a single `SIFR-PARSE-0002` line with no Ruff-fork category breakdown despite `milestone_diag_7` requiring categorized parser codes; and per-category recovery expectations called out by the DoD are not recorded. There are also **three non-blocking but real issues** worth addressing in the same PR: a methodology error inflates the `CompileError` count by ~7 across four files, several HIR file rows have wrong/missing target families that the migration milestones will inherit, and several non-error emission paths (`ctx.warn`, `reveal_types`, the HIR-internal `catch_unwind`) are absent from the inventory even though they are user-visible diagnostic surfaces.

A pass-2 update to the inventory should clear these. Nothing here invalidates `milestone_diag_2b` from starting once the gaps are filled.

## Definition-of-done coverage

| DoD bullet | Status | Evidence / notes |
| --- | --- | --- |
| Inventory covers all raw HIR `ctx.error(...)` call sites | ✅ | Per-file counts in the HIR Lowering Surface table sum to 489 and every per-file count reproduces under `rg "ctx\.error\(" crates/sifr_hir/src -g '*.rs' -c`. All 22 emitting files are listed. |
| Inventory covers all `CompileError` construction paths | ⚠️ partial | All 16 emitting files appear, but the headline count of 54 is inflated by struct/impl/fn-signature lines (see B2 / Findings → B2). Test-only sites (4) are correctly partitioned out. |
| Inventory covers all `sifr_type_system::TypeError` and `TypeErrorKind` variants | ✅ | All 8 `TypeErrorKind` variants (TypeMismatch, UndefinedVariable, UndefinedFunction, WrongArgumentCount, UseAfterMove, MissingTypeAnnotation, InvalidOperator, NotCallable) appear in the Type System Surface table. The 24 construction sites in `sifr_type_system/src/check.rs` reproduce. |
| Inventory covers e2e expectation parsing **and verification baselines** | ⚠️ partial | E2E `# expect-error` markers and harness samples are covered. **Verification baselines under `crates/sifr/tests/verification/` are not enumerated at all** despite the issue's `Existing Surface Inventory` listing them as a required target. See B1. |
| No diagnostic category migrated without a known target code and fixture plan | ⚠️ partial | The Target Code And Fixture Plan covers most categories, but several rows defer fixtures to later milestones (`add/confirm fixture during milestone_diag_2b`, `add parser fail fixture or use existing invalid source tests`), the parser-family plan is a single line, and "category" coverage misses the 88 fail-fixtures with no `expect-error` annotation today (see B3, B4). |

## Findings

### Blocking

#### B1. Verification baselines are not inventoried

The issue's `Existing Surface Inventory` section explicitly lists **"Verification baselines under `crates/sifr/tests/verification`"** as one of the things the inventory must cover, and milestone DoD reads "covers e2e expectation parsing and verification baselines."

The inventory's `E2E Expectation And Baseline Surface` section talks only about `# expect-error` markers in `crates/sifr/tests/e2e/fail/` plus `crates/sifr/tests/e2e.rs` harness samples. It does not enumerate `crates/sifr/tests/verification/`, which today contains:

- `crashes/` — `CR-0001`, `CR-0002` (parser/cfg invariant minimized).
- `diagnostics/decimal_invalid_literal/` — baselines that bake in `[E2501]` and `SIFR-TYPE-0001` for the same diagnostic.
- `project/missing_import_reports_error/`, `project/multi_module_run/`, `project/workspace_ambiguous_import/`, `project/workspace_dotted_helper_run/`, `project/workspace_malformed_manifest/`, `project/workspace_unresolved_import/` — checked-in `check-{compact,human,json}.{stdout,stderr,exit-code}.txt` baselines that hard-code `SIFR-WORKSPACE-0001`, `SIFR-WORKSPACE-0101`, `SIFR-WORKSPACE-0102`, `SIFR-TYPE-0001`, `[E2501]`, and absolute-path remnants ("<WORKSPACE>" sentinel notwithstanding).

Greppable proof:

```bash
grep -h "SIFR-\|\[E25" crates/sifr/tests/verification/**/baselines/*.txt | sort -u
# returns SIFR-TYPE-0001, SIFR-WORKSPACE-0001, SIFR-WORKSPACE-0101, SIFR-WORKSPACE-0102, [E2501]
```

These baselines are exactly what `milestone_diag_4a`/`diag_5`/`diag_6`/`diag_11` will need to regenerate, so the inventory must list them so the migration plan does not accidentally treat them as out of scope. Action: add a `Verification Baseline Surface` subsection to the inventory enumerating each verification subdirectory, the diagnostic codes/markers it asserts today, and which migration milestone owns its regeneration.

#### B2. Parser surface is collapsed to one line; Ruff-fork category breakdown is missing

`milestone_diag_7` scope: "Map upstream Ruff-fork parser error categories to distinct `SIFR-PARSE-*` codes where the parser exposes a condition category." `milestone_diag_3`'s job is to surface those categories so 2b can populate the registry and 7 can migrate — categories cannot land in the registry if they are not first identified by the inventory.

The inventory's Target Code And Fixture Plan has only a single parser entry:

> `SIFR-PARSE-0002` | syntax parse failure with source span | parser adapter / driver frontend | add parser fail fixture or use existing invalid source tests

That collapses every Ruff parser-error category into one bucket, which is exactly what the policy forbids ("`SIFR-PARSE-0001` must not remain a general semantic fallback"). It also leaves `SIFR-PARSE-0002` as a single catch-all that is structurally identical to the retired `SIFR-TYPE-0001`.

The Driver And CLI Surface row for `frontend/api.rs` records the two `CompileError` construction sites that wrap parser failures but does not break out parser error categories. The upstream `parsed.errors()` source (Ruff fork at `sifr_python_parser`) does expose distinct error variants — those need to be enumerated here, with at least an initial cut into named groups (e.g., unbalanced brackets, indentation, invalid token, unsupported syntax, etc.) and a mapping to proposed `SIFR-PARSE-000x` codes. The exact taxonomy can be refined in 2b, but a single `0002` row is not "a known target code and fixture plan" for the parser family.

Action: add a Parser Surface subsection that enumerates the Ruff-fork parser error variants currently produced by `sifr_python_parser::parse_module`, propose at least 3–5 distinct `SIFR-PARSE-000x` codes, and indicate which existing invalid-source tests (or new fixtures) lock each.

#### B3. 88 of 179 fail-fixtures have no `# expect-error` annotation; the inventory ignores them

`crates/sifr/tests/e2e/fail/` contains 179 `.sifr` files; only 91 of them contain any `expect-error` line. The other 88 fixtures are fail tests that today rely on bare "compilation must fail" without asserting a code. The harness loop in [crates/sifr/tests/e2e.rs:2532](../crates/sifr/tests/e2e.rs:2532) runs them through `compile_source` and panics if compilation succeeds, but performs no code assertion when `expected` is empty.

That set is structurally part of the fixture migration scope: every one of those fixtures is a category that today has no fixture-asserted code, and milestone DoD #5 ("No diagnostic category is migrated without a known target code and fixture plan") requires the inventory to either (a) declare which target code each will assert post-migration, or (b) state explicitly that they are exempt and explain why.

The 88 files cluster into recognizable categories by filename — `bytes_*`, `argparse_*`, `bisect_*`, `borrowed_mut_parameter_*`, `async_*`, etc. — which strongly suggests target families like `STDLIB`, `OWN`, etc. The inventory should map each filename pattern to its planned target code so milestone_diag_5/diag_8 do not need to re-derive that classification under time pressure.

Action: either enumerate the 88 unannotated fail fixtures and assign each a target code (likely as a grouped table by category), or add an explicit policy line that those fixtures will gain `expect-error` annotations in the milestone that migrates their family, with the migration-milestone column populated.

#### B4. Per-category recovery expectations are not recorded

`milestone_diag_3` Scope bullet: "Identify expected recovery behavior for each diagnostic category." DoD does not list recovery as a separate facet, so this is debatable as a strict gate, but the Scope bullet is unambiguous.

The inventory's `Span, Related-Span, And Recovery Notes` section says only:

> Recovery deduplication remains `milestone_diag_10`. Until then, repeated type errors keep the existing recovery behavior but must share `message_template` and explicit dedupe args once migrated.

That is a general policy statement, not a per-category identification. Recovery decisions differ materially per category — e.g., `OWN` use-after-move should taint the moved binding to suppress cascades; `CALL` arity should not poison the callable; `MATCH` non-exhaustiveness should not poison the subject; `TYPE` mismatch sometimes returns `Type::Any` to keep lowering progressing; decimal mixed arithmetic may return the inferred operand type. These per-category expectations are exactly what `milestone_diag_10` will encode; they should be inventoried now.

Action: extend the Target Code And Fixture Plan with a "recovery expectation" column (or add a parallel table), with one of {`taint binding`, `taint expression`, `non-tainting`, `dedupe by primary span+args`, `cap-summarize`} per code.

### Non-blocking — correctness/coverage issues that should be fixed in the same PR

#### N1. The `CompileError` headline count is inflated by ~7 due to a methodology error

The inventory's coverage-snapshot bullet:

> `rg "CompileError \\{" crates/sifr_driver/src crates/sifr/src -g '*.rs'` finds 54 legacy driver/CLI construction sites: 50 production sites and 4 test-only diagnostic construction sites.

The pattern `CompileError \{` matches `pub struct CompileError {`, `impl CompileError {`, `impl … for CompileError {`, and `fn …(…) -> CompileError {` in addition to genuine `CompileError { … }` literal constructions. Concretely:

| File | Inventory says | Actual constructions | Inflated by |
| --- | ---: | ---: | ---: |
| `crates/sifr_driver/src/diagnostics.rs` | 4 | 1 (line 262) | +3 (struct decl line 26, `impl` block line 95, `impl Display` line 223) |
| `crates/sifr_driver/src/build/materialize.rs` | 2 | 1 (line 138) | +1 (fn signature line 137) |
| `crates/sifr_driver/src/workspace/mod.rs` | 4 | 2 (lines 162, 176) | +2 (fn signatures lines 161, 175) |
| `crates/sifr_driver/src/project/discovery.rs` | 7 | 6 (lines 196, 200, 215, 392, 404, 414) | +1 (fn signature line 194) |

True production+test count is **47** (43 production + 4 test), not 54. The inflation is concentrated in the file-by-file rows the migration plan will read line by line — particularly `diagnostics.rs`, where saying "4 sites" implies four separate code paths to migrate when there is only one (the codegen panic boundary at line 262); the other three "matches" are the type's own definition.

Action: rerun with a stricter regex (e.g. `rg "(\\(|\\[|^\\s*)CompileError \{"` to filter struct/impl/return-type), or hand-correct the four affected rows, and update the headline. The Driver And CLI Surface table's interpretation column already correctly describes what each *file* does, so only the count column needs adjustment.

#### N2. HIR row target families have wrong/missing assignments

A few HIR file rows assign target families that don't match what those files actually emit:

- **`lower/decimal_methods.rs`** is listed with target `DECIMAL` only. Inspection shows it also emits `decimal.sqrt() takes no arguments`, `decimal.abs() takes no arguments`, and `type 'decimal' has no method '{method}'` — the first two are arity (`CALL`), the third is method-not-found (`NAME` or `TYPE`). The 18 emissions are not all `DECIMAL`. Either widen the family list to `DECIMAL`, `CALL`, `NAME`, or split the row.

- **`lower/classes.rs`** is listed with target `CLASS, TYPE, NAME`. It also emits iter/next/reversed protocol shape errors (`class 'X.__iter__' must not declare parameters besides self`, `class 'X.__iter__' must return 'Iterator[T]' or 'Iterable[T]'`, `class 'X' iteration protocol mismatch: ...`). Per the issue's family policy ("Missing or malformed protocol methods are `SIFR-PROTO-*`; ordinary missing class fields or constructors are `SIFR-CLASS-*`"), these are `PROTO`. Add `PROTO` to the row.

- **`lower/expressions.rs`** is listed with target `NAME, TYPE, CALL, STDLIB, PROTO, DECIMAL, FLOW`. It also emits `ctx.error("unsupported expression type")` and `matrix multiplication operator (@) is not supported` — neither of these maps cleanly to any listed target. Per Wrong-Layer policy these would either be a new "unsupported language feature" code under `INTERNAL` (post-panic-boundary) or remain a TYPE-shaped error. Decide and record explicitly; don't leave them in an unnamed bucket.

- **`lower/mod.rs`** is listed with target `TYPE, IMPORT, NAME, STDLIB, WORKSPACE`. Including `WORKSPACE` is contradictory because the Wrong-Layer note immediately below says workspace failures should *leave* HIR for the driver. The list reads as "this file will emit WORKSPACE codes" when the migration plan is "this file will stop emitting WORKSPACE codes and the driver will own them." Re-label the row's WORKSPACE entry as "→ moves to driver" or split off the wrong-layer subset into a separate table cell.

#### N3. Non-error emission paths are not inventoried

The inventory limits itself to `ctx.error(…)`. The HIR has at least three other user-visible diagnostic streams that need migration:

- **`ctx.warn(...)`** — 8 sites across `lower/typing_and_functions.rs`, `lower/statements.rs`, `lower/arithmetic_warnings.rs`, `lower/mod.rs`. These flow into `LowerCtx::warnings` and are surfaced to the user. Once `Severity::Warning` becomes a first-class part of the diagnostic model (it already is in `sifr_diagnostics`), each of these warnings needs a code. Today they have none.

- **`ctx.reveal_types`** — `reveal_type(...)` developer diagnostic. `milestone_diag_10` already calls out the recovery-cap fixture for >50 `reveal_type(...)` calls, so this is a known surface. The inventory should record it (presumably as `Severity::Note` with a dedicated `INTERNAL` or new family code, e.g. a `SIFR-DEV-*` or reusing `SIFR-INTERNAL-*` Note severity).

- **HIR-internal `catch_unwind`** at [crates/sifr_hir/src/lower/typing_and_functions.rs:780](../crates/sifr_hir/src/lower/typing_and_functions.rs:780). On panic, it calls `ctx.warn(...)` to skip exhaustive-return validation. Per the Architecture Source Mapping section, codegen/HIR panics that survive boundaries should become `SIFR-INTERNAL-*`, not user warnings. This is a wrong-layer case the inventory misses.

Action: add a "Non-Error Emission Paths" subsection enumerating warnings (with target family decision per call site), `reveal_type` (target code + recovery policy), and HIR-internal `catch_unwind` boundaries.

#### N4. CLI renderer-test `CompilerDiagnostic` constructions are not enumerated

The issue's `Existing Surface Inventory` lists "CLI renderer tests that manually construct `CompilerDiagnostic`" as a required target. The inventory's table tracks `CompileError` constructions but not `CompilerDiagnostic` constructions. There are 11 such test-only constructions today:

- `crates/sifr/src/main.rs` — 9 sites at lines 1272, 1311, 1322, 1333, 1363, 1375, 1400, 1411, 1422 (all inside `#[test]` fns), each hard-coding `SIFR-TYPE-0001`/`SIFR-PARSE-0001`/`[E2507]`-style strings.
- `crates/sifr_driver/src/tests/diagnostics.rs` — 2 sites at lines 74, 102.

These will all need updating in `milestone_diag_5` (test harness contract cleanup). Add them to the inventory either as a row under Driver And CLI Surface or in a new `CLI Test Surface` subsection.

#### N5. Marker-count table heading is misleading

The `E2E Expectation And Baseline Surface` table is titled "Current fail-fixture code markers" and reports e.g. `SIFR-TYPE-0001 | 95` and `[E2507] | 5`. Reproducing those numbers requires counting **fail-fixture markers + `crates/sifr/tests/e2e.rs` harness samples together**:

- `SIFR-TYPE-0001`: 91 in `tests/e2e/fail/*.sifr` (one fixture has 2 markers) + 4 in `e2e.rs` = 95.
- `[E2507]`: 2 in `tests/e2e/fail/` + 3 in `e2e.rs` = 5.

The numbers themselves are correct as totals, but the table heading says "fail-fixture code markers", which is what motivated me to verify and recount. Either rename the heading to "Fail-fixture and harness-sample code markers" or split fixture vs. harness-sample columns. Otherwise the migration plan may treat all 95 as fixture rewrites when 4 are harness-test rewrites (different milestone owner — `diag_5`, not the family-specific milestones).

Same issue at the snapshot bullet: "92 fail-fixture expectations plus 8 harness test samples" is correct, but the marker-count table mixes the two streams.

#### N6. Inventory doesn't separate "today's code source" from "today's code value"

The Driver And CLI Surface table mixes two different "current code source" things into one column:

1. Phase-derived (`CompilePhase::TypeCheck → SIFR-TYPE-0001`).
2. Prefix-string-match classifier (`CompileError::workspace_diagnostic_code`).
3. Hard-coded constructor strings (CLI test fixtures).

For migration it matters which one applies to which row. For example, the `frontend/api.rs` row says "parser frontend errors become `CompilePhase::Parse`; HIR lowering errors become `CompilePhase::TypeCheck`" — so today's code source for those rows is "phase-derived", and the workspace classifier is irrelevant. The `workspace/mod.rs` row says "manifest parse/source-root validation" — today's code source for those is "phase=Build, then prefix-classified to WORKSPACE-000x", a two-stage pipeline that needs both stages eliminated.

Adding a "current code mechanism" column ({phase-derived, prefix-classified, hard-coded string, typed-error-forwarded}) would make the migration plan unambiguous and surface the prefix classifier as a discrete deletion target rather than burying it in prose.

### Non-blocking — minor

#### M1. `lower/typing_and_functions.rs` row may be missing `CALL`

Row lists targets `NAME, TYPE, RESULT, PROTO`. The file emits "unsupported default argument expression for parameter '{p}'", "function 'X' must return a value of type 'Y' on all control-flow paths", and callable annotation shape checks (`Callable[...]`). Some of these are CALL-flavored. Verify and add `CALL` if applicable.

#### M2. Fixture-plan column uses two different "fixture pending" phrases

Some rows use "add/confirm fixture during `milestone_diag_2b`", others use "add/confirm fixture", others use just "add parser fail fixture or use existing invalid source tests". Pick one phrasing and target milestone so the registry-population step has a single grep target for "fixture missing".

#### M3. Decimal subscript on snapshot bullets vs. table

The snapshot says `[E2507]` → 5, but the per-row table heading is "fail-fixture code markers". As noted in N5, the heading should be widened or the harness-sample contribution tabulated separately.

#### M4. `SIFR-CLASS-0001`/`0002` scope is ambiguous

Plan table:

- `SIFR-CLASS-0001` | "class has fields but no required initializer/super initializer"
- `SIFR-CLASS-0002` | "required field declared after default"

There is also a fixture `auto_init_inheritance_missing_super.sifr` which today emits `SIFR-TYPE-0001` and lives in the same logical category as 0001. The inventory should pin which fixture locks 0001 (probably `auto_init_inheritance_missing_super.sifr` or the broader auto-init group) so 2b registry population is unambiguous.

## What is solid

- HIR `ctx.error(...)` per-file count table is exact, complete, and reproducible.
- `TypeErrorKind` variant coverage is exhaustive (all 8 variants).
- Family assignments at the code level (target codes in the plan) align with the issue's identity policy: `SIFR-NAME-0001` for undefined variable, `SIFR-CALL-0001..0005` for call shape errors, `SIFR-OWN-0001..0004` for ownership, `SIFR-DECIMAL-0001..0008` for decimal codes (matches the issue's Decimal Code Migration table exactly), `SIFR-FLOW-0001..0003` for break/continue/nonlocal — these are clean and consistent with the issue's stated policies.
- All inventoried fixture paths I spot-checked exist on disk; none are broken references.
- Wrong-Layer notes correctly identify three real wrong-layer cases (import resolution, decimal pseudo-codes, builtin/stdlib helper split) — they are the three that matter most for `diag_4a`/`diag_7`.
- The 4 test-only `CompileError` sites in `crates/sifr_driver/src/tests/diagnostics.rs` are correctly partitioned out and explicitly marked for "rewrite or delete with the legacy diagnostic abstraction."
- The `milestone_diag_2b` workspace-code review at the bottom of the Driver And CLI Surface section is the correct level of decisiveness — keep `0001..0004`, keep `0101..0103`, propose `0104` for project import-cycle if driver-owned. That's exactly the per-code review the policy demands.

## Recommendation

Address blockers B1–B4 in a pass-2 update to the inventory, then merge. The non-blocking items (N1–N6, M1–M4) are best fixed in the same pass since most are small text/cell edits in the same document. None of the non-blocking items would by themselves justify holding the milestone, but together they meaningfully reduce the inventory's load-bearing accuracy for `milestone_diag_2b` and `milestone_diag_4a`.

## Validation

Local validation evidence accepted:

- `scripts/run_all_tests.sh --profile quick` (signature `e1bf653aaa770517`) — same signature reported on the `milestone_diag_2a` branch, which is consistent with this branch being inventory-only (no source changes that should affect the report).

Inventory numbers re-verified against the working tree:

- `rg "ctx\.error\(" crates/sifr_hir/src -g '*.rs'` → 489 ✓
- `rg "ctx\.error\(" crates/sifr_hir/src -g '*.rs' -c` per-file totals reproduce the inventory's HIR table exactly ✓
- `rg "TypeErrorKind::" crates/sifr_type_system/src crates/sifr_hir/src -g '*.rs'` → 24 ✓
- `rg "# expect-error" crates/sifr/tests/e2e/fail crates/sifr/tests/e2e.rs -g '*.sifr' -g '*.rs'` → 100 (92 fixture + 8 harness) ✓
- `rg "CompileError \{" crates/sifr_driver/src crates/sifr/src -g '*.rs'` → 54 raw matches; **actual constructions = 47** (43 production + 4 test). See N1.
