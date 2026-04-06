# Review (Pass 1): Ad-hoc Operator/Truthiness + Contract Closure Phase (2026-04-07)

Reviewed doc: `issues/ad-hoc-operator-truthiness-contract-closure-2026-04-07.md`
Source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun1.json`
Source results: `verification/leetcode/full_corpus_current_results_20260407_live_rerun1.json`
Reviewer: implementation-readiness pass

## TL;DR

- **Bucket counts and fixture membership are correct.** All 14 fixtures map 1:1 to the three buckets in the source artifacts.
- **Resolution lane (sifr_adaptation) is correct for all 14 fixtures.** None of these merit a Sifr language/compiler change.
- **The phase is NOT implementation-ready as written.** Eight fixtures (0371, 0473, 0516, 0735, 0931, 0973, 1220, 1514) have multi-diagnostic stderr that the doc treats as if it were a single-diagnostic fix. Fixing only the first diagnostic will move them to a different bucket but **leave them failing under check**.
- **Phase exit criteria is technically achievable but semantically hollow.** "Out of the three target buckets" is satisfied by addressing only the first reported diagnostic; the doc never commits to "fixture passes check + main()". This needs to be tightened or the workstreams need expansion.
- **Workstream 4 (validation) does not gate on net failure-count reduction**, so a successful "phase exit" could co-exist with zero net improvement in `failing_cases`.

## 1. Bucket Membership / Fixture List Validation

Cross-checked against `full_corpus_failure_taxonomy_20260407_live_rerun1.json`:

| Bucket | Doc count | Taxonomy count | Match |
|---|---|---|---|
| `operator_and_truthiness_typing_gap` | 11 | 11 | ✅ |
| `callable_argument_contract_mismatch` | 1 | 1 | ✅ |
| `return_path_and_function_contract_gap` | 2 | 2 | ✅ |
| **Total** | **14** | **14** | ✅ |

Fixture slugs in scope match exactly. No drift.

## 2. Per-Fixture Validation

Legend: **Doc lane** = doc-proposed resolution lane. **Closure verdict** = whether the doc's prescribed action will actually exit the failure list (not just exit the bucket).

### `operator_and_truthiness_typing_gap` (11)

#### 1. `0007_reverse_integer` — ✅ correct, complete
- Stderr (1 diag): `while condition must be bool ... got 'int'`
- Doc fix: `while x != 0`
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** clean. Will exit failure list after WS1 rewrite.
- Note: relies on `math.fmod` and `int(x / 10)` continuing to be supported — verify in targeted check.

#### 2. `0068_text_justification` — ✅ correct, complete
- Stderr (1 diag): `if condition must be bool ... got 'int'` (the `if remainder` line)
- Doc fix: `if remainder != 0`
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** clean. Note also has `line, length = [], 0` pattern (multi-target assign); not currently flagged, leave untouched.

#### 3. `0201_bitwise_and_of_numbers_range` — ✅ correct, complete
- Stderr (2 diags): `bad operand type for unary not: 'int'` + `duplicate function definition in module: 'rangeBitwiseAnd'`
- Doc fix: `if bit == 0` + keep one implementation
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** clean. Both diagnostics are addressed; doc correctly cross-routes the duplicate-def part to WS3.

#### 4. `0371_sum_of_two_integers` — ⚠️ doc captures core, prescription too vague
- Stderr (4 diags):
  ```
  '<' not supported between instances of 'bool' and 'int'
  bad operand type for unary not: 'int'
  cannot compare 'bool' and 'int' with ==
  return type mismatch: expected 'int', got 'bool'
  ```
- Root cause: untyped nested `def add(a, b)` whose return path `return a or b` infers as `bool` (because `a or b` short-circuits and the surrounding `not a / not b` taints the inference).
- Doc fix: "explicit int guards and typed helper signature"
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** likely OK, but requires the rewrite to be MORE explicit than the doc states. The fix must:
  1. Annotate `add(a: int, b: int) -> int`
  2. Replace `if not a or not b: return a or b` with explicit `if a == 0: return b\nif b == 0: return a`
  3. Recheck `if add(~a, 1) == b` and `if add(~a, 1) < b` (these become int comparisons once `add` is typed)
- **Required correction:** doc should state the rewrite explicitly so downstream implementer doesn't introduce another bool-tainted return path.

#### 5. `0416_partition_equal_subset_sum` — ✅ correct, complete
- Stderr (1 diag): `if condition must be bool ... got 'int'`
- Doc fix: `if (sum(nums) % 2) != 0`
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** clean.

#### 6. `0473_matchsticks_to_square` — 🛑 **cross-bucket blocker**, doc severely under-specifies
- Stderr (3 diags):
  ```
  cannot compare 'float' and 'int' with !=
  sort() got an unexpected keyword argument 'reverse'
  unsupported operand type(s) for +: 'int | None' and 'int'
  ```
- Doc fix: `sum % 4 != 0` and `//` for side length. Note "secondary non-scoped diagnostics; keep rewrites local and explicit".
- **Closure verdict:** FAILS. After the float/int fix:
  - `matchsticks.sort(reverse=True)` still trips `python_stdlib_and_builtin_parity_gap` (sort signature)
  - `sides[j] + matchsticks[i]` still trips `optional_none_flow_and_narrowing_gap` (subscript-returns-Optional)
  - Fixture moves to a different bucket. Phase exit criterion ("out of the three target buckets") is technically met but the fixture still fails check.
- Additional latent risk: `backtrack` is a recursive nested function that mutates closure-captured `sides[j]` via `sides[j] += matchsticks[i]`. Element mutation may or may not trigger `nonlocal_mutable_capture_not_supported` — needs validation.
- **Required correction:** either (a) extend the in-scope WS1 prescription to include `matchsticks.sort(); matchsticks.reverse()` and an explicit guard pattern for the `int|None` subscript, or (b) explicitly list 0473 as "partial closure only" in the phase doc with the residual diagnostic owners noted. Without one of these, this fixture cannot exit the failing-cases list.

#### 7. `0735_asteroid_collision` — ⚠️ **cross-bucket blocker**, doc misses second diagnostic
- Stderr (2 diags):
  ```
  if condition must be bool ... got 'int'
  unsupported operand type(s) for +: 'int' and 'int | None'
  ```
- Doc fix: `if a != 0`
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** FAILS. The second diagnostic comes from `diff = a + stack[-1]` where `stack[-1]` is typed `int | None`. After the truthiness fix, the fixture moves to `optional_none_flow_and_narrowing_gap` and stays failing.
- **Required correction:** doc must include an explicit subscript-narrow for `stack[-1]` (e.g., bind `top = stack[-1]; if top is not None: ...`) or take this as out-of-scope and mark the fixture as partial closure.

#### 8. `0846_hand_of_straights` — ✅ correct, complete
- Stderr (1 diag): `if condition must be bool ... got 'int'`
- Doc fix: `if (len(hand) % groupSize) != 0`
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** clean.

#### 9. `0973_k_closest_points_to_origin` — 🛑 **cross-bucket blocker**, doc severely under-specifies
- Stderr (8 diags):
  ```
  cannot compare 'list[list[int]]' and 'list[tuple[int, int]]' with == (×2)
  cannot unpack non-tuple type 'None | tuple[int, float, float]'
  for loop tuple target expects iterable elements of tuple type, got 'list[int]'
  return type mismatch: expected 'list[list[int]]', got 'list[tuple[float, float]]'
  type 'tuple[int, float, float]' does not implement protocol 'Comparable' required by type parameter 'T' (×2)
  undefined variable: 'x'
  ```
- Doc fix: "normalize to one representation (`list[list[int]]` end-to-end)"
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** FAILS as written. The doc's normalization is necessary but not sufficient. Concrete blockers the doc does not address:
  1. `for x, y in points:` where `points: list[list[int]]` is the same destructuring pattern that fails in `1029_two_city_scheduling` (`destructuring_and_assignment_target_surface_gap`). Must rewrite to indexed access.
  2. `_, x, y = heapq.heappop(minHeap)` where `heappop` returns `Optional[T]` — needs guard or unwrap (`optional_none_flow_and_narrowing_gap`).
  3. Heap element is `tuple[int, int, int]` (or `tuple[float, int, int]` after dist) which does not implement `Comparable` per Sifr's heap protocol — must wrap as `(dist,)` key or use a sortable stub.
  4. The assertion uses tuple literals `[(-2, 2)]`. Normalizing to `list[list[int]]` requires rewriting the assertions to `[[-2, 2]]`.
- **Required correction:** rewrite this fixture's prescription as a multi-step plan covering points 1-4, OR exclude 0973 from this phase and route it to a multi-bucket joint phase.

#### 10. `1220_count_vowels_permutation` — ⚠️ doc captures core, missing details
- Stderr (many):
  ```
  cannot compare 'Never' and 'str' with == (×N)
  function 'countVowelPermutation' must return a value of type 'int' on all control-flow paths
  parameter 'c' missing type annotation
  parameter 'n' missing type annotation
  return type mismatch: expected 'int', got 'Any | None'
  unsupported operand type(s) for -: 'Never' and 'int'
  ```
- Doc fix: "explicit parameter types and deterministic return-path typing"
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** likely OK if rewrite is exhaustive. Required additions to doc prescription:
  1. Annotate `def countVowelPermutation(n: int, c: str = '') -> int:`
  2. Add a fall-through `return 0` (or a default branch) so all paths return int. The current `else` block is a chain of `if c == 'a' / 'e' / 'i' / 'o' / 'u' / ''` with no terminal return for unmatched `c`.
  3. Confirm module-level `Memo = {}` infers as `dict[tuple[str, int], int]` from usage (do not require explicit annotation unless inference fails after step 1).
- Note: this fixture's *primary diagnostic* is in `operator_and_truthiness_typing_gap` but its *real bucket* after rewrite is `return_path_and_function_contract_gap` — same phase, but workstream allocation should make this explicit (currently in WS2).

#### 11. `1514_path_with_maximum_probability` — 🛑 **cross-bucket blocker**, doc misidentifies root cause
- Stderr (9 diags):
  ```
  cannot compare 'float' and 'int' with ==
  cannot unpack non-tuple type 'None | tuple[int, int]'
  cannot unpack non-tuple type 'list[int]'
  type 'tuple[int, int]' does not implement protocol 'Comparable' required by type parameter 'T'
  undefined variable: 'cur' (×3)
  undefined variable: 'dst'
  undefined variable: 'src'
  ```
- Doc fix: "compare with `0.0` and keep graph state/tuple shapes explicit"
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** FAILS as written. The doc's "compare with 0.0" addresses none of the actual blockers:
  1. `pq = [(-1, start)]` then `heappush(pq, (prob * edgeProb, nei))` — element types diverge int→float. Must initialize as `pq: list[tuple[float, int]] = [(-1.0, start)]`.
  2. `prob, cur = heapq.heappop(pq)` — `heappop` returns `Optional[T]`; needs guard or unwrap (the `undefined variable: 'cur'` cascade is a downstream effect of this unpack failing).
  3. `src, dst = edges[i]` where `edges[i]: list[int]` — same destructuring blocker as 0973 / 1029.
  4. `tuple[int, int]` heap element does not implement `Comparable` — needs `(prob,)` key or wrapper.
  5. `return 0` at the bottom; signature returns `float`. Must be `return 0.0`.
- **Required correction:** rewrite the prescription to enumerate the five steps above, OR exclude 1514 from this phase. The current one-liner is insufficient.

### `callable_argument_contract_mismatch` (1)

#### 12. `0931_minimum_falling_path_sum` — ⚠️ **cross-bucket blocker**, doc captures core
- Stderr (13 diags, condensed):
  ```
  argument 2 of callable 'Path': expected 'int', got 'float'
  cannot index type 'list[int] | None' with '0' (×2)
  dict subscript assignment value type 'Unknown' is not compatible with dict value type 'int' (×2)
  return type mismatch: expected 'int', got 'int | None'
  unsupported operand type(s) for +: 'Any' and 'int' (×2)
  unsupported operand type(s) for +: 'int | None' and 'int' (×5)
  ```
- Doc fix: "explicit helper parameter annotations and integer-only index path"
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** partial. Annotating `def Path(i: int, k: int, n: int) -> int` and `Memo: dict[tuple[int, int], int] = {}` should resolve the callable contract diagnostic and the dict-subscript Unknown errors. BUT `matrix[i][k]` and `matrix[0]` produce `list[int] | None` (subscript-returns-Optional) — the fixture will then fail under `optional_none_flow_and_narrowing_gap`.
- **Required correction:** doc must include explicit guard or unwrap pattern for `matrix[i][k]` access (bind to local with narrowing), or accept partial closure and document residual.
- Note: this fixture is genuinely in `callable_argument_contract_mismatch` — once helper is typed, the callable diagnostic disappears. The remaining failures move to a different bucket. Same hollow-exit risk as the WS1 cluster.

### `return_path_and_function_contract_gap` (2)

#### 13. `0162_find_peak_element` — ✅ correct, complete
- Stderr (1 diag): `undefined variable: 'mid'`
- Doc fix: explicit initialization + empty-input guard
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** clean. Initialize `mid = 0` before loop. Existing test `findPeakElement([1,2,3,1])` is non-empty so behavior is preserved.

#### 14. `0516_longest_palindromic_subsequence` — 🛑 **cross-bucket blocker**, doc misses core failure
- Stderr (7 diags):
  ```
  duplicate function definition in module: 'longestPalindromeSubseq'
  max() with 2 arguments does not accept optional operands; got 'int | None' and 'int | None' (×2)
  max() with 2 arguments does not accept optional operands; got 'int' and 'int | None'
  return type mismatch: expected 'int', got 'int | None'
  unsupported operand type(s) for +: 'int | None' and 'int | None'
  unsupported operand type(s) for +: 'int' and 'int | None'
  ```
- Doc fix: "single canonical implementation"
- Lane: `sifr_adaptation` ✅
- **Closure verdict:** FAILS as written. Removing the duplicate definition is necessary but the surviving impl is the source of the 6 `int | None` errors (`dp[i][j]` 2D list subscript returns Optional, then propagates through `max()` and `+`). Even after dedup, the fixture stays failing under `optional_none_flow_and_narrowing_gap` / `python_stdlib_and_builtin_parity_gap` (`max` with optional operands).
- **Required correction:** the WS3 prescription must commit to either:
  1. Replacing the kept implementation with the LCS variant at lines 46-61 (which uses `dp[i+1][j+1]` and clean int math, no negative indices) and removing both the first impl and the dead code block.
  2. Or rewriting the kept impl to add explicit subscript guards / unwrap for `dp[i][j]` and the recurrence terms.
- Recommendation: option 1 (keep the LCS variant) is the smaller, cleaner closure path.

## 3. Resolution Lane Audit (compiler vs adaptation)

| # | Fixture | Doc lane | Verdict | Justification |
|---|---|---|---|---|
| 1 | 0007 | sifr_adaptation | ✅ | Banned by guardrail (numeric truthiness) |
| 2 | 0068 | sifr_adaptation | ✅ | Banned by guardrail (numeric truthiness) |
| 3 | 0201 | sifr_adaptation | ✅ | `not int` banned + dup-def is fixture hygiene |
| 4 | 0371 | sifr_adaptation | ✅ | Untyped helper + numeric truthiness |
| 5 | 0416 | sifr_adaptation | ✅ | Banned by guardrail |
| 6 | 0473 | sifr_adaptation | ✅ | Banned by guardrail (numeric widening) |
| 7 | 0735 | sifr_adaptation | ✅ | Banned by guardrail |
| 8 | 0846 | sifr_adaptation | ✅ | Banned by guardrail |
| 9 | 0973 | sifr_adaptation | ✅ | Fixture inconsistency, not language |
| 10 | 1220 | sifr_adaptation | ✅ | Untyped fixture surface |
| 11 | 1514 | sifr_adaptation | ✅ | Mixed numeric containers in fixture |
| 12 | 0931 | sifr_adaptation | ✅ | Untyped helper |
| 13 | 0162 | sifr_adaptation | ✅ | Definite-assignment is correct policy |
| 14 | 0516 | sifr_adaptation | ✅ | Multi-impl is fixture hygiene |

**Lane verdict: all 14 fixtures are correctly classified as `sifr_adaptation`.** No compiler relaxation is justified for any of these. The doc's "Required compiler changes for this scoped phase: `none`" is correct.

## 4. Cross-Bucket Blockers Summary

The doc's central technical risk is that **8 of 14 fixtures have multi-diagnostic stderr spanning multiple taxonomy buckets**, and the doc's prescriptions only address the first diagnostic in 4 of those 8. Result: even on a successful execution, those fixtures will not reach `PASS` status — they will simply move from one failure bucket to another.

| Fixture | First diagnostic bucket | Latent blocker bucket(s) after WS rewrite |
|---|---|---|
| 0371 | operator_and_truthiness_typing_gap | (none — likely clean if rewrite is explicit) |
| 0473 | operator_and_truthiness_typing_gap | python_stdlib_and_builtin_parity_gap, optional_none_flow_and_narrowing_gap |
| 0735 | operator_and_truthiness_typing_gap | optional_none_flow_and_narrowing_gap |
| 0973 | operator_and_truthiness_typing_gap | destructuring_and_assignment_target_surface_gap, optional_none_flow_and_narrowing_gap, other_type_surface_and_api_mismatch |
| 1220 | operator_and_truthiness_typing_gap | (closes inside this phase if return-path branch added) |
| 1514 | operator_and_truthiness_typing_gap | destructuring_and_assignment_target_surface_gap, optional_none_flow_and_narrowing_gap, other_type_surface_and_api_mismatch |
| 0931 | callable_argument_contract_mismatch | optional_none_flow_and_narrowing_gap |
| 0516 | return_path_and_function_contract_gap | optional_none_flow_and_narrowing_gap, python_stdlib_and_builtin_parity_gap |

## 5. Workstream Allocation Issues

- **WS1 owns 1514 but not 0973**, even though both have analogous structural typing issues plus identical destructuring + Optional + Comparable cascades. They should either both be in WS1 or both in WS2. Pick one and document the criterion.
- **WS2 owns 1220** but 1220's residual issue (return-path exhaustiveness) is actually WS3 territory. Either move to WS3 or acknowledge the cross-workstream coupling.
- **WS3 owns "duplicate-definition part of 0201"** — this is fine as a partial-ownership entry but the execution checklist should make the cross-WS handoff explicit so the duplicate-def is not silently dropped.

## 6. Phase Exit Criteria — Tightening Required

Current criteria:
> - all 14 scoped fixtures are out of the three target buckets
> - full run artifacts regenerated and committed
> - no new regressions in `codegen_runtime_build_gap`

**Issues:**
1. "Out of the three target buckets" is satisfied by addressing only the first diagnostic. Fixture 0473 moves from `operator_and_truthiness_typing_gap` to `python_stdlib_and_builtin_parity_gap` — exit criteria satisfied, fixture still failing.
2. Criteria does not require net `failing_cases` reduction. A pessimistic outcome is: 14 fixtures exit the three buckets, ~8 of them re-enter other buckets, net `failing_cases` reduction is 6, not 14.
3. No regression check on `optional_none_flow_and_narrowing_gap`, `destructuring_and_assignment_target_surface_gap`, or `python_stdlib_and_builtin_parity_gap`, which are the most likely receivers of bucket-shifted fixtures.

## 7. Implementation-Readiness Verdict

**Status: NOT READY** as written. The phase will execute, but it will not deliver clean closure for ~half its scope.

- **Cleanly closable as written (6):** 0007, 0068, 0162, 0201, 0416, 0846 — these have single-diagnostic stderr and the doc's prescription is sufficient.
- **Closable with minor doc tightening (2):** 0371, 1220 — prescriptions need to be made explicit, no scope expansion.
- **Closable only with scope expansion (4):** 0473, 0735, 0931, 0516 — doc prescription must add Optional-narrowing and stdlib adaptations to genuinely close the fixture.
- **Should be deferred or rewritten (2):** 0973, 1514 — multi-diagnostic structural rewrites that exceed the implied scope of this phase. Either commit to the full multi-step rewrite in the doc, or move them to a multi-bucket joint phase.

## 8. Required Corrections

**Mandatory before phase can be marked implementation-ready:**

### A. Tighten phase exit criteria
Replace the existing criteria block with:
```
- all 14 scoped fixtures reach `PASS` status (check + main()), OR are explicitly
  documented as partial-closure with named residual diagnostic owners
- net `failing_cases` reduction must be >= 12 (allow 2 partial closures)
- no regressions in: codegen_runtime_build_gap, optional_none_flow_and_narrowing_gap,
  destructuring_and_assignment_target_surface_gap, python_stdlib_and_builtin_parity_gap
- regenerated taxonomy committed alongside results JSON
```

### B. Expand per-fixture prescriptions for the eight cross-bucket fixtures
For each of 0371, 0473, 0516, 0735, 0931, 0973, 1220, 1514: replace the one-line "canonical fix" with the explicit multi-step rewrite spelled out in §2 of this review. Each step should reference its target diagnostic so a downstream implementer can validate it locally.

### C. Decide explicit policy for 0973 and 1514
Pick one:
1. Keep them in scope and adopt the multi-step rewrites (acknowledging the phase grows in surface area).
2. Drop them from this phase, mark as `deferred_to_multi_bucket_joint_phase`, and update the totals in the Scope section to `12/14`.

Recommendation: option 2. These two fixtures dilute the phase's "operator/truthiness/contract closure" focus.

### D. Re-allocate workstream ownership consistently
- Move 0973 to WS2 if kept (✓ already there) or remove if dropped.
- Either move 1220 to WS3 or add a sub-bullet under WS2 acknowledging the WS3-class residual work.
- Add an explicit sub-task under WS3 for "remove duplicate def from 0201" so the cross-WS handoff is visible on the execution checklist.

### E. Add a verification probe to WS4
Add to the validation protocol:
```
- after each workstream applied, run targeted check on its fixtures and parse stderr;
  any fixture that still has any diagnostic must be either rewritten further or
  explicitly marked partial-closure before moving on
- after full corpus rerun, diff failing_cases by category vs the 20260407 baseline;
  any net increase in optional_none_flow_and_narrowing_gap,
  destructuring_and_assignment_target_surface_gap, or
  python_stdlib_and_builtin_parity_gap is a phase blocker
```

## 9. Net Assessment

- Bucket math: ✅
- Lane assignment: ✅ (all sifr_adaptation, no compiler change)
- Per-fixture diagnostic understanding: ⚠️ (correct for primary diag, missing secondary diags on 8 fixtures)
- Closure prescriptions: ⚠️ (sufficient for 6, insufficient for 8)
- Workstream allocation: ⚠️ (minor coherence issues)
- Exit criteria: 🛑 (allows hollow exit)
- Validation protocol: ⚠️ (no bucket-shift detection)

**Recommendation: return to author for revision per §8 corrections A-E. Re-review after revisions.**
