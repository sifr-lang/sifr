# `wave_psp_a1` CPython Traceability Matrix

Status: in_progress
Phase: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`
Wave: `wave_psp_a1`

## Inputs Reviewed

- `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_builtin.py`
- `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_list.py`
- `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_dict.py`
- `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_tuple.py`
- `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_str.py`
- `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_range.py`

## Adopt / Adapt / Waive

| surface | CPython family | state | local evidence | note |
| --- | --- | --- | --- | --- |
| `list()` / `list(iterable)` constructors | `test_list.py:15-24` | `adopted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr` | Empty and iterable-backed constructor entry works directly from Python-shaped source. |
| `list(sequence=...)` keyword rejection | `test_list.py:51` | `adapted` | `crates/sifr_lowering/src/lower/builtin_calls.rs` | CPython raises `TypeError`; Sifr rejects the unsupported keyword at compile time. |
| `tuple()` / `tuple(list literal)` / `tuple(str literal)` constructors | `test_tuple.py:30-38` | `adapted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr` | Literal-backed tuple construction is supported directly. |
| `tuple(dynamic_iterable)` on list variables | `test_tuple.py:34-38` | `waived` | `crates/sifr/tests/e2e/fail/tuple_dynamic_list_shape.sifr` | Sifr tuples are fixed-length typed values, so dynamic iterable-to-tuple conversion remains an explicit intentional difference for this wave. |
| `dict()` / `dict(iterable_of_pairs)` / `dict(mapping, **keywords)` style entry | `test_dict.py:37`, `test_dict.py:382-389`, `test_dict.py:1118-1125`, `test_dict.py:1527-1540` | `adapted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr` | Iterable-of-pairs and keyword merges work; unpacked `**kwargs` remain out of scope for this wave. |
| `ord()` code-point behavior | `test_builtin.py:1714-1739` | `adapted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr` | Literal single-character calls fold to `int`; dynamic strings use `Result[int, ValueError]` instead of exception control flow. |
| `chr()` Unicode code-point behavior | `test_str.py:746-747`, `test_builtin.py:1727-1739` | `adapted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr` | Literal valid code points fold to `str`; dynamic values use `Result[str, ValueError]`. |
| `sorted(iterable=..., key=None, key=callable, reverse=True)` | `test_builtin.py:2771-2793` | `adopted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr`, `crates/sifr/tests/e2e/fail/sorted_unexpected_keyword.sifr` | Natural positional and keyword entry shapes are lowered explicitly; unsupported keywords now diagnose instead of being silently ignored. |
| `reversed(sequence)` | `test_list.py:185-214`, `test_tuple.py:409-419`, `test_range.py:463-469` | `adapted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr` | Sequence-backed reversed results now flow through the iterator protocol (`Iterator[T]`) with explicit `list(...)` materialization at eager boundaries. |
| `enumerate(iterable, start)` / `enumerate(iterable, start=...)` | `test_builtin.py:2157-2158` | `adopted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr` | Optional start parity now works for both positional and keyword forms. |
| variadic `zip(*iterables)` | `test_builtin.py:2125-2140` | `adopted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr` | Zero-, one-, two-, and three-iterable lowering now works through a shared variadic path. |
| `zip(..., strict=True)` | `test_builtin.py:2181-2286` | `waived` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | `strict` is intentionally deferred; this wave closes variadic arity and base iterable lowering first. |
| multi-iterable `map()` | `test_builtin.py:1323-1355` | `adopted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr`, `crates/sifr/tests/e2e/fail/map_callable_arity_mismatch.sifr` | Callable arity is checked against iterable count, lambda/context typing flows through all iterable arguments, and the builtin now returns `Iterator[T]` with explicit `list(...)` materialization where eager values are required. |
| `map(..., strict=True)` | `test_builtin.py:1395-1504` | `waived` | `verification/stdlib/wave_psp_a1_cpython_traceability.md` | `strict` is deferred with the rest of the iterator-family parity work. |
| keyword `range(start=..., stop=..., step=...)` | `test_range.py:46-52`, `test_range.py:94-105` | `adapted` | `crates/sifr/tests/e2e/pass/builtin_callables_and_constructors.sifr`, `crates/sifr/tests/e2e/fail/range_duplicate_stop_keyword.sifr` | Sifr intentionally normalizes keyword forms for `range(...)` as a typed ergonomics adaptation while still rejecting positional/keyword duplication at compile time. |

## Executable CPython-Derived Subset Fixture

- `crates/sifr/tests/e2e/pass/cpython_builtins_subset.sifr`
  - Consolidates CPython-derived constructor and call-shape assertions for `list`, `tuple`, `dict`, `sorted`, `reversed`, `enumerate`, `zip`, `map`, `range`, `ord`, and `chr`.

## Result

`wave_psp_a1` closes the root-cause gap where builtin-special lowering either missed Python-shaped constructor/call forms entirely or silently dropped builtin keywords. Remaining gaps in this slice are explicit waivers, not accidental fallthrough behavior.
