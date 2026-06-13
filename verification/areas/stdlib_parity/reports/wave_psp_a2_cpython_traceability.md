# `wave_psp_a2` CPython Traceability

## Reviewed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_list.py` | `list.pop(index)` and `list.index(value, start, stop)` | `crates/sifr/tests/e2e/pass/collection_methods.sifr` | adapted | Safe return remains `T | None` instead of raising on misses/out-of-range, and Sifr accepts `start=` / `stop=` keywords for `list.index(...)` as an intentional compile-time-normalized adaptation even though CPython keeps those bounds positional-only. |
| `Lib/test/test_list.py` | unexpected keyword rejection on list methods | `crates/sifr/tests/e2e/fail/list_unexpected_keyword.sifr` | adapted | Sifr rejects unsupported method keywords at compile time. |
| `Lib/test/test_dict.py` | `dict.update()`/iterable-of-pairs/`**kwargs`, `dict.pop(key, default)`, and `dict.setdefault(key, default)` | `crates/sifr/tests/e2e/pass/collection_methods.sifr`, `crates/sifr/tests/e2e/pass/cpython_core_object_model_subset.sifr` | adapted | Default values remain statically typed to the dict value type; Sifr accepts `default=` keyword binding for `dict.pop(...)`/`dict.setdefault(...)` as an intentional convenience adaptation even though CPython keeps those parameters positional-only. |
| `Lib/test/test_dict.py` | invalid `dict.update()` iterable shape | `crates/sifr/tests/e2e/fail/dict_update_invalid_pairs.sifr` | adapted | Compile-time rejection replaces CPython runtime `ValueError`/`TypeError`. |
| `Lib/test/test_dict.py` | duplicate `dict.get(..., default=...)` binding | `crates/sifr/tests/e2e/fail/dict_get_duplicate_default.sifr` | adapted | Duplicate optional arguments are rejected during lowering instead of being silently accepted, and the accepted `default=` keyword itself is an intentional Sifr-only adaptation over CPython's positional-only API. |
| `Lib/test/test_dict.py` | incompatible `dict.setdefault(key, default)` default value type | `crates/sifr/tests/e2e/fail/dict_setdefault_invalid_default.sifr` | adapted | Compile-time type enforcement rejects defaults that cannot flow into the dict value type. |
| `Lib/test/test_set.py` | variadic `set.update`/`intersection`/`intersection_update`/`difference_update`/`symmetric_difference_update` and iterable inputs | `crates/sifr/tests/e2e/pass/collection_methods.sifr`, `crates/sifr/tests/e2e/pass/cpython_core_object_model_subset.sifr` | adapted | Iterable element compatibility is enforced statically across both pure and mutating set operations, including first-mutation local-variable call paths. |
| `Lib/test/test_set.py` | non-iterable set-update argument | `crates/sifr/tests/e2e/fail/set_update_non_iterable.sifr` | adapted | Compile-time rejection replaces CPython runtime `TypeError`. |
| `Lib/test/test_tuple.py` | `tuple.count` and `tuple.index(value, start)` | `crates/sifr/tests/e2e/pass/collection_methods.sifr` | adapted | `tuple.index` returns `int | None` instead of raising on misses, and Sifr accepts `start=` keyword binding for the optional bound even though CPython keeps it positional-only. |
| `Lib/test/test_tuple.py` | tuple index bound typing | `crates/sifr/tests/e2e/fail/tuple_index_invalid_bound.sifr` | adapted | Bound typing is enforced at compile time. |
| `Lib/test/test_str.py` | `str.split(sep=..., maxsplit=...)` and `str.replace(..., count=...)` | `crates/sifr/tests/e2e/pass/collection_methods.sifr` | adapted | `count < 0` still means “replace all”, aligned with CPython intent. |
| `Lib/test/test_str.py` | invalid `count=` type | `crates/sifr/tests/e2e/fail/str_replace_invalid_count.sifr` | adapted | Compile-time rejection replaces CPython runtime `TypeError`. |

## Executable CPython-Derived Subset Fixture

- `crates/sifr/tests/e2e/pass/cpython_core_object_model_subset.sifr`
  - Consolidates CPython-derived method/object-model assertions for `list`, `dict`, `set`, `tuple`, and `str` surfaces closed in this wave.

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| CPython `bytes` / `bytearray` object model families from `Lib/test/test_bytes.py` | `unsupported` | Historical wave-`a2` status: first-class `bytes` had not landed yet. As of `wave_psp_bytes_2` and `wave_psp_bytes_3`, immutable first-class `bytes` is shipped and downstream binary contracts use `bytes`; remaining waivers are now narrowed to `bytearray`/`memoryview`/buffer-protocol families and tracked in `verification/areas/stdlib_parity/reports/wave_psp_bytes_3_cpython_traceability.md`. |
