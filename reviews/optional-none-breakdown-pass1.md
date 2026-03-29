# Optional/None Failure Family — Root-Cause Breakdown Pass 1

**Date:** 2026-03-29
**Corpus run:** PASS=97 / CHECK_ERROR=290 / RUN_ERROR=24
**Category under analysis:** Optional/None flow and narrowing gap (62 fixtures)
**Subclusters from live data:** 79 total (28 other + 16 argument + 15 arithmetic/comparison + 14 return-leak + 2 membership + 2 index-expr + 1 local-assign + 1 container-inference + 1 iteration-source)

---

## 1. Real Compiler Subproblems (not symptom buckets)

The surface diagnostic labels (optional_arithmetic, return_type_optional_leak, etc.) are all manifestations of a smaller set of genuine compiler subproblems. This section names them precisely.

### SP-1 · Option result used without narrowing at an arithmetic/comparison/index/iteration site

**What is happening:** `list[i]`, `dict[k]`, and `str[i]` all correctly return `T | None` per Sifr's safe-indexing contract. The fixtures then use the result directly in arithmetic (`roman[s[i]] + ...`), as a subsequent index (`nums[nums[fast]]`), as a membership operand (`s[i] in "0123456"`), or as an iteration source (`for nei in graph[i]`). None of these are handled.

**Representative fixtures:** 0013, 0020, 0091, 0122, 0287, 0438, 0802
**Affects subclusters:** optional_arithmetic_or_comparison (15), optional_index_expression (2), optional_membership_operand (2), optional_iteration_source (1), large fraction of other_optional_flow (28)

**Why it is one subproblem, not four:** The mechanism is identical in all four surface forms — the checker propagates `T | None` from a subscript result into a downstream position that requires `T`. The downstream position is what varies (operand, index, membership element, iteration source). One fix in the type narrowing / implicit-guard layer handles all four shapes.

### SP-2 · DP/memoization table: Option contaminates the stored value, which poisons the return type

**What is happening:** A DP table or memo dict (`dp`, `cache`, `row`) is built by assigning subscript results into cells. Because `dp[i][j+2]` returns `bool | None` (safe indexing), that `bool | None` is stored back into the table, making the table's inferred element type `bool | None`. The function's return expression `dp[0][0]` or `row[0]` then has type `int | None` or `bool | None`, which conflicts with the declared `int` or `bool` return type.

**Representative fixtures:** 0010 (bottom-up portion), 0062, 0063, 0097, 0115
**Affects subclusters:** return_type_optional_leak (14), optional_contaminates_container_inference (1)

**Relationship to SP-1:** SP-1 is "Option used immediately at a use-site." SP-2 is "Option stored into a container and then returned." They share the same root (safe indexing returns Option) but require different fixes: SP-1 needs narrowing before use; SP-2 needs either narrowing before storage or unwrap-on-return.

### SP-3 · Recursive function called with an Optional argument where the declaration is concrete

**What is happening:** A tree or linked-list function is declared with a concrete parameter (`def maxDepth(root: TreeNode)`) but the recursive call site passes an Optional field (`maxDepth(root.left)` where `root.left: TreeNode | None`). The checker correctly rejects this.

**Representative fixtures:** 0024, 0104, 0133, 0206, 0023
**Affects subclusters:** argument_position_requires_concrete (16)

**This is NOT the same as SP-1 or SP-2.** The Option does not originate from safe indexing here; it originates from a struct field declared as `T | None`. The function signature itself is the problem — in idiomatic Python these functions accept `None` and handle it internally (often as the base case). The fixture declarations are wrong for Sifr: either the parameter should be `T | None` with an internal None-guard, or the call site should narrow before passing.

### SP-4 · Recursive closure / nested function: inferred return type degrades to `Unknown | None`

**What is happening:** A nested `dfs` function (0010 top-down memoization) returns `cache[(i,j)]` where `cache: dict[tuple[int,int], bool]`. Since `dict[K]` returns `V | None`, the inferred return type of `dfs` is `bool | None`. The outer function then returns `dfs(0, 0)` against a declared `bool` return type. A separate symptom is the `Unknown | None` form — where the checker cannot determine the concrete type of a recursive nested function at all and falls back to `Unknown`.

**Representative fixtures:** 0010 (top-down), 0097 (if similar nested pattern exists)
**Affects subclusters:** return_type_optional_leak (partially), a portion of other_optional_flow

**What distinguishes this from SP-2:** SP-2 is about reading from a DP table at the *return expression* of a non-nested function. SP-4 is about the *return type of a nested recursive function* degrading because of safe-indexing pollution combined with possible inference cycles. The `Unknown` variant specifically indicates the type inference engine is not converging on a fixed point for the recursive call.

### SP-5 · Builder / helper returns `T | None`; callers place result in a `list[T]` container

**What is happening:** `buildListNode(values)` returns `ListNode | None` (correct — an empty input could yield None). The caller builds `[buildListNode([1,4,5]), buildListNode([1,3,4]), ...]`. The inferred type is `list[ListNode | None]`. `mergeKLists` is declared as `list[ListNode]`. The mismatch is caught correctly.

**Representative fixtures:** 0023
**Affects subclusters:** argument_position_requires_concrete (partially)

**This is a fixture design issue specific to test harness helpers.** The builder function is semantically correct to return Optional (it handles the empty-list case), but in the `main()` test harness the caller knows the input is non-empty and the result is never None. The checker cannot know this.

---

## 2. What Each Subproblem Requires

| Subproblem | Correct fix layer | Compiler work needed |
|---|---|---|
| SP-1 | Control-flow narrowing + explicit unwrap discipline in fixtures | Range-provable in-bounds narrowing is out of scope; narrowing after `if val is not None` already works. Fixtures must add guards or use a safe-unwrap idiom. |
| SP-2 | Container element refinement + fixture narrowing at return site | Compiler could provide better diagnostics pointing to where None contaminated the container; the actual fix is fixture-side narrowing before store or before return. |
| SP-3 | Fixture signature / caller-side narrowing | Truthy narrowing (`if not root: return 0`) should already eliminate None from `root` in the subsequent branch. If this narrowing is not working, that is a compiler gap. If it is working, the remaining failures are from the *call site* passing `root.left` un-narrowed. |
| SP-4 | Recursion / inference convergence | The `Unknown` form is a compiler inference issue. Nested recursive closures should have their return type inferred from the explicit `return` statements, not from read-back of a polymorphic container. This is a genuine type-inference gap. |
| SP-5 | Fixture canonicalization | No compiler work needed. The builder should return `ListNode` when input is guaranteed non-empty, or the harness should use a post-hoc unwrap. |

---

## 3. What NOT to Do (Sifr Principles)

1. **Do not weaken safe indexing.** `list[i]`, `dict[k]`, and `str[i]` must continue to return `T | None`. This is a core safety invariant, not a convenience policy.

2. **Do not add nonlocal.** This is explicitly unsupported by design (architecture.md lines 149–151). Nested mutable state shared via `nonlocal` is not a permitted workaround for any of these failures, including the SP-4 nested closure cases.

3. **Do not add implicit None-coercion.** Silently converting `T | None` to `T` at arithmetic, index, or return sites — even when the compiler could prove the index is in-bounds — would undermine the ownership/safety guarantee. It would also mask real bugs in non-LeetCode code.

4. **Do not add range-based index narrowing as a short-path fix.** It is theoretically possible to narrow `list[i]` to `T` (not `T | None`) when `i` is provably in the range `[0, len(list))`. However, this requires a non-trivial integer range analysis, and even then it only helps SP-1 cases where the index is a loop variable. The majority of SP-1 failures also involve dict lookups (not bounds-checkable) and would not benefit. Build this only if the roadmap scope explicitly calls for it. Do not build it to paper over fixture gaps.

5. **Do not fix these failures by loosening return-type checking.** If a function declares `-> int` and the inferred return is `int | None`, that is a real type error. The function is wrong, not the checker.

6. **Do not auto-wrap `for x in optional_collection` into a null-safe loop.** Silently skipping a `for` loop body when the iterable is None would hide logical errors. If `graph[i]` is None, that is probably a bug in the algorithm, not a case to silently skip.

---

## 4. Implementation Order for Maximum Collapse

Order is determined by: (a) number of fixtures affected, (b) whether a compiler fix enables a class of fixture fixes without touching each fixture individually.

### Step 1 — Fix SP-3: Truthy narrowing for Optional tree/graph/list node parameters (compiler)

**Expected collapse:** 12–16 fixtures in argument_position_requires_concrete.

`if not root: return 0` is the standard base-case guard in every recursive tree/graph problem. If the checker correctly narrows `root: TreeNode | None` to `TreeNode` in the else-branch after a falsy check, and if the function parameter is changed to `TreeNode | None`, then every recursive call `maxDepth(root.left)` becomes valid without touching the algorithm body.

The compiler work is: ensure that `if not x:` (where `x: T | None`) narrows `x` to `T` in the branch where the condition is false (the non-None branch). This is straightforward narrowing that should already exist for `if x is not None`; verify it also works for truthiness.

The fixture work is: change `def maxDepth(root: TreeNode)` → `def maxDepth(root: TreeNode | None)` for all recursive tree/list/graph functions. This is a mechanical, one-line change per fixture.

### Step 2 — Fix SP-4: Nested recursive closure return-type convergence (compiler)

**Expected collapse:** 4–8 fixtures in return_type_optional_leak / other_optional_flow.

The `Unknown | None` return type from a recursive nested function is a pure compiler inference gap. The fix is to ensure the checker seeds the recursive closure's return type from its explicit `return` statements first, then converges — rather than propagating `Unknown` from an un-inferred recursive call back through a dict read. This is a standard "type inference with recursive types" problem. Fixing it would immediately correct cases like 0010's top-down memoization where `dfs` should infer `bool`, not `Unknown | None`.

### Step 3 — Fixture canonicalization: SP-2 and SP-5 (fixture-side)

**Expected collapse:** 14–15 fixtures (return_type_optional_leak + optional_contaminates_container_inference + SP-5 harness issues).

For SP-2 (DP tables): the pattern `return dp[0][0]` must become one of:
- `if val := dp[0][0]: return val` (truthy-narrowing idiom)
- A dedicated safe-unwrap expression if Sifr provides one (e.g., `dp[0][0] or 0` for int, or `dp[0][0]!` if an assert-unwrap operator is available)

For SP-5 (builder returns Optional in harness): change the `buildListNode` builder to return `ListNode` when input is guaranteed non-empty, or add explicit narrowing in `main()`.

This is fixture work, but it is mechanical and concentrated — most DP problems have exactly one return statement reading from the table.

### Step 4 — Fixture canonicalization: SP-1 (fixture-side, bulk)

**Expected collapse:** 20–30 fixtures across optional_arithmetic, optional_index, optional_membership, optional_iteration, other_optional_flow.

This is the largest single class but also the most spread out. The patterns to address:

- **Dict lookup used in arithmetic** (`roman[s[i]]`): must be narrowed. Preferred idiom: `if val := roman[s[i]]: res += val`. The underlying cause is that `s[i]` also returns `str | None`, so even `roman.get(s[i])` is insufficient — the key lookup itself is Optional.

  **Two-level fix required:** (a) `s[i]` returns `str | None` → narrow the character first, (b) `roman[char]` returns `int | None` → narrow the value.

- **Index used as subsequent index** (Floyd's cycle, 0287): `slow = nums[slow]` where after the first iteration `slow: int | None`. The fixture must be restructured to assert non-None after each step, or use a typed intermediate: `if next_val := nums[slow]: slow = next_val else: break`. Note that `assert slow is not None` (the `assert` keyword being Sifr's one panic point) is explicitly allowed and appropriate here — it represents a programmer invariant that the cycle is guaranteed to close.

- **String character used in membership test** (`s[i] in "0123456"`): `s[i]` returns `str | None`, must be narrowed before use. Idiom: `if ch := s[i]: ...` or iterate directly over `s` (which yields `str` elements, not `str | None`).

- **Iteration over Optional collection** (`for nei in graph[i]`): must add `if row := graph[i]: for nei in row`. This is a 2-line refactor per occurrence.

### Step 5 — Audit residual `other_optional_flow` (28 cases)

After steps 1–4 address the named subclusters, audit what remains in `other_optional_flow`. These likely include:
- Stack operations: `stack[-1]` returns `T | None` (safe indexing on a list) — needs narrowing before comparison.
- `deque.popleft()` / `heapq.heappop()` returning Optional in some fixtures.
- Local variable assignments that receive Optional from one branch and non-Optional from another without union-resolving.

These must be triaged individually after the structural fixes in steps 1–4 are applied, since some may disappear as side effects.

---

## 5. Is This One Root Cause or Several Independent Root Causes?

**Opinion: it is one primary root cause with three secondary complications.**

The primary root cause is: **safe-indexing returns Option, and the fixture corpus was written in Python style where the programmer implicitly assumes in-bounds access.** Every single subproblem (SP-1 through SP-5) ultimately traces back to this gap between "Python programmer intent" and "Sifr's mandatory Option handling."

The three secondary complications that prevent a single mechanical fix:

1. **SP-3 (parameter signatures):** Function signatures that declare `T` instead of `T | None` are a fixture correctness issue, not an indexing issue. The values come from struct fields, not subscripts. Compiler narrowing (truthy-checks) can fix this without touching every call site.

2. **SP-4 (inference for recursive closures):** The `Unknown | None` regression is a genuine inference convergence bug in the compiler, independent of whether safe-indexing was involved. It would manifest even if the fixtures were well-written.

3. **SP-5 (builder helpers returning Optional into typed containers):** This is a test-harness design pattern issue. It is concentrated in linked-list/tree problems that use a shared builder, not spread across the corpus.

If SP-4 (compiler) and SP-3 (truthy narrowing) are fixed first, the remaining 45–50 fixtures are all variations on one mechanical problem: "unwrap the Option from a subscript result before using it." Those can be fixed with fixture canonicalization following a consistent style guide.

---

## 6. Architecture Inconsistencies Observed

### 6.1 Fixture parameter signatures are inconsistent with Sifr's Option model for recursive functions

`def maxDepth(root: TreeNode)` vs. the recursive call `maxDepth(root.left)` where `root.left: TreeNode | None` — this cannot work unless either:
- The parameter is `TreeNode | None` (correct for Sifr), OR
- The call site narrows before passing (verbose, against convention for base-case guard patterns)

The broader LeetCode fixture corpus has used `T` (non-optional) for these parameters, which is Python convention (the `None` is implicit). For Sifr, the canonical form is `T | None` with an explicit guard. The corpus needs a consistent policy here; currently it is mixed.

### 6.2 `dict[K]` returns `V | None` but `dict.get(K, default)` semantics are unclear in the fixture corpus

Several fixtures use `dict.get(key, 0)` correctly (e.g., 0438's `pMap.get(char, 0)`) while others use `dict[key]` with no narrowing. The fixtures that correctly use `.get()` likely pass; those that use `dict[key]` fail. The corpus is not consistent on when `.get()` vs. `[]` is used, and this inconsistency generates a large portion of SP-1 failures.

If `dict.get(key, default)` returns `V` (non-optional, because a default is provided), then `.get()` is the idiomatic Sifr fix for most dict-access SP-1 failures. The audit corpus should use `.get(key, default)` wherever a fallback is natural, reserving `dict[key]` for cases where the programmer genuinely needs to handle absence.

### 6.3 The `assert` keyword (permitted panic) is under-used as a narrowing tool

For SP-1 / SP-2 cases where the programmer genuinely knows an index is valid (e.g., Floyd's cycle detection invariants, DP base cases), `assert val is not None` is the correct Sifr idiom. It represents a programmer invariant check, which is exactly what `assert` is for per the safety philosophy. The fixtures do not use this idiom at all — they were written before Sifr's Option model was in place. Establishing this as a documented pattern would make fixture canonicalization faster and more consistent.

### 6.4 String character iteration vs. string character indexing

`for ch in s` yields `str` elements (no Option wrapping) in standard iterable semantics. `s[i]` returns `str | None`. In many SP-1 membership failures (0091, 0438), the fixture uses `s[i]` inside a `range(len(s))` loop when it could use direct character iteration instead. Establishing a style guideline — "prefer `for ch in s` over `for i in range(len(s)): use s[i]`" — eliminates an entire class of Optional contamination without compiler changes. This is not yet reflected in the fixture corpus.

---

## Summary Table

| Subproblem | Count | Fix Layer | Compiler Work | Fixture Work | Priority |
|---|---|---|---|---|---|
| SP-1: Subscript Option used without narrowing | ~30 | Fixture | None (narrowing already exists, isn't used) | Per-fixture guard/idiom | Step 4 |
| SP-2: Option contaminates DP/memo table, leaks to return | ~15 | Fixture | Better diagnostic pointing to contamination source | Unwrap at return/store site | Step 3 |
| SP-3: Concrete param receives Optional tree/list/graph field | ~16 | Compiler + Fixture | Verify truthy-narrowing works for `T | None` params | Change param signatures to `T | None` | Step 1 |
| SP-4: Recursive closure infers `Unknown | None` | ~5 | Compiler | Fix inference convergence for recursive nested closures | None after compiler fix | Step 2 |
| SP-5: Builder returns Optional, placed in typed container | ~2 | Fixture | None | Fix builder return type or add harness narrowing | Step 3 |

**Total addressable in this pass: ~68 fixture failures** (62 identified + 6 likely miscategorized in other families).
