# `stdlib_parity_e1` CPython Traceability

## Reviewed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_datetime.py` | `timedelta`, `datetime/date/time` formatting, timestamp conversion, and invalid timestamp handling | `crates/sifr/tests/e2e/pass/cpython_datetime_subset.sifr`<br>`crates/sifr/tests/e2e/pass/datetime_regex_math_and_hashing.sifr`<br>`crates/sifr/tests/e2e/pass/uuid_and_datetime.sifr`<br>`crates/sifr/tests/e2e/fail/datetime_from_timestamp_non_float.sifr` | adapted | Sifr keeps a lightweight class model and typed `ValueError` surfaces in place of exception-only control flow. `stdlib_parity_struct_3` closes the canonical fixed-offset timezone entry point (`UTC`), `now(tz)`, `from_timestamp(ts, tz)`, and `astimezone` while preserving deterministic typed behavior. |
| `Lib/test/test_re.py` | search/findall/split/sub/fullmatch behavior, flags, invalid pattern rejection, and iterator surfaces (`finditer`, `Pattern.finditer`) | `crates/sifr/tests/e2e/pass/cpython_re_subset.sifr`<br>`crates/sifr/tests/e2e/pass/datetime_regex_math_and_hashing.sifr`<br>`crates/sifr/tests/e2e/pass/regex_filesystem_iterators.sifr`<br>`crates/sifr/tests/e2e/fail/re_search_non_string_pattern.sifr` | adapted | Public aliases and flag constants are Python-shaped while keeping typed regex-error handling. `finditer` and `Pattern.finditer` now expose iterator-returning `Match` streams with explicit materialization boundaries where reusable collections are needed. |
| `Lib/test/test_math.py` | combinatorics helpers, floating-point tolerance semantics, and numeric edge behavior | `crates/sifr/tests/e2e/pass/cpython_math_missing_surface_subset.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_math_semantic_corrections_subset.sifr`<br>`crates/sifr/tests/e2e/pass/datetime_regex_math_and_hashing.sifr`<br>`crates/sifr/tests/e2e/fail/math_isclose_non_float_tol.sifr` | adapted | Broad scalar parity is closed; return-shape differences from tuple/object-heavy CPython helpers remain explicit. Invalid combinatorics domains currently use deterministic `0` adaptation (`factorial(-1)`, `comb(5,10)`, `perm(5,10)`) and are asserted in CPython-derived fixtures. |
| `Lib/test/test_statistics.py` | mean/median/variance families, regression/correlation helpers, and invalid-input error behavior | `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`<br>`crates/sifr/tests/e2e/pass/datetime_regex_math_and_hashing.sifr`<br>`crates/sifr/tests/e2e/fail/statistics_mean_non_float_list.sifr` | adapted | Sifr keeps `Result[..., StatisticsError]` for numeric-domain failures and compile-time type rejection for invalid container element types; `stdlib_parity_rng_3` extends this family with `median_grouped` deterministic coverage. |
| `Lib/test/test_hashlib.py` | hash object construction/update/hexdigest, available algorithm inventory, and unsupported algorithm behavior | `crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`<br>`crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr`<br>`crates/sifr/tests/e2e/pass/datetime_regex_math_and_hashing.sifr`<br>`crates/sifr/tests/e2e/fail/hashlib_new_non_string_name.sifr` | adapted | Hash-object parity is closed for shipped algorithms with explicit typed errors for unsupported algorithm names. Bytes-native digest/object expansion is now governed by `stdlib_parity_rng_2`. |

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| Full timezone-aware/calendar object parity in `datetime` (fold/zoneinfo/tz database semantics) | `unsupported` | Current runtime closes fixed-offset timezone helpers only and intentionally does not ship `zoneinfo`, DST folding, or extensible `tzinfo` ecosystems. |
| Full `re` Match/Pattern object matrix (named groups, groupdict, and full capture-object APIs) | `unsupported` | Current `Match`/`Pattern` surfaces close high-value entry points (including `finditer`) but do not mirror every CPython capture-object API. |
| Decimal/Fraction-specific and context-sensitive numeric semantics in `math` / `statistics` | `unsupported` | Capability e1 targets shipped float/int behavior and typed error ruless, not CPython decimal-context integration. |
| SHA3/SHAKE constructor families in `hashlib` | `unsupported` | Runtime currently closes the guaranteed algorithm set and explicit placeholders raise typed errors for unsupported SHA3/SHAKE families; bytes-native digest/object APIs are now shipped by `stdlib_parity_rng_2`. |

## Structured/Class-Surface Continuation Readiness (2026-03-18)

- Continuation capability: `structured-data-and-class-surface-parity-expansion record`
- Capability ownership: `stdlib_parity_struct_3` expanded `datetime` under fixed-offset timezone semantics only (completed).
- Closed in continuation:
  - the fixed-offset `UTC` constructor and timezone-aware conversion entry points for `now`, `from_timestamp`, and `datetime.astimezone`.
  - continuation fixture: `crates/sifr/tests/e2e/pass/uuid_and_datetime.sifr`
- Locked permanent diff carried into continuation:
  - timezone-database / extensible `tzinfo` ecosystems remain explicitly `unsupported`.
- Enforcement fixture: `crates/sifr/tests/e2e/fail/datetime_tzinfo_zoneinfo_unsupported.sifr`
