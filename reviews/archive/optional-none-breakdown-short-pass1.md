# Optional/None Flow and Narrowing — Failure Analysis Pass 1

**Date:** 2026-03-29
**Corpus:** `full_corpus_current_results_20260329_live.json` (411 LeetCode cases)
**Status:** 97 PASS (23.6%) / 314 FAIL (76.4%)

---

## Raw Numbers

From all failing cases, error line categorization:

| Error family | Line count across all failures |
|---|---|
| `None union operand / assignment` (`X \| None` used where `X` expected) | 211 |
| `Attribute access on node fields` (`.next`, `.val`, `.left`, `.right`) | 114 |
| `Return type mismatch (None)` (function returns `T \| None`, sig says `T`) | 70 |
| `Borrowed parameter escape` | 37 |
| `Use of moved value` | 29 |
| `Cannot assign X\|None to X` | 5 |

**Optional/None is the dominant failure family by a wide margin.** The 211 union operand errors + 70 return type mismatches + the majority of the 114 attribute access errors (triggered because the accessed value hasn't been narrowed) sum to ~300+ error lines across ~190 failing cases.

---

## Real Compiler Subproblem Split

There are four distinct subproblems. They co-occur in linked list / tree problems, which is why they look like one blob.

### Subproblem 1: Flow-Sensitive Narrowing (CFA)

**What it is:** The narrowing engine must track that after `if x is not None:` or `while x is not None:`, the type of `x` inside the branch/body is `T`, not `T | None`. It must also handle truthiness narrowing: `if x:` should strip `None` from a union when no other falsy member remains.

**What is broken:** It's currently not happening. The compiler doesn't reduce the type after the guard.

**Concrete evidence:**
```python
# 0021 — Merge Two Sorted Lists
cur: ListNode | None = node
while cur is not None:
    parts.append(str(cur.val))   # error: attribute access on X | None
    cur = cur.next               # error: attribute access on X | None
```
Inside the `while cur is not None:` body, `cur` should be `ListNode`. It isn't.

```python
# 0021 — recursive variant
def mergeTwoLists(list1: ListNode | None, list2: ListNode | None) -> ListNode | None:
    if not list1:
        return list2
    if not list2:
        return list1
    # Here list1 and list2 should both be ListNode, not ListNode | None
    lil, big = (list1, list2) if list1.val < list2.val else (list2, list1)
```
After `if not list1: return list2`, `list1` should be narrowed to `ListNode`. It isn't.

**Loop-back-edge re-narrowing:** A specific sub-case of this. After `cur = cur.next` at the end of a loop body, `cur` becomes `ListNode | None`. On the next iteration, the `while cur is not None` condition must re-narrow it before entering the body. This requires the narrowing engine to treat loop conditions as guards that fire on every iteration, not just the first.

**Scope:** Fixes a large chunk of every linked list and tree problem. Probably 100+ failing cases.

---

### Subproblem 2: Container Indexing Inference Propagation

**What it is:** `list[i]` correctly returns `T | None` per Sifr's safe indexing contract. This is NOT a bug. But the type correctly propagates downstream, and LeetCode DP code that does `row[j + 1] + row[j]` or `return row[0]` breaks because neither arithmetic on `int | None` nor returning `int | None` from an `int` function is allowed.

**What is broken:** Nothing in the type system. The user code is relying on indices being in-bounds (a runtime invariant the compiler can't see), and the compiler forces them to handle it. The missing piece is ergonomic handling of the "I know this index is in bounds" case.

**Concrete evidence:**
```python
# 0062 — Unique Paths
row = [1] * n
for j in range(n - 2, -1, -1):
    newRow[j] = newRow[j + 1] + row[j]   # error: int | None + int | None
return row[0]                             # error: int | None, expected int
```
`[1] * n` produces `list[int]`. `row[j]` is correctly `int | None`. The user knows `j` is in bounds.

**Scope:** Dominates DP problems (0062, 0063, 0064, 0013, 0016, and every 2D grid problem). These are among the most common LeetCode problem types.

**What the user must do (Sifr-compliant pattern):** Either use iterators instead of indexed access, or use `assert` to express the invariant:
```python
val = row[j]
assert val is not None   # programmer invariant: j is in bounds
newRow[j] = val + ...
```
The `assert` path is already available. The ergonomics are painful for DP code.

---

### Subproblem 3: Recursive-Shape Field Traversal

**What it is:** Classes like `ListNode` and `TreeNode` have fields that are optionally `self`-typed: `next: ListNode | None`, `left: TreeNode | None`. Every field read on such a node produces `T | None`. This chains: `node.next.val` is three levels of optionality. Narrowing must survive across the traversal.

**What is broken:** Two things:
1. Subproblem 1 (narrowing) not working means the base case `node.next` is already `ListNode | None` even inside a narrowed scope.
2. Assignment `node = node.next` re-assigns the narrowed variable to an un-narrowed value, which is correct behavior — but the loop condition must re-narrow it.

**This is NOT a separate fix from Subproblem 1.** It's the same narrowing engine applied to recursive shapes. If CFA is fixed, recursive shape traversal works. No special case needed.

**Concrete evidence:** 0002 (add two numbers), 0019 (remove nth node), 0021, 0023, 0024, 0025, 0141 (linked list cycle), all tree problems. The `.next`, `.val`, `.left`, `.right` attribute access errors (114 lines) are overwhelmingly here.

**Note on the attribute access error message:** The error "attribute access '.next' is not supported as an expression; use as a method call" may indicate a second orthogonal bug — that the compiler restricts field reads in expression position for class instances generally (not just Optional ones). If so, fixing narrowing alone won't fix these failures. The field-as-expression limitation is a separate compiler gap to diagnose independently.

---

### Subproblem 4: Container Refinement (list of Optional → list of T)

**What it is:** When a function builds a `list[T | None]` and returns or passes it where `list[T]` is expected, there's no way to statically prove all elements are non-None without iterating and filtering.

**Concrete evidence:**
```python
# 0056 — Merge Intervals
result: list[list[int] | None] = [...]
return result   # error: expected list[list[int]], got list[list[int] | None]

# 0010 — Regular Expression Matching
cache = [[False] * (len(p) + 1) for i in range(len(s) + 1)]
return cache[0][0]   # returns bool | None, expected bool
```
In 0010, the cache is built with all-`False` values, so `cache[i][j]` is semantically always `bool`. The type system correctly sees `bool | None`.

**Scope:** Affects DP memoization, interval merging, any code that builds a result list via append then returns it. Roughly 15–25 cases at the direct container level.

**What the user must do:** Filter None values explicitly, use an explicit annotation with assertion, or restructure to avoid optional container elements.

---

## What Not to Do (Preserving Sifr Principles)

1. **Don't auto-unwrap `T | None` to `T`.** Making `list[i]` "just work" like Python's panicky indexing would violate the "if it compiles, it works" guarantee. The whole point is that callers handle absence.

2. **Don't add `.unwrap()` or `.expect()`.** These generate `panic!()`. Panics are prohibited in user-facing runtime paths. The no-panic contract is non-negotiable.

3. **Don't special-case linked list / tree traversal** with magic narrowing outside the general CFA engine. The architecture explicitly models this as TypeScript-style flow narrowing. Adding ad-hoc "if the loop condition is `x is not None`, auto-narrow `x`" as a special rule outside the narrowing engine creates a divergent codepath that will break in edge cases and obscures the real fix.

4. **Don't weaken container type tracking** by making `[1] * n` produce `list[int]` where `list[int][i]` returns `int` (not `int | None`). Safe indexing is a first-class language guarantee. The fix for DP code is making the `assert`/narrowing ergonomics better, not silently making indexing unsafe.

5. **Don't add `Optional[T]` as a special type distinct from `T | None`.** The architecture is explicit: `T | None` is `Union([T, None])`, Rust codegen is `Option<T>`. Creating a divergent `Optional` path adds complexity without fixing any of the actual subproblems.

6. **Don't add implicit None-filtering on container operations** (e.g., "if you append `T | None` to a `list[T]`, silently drop the `None`"). Caller must handle explicitly.

---

## Best Fix Order

Ranked by case coverage / fix-to-impact ratio:

### 1. `is not None` / `is None` narrowing in `if` and `while` (Subproblem 1 core)

This single fix has the highest case coverage. Every linked list problem, every tree problem, every function that guards on None before using a value depends on it. Implement TypeScript-style CFA: when the condition is `x is not None`, the true branch has `x: T`, the false branch has `x: None`. When the condition is `x is None`, flip. This is the architecture's stated design — it just needs to be correctly implemented and exercised by the narrowing engine.

**Estimated impact:** 80–100 cases unblocked once field access also works.

### 2. Truthiness narrowing for None (`if x:` strips None)

After `if x:`, if `x: T | None`, narrow `x` to `T` in the body (when `T` has no falsy instances). This is how Python idioms like `if not list1: return list2` and the recursive `mergeTwoLists` variant work. Required for tree/list recursive solutions.

**Estimated impact:** 20–40 additional cases.

### 3. Loop-back-edge re-narrowing

Ensure the narrowing from a `while` condition is re-applied after each loop iteration, so that `cur = cur.next` (which makes `cur: ListNode | None`) is re-narrowed to `ListNode` by the `while cur is not None` condition on the next pass. This is flow-sensitive analysis across loop back-edges — harder to implement but critical for any while-based traversal.

**Estimated impact:** ~30 cases that use while-based linked list traversal.

### 4. Diagnose and fix field access in expression position (Subproblem 3 dependency)

The "attribute access '.next' is not supported as an expression; use as a method call" error needs a root cause investigation. If field reads on class instances are broken as a general matter (not just Optional-related), narrowing won't help those cases. Fix this in parallel with or before the narrowing work.

**Estimated impact:** Unblocks the 114 attribute-access error lines across ~50 cases.

### 5. Better ergonomics for "I know this is in bounds" (Subproblem 2)

The `assert val is not None` pattern already works as a programmer invariant (generates `assert!(...)` in Rust). The problem is LeetCode code doesn't have these assertions. Two options:

- **Short-term:** Make the compiler error for `X | None` used in arithmetic very actionable, pointing to the `assert`/narrowing pattern explicitly.
- **Medium-term:** Consider a stdlib helper or syntax sugar for the "checked unwrap" pattern that is still safe (e.g., a `sifr.unwrap(x, default)` that is not a panic but a fallback).

Do NOT do this by weakening container indexing types.

**Estimated impact:** 20–30 DP cases, but requires user code changes, not just compiler fixes.

### 6. Container refinement ergonomics (Subproblem 4)

Last because it requires user code changes regardless. What the compiler can do: emit an actionable error suggesting `.filter_some()` (if such a method exists or is added to stdlib) when a `list[T | None]` is returned where `list[T]` is expected. Or suggest explicit narrowing loop.

**Estimated impact:** 15–20 cases, but mostly requires user code to change.

---

## Summary Table

| # | Fix | Subproblem | Compiler change | User code change | Estimated cases unblocked |
|---|---|---|---|---|---|
| 1 | `is not None` / `is None` CFA in if/while | 1 | Yes | No | 80–100 |
| 2 | Truthiness narrowing (`if x:` strips None) | 1 | Yes | No | 20–40 |
| 3 | Loop back-edge re-narrowing | 1 | Yes (harder) | No | ~30 |
| 4 | Field access as expression (diagnose) | 3 | Yes | No | ~50 |
| 5 | Assert/narrowing ergonomics for indexing | 2 | Minor | Yes | 20–30 |
| 6 | Container refinement error messages | 4 | Diagnostic only | Yes | 15–20 |

Fix 1+2+3+4 together unblock linked list and tree problems entirely if the user code is otherwise correct. Fix 5 requires thinking about DP idioms against safe indexing. Fix 6 is polish.
