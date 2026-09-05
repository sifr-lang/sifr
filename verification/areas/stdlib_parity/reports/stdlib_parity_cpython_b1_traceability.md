# `stdlib_parity_b1` CPython Traceability

## Validationed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_collections.py` | `Counter.most_common([n])`, `Counter.get(key[, default])`, dict-backed constructor, and `deque` rotate/count/remove/copy/reverse behavior | `crates/sifr/tests/e2e/pass/ordered_collections.sifr` | adapted | `Counter` remains a typed class rather than a `dict` subclass, but the closed object-model surface now follows the CPython entry points more closely. |
| `Lib/test/test_collections.py` | `defaultdict(factory[, mapping])` auto-initialization for `int`/`list`/`set` factories | `crates/sifr/tests/e2e/pass/defaultdict_len_and_deque.sifr` | adapted | Implemented as a typed compiler-lowered compat surface (`defaultdict(...)` call + index auto-init), not a full runtime class object model. |
| `Lib/test/test_collections.py` | invalid deque index bound typing | `crates/sifr/tests/e2e/fail/deque_index_invalid_bound.sifr` | adapted | Type mismatches are rejected at compile time instead of raising at runtime. |
| `Lib/test/test_collections.py` | `deque.index` missing-value behavior | `crates/sifr/tests/e2e/pass/ordered_collections.sifr` | adapted | Returns `None` for not-found lookups instead of CPython's `ValueError`, preserving panic-free typed behavior. |
| `Lib/test/test_bisect.py` | `bisect`/`bisect_left`/`insort` optional `lo`/`hi` forms and aliases | `crates/sifr/tests/e2e/pass/ordered_collections.sifr` | adapted | `lo < 0` and oversized `hi` clamp safely instead of raising `ValueError`. |
| `Lib/test/test_bisect.py` | unsupported `key=` call shape | `crates/sifr/tests/e2e/fail/bisect_key_unsupported.sifr` | waived | The current callable/type-overload surface does not support CPython’s `key=` bisect rules without a broader signature model change. |
| `Lib/test/test_heapq.py` | mutating `heappushpop`, `heapreplace`, and shipped max-heap helpers | `crates/sifr/tests/e2e/pass/ordered_collections.sifr` | adapted | Empty replacement remains panic-free via `None`, and the shipped underscore max-heap helpers keep the current compatibility surface. |
| `Lib/test/test_heapq.py` | `merge(*iterables)` merge ordering semantics | `crates/sifr/tests/e2e/pass/cpython_heapq.sifr`, `crates/sifr/tests/e2e/pass/cpython_heapq_subset.sifr` | adapted | Vararg merge is wired for `sifr.heapq.merge(...)` and now has explicit CPython-derived regression coverage. |

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| `Counter(**kwargs)` constructor form from `Lib/test/test_collections.py` | `unsupported` | `stdlib_parity_struct_2` closed mapping + keyword-iterable constructor parity for the bounded typed surface, but dynamic kwargs constructor overloading remains intentionally out of scope. |
| `collections.defaultdict(..., default_factory=..., **kwargs)` dynamic keyword/class ecosystem | `intentional-diff` | `stdlib_parity_struct_2` promotes `defaultdict` to an explicit typed class (`default_factory: int`, typed methods), while dynamic keyword and broad class ecosystem parity remain intentionally bounded. |
| non-exported `heapq` private helpers `_heappush_max` / `_heappushpop_max` | `unsupported` | These private helpers are intentionally not part of the shipped compatibility surface in this implementation pass; supported max-heap helpers remain `_heapify_max`, `_heappop_max`, and `_heapreplace_max`. |

## Waiver enforcement fixtures

- `Counter(positional-iterable)` rejection: `crates/sifr/tests/e2e/fail/counter_iterable_constructor_unsupported.sifr`
- `Counter(**kwargs)` rejection: `crates/sifr/tests/e2e/fail/ordered_counter_kwargs_constructor_unsupported.sifr`

## Structured/Class-Surface Continuation Readiness (2026-03-18)

- Continuation capability: `structured-data-and-class-surface-parity-expansion record`
- Capability readiness: `stdlib_parity_struct_2` broadens constructor/object-model parity for `collections`.
- Closed in continuation:
  - `Counter(iterable=...)` typed constructor form alongside mapping input.
  - explicit typed `defaultdict` class surface with deterministic missing-key initialization (`ensure`).
  - CPython-derived continuation fixture: `crates/sifr/tests/e2e/pass/counter_defaultdict_and_argparse.sifr`
- Locked permanent diff carried into continuation:
  - `Counter(**kwargs)` remains explicitly `unsupported` until generic constructor-overload support is approved.
- Enforcement fixture: `crates/sifr/tests/e2e/fail/counter_kwargs_constructor_unsupported.sifr`
