# `wave_psp_d2` CPython Traceability

## Reviewed upstream families

| CPython family | Surface | Local regression/demo | State | Notes |
| --- | --- | --- | --- | --- |
| `Lib/test/test_os/` | runtime command/process helpers, cwd, directory/file mutation helpers, stat-like checks, locator utilities | `crates/sifr/tests/e2e/pass/cpython_os_subset.sifr`<br>`crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr`<br>`crates/sifr/tests/e2e/fail/os_mkdir_non_string_path.sifr` | adapted | Sifr keeps a focused `os` surface with typed `IOError` results while preserving high-value process/filesystem helpers. |
| `Lib/test/test_subprocess.py` | process execution result surface (`run`, return code/stdout/stderr), stdin forwarding, and error handling | `crates/sifr/tests/e2e/pass/cpython_subprocess_subset.sifr`<br>`crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr`<br>`crates/sifr/tests/e2e/fail/subprocess_non_string_cmd.sifr` | adapted | `CompletedProcess` parity is preserved for sync command execution; unsupported options are explicitly waived below. |
| `Lib/test/test_sys.py` | argv/version/platform/maxsize introspection and process exit type safety | `crates/sifr/tests/e2e/pass/cpython_sys_subset.sifr`<br>`crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr`<br>`crates/sifr/tests/e2e/fail/sys_exit_non_int_code.sifr` | adapted | Sifr surfaces the stable metadata/introspection subset and keeps typed exit-code boundaries. |
| `Lib/test/test_logging.py` | logger level filtering, file emission, formatter/handler helpers, and missing-path safety | `crates/sifr/tests/e2e/pass/cpython_logging_subset.sifr`<br>`crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr` | adapted | Logging remains intentionally lightweight and synchronous while preserving Python-shaped naming and level semantics. |
| `Lib/test/test_platform.py` | system/machine/node/release/version/processor host identity helpers | `crates/sifr/tests/e2e/pass/cpython_platform_subset.sifr`<br>`crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr` | adapted | Host probe helpers are direct and deterministic with alias coverage. |
| `Lib/test/test_time.py` | wall clock access, format/parse helpers, monotonic/perf counters, and invalid parse behavior | `crates/sifr/tests/e2e/pass/cpython_time_subset.sifr`<br>`crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr` | adapted | Parsing errors map to typed `ValueError` and out-of-range runtime paths are kept panic-free. |
| `Lib/test/test_timeit.py` | default timer, statement timing loops, repeat counts, and edge-count behavior | `crates/sifr/tests/e2e/pass/cpython_timeit_subset.sifr`<br>`crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr`<br>`crates/sifr/tests/e2e/fail/timeit_non_callable_stmt.sifr` | adapted | Callable-based `timeit/repeat` parity is preserved with non-negative elapsed guarantees. |
| `Lib/test/test_os.py` + environment-related families | environment read/write/unset and enumeration helpers | `crates/sifr/tests/e2e/pass/cpython_env_subset.sifr`<br>`crates/sifr/tests/e2e/pass/process_runtime_and_platform.sifr` | adapted | `env` remains a classified custom bridge surface mapped to CPython environment semantics. |

## Classified waivers

| Surface | State | Rationale |
| --- | --- | --- |
| `subprocess.Popen` async lifecycle and full argument matrix (`shell`, `cwd`, `env`, pipes object model, signals) | `unsupported` | Current wave closes synchronous process execution via `run`/`run_raw`/`run_with_input` and `CompletedProcess` only. |
| Full CPython `sys` mutable/global runtime hooks (`settrace`, `setprofile`, import-system mutation hooks, recursion/thread controls) | `unsupported` | Sifr exposes deterministic introspection + exit helpers but does not mirror Python interpreter mutation hooks. |
| CPython `logging` hierarchy/config APIs (`dictConfig`, handler trees, propagation graph semantics, filters module parity) | `unsupported` | Current logging surface remains intentionally lightweight while preserving core logger/level/handler use cases. |
| Rich `time`/`timeit` object model parity (`struct_time`, timezone mutation helpers, Timer class/string-statement execution model) | `unsupported` | Current wave keeps safe functional timing helpers and avoids dynamic eval/string execution models. |
