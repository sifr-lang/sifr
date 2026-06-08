All checks verified. Summary:

**Fixture honesty** — `crates/sifr/tests/e2e/pass/signal_stream_delivery_unix.sifr:1-62`
- Uses only existing public APIs: `ctrl_c()`, `terminate()`, `shutdown_stream().next()`, `strsignal()`, `Signal`, `SignalError`, `async_spawn`, `Command`, `AsyncChild`, `ProcessError`, `Status`, `getpid`, `system`. No new runtime API added.
- Each helper spawns a delayed child shell (`sleep 0.10; kill -<SIG> <pid>`) then awaits the corresponding signal, asserts `strsignal(received)` matches `"SIGINT"`/`"SIGTERM"`, then `await child.wait()` and asserts `status.success` — the child is always waited on regardless of inner SignalError handling.
- Windows-gated at `signal_stream_delivery_unix.sifr:56-57` (`if system() == "Windows": return`) — does not claim Windows signal delivery.
- No use of Unix-only constants (e.g., SIGHUP); no non-Unix semantics claimed.

**Lowering parity** — `crates/sifr_codegen/src/intrinsics/registry/signal.rs:9-72` confirms `ctrl_c()` uses `tokio::signal::ctrl_c()`, `terminate()` uses `tokio::signal::unix::SignalKind::terminate` under `#[cfg(unix)]` (typed `SignalError` on non-Unix), and `shutdown_stream()` selects between them on Unix and waits Ctrl-C only on non-Unix.

**Traceability & host matrix** — `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md` and `verification/platform/supported_host_matrix.md` row updates are honest:
- `ctrl_c` / `terminate` / `shutdown_stream().next()` rows: `supported / supported / host-limited` (macOS/Linux/Windows) with text citing `signal_stream_delivery_unix`.
- Unix-only constants (`SIGHUP`) row remains `host-limited` across all hosts.
- Umbrella row text explicitly notes "Unix-only constants and non-Unix delivery semantics remain host-limited follow-up."
- Status sentence in traceability updated to add "deterministic Unix signal-delivery evidence covers child-sent Ctrl-C, SIGTERM, and shutdown-stream waits" while keeping cleanup stacks, context propagation, and diagnostics as remaining M5 work.

**Issues ledger** — `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:740-752` records the wave with accurate scope and validation metrics matching the user's expected report.

**Targeted validation re-run**:
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/signal_stream_delivery_unix.sifr` → PASS (cache hit).
- `python3 -m json.tool` on both manifests → VALID JSON.
- `python3 scripts/check_file_size_guardrails.py` → PASS (2240 files under 900 lines).
- `git diff --check` → PASS.
- Manifests added `signal_stream_delivery_unix` at lane positions adjacent to the existing M5 signal entries.
- Ledger-recorded create-pr e2e: `121 passed, 0 failed, cache_hits=23/28, report_signature=d760194c89dbc954` — matches user's expected values.

**Verdict: PASS** — No blocking issues. The wave validates real Unix `ctrl_c()`, `terminate()`, and `shutdown_stream().next()` delivery via deterministic child-sent signals, is Windows-gated without claiming Windows delivery, adds no public runtime API, and does not overclaim Unix-only constants or non-Unix semantics.
