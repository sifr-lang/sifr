# Wave 6 Parent-Repository Review — `sifr-lang/sifr` PR #3085 (draft)

**Issue:** `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` (Wave 6)
**Parent range reviewed:** `441f667f03be45975d2e8a9aaa34ed2d47a852cf..d1b688b0811df723d19ca238b5170f3fdc2fbd24`
**Head:** `d1b688b0811df723d19ca238b5170f3fdc2fbd24` — matches `gh pr view 3085` `headRefOid`; branch `codex/algorithmic-corpus-remove-dead-0377`, base `main`, `MERGEABLE`, draft
**Corpus (submodule) range adopted:** `a20d9d5020dae9c19913a598d262ab931924cfe9..d50fa735034652ab70f26cd2b351efd7f7b689e3`
**Reviewer:** agent, independent parent pass 1. No files were modified.
**Verdict:** **Approved — zero actionable findings.**

## Methodology

1. Enumerated the exact parent range: one commit (`d1b688b08`), three paths — the issue ledger (+8/−6), the new corpus review artifact (+97), and the corpus gitlink (1 line). `gh pr view 3085` reports the identical file set and counts.
2. Verified gitlink provenance: `git ls-tree d1b688b08` pins `160000 commit d50fa735…`; `d50fa735` equals corpus `origin/main` and `git merge-base --is-ancestor` confirms reachability. `.gitmodules` tracks `branch = main` for that path.
3. Verified the squash-merge introduced nothing beyond the reviewed corpus change: `git diff e75af09 d50fa73` is exactly `src/0377_combination_sum_iv.sifr | 15 -----`, and `d50fa735:src/0377_combination_sum_iv.sifr` is blob `35a426bb…`, byte-identical to the corpus reviewer's head `2c2ea24:src/0377…sifr`.
4. Read all three adopted fixture diffs (`0146_lru_cache`, `0189_rotate_array`, `0377_combination_sum_iv`) plus the full current text of each fixture and the `0377` Python sibling.
5. Confirmed the `0377` Python oracle is untouched by object identity, not text: `a20d9d502:src/0377…py` and `d50fa735:src/0377…py` are both blob `416e7754…`. `git diff --stat a20d9d502..d50fa735 -- 'src/*.py'` is empty, so no Python sibling changed anywhere in the range.
6. Reproduced the pre-change state independently: extracted all three `a20d9d502` fixture blobs to a scratch directory outside any package root and ran `check`/`build`/`run` with an explicit `SIFR_SYSROOT` against `target/release/sifr`.
7. Probed the compiler's unreachable-code behavior directly with an *annotated* unreachable nested `def` to test the corrected ledger sentence about `SIFR-FLOW-0901` versus inference-produced hard diagnostics.
8. Re-validated the post-change fixtures at their real corpus paths with real exit codes (not pipeline status), plus each Python sibling, and confirmed `run` genuinely executes assertions by verifying a deliberately failing `assert` panics with non-zero status.
9. Audited parent-side expectation surfaces for staleness: `verification/areas/algorithmic_compatibility/manifest.json`, `data/leetcode_profile_manifest.json`, `data/leetcode_full_baseline_results.json`, `data/leetcode_full_baseline_taxonomy.json`, `taxonomy_smoke_results.json`, `runner.py` (`run_leetcode_check`, `validate_profile_manifest`, `expected_fixture_count`), the `pinned_corpus` checksum consumer in `coverage_matrix.py`, and the `algorithmic_compatibility` suite selection in all four profiles.
10. Executed the relevant lane on this head: `runner.py --suite profile-manifest` (pass), plus `check_submodule_ownership.py` (PASS), `check_file_size_guardrails.py` (PASS, 3027 files), `check_docs_error_code_links.py` (pass).
11. Verified the Wave 5 merged/gate claims against primary evidence: `gh pr view 3081` merge commit and the retained `target/validation_lane_reports/create-pr.latest.log`.
12. Cross-checked the ledger's diagnosis inventory, wave list, status transitions, and the PR description against everything above. No full corpus sweep was run, per scope.

## Disposition of the corpus-side review

The adopted artifact (`…wave-6-corpus-agent-review-pass-1.md`, 97 lines) is accurate on every claim I could re-derive, and I reproduced its key evidence independently rather than accepting it: the two-diagnostic pre-change failure, the byte-identical Python blob hash, the deletion-only diff shape, and the absence of `0377` from every parent expectation surface. Its zero-actionable-findings approval stands.

Two of its four informational notes were carried correctly into this PR:

- Its ledger-wording note ("*there is no standalone unreachable-code diagnostic*" is imprecise) is the origin of this PR's Separately Tracked Findings correction — adopted and, per §"Ledger and separately-tracked wording" below, correctly.
- Its explicit hand-off instruction — *"the superproject currently pins `a20d9d5`, so bumping the submodule pointer to post-merge `main` will also adopt `7772857` … Validate `0146_lru_cache` alongside `0377` at pointer-bump time"* — was honored: both `0146` and `0189` are validated here and named in the ledger row.

Its artifact header cites the pre-squash branch head `2c2ea24`, while the parent pins the squash-merge commit `d50fa735`. I verified the two carry an identical `0377` blob and that the squash added no other edit, and the parent ledger row states the merged head explicitly, so the artifact is not misleading.

## Fixture-change verification

### `0377_combination_sum_iv` — dead-code removal

The diff is a pure 15-line deletion with no `+` lines. Removed: the nested `def dfs(total)` body and the trailing `return dfs(0)`, both sitting *after* the unconditional `return cache.get(target, 0)` at the same function-body indent, plus one blank line. Provably unreachable under both Python and Sifr semantics.

Retained verbatim and confirmed by reading the current file: the header comment, the full bottom-up DP loop (`ways: int = 0`, `cache.get(total - n, 0)`, `cache[total] = ways`), the live `return cache.get(target, 0)`, and `main()`'s assertion `assert str(combinationSum4([1,2,3], 4)) == '7'`. No reachable statement, annotation, or assertion was touched, and no assertion was weakened.

Pre-change reproduction on this head (scratch copy of the `a20d9d502` blob):

```
error[SIFR-TYPE-0004]: parameter 'total' in function 'dfs' is missing a type annotation and could not be inferred  --> :13:13
error[SIFR-TYPE-0004]: function 'dfs' return type could not be inferred deterministically                          --> :13:9
```

Exactly two `SIFR-TYPE-0004`, both anchored to the dead `def dfs` — matching the ledger and PR claims precisely. Post-change: `check` reports `no errors found`, `run` exits 0 with no panic, and `python3 src/0377_combination_sum_iv.py` exits 0. The Python sibling retains its own dead block, which is correct: it is the untouched upstream parity oracle.

### `0146_lru_cache` and `0189_rotate_array` — snapshot-before-mutable-call

`0146` replaces `self.insertAfter(node, self.head)` with `head = self.head; self.insertAfter(node, head)` at two sites (`moveToFront`, `put`). Semantically inert: `self.head` is assigned only in `__init__`, `insertAfter` never writes it, and in `moveToFront` the read still follows `self.detach(node)` exactly as before. The snapshot is taken immediately before the call, so the observed value is unchanged on both paths.

`0189` hoists `nums_len = len(nums)` and uses it for the first and third `_reverse_range` calls. Inert: `_reverse_range` only swaps elements in place and cannot change length, and the second call still uses the loop-computed `rot`. The hoist happens after the `while rot >= len(nums)` normalization loop, so `rot` is unaffected.

Runtime confirmation (real exit codes, `panicked` grep = 0 in all cases): post-change `0146` and `0189` both `check`/`build`/`run` clean, and their in-fixture assertion batteries hold. Their Python siblings are unchanged in the range and exit 0. No semantic regression on either fixture.

Notably, the pre-change `0146` and `0189` blobs *also* check, build, and run clean on this parent head — see observation 1.

## Ledger, separately-tracked wording, and Wave 5 evidence

**Corrected separately-tracked sentence.** The new text — *"unreachable nested function bodies still reach type inference after `SIFR-FLOW-0901` and can produce hard type diagnostics"* — is accurate on both halves, which I verified against the compiler rather than the prose. `SIFR-FLOW-0901` does exist and does fire on this exact shape: an annotated unreachable nested `def` after a `return` yields two `warning[SIFR-FLOW-0901]: unreachable statement ignored` (one spanning the `def` and its body, one on the trailing `return`) and exits 0. Remove the annotations and the same shape yields the two hard `SIFR-TYPE-0004` errors. So the gap is precisely inference-after-warning, not a missing diagnostic, and the replaced claim ("there is no standalone unreachable-code diagnostic") was indeed wrong. The finding is framed as pre-existing and not used as an exclusion, consistent with the section's stated policy.

**Wave 6 row.** Every checkable claim holds: sole unreachable `dfs` fallback removed; live DP, assertion, and Python sibling unchanged; Python blob byte-identical across the range; exactly two pre-change `SIFR-TYPE-0004`; corrected fixture checks/builds/runs; Python oracle runs; corpus agent pass 1 approved with zero findings; pointer advances `a20d9d502 → d50fa7350`; range also contains the `0146`/`0189` changes; all three changed fixtures plus siblings pass; ownership guard passes. Status `implemented; corpus PR #42 merged; parent review pending` matches the draft state, and `Waves 7-8 | pending | start sequentially after the Wave 6 parent PR merges` is consistent with the nine-item wave list and the unchanged `Full-corpus closeout | blocked` row.

**No stale corpus baseline or profile surface.**
- `leetcode_profile_manifest.json` references `0146_lru_cache` in `representative_subset` with `expected_classification: PASS` via `target/debug/sifr check` — still PASS post-change (verified). `0377` and `0189` appear in no data file.
- `full_corpus.expected_fixture_count` is validated against `len(glob("*.sifr"))`; the corpus still holds 411 `.sifr` fixtures (and 395 Python siblings), unchanged by a within-file deletion. The `profile-manifest` lane passes on this head (`variants=1, failures=0`).
- The two baseline files are placeholders (411 cases, all `PASS`, empty `results`/`failures`); `0377` moving from check-fail to pass moves toward them, never away. `taxonomy_smoke_results.json` references none of the three.
- `merge` and `release` select `representative-subset` (covers `0146`, verified passing); `create-pr` selects only `profile-manifest`; `nightly` selects `leetcode-full`, whose fixture-count and baseline inputs remain valid. `leetcode-check`, which has no known-failure allowlist, is not selected by any profile — consistent with the issue's deferred `leetcode-full` release restoration.
- `manifest.json`'s `pinned_corpus.checksum` is only presence-checked by `coverage_matrix.py` (it asserts `required`/`revision`/`checksum` exist and are well-typed); the digest is never recomputed against corpus contents, so the fixture edits cannot stale it.

**Wave 5 merged/gate evidence is accurate.** `gh pr view 3081` reports `MERGED`, `mergeCommit.oid = 441f667f03be45975d2e8a9aaa34ed2d47a852cf` — exactly the `merged at 441f667f03` claim and the range base. Every gate number in the new row reconciles line-for-line with `target/validation_lane_reports/create-pr.latest.log` (`profile=create-pr`): `python interop … variants=19, failures=0` at `elapsed_ms=506546` (= 506.546 s); `rust interop … variants=10` at `elapsed_ms=9106` (= 9.106 s); `developer tooling … variants=18`; `performance … variants=7`; generated-code-quality step `variants=5`; `crate_tests`, `runtime_platform_suites`, and `e2e_pass_suite` all `status=pass`; `131 pass tests completed (131 passed, 0 failed)` with `report_signature=7c39b8c1dd4fec7c`. All `[sifr-lane-step-budget]` entries in the log are `status=pass`.

## Hygiene and artifact assessment

Three files, all appropriate: a ledger update, an adopted review artifact, and a one-line gitlink advance. No compiler or generated code, no `sifr_output/`, no lockfile, no baseline churn, no unrelated reverts. `check_submodule_ownership.py`: PASS. `check_file_size_guardrails.py`: PASS. `check_docs_error_code_links.py`: pass. Submodule ownership is correct — the fixture edits live in `sifr-lang/leetcode` and reach the parent only as a pointer bump. The working tree's `third_party/ruff` modification and the untracked corpus artifacts (`.DS_Store`, `src/__pycache__/`, `src/sifr_output/`) are pre-existing, unrelated, and outside the commit.

The PR description is accurate on all six of its claims, including the two I could most easily have found overstated (the two-diagnostic reproduction and the "each pass check/build/run" claim across all three fixtures plus siblings).

## Findings

**Actionable findings: none.**

Non-blocking observations, requiring no change to PR #3085:

1. *Informational — `0146`/`0189` are behavior-neutral and were already green pre-change.* I ran the `a20d9d502` blobs of both fixtures on this head: `check` reports `no errors found` and `run` exits 0 with no panic for each. So neither edit repaired a failure reproducible against this compiler; both are forward-looking snapshot adaptations (commit titles: "Snapshot LRU head before mutable receiver calls", "Snapshot rotate length before mutable calls"), evidently aligned with the separately owned `plans/issues/active/ad-hoc-class-field-mutating-receiver-place-semantics.md` workstream, and they appear in no parent record other than this Wave 6 row. Calling them "fixture corrections" is loose in that light. If the ledger is revised for another reason, a clause noting they are behavior-neutral snapshots owned by the receiver-place workstream would improve traceability. Nothing the row asserts is false, and adopting them carries no semantic risk.
2. *Informational — the authoritative create-PR profile is not yet recorded for this head.* Every prior wave row records a `scripts/run_all_tests.sh --profile create-pr` result; the Wave 6 row records focused fixture validation and the ownership guard instead, and claims nothing more. Since the pointer bump changes verification *inputs*, I ran the lanes that actually consume them — `algorithmic_compatibility --suite profile-manifest`, plus the submodule, file-size, and docs-link guards — and all pass, so residual risk is minimal. Per AGENTS.md the profile run should still be recorded before this draft is merged.
3. *Informational — artifact head citation.* The adopted corpus artifact reviews `…..2c2ea24` (pre-squash branch head) while the parent pins `d50fa735`. Verified equivalent for the reviewed content: identical `0377` blob and a squash-parent diff of exactly the 15 deletions.
4. *Informational — out of range.* `plans/reviews/active/…wave-6-parent-agent-review-pass-1.md` currently exists as a 0-byte untracked placeholder for this pass. It is not part of the reviewed range, and I did not write to it.

## Validation assessment

| Claim | Independent result on `d1b688b08` |
| --- | --- |
| Pre-change `0377` yields two `SIFR-TYPE-0004` | Reproduced exactly — parameter `total` and `dfs` return type, both at line 13 |
| `0377` post-change check/build/run | `no errors found`; `run` exit 0, no panic |
| `0146` post-change check/build/run | `no errors found`; `run` exit 0, no panic |
| `0189` post-change check/build/run | `no errors found`; `run` exit 0, no panic |
| Python siblings pass | `0146`, `0189`, `0377` each exit 0; no `.py` file changed in the corpus range |
| `0377` Python sibling unchanged | Stronger: identical blob `416e7754…` at both range endpoints |
| Submodule ownership guard | `submodule ownership guardrail: PASS` |
| Corpus pass 1 approved, zero findings | Confirmed; its key evidence independently reproduced |
| Wave 5 merge + gate numbers | Confirmed against `gh pr view 3081` and the retained create-PR lane log |

The stated validation is accurate and, for a documentation-plus-gitlink change adopting a deletion-only fixture edit and two inert snapshots, sufficient. I extended it with the pre-change green-state probe on `0146`/`0189`, the `SIFR-FLOW-0901` control probe, an assertion-execution sanity check on `sifr run`, the squash-provenance diff, the profile/baseline staleness sweep including the `pinned_corpus` checksum consumer, the `profile-manifest` lane, and the Wave 5 lane-log reconciliation. None surfaced a defect.

## Final verdict

**Approved — zero actionable findings.** The parent range advances the corpus pointer to the merged head `d50fa735` and adopts three fixture changes that are all safe: `0377` is a minimal, deletion-only removal of provably unreachable code that was the sole cause of its two `SIFR-TYPE-0004` failures, with the live DP path, its assertion, and the Python parity oracle bit-for-bit intact; `0146` and `0189` are semantically inert snapshots verified green both before and after. The corrected separately-tracked wording is technically accurate against the compiler's actual behavior, the Wave 5 merged and gate evidence reconciles with primary artifacts, no baseline or profile surface became stale, submodule ownership and gitlink provenance are correct, the diff is clean and minimal, and the adopted review artifact is faithful.
