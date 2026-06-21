I have a complete picture. Producing findings now — no files modified.

## M39.7 Rust Interop Review — Async/Blocking/Tokio Contract Surface

### Verdict
**Another review round is required.** Findings 1 and 2 break the central spec invariants ("async direct Rust probes require returned futures to be Send by default" and the `thread_affinity=tokio_current_thread` opt-out), and finding 3 explains why the existing tests don't catch this — the unit tests exercise a plan shape lowering can never produce. The other items are quality cleanups, not blockers.

---

### Blockers

**B1. The default async probe (Send-by-default + thread_affinity opt-out) never fires for real user code.**
- `crates/sifr_lowering/src/lower/rust_interop.rs:114-130, 416-423` — `parse_declaration` sets `abi_requirements.async_boundary = (kind == Async)`. A bare `@rust(target)` decorator always parses with `kind = Function`, so its `async_boundary` is `false` regardless of whether the owning `def` is `async`.
- `crates/sifr_lowering/src/lower/rust_interop.rs:397-414` — `declaration_effect()` only escalates to `Async` when a `@rust.async(…)` decorator is present. The function's `is_async` flag is passed in as `is_async_decl` but only consumed by the early `@rust.async`-on-sync-def check at line 42-49; it does not influence the resulting declarations.
- `crates/sifr_driver/src/build/rust_interop_probe.rs:237-244, 171-215` — `is_async_probe` looks only at `declaration.kind == Async || abi_requirements.async_boundary`. For the canonical pattern documented in `internal_docs/rust_interop_architecture.md:573-576`:

```sifr
@rust(http_client.fetch)
async def fetch(url: str) -> Result[Response, HttpError | RustPanicError]: ...
```

lowering emits exactly one direct-probe-eligible declaration with `kind = Function`, `async_boundary = false`. The probe generated is `fn(&str) -> Result<…>` — a **sync** assertion. The `+ Send` Fut bound is never emitted; the doc's "async Rust probes require returned futures to be Send by default" is not enforced. Worse, an async Rust target would *fail* the sync probe (because `async fn` doesn't satisfy `fn(_) -> _`), and a sync Rust target would *pass* even though the caller is `async def`.

Codegen knows the truth — `crates/sifr_codegen/src/rust_interop_direct.rs:54-58` reads `func.is_async` — but the probe side only has `RustInteropPlanDeclaration`, which discards that bit. Fix the lowering to propagate `is_async_decl` into `abi_requirements.async_boundary` (or extend `RustInteropPlanDeclaration` with an `is_async` flag and read it in `is_async_probe`).

**B2. Class-level `thread_affinity=tokio_current_thread` is ignored by the async-Send probe.**
- `crates/sifr_driver/src/build/rust_interop_probe.rs:246-270` — `async_future_requires_send` reads only the function-level `@rust.async(thread_affinity=…)` argument from `probe.declaration.declaration.arguments`. There is no lookup against the opaque class's `thread_affinity`.
- `internal_docs/rust_interop_architecture.md:598-600` is explicit: non-Send futures are allowed when pinned "**on the opaque type** or through an explicit function-level `@rust.async(thread_affinity=tokio_current_thread)` declaration." With B1 fixed, an async method on a class declared `@rust.opaque(thread_affinity=tokio_current_thread, send=False, …)` will still be rejected because the probe demands `+ Send`. Opaque-class affinity must reach the probe (either via the `opaque_contracts` map already in the resolver or via a new field on the plan declaration).

**B3. The four `SIFR-RUST-ASYNC-0001` tests exercise a synthetic plan shape that lowering can never emit.**
- `crates/sifr_driver/src/build/rust_interop_async_contract_tests.rs:12-50, 99-176, 178-207, 209-236` use `declaration_entry("native.hash", RustInteropDecoratorKind::Async)` (defined in `rust_interop_contract_tests.rs:533-608`), which produces `RustInteropDecoratorKind::Async` *with a `Some(target)`*. Real lowering at `rust_interop.rs:132-168` forbids positional args on `@rust.async`, so a `kind == Async` declaration **always** has `target == None` after lowering. The path-resolving direct probe is therefore the `kind == Function` one with no async-ness, and the async probe path the tests claim to exercise is unreachable from user code (see B1).
- `verification/areas/rust_interop/fixtures/async_ecosystem_matrix/README.md:5-11` and `verification/areas/rust_interop/fixtures/blocking_diagnostics/README.md:5-9` cite these tests as "passing evidence" for `current_thread_non_send_future`, `non_send_future_without_affinity`, and `classified_async_declarations_rejected`. After B1+B2 are fixed, replace the synthetic constructors with end-to-end fixtures that go through `lower_module` → plan → driver, so the evidence reflects what users actually compile.

---

### Non-blocking suggestions

**N1.** `rust_interop_probe.rs:253-257` — `stderr_reports_non_send_future` matches three substrings. "`cannot be sent between threads safely`" and "``Send`` is not implemented" are wider than future-only Send errors (e.g., a non-Send opaque return type used elsewhere in the assertion); they can misclassify an unrelated probe failure as `SIFR-RUST-ASYNC-0001`. Tighten to the "`future cannot be sent`" prefix and fall through to `RUST_TYPE_PROBE_FAILURE` for ambiguous cases.

**N2.** `rust_interop/async_validation.rs:49-58` — the `Effect::BlockingIo | CpuHeavy` rejection on `kind == Async` is effectively dead. Lowering at `workload_annotations.rs:76-94` already emits `ASYNC_WORKLOAD_ANNOTATION_ON_ASYNC_DEF` for `@blocking_io`/`@cpu_heavy` on `async def`, which halts the pipeline before this driver check runs. M39.7's spec says the rejection should produce `SIFR-RUST-ASYNC-0001`; today only `ASYNC_WORKLOAD_ANNOTATION_ON_ASYNC_DEF` fires for real user code. Either move/duplicate the diagnostic into lowering under `RUST_ASYNC_CONTRACT`, or remove the unreachable driver branch — pick one to align the diagnostic family with the doc.

**N3.** `crates/sifr_lowering/src/lower/classes/class_body_lowering.rs:139-146, 245-252` — for enum and newtype class methods, `collect_rust_interop_declarations` is invoked with `is_async_decl = false`, ignoring `func.is_async`. Regular classes pass `func.is_async` correctly (line 429-436). `@rust.async` on an async method inside an enum/newtype class will be wrongly rejected. Edge case but inconsistent.

**N4.** `rust_interop_probe.rs:27-110, 564-578` — `execute_direct_cargo_probe` shells out to `cargo check` per probe in a fresh temp project, serially. The invocation also doesn't propagate `--locked`/`--offline`/`--frozen` from the parent build, which is inconsistent with the Cargo-source-of-truth invariant in `internal_docs/rust_interop_architecture.md:763-765`. Probe runs may succeed where the real build would fail under `--offline`.

**N5.** `rust_interop_probe.rs:295-342` — `collect_generated_bridge_names` walks `rust_*_type` strings and harvests any token ending in `Bridge` not prefixed with `__`. A user crate that happens to expose a `MyBridge` type would collide with the heuristic and produce a duplicate stub. Low risk, but the heuristic should be replaced by structured names carried on `RustBridgeTypeContract` (the codegen side already knows which types it generated).

**N6.** Phase 39_7 scope (`plans/phases/39_rust_interop.md:172`) lists "Own converted borrowed inputs inside generated async wrapper futures before exposing them to Sifr async lifetime and spawn checks" — there is no codegen for the wrapper future ownership and no test asserting that a borrowed `async fn fetch(url: &str)` reaches Sifr with owned-input lifetime. The contract surface is exercised but the wrapper isn't. Either move this to a sub-item with explicit "deferred to M39.8/M39.13" wording, or land a wrapper-generation test before exit.

**N7.** `rust_interop_probe.rs:36-68` — probe temp roots use `std::process::id()` + nanosecond nonce + hashed path, and cleanup uses `let _ = fs::remove_dir_all(…)`. If `cargo check` leaves a `target/` open or fails mid-build, the leak is silent and can balloon under `/tmp`. Log the leaked path or move probe roots into a project-local probe dir that `cargo clean` reaches.

**N8.** `rust_interop_probe.rs:73-103` — when `stderr_reports_non_send_future` matches but the failure was actually e.g. a missing item, the user sees `SIFR-RUST-ASYNC-0001` instead of `SIFR-RUST-RESOLVE-*`. Consider checking the resolve/not-found substrings *first* and the Send substring last, to prevent misordered classification.

---

### Recommendation

Block on B1–B3. Once async-ness propagates from the function into the plan declaration and class-level `thread_affinity` reaches the probe, rewrite the async tests to drive `lower_module` → driver end-to-end and re-cite them in the fixture READMEs. Re-run the existing local-validation suite (`cargo test -p sifr_lowering rust_interop`, `cargo test -p sifr_driver --lib rust_interop`, the rust_interop fixture matrix check) — these previously passed because the synthetic test shape sidesteps B1. A follow-up round should verify the corrected tests actually exercise the spec contract.
