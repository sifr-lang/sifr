# `stdlib_parity_iter_fix_0` Architecture Lock (Canonical Iteration Model and Lazy Parity Readiness)

Capability: `issues/canonical-iteration-model-and-lazy-parity-readiness.md`
Execution ledger: `issues/canonical-iteration-model-and-lazy-parity-readiness-execution.md`

## Objective

Lock one canonical iteration rules across type system, HIR lowering, codegen, generators, builtins, and stdlib adapters before capability/codegen implementation passes begin.

This lock exists to prevent later implementation passes from reintroducing one-off iterator lowering or container-only assumptions.

## Locked Public Rules Snapshot

| Surface | Locked direction for this capability |
| --- | --- |
| Canonical iteration types | `Iterable[T]`, `Iterator[T]`, and `Reversible[T]` are the authoritative iteration ruless for this capability. |
| Lazy/eager boundary | `iter`/`next`/`reversed`/`map`/`filter`/`zip`/`enumerate` and generator families are lazy; collectors (`list`/`set`/`dict`/`tuple`/`sorted`) are explicit eager boundaries. |
| Capability model | Lowering/codegen must preserve iterator capabilities (`single-pass`, `multi-pass`, `double-ended`, `exact-size`) instead of recovering semantics from erased backend types. |
| `next` safety model | `next(it)` remains `T | None` (no user-facing `StopIteration`). |
| Tuple iteration rules | Homogeneous tuples are intended to iterate their shared element type; heterogeneous tuples without one statically provable element type are intentionally rejected. |
| Generator rules | Generator functions/expressions are first-class iterator producers and must use the same canonical iterator semantics as builtin iterator adapters. |

## Baseline Fractures Recorded at Capability 0 Entry

The following baseline fracture cases currently type-check but fail during Rust build in `run` mode (backend rules mismatch):

- `any(iter(xs))` over a list-backed iterator:
  - rustc failure anchor: `no method named 'iter' found for struct 'Box<dyn Iterator<Item = i64>>'`
- `filter(pred, iter(xs))`:
  - rustc failure anchor: clone/trait-bound failure on `Box<dyn Iterator<Item = i64>>`
- `reversed(iter(xs))`:
  - rustc failure anchor: `dyn Iterator<Item = i64>: DoubleEndedIterator` bound not satisfied
- `sorted(iter(xs))`:
  - rustc failure anchor: unresolved `sorted` call shape in emitted Rust

Additional baseline mismatch captured at baseline capability:

- homogeneous tuple `for`-iteration is currently rejected with:
  - `for-loop iterable must have a statically-known element type, got 'tuple[int, int, int]'`
- this is tracked as a implementation pass-owned readiness target, not a permanent divergence.

## Permanent Sifr-Safe Diffs (Locked for This Capability)

| Surface | Classification | Enforcement fixture |
| --- | --- | --- |
| Async iteration families (`aiter`, `anext`, `async for`) | `unsupported` | policy lock in this capability (no async iterator expansion) |
| Advanced iterator-object families (`itertools.tee`, `itertools.groupby`) | `unsupported` | `crates/sifr/tests/e2e/fail/itertools_tee_unsupported.sifr`, `crates/sifr/tests/e2e/fail/itertools_groupby_unsupported.sifr` |
| General-arity `itertools.starmap` callable/row parity | `unsupported` | `crates/sifr/tests/e2e/fail/itertools_starmap_non_binary_callable.sifr` |
| Implicit heterogeneous tuple union-yield iteration | `unsupported` | `crates/sifr/tests/e2e/fail/tuple_heterogeneous_iteration_unsupported.sifr` |

## CPython Family Mapping (Capability Ownership)

| CPython family | Direction | Owning capability | Local anchor |
| --- | --- | --- | --- |
| `Lib/test/test_iter.py` | `adapted` | `stdlib_parity_iter_fix_1` + `stdlib_parity_iter_fix_2` + `stdlib_parity_iter_fix_3` | canonical typing/lowering/codegen readiness targets |
| `Lib/test/test_filter.py` | `adapted` | `stdlib_parity_iter_fix_3` + `stdlib_parity_iter_fix_5` | backend chain correctness + lazy builtin cleanup |
| `Lib/test/test_enumerate.py` | `adapted` | `stdlib_parity_iter_fix_5` | builtin lazy/eager rules readiness |
| `Lib/test/test_generators.py` | `adapted` | `stdlib_parity_iter_fix_4` | generator backend unification and diagnostics |
| `Lib/test/test_itertools.py` | `adapted` | `stdlib_parity_iter_fix_6` | iterable signatures, lazy composition, buffered-helper governance |
| `Lib/test/test_tuple.py` (iteration-relevant subset) | `adapted` | `stdlib_parity_iter_fix_1` + `stdlib_parity_iter_fix_8` | tuple iteration consistency and downstream alignment |

## Architecture-Lock Validation Artifacts (Capability 0)

- Positive lock fixture: `crates/sifr/tests/e2e/pass/iterator_basics.sifr`
- Capability-set-0 demo:
  - `demos/lazy_iterators_basics/main.sifr`
- Permanent-diff negative fixtures:
  - `crates/sifr/tests/e2e/fail/itertools_tee_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/itertools_groupby_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/tuple_heterogeneous_iteration_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/itertools_starmap_non_binary_callable.sifr`
