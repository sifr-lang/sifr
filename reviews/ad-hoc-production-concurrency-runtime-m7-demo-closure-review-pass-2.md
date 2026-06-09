VERDICT: PASS

## Findings

### Validation evidence (pass-1 soft gap) — ✅
The ledger (`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:1524-1539`) now records the additional validation commands flagged in pass-1:

- `cargo fmt --check` -> PASS
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS
- `cargo test -p sifr -- --skip test_e2e_pass` -> PASS (CLI unit `95 passed`; e2e harness `36 passed`, `1 filtered out`)
- `scripts/run_all_tests.sh --profile create-pr` -> PASS with exact metrics: `wall_time=417.71s`, `budget_ok=no` (warm wall-time advisory), `max_rss=518.6MiB`, `swaps=0`, platform golden `pass=6 skip=1`, e2e `125 passed`, `0 failed`, `cache_hits=0/37`, `report_signature=50edc954137c87b4`, slowest step `e2e_pass_suite 279192ms`, report `target/validation_lane_reports/create-pr.latest.json`.

These cover the project's authoritative create-pr gate and the traceability validation plan.

### Inherited clippy failure — ✅ recorded honestly
Line 1539: `cargo clippy --workspace -- -D warnings` -> FAIL on `crates/sifr_codegen/src/intrinsics/registry/runtime.rs`, `crates/sifr_codegen/src/preamble/process_async_child_runtime.rs`, `crates/sifr_codegen/src/preamble/process_async_runtime.rs`. I reproduced clippy and confirmed errors at exactly those file paths:

- `crates/sifr_codegen/src/intrinsics/registry/runtime.rs:14:26` (uninlined-format-args)
- `crates/sifr_codegen/src/preamble/process_async_child_runtime.rs:57:1` (fn_params_excessive_bools)
- `crates/sifr_codegen/src/preamble/process_async_runtime.rs:231:1` (too_many_arguments + fn_params_excessive_bools)

`git log` confirms these files were last touched by M5 work (`a13950d34`, `134963a2b`, `df95159dd`), not by this demo slice. The ledger explicitly notes "this demo slice does not touch those files, and the M7 generated dependency/panic-scan quality slice retains final clippy cleanup before phase closeout" — the failure is scoped outside the demo PR without hiding that M7 phase closeout must fix it.

### git diff --check and file-size guardrail (req 3) — ✅ accurate
Re-ran both: `git diff --check` PASS; `python3 scripts/check_file_size_guardrails.py` -> PASS, `2271 files`, `900` line limit. Ledger matches.

### Gate states (req 4) — ✅
- `issues/...execution.md:480-481`: `- M7 demo closure: pending PR.` / `- M7: in progress.`
- `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md:20-25`: Required demos `pending PR`; Generated Cargo dependency snapshots `open`; Panic scan and emitted-code quality coverage `open`; Validation lane manifests `partial`; Inventory closure `open`; Final external review `open`.
- Slice rows (lines 43-49): scaffold/docs/architecture `complete`; Demo closure `pending PR`; Generated dependency and panic-scan `pending`; Validation lane and inventory closure `pending`; Final review and merge gate `pending`.

Pass-1 minor terminology inconsistency (`pending-pr` vs `pending PR`) is resolved — both gate and slice rows now use `pending PR`.

### Demo topic coverage and no-overclaim (req 5) — ✅
All seven required M7 demo topics still covered without CPython-shaped compatibility surfaces:

- Structured task group: `demos/structured_concurrency_demo/main.sifr` (`task.scope`, `task.gather`, `task.select`, `task.TaskGroup`).
- Producer/consumer channel pipeline: `demos/sync_channel_demo/main.sifr` (`bounded_channel` + drain consumer).
- Blocking offload: `demos/blocking_offload_demo/main.sifr` (`task.spawn_blocking`, `@cpu_heavy`, `@blocking_io`).
- CPU parallel map: `demos/parallel_map_demo/main.sifr` (`sifr.parallel.map` + `WorkerRuntimeError`).
- Async subprocess pipeline: `demos/process_shutdown_cleanup_demo/main.sifr:34-53` (`async_spawn`, owned `AsyncPipeReader`/`AsyncPipeWriter`, `child.wait`).
- Structured shutdown shape: `demos/process_shutdown_cleanup_demo/main.sifr:22-31` (annotates `Awaitable[Result[Signal, SignalError]]` from `ctrl_c()`/`terminate()`/`shutdown_stream().next()` without awaiting; `_`-prefixed bindings; `strsignal` value-shape only — no actual signal-delivery claim).
- Cleanup under cancellation: `demos/process_shutdown_cleanup_demo/main.sifr:56-82` (`task.timeout(0.0)` + `try/finally` writing a PID-scoped marker, then asserting cleanup ran).

Imports are all production surfaces (`sifr.parallel`, `sifr.process`, `sifr.signal`, `sifr.io`, `sifr.os`) — no CPython-shaped names. Re-ran both new demos: PASS.

### No new blockers
No CPython-shaped surfaces introduced; no signal-delivery overclaim; demo closure remains pending PR; M7 remains open/in progress; remaining gates remain open or pending; honest, scope-bounded record of the inherited clippy debt.

## Conclusion
The pass-1 soft gap is resolved. The slice is ready to open as a PR with the recorded validation block.
