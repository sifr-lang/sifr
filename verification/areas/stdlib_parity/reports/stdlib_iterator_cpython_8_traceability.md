# stdlib_parity_iter_fix_8 CPython Traceability Matrix

Wave: `stdlib_parity_iter_fix_8`
Scope: downstream iterator-sensitive alignment for inherited bytes/runtime/stdlib surfaces

## Upstream references reviewed

- `Lib/test/test_iter.py`
- `Lib/test/test_generators.py`
- `Lib/test/test_itertools.py`
- `Lib/test/test_pathlib.py`
- `Lib/test/test_re.py`

## Mapping summary

| CPython family | Sifr closure in wave 8 | Status | Evidence |
| --- | --- | --- | --- |
| `test_iter` iterable/iterator assignability and materialization boundaries | close local/return lowering gap for `Iterator[T] -> Iterable[T]` so typed bindings/returns compile through one shared iterable coercion path | adapted | `crates/sifr/tests/e2e/pass/iterator_integration.sifr`, `crates/sifr_codegen/src/lib.rs`, `crates/sifr_codegen/src/stmt_support_emitter.rs` |
| `test_itertools` downstream composition over non-container iterables | validate inherited lazy helper composition after wave-8 coercion closure (no unresolved lowering split between top-level and nested stmt lowering) | adapted | `crates/sifr/tests/e2e/pass/iterator_integration.sifr`, `demos/iterator_integration/main.sifr` |
| `test_pathlib` iterator-returning filesystem methods (`iterdir`, `rglob`) | revalidate runtime/file iterators compose with canonical iterator consumers (`islice`, `list`) under final phase contract | adapted | `crates/sifr/tests/e2e/pass/iterator_integration.sifr`, `demos/iterator_integration/main.sifr` |
| `test_re` iterator-returning match streams (`finditer`) | revalidate regex iterator-returning API composition against final iteration contract | adapted | `crates/sifr/tests/e2e/pass/iterator_integration.sifr`, `demos/iterator_integration/main.sifr` |
| reverse iteration over single-pass runtime iterators | preserve canonical reversible-capability rejection for runtime/file iterator outputs | adapted (diagnostic) | `crates/sifr/tests/e2e/fail/reversed_runtime_iterator_not_reversible.sifr` |

## Intentional differences retained

- Full CPython iterator-object parity for advanced stateful iterator classes remains scoped to prior explicit unsupported entries (`tee`, `groupby`) and is unchanged by this closure wave.
- Regex iterator behavior still materializes match results in `sifr.re` before yielding `Iterator[Match]`; this remains an explicit Sifr safety/runtime tradeoff and is not reclassified in wave 8.
