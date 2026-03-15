# `wave_psp_b1` CPython Traceability

## Reviewed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_collections.py` | `Counter.most_common([n])`, dict-backed constructor, and `deque` rotate/count/remove/copy/reverse behavior | `crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr` | adapted | `Counter` remains a typed class rather than a `dict` subclass, but the closed object-model surface now follows the CPython entry points more closely. |
| `Lib/test/test_collections.py` | invalid deque index bound typing | `crates/sifr/tests/e2e/fail/phase_psp_b1_deque_index_invalid_bound.sifr` | adapted | Type mismatches are rejected at compile time instead of raising at runtime. |
| `Lib/test/test_bisect.py` | `bisect`/`bisect_left`/`insort` optional `lo`/`hi` forms and aliases | `crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr` | adapted | `lo < 0` and oversized `hi` clamp safely instead of raising `ValueError`. |
| `Lib/test/test_bisect.py` | unsupported `key=` call shape | `crates/sifr/tests/e2e/fail/phase_psp_b1_bisect_key_unsupported.sifr` | waived | The current callable/type-overload surface does not support CPython’s `key=` bisect contract without a broader signature model change. |
| `Lib/test/test_heapq.py` | mutating `heappushpop`, `heapreplace`, and shipped max-heap helpers | `crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr` | adapted | Empty replacement remains panic-free via `None`, and the shipped underscore max-heap helpers keep the current compatibility surface. |

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| `Counter(iterable)` and `Counter(**kwargs)` constructor forms from `Lib/test/test_collections.py` | `unsupported` | Generic class-constructor overloading for iterable-vs-mapping-vs-keyword shapes is not yet available; `Counter(dict)` and `from_list(...)` remain the closed typed entry points in this wave. |
| `collections.defaultdict(..., default_factory=..., **kwargs)` keyword constructor variants | `unsupported` | The compat `defaultdict` lowering still accepts only positional factory/mapping forms in this wave. |
| `heapq.merge(*iterables)` and the non-exported `_heappush_max` / `_heappushpop_max` family | `unsupported` | Imported vararg metadata and the remaining max-helper export surface are not fully wired through the current module-import path in this wave. |
