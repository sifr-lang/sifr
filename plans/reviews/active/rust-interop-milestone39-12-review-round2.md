# M39.12 Rust Interop Tooling, Diagnostics, Docs — Review Round 2

Branch: `phase39-rust-interop-m39-12`
Scope: items listed under `milestone_39_12` in `plans/phases/39_rust_interop.md`.
Result: **Reviewer is satisfied.** All four round 1 blocking findings (C1, C2, H1, M1, M2) are resolved. Two non-blocking carry-overs (L1, L2) plus the doctest worktree noise remain as previously documented.

## Verification of round 1 blockers

### C1 — async example no longer asserts a rejected `thread_affinity` symbol — resolved.

`docs/rust-interop.mdx:92-104` now reads:

> Async Rust interop must be declared on an `async def`. By default, returned futures must be `Send`; add `@rust.async(thread_affinity=tokio_current_thread)` only for explicitly current-runtime futures.
>
> ```sifr
> @rust(bridge.http.fetch_text, panic=map_error(bridge.http.map_panic))
> async def fetch_text(url: str) -> Result[str, HttpError | RustPanicError]: ...
> ```

This matches `crates/sifr_driver/src/build/rust_interop/async_validation.rs:55-67` (only `none` and `tokio_current_thread` are accepted) and `internal_docs/rust_interop_architecture.md:597-615` (`Send` is the default async contract; current-thread is the only opt-out).

### C2 — callback example no longer uses a rejected `lifetime=` key — resolved.

`docs/rust-interop.mdx:151-164` now reads:

> Backpressure, overflow, and shutdown behavior must be visible in callback registration declarations. Thread-safety is part of the callback type and the explicit `@rust.callback(...)` contract.
>
> ```sifr
> @rust.callback(
>     backpressure=bounded(1024),
>     overflow=error,
>     shutdown=drain,
> )
> @rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
> def subscribe(handler: ThreadsafeCallback[[Event], Result[None, EventError]]) -> Result[Subscription, EventError | RustPanicError]: ...
> ```

The keys match `crates/sifr_driver/src/build/rust_interop/callback_validation.rs:136-156` exactly. The recommended remediation from round 1 (switch the parameter type to `ThreadsafeCallback[...]` to mirror `internal_docs/rust_interop_architecture.md:704-707`) was also adopted.

### H1 — policy-key completion is now per-decorator — resolved.

`crates/sifr_analysis/src/completion.rs:228-275` dispatches on `RustInteropCompletionDecorator`, and each arm matches its validator:

- `Function → ["panic"]`, matching `panic_validation.rs:125-149` (only `panic` is consumed; nothing else is suggested).
- `Async → ["thread_affinity"]`, matching `async_validation.rs:55-77`.
- `Opaque → ["type","send","sync","clone","close","borrow","thread_affinity"]`, matching `opaque_contract.rs:84-143` and `internal_docs/rust_interop_architecture.md:152-170`.
- `ZeroCopy → ["owner","view"]`, matching `zero_copy_validation.rs:225-251`.
- `View → ["owner","lifetime","mutability","send","sync"] + advanced data keys`, matching `zero_copy_validation.rs:254-291` plus `advanced_data_validation.rs:162-175`.
- `Callback → ["backpressure","overflow","shutdown"]`, matching `callback_validation.rs:136-156`.

Three negative-set tests guard the regression that round 1 flagged (`completion.rs:374-413`): callback excludes `lifetime`/`panic`, opaque excludes `backpressure`/`owner`, view excludes `backpressure`. The host pipeline is now covered too at `crates/sifr_analysis/src/host/tests.rs:231-257` — observation 3 from round 1 is also resolved.

### M1 — multi-line decorator completion works — resolved.

`crates/sifr_analysis/src/completion.rs:145-176` walks backward from the cursor for `@rust`, then parses the suffix (`parse_decorator_suffix` at `:189-204`) and the argument depth (`policy_keys_available_before_cursor` at `:206-226`). The canonical multi-line shape used in `docs/rust-interop.mdx:110-146` is locked by `completion.rs:415-434` (`@rust.view(\n    owner=input,\n    schema=\n)` returns `lifetime`, `schema`, `protocol`).

### M2 — `@rust(` no longer suggests policy keys before the target separator — resolved.

`completion.rs:206-226` only returns `policy_keys_available = true` when `decorator != Function` OR a `,` was observed at depth 1. `completion.rs:436-461` locks this directly: `@rust()` returns empty, `@rust(bridge.hash, )` returns `panic`.

## Other DoD checks

- **Tooling parity** — `cli_model_and_entrypoint.rs:421-443` routes `sifr bridge check` through `cmd_check` with the same selection/lock-mode shape used by `Commands::Check` (`:444-466`). There is no second diagnostic stack, no second metadata path, and no command-specific filter. Argument-parsing coverage is at `bridge_cli_tests.rs:4-37`.
- **Docs alignment** — diagnostic family table at `docs/diagnostics/error-codes.mdx:138-155` links each `SIFR-RUST-*` family to its `/errors/SIFR-RUST-*-0001` page; all ten target pages exist under `docs/errors/`. Trust policy keys in `docs/packages/manifest.mdx:161-168` and `docs/rust-interop.mdx:22-34` match `crates/sifr_package/src/manifest/sifr_fields.rs` (no schema drift).
- **LSP completion kinds** — `crates/sifr_lsp/src/conversion.rs:511-521` maps `decorator → 15` and `property → 10`, locked by `:579-594`.
- **Rejected designs** — `docs/rust-interop.mdx:185-207` lists `@rust("crc32fast::hash")`, `extern rust`, `from rust import`, and `@rust(crate=..., path=...)` as invalid forms without remediation prose. The closing sentence at `:207` explicitly forbids the three banned silent fallbacks (zero-copy → copy, hidden Tokio runtime, unwinding panic in recoverable builds).
- **User-facing examples** — `docs/rust-interop.mdx` ships the six required walkthroughs (crc32fast direct, blake3 with `map_error`, tokenizer opaque handle, async HTTP, Arrow + DLPack zero-copy/view stacks, and threadsafe callback registration).

## Non-blocking carry-overs

### L1 (deferred from round 1) — `completion_item` still emits no `insertText`/`textEdit`.

`crates/sifr_lsp/src/conversion.rs:191-198` returns only `label`, `kind`, `detail`, and `data`. Dotted labels (`rust.callback`, etc.) can still be spliced by the client into `@rust.rust.callback` when the cursor sits mid-token. Round 1 categorised this as Low and deferrable; flagging again so it stays on the M39.13 / follow-up list. Remediation unchanged: emit `filterText` + `insertText` for dotted decorator labels, then extend the existing kind-mapping test at `conversion.rs:579-594` with an inserted-text assertion.

### L2 (deferred from round 1) — no end-to-end test asserts `sifr bridge check` and `sifr check` produce identical diagnostics.

`crates/sifr/src/bridge_cli_tests.rs` still covers argument parsing only. `mode_resolution_tests.rs` adds no bridge fixture (grep for `bridge` finds only an unrelated `rust_member` Cargo manifest at `:517-528`). Equivalence holds by construction today, but the DoD wording — "Tooling surfaces the same target resolution and diagnostics as the compiler" — is a behavioural promise that should be locked with one regression test. Remediation unchanged: drive both commands against a shared fixture and assert identical `RenderedDiagnostic` output.

## Observations (non-blocking)

- `crates/sifr_analysis/Cargo.toml` and `crates/sifr_lint/Cargo.toml` add `[lib] doctest = false` (validation support). These changes are unrelated to M39.12 scope. Per the reviewer brief, they should land in a separate PR or be reverted before merge of this milestone, but they do not block the technical correctness of the M39.12 work.
- `crates/sifr_analysis/src/completion.rs:189-204`: the decorator suffix parser uses raw `strip_prefix`, so `.viewer` would briefly match `.view` (next char check then fails on `(`). The current flow rejects the false hit safely. Worth tightening if any new decorator name shares a prefix with an existing one; not a real bug today.
- `docs/rust-interop.mdx:60` references `bridge.hash.map_panic` but the doc never shows a `src/bridges/hash.rs` companion (the crc32fast example two lines above is self-contained). Round 1 observation; still present and still non-blocking.

## Sign-off

C1, C2, H1, M1, and M2 are resolved with corresponding negative-set tests. The M39.12 implementation is technically complete. Recommend addressing L1/L2 and clearing the unrelated `doctest = false` worktree noise as their own follow-up changes; this milestone PR is otherwise mergeable.
