# Independent Review — Wave 5, PR #3081 (pass 12, final exact-head)

**Exact range:** `f1c34cf9aaabadda546e670fca190decc580c935` (base, merged Wave 4) … `cb1b51205436b61bb6fabd810d399140dff37b17` (head)
**Published PR head:** `gh pr view 3081` → `headRefOid = cb1b51205436b61bb6fabd810d399140dff37b17`, base `main`, state `OPEN`, `mergeable: MERGEABLE`, `mergeStateStatus: CLEAN` — the reviewed head is exactly the published head.
**Delta under review:** the single commit `cb1b51205` ("docs(plan): record Wave 5 approval") on top of the pass-11-approved implementation head `5e079b127`.
**Working tree:** not modified by this review. Pre-existing dirty submodule pointers (`third_party/ruff`, `verification/.../leetcode`) and the untracked pass-12 artifact placeholder are present but appear in no commit in the range.

## Methodology

- Enumerated the range commit-by-commit and confirmed the nine implementation commits `2fc5c6f76 … 5e079b127` are unchanged, with `cb1b51205` as the only addition.
- Verified the delta is documentation-only two ways: `git show --stat cb1b51205` (2 files, both under `plans/`), and a tree-hash comparison of `5e079b127` vs `cb1b51205` for `crates`, `scripts`, `verification`, `internal_docs`, `docs`, `demos`, `Cargo.lock`, `Cargo.toml`, `third_party` — **all identical**. No submodule pointer moves in the range.
- Read the added pass-11 report in full and cross-checked its factual claims against the repository at head rather than accepting them: head SHA, commit list, cited test/fixture line ranges, cited file sizes, internal arithmetic.
- Checked the ledger row's status wording, link target, and every new claim against the pass-11 report and against the unchanged surrounding text for claim drift.
- Checked mergeability context: `git merge-base HEAD origin/main` and file overlap with commits landed on `main` since the base.
- Per instruction, did not rerun the 679-fixture native sweep or workspace gates; the code tree is byte-identical to the tree pass 11 validated, so rerunning could not produce new signal about this delta.

## Delta verification

**1. `plans/reviews/active/…wave-5-claude-opus-review-pass-11.md` (+88, new file)**

| Claim in the report | Verified against repo | Result |
|---|---|---|
| Head `5e079b1276b77a5e8cf4ee76c430f2cb5e051d54` | `git rev-parse` | ✓ exact |
| 9 commits `2fc5c6f76 … 5e079b127` | `git log f1c34cf9aa..5e079b127` | ✓ exact, same order |
| Pass-10 head was `d819a5a82` | pass-10 report line 7 | ✓ |
| `5e079b127` is test-and-ledger only, no emitter/lowering change | `git show --numstat` | ✓ (fixture, codegen test, ledger, pass-10 artifact only) |
| Set assertions at codegen test `:132-146`; list at `:162-176` | read file | ✓ both exact, including the `key < preinsert < items < bucket` chain and the retained `retain(` ≥ 4 / no-`.clone().retain(` assertions |
| Native function at fixture `:185-192`, `update({len(groups)}, {1})`, asserts len 2 / `1 in` / `2 in`, wired into `main` as `== 2` | read file | ✓ exact |
| File sizes 833 / 203 / 871 / 829 / 289 / 177 / 213, all < 900 | `wc -l` at head | ✓ all seven exact |
| Self-label "pass 11" matches filename and ledger link | ✓ (the wave-3 pass-11 mislabel class does not recur) |
| Internal arithmetic (950 passed + 1 failed vs 951 total) | ✓ consistent |

**2. `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` (1 line changed)**

- Status `implemented; review remediation pending` → `approved; [PR #3081](…/pull/3081) pending exact-head merge gate`. Accurate: PR #3081 is open and unmerged, and the row does not claim a merge. Consistent with the table's convention (merged waves use `merged; PR link`).
- Link target `../../reviews/active/…wave-5-claude-opus-review-pass-11.md` resolves from `plans/issues/active/` to the file added in the same commit. ✓
- The appended clause — "independently mutation-tested both list and set pre-insertion sites, revalidated the complete Wave 5 behavior and all prior remediations, and approved the exact implementation head with zero actionable findings" — is a fair, non-inflating summary of the pass-11 report (§ mutation matrix, § pass-1–9 re-probe, verdict APPROVE, "None. No actionable findings").
- No other text in the row changed; the evidence prose (12/12 focused, 920/951 full suites, `679/679` post-pass-7…post-pass-10 sweeps) is untouched and no new gate or sweep is claimed for this head. No claim mismatch introduced.
- Line 330 is a single well-formed table row; the pass-11 entry follows the same `[Opus pass N](…)` pattern as passes 1–10. No other section of the document required updating for this delta (the Wave 5 narrative at lines 279–303 is status-neutral).

**3. Suitability of the complete published head for merge**

- No non-documentation change exists after the approved implementation head — proven by tree-hash equality, not just by diffstat.
- `git merge-base HEAD origin/main` = `f1c34cf9aa`, i.e. the reviewed range is exactly the PR's diff against its merge base. `main` has since advanced to `9893f2d9c` (Rust-interop closure work); the intersection of files touched by the branch and files touched by `main` since the base is **empty**, and GitHub reports `MERGEABLE / CLEAN`.
- The authoritative `scripts/run_all_tests.sh --profile create-pr` gate on the exact prospective merge is still outstanding, which is precisely what the new status wording ("pending exact-head merge gate") records.

## Findings

**None.** Zero actionable findings.

### Non-blocking observations (not findings; no action required for approval)

1. **Off-by-one in a descriptive stat in the pass-11 report** — line 5 describes `5e079b127` as "2 non-artifact files, **+28/−1**". The actual non-artifact delta is fixture `11+/0−` plus codegen test `16+/1−` = **+27/−1** (no grouping of the four files yields +28/−1; including the ledger gives +28/−2). Purely descriptive; it does not touch any behavioral claim, gate result, or the verdict, and every other figure in the report reproduces exactly.
2. **Pass-11's own carried-forward observations remain open by design** — the shadowed `collections.deque()` arm in `binding_hint_adoption.rs`, the silent `list.remove`-of-absent no-op, and the nested-return strictness narrowing. All three were explicitly classified non-blocking and pre-existing/intended by pass 11; nothing in the docs delta claims otherwise.
3. **The branch is behind `main`** with no file overlap. Whether to rebase/merge before the gate is a process call; the ledger's Wave 3 precedent records running the gate on the exact prospective merge, which this row's wording already anticipates.

## Verdict: **APPROVE**

The `f1c34cf9aa…cb1b51205` head differs from the pass-11-approved implementation head `5e079b127` only by documentation: the standalone pass-11 report and one ledger row. The non-documentation trees are byte-identical, so pass 11's validation transfers to this head without reservation. The pass-11 report's head, commit list, cited line ranges, and cited file sizes all reproduce exactly against the repository; the ledger's new status, link, and summary clause are accurate, correctly targeted, and introduce no claim the evidence does not support. The published PR head matches the reviewed head, merges cleanly, and does not overlap files changed on `main` since the base. Zero actionable findings — approved for the authoritative local `create-pr` gate and merge.
