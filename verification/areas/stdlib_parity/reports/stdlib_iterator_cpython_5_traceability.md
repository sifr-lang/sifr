# stdlib_parity_iter_fix_5 CPython Traceability Matrix

Wave: `stdlib_parity_iter_fix_5`
Scope: builtin lazy/eager boundary cleanup (`filter` laziness and iterable-input parity for core builtin consumers)

## CPython Harvest Inputs

- `Lib/test/test_filter.py` (lazy filter iterator behavior, explicit materialization via `list(...)`)
- `Lib/test/test_iter.py` (iterable-vs-iterator builtin consumer parity)
- `Lib/test/test_builtin.py` (core builtin iterable consumption for `sum`, `min`, `max`, `sorted`)

## Adopt / Adapt / Waive (Wave 5)

| CPython family | Sifr surface direction | State | Evidence |
| --- | --- | --- | --- |
| `test_filter` lazy iterator contract | `filter(func, iterable)` returns lazy iterator; concrete list requires explicit `list(...)` | `adapted` (closed in wave 5) | `crates/sifr/tests/e2e/pass/lazy_builtins.sifr` |
| `test_iter` builtin iterable consumers | `sum`, unary `min`, unary `max` accept general iterable inputs (including iterator values) | `adapted` (closed in wave 5) | `crates/sifr/tests/e2e/pass/lazy_builtins.sifr` |
| `test_builtin` explicit collection materialization boundary | assignment to concrete list from `filter(...)` requires `list(filter(...))` | `adapted` (diagnostic closure) | `crates/sifr/tests/e2e/fail/filter_requires_explicit_materialization.sifr` |

## Local Fixture Anchors (Wave 5)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/lazy_builtins.sifr`
- Negative fixture:
  - `crates/sifr/tests/e2e/fail/filter_requires_explicit_materialization.sifr`
- Demo:
  - `demos/iterator_builtins/main.sifr`
