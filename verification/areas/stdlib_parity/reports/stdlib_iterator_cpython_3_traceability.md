# stdlib_parity_iter_fix_3 CPython Traceability Matrix

Wave: `stdlib_parity_iter_fix_3`
Scope: concrete iterator codegen pipelines for iterator-consuming builtin surfaces

## CPython Harvest Inputs

- `Lib/test/test_iter.py` (`iter(...)`/iterator-object consumption)
- `Lib/test/test_filter.py` (`filter(...)` lazy iterator behavior)
- `Lib/test/test_sort.py` (`sorted(...)` iterable consumption and eager materialization boundary)

## Adopt / Adapt / Waive (Wave 3)

| CPython family | Sifr surface direction | State | Evidence |
| --- | --- | --- | --- |
| `test_iter` iterator-consumer codegen | lower iterator consumers through canonical iterable-to-owned-iterator conversion path | `adapted` (closed in wave 3 codegen layer) | `crates/sifr/tests/e2e/pass/iterator_consumers.sifr` |
| `test_filter` iterator-input closure | `filter(pred, iter(xs))` must compile through concrete iterator chain instead of unresolved builtin symbol fallback | `adapted` (closed in wave 3 codegen layer) | `crates/sifr/tests/e2e/pass/iterator_consumers.sifr` |
| `test_sort` iterable-input closure | `sorted(iter(xs))` must materialize only at explicit eager boundary and accept iterator-typed sources | `adapted` (closed in wave 3 codegen layer) | `crates/sifr/tests/e2e/pass/iterator_consumers.sifr` |
| `test_iter` reversible capability guard | `reversed(iter(xs))` remains rejected through explicit capability-aware typing (`Iterator[T]` is single-pass and not reversible by default) | `adapted` (guard retained) | `crates/sifr/tests/e2e/fail/reversed_iterator_not_reversible.sifr` |

## Local Fixture Anchors (Wave 3)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/iterator_consumers.sifr`
- Demo:
  - `demos/iterator_codegen/main.sifr`
