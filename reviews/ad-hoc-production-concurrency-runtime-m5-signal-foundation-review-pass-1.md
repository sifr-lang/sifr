Now I have enough context to write the focused review.

---

# M5 Signal/Shutdown Foundation — Review Report

## Scope Honesty (Q1)

The wave is clearly scoped to the signal/shutdown surface and visibly defers the remaining M5 work:

- `verification/stdlib/concurrency_runtime_m5_ergonomics_traceability.md:5` states "Status: In progress; signal/shutdown foundation wave adds the first production `sifr.signal` surface and keeps cleanup scopes, task/request context, and structured diagnostics pending."
- `verification/stdlib/concurrency_runtime_m5_ergonomics_traceability.md:34-39` "Follow-up Boundaries" enumerates the deferred items (external-signal harness, cleanup-scope traceability, `sifr.task.Context`/`ContextKey[T]`, structured diagnostics / warning rejection).
- `verification/platform/supported_host_matrix.md:33` moves the signal row from `blocked-on-concurrency-runtime-m5` to `in-progress` with an explicit list of what is and isn't covered.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:550-567` records the wave + targeted validation runs.

No overclaim of cleanup scopes, context, diagnostics, warnings, or runtime delivery; the unsupported CPython-shaped APIs (`signal.signal`, `pause`, `getsignal`, `raise_signal`, `pthread_sigmask`) are intentionally absent and produce diagnostics, matching the M5 scope on `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:683-712`.

## Async Surface Shape (Q2)

`Awaitable[Result[Signal, SignalError]]` returned from a sync `def` matches the established stdlib idiom — see `lib/sifr/process.sifr:521`, `:544`, `:558`, etc. `lib/sifr/signal.sifr:41-51` follows the same pattern (sync wrapper that returns the intrinsic's awaitable directly). End-to-end emit (verified via `sifr emit` and `sifr build` against an async driver using `await ctrl_c()`, `await terminate()`, and `stream.next().await`) lowers to `Pin<Box<dyn Future<Output = Result<Signal, SignalError>>>>` and compiles cleanly with Tokio's `signal` feature.

## Host Semantics Honesty (Q3)

Host gating is honest at the codegen and intrinsic level:

- `crates/sifr_codegen/src/intrinsics/registry/signal.rs:25` lowers `signal_sigterm_supported` to `cfg!(unix)`, and `lib/sifr/signal.sifr:30` consumes that into `sigterm().supported` — non-Unix never claims SIGTERM support.
- `crates/sifr_codegen/src/intrinsics/registry/signal.rs:32-46` returns a typed `SignalError` on non-Unix `terminate()` rather than silently waiting forever or panicking.
- `crates/sifr_codegen/src/intrinsics/registry/signal.rs:54-77` makes non-Unix `shutdown_stream().next()` await Ctrl-C only — SIGTERM parity is not faked.

**Minor — host-conditional pass fixture**: `crates/sifr/tests/e2e/pass/signal_constants_strsignal.sifr:16` does `assert terminate_signal.supported` unconditionally. Since `sigterm().supported` lowers to `cfg!(unix)`, this fixture is implicitly Unix-only. Validation lanes currently run on macOS/Linux so it passes today, but the fixture itself doesn't reflect the "Windows host-limited" claim. Acceptable for this wave; worth a follow-up note if Windows e2e is ever enabled.

**Minor — wording nit**: `crates/sifr_codegen/src/intrinsics/registry/signal.rs:44` returns `"SIGTERM shutdown stream is unsupported on this host"` from `lower_signal_terminate`. `terminate()` is a single-shot await, not a stream — the message should probably read "SIGTERM is unsupported on this host" or similar. Not a blocker; semantics are still correct.

## Dependency Features, Panic Discipline, Coverage (Q4)

- `crates/sifr_stdlib/src/features.rs:189` adds `"signal"` to `TOKIO_DEPS` correctly; `features.rs:413` maps `sifr.signal`/`_sifr.signal` to `StdlibFeature::Tokio` only. `crates/sifr_codegen/src/intrinsics/registry.rs:701-714` attaches `Some(StdlibFeature::Tokio)` to the three async signal intrinsics, so generated `Cargo.toml` won't pull the feature into projects that never import the module.
- No `unwrap()`/`expect()` in any of the lowerings — IO errors flow through `SignalError::new(...)` (the auto-generated constructor — verified via `sifr emit`). The only `assert!` calls in generated Rust come from user-authored `assert` statements in the test fixture, consistent with the codebase policy.
- `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:271-303` pins all four intrinsics by required feature + rendered substrings. Local validations listed in the prompt confirm `cargo fmt --check`, `cargo check`, `cargo test -p sifr_stdlib`, `cargo test -p sifr_driver` (Cargo feature gates), `cargo test -p sifr_codegen lowers_signal_intrinsics_via_registry`, and the e2e pass fixture all pass.

**Minor — fail fixtures lack `expect-error` markers**: `crates/sifr/tests/e2e/fail/signal_handler_registration_unsupported.sifr`, `signal_pause_unsupported.sifr`, `signal_getsignal_unsupported.sifr`, `signal_raise_signal_unsupported.sifr`, and `signal_pthread_sigmask_unsupported.sifr` have no `# expect-error[...]: SIFR-NAME-0004` headers. Per `crates/sifr/tests/e2e_support/e2e_entrypoints.rs:287` and `harness_model.rs:533-563`, the harness will accept any compile failure when no marker is present — so the SIFR-NAME-0002/0004 claim in the execution ledger is human-narrative only, not machine-enforced. This is consistent with many existing fail fixtures (bytes_*, argparse_*, etc.), so not a blocker, but adding markers would harden the contract and is easy to do.

## Other Style Nits (Non-Blocking)

- `crates/sifr_codegen/src/intrinsics/registry/signal.rs:15,40,63,67,73` use `Signal { number: ..., name: ..., supported: ... }` struct-literal syntax in the lowered Rust, while user-facing `lib/sifr/signal.sifr` lowers to `Signal::new(...)` (visible in `sifr emit`). Both compile because Signal is in the same generated module; cosmetic only.
- `crates/sifr_codegen/src/intrinsics/registry/signal.rs:5-7` builds the async body via `RustExpr::Ident(format!("Box::pin(async move {{ {body} }})"))` (raw string interpolation rather than a structured RustExpr tree). This mirrors `process.rs`'s pre-existing style; not new tech debt, but worth flagging as an existing pattern that could harden later.

## Blockers (Q5)

None. The wave is coherent, scope-honest, host-honest at the codegen level, and adequately tested for a foundation wave. The minor items above (Windows-implicit pass-fixture assertion, fail-fixture marker absence, message wording on non-Unix `terminate()`) are non-blocking polish items that fit naturally in a follow-up rather than this wave's first PR.

RESULT: PASS
