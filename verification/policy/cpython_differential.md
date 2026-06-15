# CPython Differential Oracle Policy

Schema version: 1

This policy defines the first merge-blocking CPython differential subset. It is intentionally narrow: every generated or hand-seeded oracle program must stay inside the supported constructs table and must avoid every excluded divergence unless a later policy revision explicitly moves that behavior into the supported subset.

The supported CPython interpreter is the local `python3` satisfying `verification/pyproject.toml`'s `requires-python` range. Oracle reports must include the exact `sys.version`. Generated-corpus results are not comparable across Python minor versions unless a later policy revision lists the program id in the exit-code-stable table with a cross-minor rationale.

Every accepted oracle program prints exactly one JSON line. The accepted value grammar is version 1: `null`, booleans, strings, bounded integers, homogeneous lists, and string-keyed dictionaries whose keys are canonicalized before output unless the program explicitly tests insertion order. Strings are compared as Unicode scalar value sequences after source-level NFC normalization; the runner performs only CRLF-to-LF line-ending normalization. Integers must stay within `[-1000000, 1000000]`. Recursive/container depth is limited to 4.

## Table 1. Supported Constructs

| No. | Construct | Exact Sifr-equivalent behavior |
| --- | --- | --- |
| 1 | Module-level `def main()` with explicit annotations on non-obvious locals | Sifr runs `main` as the program entry point and type checks all annotations before execution. |
| 2 | Bounded integer literals, `+`, `-`, `*`, `//`, `%`, unary `-`, and integer comparisons within `[-1000000, 1000000]` | Sifr integer results must match CPython for the bounded values used by the oracle; overflow and fixed-width casts are excluded by `D003_FIXED_WIDTH_OVERFLOW`. |
| 3 | Boolean literals and `and`/`or`/`not` over booleans | Sifr boolean short-circuit behavior must match CPython for side-effect-free boolean operands. |
| 4 | String literals, equality, concatenation, and `len` for NFC-normalized ASCII or NFC Unicode strings | Sifr string values must compare equal to CPython values after the source normalization policy; encoding boundary behavior is excluded by `D007_UNICODE_ENCODING`. |
| 5 | List literals of homogeneous supported values, indexing with in-range non-negative integer literals, `len`, and iteration without mutation | Sifr list read and iteration results must match CPython for immutable-in-loop programs. Mutation during iteration is excluded by `D006_DICT_ORDER_MUTATION`. |
| 6 | Dict literals with string keys and homogeneous supported values, lookup by existing key, `len`, and insertion-order observation without mutation | Sifr dict lookup and insertion-order observation must match CPython for the fixed dictionaries in the smoke suite. Mutation during iteration is excluded by `D006_DICT_ORDER_MUTATION`. |
| 7 | `if` statements and `for` loops over `range` or supported lists | Sifr control flow must produce the same serialized value as CPython for statically accepted programs. |
| 8 | Functions with positional required parameters and explicit return annotations, no default arguments, no varargs, no kwargs | Sifr call and return behavior must match CPython for the supported value grammar. Default-argument timing is excluded by `D004_DEFAULT_ARGUMENT_EVALUATION`. |
| 9 | Canonical JSON-like serialization through the oracle fixture helper | Both sides must print one line using the versioned value grammar. The comparison never depends on `repr`, display formatting, or exception message text. |

## Table 2. Excluded Divergences

| No. | Exclusion id | Divergence | Generator exclusion rule |
| --- | --- | --- | --- |
| 1 | `D001_RESULT_OPTION_ERRORS` | Sifr models recoverable failure with `Result`/`Option`; CPython raises exceptions. | Do not generate programs whose expected outcome requires raising, catching, comparing, or formatting exceptions. |
| 2 | `D002_OWNERSHIP_BORROW` | Sifr enforces ownership, moves, and borrow restrictions statically. | Do not generate aliasing, mutation, lifetime, captured-borrow, or move-after-use programs that CPython would accept dynamically. |
| 3 | `D003_FIXED_WIDTH_OVERFLOW` | Sifr has fixed-width numeric policies and explicit integer overflow diagnostics where CPython integers are arbitrary precision. | Keep integer values and intermediate results within the bounded range unless the case is explicitly an overflow exclusion test. |
| 4 | `D004_DEFAULT_ARGUMENT_EVALUATION` | CPython evaluates default arguments at function definition time; Sifr support is not part of this oracle subset. | Do not generate default arguments, mutable defaults, keyword defaults, or tests that depend on definition-time evaluation. |
| 5 | `D005_DIVISION_FLOOR` | Division and floor semantics can diverge for unsupported numeric families and error cases. | Generate only bounded integer `//` and `%` pairs with non-zero divisors whose CPython and Sifr semantics are declared supported. |
| 6 | `D006_DICT_ORDER_MUTATION` | Dict insertion order and mutation behavior must be explicit; mutation during iteration is outside the initial subset. | Do not generate dict or list mutation while iterating; canonicalize dict keys unless insertion order is the explicit tested behavior. |
| 7 | `D007_UNICODE_ENCODING` | Unicode normalization, encoding errors, byte/string boundaries, and platform text encodings can diverge. | Generate only source-normalized strings and avoid encode/decode, filesystem, locale, and byte-boundary behavior. |
| 8 | `D008_ASYNC_RUNTIME` | Sifr async/task runtime semantics are safety-oriented and not CPython event-loop semantics. | Do not generate `async`, `await`, tasks, channels, subprocesses, event loops, timers, or scheduling-sensitive behavior. |
| 9 | `D009_STATIC_NARROWING_REJECTION` | Sifr rejects some CPython-valid programs through static typing, narrowing, and ownership checks. | Generate only programs that `sifr check` accepts; rejected programs belong in diagnostics/regression suites, not differential success comparison. |
| 10 | `D010_FLOAT_PRECISION` | Float formatting, NaN, infinities, signed zero, and platform math precision are not in the first oracle subset. | Do not generate floats unless a later policy row gives an operation-specific exactness contract. |
| 11 | `D011_REPR_FORMATTING` | CPython `repr` and Sifr display formatting are not a semantic comparison boundary. | Do not compare `repr`, debug formatting, or human display strings; use canonical JSON-like serialization only. |
| 12 | `D012_EXCEPTION_MESSAGES` | CPython exception message text is not comparable to Sifr diagnostics or typed errors. | Do not compare stderr or exception message text; only compare declared exit-code buckets and JSON stdout for supported success cases. |

## Table 3. Exit-Code-Stable Programs

| No. | Program id | Allowed exit codes | Rationale |
| --- | --- | --- | --- |
| 1 | `bounded_int_arithmetic` | `0` | Pure supported integer operations with non-zero divisors and bounded intermediates must succeed in both CPython and Sifr. |
| 2 | `boolean_string_logic` | `0` | Pure boolean and string logic within the supported value grammar must succeed in both runtimes. |
| 3 | `list_iteration_indexing` | `0` | Homogeneous list reads, `len`, range loops, and non-mutating iteration must succeed in both runtimes. |
| 4 | `dict_lookup_order_independent` | `0` | Existing-key dict lookup, `len`, and explicit insertion-order observation without mutation must succeed in both runtimes. |

## Table 4. Exclusion Id References

| No. | Exclusion id | Referenced by |
| --- | --- | --- |
| 1 | `D001_RESULT_OPTION_ERRORS` | generator rule `exclude_result_option_error_paths` and smoke manifest `forbidden_exclusions`. |
| 2 | `D002_OWNERSHIP_BORROW` | generator rule `exclude_aliasing_and_borrow_paths` and smoke manifest `forbidden_exclusions`. |
| 3 | `D003_FIXED_WIDTH_OVERFLOW` | generator rule `bound_integer_intermediates` and smoke manifest `forbidden_exclusions`. |
| 4 | `D004_DEFAULT_ARGUMENT_EVALUATION` | generator rule `exclude_default_argument_timing` and smoke manifest `forbidden_exclusions`. |
| 5 | `D005_DIVISION_FLOOR` | generator rule `restrict_integer_division_pairs` and smoke manifest `forbidden_exclusions`. |
| 6 | `D006_DICT_ORDER_MUTATION` | generator rule `exclude_mutation_during_iteration` and smoke manifest `forbidden_exclusions`. |
| 7 | `D007_UNICODE_ENCODING` | generator rule `source_normalized_strings_only` and smoke manifest `forbidden_exclusions`. |
| 8 | `D008_ASYNC_RUNTIME` | generator rule `exclude_async_runtime_surfaces` and smoke manifest `forbidden_exclusions`. |
| 9 | `D009_STATIC_NARROWING_REJECTION` | generator rule `sifr_check_must_accept` and smoke manifest `forbidden_exclusions`. |
| 10 | `D010_FLOAT_PRECISION` | generator rule `exclude_float_semantics` and smoke manifest `forbidden_exclusions`. |
| 11 | `D011_REPR_FORMATTING` | generator rule `canonical_json_only` and smoke manifest `forbidden_exclusions`. |
| 12 | `D012_EXCEPTION_MESSAGES` | generator rule `do_not_compare_exception_messages` and smoke manifest `forbidden_exclusions`. |
