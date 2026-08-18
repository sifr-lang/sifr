# stdlib_parity_iter_fix_6 CPython Traceability

Capability: `canonical-iteration-model-and-lazy-parity-readiness`
Capability: `stdlib_parity_iter_fix_6` (`sifr.itertools` + iterator-returning stdlib readiness)

## Scope Readiness

This implementation pass closes list-only assumptions in iterator-focused `sifr.itertools` helpers by allowing
`Iterable[T]` inputs where semantics are naturally iterable-driven.

Implemented iterable-first surfaces in `stdlib/sifr/itertools.sifr`:

- `islice`
- `accumulate`
- `compress`
- `dropwhile`
- `takewhile`
- `filterfalse`
- `take`
- `pairwise`
- `batched`
- `flatten`
- `permutations`
- `combinations`
- `combinations_with_replacement`
- `starmap`
- `zip_longest`
- `cycle`

Notes:

- `chain` and `product` remain list-vararg entry points in this implementation pass due current vararg list-invariance constraints in generic call checking. They still preserve lazy/eager behavior as previously defined.
- Buffered combinatoric helpers continue to materialize internally by design.

## CPython Family Mapping

Primary references:

- `Lib/test/test_itertools.py`
- `Lib/test/test_iter.py`

Mapped behavior assertions in implementation pass fixtures:

- iterator input accepted by adapter helpers (`islice`, `accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`)
- iterator input accepted by iterable-first helpers (`pairwise`, `batched`, `cycle`)
- materialization boundary remains explicit (`list(...)`)
- non-iterable rejection remains explicit (`islice(42, ...)` fail fixture)

## Runtime/File Iterator Composition

Inherited iterator-returning runtime/file surfaces now compose with iterator-first helpers:

- `Path.iterdir()` + `islice(...)`
- `Path.rglob()` + `islice(...)`

This is validated in `iterable_stdlib.sifr`.

Cross-check:

- `demos/itertools/main.sifr` exercises `pairwise` and is expected to compile/run with this implementation pass after the iterable-`pairwise` Option-state fix.

## Post-Readiness Add-On (2026-03-20)

Expanded CPython `test_itertools` parity-port coverage for all shipped `sifr.itertools` helpers
was landed in:

- `crates/sifr/tests/e2e/pass/cpython_itertools.sifr`

Ported CPython families (adapted to Sifr ruless where intentional diffs are documented):

- `TestBasicOps.test_chain`
- `TestBasicOps.test_repeat` / `test_repeat_with_negative_times`
- `TestBasicOps.test_batched`
- `TestBasicOps.test_islice`
- `TestBasicOps.test_accumulate`
- `TestBasicOps.test_compress`
- `TestBasicOps.test_count` / `test_count_with_step`
- `TestBasicOps.test_cycle` (adapted to finite `cycle(data, n)`)
- `TestBasicOps.test_takewhile` / `test_dropwhile` / `test_filterfalse`
- `TestBasicOps.test_ziplongest`
- `TestBasicOps.test_product`
- `TestBasicOps.test_permutations`
- `TestBasicOps.test_combinations`
- `TestBasicOps.test_combinations_with_replacement`
- `TestBasicOps.test_starmap` (binary callable-row scope)
- `TestExamples` equivalents for the same shipped helper family

Intentional-diff boundaries remain unchanged and explicitly governed:

- `itertools.tee` / `itertools.groupby` remain unsupported
- general-arity `starmap` callable rows remain unsupported
- `cycle` remains finite (`cycle(data, n)`)
- `product(..., repeat < 0)` remains bounded empty-iterator adaptation
- `count(start=0, step=1)` remains a bounded-prefix iterator in Sifr (`count_from(..., 10000)`), preserving CPython-leading-value behavior while avoiding unsupported unbounded generator lowering
