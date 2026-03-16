# `wave_psp_e2` CPython Traceability

## Reviewed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_argparse.py` | parser construction, option/default handling, positional binding, and boolean flag action | `crates/sifr/tests/e2e/pass/cpython_argparse_subset.sifr`<br>`crates/sifr/tests/e2e/pass/phase_psp_e2_class_heavy_custom_cleanup.sifr`<br>`crates/sifr/tests/e2e/fail/phase_psp_e2_argparse_parse_args_non_string_list.sifr`<br>`demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr` | adapted | `ArgumentParser`/`Namespace` class model is closed for typed Sifr usage. Dynamic attribute mutation and full CLI error/reporting behavior remain explicit waivers. |
| `Lib/test/test_ipaddress.py` | IPv4 validation, parsing, classification helpers, and constructor/error behavior | `crates/sifr/tests/e2e/pass/cpython_ipaddress_subset.sifr`<br>`crates/sifr/tests/e2e/pass/phase_psp_e2_class_heavy_custom_cleanup.sifr`<br>`crates/sifr/tests/e2e/fail/phase_psp_e2_ip_address_non_string.sifr`<br>`demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr` | adapted | `IPv4Address` class/object behavior and `AddressValueError`-typed construction are closed; IPv6-heavy families remain unsupported in this wave. |
| `Lib/test/test_uuid.py` | UUID v4 generation, parse/validation behavior, class properties, and canonical text shape | `crates/sifr/tests/e2e/pass/cpython_uuid_subset.sifr`<br>`crates/sifr/tests/e2e/pass/stdlib_uuid_consolidated.sifr`<br>`crates/sifr/tests/e2e/pass/phase_psp_e2_class_heavy_custom_cleanup.sifr`<br>`crates/sifr/tests/e2e/fail/phase_psp_e2_uuid_from_hex_non_string.sifr` | adapted | UUID object-model parity for shipped v4 + parse surfaces is preserved; non-shipped UUID families stay explicitly classified. |
| `Lib/test/test_graphlib.py` | `TopologicalSorter` DAG behavior, static ordering, incremental readiness flow, and cycle errors | `crates/sifr/tests/e2e/pass/cpython_graphlib_subset.sifr`<br>`crates/sifr/tests/e2e/pass/phase_psp_e2_class_heavy_custom_cleanup.sifr`<br>`crates/sifr/tests/e2e/fail/phase_psp_e2_graphlib_add_non_int_predecessor.sifr`<br>`demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr` | adapted | `TopologicalSorter` object methods (`add`, `add_many`, `prepare`, `get_ready`, `done`, `is_active`, `static_order`) are closed for typed int-node graphs. |
| `Lib/test/test_unittest.py` (assertion semantics used as proxy for `sifr.test`) | assertion helpers and typed comparison contracts | `crates/sifr/tests/e2e/pass/cpython_unittest_assertions_subset.sifr`<br>`crates/sifr/tests/e2e/pass/phase_psp_e2_class_heavy_custom_cleanup.sifr` | adapted | `test` is Sifr infrastructure rather than CPython stdlib parity target; assertion hardness is mapped via a `unittest`-style subset and classified below. |

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| `argparse` advanced parser features (`subparsers`, `nargs` matrices, help rendering, formatter classes) | `unsupported` | Wave e2 closes typed class/object fundamentals; dynamic CLI formatting/reporting matrix is intentionally out of scope. |
| `ipaddress` IPv6 constructors/networks/interfaces and mixed-family parsing | `unsupported` | Current shipped surface is explicitly IPv4-focused; IPv6 class families are not shipped in this runtime. |
| `uuid` non-v4 generation families (`uuid1`, `uuid3`, `uuid5`, `uuid6/7/8`) | `unsupported` | Runtime exposes deterministic v4 generation + canonical parse/object behavior only. |
| `graphlib` full CPython incremental multi-node frontier semantics | `intentional-diff` | Current typed API returns deterministic ready batches and explicit `done(node)` progression without dynamic hashable-node generality. |
| `test` as CPython-equivalent public stdlib module | `intentional-diff` | `sifr.test` is compiler/runtime verification infrastructure and is not claimed as one-to-one CPython module parity. |
