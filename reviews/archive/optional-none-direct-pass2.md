Now I have enough evidence. Let me provide the rigorous critique.

---

## Pressure-Test Results: Refined Analysis

### Overclaim #1: "Truthy narrowing should already work"

**Verdict: Partially false.**

The `detect_narrowing_condition` implementation at `statements.rs:1838` handles:
- `x is None` / `x is not None`
- `x == literal`  
- `isinstance(x, T)`
- Simple variable names for truthiness (`x:` in `if x:`)

For `while curr and curr.next:` (0024_swap_nodes_in_pairs), the BoolOp handler collects conditions from each operand. `curr` is a `Name` → gets `Truthiness(curr)`. `curr.next` is an `Attribute` → falls through to `_ => None`. **Only `curr` is narrowed, not `curr.next`.**

The claim "truthy narrowing should already eliminate None from `root` in the subsequent branch" is imprecise. It eliminates None from the **variable itself**, not from **fields accessed on that variable**. `root.left` on a narrowed `root` is still `TreeNode | None`.

### Overclaim #2: "The bucket is mostly one root cause"

**Verdict: Conflates three fundamentally different problem types.**

| Problem | Root cause | Fix layer |
|---|---|---|
| SP-1: `roman[s[i]]` where `s[i]` is `str \| None` | Fixture uses Python indexing idiom, Sifr returns `T \| None` | Fixture canonicalization |
| SP-2: DP table return `bool \| None` | `dict[K]` returns `V \| None` stored back, pollutes return | Fixture unwrap + possibly SP-4 inference |
| SP-3: `maxDepth(root.left)` where param is `TreeNode` | Function signature is wrong for Sifr's Optional model | Fixture signature fix |
| SP-4: `dfs` infers `Unknown \| None` | Recursive closure inference doesn't converge | Compiler inference fix |
| SP-5: `list[ListNode \| None]` vs `list[ListNode]` | Builder returns Optional, container is homogeneous | Fixture canonicalization |

The **primary root cause** framing (safe-indexing returns Option) is correct as a design observation but unhelpful as a fix driver. Each subproblem requires different action.

### Overclaim #3: "Fix order: narrowing → inference cleanup → container refinement → recursive base-case typing → fixture canonicalization"

**Verdict: The ordering is wrong and incomplete.**

**Problem 1**: SP-3 (truthy narrowing) is listed first and claimed to fix `maxDepth`. But:
- `if not root: return 0` narrows `root` to `TreeNode` in the else-branch
- `root.left` is STILL `TreeNode | None` — the field type doesn't change
- The recursive call `maxDepth(root.left)` still passes `TreeNode | None` to `TreeNode` parameter
- SP-3 is **NOT a narrowing issue** — it's a fixture signature issue. Changing `def maxDepth(root: TreeNode)` → `def maxDepth(root: TreeNode | None)` is the fix, with an internal base-case guard.

**Problem 2**: SP-4 (recursive closure inference) is listed second, but it's **blocking** SP-2. In 0010's top-down memoization, the `dfs` return type is degraded to `Unknown | None` because of inference failure. Fixing inference first makes SP-2 clearer (it's just an unwrap issue), leaving it in makes SP-2 look like a deeper type system problem.

**Problem 3**: "Container refinement" as a separate step is mis-framed. There's no mechanism in the type system to refine `list[T | None]` → `list[T]` based on a guard. The fix is unwrap-at-storage or unwrap-at-retrieval, both fixture-side.

### What Actually Blocks What

```
SP-4 (recursive closure inference) ──┐
                                    ├──→ SP-2 (DP/memo table Optional leak)
SP-1 (subscript Optional) ──────────┘         │
                                             │
SP-3 (parameter signatures) ─────────────────┼──→ SP-5 (builder Optional)
         │                                   │
         │                                   │
         └───────────────────────────────────┴──→ Fixture canonicalization (SP-1 bulk)
```

**Parallelizable**: SP-3 (fixture signature fixes) and SP-5 (builder return types) can proceed immediately — they're fixture-only. SP-1 bulk canonicalization is fixture-only and can proceed in parallel.

**Blocking**: SP-2 (DP/memo) depends on SP-4 (inference) because SP-4 causes the inference degradation that makes SP-2 look worse than it is.

### Revised Workstream Decomposition with Relative Impact

| Workstream | Fixtures | Type | Priority |
|---|---|---|---|
| **WS-A**: Recursive closure inference convergence | ~5 (0010, 0097, 0017, 0039, 0050) | Compiler bug | **Blocker for WS-C** |
| **WS-B**: Fixture signatures for recursive tree/graph/list functions | ~16 (0024, 0104, 0133, 0206, 0023, etc.) | Fixture | Parallel with WS-A |
| **WS-C**: DP/memo return unwrap (dict[K] → V, not V \| None) | ~15 (0010 bottom-up, 0062, 0063, 0097, 0115, etc.) | Fixture (after WS-A) | After WS-A |
| **WS-D**: Subscript Optional guards (SP-1 bulk) | ~30 (0013, 0020, 0091, 0122, 0287, 0438, 0802, etc.) | Fixture | Parallel |
| **WS-E**: Builder return type canonicalization | ~2 (0023 harness, maybe 0148) | Fixture | Parallel |

### Revised Fix Order

1. **WS-A first** (compiler): Fix recursive closure inference convergence. This unblocks WS-C and clarifies whether SP-2 is truly just an unwrap issue.

2. **WS-B, WS-D, WS-E in parallel** (fixture): These are independent fixture-only fixes. No blocking relationships.

3. **WS-C third** (fixture, after WS-A clarifies): After inference is fixed, the DP/memo failures become clearly about unwrap-at-return. The fix is mechanical: `return dp[0][0]` → `if val := dp[0][0]: return val`.

### Actual Compiler Bugs vs Fixture Rewrites

**Compiler bugs (real)**:
- WS-A: Recursive closure inference doesn't converge → `Unknown | None` return types

**Compiler bugs (claimed but actually fixture)**:
- SP-3 "truthy narrowing doesn't work for tree/list parameters" — narrowing DOES work for the variable itself. The fixture signatures are simply wrong for Sifr's model. No compiler change needed.

**Fixture rewrites only**:
- SP-1: Add guards/walks around every subscript Optional use
- SP-2: Unwrap at return site or restructure DP table access
- SP-3: Change parameter types from `T` to `T | None`
- SP-5: Change builder return types or add harness unwrap

### The `Unknown | None` Specific Concern

Your concern about 0010 and 0309 producing `Unknown | None` is the clearest signal that **WS-A (recursive closure inference) is a genuine compiler bug**, not a fixture issue. The `dfs` function in 0010's top-down approach:
```python
def dfs(i, j):
    ...
    return cache[(i, j)]  # cache: dict[tuple[int,int], bool]
```

`cache[(i,j)]` returns `bool | None` from dict indexing. But the inferred return type becomes `Unknown | None`, which means the inference engine can't even figure out `bool | None`. This is a real inference problem that would persist even if the fixture were perfectly written.

### What the Previous Conclusion Got Right

- The "no implicit coercion" principle is correct and must be preserved.
- The five subproblems (SP-1 through SP-5) are the right distinctions.
- The 62 fixture count and the representative samples are accurate.
- The **"what NOT to do"** list is correct.

### What the Previous Conclusion Missed

1. **SP-3 is NOT a narrowing failure** — it's a fixture signature failure. The fix is changing `def maxDepth(root: TreeNode)` → `def maxDepth(root: TreeNode | None)`, not a compiler narrowing enhancement.

2. **"Truthy narrowing" is incomplete for attribute access** — `while curr and curr.next:` only narrows `curr`, not `curr.next`. If there's a real narrowing gap here, it's "attribute narrowing in compound BoolOp conditions."

3. **WS-A (recursive closure inference) is a true blocker** — it makes SP-2 look worse than it is. Fix inference first, then SP-2 becomes obvious fixture work.

4. **"Container refinement" isn't a real workstream** — there's no mechanism to refine `list[T | None]` → `list[T]` via guards. The fix is unwrap-at-boundary, not element refinement.

5. **The ordering should be: WS-A first (compiler), then WS-B/WS-D/WS-E parallel, then WS-C** — not the original "narrowing → inference → container → recursive → fixture" which puts truthy narrowing (SP-3) before recognizing SP-3 is fixture-only.
