I have enough evidence to issue the review.

## M39.7 Rust Interop Follow-up Review — Round 4

### Verdict

**B1, B2, and B3 are all fixed. No new blockers introduced. Another review round is not required for the async/blocking/Tokio contract surface.** The remaining items are quality cleanups, two of which carry over unchanged from the prior round.

---

### Blocker disposition

**B1 — FIXED.** `is_async_decl` now propagates into `abi_requirements.async_boundary` and the effect:
- `crates/sifr_lowering/src/lower/rust_interop.rs:430-440` — `abi_requirements` sets `async_boundary = kind == Async || is_async_decl`.
- `crates/sifr_lowering/src/lower/rust_interop.rs:408-428` — `declaration_effect` escalates to `Async` when `is_async_decl && !declarations.is_empty()`.
- `crates/sifr_driver/src/build/rust_interop_probe.rs:239-246` — `is_async_probe` still reads `abi_requirements.async_boundary`, so a bare `@rust(target)` on `async def` now produces an async-shaped probe.
- New end-to-end coverage exists: `rust_interop_tests.rs:96-129` asserts the lowered `kind=Function` declaration on an `async def` carries `async_boundary=true`, and `rust_interop_async_contract_tests.rs:38-101` exercise the actual probe outcome for both compatible async signatures and sync-bound-to-async-misbinding.

**B2 — FIXED.** Opaque-class `thread_affinity=tokio_current_thread` reaches the async-Send probe:
- `crates/sifr_driver/src/build/rust_interop/async_validation.rs:82-111` — `async_thread_affinity_for_probe` first consults explicit per-function async contracts and then falls back to the owning opaque class's `thread_affinity` via `opaque_owner_thread_affinity`.
- `crates/sifr_driver/src/build/rust_interop.rs:558-566` — `push_probe` now clears `requires_send` when the declaration has `async_boundary` and the resolved affinity is `TokioCurrentThread`.
- The ordering invariant holds: classes precede their methods in `rust_interop_plan.rs:228-263`, so `opaque_contracts` is populated before any method `push_probe`.
- New test `package_rust_interop_opaque_current_thread_clears_async_method_send_probe` (`rust_interop_async_contract_tests.rs:159-186`) verifies an async method on a current-thread opaque class produces `requires_send=false` through the real lowering → resolver path.

**B3 — FIXED.** All four async tests now run end-to-end through `lower_module`:
- `rust_interop_async_contract_tests.rs:38-186` use `generated_from_source(...)` (lines 188-208), which parses real Sifr source, lowers it via `sifr_lowering::lower_module`, and feeds the resulting `RustInteropPlan` into the driver.
- No more synthetic `declaration_entry(..., RustInteropDecoratorKind::Async)` with `Some(target)` — those shapes are no longer reachable in the async suite.
- Fixture READMEs at `verification/areas/rust_interop/fixtures/async_ecosystem_matrix/README.md:5-11` and `verification/areas/rust_interop/fixtures/blocking_diagnostics/README.md:5-9` now cite the source-driven tests.
- The diagnostic registry witness at `crates/sifr_diagnostics/src/codes/registry/registry_entries/rust_interop.rs:67` points to `package_rust_interop_async_requires_send_future_by_default`, which is a real source-driven test.

---

### Non-blocking suggestions (round 4)

**N1 (carryover).** `crates/sifr_driver/src/build/rust_interop_probe.rs:255-259` — `stderr_reports_non_send_future` still matches three substrings, including the broad `"cannot be sent between threads safely"` and `"future is not \`Send\`"`. A non-future `Send` error elsewhere in the probe assertion can still misclassify as `SIFR-RUST-ASYNC-0001`. The resolution check at `:81-86` happens first (good), but resolution check ordering doesn't help when only a Send error fires.

**N2 (carryover).** `crates/sifr_driver/src/build/rust_interop/async_validation.rs:27-39` — the BlockingIo/CpuHeavy rejection on `async_boundary` is dead code: `crates/sifr_lowering/src/lower/rust_interop.rs:55-63` already emits `RUST_ASYNC_CONTRACT` and lowering returns `Err` before the driver runs. Either move the diagnostic into the driver and make lowering tolerant, or delete the branch — currently the witness path is asymmetric.

**N3 (new, minor).** `crates/sifr_lowering/src/lower/rust_interop.rs:420-424` — the second arm `declarations.iter().any(|d| d.kind == Async)` is unreachable because `parse_declaration` at `:42-49` only emits `kind=Async` when `is_async_decl=true`, so the first `is_async_decl && !declarations.is_empty()` branch always fires first. Dead code, not a defect.

**N4 (new, minor).** `crates/sifr_driver/src/build/rust_interop.rs:558-566` — the Send-clearing logic checks `async_boundary`, but `async_thread_affinity_for_probe` (`async_validation.rs:82-111`) is always called regardless. For non-async declarations the affinity lookup is wasted work (default `None`). Trivial cost; gating the call behind `if requires_send && declaration.declaration.abi_requirements.async_boundary` would be clearer.

**N5 (new, minor).** A `@rust.async(...)` decorator without a paired `@rust(target)` on the same `async def` produces a single `kind=Async, target=None` declaration that creates a metadata probe with no Rust path to validate against. No diagnostic is emitted. Either reject this as orphaned (no target to bind) or document it as intentionally a no-op.

**N6 (carryover from prior N6/N4/N7/N8).** Probe execution still shells out to `cargo check` per declaration in `/tmp`, without propagating `--locked`/`--offline`, and without logging on cleanup failure. Not introduced by this round; flagged again only because it remains a quality gap relative to the doc's "Cargo source of truth" invariant.

---

### Recommendation

Land M39.7. The blocker surface is closed, the new tests drive the real lowering path, and the spec invariants ("async Rust probes require `Send` by default" and "opaque-class `tokio_current_thread` opts out") are now enforced. N1–N6 can be queued for a follow-up cleanup pass or rolled into M39.8, but none warrant blocking the merge.
rc/build/rust_interop.rs` is now 883 lines (was 863), 17 below the 900 cap. Not over, but close — M39.8 work that touches this file should plan a responsibility split (the `RustInteropResolver` is already partially fan-out into `async_validation`/`opaque_validation`/`direct_panic_policy` sub-modules; same pattern can absorb the rest).

**NB7.** Round-2 N5 (`Bridge` name heuristic) and N7 (probe-root leak under failure) carry over — minor cleanliness, no behavior change needed for M39.7 exit.

---

### Recommendation

**Approve M39.7 contract surface.** B1/B2/B3 are correctly fixed with end-to-end test evidence. The remaining items are tracked best as M39.8 cleanup (NB2, NB6), runtime/observability follow-up (NB3), or scope/doc honesty (NB4). NB4 is the only one I'd actively call out: if the borrowed-input wrapper future isn't going to ship in M39.7, the scope bullet in `plans/phases/39_rust_interop.md:172` should explicitly say so before the milestone is declared done.
