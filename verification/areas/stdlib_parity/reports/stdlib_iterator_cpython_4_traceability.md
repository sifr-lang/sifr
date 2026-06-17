# stdlib_parity_iter_fix_4 CPython Traceability Matrix

Capability: `stdlib_parity_iter_fix_4`
Scope: generator backend unification and filtered generator-expression readiness

## CPython Harvest Inputs

- `Lib/test/test_generators.py` (generator functions with multiple yield sites and loop-backed iteration)
- `Lib/test/test_iter.py` (iterator-protocol behavior for generator-returning surfaces)
- `Lib/test/test_filter.py` (filtered lazy iterator behavior, adapted through generator expressions)

## Adopt / Adapt / Waive (Capability 4)

| CPython family | Sifr surface direction | State | Evidence |
| --- | --- | --- | --- |
| `test_generators` mixed-yield generator body | support loop-backed generator functions with multiple yield sites under one iterator backend | `adapted` (closed in translation-catalog capability backend) | `crates/sifr/tests/e2e/pass/generator_iterators.sifr` |
| `test_generators` for-loop generator body | allow generator functions that yield from `for` loops over iterable inputs | `adapted` (closed in translation-catalog capability backend) | `crates/sifr/tests/e2e/pass/generator_iterators.sifr` |
| `test_filter` filtered generator expressions | ensure `(expr for x in iter if cond)` lowers through canonical lazy iterator chains | `adapted` (closed in translation-catalog capability backend) | `crates/sifr/tests/e2e/pass/generator_iterators.sifr` |

## Local Fixture Anchors (Capability 4)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/generator_iterators.sifr`
- Demo:
  - `demos/generator_iterators/main.sifr`
