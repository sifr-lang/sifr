I have enough. Here's my review.

## FAIL — do not open the M1 PR yet

Three blocking findings against the M0 contract / DoD, plus several non-blocking items that should land in the same PR or be acknowledged in the traceability.

### Blocking

**B1. `+ Send` bound applied to ALL Awaitable/Coroutine futures was not validated by the full e2e suite.**
`crates/sifr_type_system/src/types/type_queries.rs:415,440` adds `+ Send` to every emitted `Pin<Box<dyn Future<…>>>`, not just net. This is a cross-cutting change that affects every async pass fixture (e.g., `async_basic`, `async_generator_*`, `async_for_*`, `join_set_*`, `process_async_*`, `async_with_*`, ~30 in `crates/sifr/tests/e2e/pass/`). The validation list only ran `cargo test -p sifr --test e2e test_e2e_pass` with `SIFR_E2E_FIXTURE_MANIFEST` restricting to the two M1 net fixtures. `scripts/run_e2e_pass.sh` and `scripts/run_all_tests.sh --profile create-pr` (the project-mandated authoritative gate) were skipped.
**Required fix:** Run `scripts/run_e2e_pass.sh` and `scripts/run_all_tests.sh --profile create-pr` (followed by `scripts/run_all_tests.sh`) on this branch. If any existing async fixture or stdlib feature breaks, either scope the `+ Send` change to net-specific awaitables (e.g., type the net intrinsics with a dedicated Send-bounded variant, leaving general `Type::Awaitable` unchanged — `intrinsics/registry/net.rs:18-22` already constructs its own `send_awaitable_type`) or fix the regressions. Record the new validation rows in `verification/stdlib/network_http_m1_async_network_traceability.md`. AGENTS.md and the M0 phase doc make running the local merge gate a non-negotiable pre-PR step.

**B2. `TcpStream.split()` is fallible, but M0 contract requires it to be infallible.**
Phase doc lines 478-484 lock the contract: *"`split()` consumes a live `TcpStream` and is infallible; closed or moved streams cannot be split because the affine handle is no longer available"*. Current implementation:
- `lib/sifr/net.sifr:61` declares `def split(own self) -> Result[tuple[TcpReadHalf, TcpWriteHalf], NetError]` and raises `NetError` if `self._closed`.
- `crates/sifr_codegen/src/intrinsics/registry/net.rs:159-172` registers `net_tcp_stream_split` as `net_error_result(Type::Tuple(...))`, i.e., `Result[..., NetError]`.
- `crates/sifr_runtime/src/net.rs:346-371` returns `Result` and can return `Err("TCP stream cannot be split while an operation is active")` when `Arc::try_unwrap` fails.

`own self` already enforces affine consumption at the Sifr layer, so the `_closed` check and the `Arc::try_unwrap` defensive recovery are redundant against the M0 contract.
**Required fix:** Make split infallible at all three layers. Drop the `_closed` raise in `lib/sifr/net.sifr` (rely on `own self`); change the intrinsic registry return type to plain `Type::Tuple(...)` instead of `net_error_result(...)`; in the runtime, ensure the affine handle invariant guarantees the Arc count is 1 at `split()` time and replace `Arc::try_unwrap` with a direct ownership transfer (e.g., drop the Arc/Mutex layer for unsplit `TcpStream` storage, or pass the `tokio::net::TcpStream` directly through the handle table without `Arc<Mutex<…>>` indirection — the alternative is a panic, which M0 also forbids). If the impedance with handle-table storage genuinely can't be reconciled, that needs a formal M0 contract amendment before M1 ships, not a silent divergence.

**B3. Cancellation behavior is asserted in M0 DoD but not covered by any M1 fixture.**
M0 DoD (M1 milestone): *"Timeout and cancellation behavior is deterministic, typed, and panic-free."* The two M1 e2e fixtures only exercise input-validation typed errors (`timeout=0.0`, `timeout=-1.0`, `backlog=0`). Nothing exercises provider task-scope cancellation propagating into an in-flight `accept()`/`read_chunk()`/`write_all()`. M0 explicitly forbids a parallel cancellation token and requires consumption of `sifr.task`'s deadline/cancellation model; without an active fixture, the integration claim in `network_http_m1_async_network_traceability.md` is unverified.
**Required fix:** Add a loopback fixture that spawns a server task into `task.scope(...)`, blocks an `accept()` or `read_chunk()`, cancels the scope, and asserts a typed `NetError`/`CancelledError` (whichever the M0 error mapping designates) with deterministic partial-progress evidence. Record the new fixture row in the M1 traceability.

### Non-blocking but should be resolved before PR

**N1. Write-after-shutdown is not a stable typed variant.** `crates/sifr_runtime/src/net.rs:271-274` returns a single `NetError { message: "TCP write side is already shut down" }`, and `network_http_m1_tcp_loopback_split.sifr:49` substring-matches `"shut"`. M0 phase doc requires "a stable typed write-after-shutdown error". Either expose a distinct error variant (preferred) or document in M1 traceability that the message string is the M1 stable contract until the full taxonomy lands.

**N2. Tokio workspace features exceed the M1 locked set.** `Cargo.toml:97` now enables `process, signal` in addition to M1's `macros, rt, sync, time, net, io-util`. M0 ecosystem decisions (network feature row) lock the set to `macros, rt, sync, time, net, io-util` and explicitly reject `process`/`signal` for the network feature (rejected-features row, lines 388-390). If these are needed for other workspace crates (sifr_runtime tests, concurrency/runtime provider), document the justification in M1 traceability or scope this change to the provider PR — don't smuggle it under M1.

**N3. `StdlibFeature::Bytes`, `Socket2`, `TokioUtil` declared but unused.** `crates/sifr_stdlib/src/features.rs` registers all three and `features_for_stdlib_module("sifr.net")` pulls `Bytes` into every generated network program, but `sifr.net.sifr` and `sifr_runtime::net` use `Vec<u8>`. Generated programs will compile `bytes` for no functional reason. Either start using `bytes::Bytes` for body chunks internally (matches M0 ecosystem row) or drop `Bytes` from `features_for_stdlib_module("sifr.net")`. `Socket2`/`TokioUtil` are unwired entirely — keep them out of `features.rs` until the M2/M4 PRs that actually need them, otherwise the locked-decision audit slips.

**N4. Hardcoded `set_nodelay(true)` on every connect is an undocumented policy choice.** `crates/sifr_runtime/src/net.rs:162`. M0 lists `TCP_NODELAY` as an accepted `socket2` option but doesn't mandate it as a default for `connect_tcp`. Either document the default in the M1 traceability or expose a knob.

**N5. Inventory artifacts not updated.** Working tree only touches `verification/stdlib/network_http_m1_async_network_traceability.md` and the execution ledger; `verification/stdlib/network_http_substrate_inventory.{md,json}` still mark `sifr.net` surfaces as `open`. Technically allowed during M1, but if reviewers expect surfaces to flip to `production-public` on M1 PR, do it now to avoid carrying the gap to M5.

**N6. Runtime `MAX_READ_BYTES = 1_048_576` is hardcoded and unconfigurable.** `crates/sifr_runtime/src/net.rs:17`. M0 phase doc requires *"explicit configured limits"* on every reader. Hardcoding a per-call cap satisfies "explicit" but not "configured". Acceptable as a substrate default for M1; flag in M1 traceability so M4/M5 can layer configuration.

### What is good

- TCP loopback (connect / listen / accept / read / write / shutdown_write / split / close) is functionally complete and panic-free in the surveyed paths.
- Typed-error mapping out of the runtime via `__sifr_net_error` is consistent; runtime functions only return `Result<_, String>` and never panic on user data.
- `net_*` intrinsics are properly gated behind `StdlibFeature::SifrRuntime` and `StdlibFeature::Tokio`; the runtime crate hides `tokio::net::TcpStream` behind opaque `i64` handles, so no Tokio type or raw fd leaks to the Sifr surface.
- Generated Cargo feature set for `sifr.net` (snapshot test at `crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs:62-94`) matches the M0 locked ring decisions for the net feature.
- UDP is properly deferred via the `SIFR-NAME-0004` fail fixture; no partial public API ships.
- `shutdown_write()` correctly preserves the read side (idempotent on the bitset, read path unaffected), matching the M0 half-close contract.

Once B1-B3 are fixed and the full local validation gate is rerun, this is close to PR-ready.
