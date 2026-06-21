Round-2 verification complete. I verified each round-1 fix against the current source.

## Round-1 fixes verified

1. **Finding 1 (View signature loss)** — `crates/sifr_codegen/src/rust_interop_bridge_contract.rs:119-122` now skips only `RustInteropDecoratorKind::Callback`, not the broader `!matches!(Function | Async)`. Standalone `@rust.view(...)` declarations again produce signature entries that `validate_view_owner` can look up.

2. **Finding 2 (Callback in advanced-data grouping)** — `crates/sifr_driver/src/build/rust_interop/advanced_data_validation.rs:46-49` now matches only `Function | View`. Reverted to the M39.10 behavior.

3. **Finding 3 (Probe-source Callback arm)** — `crates/sifr_driver/src/build/rust_interop_probe.rs:140-142` replaced the dead arm with `unreachable!("callback metadata is targetless and never enters probe planning")`. The invariant holds: `probe_planning::probe_kind` returns `None` for Callback (`probe_planning.rs:27`) and `resolve_declaration` returns early at `rust_interop.rs:228-230` before `push_probe`.

4. **Finding 4 (`generated_token()`)** — `crates/sifr_runtime/src/interop.rs:176-180` keeps `ThreadsafeCallbackBridge` as a contract marker with `#[derive(Debug)]`, a `_private: ()` field, and the comment "Contract marker for generated Rust callback bridge signatures." Constructor removed. Only the type name is referenced (codegen at `rust_interop_bridge_contract.rs:449-451`, plan test at `rust_interop_plan_tests.rs:207`).

5. **Finding 5 (Single per-declaration policy assumption)** — `internal_docs/rust_interop_architecture.md:732-734` documents "A single `@rust.callback(...)` policy applies uniformly to all top-level callback parameters on that declaration; per-parameter callback policy requires a later extension."

## Side-effects of Finding 1 fix — checked, not blocking

Restoring the broader signature-extraction loop means paired `@rust(...) + @rust.view(...) + @rust.zero_copy(...)` decorations now produce 3 identical entries in `RustBridgeContractPlan.signatures`. Verified that:

- The `signature_contracts: HashMap<String, …>` keyed by `canonical_target_path` (`rust_interop.rs:129-137`) collapses duplicates, so `validate_view_owner` / `validate_signature_ownership` / `validate_panic_declaration` lookups remain single-valued.
- `validate_bridge_contracts(signatures)` (`rust_interop.rs:621-630`) iterates the Vec, so a hypothetical paired-decorator declaration with an unsupported bridge type would emit duplicate `SIFR-RUST-TYPE-0001` diagnostics. This is **pre-existing M39.10 behavior** — the M39.11 `!matches!(Function | Async)` filter was the new addition that suppressed it; reverting it returns to the prior state. No existing fixture pairs a metadata decorator with an unsupported parameter, so no observed regression. Worth a follow-up dedup at some point, but not a M39.11 blocker.

## Spot-checks

- New `callback_validation.rs` groups by Function + Callback only — View/ZeroCopy don't enter the by-target map, no spurious diagnostics on paired declarations.
- Diagnostic registry `SIFR-RUST-CB-0001` (`rust_interop.rs:84-94`) matches the docs page (`docs/errors/SIFR-RUST-CB-0001.md`) on code, family, message template, args, dedupe keys, and representative fixture.
- Phase doc (`plans/phases/39_rust_interop.md:228`) accurately scopes runtime-observed behavior as staged behind `callbacks_call_scoped` / `callback_subscription_matrix`.

**The implementation is reviewer-satisfied.** No remaining blocking or important findings.
