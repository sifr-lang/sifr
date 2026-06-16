# stdlib_parity_runtime_3 CPython Traceability Matrix

Wave: `stdlib_parity_runtime_3`
Scope: `logging`, `time`, and `timeit` object-surface expansion

## CPython Harvest Inputs

- `Lib/test/test_logging.py` (logger/handler hierarchy, formatter, constants, safety behavior)
- `Lib/test/test_time.py` (`struct_time`, `gmtime/localtime`, `mktime`, timezone constants)
- `Lib/test/test_timeit.py` (`default_timer`, callable statement timing, repeat/timer object behavior)

## Adopt / Adapt / Waive (Wave 3)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_logging` logger core (`Logger`, level controls, constants, `basicConfig`, `getLogger`) | ship deterministic single-process logger object model with explicit level filtering and sink control | `adapted` | `crates/sifr/tests/e2e/pass/logging_time_and_timers.sifr`, `crates/sifr/tests/e2e/pass/stdlib_logging_consolidated.sifr` |
| `test_logging` handler classes (`Handler`, `StreamHandler`, `FileHandler`, `NullHandler`, `Formatter`) | ship class surfaces and deterministic handler gating under synchronous host model | `adapted` | `crates/sifr/tests/e2e/pass/cpython_logging_subset.sifr`, `demos/logging_and_timers/main.sifr` |
| `test_time` structured clock objects (`struct_time`, `mktime`, timezone constants) | ship immutable `struct_time` and explicit struct wrappers (`gmtime_struct`, `localtime_struct`) over intrinsic ISO clock strings | `adapted` | `crates/sifr/tests/e2e/pass/cpython_time_subset.sifr`, `crates/sifr/tests/e2e/pass/stdlib_time_consolidated.sifr` |
| `test_timeit` callable timing API (`default_timer`, `timeit`, `repeat`, `Timer`) | ship callable-only timer model including `Timer.timeit`, `Timer.repeat`, and `Timer.__call__` | `adapted` | `crates/sifr/tests/e2e/pass/cpython_timeit_subset.sifr`, `crates/sifr/tests/e2e/pass/stdlib_timeit_consolidated.sifr` |
| logging graph configuration (`dictConfig`, `LoggerAdapter`) | keep explicitly unsupported (locked from wave 0) | `unsupported` | `crates/sifr/tests/e2e/fail/logging_dictconfig_unsupported.sifr`, `crates/sifr/tests/e2e/fail/logging_loggeradapter_unsupported.sifr` |
| timeit string-eval statements | keep explicitly unsupported; callable-only execution model remains enforced | `unsupported` | `crates/sifr/tests/e2e/fail/timeit_string_eval_unsupported.sifr` |
| timezone mutation helpers (`tzset`, mutable timezone env surfaces) | keep explicitly unsupported; stable constants only | `unsupported` | `crates/sifr/tests/e2e/fail/timezone_mutation_unsupported.sifr` |

## Explicit Waivers / Boundaries (Wave 3)

- Logging remains deterministic single-process/synchronous and does not claim thread-order parity.
- Logger handler wiring intentionally keeps one active sink mode at a time (`file`, `stream`, or `null`) to preserve deterministic host behavior.
- File-backed logging stays fail-soft on host I/O failures (errors are suppressed rather than escalated to user panics).
- `sifr.time.gmtime` and `sifr.time.localtime` remain intrinsic ISO-string surfaces; structured object parity is exposed via `gmtime_struct` and `localtime_struct`.
- `timeit` remains callable-only and rejects CPython string-eval execution.
- Timezone mutation helpers remain out-of-scope and explicitly rejected by fail fixtures.

## Local Fixture Anchors (Wave 3)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/logging_time_and_timers.sifr`
- Demo:
  - `demos/logging_and_timers/main.sifr`
- Consolidated/CPython regressions:
  - `crates/sifr/tests/e2e/pass/stdlib_logging_consolidated.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_logging_subset.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_time_consolidated.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_time_subset.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_timeit_consolidated.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_timeit_subset.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/logging_dictconfig_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/logging_loggeradapter_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/timeit_string_eval_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/timezone_mutation_unsupported.sifr`
