# wave_psp_iter_fix_0 CPython Traceability Matrix

Wave: `wave_psp_iter_fix_0`  
Scope: contract freeze and governance lock for canonical iterator semantics

## CPython Harvest Inputs

- `Lib/test/test_iter.py`
- `Lib/test/test_filter.py`
- `Lib/test/test_enumerate.py`
- `Lib/test/test_generators.py`
- `Lib/test/test_itertools.py`
- `Lib/test/test_tuple.py` (iteration-focused subset)

## Adopt / Adapt / Waive (Wave 0 Lock)

| CPython family | Sifr surface direction | State | Owning wave |
| --- | --- | --- | --- |
| `test_iter` protocol semantics (`iter`, `next`, iterator-vs-iterable behavior) | preserve first-class iterable/iterator model while closing current backend-capability fractures | `adapted` (planned) | `wave_psp_iter_fix_1` + `wave_psp_iter_fix_2` + `wave_psp_iter_fix_3` |
| `test_filter` lazy behavior | enforce true lazy `filter` semantics through canonical iterator lowering/codegen | `adapted` (planned) | `wave_psp_iter_fix_3` + `wave_psp_iter_fix_5` |
| `test_enumerate` lazy iterator behavior | keep iterator-returning behavior across typing/lowering/execution with explicit materialization boundaries | `adapted` (planned) | `wave_psp_iter_fix_5` |
| `test_generators` iterator-producing generator behavior | align generator functions and expressions with canonical iterator backend semantics | `adapted` (planned) | `wave_psp_iter_fix_4` |
| `test_itertools` lazy adapter composition | rewrite `sifr.itertools` around `Iterable[...]`/lazy semantics where valid; preserve explicit buffered helpers | `adapted` (planned) | `wave_psp_iter_fix_6` |
| `test_tuple` iteration behavior | support homogeneous tuple iteration and explicitly reject heterogeneous tuple union-yield iteration | `adapted` (planned) | `wave_psp_iter_fix_1` + `wave_psp_iter_fix_8` |

## Explicit Waivers / Boundaries Locked in Wave 0

- Async iteration families (`aiter`, `anext`, `async for`) remain `unsupported` in this phase.
- Advanced iterator-object families (`itertools.tee`, `itertools.groupby`) remain `unsupported`.
- General-arity `itertools.starmap` callable/row parity remains `unsupported` (binary rows only).
- Heterogeneous tuple union-yield iteration remains `unsupported`.

## Local Fixture Anchors (Wave 0)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/iterator_basics.sifr`
- Demo:
  - `demos/lazy_iterators_basics/main.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/itertools_tee_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/itertools_groupby_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/tuple_heterogeneous_iteration_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/itertools_starmap_non_binary_callable.sifr`
