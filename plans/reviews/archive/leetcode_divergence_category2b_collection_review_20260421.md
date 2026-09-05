# Category 2b Review: Collection / Index / Stdlib Ergonomics

**Reviewer:** agent (category review)
**Date:** 2026-04-21
**Fixtures reviewed:** 20 fixtures listed in Section 2b of `leetcode_divergence_decision_analysis_20260409.md`

---

## Summary Assessment

The Category 2b classification is **largely correct** — all 20 fixtures exhibit the stated ergonomics pattern (verbose dead guards on proven collection/index access, missing stdlib primitives). However, 3 fixtures are borderline (discussed below), and 2 fixtures reveal a subtle risk: the Sifr version may use an intentionally inferior algorithm *because* the enabling stdlib is absent, which is a stronger ergonomics pressure than the category description makes explicit.

**Proposed category boundary clarification:** Section 2b should note that stdlib parity gaps (especially `heap`) can produce *algorithm-level* divergence, not just syntactic verbosity. When a fixture substitutes O(n²) exhaustive search for the canonical Dijkstra/Prim heap approach, the repair is stdlib parity, not just collection ergonomics.

---

## Fixture-by-Fixture Analysis

### 0130_surrounded_regions — **Correctly in 2b**

Sifr and Python versions share the same border-detection DFS algorithm. Sifr has no significant index/collection guard overhead — the divergence is minimal and algorithmic parity is close. The remaining gap is string/index access pattern verbosity. Classification holds.

### 0150_evaluate_reverse_polish_notation — **Borderline / should be Category 3**

The `pop()` fallback guard in `popInt` (`if value is None: return 0`) is not a collection/index ergonomics issue — it handles the *expected* empty-stack condition, not a proven non-Optional index access. The algorithm (stack-based evaluation) maps cleanly to Sifr without structural divergence. The diff appears driven by string parsing verbosity (`digitValue`, `parseIntToken`) and a manual integer sign parser that Python handles with builtins. **Recommendation:** Move to Category 3 (okay as-is). The Sifr solution is a legitimate, complete port.

### 0261_graph_valid_tree — **Correctly in 2b**

Both versions implement the same DSU-based cycle-check + component-count algorithm. The Sifr version has verbose list-index access with `None` guards on `parents[node]` and `ranks[rootA]`. This is the canonical 2b pattern: proven non-Optional list access requiring dead guard boilerplate. The DSU list ergonomics are explicitly listed in the category's "what should improve" (DSU / union-find helpers). Classification holds.

### 0269_alien_dictionary — **Correctly in 2b**

Python uses `dict[char, set[char]]` (adjacency) and `{char: bool}` visited dict. Sifr replaces both with 2D boolean array + integer indegree array — a structural adaptation, not just syntactic noise. The missing `dict-of-set` stdlib primitive is a material ergonomics gap that forces this structural change. Classification holds.

### 0286_walls_and_gates — **Correctly in 2b**

Both versions use the same BFS-from-gates algorithm. Sifr's row-wise BFS has verbose `None` guards on tuple unpacking and grid access. The `deque` stdlib gap forces a manual queue with manual head-index increment, which contributes significantly to diff. The "what should improve" explicitly mentions `deque`. Classification holds.

### 0355_design_twitter — **Correctly in 2b**

Python uses `defaultdict(list)` for both `tweetMap` and `followMap`. Sifr replaces this with manual `dict.get(key, [])` calls and verbose unwrap helpers. The `defaultdict`-like ergonomics gap is the core pressure. Classification holds.

### 0394_decode_string — **Correctly in 2b**

Both versions use the same stack-based decode algorithm. Sifr has `None` guards on `stack.pop()` and `stack[len(stack)-1]`. The canonical 2b pattern: proven non-Optional stack access still requires dead guards because Sifr's `list[T].pop()` returns `T | None`. The diff is driven by this ergonomics gap plus manual `isDigit` / `digitValue` functions that Python handles with `char.isdigit()`. Classification holds.

### 0417_pacific_atlantic_water_flow — **Borderline / re-evaluate after 1584/1631**

Both versions use the same multi-source BFS-from-borders algorithm. Sifr has verbose tuple-unpacking guards (`if r0 is None or c0 is None or prev0 is None`) in the flood loop. The algorithm is clean and parity is good. The diff is dominated by the same collection/index guard pattern. **Recommendation:** Keep in 2b but flag that this fixture's ergonomic pressure is identical to 2a (tuple/Optional guard patterns). If tuple narrowing improves under 2a, this fixture improves automatically.

### 0567_permutation_in_string — **Correctly in 2b**

Both versions use the identical sliding window O(n) algorithm. The `defaultdict`-like dict gap (Python uses `collections.Counter` / direct indexing) forces verbose `valueAt`/`addAt` helpers with manual bounds and `None` checks. The array-of-26-int counting array pattern is identical between Python and Sifr — the diff is purely ergonomics. Classification holds.

### 0721_accounts_merge — **Correctly in 2b**

Both versions use the same UnionFind DSU + email-grouping algorithm. Python's `defaultdict(list)` for email grouping is replaced by Sifr's manual `dict.get(email, [])` pattern. The DSU list ergonomics (verbose `unwrapInt` on `parent[x]`, `size[x]`) are the canonical 2b pattern. Classification holds.

### 0743_network_delay_time — **Correctly in 2b** (but note algorithmic risk)

Both versions implement Dijkstra's algorithm. Sifr correctly imports `sifr.heapq`. The encoding trick (multiplying `node_base` to pack distance+node into a single int for Rust's `heappush`) is a workaround for missing `heapq`-style tuple ordering. The `from sifr.heapq import heappush, heappop` import is the category's explicit "what should improve" for heap stdlib. Classification holds.

### 0752_open_the_lock — **Should be Category 3**

Both versions use identical BFS over lock state space. Sifr has `None` guards on `state` from `q[head]` (which is `tuple[str, int] | None` — the queue can contain `None` entries if a previous pop left a gap). The guard is necessary but mechanical. The algorithmic divergence is minimal. **Recommendation:** Move to Category 3 — the Sifr version is a clean, parity-faithful port with only incidental verbosity from `None` guards on tuple extraction.

### 0778_swim_in_rising_water — **Correctly in 2b** (but note algorithmic divergence)

Both versions target the same problem. **Python uses Dijkstra's algorithm (O(n² log n) with heap). Sifr uses exhaustive O(n²) linear scan.** This is not merely ergonomic friction — it is a different algorithm chosen because the heap stdlib is absent. The category description mentions `heap` as a stdlib unblock, which applies here. The `getIntCell`/`setIntCell` guard pattern is the 2b ergonomic fingerprint, but the algorithm substitution is a stronger signal. Classification holds, but the analysis doc should explicitly note this fixture as a heap stdlib dependency case.

### 1203_sort_items_by_groups_respecting_dependencies — **Correctly in 2b**

Both versions use the same topological sort (Kahn's algorithm). Sifr has verbose `getBucket`/`appendEdge` helpers for adjacency list access, and `unwrapInt` guards on list indexing. The `deque` stdlib gap forces manual queue with head-index increment. Classification holds.

### 1397_find_all_good_strings — **Correctly in 2b**

Both versions use the same DP + KMP LPS matching for the evil string. Sifr has verbose string index access (`charAt` with `None` guard), manual 26-char alpha iteration, and `dict[int, int]` memo with verbose access patterns. The string/index ergonomics gap (Python's direct `s[i]` access without `None` concern) is the core pressure. Classification holds.

### 1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree — **Correctly in 2b**

Both versions use Kruskal's algorithm with manual DSU. Sifr has `unwrapInt` guards on `par[cur]`, `rank[p1]` etc. The DSU list ergonomics gap is explicitly the category's domain. Classification holds.

### 1584_min_cost_to_connect_all_points — **Correctly in 2b** (with algorithmic note)

**Python uses Prim's algorithm with `heapq` (O(n² log n)). Sifr uses O(n²) manual linear scan** to find the minimum-distance unvisited node. This is the same pattern as 0778: absent `heap` stdlib causes an algorithm substitution. The `coord()` helper and `getIntAt`/`setIntAt` guards on the 2D point array are the 2b ergonomic fingerprint, but the heap absence is the root cause. Classification holds.

### 1631_path_with_minimum_effort — **Correctly in 2b** (with algorithmic note)

**Python uses Dijkstra with `heapq`. Sifr uses O(mn²) exhaustive linear scan.** This is the strongest case for heap stdlib as a Category 2b unblock. The `getIntCell`/`setIntCell`/`getBoolCell`/`setBoolCell` guard pattern on 2D arrays appears identically in 0778, 1584, and 1631 — all four fixtures share the same ergonomic debt and the same heap stdlib gap as root cause. Classification holds, and this fixture should be explicitly flagged as the canonical Dijkstra heap unblock target.

### 2092_find_all_people_with_secret — **Correctly in 2b**

Both versions use the same per-time-step BFS over meeting graph. Sifr has verbose `None` guards on tuple extraction from adjacency dict values and on node IDs. The `dict[int, list[tuple[int, int]]]` adjacency structure with verbose unwrap is the 2b pattern. Classification holds.

### 2709_greatest_common_divisor_traversal — **Correctly in 2b**

Both versions use the same DSU + factor-index algorithm. Sifr has `unwrapInt` guards on `par[cur]` in the DSU find operation. This is identical to 0261 and 1489 — the DSU list ergonomics gap is the explicit 2b target. Classification holds.

---

## Proposed Category 2b Boundary Refinements

### Misclassifications (2 fixtures)

| Fixture | Current | Recommended | Reason |
|---------|---------|-------------|--------|
| 0150_evaluate_reverse_polish_notation | 2b | Category 3 | `pop()` guard handles expected empty-stack case, not proven non-Optional access; algorithm parity is high |
| 0752_open_the_lock | 2b | Category 3 | Minimal divergence; clean BFS port with only mechanical `None` guards on tuple extraction |

### Algorithmic divergence signal (4 fixtures)

These fixtures (0778, 1584, 1631, and also 0743 to a lesser extent) exhibit more than ergonomic friction — they substitute the canonical algorithm because the enabling stdlib is absent:

- **0778_swim_in_rising_water:** Python Dijkstra with heap → Sifr O(n²) linear scan
- **1584_min_cost_to_connect_all_points:** Python Prim's with heap → Sifr O(n²) linear scan
- **1631_path_with_minimum_effort:** Python Dijkstra with heap → Sifr O(mn²) linear scan
- **0743_network_delay_time:** Uses heap correctly but with encoding workaround

**Recommendation:** These 4 fixtures should be flagged in the analysis as "heap stdlib dependency cases" within Category 2b. The ergonomic improvement from adding `heap` stdlib will also restore the canonical algorithm — this is the strongest argument for heap stdlib priority.

---

## Boundary Preservation Check

Do the proposed improvements for Category 2b preserve Sifr principles?

| Proposed improvement | Preserves principles? | Risk |
|---|---|---|
| Preserve proven non-Optional values across flow (no dead guards on `list[idx]` after bounds check) | Yes — this is a compiler narrowing improvement, not a weakening | None |
| Safer owned collection helpers with minimal cloning | Yes — ownership-correct helpers are a net win | Must not add `Clone` bound implicitly |
| `heap` stdlib parity | Yes — deterministic priority queue is ownership-safe | None |
| `deque` stdlib parity | Yes — owned deque with push/pop is ownership-safe | None |
| DSU / union-find helpers | Yes — ownership-correct DSU is straightforward | Must not emulate Python's in-place mutation aliasing |
| `defaultdict`-like dict ergonomics | Yes — a `get_or_insert` helper is ownership-safe | Must not add truthiness coercion |

**No boundary violations identified.** None of the proposed improvements imply Python-style dynamic behavior, implicit nullable access, or weakened ownership semantics. All improvements are additive and type-safe.

---

## Recommendations

1. **Move 0150 and 0752 to Category 3** (okay as-is). Both are clean, parity-faithful ports with only incidental mechanical verbosity.

2. **Add a "heap stdlib dependency" sub-label within Category 2b** covering 0778, 1584, 1631, and 0743. This makes the unblock priority concrete: adding `heap` stdlib simultaneously resolves 4 fixtures' ergonomic debt AND restores their canonical algorithms.

3. **Update Category 2b description** to note that stdlib gaps (particularly `heap`) can produce algorithm-level divergence, not just syntactic verbosity. This distinction matters for prioritization: fixing `heap` is higher-leverage than the collection/index guard improvements.

4. **No changes to Sifr principles are required** by any proposed Category 2b improvements.
