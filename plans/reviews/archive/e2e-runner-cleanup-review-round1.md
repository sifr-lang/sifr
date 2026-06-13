

## Review Findings

### Blocking Issues

**1. Scripts export dead env var `SIFR_E2E_RUNNER_MODE`**

The Rust code no longer reads `SIFR_E2E_RUNNER_MODE` (it was removed from `RunnerConfig` and all parsing functions). The script still sets and exports it:

- `scripts/run_e2e_pass.sh:136` — `MODE="${SIFR_E2E_RUNNER_MODE:-${MODE}}"`
- `scripts/run_e2e_pass.sh:190` — `SIFR_E2E_RUNNER_MODE="${MODE}" \`
- `scripts/ci_e2e_throughput.sh:5` — `MODE="${SIFR_E2E_RUNNER_MODE:-new}"`
- `scripts/ci_e2e_throughput.sh:19` — `SIFR_E2E_RUNNER_MODE="$MODE"`

Additionally, `scripts/run_e2e_pass.sh:31` has a dead `--mode <legacy|new|compare>` option in its usage text and `:88` parses it into `MODE_OVERRIDE`, which then sets `SIFR_E2E_RUNNER_MODE` at line 145-147.

**Impact**: Environment pollution. Users running via these scripts will set an env var that has no effect on the harness. If someone later searches for where `SIFR_E2E_RUNNER_MODE` is used, they'll find script references that are dead code.

---

### Non-Blocking Observations

**2. Shell `MODE` variable is now vestigial**

In both scripts, `MODE` is set (line 136 in `run_e2e_pass.sh`, line 5 in `ci_e2e_throughput.sh`), exported, and echoed in the run summary, but since `SIFR_E2E_RUNNER_MODE` is no longer consumed by the Rust harness, `MODE` is a no-op. It can be removed entirely, or the scripts can be simplified to remove mode handling.

**3. Doc comment drift in `e2e.rs:9-12`**

The contract now describes parallelism controls (`SIFR_E2E_SIFR_JOBS`, `SIFR_E2E_RUST_JOBS`, `SIFR_E2E_RUN_JOBS`, `SIFR_E2E_CARGO_BUILD_JOBS`, `SIFR_E2E_DISABLE_CACHE`). These are correct and current — no action needed.

---

### What Was Done Well

The Rust-side cleanup is thorough and correct:

- `RunnerMode` enum removed — no dead enum variants remain
- `parse_runner_mode()`, `parse_runner_mode_from_env()`, `parse_optional_bool_env_value()` removed
- `run_legacy_pass_suite()` removed (the sequential-legacy path)
- `compare_pass_reports()` removed (the diff-against-legacy path)
- `mode` field removed from `RunnerConfig`
- `SIFR_E2E_RUNNER_MODE`, `SIFR_E2E_NEW_RUNNER`, `SIFR_E2E_LEGACY_RUNNER` removed from env allowlist
- `test_runner_mode_resolution()` test removed (the parsing test for the dead enum)
- `run_new_pass_suite` → `run_pass_suite` (clean, single-path name)
- "legacy" → "retired" renaming in error messages and test names is applied consistently

The test assertion path, report signature format, and failure aggregation semantics are unchanged — no cache correctness or validation impact.

---

### Verdict

**Not satisfied.** The scripts (`run_e2e_pass.sh`, `ci_e2e_throughput.sh`) must be updated to remove `SIFR_E2E_RUNNER_MODE` and `--mode` handling before this can be considered clean. Once that is done, the Rust diff is sound.
