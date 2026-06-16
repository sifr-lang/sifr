# stdlib_parity_iter_fix_2 CPython Traceability Matrix

Wave: `stdlib_parity_iter_fix_2`
Scope: canonical iterator HIR for protocol entry and iterator builtins (including generator-expression and comprehension sources)

## CPython Harvest Inputs

- `Lib/test/test_iter.py` (iter/next protocol entry + iterator builtin behavior)
- `Lib/test/test_generators.py` (generator-expression iterator source semantics)
- `Lib/test/test_enumerate.py` (enumerate iterator shape)

## Adopt / Adapt / Waive (Wave 2)

| CPython family | Sifr surface direction | State | Evidence |
| --- | --- | --- | --- |
| `test_iter` protocol-entry lowering (`iter(...)` / `next(...)`) | lower protocol entry through canonical HIR node instead of generic string call | `adapted` (closed in wave 2 lowering layer) | `crates/sifr_lowering/src/lower/expressions_tests.rs` (`test_iterator_builtins_lower_to_canonical_iterator_call_nodes`) |
| `test_iter` lazy builtin family (`reversed`, `map`, `filter`, `zip`, `enumerate`) | lower builtin iterator operations through canonical `IteratorCall` HIR family | `adapted` (closed in wave 2 lowering layer) | `crates/sifr/tests/e2e/pass/iterator_sources.sifr` |
| `test_generators` generator-expression source protocol entry | canonicalize generator-expression source lowering through `IteratorCall::Iter` | `adapted` (closed in wave 2 lowering layer) | `crates/sifr_lowering/src/lower/expressions_tests.rs` (`test_iterator_builtins_lower_to_canonical_iterator_call_nodes`) |

## Local Fixture Anchors (Wave 2)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/iterator_sources.sifr`
- Demo:
  - `demos/iterator_lowering/main.sifr`
