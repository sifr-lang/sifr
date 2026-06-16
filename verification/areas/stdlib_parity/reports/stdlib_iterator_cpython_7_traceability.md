# stdlib_parity_iter_fix_7 CPython Traceability

Phase: `ad-hoc-canonical-iteration-model-and-lazy-parity-closure`
Wave: `stdlib_parity_iter_fix_7` (user-defined iterable protocol participation)

## Scope Closure

This wave enables user-defined classes to participate in canonical iterable semantics through
protocol-shaped methods:

- `__iter__`
- `__next__`
- `__reversed__`

Implemented closure in compiler layers:

- type-system iterable/iterator/reversible inference from user-class protocol methods
- HIR `next(...)` typing support for user-defined iterator classes
- codegen iterable lowering over user classes through protocol methods rather than container-only assumptions
- protocol-conformance diagnostics for malformed user-defined iteration methods

## CPython Family Mapping

Primary references:

- `Lib/test/test_iter.py`
- `Lib/test/test_generators.py` (iterator next semantics)

Mapped behavior assertions in wave fixtures:

- user-defined iterable class composes with `list(...)` and `for` iteration
- user-defined reversible class composes with `reversed(...)`
- user-defined iterator class composes with `next(...)`
- malformed protocol signatures are rejected with explicit diagnostics

## Validation Artifacts

- pass fixture:
  - `crates/sifr/tests/e2e/pass/user_defined_iterables.sifr`
- fail fixtures:
  - `crates/sifr/tests/e2e/fail/invalid_iter_signature.sifr`
  - `crates/sifr/tests/e2e/fail/invalid_next_signature.sifr`
  - `crates/sifr/tests/e2e/fail/invalid_reversed_signature.sifr`
- demo:
  - `demos/custom_iterables/main.sifr`
