# `wave_psp_b2` CPython Traceability

## Reviewed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_itertools.py` | variadic `chain`, `islice(start, stop, step)`, `product(..., repeat=)`, `permutations`, `combinations`, `combinations_with_replacement`, `starmap`, and `accumulate(..., initial=...)` | `crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr` | adapted | Sifr keeps eager `list[...]` materialization instead of CPython's lazy iterator objects, and `starmap` remains typed to two-argument tuple rows in this wave; closed call shapes and combinator results are now covered by both wave and CPython-subset fixtures. |
| `Lib/test/test_functools.py` | higher-order callable acceptance around `reduce(...)` and downstream callable-wrapper infrastructure | `crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr`<br>`crates/sifr/tests/e2e/fail/phase_psp_b2_functools_partial_unsupported.sifr` | adapted | `reduce(...)` remains the shipped functools entry point in this wave, and the compiler root fix now makes user-defined `__call__` objects callable directly instead of being rejected as undefined function names. |
| `Lib/test/test_operator.py` | `getitem`, `contains`, `truth`, and core boolean helpers (`and_`, `or_`, `not_`) | `crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_operator.sifr`<br>`crates/sifr/tests/e2e/fail/phase_psp_b2_operator_attrgetter_unsupported.sifr`<br>`crates/sifr/tests/e2e/fail/phase_psp_b2_operator_methodcaller_unsupported.sifr` | adapted | The public helpers now use the Python names directly. `itemgetter(items, index)` remains a direct helper in this wave rather than the CPython callable factory. |
| `Lib/test/test_random.py` | mutating `shuffle`, `randrange(stop)` / `randrange(start, stop, step)`, `choice`, `choices`, and `getrandbits` | `crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_random_subset.sifr` | adapted | Empty-population and invalid-argument paths now raise typed `ValueError` results instead of panicking. `shuffle` mutates in place and returns `None`, aligned with CPython's public contract. |
| `Lib/test/test_secrets.py` | `compare_digest`, `randbits`, `choice`, `randbelow`, and `token_hex` | `crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_secrets_subset.sifr` | adapted | `compare_digest` preserves boolean equality semantics for `str` inputs only. CPython's timing-safe constant-time guarantee is intentionally not claimed in this wave. |

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| Lazy iterator object families from `Lib/test/test_itertools.py` | `unsupported` | Sifr still lowers these helpers to eager `list[...]` materializations; closing the exact CPython iterator-object model would require a broader lazy-iterator runtime layer. |
| `functools.partial`, `cmp_to_key`, and cache/decorator families | `unsupported` | Returned callable values and callable-object use inside higher-order stdlib helpers still hit separate codegen limitations outside this wave's closed direct callable-object invocation work, and the broader CPython callable-wrapper matrix also needs ParamSpec-style callable typing that the current type system does not yet expose (guarded by `phase_psp_b2_functools_partial_unsupported.sifr`). |
| `operator.attrgetter` and `operator.methodcaller` callable factories | `unsupported` | Reflective attribute and method lookup by string is not available in the current statically typed object model (guarded by `phase_psp_b2_operator_attrgetter_unsupported.sifr` and `phase_psp_b2_operator_methodcaller_unsupported.sifr`). |
| Weighted `random.choices(...)`, `seed`, `getstate`, `setstate`, and `Random` / `SystemRandom` object families | `unsupported` | The current crypto-backed randomness layer does not expose deterministic stateful generator objects or weighted-distribution helpers (guarded by `phase_psp_b2_random_choices_weights_unsupported.sifr`). |
| `secrets.token_urlsafe(...)` and bytes-oriented `compare_digest(...)` parity | `unsupported` | This wave stays on `str`-only parity and does not claim CPython's bytes/base64-oriented security surface without a first-class bytes type (guarded by `phase_psp_b2_secrets_token_urlsafe_unsupported.sifr`). |
| Constant-time timing-safety guarantees for `secrets.compare_digest(...)` | `unsupported` | The current implementation keeps value-equality semantics but does not provide a constant-time side-channel-hardening contract; callers must not treat it as timing-safe in this wave. |

## Negative Coverage Evidence

- Compile-time waiver guards:
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_functools_partial_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_operator_attrgetter_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_operator_methodcaller_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_secrets_token_urlsafe_unsupported.sifr`
- Runtime negative-path assertions:
  - `crates/sifr/tests/e2e/pass/cpython_random_subset.sifr` asserts error behavior for empty `choice`/`choices`, invalid `randrange` bounds/step, and invalid `getrandbits` bit width.
