# Ad-hoc Phase: Operator/Truthiness + Callable/Return Contract Closure (2026-04-07)

Status: done
Owner: phase_ad_hoc_operator_truthiness_contract_closure
Source run artifact: `verification/leetcode/full_corpus_current_results_20260407_live_rerun1.json`
Source taxonomy artifact: `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun1.json`

## Closure Snapshot (2026-04-07)

- scoped 14 fixtures: `PASS` on both `check` and `run`:
  - `verification/leetcode/ad_hoc_operator_truthiness_contract_closure_20260407_scoped_check_run_rerun2.tsv`
- full-corpus rerun2 artifacts:
  - `verification/leetcode/full_corpus_current_results_20260407_live_rerun2.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun2.json`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun2.md`
  - `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun2_delta_vs_rerun1.md`
- exit-criteria deltas:
  - `failing_cases`: `52 -> 38` (`-14`)
  - no-regression gates held:
    - `codegen_runtime_build_gap`: `5 -> 5`
    - `optional_none_flow_and_narrowing_gap`: `1 -> 1`
    - `destructuring_and_assignment_target_surface_gap`: `1 -> 1`
    - `python_stdlib_and_builtin_parity_gap`: `10 -> 10`
- implementation PRs:
  - `https://github.com/sifr-lang/sifr/pull/1592` (WS1)
  - `https://github.com/sifr-lang/sifr/pull/1593` (WS2)
  - `https://github.com/sifr-lang/sifr/pull/1594` (WS3)

## Scope

Target buckets from the latest full scan:

- `operator_and_truthiness_typing_gap`: `11`
- `callable_argument_contract_mismatch`: `1`
- `return_path_and_function_contract_gap`: `2`

Total scoped fixtures: `14`.

Scoped fixture list:

- `0007_reverse_integer`
- `0068_text_justification`
- `0201_bitwise_and_of_numbers_range`
- `0371_sum_of_two_integers`
- `0416_partition_equal_subset_sum`
- `0473_matchsticks_to_square`
- `0735_asteroid_collision`
- `0846_hand_of_straights`
- `0973_k_closest_points_to_origin`
- `1220_count_vowels_permutation`
- `1514_path_with_maximum_probability`
- `0931_minimum_falling_path_sum`
- `0162_find_peak_element`
- `0516_longest_palindromic_subsequence`

## Canonical Language Policy (Do Not Relax)

- No Python-style numeric truthiness for numeric scalars (`if x`, `while x` where `x: int/float`).
- No implicit numeric widening/coercion for mixed numeric comparisons (`int` vs `float`).
- No implicit auto-unwrap of `Optional[T]` values.
- No semantic language broadening in this phase. Keep fixes in fixture adaptation lane.

## Category Breakdown and Root Causes

### `operator_and_truthiness_typing_gap` (11)

#### Simple truthiness/operator rewrites (single-diagnostic closures)

1. `0007_reverse_integer`
- First diagnostic: `while condition must be bool ... got 'int'`.
- Root cause: Python `while x` int-truthiness pattern.
- Required fix lane: `sifr_adaptation`.
- Closure recipe: `while x != 0`.

2. `0068_text_justification`
- First diagnostic: `if condition must be bool ... got 'int'`.
- Root cause: Python `if remainder` int-truthiness pattern.
- Required fix lane: `sifr_adaptation`.
- Closure recipe: `if remainder != 0`.

3. `0416_partition_equal_subset_sum`
- First diagnostic: `if condition must be bool ... got 'int'`.
- Root cause: Python `if sum(nums) % 2` pattern.
- Required fix lane: `sifr_adaptation`.
- Closure recipe: `if (sum(nums) % 2) != 0`.

4. `0846_hand_of_straights`
- First diagnostic: `if condition must be bool ... got 'int'`.
- Root cause: Python `if len(hand) % groupSize` pattern.
- Required fix lane: `sifr_adaptation`.
- Closure recipe: `if (len(hand) % groupSize) != 0`.

#### Multi-diagnostic closures (must be fixed to PASS, not bucket-shift)

5. `0201_bitwise_and_of_numbers_range`
- Diagnostics: unary-not on int + duplicate function definition.
- Root cause: numeric truthiness/operator shortcut and two solution variants left in one module.
- Required fix lane: `sifr_adaptation`.
- Closure recipe:
  - Rewrite `if not bit` to `if bit == 0`.
  - Keep one `rangeBitwiseAnd` definition only.

6. `0371_sum_of_two_integers`
- Diagnostics: bool-int compare, unary-not on int, return type mismatch (`int` vs `bool`).
- Root cause: untyped nested helper with `not a or not b` and `return a or b`, which taints return inference to bool.
- Required fix lane: `sifr_adaptation`.
- Closure recipe:
  - Type helper: `def add(a: int, b: int) -> int`.
  - Replace `if not a or not b: return a or b` with explicit int guards:
    - `if a == 0: return b`
    - `if b == 0: return a`
  - Keep comparisons operating on typed-int values only.

7. `0473_matchsticks_to_square`
- Diagnostics: float-vs-int compare, unsupported `sort(reverse=True)`, optional arithmetic on indexed list values.
- Root cause: Python-centric numeric/operator convenience + unsupported sort keyword parity + Optional index surface not narrowed.
- Category note: this fixture remains in `operator_and_truthiness_typing_gap` because the first/trigger diagnostic is numeric operator typing (`float` vs `int` compare), while structural secondary diagnostics are closed in the same recipe.
- Required fix lane: `sifr_adaptation`.
- Closure recipe:
  - Replace `sum(matchsticks) / 4 != length` with integer-safe check:
    - `total = sum(matchsticks)`
    - `if (total % 4) != 0: return False`
    - `length = total // 4`
  - Replace `matchsticks.sort(reverse=True)` with parity-safe sequence:
    - `matchsticks.sort()` then `matchsticks.reverse()`.
  - Narrow `sides[j]` before arithmetic (bind local, guard `is not None`, then update).

8. `0735_asteroid_collision`
- Diagnostics: int truthiness + `int + (int | None)`.
- Root cause: Python truthiness plus unchecked indexed stack access in arithmetic.
- Required fix lane: `sifr_adaptation`.
- Closure recipe:
  - Replace `if a` with `if a != 0`.
  - Narrow `stack[-1]` to non-optional local before `diff = a + top`.

9. `0973_k_closest_points_to_origin`
- Diagnostics: list-vs-tuple assertion mismatch, for-loop tuple destructuring over `list[int]`, optional heappop unpack, heap comparable constraint, undefined variable cascade.
- Root cause: mixed shape contracts and Python destructuring assumptions that do not satisfy Sifr contracts.
- Required fix lane: `sifr_adaptation`.
- Closure recipe:
  - Keep this fixture in-scope (not deferred) because it is a direct blocker for the 14-fixture zero-failure goal of this phase.
  - Replace heap-based implementation with deterministic selection scan to eliminate both `heappop` optional unpack and tuple comparability constraints.
  - Normalize representation to `list[list[int]]` end-to-end (function return and assertions).
  - Replace tuple destructuring with explicit index extraction + Optional guards on indexed values.

10. `1220_count_vowels_permutation`
- Diagnostics: `Never` comparisons, missing parameter annotations, non-exhaustive return path.
- Root cause: untyped recursive surface with default parameter driving unstable inference.
- Required fix lane: `sifr_adaptation`.
- Closure recipe:
  - `def countVowelPermutation(n: int, c: str = '') -> int`.
  - Ensure all control paths return `int` (explicit terminal fallback).
  - Keep memo values typed as int along all branches.

11. `1514_path_with_maximum_probability`
- Diagnostics: float-int compare, optional heappop unpack, list destructuring mismatch, heap comparable mismatch, undefined variable cascade.
- Root cause: mixed numeric and container contracts + unpack assumptions without Optional narrowing.
- Category note: this fixture remains in `operator_and_truthiness_typing_gap` because the first/trigger diagnostic is numeric operator typing (`float` vs `int` compare), even though major secondary diagnostics are structural and are closed in the same rewrite.
- Required fix lane: `sifr_adaptation`.
- Closure recipe:
  - Keep this fixture in-scope (not deferred) because phase objective is strict closure of all 14 scoped fixtures.
  - Replace heap-based Dijkstra variant with edge-relaxation DP (Bellman-Ford style) to avoid heap tuple comparability and `heappop` Optional unpack surfaces.
  - Use consistent float literals (`1.0`, `0.0`) and float-only probability arithmetic.
  - Replace edge tuple destructuring with explicit indexed extraction + Optional guards.
  - Preserve output contract as `float` on all paths (`return 0.0` fallback).

### `callable_argument_contract_mismatch` (1)

12. `0931_minimum_falling_path_sum`
- First diagnostic: callable arg mismatch (`Path` arg expected `int`, got `float`).
- Additional diagnostics: Optional index arithmetic and return mismatch.
- Root cause: nested helper contract under-specified; inferred types drift; indexed matrix access used without Optional narrowing.
- Required fix lane: `sifr_adaptation`.
- Closure recipe:
  - Type helper contract explicitly: `def Path(i: int, k: int, n: int) -> int`.
  - Keep memo dictionary typed to int values.
  - Narrow indexed `matrix[...]` values to non-optional locals before arithmetic.

### `return_path_and_function_contract_gap` (2)

13. `0162_find_peak_element`
- First diagnostic: `undefined variable: 'mid'`.
- Root cause: loop-scoped symbol returned without guaranteed assignment on all paths.
- Required fix lane: `sifr_adaptation`.
- Closure recipe: initialize `mid` before loop and keep explicit total return contract.

14. `0516_longest_palindromic_subsequence`
- Diagnostics: duplicate definition + optional arithmetic/max contract failures.
- Root cause: multiple competing implementations in one module and a DP variant that propagates optional indexed values through `max`/`+`.
- Required fix lane: `sifr_adaptation`.
- Closure recipe:
  - Keep exactly one canonical implementation.
  - Prefer LCS variant (already present) as survivor because it keeps recurrence on deterministic integer cells.
  - Remove duplicate/dead implementations that emit Optional arithmetic cascades.

## Compiler vs Adaptation Judgment

- Semantic compiler/language change required: `none`.
- Fixture adaptation required: `14/14`.
- Optional compiler follow-up (separate phase, non-blocking): better diagnostic grouping/prioritization only.

## Scope Decision: 0973 and 1514

- Reviewer pass1 offered deferral of `0973` and `1514` as an optional path.
- Decision for this phase: keep both in scope to maintain the explicit close target of all 14 scoped fixtures.
- Risk control: avoid high-friction heap-comparability surfaces by rewriting both fixtures to non-heap formulations that stay within current Sifr contracts.

## Root-Cause Clusters (Across 14 Fixtures)

1. Numeric truthiness/operator assumptions: `6` fixtures.
2. Mixed numeric/container contract drift: `4` fixtures.
3. Untyped nested helper + inference drift: `3` fixtures.
4. Definite assignment/return totality contract issues: `2` fixtures.
5. Duplicate/multi-implementation fixture hygiene: `2` fixtures.

Note: counts overlap by fixture because several fixtures have multi-causal failures.
Specifically, `0473` and `1514` are bucketed under operator/truthiness by trigger diagnostic, but are implemented as cross-surface closures that also eliminate structural residuals.

## Ready-to-Implement Workstreams

Phase ID: `ad_hoc_operator_truthiness_contract_closure_20260407`

### WS1: Simple Operator/Truthiness Canonicalization

Fixtures:
- `0007`, `0068`, `0416`, `0846`

Actions:
- Replace numeric truthiness conditions with explicit bool predicates.

Success gate:
- Each fixture reaches `PASS` under targeted run.

### WS2: Contract-Safe Helper Typing and Return Closure

Fixtures:
- `0201`, `0371`, `1220`, `0931`, `0162`

Actions:
- Add explicit helper signatures and return-totality contracts.
- Remove duplicate in-file definitions where present.
- Add required Optional narrowing at indexed arithmetic points.

Success gate:
- No callable/return/duplicate diagnostics remain in these fixtures.
- Each fixture reaches `PASS` under targeted run.

### WS3: Multi-Surface Structural Rewrite (Heap/Destructure/Optional)

Fixtures:
- `0473`, `0735`, `0973`, `1514`, `0516`

Actions:
- Replace unsupported Python destructuring patterns with explicit index extraction.
- Guard Optional values before tuple unpack/arithmetic.
- Normalize container and assertion shapes.
- Keep heap element contracts type-consistent and comparable.
- Select one canonical implementation in multi-solution fixtures.

Success gate:
- No residual diagnostics in these fixtures after rewrite.
- Each fixture reaches `PASS` under targeted run.

### WS4: Validation and Exit

Validation sequence:

1. Run targeted checks/runs for all 14 scoped fixtures after each workstream.
2. Any fixture still failing must be fixed in-phase (no bucket-shift acceptance).
3. Run full corpus and regenerate results + taxonomy artifacts.
4. Diff category counts against baseline artifacts from 2026-04-07 rerun1.
5. Use baseline diagnostics evidence file:
   - `verification/leetcode/ad_hoc_operator_truthiness_contract_closure_20260407_baseline_checks.txt`
   - generated by targeted `check` across all 14 fixtures before implementation.

Command anchors:

- `target/release/sifr check audits/leetcode/<fixture>.sifr`
- `target/release/sifr run audits/leetcode/<fixture>.sifr`
- full-corpus rerun command used by current verification harness.

## Phase Exit Criteria (Strict)

- All 14 scoped fixtures reach `PASS` (`check` + `run`).
- Net `failing_cases` reduced by at least `14` from baseline rerun1.
- No regressions in:
  - `codegen_runtime_build_gap`
  - `optional_none_flow_and_narrowing_gap`
  - `destructuring_and_assignment_target_surface_gap`
  - `python_stdlib_and_builtin_parity_gap`
- Updated results and taxonomy artifacts are committed with this phase close note.

## Execution Checklist

- [x] WS1 implemented and validated
- [x] WS2 implemented and validated
- [x] WS3 implemented and validated
- [x] Scoped 14-fixture PASS confirmation (`check` + `run`)
- [x] Full-corpus rerun completed
- [x] Taxonomy + delta report refreshed
- [x] Phase close note recorded in `issues/`
