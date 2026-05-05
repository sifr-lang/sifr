# milestone_diag_5 slice 3 review (pass 4)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-expectation-contradictions` against `origin/main`, layered on top of [reviews/semantic-diagnostic-code-taxonomy-diag-5-expectation-contradictions-review-pass-3.md](reviews/semantic-diagnostic-code-taxonomy-diag-5-expectation-contradictions-review-pass-3.md). Slice intent (per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76), unchanged this pass): "add e2e fixture expectation contradiction detection so overlapping explicit `expect-error[col=N]` assertion locations cannot claim incompatible diagnostic codes, while unqualified markers continue to assert code existence only; load all fail-fixture expectation contracts before compiling the fail corpus."

Files in scope (uncommitted diff only):

- [crates/sifr/tests/e2e.rs](crates/sifr/tests/e2e.rs) — same surface as pass 3 (`LocatedCompileFailureExpectation`, `expectation_locations_overlap`, `expectation_location_label`, `validate_expectation_contradictions`, non-panicking `parse_compile_failure_expectations`, thinned `extract_compile_failure_expectations`, accumulator wiring in `test_e2e_fail`, `test_expected_error_contract_rejects_contradictory_overlapping_locations`), with one additional negative-case block added inside the contradiction unit test.
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) — slice 3 in-progress status line at [:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) unchanged from pass 3.

Out-of-scope DoD bullets (centralized baseline normalization and the JSON/compact/human renderer fixture-level test) remain explicitly carried forward to later slices as in passes 1–3.

## Verdict

**Satisfied — no must-fix blockers.** The pass-3 verdict was already satisfied. Pass 4 is a single, additive non-blocking improvement that closes pass-3 Observation J without touching any production code path or any other test.

The new block at [crates/sifr/tests/e2e.rs:3113-3129](crates/sifr/tests/e2e.rs:3113) (`unqualified_markers_do_not_conflict`) wires an explicit `(None, None)` distinct-code negative case directly into `test_expected_error_contract_rejects_contradictory_overlapping_locations`:

```
LocatedCompileFailureExpectation { line_number: 5, expectation: { code: "SIFR-TYPE-0002", column: None } },
LocatedCompileFailureExpectation { line_number: 6, expectation: { code: "SIFR-NAME-0001", column: None } },
```

asserted `is_ok()`. This case is the marker shape most likely to drift back under reading A — a future refactor could plausibly change `(None, _) | (_, None) => false` to `(None, None) => true` to "match all-codes-on-this-fixture," and this assertion now traps that drift inside the contradiction contract's own self-contained test, instead of relying on the indirect coverage from `test_expectation_parsing_contract` noted in pass-3 finding J.

The change is exactly the minimum needed to satisfy pass-3 Observation J — distinct codes, both `None`, different `line_number`s so the case is unambiguously outside the same-line equivalence class, and a clear identifier name. No other test was modified, no production code changed.

## Pass 3 follow-up status

### Observation I (non-blocking — fail corpus has no `[col=N]` markers) — UNCHANGED

`grep -rcE "expect-error\[col=" crates/sifr/tests/e2e/fail/` still returns zero hits across the fail corpus. The validator continues to be exercised only through unit tests, which is the deliberate consequence of reading B; no slice-scope action.

### Observation J (non-blocking — explicit `None × None` distinct-code negative case missing) — RESOLVED

The new block at [crates/sifr/tests/e2e.rs:3113-3129](crates/sifr/tests/e2e.rs:3113) is the explicit `None × None` distinct-code negative case. With it, the contradiction unit test now self-contains all four corner cases of `(left.column, right.column)` × `(same code, distinct code)` that the validator must distinguish:

| Left column | Right column | Codes | Block | Asserted |
| --- | --- | --- | --- | --- |
| `Some(4)` | `Some(4)` | distinct | [:3048-3071](crates/sifr/tests/e2e.rs:3048) | flagged (positive) |
| `None` | `Some(9)` | distinct | [:3095-3111](crates/sifr/tests/e2e.rs:3095) | not flagged |
| **`None`** | **`None`** | **distinct** | **[:3113-3129](crates/sifr/tests/e2e.rs:3113)** | **not flagged (new)** |
| `Some(4)` | `Some(9)` | distinct | [:3131-3147](crates/sifr/tests/e2e.rs:3131) | not flagged |
| `None` | `Some(9)` | same | [:3149-3165](crates/sifr/tests/e2e.rs:3149) | not flagged |

The test now reads as a direct expression of reading B's matrix: only `(Some(c), Some(c))` with distinct codes flags. ✓

### Observation K (non-blocking — `extract_compile_failure_expectations` panic path not directly tested) — UNCHANGED

Still transitively covered through `test_e2e_fail`'s panic-on-contract-violation path. Not addressed in pass 4 and not a blocker.

## New observations (non-blocking)

None. Pass 4's surface is one additive `LocatedCompileFailureExpectation` `vec!` literal plus a single `assert!` line; there are no new constructs to evaluate.

## Validation reproduction

| Command | Result |
| --- | --- |
| `cargo fmt --check` | clean (no output) |
| `git diff --check` | clean (no output) |
| `cargo test -p sifr expected_error_contract` | `4 passed; 0 failed` (`test_expected_error_contract_rejects_contradictory_overlapping_locations` passes with the new block) |

These mirror what the user reported running. The same tests that passed in pass 3 continue to pass; no broader test surface was disturbed by this change. As in pass 3, recommend `scripts/run_all_tests.sh --profile quick` before flipping the issue checkbox per [AGENTS.md](AGENTS.md), but that is the merge gate, not a review gate.

## Working-tree note (out of scope)

The branch working tree is in an unresolved merge state for 27 `audits/leetcode/*.sifr` files (`git status` reports them as `deleted by us` in the unmerged paths section), and contains additional untracked files (`package.json`, `package-lock.json`, multiple unrelated `issues/` and `reviews/` files, `verification/leetcode/`). None of these are part of the slice 3 diff (`git diff origin/main -- crates/sifr/tests/e2e.rs issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md` is the slice's contribution and is what this review evaluated). The merge-conflict and untracked files are outside slice 3's scope and outside this review's verdict, but flagging here so they are not accidentally picked up into a slice-3 commit. Per [AGENTS.md](AGENTS.md) safety rules ("If unexpected repo modifications appear, stop and ask before proceeding"), the author should resolve or stash that conflict before staging the slice-3 commit so the resulting PR matches the diff reviewed here.

## Summary

- **Pass 3 verdict (Satisfied — no must-fix blockers)**: still holds.
- **Pass 3 Observation J**: resolved in this pass by the explicit `None × None` distinct-code negative case at [crates/sifr/tests/e2e.rs:3113-3129](crates/sifr/tests/e2e.rs:3113).
- **Pass 3 Observations I and K**: unchanged, still non-blocking.
- **No new findings** in pass 4. Pass 4's diff is minimal, additive, and tightly scoped.
- **Working-tree hygiene**: an unresolved leetcode merge conflict and unrelated untracked files exist on the branch — out of scope for this review but should be resolved before committing the slice.

Slice 3 is ready to ship pending `scripts/run_all_tests.sh --profile quick` green and a clean working tree at commit time.
