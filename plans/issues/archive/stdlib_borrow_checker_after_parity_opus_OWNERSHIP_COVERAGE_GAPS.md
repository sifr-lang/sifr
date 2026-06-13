# Ownership Coverage Gaps: `mut`/`own` Avoidance in Stdlib

Generated: 2026-02-17

## Overview

The architecture defines three parameter conventions (Contract #2):

| Convention | Syntax | Rust Codegen | Purpose |
| --- | --- | --- | --- |
| Borrow (default) | `x: list[int]` | `x: &Vec<i64>` | Read-only access |
| Mutable borrow | `mut x: list[int]` | `x: &mut Vec<i64>` | In-place mutation |
| Ownership transfer | `own x: list[int]` | `x: Vec<i64>` | Consume the value |

The stdlib uses **only** the first convention. This report analyzes why, and what it means for the ownership model's real-world coverage.

---

## Finding 1: Zero `mut` Parameters in 37 Stdlib Modules

**Search:** `grep -r "^def.*mut " lib/sifr/` — 0 results.

No stdlib function accepts a mutable borrow. This means:

- The `&mut T` codegen path for function parameters is **untested by any stdlib code**
- The borrow exclusivity checker (prevents `mut` + immutable borrow of same variable) is **untested by stdlib**
- The `MUTATING_METHODS` list in codegen (`append`, `extend`, `insert`, `clear`, `reverse`, `sort`, `pop`, `remove`) only applies to local variables, never to borrowed parameters

### Consequence: Functional Copies Instead of In-Place Mutation

The stdlib compensates by returning new collections:

| Module | Function | What It Does | What `mut` Would Allow |
| --- | --- | --- | --- |
| `heapq` | `heappush(heap, item)` | Copies entire list, appends, sifts up, returns new list | `mut heap: list[int]` — push in-place, return nothing |
| `heapq` | `heapify(data)` | Copies entire list, sifts down, returns new list | `mut data: list[int]` — heapify in-place |
| `heapq` | `_swap(data, i, j)` | Creates entirely new list with swapped elements | `mut data: list[int]` — swap two elements |
| `bisect` | `insort_left(a, x)` | Creates new list with element inserted | `mut a: list[T]` — insert in-place |
| `bisect` | `insort_right(a, x)` | Creates new list with element inserted | `mut a: list[T]` — insert in-place |

**Performance impact:** `heapq._swap` creates a full list copy for every swap. A heapify of n elements does O(n) swaps, each copying the entire list. This is O(n^2) instead of O(n).

### Why `mut` Is Avoided

The comment in `heapq.sifr` line 3 is explicit:

> Functions return new lists (functional style) to work with Sifr's borrow-by-default parameter convention.

And `itertools.sifr` line 57-58:

> references borrowed parameter inside loop condition, which conflicts with move closure in generator codegen

The stdlib authors chose functional style because:
1. It avoids the complexity of `mut` parameter codegen
2. It sidesteps potential borrow exclusivity issues
3. The `mut` codegen path may have bugs (7 Rust compile failures in borrow audit suggest this)

---

## Finding 2: Zero `own` Parameters in 37 Stdlib Modules

**Search:** `grep -r "^def.*own " lib/sifr/` — 0 results.

No stdlib function takes ownership of its arguments. This means:

- The ownership transfer codegen path is **untested by stdlib**
- The use-after-move diagnostic for `own` parameters is only tested by the audit suite, not by real code
- Builder patterns, consuming iterators, and resource-transfer APIs are not represented

### Where `own` Would Be Natural

| Module | Function | Current Behavior | Natural `own` Design |
| --- | --- | --- | --- |
| `collections` | `Counter.__init__(data)` | Borrows JSON string | `own data: str` — take ownership of initialization data |
| `heapq` | `heappush(heap, item)` | Copies heap, returns new | `own heap: list[int]` — consume old heap, return modified |
| `itertools` | `chain(a, b)` | Borrows both, copies elements | `own a: list[int], own b: list[int]` — consume both, concatenate |

---

## Finding 3: Class Mutation Workarounds

### Counter: JSON Serialization to Avoid Dict Ownership

```
class Counter:
    data: str  # JSON-encoded dict[str, int]

    def increment(self, key: str) -> None:
        self.data = counter_increment(self.data, key)
```

The `Counter` class stores its data as a JSON string rather than `dict[str, int]`. The `increment` method:
1. Passes `self.data` (a `str`, which is Move type) to the intrinsic
2. The intrinsic deserializes JSON, increments, re-serializes
3. The result is assigned back to `self.data`

This works because `str` is a simple Move type that the method receiver inference handles via `&mut self`. But it's a workaround — a `dict[str, int]` field would be more natural but would require the codegen to handle `&mut self` accessing a mutable dict field correctly.

### TopologicalSorter: Append to Self Fields

```
class TopologicalSorter:
    from_nodes: list[int]
    to_nodes: list[int]
    max_node: int

    def add(self, node: int, predecessor: int) -> None:
        self.from_nodes.append(predecessor)
        self.to_nodes.append(node)
```

This works because:
- Method receiver inference detects `self.from_nodes.append(...)` as a mutation → emits `&mut self`
- `.append()` on a `Vec` field of `&mut self` is valid Rust

This is the **only pattern** in the stdlib that exercises mutable method receivers with collection fields. It works, but it's a narrow test of the ownership model.

### Logger: Primitive Field Mutation

```
class Logger:
    _level: int

    def set_level(self, level: int) -> None:
        self._level = level
```

This works because `int` is Copy — assigning to `self._level` via `&mut self` is straightforward. This doesn't test any ownership complexity.

---

## Finding 4: For-Loop Borrow + Local Mutation Pattern

The most common pattern in the stdlib is:

```python
def some_function(data: list[int]) -> list[int]:
    result: list[int] = []       # local mutable list
    for val in data:              # borrows `data` immutably
        result.append(val)        # mutates local `result`
    return result                 # returns owned local
```

This pattern is ownership-safe because:
- `data` is borrowed (`&Vec<i64>`) — the for-loop iterates via `.iter().cloned()`
- `result` is a local owned value — `.push()` is fine
- No aliasing: `data` and `result` are different variables

**This is the only ownership pattern the stdlib exercises.** It works, but it's the simplest possible case.

---

## Finding 5: Borrow Checker Audit Regressions Affect Stdlib-Relevant Patterns

The borrow audit (`audits/borrowing/REPORT.md`) shows 7 Rust compile failures. Several affect patterns the stdlib would use:

| Test | Pattern | Stdlib Relevance |
| --- | --- | --- |
| `13_string_method_borrow` | `&String` vs `String` comparison | Any stdlib function comparing borrowed strings |
| `14_reassignment_resets_move` | Reassignment after borrow | Stdlib functions that reassign variables |
| `19_class_instance_move` | Class field destructuring | Stdlib classes with multiple fields |
| `22_class_method_mut_self` | `&mut self` type mismatch | Any stdlib class with mutating methods |
| `30_multiple_function_calls_same_var` | Same var in multiple calls | Common stdlib pattern |
| `33_move_in_both_branches` | Borrow in conditional | Stdlib functions with branching |
| `42_chained_string_methods` | Chained method calls | String processing in stdlib |

These bugs explain why the stdlib avoids these patterns — they would trigger codegen failures.

---

## Coverage Gap Summary

| Ownership Feature | Architecture Defines | Stdlib Tests | Borrow Audit Tests | Gap |
| --- | --- | --- | --- | --- |
| `&T` function params | Yes | Yes (all functions) | Yes (29 pass) | None |
| `&mut T` function params | Yes | **No** | Yes (some fail) | **Complete** |
| `T` (own) function params | Yes | **No** | Yes (some fail) | **Complete** |
| Copy type pass-by-value | Yes | Yes | Yes | None |
| Move-on-assignment | Yes | Implicit | Yes | None |
| `&self` method receiver | Yes | Yes (read-only methods) | Yes | None |
| `&mut self` method receiver | Yes | Yes (3 classes) | Yes (1 fails) | **Partial** |
| `self` (consuming) receiver | Yes | **No** | Not tested | **Complete** |
| Borrow exclusivity (same call) | Yes | **No** | Not tested | **Complete** |
| Use-after-move diagnostic | Yes | **No** (never triggers) | Yes (12 correct) | Stdlib gap |
| Closure capture inference | Deferred | N/A | N/A | N/A |
| Escape analysis | Yes | **No** | Not tested | **Complete** |

The stdlib exercises approximately **30%** of the ownership model's designed surface area.
