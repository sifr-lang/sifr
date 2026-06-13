All five acceptance criteria check out cleanly.

**PASS**

Verification summary:

- **fixture worker is test-gated** — `crates/sifr_stdlib/Cargo.toml:16-23` declares the `__test_fixture` feature and the `sifr-stdlib-ipc-pipe-fixture-worker` `[[bin]]` requires it (`required-features = ["__test_fixture"]`, `test = false`, `bench = false`), with its source under `tests/fixtures/`. A plain `cargo build -p sifr_stdlib` will not build a production worker binary.
- **real Unix pipes + existing helpers** — `tests/ipc_process_pipe_fixture.rs:1` is `#![cfg(unix)]`; `WorkerProcess::spawn` runs `cargo run --features __test_fixture --bin sifr-stdlib-ipc-pipe-fixture-worker` with `Stdio::piped()` on stdin/stdout, and tests drive frames through `read_frame`, `write_frame`, `IpcConnectionState`, `IpcConnectionConfig`, and `IpcEnvelope` exported from `sifr_stdlib`.
- **coverage** — bootstrap via `connect()` (Hello/Ready); completion in `unix_child_process_pipes_complete_run_and_shutdown`; in-flight `Cancel`→`Failed { error: b"cancelled" }` in `unix_child_process_pipes_cancel_in_flight_request`; `Shutdown`→`Terminating` close in the shared `shutdown` helper; truncated-frame report (3-byte length prefix + 1-byte payload + EOF) in `unix_child_process_pipes_report_malformed_frame`, with parent transition to `IpcConnectionPhase::Closed` asserted.
- **docs honesty** — design doc row at `concurrency_runtime_m6_typed_ipc_design.md:35` claims only Unix child-process pipe transport and explicitly leaves Windows fixtures, payload eligibility diagnostics, and generated worker integration as follow-ups; `supported_host_matrix.md:44` records macOS/Linux `supported`, Windows `host-limited`, with the same follow-up callouts. The status banner at `concurrency_runtime_m6_typed_ipc_design.md:5` is consistent.
- **line counts / guardrail** — touched files: Cargo.toml 27, fixture worker 134, fixture test 248, design doc 250, host matrix 46. All well under the 900-line cap. Issues ledger line count is pre-existing doc, not source.
- **API surface used by the fixture is real** — `IpcConnectionPhase`, `IpcHandshakeDecision`, `protocol_error_frame`, `begin_parent_handshake`, `accept_worker_bootstrap`, `accept_parent_hello`, `apply_established_frame`, `in_flight_len`, `phase` are all `pub` and re-exported from `sifr_stdlib::lib.rs`.
