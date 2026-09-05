Review complete. Independent verification of PR #42 and range `a20d9d5..2c2ea24` found **zero actionable findings**. Report below.

---

# Wave 6 Corpus-Side Review — `sifr-lang/leetcode` PR #42

**Issue:** `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` (Wave 6)
**PR:** [sifr-lang/leetcode#42](https://github.com/sifr-lang/leetcode/pull/42) — "Remove dead Combination Sum IV fallback"
**Corpus range reviewed:** `a20d9d5020dae9c19913a598d262ab931924cfe9..2c2ea24d2aaf390fe41689d25fe3bf01415f6bf5`
**Head:** `2c2ea24d2aaf390fe41689d25fe3bf01415f6bf5` (branch `codex/remove-dead-0377-sifr-block`, base `main`)
**Parent compiler head used for validation:** `441f667f0` ("Fix order-independent defaultdict declaration inference (#3081)")
**Reviewer:** agent, independent pass 1. No files were modified.
**Verdict:** **Approved.**

## Methodology

1. Enumerated the exact range (`git log a20d9d5..2c2ea24`) — one commit, one file, `0	15` numstat. Confirmed the pushed GitHub head equals the local head and that `gh pr view 42` reports the same single file, `additions: 0`, `deletions: 15`.
2. Confirmed the Python oracle blob is byte-identical across the range by object identity, not by textual diff: `a20d9d5:src/0377_combination_sum_iv.py` and `2c2ea24:src/0377_combination_sum_iv.py` are both `416e7754d7684a19e6b154efe8a12ef3c8c41444`. Working-tree `.sifr` hashes to `35a426bb…`, matching `2c2ea24:src/0377_combination_sum_iv.sifr` (no uncommitted fixture drift).
3. Reproduced the pre-change failure independently: extracted `a20d9d5:src/0377_combination_sum_iv.sifr` to a scratch directory and ran `check --isolated`. Observed exactly two `SIFR-TYPE-0004` diagnostics — unannotated parameter `total` and non-deterministic return type of `dfs` — both anchored to the unreachable `def dfs` at line 13.
4. Re-ran the post-change gates on the real fixture path with the release binary: `check --isolated` (`no errors found`, exit 0, and notably **zero** `SIFR-FLOW-0901` warnings remain), `build --quiet --isolated` (exit 0), `run --isolated` (exit 0).
5. Executed the Python sibling directly (`python3 src/0377_combination_sum_iv.py`, exit 0) and imported it to sample `combinationSum4` across a value grid.
6. Built a differential parity probe containing the *exact* post-change Sifr function body and compared its native output against the Python oracle over `target ∈ [0,8]` for `nums=[1,2,3]`, plus `([1], 1000)`, `([2], 3)`, and `([9], 4)`.
7. Audited every downstream consumer of the fixture: `run_audit.py` expectation table, `benchmarks/problems/1_d_dynamic_programming.json`, `benchmarks/cases/.../_1_d_dynamic_programming_common.py`, `benchmarks/specs.py` (`mutating_sifr_container_args` regex), `benchmarks/bench.py` (`strip_sifr_main` → `render_sifr_runner`), and the checked-in `benchmarks/fixtures/0377_combination_sum_iv/` inputs/expected values.
8. Audited parent-side expectation surfaces for stale references: `verification/areas/algorithmic_compatibility/manifest.json`, `data/leetcode_full_baseline_results.json`, `data/leetcode_full_baseline_taxonomy.json`, `data/leetcode_profile_manifest.json` — no `0377` references exist, so nothing became stale.
9. Ran `scripts/check_submodule_ownership.py` (PASS) and inspected superproject `git status` / `git diff --submodule=diff`.
10. Swept all 411 `.sifr` fixtures and all Python siblings for the same "statement after a function-level `return`" shape to confirm scope completeness, then probed the compiler's unreachable-code behavior on an annotated nested `def` to establish why removal (not annotation) is the correct remedy.

## Diff minimality

The entire range is a pure 15-line deletion in one file:

```
src/0377_combination_sum_iv.sifr | 15 ---------------
1 file changed, 15 deletions(-)
```

Removed: the nested `def dfs(total)` body and its trailing `return dfs(0)`, both positioned *after* `return cache.get(target, 0)` at the same function-body indent, plus the one separating blank line. Retained verbatim: the header comment, the full bottom-up DP loop (`ways: int = 0` accumulator, `cache.get(total - n, 0)`, `cache[total] = ways`), the live `return cache.get(target, 0)`, and `def main()` with its assertion `assert str(combinationSum4([1,2,3], 4)) == '7'`.

No reachable statement, no annotation, and no assertion was touched. No `+` lines exist anywhere in the range, so no behavior could have been added. The resulting single blank line before `def main` matches the corpus's dominant convention (396 of 411 fixtures use one blank line there; 15 use two).

## Semantic and parity analysis

**The removed block was genuinely unreachable.** It sat after an unconditional `return` in the same function body. Under Python semantics the `def dfs` binding is never evaluated and `return dfs(0)` never executes; under Sifr semantics the compiler classifies it as unreachable (`SIFR-FLOW-0901`) but still submits it to type inference, which is precisely why it produced two hard `SIFR-TYPE-0004` errors. I confirmed this distinction with a control probe: an *annotated* unreachable nested `def` yields only `SIFR-FLOW-0901` warnings and exit 0, so the pre-change failure was caused specifically by inference over dead, unannotated code. Deleting the block is therefore the minimal root-cause remedy; annotating it would have preserved dead code purely to satisfy the checker.

**Live-path parity is exact.** Differential results, Python oracle vs. post-change Sifr native binary:

| Input | Python | Sifr | Match |
| --- | --- | --- | --- |
| `([1,2,3], t)` for `t = 0…8` | `1, 1, 2, 4, 7, 13, 24, 44, 81` | `1, 1, 2, 4, 7, 13, 24, 44, 81` | ✅ |
| `([1], 1000)` | `1` | `1` | ✅ |
| `([2], 3)` | `0` | `0` | ✅ |
| `([9], 4)` | `0` | `0` | ✅ |

`combinationSum4([1,2,3], 4) == 7` is the canonical LeetCode 377 answer, matching both the in-fixture assertion and the `run_audit.py` expectation `("0377": ("combinationSum4", [("[1,2,3], 4", "7")]))`. The `target = 0` boundary agrees (`1`), which is the one case where Python's `cache[target]` and Sifr's `cache.get(target, 0)` could have diverged on an in-contract input, and it does not.

**No test assertion was weakened.** The Sifr fixture's sole assertion is unchanged. The Python sibling's `assert combinationSum4([1,2,3], 4) == 7` is unchanged. The `run_audit.py` expectation row is unchanged. The checked-in benchmark expected value (`n=0001000.expected` = `1`, for generated input `target=1000, nums=[1]`) remains correct under both implementations, which I verified by evaluating the Python oracle at that exact input.

**Downstream consumers are unaffected or improved.** `benchmarks/specs.py::mutating_sifr_container_args` regexes the first `def combinationSum4(...)` signature, which is untouched. `benchmarks/bench.py::strip_sifr_main` truncates the fixture at `def main(` and embeds the remainder as the benchmarked algorithm — so pre-change it embedded the invalid `dfs` block into the generated Sifr benchmark runner (`'def dfs' in algorithm == True`, 26 lines), while post-change it embeds only the 11-line live DP function (`False`). This PR incidentally repairs the Sifr benchmark rendering for 0377 rather than regressing it.

## Artifact and hygiene assessment

The commit contains exactly one file. No generated Rust, no `sifr_output/`, no `.DS_Store`, no benchmark output, and no lockfile is included. `scripts/check_submodule_ownership.py` passes. Superproject `git status` shows the expected pointer-only modification for the corpus submodule; the concurrent `third_party/ruff` modification and the untracked review-artifact path under `plans/reviews/active/` are pre-existing and unrelated to this range.

## Wave 6 ledger conformance

The issue's Wave 6 remedy (line 233) reads: *"Removal of the dead invalid `0377` Sifr fixture block while deliberately leaving the Python reference sibling unchanged as the upstream parity source."* Both halves are satisfied literally — the invalid block is gone, and the Python sibling is byte-for-byte identical by blob hash. The diagnosis section's classification of `0377_combination_sum_iv` as the sole *"Dead invalid fixture surface"* (line 146) is corroborated by my corpus sweep: after this change, no `.sifr` fixture retains unreachable code that fails `check`, and the only two fixtures still carrying unreachable-but-*valid* blocks (`0091_decode_ways`, `0518_coin_change_ii`) both check with exit 0 under `SIFR-FLOW-0901` warnings only — correctly outside a scope defined as "dead **invalid**". Seven Python siblings retain the same shape, consistent with the deliberate parity-oracle preservation.

## Findings

**Actionable findings: none.**

Non-blocking observations, recorded for completeness and requiring no change to PR #42:

- *Informational — corpus lacks a `.gitignore`.* The corpus worktree carries untracked `.DS_Store` and `src/sifr_output/` (the latter a byproduct of `run --isolated`), and `git check-ignore` matches neither. Neither is in this commit, and the commit is clean, but the repository offers no protection against a future `git add -A` capturing build output. Worth a separate hygiene commit, not a change here.
- *Informational — branch base is one commit behind corpus `main`.* The head is based on `a20d9d5`, while `origin/main` has advanced to `7772857` ("Snapshot LRU head before mutable receiver calls (#40)", touching `src/0146_lru_cache.sifr`). `a20d9d5` is an ancestor of `origin/main` and GitHub reports `MERGEABLE` / `CLEAN`, so this PR merges without conflict. Note for the follow-on parent-repo step: the superproject currently pins `a20d9d5`, so bumping the submodule pointer to post-merge `main` will also adopt `7772857`, which the parent has not yet pinned or validated. Validate `0146_lru_cache` alongside `0377` at pointer-bump time.
- *Informational — pre-existing out-of-contract divergence, untouched.* For `target < 0`, Python raises `KeyError` while Sifr returns `0` (`cache[target]` vs. `cache.get(target, 0)`), and the Sifr assertion wraps the result in `str()` where Python compares an `int`. Both stem from the earlier canonicalization recorded in `plans/issues/archive/ad-hoc-optional-none-and-narrowing-closure-execution.md` (line 320), sit on lines this diff does not touch, and lie outside the LeetCode 377 constraint `target >= 1`. Flagged only so it is not mistaken for something this PR introduced.
- *Informational — ledger wording, parent repo, outside this diff.* The Separately Tracked Findings claim that *"there is no standalone unreachable-code diagnostic"* is imprecise: `SIFR-FLOW-0901` ("unreachable statement ignored") exists and fires on this shape, including on unreachable nested `def`s. The actual gap this fixture exposed is narrower — type inference runs on unreachable code and can turn a warning-only situation into hard `SIFR-TYPE-0004` errors. Consider tightening that sentence in the parent-repo Wave 6 ledger update; it does not affect the correctness of the corpus change.

## Validation assessment

Every claim in the PR description reproduced independently on parent head `441f667f0` using `target/release/sifr` with an explicit `--sysroot`:

| Claim | Independent result |
| --- | --- |
| Pre-change fixture fails with two `SIFR-TYPE-0004` on the unreachable `dfs` | Reproduced exactly — parameter `total` and `dfs` return type, both at line 13 |
| `check --isolated` passes post-change | `no errors found`, exit 0, and zero residual `SIFR-FLOW-0901` warnings |
| Release `build --quiet --isolated` passes | Exit 0, binary produced |
| `run --isolated` passes | Exit 0 (in-fixture assertion holds) |
| Python sibling executes successfully | Exit 0 |
| Python sibling has no diff | Stronger: identical blob `416e7754…` on both range endpoints |
| Submodule diff check passes | `scripts/check_submodule_ownership.py`: PASS |

The stated validation is accurate and, for a deletion-only fixture change, sufficient. I extended it with the differential parity grid, the downstream-consumer audit, the parent-side staleness sweep, and the compiler unreachable-code control probe; none surfaced a defect.

## Final verdict

**Approved — zero actionable findings.** The range is a minimal, deletion-only removal of provably unreachable code that was the sole cause of the fixture's two `SIFR-TYPE-0004` check failures. The executable bottom-up DP implementation is preserved unchanged and verified parity-exact against the Python oracle across the tested domain, including the `target = 0` boundary. The Python sibling is byte-for-byte identical by blob hash, satisfying the deliberate upstream-parity-oracle constraint. No assertion or expectation was weakened, no unrelated or generated artifact is included, no parent-side baseline became stale, and the Wave 6 remedy as written in the phase ledger is fully satisfied.
