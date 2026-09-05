# `stdlib_parity_e2` CPython Traceability

## Validationed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_argparse.py` | parser construction, option/default handling, positional binding, boolean flag action, and token-shape handling (`--name=value`, `--`) | `crates/sifr/tests/e2e/pass/cpython_argparse_subset.sifr`<br>`crates/sifr/tests/e2e/pass/class_based_stdlib.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_argparse.sifr`<br>`crates/sifr/tests/e2e/fail/argparse_parse_args_non_string_list.sifr`<br>`demos/utility_classes/main.sifr` | adapted | `ArgumentParser`/`Namespace` class model is closed for typed Sifr usage, including inline option values, end-of-options positional mode, and missing-option fallback when a pending option is followed by another option token. Dynamic attribute mutation and full CLI error/reporting behavior remain explicit waivers. |
| `Lib/test/test_ipaddress.py` | IPv4 validation, parsing, leading-zero rejection, classification helpers, and factory error behavior | `crates/sifr/tests/e2e/pass/cpython_ipaddress_subset.sifr`<br>`crates/sifr/tests/e2e/pass/class_based_stdlib.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_ipaddress.sifr`<br>`crates/sifr/tests/e2e/fail/ip_address_non_string.sifr`<br>`demos/utility_classes/main.sifr` | adapted | `ip_address`/`ipv4_address` reject leading-zero IPv4 forms and preserve `AddressValueError`-typed failures; IPv4 classification was aligned with CPython special-range behavior for `is_private`/`is_global`, including 100.64/10 and 192.0.0.9/.10 exceptions. Direct `IPv4Address(...)` constructor remains non-raising and marks invalid input with sentinel state (`packed_int() == -1`). |
| `Lib/test/test_uuid.py` | UUID generation, parse/validation behavior, URN/curly normalization, class properties, and canonical text shape | `crates/sifr/tests/e2e/pass/cpython_uuid_subset.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_uuid_consolidated.sifr`<br>`crates/sifr/tests/e2e/pass/class_based_stdlib.sifr`<br>`crates/sifr/tests/e2e/pass/uuid_and_datetime.sifr`<br>`crates/sifr/tests/e2e/fail/uuid_from_hex_non_string.sifr` | adapted | UUID parse parity is strengthened via `uuid_from_hex` (supports canonical hex, hyphenated, `urn:uuid:...`, and `{...}` forms). `stdlib_parity_struct_3` closes deterministic name-based generation (`uuid3`, `uuid5`) with namespace constants while raw `UUID(...)` construction remains pass-through due current constructor-lowering constraints. |
| `Lib/test/test_graphlib.py` | `TopologicalSorter` DAG behavior, sparse-node static ordering, incremental readiness flow, and cycle errors | `crates/sifr/tests/e2e/pass/cpython_graphlib_subset.sifr`<br>`crates/sifr/tests/e2e/pass/class_based_stdlib.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_graphlib_class.sifr`<br>`crates/sifr/tests/e2e/fail/graphlib_add_non_int_predecessor.sifr`<br>`demos/utility_classes/main.sifr` | adapted | `TopologicalSorter` now tracks explicit added nodes and no longer leaks undeclared intermediary nodes in sparse-ID graphs; incremental flow remains deterministic one-node batches (`done(node)` progression). |
| `Lib/test/test_unittest.py` (assertion semantics used as proxy for `sifr.test`) | assertion helpers and typed comparison ruless | `crates/sifr/tests/e2e/pass/cpython_unittest_assertions_subset.sifr`<br>`crates/sifr/tests/e2e/pass/class_based_stdlib.sifr` | adapted | `test` is Sifr infrastructure rather than CPython stdlib parity target; assertion hardness is mapped via a `unittest`-style subset and classified below. |

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| `argparse` formatter/help ecosystem (`formatter_class`, rich help formatting subclasses) | `unsupported` | `stdlib_parity_struct_2` closes bounded parser expansion (`subparsers`, bounded `nargs`, typed coercion), but dynamic help-rendering ecosystems remain intentionally out of scope. |
| `ipaddress` constructor parity for strict CPython-style `IPv4Address(...)` raising | `intentional-diff` | Factory APIs (`ip_address`, `ipv4_address`) carry typed error behavior; direct constructor remains non-raising under current Sifr constructor lowering. |
| `ipaddress` IPv6 constructors/networks/interfaces and mixed-family parsing | `unsupported` | Current shipped surface is explicitly IPv4-focused; IPv6 class families are not shipped in this runtime. |
| `uuid` generation families beyond shipped subset (`uuid1`, `uuid6/7/8`) | `unsupported` | Runtime now closes deterministic v3/v4/v5 generation and namespace constants; time-ordered or host-identity variants remain intentionally unshipped. |
| `uuid` strict CPython constructor overload family (`UUID(...)` with raising validation and multi-source overloads) | `intentional-diff` | `uuid_from_hex` carries typed parse/validation behavior; raw `UUID(...)` remains pass-through in this implementation pass. |
| `graphlib` full CPython incremental multi-node frontier semantics | `intentional-diff` | Current typed API uses deterministic one-node `get_ready()` progression and explicit `done(node)` sequencing without dynamic hashable-node generality. |
| `test` as CPython-equivalent public stdlib module | `intentional-diff` | `sifr.test` is compiler/runtime verification infrastructure and is not claimed as one-to-one CPython module parity. |

## Structured/Class-Surface Continuation Readiness (2026-03-18)

- Continuation capability: `structured-data-and-class-surface-parity-expansion record`
- Capability ownership:
  - `stdlib_parity_struct_2` for bounded `argparse` expansion (completed),
  - `stdlib_parity_struct_3` for `uuid` typed generation/namespace expansion (completed).
- Closed in continuation:
  - `argparse` `subparsers` support with deterministic namespace merge behavior.
  - bounded `nargs` support (`int`, `?`, `*`, `+`) and typed conversion for `str`/`int`/`float`/`bool`.
  - `uuid` typed generation (`uuid3`, `uuid5`) and namespace constant accessors (`NAMESPACE_DNS`, `NAMESPACE_URL`, `NAMESPACE_OID`, `NAMESPACE_X500`).
  - CPython-derived continuation fixture: `crates/sifr/tests/e2e/pass/counter_defaultdict_and_argparse.sifr`
  - CPython-derived continuation fixture: `crates/sifr/tests/e2e/pass/uuid_and_datetime.sifr`
- Locked permanent diffs carried into continuation:
  - `argparse` formatter-class/help-formatting ecosystems remain `unsupported`,
  - strict raising direct `UUID(...)` constructor parity remains `intentional-diff` until constructor-lowering architecture changes.
- Enforcement fixture: `crates/sifr/tests/e2e/fail/argparse_formatter_class_unsupported.sifr`
