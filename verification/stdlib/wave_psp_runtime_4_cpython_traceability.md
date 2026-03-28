# wave_psp_runtime_4 CPython Traceability Matrix

Wave: `wave_psp_runtime_4`  
Scope: synchronous `subprocess` boundary cleanup and governance closure

## CPython Harvest Inputs

- `Lib/test/test_subprocess.py` (sync run/check helpers, output capture behavior, constants, error handling)

## Adopt / Adapt / Waive (Wave 4)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| sync command execution (`run`, `CompletedProcess`, `stdout/stderr/returncode`) | preserve deterministic sync process execution with typed `CompletedProcess` return surface | `adapted` | `crates/sifr/tests/e2e/pass/phase_psp_runtime_4_subprocess_sync_boundary_governance.sifr`, `crates/sifr/tests/e2e/pass/cpython_subprocess_subset.sifr` |
| sync helper APIs (`check_call`, `check_output`) | ship explicit non-zero-exit rejection as typed `IOError` while preserving panic-free behavior | `adapted` | `crates/sifr/tests/e2e/pass/phase_psp_runtime_4_subprocess_sync_boundary_governance.sifr`, `crates/sifr/tests/e2e/pass/stdlib_subprocess.sifr` |
| subprocess constants (`PIPE`, `STDOUT`, `DEVNULL`) | ship stable constants for sync option-matrix parity anchors | `adapted` | `crates/sifr/tests/e2e/pass/phase_psp_runtime_4_subprocess_sync_boundary_governance.sifr`, `demos/runtime_subprocess_sync_boundary_governance/main.sifr` |
| non-string command inputs | keep compile-time type rejection for process command boundaries | `adapted` | `crates/sifr/tests/e2e/fail/phase_psp_d2_subprocess_non_string_cmd.sifr` |
| async lifecycle/process orchestration (`Popen`, signal/process-group controls, full option matrix) | keep explicitly unsupported and outside this phase | `unsupported` | `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_async_popen_unsupported.sifr` |

## Explicit Waivers / Boundaries (Wave 4)

- Async `subprocess.Popen` lifecycle and full process orchestration remain explicitly unsupported.
- `check_call` and `check_output` are intentionally typed wrappers over sync `run` semantics and raise `IOError` on non-zero exit status.
- The subprocess surface remains command-string based (no full CPython argv/list invocation matrix in this phase).

## Local Fixture Anchors (Wave 4)

- Positive fixture:
  - `crates/sifr/tests/e2e/pass/phase_psp_runtime_4_subprocess_sync_boundary_governance.sifr`
- Demo:
  - `demos/runtime_subprocess_sync_boundary_governance/main.sifr`
- Consolidated/CPython regressions:
  - `crates/sifr/tests/e2e/pass/cpython_subprocess_subset.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_subprocess.sifr`
  - `crates/sifr/tests/e2e/pass/phase_psp_d2_process_runtime_platform.sifr`
- Negative fixtures:
  - `crates/sifr/tests/e2e/fail/phase_psp_d2_subprocess_non_string_cmd.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_runtime_0_async_popen_unsupported.sifr`
