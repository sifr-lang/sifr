# `wave_psp_b1` CPython Traceability

## Reviewed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_collections.py` | `Counter.most_common([n])`, `Counter.get(key[, default])`, dict-backed constructor, and `deque` rotate/count/remove/copy/reverse behavior | `crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr` | adapted | `Counter` remains a typed class rather than a `dict` subclass, but the closed object-model surface now follows the CPython entry points more closely. |
| `Lib/test/test_collections.py` | `defaultdict(factory[, mapping])` auto-initialization for `int`/`list`/`set` factories | `crates/sifr/tests/e2e/pass/phase31_defaultdict_len_deque_compat.sifr` | adapted | Implemented as a typed compiler-lowered compat surface (`defaultdict(...)` call + index auto-init), not a full runtime class object model. |
| `Lib/test/test_collections.py` | invalid deque index bound typing | `crates/sifr/tests/e2e/fail/phase_psp_b1_deque_index_invalid_bound.sifr` | adapted | Type mismatches are rejected at compile time instead of raising at runtime. |
| `Lib/test/test_collections.py` | `deque.index` missing-value behavior | `crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr` | adapted | Returns `None` for not-found lookups instead of CPython's `ValueError`, preserving panic-free typed behavior. |
| `Lib/test/test_bisect.py` | `bisect`/`bisect_left`/`insort` optional `lo`/`hi` forms and aliases | `crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr` | adapted | `lo < 0` and oversized `hi` clamp safely instead of raising `ValueError`. |
| `Lib/test/test_bisect.py` | unsupported `key=` call shape | `crates/sifr/tests/e2e/fail/phase_psp_b1_bisect_key_unsupported.sifr` | waived | The current callable/type-overload surface does not support CPython’s `key=` bisect contract without a broader signature model change. |
| `Lib/test/test_heapq.py` | mutating `heappushpop`, `heapreplace`, and shipped max-heap helpers | `crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr` | adapted | Empty replacement remains panic-free via `None`, and the shipped underscore max-heap helpers keep the current compatibility surface. |
| `Lib/test/test_heapq.py` | `merge(*iterables)` merge ordering semantics | `crates/sifr/tests/e2e/pass/cpython_heapq.sifr`, `crates/sifr/tests/e2e/pass/cpython_heapq_subset.sifr` | adapted | Vararg merge is wired for `sifr.heapq.merge(...)` and now has explicit CPython-derived regression coverage. |

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| `Counter(iterable)` and `Counter(**kwargs)` constructor forms from `Lib/test/test_collections.py` | `unsupported` | Generic class-constructor overloading for iterable-vs-mapping-vs-keyword shapes is not yet available; `Counter(dict)` and `from_list(...)` remain the closed typed entry points in this wave. |
| `collections.defaultdict(..., default_factory=..., **kwargs)` keyword constructor variants and class-attribute surfaces (`default_factory`, class methods, etc.) | `intentional-diff` | The closed b1 surface intentionally uses a typed compiler-lowered compat form for default-producing index access; full CPython class object-model parity is not claimed for this wave. |
| non-exported `heapq` private helpers `_heappush_max` / `_heappushpop_max` | `unsupported` | These private helpers are intentionally not part of the shipped compatibility surface in this wave; supported max-heap helpers remain `_heapify_max`, `_heappop_max`, and `_heapreplace_max`. |

## Waiver enforcement fixtures

- `Counter(iterable)` rejection: `crates/sifr/tests/e2e/fail/phase_psp_b1_counter_iterable_constructor_unsupported.sifr`
- `Counter(**kwargs)` rejection: `crates/sifr/tests/e2e/fail/phase_psp_b1_counter_kwargs_constructor_unsupported.sifr`
