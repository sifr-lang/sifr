# M5 signal stream shape/lowering review

## Overall shape — coherent intent

The wave is well-scoped: keeps the existing `Signal(name, number)` value model, adds the next layer (`strsignal`, awaitable `ctrl_c`/`terminate`, `ShutdownStream.next()`, `shutdown_stream()`) backed by Tokio signal APIs through new private `_sifr.signal` intrinsics, and honestly degrades non-Unix `terminate()` to a typed `SignalError`. The Tokio `signal` feature addition is explicitly sanctioned by the phase contract (issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:271). Public type signatures are right: `ctrl_c()/terminate()/ShutdownStream.next()` all return `Awaitable[Result[Signal, SignalError]]` and lower to `Pin<Box<dyn Future<Output = Result<Signal, SignalError>>>>` (confirmed via emit at lib/sifr/signal.sifr:37-47 → emitted `Box::pin(async move {...})`). Doc/traceability rewrites in verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:28-46 and verification/platform/supported_host_matrix.md:33-35 are honest about what is and isn't supported.

## BLOCKER 1 — Tokio feature update is incomplete; e2e pass suite fails

`crates/sifr_stdlib/src/features.rs:189` adds `"signal"` to `TOKIO_DEPS`, but three other places hard-code the old spec string and were not updated:

- `crates/sifr/tests/e2e_support/fixture_compilation.rs:480-482` — `tokio_dependency_spec()` is the helper the grouped e2e harness uses to write Cargo.toml (called at line 421 from `generate_cargo_toml`, which `batch_execution.rs:65` and `:349` invoke for every grouped fixture build). It still emits `features = [..., "rt", "sync", "time"]` with no `"signal"`.
- `crates/sifr/tests/e2e_support/harness_behavior_tests.rs:522` — `test_generate_cargo_toml_required_tokio_uses_runtime_features` only passes because both the helper and the assertion still pin the stale string.
- `crates/sifr_codegen/src/lib_codegen_tests/async_runtime_codegen_tests.rs:165` — `test_generate_project_emits_tokio_dependency_when_required` directly **FAILS** today against the updated `TOKIO_DEPS`. Verified locally:
  ```
  assertion failed: cargo_toml.contains("tokio = { version = \"1.52.3\", features = [\"io-util\", \"macros\", \"process\", \"rt\", \"sync\", \"time\"] }")
  ```

Because the grouped harness still uses the stale spec, the new fixture fails to compile under the grouped batch path. Verified by running the actual e2e pass suite locally:
```
FAIL [signal_stream_shape_strsignal]: Rust compilation failed.
group fixture list: [signal_stream_shape_strsignal]
```
This is exactly the case where `Box::pin(async move { tokio::signal::ctrl_c().await ... })` reaches rustc without the `signal` feature.

The reported "focused local validation" did not include `scripts/run_all_tests.sh --profile create-pr`, which is the project's authoritative gate per AGENTS.md. That lane would have caught both the unit-test failure and the grouped e2e failure before any PR was prepared. The execution-ledger claim of running this fixture only through `cargo run -- run` is insufficient because that path uses `generated_cargo_dependencies()` (which was updated) rather than the harness helper.

## Smaller findings (non-blocking, worth tidying)

- crates/sifr_codegen/src/intrinsics/registry/signal.rs:5-6 — `RustExpr::Ident(format!(...))` shoves a multi-line string with `?` and `tokio::select!` through the renderer rather than building it from `RustExpr`/`RustStmt` IR. Other tokio-backed intrinsics (e.g. `process_async::lower_process_async_run` at crates/sifr_codegen/src/intrinsics/registry/process_async.rs:52-60) call generated runtime helpers (`__sifr_process_async_run`) instead of inlining the body. The current approach works (emit verified) and `Box::pin(async move {...})` is the same future shape, but it locks the lowering into raw text that bypasses `syn`/prettyplease invariants. Consider following the existing helper-function pattern (a `signal::__sifr_ctrl_c()/__sifr_terminate()/__sifr_shutdown_next()` preamble) before this layer grows — the helpers also make it easier to inject the deterministic delivery harness later without rewriting raw text inside `RustExpr::Ident`.
- lib/sifr/signal.sifr:29-34 — `strsignal` is a Sifr-side number match (`if signal.number == 2`, `== 15`). Fine for the two POSIX values today, but the contract says structured signal value identity must come from `Signal` rather than ad-hoc ints; once Unix-only constants land you'll need to revisit. Not a blocker, but worth a comment-free TODO in the execution ledger so it isn't forgotten.
- lib/sifr/signal.sifr:45-51 — `ShutdownStream` exists solely to host a single `next()` method. The execution ledger already calls deterministic delivery + cleanup stacks follow-up; the wave is fine as a shape pin but `shutdown_stream()` returning a single-method stub is a small API smell. Acceptable for this wave.
- verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:11 — The `SignalError` row now points to `signal_stream_shape_strsignal`. That fixture does exercise the `unsupported signal` raise path inside `strsignal`, so the citation is accurate, but make sure the follow-up wave that adds host SIGTERM delivery still keeps a fixture for the listener-failure branch (the `failed to install SIGTERM listener:` path in registry/signal.rs:31 is currently unreachable from any fixture).
- crates/sifr/tests/e2e/pass/signal_stream_shape_strsignal.sifr:29-31 — the `_*_wait` futures are bound and dropped without polling. That's exactly the right call for a shape-only fixture (no external signal in the e2e), but please leave a one-line `# shape pin: not awaited; ...` so a future reader doesn't "fix" it.
- verification/platform/supported_host_matrix.md:34 — adding a separate "Signal stream shape and lowering" row alongside the umbrella `Signals and structured shutdown streams` row is fine, but the umbrella row at line 33 already enumerates the same evidence. One of them should reference the other to avoid drift.

## Answers to the review questions

1. **Coherent next M5 wave?** Yes in intent; no in delivery because the Tokio feature update missed the grouped harness and unit tests, so the e2e suite breaks.
2. **Sync wrapper returning `Awaitable[Result[...]]`?** Fits — generated Rust returns `Pin<Box<dyn Future<...>>>` and the user awaits at the call site. Emit confirms it.
3. **Honest host semantics?** Yes — Unix uses `tokio::signal::unix::SignalKind::terminate()`; non-Unix `terminate()` returns typed `SignalError("SIGTERM is unsupported on this host")`; non-Unix `shutdown_stream` waits on Ctrl-C only. Matches the supported-host matrix and traceability doc.
4. **Tokio feature change acceptable?** The contract sanctions exactly `signal`. The change itself is right; the rollout is not (see Blocker 1).
5. **Coverage adequate without deterministic delivery harness?** For a *shape* wave, yes — codegen registry test pins lowering strings and the pass fixture pins compile/run shape. But the unreachable listener-failure branch should be tracked.
6. **Blockers?** Yes — Blocker 1 above.

## Required fixes before this PR can merge

1. Update `crates/sifr/tests/e2e_support/fixture_compilation.rs:481` to include `"signal"` in the Tokio features list.
2. Update the matching assertion at `crates/sifr/tests/e2e_support/harness_behavior_tests.rs:522`.
3. Update the assertion at `crates/sifr_codegen/src/lib_codegen_tests/async_runtime_codegen_tests.rs:165`.
4. Run `scripts/run_all_tests.sh --profile create-pr` (the authoritative gate from AGENTS.md) and capture the report, then update the execution ledger's "targeted local validation" block accordingly. The current ledger entry at issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:591-600 only lists narrowly-scoped commands; that's why the grouped e2e regression was missed.

RESULT: FAIL
