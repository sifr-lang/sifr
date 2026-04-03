# Review: Ad Hoc Signature-Invalid Fixture Adaptation Plan

Reviewer: Claude (pass 1)
Date: 2026-03-31
Source: `issues/ad-hoc-signature-invalid-fixture-adaptation-plan-2026-03-31.md`

---

## Finding 1 — Batch classification errors: class-boundary fixtures misplaced in batch_b

**Severity: Medium — will cause confusion during execution**

Batch_b is defined as "nested and local helpers" but contains fixtures whose primary missing signatures are on **class methods and constructors**, not nested/local helpers:

- `0706_design_hashmap`: `ListNode.__init__(self, key=-1, val=-1, next=None)` and `MyHashMap.hashcode(self, key)` are **class-boundary** signatures. The nested helper policy ("do not rely on contextual or recursive signature inference") does not apply — these are class methods with untyped parameters. This fixture belongs in batch_a under the "class methods/constructors remain explicit boundaries" policy.

- `0721_accounts_merge`: `UnionFind.__init__(self, n)`, `UnionFind.find(self, x)`, and `UnionFind.union(self, x1, x2)` are all untyped class methods. The fixture also has `defaultdict` and `{}` usage that may expose container specialization issues (root cause 3), but the primary typing boundary violations are class-level, not nested-helper-level.

- `0098_validate_binary_search_tree`: This one is correctly placed — `valid(node, left, right)` is a genuine nested helper. However, the `left` and `right` params are initialized with `float("-inf")` and `float("inf")`, which creates a non-trivial type annotation question (see Finding 4).

**Correction:** Move `0706_design_hashmap` and `0721_accounts_merge` from batch_b to batch_a. Update the counts from 21/15 to 23/13.

---

## Finding 2 — No per-fixture annotation specification

**Severity: High — blocks unambiguous execution**

The plan lists 36 fixture slugs but does not specify **what types to add** to each fixture. For trivial cases (`nums: list[int]`, `target: int`) the answer is obvious, but several fixtures require non-trivial type decisions:

- `0098_validate_binary_search_tree`: `valid(node, left, right)` — what type is `left`/`right`? They receive `float("-inf")` and `float("inf")`. Is `float` a valid Sifr type here? Should these be `int | float`? Or should the fixture be rewritten to use sentinel int values instead of float infinity?

- `0332_reconstruct_itinerary`: `dfs(adj, src)` — `adj` is `dict[str, list[str]]`, but should the annotation reference the outer variable or take a parameter type? The helper mutates the outer `adj`, so copying it as a parameter type may misrepresent intent.

- `0721_accounts_merge`: `UnionFind` methods need types, but `emailToAcc = {}` and `emailGroup = defaultdict(list)` may also need explicit container annotations to avoid secondary container-specialization failures.

- `0018_4sum`: `findNsum(l, r, target, N, result, results)` has 6 parameters — `result: list[int]`, `results: list[list[int]]`, `N: int`, etc. The return type is `None` (mutates in place). This is not immediately obvious without reading the code.

**Correction:** Either (a) add a per-fixture annotation table specifying the types to add, or (b) explicitly state that the executor determines types from call-site evidence and document the decision in a per-fixture commit message. Option (b) is acceptable but should be stated as policy.

---

## Finding 3 — Missing policy for secondary failures exposed by adaptation

**Severity: Medium — creates ambiguity at execution boundary**

Batch_b policy states: "if helper signatures expose a second real defect, record that and leave it for post-rerun classification." Batch_a has **no equivalent policy**. This is a gap because:

- `0253_meeting_rooms` (batch_a): after typing `intervals: list[list[int]]`, the list comprehension `[i[0] for i in intervals]` may still trigger container/iteration issues.
- `0721_accounts_merge` (batch_b, should be batch_a): after typing the `UnionFind` class, the `defaultdict(list)` and `emailToAcc = {}` patterns may still fail on container specialization.

**Correction:** Add the same secondary-failure-recording policy to batch_a explicitly. A single sentence will do: "If adding types to a batch_a fixture exposes a secondary container or inference defect, record the residual error and leave it for post-rerun reclassification."

---

## Finding 4 — Hidden assumption: Sifr's `float` type behavior with infinity

**Severity: Low-Medium — affects at least one fixture**

`0098_validate_binary_search_tree` calls `valid(root, float("-inf"), float("inf"))`. The plan assumes adding explicit types to nested helpers is straightforward, but this fixture requires knowing:

1. Does Sifr support `float("-inf")` and `float("inf")` as valid `float` values?
2. Can `float` be compared with `int` (since `node.val` is `int`)?
3. If not, should the fixture be rewritten to use `int` sentinel values (e.g., `-2**31`, `2**31`)?

This is a type-design question, not just an annotation task. The plan should either answer it or flag it as a known decision point.

---

## Finding 5 — Acceptance criteria do not define "adapted away" vs "residual"

**Severity: Medium — validation is underdefined**

Acceptance criterion 2 says "missing-signature failures are removed from those fixtures." But criterion 4 says "residual failures from these fixtures are reclassified." These two criteria do not define:

- What **counts** as a signature failure being "removed"? Is it sufficient that the specific `missing a type annotation` error disappears, even if a new `Any`-related error appears from a different root cause?
- Where are residual failures **recorded**? A new file? An update to the root cause analysis? An inline annotation in the results JSON?
- What is the reclassification **format**? The root cause analysis defines three lanes (A/B/C). Should residuals be tagged by lane?

**Correction:** Add:
- "Removed" means the fixture no longer produces any `missing a type annotation` or `parameter ... is missing a type annotation` diagnostic.
- Residual failures are recorded in a new file `issues/signature-adaptation-residuals-YYYYMMDD.md` with per-fixture error and lane classification.

---

## Finding 6 — `0253_meeting_rooms` naming ambiguity

**Severity: Low — but could cause a wrong-fixture edit**

The corpus contains both `0253_meeting_rooms` and `0253_meeting_rooms_ii`. The plan lists `0253_meeting_rooms` in batch_a. The root cause analysis mentions `0253_meeting_rooms_ii` under root cause 3 (container specialization, the 22 compiler-owned residuals). These are distinct fixtures, but the plan's batch_a entry is just `0253_meeting_rooms` — an executor unfamiliar with the corpus might confuse them.

**Correction:** Use the full slug `0253_meeting_rooms` (not `_ii`) and add a parenthetical note: "(not `0253_meeting_rooms_ii`, which is a container specialization residual)."

---

## Finding 7 — Validation does not specify pass criteria for `check` vs `run`

**Severity: Low**

Validation step 1 says "targeted `check`/`run` on each changed fixture." The current results show all 36 fixtures fail at the `check` stage. After adaptation:

- Must `check` pass? (Expected yes.)
- Must `run` also pass? (Not necessarily — some may have runtime issues unrelated to typing.)
- What if `check` passes but `run` fails? Is that a pass or a residual?

**Correction:** State explicitly: "`check` must pass without signature-related diagnostics. `run` pass is desirable but not blocking; `run` failures are recorded as residuals for post-rerun classification."

---

## Finding 8 — No ordering or dependency management within batch_a

**Severity: Low**

Batch_a has 21 fixtures (should be 23 per Finding 1). The plan says "adapt batch_a, then batch_b" but does not specify whether batch_a fixtures can be adapted in parallel or must be sequential. Since fixtures are independent files with no cross-dependencies, this is likely fine — but stating "fixtures within a batch are independent and can be adapted in any order or in parallel" removes ambiguity.

---

## Finding 9 — `main()` functions are untyped in every fixture

**Severity: Low — needs explicit ruling**

Every fixture has a `def main():` with no return type annotation. The plan says "every function in the 36 fixtures has explicit input and output types." Taken literally, this includes `main()`. Should `main` get `-> None`? This is a trivial point but an executor following the acceptance criteria literally would need to annotate `main()` in all 36 fixtures.

**Correction:** Either exclude `main()` explicitly ("every function except the test harness `main`") or include it as a low-effort addition.

---

## Summary of Required Corrections

| # | Finding | Severity | Action |
|---|---------|----------|--------|
| 1 | Batch misclassification | Medium | Move `0706`, `0721` from batch_b to batch_a |
| 2 | No per-fixture type spec | High | Add annotation policy or per-fixture table |
| 3 | Missing secondary-failure policy in batch_a | Medium | Add one-line policy |
| 4 | `float` infinity type assumption | Low-Med | Flag as decision point |
| 5 | Undefined "adapted away" criteria | Medium | Define removal criteria and residual format |
| 6 | `0253` naming ambiguity | Low | Clarify slug |
| 7 | `check` vs `run` pass criteria | Low | Define explicitly |
| 8 | No intra-batch ordering statement | Low | One sentence |
| 9 | `main()` annotation scope | Low | Exclude or include explicitly |

---

## Verdict: Mostly Ready

The plan's strategic direction, lane separation, and fixture lists are sound. The 36-fixture scope is correctly sourced from the root cause analysis and verified against the live results. The execution order and non-goals are clear.

However, **Finding 2** (no per-fixture annotation specification) is the most significant gap. An executor can likely resolve types from call-site evidence for most fixtures, but a handful of cases (`0098` with float infinity, `0721` with `defaultdict`, `0332` with outer-scope mutation) require type-design decisions that are not addressed. This makes the plan **not fully executable without interpretation**.

Findings 1, 3, and 5 are moderate corrections that should be addressed before execution to avoid classification confusion and undefined completion criteria.

The remaining findings (4, 6–9) are minor clarifications that can be resolved inline during execution.

**Recommendation:** Address findings 1–3 and 5, then the plan is ready for execution.
