# Recommendations: Prioritized Fix Plan

Generated: 2026-02-17

## Guiding Principle

The architecture's core promise is **"if it compiles, it works."** The most impactful fixes are those that close the gap between this promise and reality. Ownership model coverage is secondary — the safety contract violations are more urgent because they cause runtime crashes in shipped code.

---

## Priority 1: Close the Safety Contract (Critical)

### 1A. Add `Result` return types to I/O intrinsics

**Effort:** Medium (type signatures + codegen changes + stdlib adaptation)
**Impact:** Eliminates 10 panic paths across 6+ modules

**Steps:**
1. Change intrinsic type signatures in `crates/sifr_hir/src/stdlib.rs` to return `Result[T, IOError]`:
   - `read_text(path: str) -> Result[str, IOError]`
   - `write_text(path: str, content: str) -> Result[None, IOError]`
   - `read_lines(path: str) -> Result[list[str], IOError]`
   - Same for `mkdir`, `rmdir`, `remove_file`, `rename`, `listdir`, `getcwd`, `append_text`
2. Update codegen to emit `Result::Ok(...)` / `Result::Err(...)` instead of `.unwrap()`
3. Update stdlib modules (`io.sifr`, `pathlib.sifr`, `os.sifr`, `shutil.sifr`, `tempfile.sifr`, `tomllib.sifr`) to propagate or handle `Result`
4. Update E2E tests to use `?` or `match` on I/O results

**Prerequisite:** The `Result` type and `?` operator must work for user-defined error types. Verify this works end-to-end before starting.

### 1B. Fix `list.remove()`, `list.index()`, `set.pop()` to return `Option`/`Result`

**Effort:** Low (codegen-only changes)
**Impact:** Eliminates 3 panic paths

**Steps:**
1. `list.index(val)` → return `Option[int]` (codegen: `.iter().position(...).map(|p| p as i64)`)
2. `list.remove(val)` → return `Result[None, ValueError]` or `bool` (codegen: use `position` + check)
3. `set.pop()` → return `Option[T]` (codegen: `.iter().next().cloned()` without `.unwrap()`)

### 1C. Fix `min()`/`max()` to return `Option`/`Result` on empty input

**Effort:** Low (codegen-only changes)
**Impact:** Eliminates 4-5 panic paths

**Steps:**
1. `min(list)` → return `Option[T]` when list might be empty
2. `max(list)` → return `Option[T]` when list might be empty
3. `sorted()` with floats → handle NaN comparison safely (use `total_cmp` instead of `partial_cmp().unwrap()`)

### 1D. Make `SubscriptAssign` safe

**Effort:** Medium (codegen + possibly type system changes)
**Impact:** Eliminates 1 panic path, closes asymmetry in safe indexing contract

**Options:**
- **Option A (bounds check):** `list[i] = val` → `if (i as usize) < list.len() { list[i as usize] = val; }` — silently ignores out-of-bounds
- **Option B (Result):** `list[i] = val` returns `Result[None, IndexError]` — requires changing assignment semantics
- **Option C (grow-on-assign):** Auto-extend list if index is out of bounds — diverges from both Python and Rust
- **Recommended: Option A** with a compiler warning, matching the "safe indexing" philosophy of returning None/no-op rather than panicking

---

## Priority 2: Fix Sentinel Returns (Medium)

### 2A. Statistics functions should return `Result`

**Effort:** Low (pure Sifr changes only)
**Impact:** 8 functions in `sifr.statistics` return correct error types

**Steps:**
1. Change return types: `mean(data: list[float]) -> Result[float, StatisticsError]`
2. Return `Err(StatisticsError("no data"))` for empty input instead of `0.0`
3. This requires the `Result` type to work with user-defined error classes

**Alternative (if Result not ready):** Change return type to `float | None` (Option) and return `None` for empty data. Less correct but better than sentinel `0.0`.

### 2B. `heappop()` should return `Option[int]`

**Effort:** Low (pure Sifr change)
**Impact:** 1 function in `sifr.heapq`

**Steps:**
1. Change `heappop(heap: list[int]) -> int` to `heappop(heap: list[int]) -> int | None`
2. Return `None` for empty heap instead of `0`

---

## Priority 3: Exercise `mut` and `own` in Stdlib (Medium)

### 3A. Convert `heapq` to use `mut` parameters

**Effort:** Medium (requires fixing `mut` codegen bugs first)
**Impact:** Proves `mut` parameter path works; improves heapq from O(n^2) to O(n)

**Steps:**
1. Fix the 7 Rust compile failures in the borrow audit (especially `&String` vs `String` and `&mut self` mismatches)
2. Convert `heapq` functions to use `mut`:
   ```python
   def heappush(mut heap: list[int], item: int) -> None:
       heap.append(item)
       # sift up in-place
   ```
3. Convert `_swap` to use `mut` instead of copying
4. Add E2E tests for `mut` parameter stdlib functions

### 3B. Convert `bisect.insort_*` to use `mut` parameters

**Effort:** Low (after 3A proves the path works)
**Impact:** Proves `mut` works for generic functions

### 3C. Add at least one `own` parameter stdlib function

**Effort:** Low
**Impact:** Proves `own` path works in real code

**Candidate:** `itertools.chain(own a: list[int], own b: list[int]) -> list[int]` — consumes both lists and concatenates. This is a natural ownership transfer.

---

## Priority 4: Fix Generator + Borrow Interaction (Medium-High)

### 4A. Generator codegen should handle borrowed parameters in loop conditions

**Effort:** High (generator codegen is complex)
**Impact:** Unblocks lazy `itertools` functions

**Steps:**
1. Analyze why `len(data)` in a `while` condition conflicts with generator closure capture
2. The issue is likely that the generator state machine captures `data` by move, but `data` is a borrowed `&Vec` — the closure needs to capture the reference, not move it
3. Fix the generator codegen to capture borrowed parameters by reference in the state machine struct
4. Convert eager `itertools` functions to lazy generators

---

## Priority 5: Improve Counter to Use Native Dict (Low)

### 5A. Replace JSON string with `dict[str, int]` field

**Effort:** Medium (requires verifying `&mut self` + dict field codegen)
**Impact:** Better performance, proves dict-field mutation works

**Steps:**
1. Verify that a class with a `dict[str, int]` field can have methods that call `.insert()` / `.get()` via `&mut self`
2. Replace `Counter.data: str` with `Counter.data: dict[str, int]`
3. Replace intrinsic calls with direct dict operations
4. Remove JSON serialization overhead

---

## Implementation Order

```
Phase 1 (Safety — blocks "if it compiles, it works" guarantee):
  1A → 1B → 1C → 1D → 2A → 2B

Phase 2 (Ownership coverage — proves the model works):
  Fix borrow audit regressions (7 Rust compile failures)
  → 3A → 3B → 3C

Phase 3 (Advanced patterns):
  4A → 5A
```

## Effort Estimates

| Item | Effort | Files Changed | Risk |
| --- | --- | --- | --- |
| 1A (I/O Result types) | 3-5 days | stdlib.rs, lib.rs, 6 .sifr files, E2E tests | Medium — touches type signatures |
| 1B (list/set methods) | 1 day | lib.rs | Low — codegen only |
| 1C (min/max) | 1 day | lib.rs | Low — codegen only |
| 1D (subscript assign) | 1-2 days | lib.rs, possibly lower.rs | Medium — semantic question |
| 2A (statistics Result) | 1 day | statistics.sifr | Low — pure Sifr |
| 2B (heappop Option) | 0.5 day | heapq.sifr | Low — pure Sifr |
| 3A (heapq mut) | 2-3 days | heapq.sifr, fix codegen bugs | Medium — depends on bug fixes |
| 3B (bisect mut) | 0.5 day | bisect.sifr | Low — after 3A |
| 3C (own parameter) | 0.5 day | itertools.sifr | Low |
| 4A (generator + borrow) | 3-5 days | lib.rs generator codegen | High — complex |
| 5A (Counter dict) | 1-2 days | collections.sifr, stdlib.rs | Medium |
| **Total** | **~15-22 days** | | |

## Success Criteria

After completing Phase 1:
- Zero `.unwrap()` calls in intrinsic codegen for user-facing operations
- All fallible stdlib operations return `Result` or `Option`
- The borrow audit passes with 0 Rust compile failures
- A sifr program that handles all `Result`/`Option` values cannot crash at runtime

After completing Phase 2:
- At least 3 stdlib functions use `mut` parameters
- At least 1 stdlib function uses `own` parameters
- The heapq module has O(n) heapify instead of O(n^2)

After completing Phase 3:
- `itertools` functions can be lazy generators even when referencing borrowed parameters
- `Counter` uses native dict instead of JSON serialization
