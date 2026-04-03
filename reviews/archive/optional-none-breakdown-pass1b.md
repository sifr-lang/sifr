# Optional/None Failure Family — Root-Cause Breakdown Pass 1b

**Date:** 2026-03-29
**Corpus run:** PASS=97 / CHECK_ERROR=290 / RUN_ERROR=24
**Category under analysis:** Optional/None flow and narrowing gap (62 fixtures, 79 diagnostic instances)
**Relation to prior work:** This is a second-pass deepening of `optional-none-breakdown-pass1.md`. It incorporates direct reading of `crates/sifr_type_system/src/narrow.rs`, representative fixture source, and architecture.md. Where pass1 described *what* the subproblems are, this pass describes *why the compiler cannot currently fix them* and *what would actually need to change*.

---

## 0. Grounding: What the Narrowing Engine Actually Does

Reading `narrow.rs` directly reveals the structural constraint that underlies all of these failures:

```
NarrowingCondition::Truthiness(String)
NarrowingCondition::IsNone(String)
NarrowingCondition::IsNotNone(String)
NarrowingCondition::IsInstance(String, Type)
NarrowingCondition::Equality(String, LiteralValue)
NarrowingCondition::TypePredicate(String, Type)
NarrowingCondition::Not(Box<NarrowingCondition>)
NarrowingCondition::And(Vec<NarrowingCondition>)
NarrowingCondition::Or(Vec<NarrowingCondition>)
```

**Every leaf narrowing condition takes a `String` — a variable name.** The narrowing engine is fundamentally variable-centric. It can narrow `x` in `if x is not None:`, but it has no concept of narrowing the *result of an expression* like `s[i]`, `dp[i][j]`, or `roman[c]`. There is no `NarrowingCondition::IndexInBounds(expr, index_expr)` or `NarrowingCondition::DictKeyPresent(dict_name, key_expr)`.

This is not an implementation gap that can be closed by adding cases to `narrow_type`. It requires a new kind of condition that tracks *expression-level* safety proofs, not just variable-level type refinements. This is the structural root cause of the entire failure family.

Two additional observations from `narrow.rs`:

1. **`And` false-branch is an approximation**: `NarrowingCondition::And` in the false branch returns `ty.clone()` with a comment "This is an approximation; for now, return the original type." This means `if guard and use_of_guarded_expr:` does NOT propagate the guard's narrowing to the expression — even when the guard is `i < len(s)` and the expression is `s[i]`. The And-chain narrowing is incomplete.

2. **`can_be_falsy` includes `Type::Int`**: This is correct — `0` is falsy. But it means `if not x:` where `x: int | None` narrows the false-branch to `int | None` (removing only non-falsy types), not to `int`. For struct types like `TreeNode` or `ListNode` (not in `can_be_falsy`), truthiness narrowing works correctly: `if not node:` removes `None` cleanly because `TreeNode` cannot be falsy.

---

## 1. Real Compiler Subproblems

### SC-1 · The narrowing engine is variable-centric; subscript results have no narrowable identity

**Scope:** affects ~50 of the 79 diagnostic instances across all clusters except SP-3/SP-5 from pass1.

**Mechanism:**
- Architecture (line 144): `x[i]` → `Option[T]`. Architecture (line 145): `d[k]` → `Option[V]`.
- The checker propagates this `T | None` into the downstream expression.
- There is no way to narrow the result of `x[i]` to `T` via the current condition vocabulary.
- Even if the programmer writes `if i < len(x): ... x[i]`, the checker cannot attach the proof from the guard to the subscript expression, because a subscript is not a named variable — it's a transient expression result.

**What would need to change:**
A new condition category: `NarrowingCondition::BoundsProven(collection_var, index_expr)` — analogous to IsNotNone but for subscript safety. This requires the HIR checker to emit this condition whenever it sees `i < len(x)` or `i in range(len(x))` as a guard preceding `x[i]`. This is non-trivial because it requires range-tracking of index variables, not just type tracking.

**Alternative that does NOT require this:** Require explicit unwrapping at every subscript use site. This is the Sifr-correct position when the compiler cannot prove safety. The fixtures need to be canonicalized to reflect it.

---

### SC-2 · `And`-chain narrowing is incomplete in the false-guard branch

**Scope:** 2 `optional_membership_operand` + subset of `other_optional_flow`.

**Mechanism:**
The pattern `if i + 1 < len(s) and s[i + 1] in "0123456":` should narrow `s[i + 1]` to `str` on the RHS. It does not, because:
1. The And condition's false-branch narrowing is a stub (returns `ty.clone()`).
2. Even if the And-narrowing were complete, `s[i + 1]` is a subscript expression, not a named variable — so SC-1 applies.

**What would need to change:**
Complete the And false-branch narrowing AND add expression-level narrowing (see SC-1). SC-2 is a secondary blocker; SC-1 is primary.

---

### SC-3 · Memoization dict type is seeded with `T | None` from read-back of its own values

**Scope:** `return_type_optional_leak` (14), `optional_contaminates_container_inference` (1).

**Mechanism:**
Consider `0097_interleaving_string`:
```python
dp = [[False] * (len(s2) + 1) for i in range(len(s1) + 1)]
dp[len(s1)][len(s2)] = True
# ...
if dp[i + 1][j]:   # dp[i+1]: list[bool] | None; then dp[i+1][j]: bool | None
    dp[i][j] = True
```
`dp[i + 1]` returns `list[bool] | None`. Then `dp[i + 1][j]` reads into that Optional list, returning `bool | None`. The written value `True` is `bool`, but `dp[i + 1][j] or dp[i][j]` produces `bool | None | bool` which unifies to `bool | None`. This `bool | None` is what gets stored back.

The effect: the element type of `dp` converges to `list[bool | None]` (or `list[list[bool | None]]`), and `return dp[0][0]` is then `bool | None`.

The function declares `-> bool`. Type error.

**Distinct from SC-1:** SC-3 is specifically about the *re-storage* path contaminating the container's inferred element type, which then poisons the return expression. SC-1 is about immediate use at a single site.

**What would need to change:**
Either (a) subscript narrowing (SC-1 fix) prevents the contamination, or (b) the checker should emit a diagnostic pointing to the source of contamination (the first read-back assignment) rather than the return site. Currently the error appears at `return dp[0][0]` with no hint that the problem originated at `dp[i+1][j]`.

For the fixture corpus, the fix is: never read from the DP table and write the result back to the same table without narrowing. Preferred idiom: iterate, compute, assign with guards. Or build the DP table bottom-up using only previously-written (concrete) values with explicit if-chains rather than reading back.

---

### SC-4 · Recursive nested function: inferred return type is `Unknown | None` or `T | None`

**Scope:** 0010 top-down memoization, portions of `other_optional_flow`.

**Mechanism:**
`0010_regular_expression_matching` (top-down variant):
```python
cache = {}

def dfs(i, j):
    if (i, j) in cache:
        return cache[(i, j)]  # cache[(i,j)]: Unknown | None
    if i >= len(s) and j >= len(p):
        return True
    if j >= len(p):
        return False
    ...
    cache[(i, j)] = dfs(i, j + 2) or (match and dfs(i + 1, j))
    return cache[(i, j)]  # same: Unknown | None
```

`cache = {}` is inferred as `dict[Unknown, Unknown]` (empty literal with no type annotation). `cache[(i, j)]` returns `Unknown | None`. The recursive call `dfs(...)` is inferred at first pass as `Unknown` (the inference hasn't converged). The Or expression `dfs(i, j+2) or ...` has type `Unknown | None`. This is stored back, and `return cache[(i,j)]` is `Unknown | None`, conflicting with the outer function's declared `-> bool`.

**Two distinct mechanisms inside SC-4:**
- **SC-4a: Empty dict literal inferred as `dict[Unknown, Unknown]`** — subscript returns `Unknown | None` instead of `bool | None`. Even after the checker sees assignments like `cache[(i,j)] = True/False`, it may not retroactively re-infer the dict's value type.
- **SC-4b: Recursive closure return type converges to Optional** — even if the dict's value type were known to be `bool`, reading back via safe indexing gives `bool | None`, so the inferred return of `dfs` becomes `bool | None`. The outer function then sees `dfs(0, 0): bool | None` as the return expression.

**What would need to change:**
For SC-4a: `cache = {}` followed by `cache[k] = bool` should retroactively narrow the dict type to `dict[tuple[int,int], bool]` through assignment flow. This is a backward-flowing type constraint.

For SC-4b: The nested function's return type should be inferred from its explicit `return True` / `return False` / `return bool_expr` paths — not from read-back of the cache dict. If the inference engine uses explicit `return` statements as anchors for nested function return types, this converges correctly.

---

### SC-5 · Function parameter declared `T`; call site passes `T | None` from struct field

**Scope:** `argument_position_requires_concrete` (16 instances).

**Mechanism:**
`0104_maximum_depth_of_binary_tree`:
```python
def maxDepth(root: TreeNode) -> int:
    if not root:
        return 0
    return 1 + max(maxDepth(root.left), maxDepth(root.right))
```
`root.left: TreeNode | None`. Call `maxDepth(root.left)` expects `TreeNode`. Type error.

The function actually handles `None` internally (the `if not root:` base case). The Python convention is to use the concrete type annotation and let `None` through implicitly; Sifr requires the declared type to match the actual type space.

**Sub-variants:**
- **SC-5a: Function signature is wrong — should be `T | None`**, and truthiness narrowing (`if not root: return 0`) already narrows correctly in the function body. This is the common case.
- **SC-5b: Fixture call site passes literal `None` to a concrete-typed function.** `0024_swap_nodes_in_pairs` calls `swapPairs(None)` in `main()`, where `swapPairs(head: ListNode)` declares a concrete parameter. The only fix is the signature.

**Compiler gap vs. fixture gap:**
The truthiness narrowing (`if not x:` where `x: TreeNode | None`) should already work — `TreeNode` is not in `can_be_falsy`, so `narrow_truthiness(TreeNode | None)` correctly removes `None`. Verify this path in the checker. If truthy-narrowing works, the fix is purely mechanical fixture updates: `def maxDepth(root: TreeNode)` → `def maxDepth(root: TreeNode | None)`.

**If truthy-narrowing is broken here,** the specific failure mode to look for is: after `if not root: return 0`, is `root` narrowed to `TreeNode` in the subsequent `return` branch? If not, the `maxDepth(root.left)` call isn't even the first error — the field accesses `root.left` and `root.right` would also fail.

---

### SC-6 · Builder helper returns `T | None`; caller places result in `list[T]`

**Scope:** 2–3 fixtures (0023, possibly others).

**Mechanism:**
`0023_merge_k_sorted_lists`:
```python
def buildListNode(values: list[int]) -> ListNode | None:
    ...

lists = [buildListNode([1, 4, 5]), buildListNode([1, 3, 4]), buildListNode([2, 6])]
# inferred: list[ListNode | None]

mergeKLists(lists)  # expects list[ListNode]
```

`list` is invariant. `list[ListNode | None]` is not a subtype of `list[ListNode]`. This is a correct rejection by the checker.

**This is a fixture design issue.** The builder is semantically correct to return Optional (an empty input list could yield `None`). The test harness uses it with guaranteed non-empty inputs, but the checker cannot verify that.

**Fix options (fixture-side only):**
1. Change `buildListNode` to return `ListNode` when called with non-empty input — requires overloading or a separate builder.
2. Add explicit narrowing in `main()` after building: `if head := buildListNode(...): ...`.
3. Separate the builder from the harness — use a direct constructor in `main()` rather than going through the Optional-returning builder.

---

## 2. Assignment to Fix Layers

| SC | Diagnostic cluster | Fix layer | Compiler change needed | Fixture change needed |
|---|---|---|---|---|
| SC-1 | ~30 across arithmetic, index, iteration, membership, other | Fixture (primarily) | SC-1 requires expression-level narrowing to avoid fixture changes; that is a significant compiler extension. Without it, fixtures must add per-use guards. | Guard + unwrap at every subscript use site. Style: prefer `for ch in s` over `s[i]` in range loops; prefer `.get(k, default)` over `d[k]` for dicts. |
| SC-2 | 2 membership + subset of other | Fixture (primarily) | Complete And-branch narrowing is needed as a prerequisite; but SC-1 is the deeper blocker. | Same as SC-1 — narrow the index before use. |
| SC-3 | 14 return_type_leak + 1 container | Fixture | Better diagnostic pointing to contamination source, not just the return site. | Add narrowing before write-back or before return. Prefer explicit conditional stores over `dp[i][j] = dp[i+1][j+1]`. |
| SC-4 | ~5 other_optional_flow + return_type_leak | Compiler | (a) Backward type flow for empty dict literal inference. (b) Seed nested recursive function return type from explicit `return` statements. | None after compiler fixes. |
| SC-5 | 16 argument_position | Compiler (verify) + Fixture | Verify truthy-narrowing (`if not x:`) works for `T | None` where T is a class. If it does, no compiler change needed. | Change parameter signatures from `T` to `T | None` in all recursive tree/list/graph functions. |
| SC-6 | 2 argument_position | Fixture | None | Fix builder return type or add explicit narrowing in harness. |

---

## 3. What NOT to Do

These prohibitions are grounded in architecture.md and Sifr's stated design invariants:

1. **Do not weaken safe indexing** (`x[i]` must return `T | None`). Architecture lines 144–145 are explicit. Even for `str[i]`, where the "None" case feels absurd for in-range indices, the contract must hold. Weakening it for specific types would make the behavior unpredictable across the corpus.

2. **Do not add `nonlocal` support.** Architecture line 151. No SP-4 nested closure case is a workaround justification for this. Recursive nested functions that need mutable shared state should be restructured as explicit iterative DP or as class-level methods (if Sifr's closure capture model cannot handle mutable capture).

3. **Do not add implicit None-coercion at use sites.** Silently treating `T | None` as `T` in arithmetic, comparison, or call positions would eliminate the entire category instantly — and introduce real bugs in non-LeetCode code. The whole point of returning `Option` is to force the caller to handle absence.

4. **Do not add range-based index narrowing as a quick patch.** Range analysis to prove `i ∈ [0, len(x))` and thereby narrow `x[i]` to `T` is theoretically sound, but: (a) it only helps list and string subscripts, not dict lookups, which are also a major source of SC-1; (b) it requires integer range tracking throughout the HIR, which is a substantial investment; (c) it would be a compiler feature built primarily to paper over fixture gaps. This decision belongs on the roadmap explicitly, not as a side-effect fix.

5. **Do not make `list[T | None]` assignable to `list[T]`.** Lists are invariant. Relaxing this for the SC-6 case would break type safety for all mutable container operations.

6. **Do not auto-null-guard iteration.** `for x in maybe_list` where `maybe_list: list[T] | None` silently skipping when None would hide bugs. The programmer must explicitly guard or restructure.

7. **Do not add `assert`-suppression as a general narrowing escape hatch.** `assert val is not None` is the correct Sifr idiom when the programmer holds an invariant the checker cannot prove. It is appropriate to use it in a few targeted fixtures (e.g., Floyd's cycle 0287). It is NOT a pattern to apply wholesale across the corpus — doing so converts static safety into dynamic panics, which defeats the purpose.

---

## 4. Implementation Order for Maximum Collapse

Ordering is by: (a) breadth across the corpus, (b) whether compiler-side fixes unlock class-wide fixture fixes without per-fixture effort.

### Step 1 — Verify and lock truthy-narrowing for class types (SC-5, compiler verification)

**Expected collapse: 12–16 fixtures**

`if not root:` where `root: TreeNode | None` should narrow `root` to `TreeNode` in the else-branch. `TreeNode` is not in `can_be_falsy`, so `narrow_truthiness` should correctly remove `None`. Write a targeted test confirming this:
```
narrow_type(Union(TreeNode, None), Truthiness("root"), is_true=true) == TreeNode
narrow_type(Union(TreeNode, None), Truthiness("root"), is_true=false) == None
```
If these pass, the compiler already handles SP-5 correctly for the function body. The remaining work is fixture-side: change `def maxDepth(root: TreeNode)` → `def maxDepth(root: TreeNode | None)` in all recursive tree/list/graph fixtures. This is a one-line-per-fixture mechanical change. Estimated 12–16 fixtures.

### Step 2 — Fix SC-4b: Seed nested recursive closure return type from explicit returns (SC-4, compiler)

**Expected collapse: 4–6 fixtures**

The 0010 top-down memoization and similar nested-function DP patterns fail because the inferred return of `dfs` is `Unknown | None` rather than `bool`. The fix: when inferring a nested function's return type, seed from explicit `return <literal_or_typed_expr>` statements first, then converge from call sites. The explicit `return True` and `return False` in `dfs` immediately constrain the return to `bool`. The recursive calls then unify with that `bool` anchor rather than pulling from `Unknown`. This should also fix SC-4a as a side effect if the dict's value type is inferred bottom-up from stores.

### Step 3 — Fixture canonicalization: SC-3 (DP table contamination) + SC-6 (builder helpers)

**Expected collapse: 14–16 fixtures**

For SC-3 (DP tables), the pattern `return dp[0][0]` (list) or `return cache[(0,0)]` (dict) is the terminal expression in ~14 fixtures. Each requires one of:
- `return dp[0][0] or 0` — if a default is semantically valid (int DP tables where 0 is a correct fallback)
- `if val := dp[0][0]: return val` — truthy-unwrap idiom
- Typed annotation on the table: `dp: list[list[bool]] = ...` followed by a subscript that forces narrowing

The cleanest fix for bottom-up DP: annotate the table explicitly and add a single guard at the return site. The contamination comes from read-back assignments; if those assignments are guarded, the table element type stays clean.

For SC-6: change `buildListNode` calls in `main()` to use direct `ListNode` constructors, or add post-build narrowing. Concentrated in linked-list problem fixtures.

### Step 4 — Fixture canonicalization: SC-1 bulk pass (subscript use sites)

**Expected collapse: 25–35 fixtures**

This is the largest single class and requires per-fixture work, but the patterns are highly regular. Apply by category:

**Dict lookup in arithmetic / comparison (e.g., 0013, 0020, 0122):**
- Prefer `.get(key, default)` over `d[key]` when a fallback is natural: `roman.get(s[i], 0)`. But note: `s[i]` itself returns `str | None`, so the key passed to `.get()` is `str | None`. This is itself an error unless `s[i]` is first narrowed.
- Better idiom: **prefer `for ch in s` over `for i in range(len(s)): s[i]`** for string traversal. `for ch in s` yields `str` elements (character iteration), not `str | None`. This is the single highest-leverage style change for string problems.
- For range-index loops where index arithmetic is needed (e.g., `s[i] vs s[i+1]`), accept the SC-1 narrowing burden: `if ch := s[i]: ...`.

**List subscript as next index (Floyd's cycle, 0287):**
- `slow = nums[slow]` where `slow` becomes `int | None`. Fix: `if next_val := nums[slow]: slow = next_val`. Or use `assert` if the invariant holds: `assert nums[slow] is not None; slow = nums[slow]` (the second subscript is still unnarrowed — this needs an intermediate variable). Clean form:
  ```python
  next_slow = nums[slow]
  assert next_slow is not None
  slow = next_slow
  ```
  This makes the programmer invariant explicit.

**List subscript as iteration source (0802):**
- `for nei in graph[i]:` → `if row := graph[i]: for nei in row:`. Two-line refactor.
- For `range(len(graph))` loops: `i` is provably in `[0, len(graph))`, but the compiler cannot prove this. The guard is still required until SC-1 compiler work is done.

**Dict lookup as membership element (0091, 0438):**
- `s[i] in "0123456"` where `s[i]: str | None`. Fix: change to direct string iteration where possible, or narrow: `if ch := s[i]: ch in "0123456"`.

### Step 5 — Triage residual `other_optional_flow` (28 cases)

After steps 1–4, re-run the corpus. The `other_optional_flow` bucket contains mixed patterns not captured by the specific subclusters. Expect it to drop substantially; the remaining cases should be individually triaged against the SC categories above. Likely survivors:
- `stack[-1]` returning `T | None` (negative indexing is also safe-indexed). Fix: prefer `stack.pop()` + push-back, or use a separate peek idiom.
- `deque.popleft()` returning Optional in some fixtures.
- Local variables assigned in one branch as `T | None` and in another as `T`, without explicit union narrowing in subsequent use.

---

## 5. Single Root Cause or Multiple?

**Opinion: functionally one root cause with two independent secondary issues.**

The primary root cause is a **design consequence that became a corpus debt**: safe indexing (`x[i]` → `T | None`, `d[k]` → `V | None`) was introduced correctly as a safety invariant. The LeetCode fixture corpus was written in Python style, where the programmer implicitly assumes in-bounds access and `None`-free dict lookups. The narrowing engine — being variable-centric — cannot bridge this gap without expression-level narrowing or explicit programmer guards. The corpus has neither.

SC-1, SC-2, SC-3, and SC-6 are all manifestations of this single gap. Roughly 45–50 of the 62 fixtures trace directly to it.

The two independent secondary issues:

1. **SC-4 (inference convergence for recursive closures):** This would exist even with perfect fixture canonicalization. An empty dict literal being inferred as `dict[Unknown, Unknown]` and a recursive nested function failing to converge on a return type are genuine compiler bugs, not corpus debt. They affect a small number of fixtures (~5) but are compiler-priority issues.

2. **SC-5 (parameter signature convention):** This is a mismatch between Python convention (accept `None`, handle it internally) and Sifr convention (declare the actual type space). It can be fully resolved by fixture changes once truthy-narrowing is confirmed working. It does not require compiler changes.

The 28 `other_optional_flow` cases are almost certainly a mixture of SC-1 and SC-3 patterns plus a small tail of SC-5 edge cases. They are not a sixth independent root cause.

---

## 6. Architecture Inconsistencies

### 6.1 The narrowing engine's variable-centric model is structurally mismatched with safe indexing

Safe indexing produces `T | None` at every subscript expression. The narrowing engine can only narrow *named variables*. There is no mechanism to narrow an expression like `s[i]` or `dp[j+1]`. The result is that the only way a programmer can use a safe-indexed value concretely is to first assign it to a variable and then narrow the variable:

```python
val = dp[i][j]
if val is not None:
    use(val)
```

Every programmer who writes `use(dp[i][j])` directly will get a type error, forever, regardless of how the code is structured. This is a significant ergonomics gap. The architecture should either:
- Document that safe-indexed values must be assigned and narrowed before use (establishing it as a required idiom), OR
- Plan expression-level narrowing as a future roadmap item and communicate the current limitation clearly.

Currently neither is done, and the fixture corpus reflects the resulting confusion.

### 6.2 String iteration vs. string indexing produces different types

`for ch in s: use(ch)` — `ch: str` (character iteration, no Option)
`for i in range(len(s)): use(s[i])` — `s[i]: str | None` (safe indexing)

These two patterns are semantically equivalent but produce different types. There is no documentation in the fixture corpus or architecture that states this distinction. Programmers who prefer the index-based idiom (needed when comparing adjacent characters) are forced into SC-1 territory without a clear documented path. The architecture should call this out explicitly as a style guideline: **prefer direct string/list iteration over index-based iteration whenever the index itself is not needed**. When the index is needed (comparisons like `s[i] vs s[i+1]`), a paired-iteration idiom or explicit narrowing must be used.

### 6.3 `0024_swap_nodes_in_pairs` has a call-site error that is independent of narrowing

The `main()` function calls `swapPairs(None)` where `swapPairs(head: ListNode)` declares a concrete parameter. This is not a narrowing gap — it is a fixture that explicitly passes `None` to a non-Optional parameter. The Python test is testing the edge case `swapPairs(None) == None`, which is valid in Python but requires `head: ListNode | None` in Sifr. This fixture has two errors: (1) the parameter signature, and (2) `return dummy.next` where `dummy.next: ListNode | None` and the return type is `ListNode`. Both are correctness errors in the fixture, not compiler gaps.

### 6.4 `0023_merge_k_sorted_lists` has a `buildListNode` helper that is incompatible with `mergeKLists`

`buildListNode` returns `ListNode | None`, which is semantically correct (empty input → `None`). `mergeKLists` takes `list[ListNode]`. The `main()` harness builds `[buildListNode(...), ...]` → `list[ListNode | None]`. These are structurally incompatible, and the incompatibility is in the test harness design. The algorithm itself is correct. The harness needs to either use a non-Optional builder, add per-element narrowing, or change `mergeKLists` to accept `list[ListNode | None]` (which changes the algorithm's contract). This fixture has an internal design tension that is independent of the narrowing engine.

### 6.5 Empty dict literal inference (`{}`) does not propagate from subsequent assignments

`cache = {}` is inferred as `dict[Unknown, Unknown]`. Subsequent `cache[k] = True` assignments should allow the checker to narrow the dict's value type to `bool`. If this backward inference is not happening, every memoization pattern that starts with `cache = {}` will produce `Unknown | None` on subscript read rather than `bool | None`. This is the SC-4a issue. The architecture does not document whether empty literal inference is forward-only or bidirectional; it should.

### 6.6 `dict.get(key, default)` vs. `dict[key]` semantics are inconsistent across the fixture corpus

Some fixtures use `d.get(k, 0)` (returns `int`, non-Optional because default is provided) and pass. Others use `d[k]` (returns `int | None`) and fail. The corpus has no consistent policy. Given that `.get(key, default)` is the preferred Sifr idiom for "access with a fallback," the fixture style guide should explicitly recommend it over `d[k]` for all lookup-with-default cases. The current mixed usage makes the pass/fail pattern in the corpus appear unpredictable.

---

## Summary Table

| SC | Cluster | ~Count | Fix | Compiler Δ | Fixture Δ | Priority |
|---|---|---|---|---|---|---|
| SC-1 | arithmetic, index-expr, membership, iteration, other | ~30 | Fixture | None (expression narrowing is a future feature) | Per-use guard or idiom shift (`for ch in s`, `.get(k,d)`) | Step 4 |
| SC-2 | membership (subset) | ~2 | Fixture | Complete And false-branch narrowing (minor) | Same as SC-1 | Step 4 |
| SC-3 | return_type_leak, container-inference | ~15 | Fixture | Better contamination diagnostic | Guard at write-back or return site | Step 3 |
| SC-4 | return_type_leak (Unknown), other | ~5 | Compiler | Seed nested-fn return type from explicit returns; backward empty-dict inference | None after compiler fix | Step 2 |
| SC-5 | argument_position | ~16 | Compiler (verify) + Fixture | Confirm truthy-narrowing for class types | Change `T` → `T | None` in recursive fn signatures | Step 1 |
| SC-6 | argument_position (subset) | ~2 | Fixture | None | Fix harness builder or add post-build narrowing | Step 3 |

**Total addressable: ~70 diagnostic instances across 62 fixtures.**

The fastest path to collapse: Steps 1 and 2 are compiler verifications/fixes that unlock Step 3 (mechanical fixtures) and significantly reduce Step 4 (bulk canonicalization). Do not start the bulk SC-1 fixture pass until SC-4 and SC-5 are resolved, or a portion of the work will be redone.
