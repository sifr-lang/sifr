# M5 signal stream shape/lowering review — pass 3 (post-rebase)

## Scope of this pass

PR #2418 (`abdd8674b`) is already on `origin/main`. The branch under review is `codex/concurrency-runtime-m5-signal-stream-ledger`; the only diff vs `origin/main` is `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` (merged-link update + post-rebase validation evidence). I re-verified the merged code + the new ledger entry against the pass-1 blocker and the post-rebase create-pr lane.

## Pass-1 blocker remains closed after rebase

The single previously-blocking class of finding (Tokio `signal` feature added in `crates/sifr_stdlib/src/features.rs` but not in the three sibling spec strings) is closed. All four spec locations are now byte-identical on the alphabetized features list `["io-util", "macros", "process", "rt", "signal", "sync", "time"]`:

- `crates/sifr_stdlib/src/features.rs:189` (`TOKIO_DEPS`)
- `crates/sifr/tests/e2e_support/fixture_compilation.rs:481` (`tokio_dependency_spec()` used by the grouped e2e harness)
- `crates/sifr/tests/e2e_support/harness_behavior_tests.rs:522` (harness drift assertion)
- `crates/sifr_codegen/src/lib_codegen_tests/async_runtime_codegen_tests.rs:165` (`test_generate_project_emits_tokio_dependency_when_required`)

No regression after rebase.

## Correctness of the lowering (re-verified)

`crates/sifr_codegen/src/intrinsics/registry/signal.rs:1-71`:

- `lower_signal_ctrl_c` (lines 9-19): `tokio::signal::ctrl_c().await` -> `Ok(Signal{...SIGINT, 2})` / `Err(SignalError::new(...))`. Typed error path is real; no `unwrap`/`expect`/`panic!`.
- `lower_signal_terminate` (lines 21-40): Unix branch installs `tokio::signal::unix::signal(SignalKind::terminate())` and propagates install failures via `?` as a typed `SignalError`. Non-Unix branch returns a typed `SignalError("SIGTERM is unsupported on this host")` — no silent fallback to Ctrl-C, which is the honest call for this surface.
- `lower_signal_shutdown` (lines 42-71): Unix branch uses `tokio::select!` between `ctrl_c()` and the SIGTERM listener; non-Unix waits on Ctrl-C only. Non-Unix degradation is documented in `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:36` ("Non-Unix waits for Ctrl-C only and does not claim SIGTERM support").

`crates/sifr_stdlib/src/signal.rs:5-39`: `signal_ctrl_c`/`signal_terminate`/`signal_shutdown` are registered with `Type::Awaitable(Box::new(result_ty(Signal, "SignalError")))`. The Sifr-side wrappers in `lib/sifr/signal.sifr:37-51` expose `ctrl_c() / terminate() / ShutdownStream.next()` with matching return types. The `Signal` / `SignalError` bare identifiers in the lowered bodies resolve because the only callers are within `sifr.signal`, where codegen for the Sifr classes emits the corresponding Rust items in the same module scope. The `lowers_signal_intrinsics_via_registry` codegen test (`crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:271-298`) pins the emitted strings (`tokio::signal::ctrl_c().await`, `SignalKind::terminate`, `SIGTERM is unsupported`, `tokio::select!`), so a rename or feature-flag drift would surface as a test failure.

User-path panic audit: no `.unwrap()`, `.expect()`, or `panic!` in `registry/signal.rs`, `sifr_stdlib/src/signal.rs`, or `lib/sifr/signal.sifr`. All host-listener install errors become typed `SignalError`.

## No overclaiming

- `verification/platform/supported_host_matrix.md` keeps the umbrella row at `in-progress` and the new "Signal stream shape and lowering" row at `in-progress` for Linux/macOS, `host-limited` for Windows. Non-Unix `terminate()` is explicitly described as "returns typed unsupported `SignalError`".
- `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:28-46` upgrades the `ctrl_c` / `terminate` / `shutdown_stream().next()` rows to `in-progress` (not `supported`), and the Follow-up Boundaries call out that "Deterministic external-signal delivery harnesses remain separate M5 follow-up evidence before stream delivery can be marked fully supported".
- The fixture `crates/sifr/tests/e2e/pass/signal_stream_shape_strsignal.sifr:23-26` carries the `# Shape pin only` comment and does not poll the futures, matching the documented scope.

The wave does not claim deterministic delivery, does not claim non-Unix SIGTERM, and does not claim Unix-only constants beyond `SIGINT`/`SIGTERM`. The ledger ("M5 signal stream shape and lowering" merged link + post-rebase validation evidence) reflects the same scope.

## Post-rebase validation evidence is sufficient

`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:611` records:

> Post-rebase `scripts/run_all_tests.sh --profile create-pr` -> PASS after resolving docs against PR #2419; report `target/validation_lane_reports/create-pr.latest.json`; e2e pass suite `120 passed`, `0 failed`, `cache_hits=27/34`, `report_signature=293aaf3695dc42f8`; platform golden `pass=6`, `skip=1`. Advisories: warm wall-time budget exceeded (`959.65s`, warm target `<=2m`) and warm-cache hit rate below advisory target (`79%`, target `>=90%`).

This is the authoritative gate from AGENTS.md, run against the rebased branch. The fixture count delta (117 -> 120) is consistent with the M5 fixtures landed in PRs #2414 / #2416 / #2419 between the two runs, and the report signature changes accordingly. Advisories are warm wall-time and cache hit rate only — both are advisory, not failure conditions, and they are recorded honestly rather than hidden.

The ledger also distinguishes the broad non-profiled probe (which still surfaces unrelated text/I/O failures) from the authoritative profiled lane, which is the correct framing.

## Generated dependency consistency

`Tokio` is the only generated dependency affected by this wave, and all four spec strings (generated runtime, grouped harness helper, harness contract assertion, codegen project assertion) are aligned. The `lowers_signal_intrinsics_via_registry` test pins `StdlibFeature::Tokio` as the required feature on each of the three intrinsics, so the registry-side wiring cannot regress silently either.

## Working-tree hygiene

`git status` shows only `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` modified and the new pass-3 review file untracked. The pass-2 scope-hygiene concern (unrelated network/HTTP edits sitting in the working tree) is no longer present — the ledger PR is cleanly scoped.

## Non-blocking follow-ups (carry-over)

1. `crates/sifr_codegen/src/intrinsics/registry/signal.rs:5-6` still emits multi-line bodies via `RustExpr::Ident(format!(...))` instead of building structured IR or delegating to a `__sifr_*` runtime helper (the pattern used by `process_async`). This layer works and is pinned by the codegen test, but should be refactored before more Tokio-backed signal lowerings land on top of it. Pass-1 / pass-2 carry-over; not a blocker for this PR.
2. The `failed to install SIGTERM listener:` branch at `registry/signal.rs:31` and the `failed to wait for SIGINT:` branch at `:16` are unreachable from any current fixture. The deterministic-delivery harness wave should add a listener-failure fixture; tracking via execution ledger only is fine for this wave.
3. `lib/sifr/signal.sifr:45-51` keeps `ShutdownStream` as a single-method class hosting `next()`. Acceptable as a shape pin; an iterator/stream protocol should replace it when deterministic delivery + cleanup stacks land.
4. Minor semantic note: `let _ = __sifr_sigterm.recv().await;` and the SIGTERM arm in `tokio::select!` treat `Option::None` (stream dropped) the same as `Some(())`. In practice tokio's signal stream only yields `None` on drop, so this is benign here, but a future deterministic-delivery harness should distinguish "listener gone" from "SIGTERM received". The traceability doc already scopes deterministic delivery as follow-up, so no change is required for this wave.

## Answers to the review questions

1. **Correctness after rebase**: The merged surface (codegen lowering, stdlib registration, Sifr wrappers, e2e fixture, codegen registry test, harness assertions) is consistent and the four Tokio spec locations are aligned.
2. **No overclaiming**: Yes — supported_host_matrix and traceability stay at `in-progress` for the streams, non-Unix `terminate()` is honestly typed-unsupported, and the fixture is documented as shape-only.
3. **No hidden runtime panic / fallback**: No `.unwrap()`, `.expect()`, or `panic!` in user paths; install failures become typed `SignalError`; non-Unix degradation is explicit and documented.
4. **Generated dependency consistency**: Verified across all four spec sites.
5. **Post-rebase validation evidence sufficiency**: Yes — `scripts/run_all_tests.sh --profile create-pr` is the AGENTS.md gate, it PASSED with the new fixture, the count delta and signature change are coherent, and the advisories are recorded honestly.

RESULT: PASS
