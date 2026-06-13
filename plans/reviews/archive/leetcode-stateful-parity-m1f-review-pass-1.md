# Code Review: leetcode-stateful-parity-m1f (Pass 1)

## Scope

Stateful parity wave M1f addresses 4 LeetCode problems:
- `1472_design_browser_history` (Linked List)
- `0380_insert_delete_getrandom_o1` (Arrays & Hashing)
- `1396_design_underground_system` (Arrays & Hashing)
- `0146_lru_cache` (Linked List)

## Changes Reviewed

```
benchmarks/problems/arrays_and_hashing.json |  9 ++++----
benchmarks/problems/linked_list.json        |  7 ++++---
benchmarks/slowness_seed.py                 |  8 ++++----
src/0380_insert_delete_getrandom_o1.sifr    | 13 +++++++-----
src/1396_design_underground_system.sifr     | 32 +++++++++--------------------
src/1472_design_browser_history.sifr        |  7 +++----
6 files changed, 34 insertions(+), 42 deletions(-)
```

## Code Correctness Review

### 1472_design_browser_history.sifr

**Before:** Truncated forward history via `while len(self.history) > self.i + 1: self.history.pop()` followed by append — a pop-loop that doesn't match Python's direct overwrite semantics.

**After:** Direct index assignment `history[self.i + 1] = str(url)` with local alias for ownership. Matches Python's `self.history[self.i + 1] = url` pattern exactly.

**Parity assessment:** Correct. Direct index overwrite matches Python logical behavior. Removed unused `list_node` import. `back()` and `forward()` remain identical to Python.

### 0380_insert_delete_getrandom_o1.sifr

**Before:** `choice` iterated the full list; `lastValue` scanned for the final element.

**After:** `choice` returns `values[0]` (indexed access); `lastValue` returns `values[len(values) - 1]` with empty-list guard. Both now match Python's `values[0]` and `values[-1]` pattern.

**Parity assessment:** Correct. Indexed access matches Python's list indexing semantics. The `None` guard on `values[0]` is sound (Sifr's `list[int]` is `Vec<i64>` in Rust, but the `int | None` annotation handles the foreign-language boundary safely).

### 1396_design_underground_system.sifr

**Before:** Hardcoded station codes (1-4 for Leyton/Paradise/Waterloo/Cambridge) with integer route keys via `startCode * 1000 + endCode`. Only worked for known stations.

**After:** Generic string station names stored directly; route key uses length-prefixed string format: `str(len(startStation)) + "#" + startStation + "#" + endStation`.

**Parity assessment:** Correct. The Python version uses `(start, stationName)` tuple as dict key — tuple keys are unique by construction. The Sifr string route key is collision-resistant for realistic station names because:
- The length prefix prevents cross-station collisions where `len(s1) != len(s2)`.
- The `#` delimiter is not present in any test station name.
- Verified no collisions exist for test fixtures.

`customerStation` stores `str` (matching Python's `stationName` string), not station codes. All three methods (`checkIn`, `checkOut`, `getAverageTime`) now use the generic `_routeKey` consistently.

### 0146_lru_cache.sifr

**No code changes.** The existing integer-node LRU implementation is marked with `lru_parity` tag and `parity_status: "equivalent"`. The Python version uses doubly-linked pointer surgery; the Sifr version uses integer-node dicts with `detach`/`insertAfter`/`moveToFront` methods that preserve the same LRU semantics.

**Parity assessment:** Correct. Both implementations maintain head/tail sentinel nodes, track `prev`/`next` as doubly-linked structure, and evict the LRU node (node adjacent to tail) when capacity is exceeded. The integer-node approach is an ownership-safe equivalent of the Python doubly-linked list.

## Metadata Review

### slowness_seed.py (authoritative source)

| Problem | Parity | Owner | Tags |
|---|---|---|---|
| `1472_design_browser_history` | `equivalent` | `mixed` | `stateful_object, field_clone, list_clone` |
| `0380_insert_delete_getrandom_o1` | `equivalent` | `mixed` | `stateful_object, field_clone, array_map_parity` |
| `1396_design_underground_system` | `equivalent` | `mixed` | `stateful_object, field_clone, string_key` |
| `0146_lru_cache` | `equivalent` | `mixed` | `stateful_object, field_clone, lru_parity` |

### JSON files (arrays_and_hashing.json, linked_list.json)

Checked via `analyze_slowness.py` output — all four problems show `equivalent` parity with `mixed` ownership and matching slowness tags. Consistent with slowness_seed.py.

## Analyzer Output Verification

Ran `python3 benchmarks/analyze_slowness.py` and confirmed all four problems appear in the measured-slower table with:
- `parity: equivalent`
- `owner: mixed`
- Corresponding slowness tags

All four are correctly excluded from the no-pair failures appendix.

## Validation Gaps

None identified. The changes are confined to:
1. Benchmark metadata (no executable content)
2. Sifr source files with direct demos already passed
3. Python seed files that compile cleanly

## Findings

No correctness bugs, parity mistakes, metadata misclassifications, or phase-scope problems found.

## Recommendation

**APPROVED — No blockers.**

The four stateful parity rows are correctly implemented and metadata-consistently marked as `equivalent` with appropriate slowness tags. Residual `mixed` classification is appropriate given that these problems still show Sifr slower than Python at some benchmark sizes due to field-clone and container-clone codegen overhead — those are M2/M3 compiler work items, not M1 parity issues.