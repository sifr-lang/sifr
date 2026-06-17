# stdlib_parity_iter_fix_1 CPython Traceability Matrix

Capability: `stdlib_parity_iter_fix_1`
Scope: type-system capability layer (`Reversible`, capability metadata, tuple iterability rules alignment)

## CPython Harvest Inputs

- `Lib/test/test_iter.py` (iterator-vs-iterable and reverse-capability checks)
- `Lib/test/test_tuple.py` (iteration-focused subset)

## Adopt / Adapt / Waive (Capability 1)

| CPython family | Sifr surface direction | State | Evidence |
| --- | --- | --- | --- |
| `test_iter` reverse-capability behavior | enforce explicit reversible capability at type-check time (`reversed(...)` rejects non-double-ended iterators) | `adapted` (closed in text encoding capability typing layer) | `crates/sifr/tests/e2e/fail/reversed_iterator_not_reversible.sifr` |
| `test_iter` iterable protocol annotations | add `Reversible[T]` protocol alias and capability-aware assignability for builtin containers | `adapted` (closed in text encoding capability typing layer) | `crates/sifr/tests/e2e/fail/reversible_annotation_rejects_set.sifr`, `crates/sifr/tests/e2e/pass/reversible_iteration.sifr` |
| `test_tuple` homogeneous vs heterogeneous iteration | allow homogeneous tuple iteration and reject heterogeneous tuple iteration with explicit diagnostics | `adapted` (closed in text encoding capability typing layer) | `crates/sifr/tests/e2e/pass/reversible_iteration.sifr`, `crates/sifr/tests/e2e/fail/iter_heterogeneous_tuple_unsupported.sifr` |

## Local Fixture Anchors (Capability 1)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/reversible_iteration.sifr`
- Demo:
  - `demos/reversible_iterables/main.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/reversed_iterator_not_reversible.sifr`
  - `crates/sifr/tests/e2e/fail/iter_heterogeneous_tuple_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/reversible_annotation_rejects_set.sifr`
