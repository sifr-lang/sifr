# Review — sifr-lang/leetcode PR #41 (`4fdb439` vs `7772857`)

## Scope

One file, one hunk: `src/0189_rotate_array.sifr` hoists `len(nums)` into a local `nums_len` before the three `_reverse_range` calls, so the mutable-borrow argument and the length read no longer occur in the same call expression.

## Verification performed

I built a **clean compiler from the parent repo's exact HEAD** (`581b363aa`) in a throwaway worktree with a separate target dir, because the parent working tree carries uncommitted edits to `crates/sifr_lowering/src/lower/method_call_verifier.rs` and its prebuilt `target/release/sifr` is newer than those edits. All results below are from the clean HEAD binary unless labelled otherwise. The temp worktree was removed; no repo file was modified.

**Diagnostic is real and the fix resolves it**

| fixture | `check` | `build` | `run` |
|---|---|---|---|
| base `0189` | fail — `SIFR-OWN-0002` ×2 (lines 22, 24) | — | — |
| head `0189` | pass (`no errors found`, exit 0) | exit 0 | exit 0, asserts hold |

Base fails at exactly the two sites the diff touches; the third call (`_reverse_range(nums, 0, rot - 1)`) never read `len`, and correctly needed no change.

**Semantic equivalence** — I ran the head `rotate` against the paired Python reference's `k % n` semantics over 11 cases: empty, `n=1` with `k=0` and `k=5`, `k=0`, `k=n`, `k=2n`, `k=100` on `n=6`, `k=7` on `n=5`, `n=2`, plus both in-fixture cases. All 11 Sifr results match the Python reference exactly (`all-ok`).

**Snapshot placement and validity** — Correct on both counts:
- `nums_len` is assigned *after* the empty guard and *after* the `while rot >= len(nums)` normalization loop, so it cannot be read on an empty list and cannot be stale relative to normalization. The remaining `len(nums)` reads in the loop condition are plain reads with no mutable borrow in the same expression, which is why they don't need hoisting.
- `_reverse_range` only swaps existing elements — no insert, remove, append, or reassignment — so length is invariant across all three calls and the single snapshot stays valid for calls 1 and 3. `rot == 0` yields `_reverse_range(nums, 0, -1)`, a no-op (`0 < -1` false), and the outer reverse then self-cancels — matching Python's `k % n == 0` path.

No functional finding in the diff.

## Findings

### 1. (Low, evidence integrity) The stated "exactly four failures" sweep baseline is not reproducible on the parent exact head — the count there is five

A `check` sweep of all 411 corpus fixtures on the clean parent-HEAD compiler yields **five** failures, not four:

```
0002_add_two_numbers                        SIFR-TYPE-0002
0036_valid_sudoku                           SIFR-TYPE-0005
0086_partition_list                         SIFR-TYPE-0002
0297_serialize_and_deserialize_binary_tree  SIFR-INTERNAL-0001
0377_combination_sum_iv                     SIFR-TYPE-0004
```

The extra one is masked by the parent's uncommitted compiler edit. On clean HEAD, `0297` reports:

```
error[SIFR-INTERNAL-0001]: internal compiler error: mutable source call 'dfs' has no checked argument place
```

On the working-tree binary the same file emits only a `SIFR-TYPE-0901` overflow warning and passes — exactly the behaviour change expected from the uncommitted removal of the `missing_declared_place` branch in `method_call_verifier.rs`. So the reported baseline was measured with a compiler that is not the parent exact head, and any pass/fail claim sourced from that binary is not attributable to `581b363aa`.

Actionable: re-run the sweep against a clean build of the parent exact head and record the baseline as five, or state explicitly which local compiler state the count was taken from.

Two mitigating facts, so this does not put the fixture change itself in doubt: `0189` head passes `check` on **both** binaries, and base `0189` fails with the same two `SIFR-OWN-0002` errors on **both**. The remediation is necessary and sufficient independent of the compiler-state question, and `0297` is a pre-existing, unrelated parent-repo defect that this PR neither causes nor touches.

## Non-blocking observations (no action requested)

- `while rot >= len(nums)` still reads `len(nums)` per iteration and is O(k/n) rather than a modulo. Pre-existing, unchanged, and not a borrow-check concern. Hoisting `nums_len` above that loop and writing `while rot >= nums_len` would be marginally more consistent with the new local, but is pure style.
- `_reverse_range`'s `if left is None or right is None: return` guard is dead for in-range indices but is the existing Option-discharge idiom in this corpus; untouched by the PR.

The one-line change is correct, minimal, and independently validated. The single finding is about the accuracy of the reported sweep baseline, not about the diff.

VERDICT: NOT SATISFIED
