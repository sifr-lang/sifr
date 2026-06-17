# `stdlib_parity_struct_0` Architecture Lock (Structured Data and Class-Surface Parity Expansion)

Capability: `issues/structured-data-and-class-surface-parity-expansion.md`
Execution ledger: `issues/structured-data-and-class-surface-parity-expansion-execution.md`

## Objective

Lock public ruless, permanent waivers, and CPython-family mapping for the structured-data/class-surface continuation capability before feature-expansion implementation passes begin.

## Locked Public Rules Snapshot

| Module | Locked direction for this capability |
| --- | --- |
| `json` | Keep typed top-level entry points; add `JSONEncoder`/`JSONDecoder` typed wrappers; do not add dynamic callback-hook matrices. |
| `configparser` | Expand interpolation/proxy/write-back parity deliberately with explicit object-model bounds. |
| `csv` | Keep iterator-returning reader APIs; add bounded process-local dialect registry with immutable dialect values. |
| `collections` | Expand `Counter(iterable)` and `Counter(mapping)` constructor parity; keep `Counter(**kwargs)` out of scope; promote `defaultdict` toward explicit class/object semantics. |
| `argparse` | Expand `subparsers`, bounded `nargs` matrix (`int`, `?`, `*`, `+`), and typed `type=` coercers under deterministic behavior. |
| `uuid` | Add `uuid3`, `uuid5`, and namespace constants; keep strict raising direct `UUID(...)` constructor parity as intentional diff unless constructor-lowering architecture closes first. |
| `datetime` | Fixed-offset timezone only in this capability; no zone database / DST / `fold`; `timezone` remains the only timezone implementation. |
| `textwrap` | Expand `TextWrapper` through explicit adjacent option fields; no open-ended formatter ecosystem claims. |
| `html` | Keep scope bounded to top-level module polish (`escape` / `unescape` family), no package-wide expansion (`html.parser`, etc.). |

## Permanent Sifr-Safe Diffs (Locked for This Capability)

| Surface | Classification | Enforcement fixture |
| --- | --- | --- |
| Dynamic JSON callback hooks (`object_hook`, `object_pairs_hook`, `parse_float`, `parse_int`, `parse_constant`, `default`) | `unsupported` | `crates/sifr/tests/e2e/fail/json_dynamic_hooks_unsupported.sifr` |
| Timezone database and extensible `tzinfo` ecosystems (`zoneinfo`, DST/fold model) | `unsupported` | `crates/sifr/tests/e2e/fail/datetime_tzinfo_zoneinfo_unsupported.sifr` |
| `Counter(**kwargs)` constructor parity | `unsupported` | `crates/sifr/tests/e2e/fail/counter_kwargs_constructor_unsupported.sifr` |
| Dynamic CSV dialect subclass registration/mutation-heavy registry semantics | `unsupported` | `crates/sifr/tests/e2e/fail/csv_dynamic_registry_unsupported.sifr` |
| `argparse` formatter-class/help-formatting ecosystems | `unsupported` | `crates/sifr/tests/e2e/fail/argparse_formatter_class_unsupported.sifr` |
| Package-wide `html` expansion (`html.parser` family) | `unsupported` | `crates/sifr/tests/e2e/fail/html_package_parser_unsupported.sifr` |

## CPython Family Mapping (Capability Ownership)

| CPython family | Module | Direction | Execution capability | Local fixture anchor |
| --- | --- | --- | --- | --- |
| `Lib/test/test_json/` | `json` | `adapted` | `stdlib_parity_struct_1` | `crates/sifr/tests/e2e/pass/cpython_json_subset.sifr`, `crates/sifr/tests/e2e/pass/parsers_and_encoders.sifr` (new) |
| `Lib/test/test_configparser.py` | `configparser` | `adapted` | `stdlib_parity_struct_1` | `crates/sifr/tests/e2e/pass/cpython_configparser_subset.sifr`, `crates/sifr/tests/e2e/pass/parsers_and_encoders.sifr` (new) |
| `Lib/test/test_csv.py` | `csv` | `adapted` | `stdlib_parity_struct_1` | `crates/sifr/tests/e2e/pass/cpython_csv_subset.sifr`, `crates/sifr/tests/e2e/pass/parsers_and_encoders.sifr` (new) |
| `Lib/test/test_collections.py` | `collections` | `adapted` | `stdlib_parity_struct_2` | `crates/sifr/tests/e2e/pass/ordered_collections.sifr`, `crates/sifr/tests/e2e/pass/counter_defaultdict_and_argparse.sifr` (new) |
| `Lib/test/test_argparse.py` | `argparse` | `adapted` | `stdlib_parity_struct_2` | `crates/sifr/tests/e2e/pass/cpython_argparse_subset.sifr`, `crates/sifr/tests/e2e/pass/counter_defaultdict_and_argparse.sifr` (new) |
| `Lib/test/test_uuid.py` | `uuid` | `adapted` | `stdlib_parity_struct_3` | `crates/sifr/tests/e2e/pass/cpython_uuid_subset.sifr`, `crates/sifr/tests/e2e/pass/uuid_and_datetime.sifr` (new) |
| `Lib/test/test_datetime.py` | `datetime` | `adapted` | `stdlib_parity_struct_3` | `crates/sifr/tests/e2e/pass/cpython_datetime_subset.sifr`, `crates/sifr/tests/e2e/pass/uuid_and_datetime.sifr` (new) |
| `Lib/test/test_textwrap.py` | `textwrap` | `adapted` | `stdlib_parity_struct_4` | `crates/sifr/tests/e2e/pass/cpython_textwrap.sifr`, `crates/sifr/tests/e2e/pass/text_wrapping_and_html.sifr` (new) |
| `Lib/test/test_html.py` | `html` | `adopted`/`adapted` | `stdlib_parity_struct_4` | `crates/sifr/tests/e2e/pass/stdlib_html.sifr`, `crates/sifr/tests/e2e/pass/text_wrapping_and_html.sifr` (new) |

## Architecture-Lock Validation Fixtures (Capability 0)

- Positive path fixture: `crates/sifr/tests/e2e/pass/json_and_datetime.sifr`
- JSON wrapper-model demo: `demos/json_values/main.sifr`
- Fixed-offset datetime-model demo: `demos/fixed_timezones/main.sifr`
- Permanent-diff negative fixtures:
  - `crates/sifr/tests/e2e/fail/json_dynamic_hooks_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/datetime_tzinfo_zoneinfo_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/counter_kwargs_constructor_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/csv_dynamic_registry_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/argparse_formatter_class_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/html_package_parser_unsupported.sifr`
