# wave_psp_iter_fix_6 CPython Traceability

Phase: `ad-hoc-canonical-iteration-model-and-lazy-parity-closure`
Wave: `wave_psp_iter_fix_6` (`sifr.itertools` + iterator-returning stdlib closure)

## Scope Closure

This wave closes list-only assumptions in iterator-focused `sifr.itertools` helpers by allowing
`Iterable[T]` inputs where semantics are naturally iterable-driven.

Implemented iterable-first surfaces in `lib/sifr/itertools.sifr`:

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

- `chain` and `product` remain list-vararg entry points in this wave due current vararg list-invariance constraints in generic call checking. They still preserve lazy/eager behavior as previously defined.
- Buffered combinatoric helpers continue to materialize internally by design.

## CPython Family Mapping

Primary references:

- `Lib/test/test_itertools.py`
- `Lib/test/test_iter.py`

Mapped behavior assertions in wave fixtures:

- iterator input accepted by adapter helpers (`islice`, `accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`)
- iterator input accepted by iterable-first helpers (`pairwise`, `batched`, `cycle`)
- materialization boundary remains explicit (`list(...)`)
- non-iterable rejection remains explicit (`islice(42, ...)` fail fixture)

## Runtime/File Iterator Composition

Inherited iterator-returning runtime/file surfaces now compose with iterator-first helpers:

- `Path.iterdir()` + `islice(...)`
- `Path.rglob()` + `islice(...)`

This is validated in `phase_psp_iter_fix_6_itertools_iterable_stdlib_closure.sifr`.

Cross-check:

- `demos/m30_1d_itertools_parity_demo/main.sifr` exercises `pairwise` and is expected to compile/run with this wave after the iterable-`pairwise` Option-state fix.
