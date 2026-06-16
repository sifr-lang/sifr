# `stdlib_parity_c1` CPython Traceability

CPython family references below indicate upstream source families used for adapted subset porting in this wave; they are not claims of full family parity.

## Reviewed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_json/test_fail.py`, `Lib/test/test_json/test_recursion.py`, `Lib/test/test_json/test_unicode.py`, `Lib/test/test_json/test_encode_basestring_ascii.py` | `json.loads`, `json.load`, `json.dumps`, `json.dump`, structured object/array access, ordered object emission, and typed decode failures | `crates/sifr/tests/e2e/pass/cpython_json_subset.sifr`, `crates/sifr/tests/e2e/pass/structured_data_formats.sifr`, `demos/structured_parsing_serialization/main.sifr` | adapted | Sifr now exposes structured `JsonValue` trees instead of raw dynamic Python objects. Decode failures remain typed `JSONDecodeError` results, and object insertion order is preserved for `items()`/`keys()` and serialized output; coverage is intentionally subset-scoped to the shipped typed surface. |
| `Lib/test/test_tomllib/test_data.py`, `Lib/test/test_tomllib/test_error.py`, `Lib/test/test_tomllib/test_misc.py` | `tomllib.loads`, `tomllib.load`, nested table/array access, datetime/string/bool/int classification, and typed parse failures | `crates/sifr/tests/e2e/pass/cpython_tomllib_subset.sifr`, `crates/sifr/tests/e2e/pass/structured_data_formats.sifr`, `demos/structured_parsing_serialization/main.sifr` | adapted | Sifr closes the string-adapter gap with structured `TomlValue` trees. The CPython `parse_float=` customization surface remains outside this wave because Sifr does not expose dynamic callback-driven decoding hooks here; coverage is intentionally subset-scoped rather than full-family exhaustive. |
| `Lib/test/test_csv.py` | `csv.Dialect`, `csv.reader`, `csv.writer`, `csv.DictReader`, `csv.DictWriter`, quoting constants, quoted field parsing, and file/object round-trips | `crates/sifr/tests/e2e/pass/structured_data_formats.sifr`, existing `crates/sifr/tests/e2e/pass/cpython_csv_subset.sifr` | adapted | Sifr keeps eager materialized row collections rather than CPython iterator objects, but the main reader/writer/dialect/constants surface now follows the natural CPython-shaped entry points instead of helper-only wrappers. |
| `Lib/test/test_configparser.py` | `ConfigParser`, `RawConfigParser`, `DEFAULTSECT`, parser defaults, strict duplicate-section handling, option lookups, converters, and mutation helpers | `crates/sifr/tests/e2e/pass/cpython_configparser_subset.sifr`, `crates/sifr/tests/e2e/pass/structured_data_formats.sifr`, existing `crates/sifr/tests/e2e/pass/stdlib_configparser.sifr` | adapted | Sifr closes the prior helper-thin parser surface with a real `ConfigParser` object model and exported error/constant types. Interpolation and converter-registration families remain outside this wave. |

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| `json` encoder/decoder hook families such as `default=`, `object_hook`, `object_pairs_hook`, `parse_float`, `parse_int`, `parse_constant`, pretty-print indentation, and CLI tooling | `unsupported` | These families require dynamic callback injection or formatting controls that are outside the current typed structured-value surface closed in this wave. |
| `json.dumps(...)` CPython-style encoder error propagation (`JSONEncodeError` on serialization failure) | `intentional-diff` | The closed C1 surface accepts typed `JsonValue` inputs and keeps panic-free emission semantics. Internal serialization failures currently fall back to `"null"` rather than surfacing a dynamic encode exception contract. |
| `tomllib.load(fp, parse_float=...)` and callback-based numeric customization | `unsupported` | Sifr does not expose CPython's dynamic parse-hook customization model for TOML decoding. |
| `tomllib` decode-position fidelity (`TOMLDecodeError.line` / `column`) | `intentional-diff` | Parse failures currently report typed decode errors with stable message semantics, while line/column metadata remains coarse (default `0/0`) in this wave. |
| Lazy streaming behavior for `csv.reader`/`csv.DictReader` and the dialect registry API from `test_csv.py` | `unsupported` | Sifr keeps eager row materialization and direct `Dialect(...)` construction rather than a global registry and iterator-based reader lifecycle. |
| `configparser` interpolation families, converter registration, mapping proxy protocols, and write-back formatting parity | `unsupported` | This wave closes the core parser/object/error surface only; the richer interpolation and proxy model would require additional class and callback infrastructure. |

## Structured/Class-Surface Continuation Lock (2026-03-18)

- Continuation phase: `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
- Wave ownership: `stdlib_parity_struct_1` expands `json`/`configparser`/`csv` under bounded typed contracts.
- Locked permanent diffs carried into continuation:
  - dynamic `json` callback hooks remain `unsupported` in this continuation tranche,
  - mutation-heavy csv dialect subclass registration remains `unsupported` in this continuation tranche.
- Enforcement fixtures:
  - `crates/sifr/tests/e2e/fail/json_dynamic_hooks_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/csv_dynamic_registry_unsupported.sifr`
