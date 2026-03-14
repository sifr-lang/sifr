# Phase 31 Strategy Synthesis Review

Status: completed on 2026-03-14

Inputs reviewed:
- `issues/phase31-ad-hoc-followup-milestones.md`
- `issues/ad-hoc-full-recursive-type-feature.md`
- `issues/ad-hoc-own-mut-parameter-convention.md`
- `verification/leetcode/phase31_current_full_results_after_m31a_wave5_rerun.json`
- `reviews/phase31-strategy-review-pass-1.md`

## Question

For the 35 still-failing Phase 31 LeetCode cases, decide case by case whether we should:

1. add or rely on a broader ad hoc language/compiler phase,
2. fix the LeetCode fixture into canonical Sifr form,
3. do both,
4. or keep the work as ordinary Phase 31 compiler/runtime closure.

## Method

- Enumerated all 35 failing cases from `phase31_current_full_results_after_m31a_wave5_rerun.json`.
- Validated the current milestone plan against the actual failure messages rather than milestone names alone.
- Read representative failing fixtures directly, especially the ambiguous ones (`0043`, `0215`, `1046`, `0502`, `0743`, `1299`).
- Ran local experiments with temporary `.sifr` files to distinguish:
  - raw-source policy mismatches from real compiler gaps,
  - multi-solution fixture noise from true language/runtime failures,
  - and prerequisite language-surface work from normal closure work.
- Spawned an external reviewer and validated their notes against the local evidence.

## Conclusion

We do **not** need any new broad ad hoc language-feature phases beyond the two already created:

- `issues/ad-hoc-full-recursive-type-feature.md`
- `issues/ad-hoc-own-mut-parameter-convention.md`

What we need is a mixed strategy:

- use those two ad hoc phases as prerequisites where the failing LeetCode cases truly depend on broader language features,
- keep most remaining cases as ordinary Phase 31 compiler/runtime closure work,
- and use canonical Sifr fixture rewrites only where the scraped Python source conflicts with intentional Sifr contracts or where the scraped file is not a canonical single-solution corpus fixture.

In short:

- **Broader prerequisite phases needed:** yes, but only the two already created
- **Canonical Sifr fixture rewrites needed:** yes, in a small number of cases
- **Both needed:** yes, for some cases

## High-Level Decision

### 1. Existing broad ad hoc phases are sufficient

Only two failing surfaces justify broader feature phases:

- recursive types / recursive-node attribute surface
- `own mut` parameter convention

No third cross-cutting ad hoc phase is justified by the remaining failures.

### 2. Most remaining failures are ordinary compiler/runtime completion work

These are still the right kind of work to keep inside the Phase 31 carry-forward milestones:

- container literal specialization
- optional-flow proof completion
- destructuring/composite lvalues
- nested function pipeline
- local name binding/shadowing

### 3. Some cases require both fixture adaptation and compiler work

This is the main place where the earlier plan needed refinement:

- `0043` needs a canonical Sifr rewrite because raw `int(str)` violates Sifr parse-safety policy, **and** the canonical rewrite still exposes a real optional-index arithmetic gap.
- `0215` and `1046` need fixture canonicalization because the scraped files contain multiple top-level solutions, **and** the canonicalized versions still expose real compiler gaps.

## Case-by-Case Classification

Legend:
- `prereq + closure`: broader ad hoc phase first, then final Phase 31 LeetCode closure
- `normal closure`: ordinary Phase 31 compiler/runtime work
- `canonical + closure`: rewrite to canonical Sifr or canonical single-solution fixture, then finish remaining compiler/runtime closure

| ID | Current failure shape | Classification | Primary owner after review |
| --- | --- | --- | --- |
| `0001` | `dict[Any, Any]` indexing | normal closure | `m31_g` |
| `0015` | local name resolves as function instead of int | normal closure | `m31_h` |
| `0017` | nested helper leaves `Any` in dict/string indexing | normal closure | `m31_d` |
| `0039` | nested helper leaves `Any` in comparisons/indexing | normal closure | `m31_d` |
| `0043` | raw parse-safety mismatch + remaining optional arithmetic proof gap | canonical + closure | `m31_k` then `m31_a` |
| `0050` | nested helper missing annotation/inference | normal closure | `m31_d` |
| `0052` | nested helper / `nonlocal` shape | normal closure | `m31_d` |
| `0053` | `nums[0]` remains `int | None` | normal closure | `m31_a` |
| `0078` | nested helper leaves `Any` in control flow | normal closure | `m31_d` |
| `0090` | nested helper leaves `Any` in indexing | normal closure | `m31_d` |
| `0100` | recursive node attribute reads | prereq + closure | `prereq_recursive_types` then `m31_e` |
| `0102` | recursive node attribute reads | prereq + closure | `prereq_recursive_types` then `m31_e` |
| `0110` | recursive node attribute reads | prereq + closure | `prereq_recursive_types` then `m31_e` |
| `0127` | `popleft()` / slice after non-empty proof | normal closure | `m31_a` |
| `0207` | nested helper inference, with possible destructuring follow-on | normal closure | `m31_d` first |
| `0215` | multi-solution scraped file; canonical sorting-only form still hits optional index proof gap | canonical + closure | `m31_i` then `m31_a` |
| `0226` | recursive node attribute reads | prereq + closure | `prereq_recursive_types` then `m31_e` |
| `0235` | recursive node attribute reads | prereq + closure | `prereq_recursive_types` then `m31_e` |
| `0238` | list element remains `int | None` in arithmetic | normal closure | `m31_a` |
| `0242` | empty dict specialization gap | normal closure | `m31_g` |
| `0295` | tuple/attribute destructuring | normal closure | `m31_b` |
| `0322` | guarded recurrence index still `int | None` | normal closure | `m31_a` |
| `0424` | empty dict specialization gap | normal closure | `m31_g` |
| `0502` | heap pop result remains `None | tuple[...]` | normal closure | `m31_a` |
| `0523` | empty dict specialization gap | normal closure | `m31_g` |
| `0560` | empty dict specialization gap | normal closure | `m31_g` |
| `0684` | nested helper inference leaves `Any` in comparisons | normal closure | `m31_d` |
| `0703` | tuple/attribute destructuring | normal closure | `m31_b` |
| `0743` | heap pop result remains `None | tuple[...]` | normal closure | `m31_a` |
| `0746` | fixed index remains `int | None` | normal closure | `m31_a` |
| `0912` | nested helper inference missing | normal closure | `m31_d` |
| `0997` | loop tuple target from `list[int]` | normal closure | `m31_b` |
| `1046` | multi-solution scraped file; canonical typed form still hits optional heap/index arithmetic | canonical + closure | `m31_i` then `m31_a` |
| `1209` | composite subscript mutation | normal closure | `m31_b` |
| `1299` | borrowed parameter returned by value | prereq + closure | `prereq_own_mut` then `m31_j` |

## Key Experiments

### `0043` is both a raw-source divergence and a real compiler gap

Evidence:

- Raw fixture uses `int(num1[i1]) * int(num2[i2])`, which conflicts with intentional Sifr parse-safety (`Result[int, ParseError]`).
- Raw fixture also uses `map(str, ...)`, and `str` is not currently available as a first-class callable.

Experiment:

- A temporary canonicalized probe that replaced `int(ch)` with a safe local digit helper still failed with:
  - `unsupported operand type(s) for %: 'int | None' and 'int'`
  - `unsupported operand type(s) for //: 'int | None' and 'int'`

Conclusion:

- `0043` is not just a raw-source rewrite issue.
- It needs `m31_k` for canonical Sifr normalization **and then** `m31_a` for the remaining optional proof gap.

### `0215` and `1046` do need fixture canonicalization

The external reviewer was directionally right that canonicalization alone does not close them, but wrong to say they are not fixture-shape issues.

Evidence from source:

- `0215` contains three top-level `findKthLargest` implementations.
- `1046` contains two top-level `lastStoneWeight` implementations, one typed and one untyped.

Experiments:

- Canonical `0215` sorting-only probe still failed with:
  - `return type mismatch: expected 'int', got 'int | None'`
- Canonical typed `1046` probe still failed with:
  - `abs() argument must be numeric, got 'int | None'`
  - `unsupported operand type(s) for -: 'None | int' and 'None | int'`

Conclusion:

- `m31_i` is still needed to canonicalize these scraped fixtures.
- But `m31_i` is not the final owner of passing them.
- After canonicalization, both cases fall into ordinary compiler closure, primarily `m31_a`.

## External Reviewer Validation

External review file:
- `reviews/phase31-strategy-review-pass-1.md`

What the external review got right:

- the overall milestone structure is mostly sound
- the two explicit prerequisites are the right broad ad hoc phases
- `0043` is correctly recognized as a canonical-Sifr case
- `m31_g`, `m31_a`, `m31_b`, `m31_d`, `m31_h`, `m31_e`, and `m31_j` are broadly the right buckets

What the external review got wrong or only partially right:

- It said `0215` and `1046` are not multi-solution fixture issues. That is false; the fixture source files do contain multiple top-level solutions.
- It recommended removing `0215` and `1046` from `m31_i`. That would lose the required corpus canonicalization step.
- It treated `0043` as only a canonical rewrite issue, but the local experiments show the canonicalized form still needs `m31_a` closure.

Best judgment:

- keep `m31_i`, but make its role explicit: canonicalize first, then reclassify remaining failures into normal compiler milestones
- keep `m31_k`, but make its role explicit: canonicalize `0043` first, then finish remaining compiler proof work under `m31_a`

## Final Recommendation

### Keep as explicit broad ad hoc prerequisites

- `issues/ad-hoc-full-recursive-type-feature.md`
- `issues/ad-hoc-own-mut-parameter-convention.md`

### Do not add a new broad ad hoc phase

No additional failing bucket currently justifies a third language-feature phase. The rest are either:

- ordinary compiler/runtime closure work, or
- corpus/canonicalization work plus ordinary compiler/runtime closure.

### Keep and tighten the Phase 31 milestones

- `m31_g`, `m31_a`, `m31_b`, `m31_d`, `m31_h`: keep as ordinary closure milestones
- `m31_e`: keep as recursive-type prerequisite follow-on closure
- `m31_j`: keep as `own mut` prerequisite follow-on closure
- `m31_k`: keep, but explicitly mark `0043` as `canonical rewrite first, then m31_a`
- `m31_i`: keep, but explicitly mark `0215` and `1046` as `canonicalize first, then reclassify into normal compiler milestones`

## Bottom Line

The right strategy is **both**, but only in a narrow subset of cases:

- broader ad hoc phase + closure: tree cases and `1299`
- canonical fixture rewrite + closure: `0043`, `0215`, `1046`
- normal compiler/runtime closure only: everything else

That is the cleanest synthesis of the actual failing corpus.
