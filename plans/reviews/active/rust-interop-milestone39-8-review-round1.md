I have enough to give a focused review. Let me write the report.

---

# M39.8 Panic Boundary & Rust Error Surface — Review (Round 1)

## Verdict

**No blockers.** The implementation matches the milestone status note's claimed scope and the validation matrix is consistent with the new contract tests. Recommend merging after a quick decision on the gap items below — none requires another review round, but two are architecture-conformance gaps the reviewer should accept as deferred *in writing*.

## What's solid

- `RustPanicError` registered as a built-in error class in both `sifr_codegen::builtin_errors` (line 28) and `sifr_lowering` (`signatures_and_effects.rs:86`), with the parent-class hierarchy set to `Error`. The new lowering test (`rust_interop_tests.rs:64-78`) covers it.
- `crates/sifr_driver/src/build/rust_interop/panic_validation.rs:17-78` cleanly enforces the four documented panic surfaces (`RustPanicError` in error channel, `map_error(path)`, `trusted_no_panic`, `abort`) and the new `SIFR-RUST-PANIC-0001` family. The diagnostic is properly registered (`registry.rs:89`, `registry_entries/rust_interop.rs:73-83`), catalogued, baseline-deferred, and documented.
- Abort policy is double-gated: `[trust].rust-panic-abort` evidence AND `[profile.release] panic = "abort"` in the package's `Cargo.toml` ancestor chain. The three abort tests (no-trust → SIFR-RUST-TRUST-0001 first, trust-but-unwind → SIFR-RUST-PANIC-0001, trust+abort → ok) all flow correctly through the two-pass diagnostic order.
- `crates/sifr_runtime/src/interop.rs:174-187` `catch_rust_panic` correctly suppresses the default panic hook (via a global `Mutex` to serialise hook swaps) and redacts payload contents to "Rust bridge panicked"; `from_panic_payload` takes `&(dyn Any + Send)` and never inspects the payload. Tests cover the redaction explicitly (`secret backend token` panic message is not leaked).
- `Handle<T>` / `PoisonOnPanic` from M39.6 are unchanged; the new code wires panic redaction through them. Poisoned-wins-over-closed and panic-drop-poisoning tests remain green.
- Old `direct_panic_policy.rs` (which used the misleading `SIFR-RUST-TYPE-0001` family) is correctly deleted and superseded; the equivalent contract test was retargeted to `SIFR-RUST-PANIC-0001` and broadened to all declarations rather than direct bindings only — improvement, not a regression.
- Fixture matrix `panic_boundary` and `panic_abort_profile` flip from `planned` to `passing`, and the two new fixture READMEs cite the exact tests as evidence.
- File sizes: every touched file is under the 900-line cap (largest is `rust_interop.rs` at 882 — see suggestion below).

## Architecture-conformance gaps (non-blockers)

These do not exceed the milestone status note (which explicitly limits the claim to "RustPanicError/`panic=map_error(...)` Result panic surfaces"), but they diverge from `internal_docs/rust_interop_architecture.md` §"Panic Surface Policy". Each should be tracked or explicitly punted before phase exit.

1. **Supertype error surface (`Result[T, Error]`) is rejected.** Architecture line 507 lists it as one of three accepted forms. `result_carries_rust_panic_error` uses a substring match on `sifr_type` (`panic_validation.rs:151-153`), so `Result[T, Error]` does not match and falls through to the rejection branch. The milestone status note narrows the claim to "explicit `RustPanicError`/`panic=map_error(...)`", so this is consistent with what was claimed — but the architecture document is normative and not updated. Either accept the supertype form (likely needs the type system's error-hierarchy lookup, not a substring) or update the architecture wording to drop the third bullet.
2. **`map_error(path)` mapper is not signature-validated.** Milestone scope (line 188) says: *"Validate `panic=map_error(...)` adapter signatures and mapper-panic fallback behavior."* `panic_validation.rs:135-140` only verifies the argument shape (`PolicyCall { name == "map_error", argument == TargetPath }`). The architecture (lines 516-521) requires: mapper resolves to a real Sifr or bridge function, is non-async/non-blocking, has the right input/output shape, AND that the public error channel can still hold `RustPanicError` after mapper failure. None of these is enforced. The status note attributes this deferral to `panic_boundary_wrapper_emission`, but see #3.
3. **`panic_boundary_wrapper_emission` is not a real fixture id.** M39.6/M39.7 status notes ("tracked by `opaque_resource_matrix`" / "tracked by `async_runtime_reqwest`") cite IDs that exist in `verification/areas/rust_interop/data/rust_interop_fixture_matrix.json`. `panic_boundary_wrapper_emission` does not — it appears only inline in the phase doc. Either add it as a `planned` fixture row in the matrix or pick an existing fixture to track the wrapper emission work, otherwise the deferral has no traceable owner.
4. **`Result[T, E]` + `panic=abort` is silently accepted.** Combined with abort, the `Err` channel can never be observed (process aborts first). Currently `validate_panic_declaration` accepts any `surface != None` on `Result`, so `Result[T, HashError] + panic=abort` requires only trust, no diagnostic. Architecture doesn't forbid it, but it's semantically nonsensical. Worth either a note in the architecture or an explicit rejection (Result + abort → require RustPanicError-aware surface or recommend non-Result return).

## Smaller suggestions (non-blocking)

- `result_carries_rust_panic_error` substring match (`panic_validation.rs:151-153`) is fragile — a user type literally named `MyRustPanicErrorThing` would satisfy. Tightening to a type-hierarchy lookup once #1 is addressed would fix both at once.
- `catch_rust_panic`'s global `Mutex` over the panic hook (`interop.rs:10`, `interop.rs:178-186`) serialises all panic-catching across threads and briefly suppresses *other* threads' panic hooks during the catch window. Documented intent, but worth a `// Why:` comment in the source so future maintainers don't try to remove it.
- `selected_panic_strategy` (`panic_validation.rs:163-192`) only checks `[profile.release]`. When Sifr eventually wires debug builds (test mode), this needs extension. Add a `// TODO` or just be aware.
- Diagnostic message-template inconsistency: the registry says `unresolved Rust target root {root}` but the source-emitter quotes the root in backticks (`rust_interop.rs:253`). Pre-existing across the family — flag for the family-wide cleanup, not this PR.
- `crates/sifr_driver/src/build/rust_interop.rs` is at 882 / 900 lines. The 8-line addition this PR was fine; next milestone in this file may need to split.

## Test coverage assessment

The 9 panic-contract tests + 4 runtime tests cover the claimed surface. Missing-but-not-required cases I'd add later:
- `Result[T, abort]` accepted-but-weird path (item #4 above).
- Lowering-level negative test: `def f() -> Result[bytes, SomethingElse]: ...` with `@rust(bridge.x)` and no policy → expect SIFR-RUST-PANIC-0001 at the driver/integration layer (lowering test currently only covers the positive `RustPanicError` case).

## Another round required?

**No.** The implementation is internally consistent and the deferrals are honestly disclosed. The four gaps above are addressable in a follow-up commit on this branch or as explicit punts during merge; none of them changes the shape of what's already landed.
