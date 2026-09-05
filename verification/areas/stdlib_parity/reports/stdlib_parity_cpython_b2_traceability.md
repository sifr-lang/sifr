# `stdlib_parity_b2` CPython Traceability

## Validationed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_itertools.py` | variadic `chain`, `islice(start, stop, step)`, `product(..., repeat=)`, `permutations`, `combinations`, `combinations_with_replacement`, `starmap`, and `accumulate(..., initial=...)` | `crates/sifr/tests/e2e/pass/iterators_random_and_secrets.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr`<br>`crates/sifr/tests/e2e/fail/itertools_starmap_non_binary_callable.sifr` | adapted | Iterator-returning ruless now cover the approved combinator set (`product`, `permutations`, `combinations`, `combinations_with_replacement`, `starmap`, `accumulate`, predicate combinators, `zip_longest`, and `cycle`) with explicit source-level materialization in fixtures where collection values are asserted. |
| `Lib/test/test_functools.py` | higher-order callable acceptance around `reduce(...)` and downstream callable-wrapper infrastructure | `crates/sifr/tests/e2e/pass/iterators_random_and_secrets.sifr`<br>`crates/sifr/tests/e2e/fail/functools_partial_unsupported.sifr` | adapted | `reduce(...)` remains the shipped functools entry point in this implementation pass, and the compiler root fix now makes user-defined `__call__` objects callable directly instead of being rejected as undefined function names. |
| `Lib/test/test_operator.py` | `getitem`, `itemgetter`, `contains`, `truth`, and core boolean helpers (`and_`, `or_`, `not_`) | `crates/sifr/tests/e2e/pass/iterators_random_and_secrets.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_operator.sifr`<br>`crates/sifr/tests/e2e/fail/operator_attrgetter_unsupported.sifr`<br>`crates/sifr/tests/e2e/fail/operator_methodcaller_unsupported.sifr` | adapted | The public helpers now use the Python names directly. `itemgetter(items, index)` remains a direct helper in this implementation pass rather than the CPython callable factory. `getitem(...)` follows Sifr safe-indexing (`None` on out-of-range) and `truth(...)` is intentionally list-only in this implementation pass. |
| `Lib/test/test_random.py` | mutating `shuffle`, `randrange(stop)` / `randrange(start, stop, step)`, `choice`, `choices`, `getrandbits`, plus helper coverage for `randint`, `random`, `uniform`, `gauss`, and `sample` | `crates/sifr/tests/e2e/pass/iterators_random_and_secrets.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_random_subset.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_random.sifr` | adapted | Empty-population and invalid-argument paths now raise typed `ValueError` results instead of panicking. `shuffle` mutates in place and returns `None`, aligned with CPython's public rules. |
| `Lib/test/test_secrets.py` | `compare_digest`, `randbits`, `choice`, `randbelow`, and `token_hex` | `crates/sifr/tests/e2e/pass/iterators_random_and_secrets.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_secrets_subset.sifr` | adapted | `compare_digest` preserves boolean equality semantics for `str` inputs only. CPython's timing-safe constant-time guarantee is intentionally not claimed in this implementation pass, and callers must not rely on constant-time behavior. |

## Classified waivers

State note: iterator readiness moved core iterator/lazy surfaces out of `capability-tracked` status. Residual gaps below are now explicit terminal classifications.

| Surface | State | Rationale |
| --- | --- | --- |
| Core iterator-object/lazy parity surfaces (`iter`, `next`, protocol `for` lowering, generators, `zip`, `enumerate`, `reversed`, `chain`, `repeat`, `islice`, `count`) | `parity-closed` | Closed by `issues/first-class-lazy-iterators-and-python-iterable-protocol.md` and execution ledger implementation pass set (`implementation pass_iter_1`-`implementation pass_iter_6`), with dedicated readiness demo and regression coverage. |
| Approved iterator-returning itertools combinators (`accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`, `zip_longest`, `cycle`, `starmap`, `product`, `permutations`, `combinations`, `combinations_with_replacement`) | `parity-closed` | Public ruless are iterator-returning and require explicit source-level materialization (`list(...)`) when reusable collection values are needed. |
| Residual advanced iterator-object families (`itertools.tee`, `itertools.groupby`) | `intentional-diff` | These families remain intentionally unshipped in the current continuation scope because they require additional iterator object-lifetime/state semantics beyond the approved combinator set. |
| `functools.partial`, `cmp_to_key`, and cache/decorator families | `unsupported` | Returned callable values and callable-object use inside higher-order stdlib helpers still hit separate codegen limitations outside this implementation pass's closed direct callable-object invocation work, and the broader CPython callable-wrapper matrix also needs ParamSpec-style callable typing that the current type system does not yet expose (guarded by `functools_partial_unsupported.sifr`). |
| General-arity `itertools.starmap(...)` callable/row parity | `intentional-diff` | This implementation pass ships a typed `starmap` over binary tuple rows only; non-binary callable rows are intentionally rejected at compile time (guarded by `itertools_starmap_non_binary_callable.sifr`). |
| `itertools.cycle(...)` infinite-iterator signature parity | `intentional-diff` | Sifr uses finite `cycle(data, n)` semantics instead of CPython's infinite `cycle(iterable)` for deterministic bounded execution. |
| `itertools.product(..., repeat=<0)` error parity | `intentional-diff` | CPython raises `ValueError` for negative `repeat`; Sifr currently returns an empty iterator and treats this as a bounded safe adaptation (`cpython_itertools_subset.sifr` assertion). |
| `operator.attrgetter` and `operator.methodcaller` callable factories | `unsupported` | Reflective attribute and method lookup by string is not available in the current statically typed object model (guarded by `operator_attrgetter_unsupported.sifr` and `operator_methodcaller_unsupported.sifr`). |
| Residual `random` waiver family (`choices(weights=...)`, `SystemRandom.getstate/setstate`) | `unsupported` | `stdlib_parity_rng_1` now ships deterministic mutable-state parity (`RandomState`, `Random`, module-level delegation, `randbytes`). Remaining unsupported pieces are weighted-distribution `choices(weights=...)` and host-backed `SystemRandom` state export/import (guarded by `random_choices_weights_unsupported.sifr` and `system_random_state_unsupported.sifr`). |
| `secrets.token_urlsafe(...)` and bytes-oriented `compare_digest(...)` parity | `unsupported` | This implementation pass stays on `str`-only parity and does not claim CPython's bytes/base64-oriented security surface without a first-class bytes type (guarded by `secrets_token_urlsafe_unsupported.sifr`). |
| Constant-time timing-safety guarantees for `secrets.compare_digest(...)` | `unsupported` | The current implementation keeps value-equality semantics but does not provide a constant-time side-channel-hardening rules; callers must not treat it as timing-safe in this implementation pass. |

## Negative Coverage Evidence

- Compile-time waiver guards:
  - `crates/sifr/tests/e2e/fail/functools_partial_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/itertools_starmap_non_binary_callable.sifr`
  - `crates/sifr/tests/e2e/fail/operator_attrgetter_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/operator_methodcaller_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/random_choices_weights_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/secrets_token_urlsafe_unsupported.sifr`
- Runtime negative-path assertions:
  - `crates/sifr/tests/e2e/pass/cpython_random_subset.sifr` asserts error behavior for empty `choice`/`choices`, invalid `randrange` bounds/step, and invalid `getrandbits` bit width.
