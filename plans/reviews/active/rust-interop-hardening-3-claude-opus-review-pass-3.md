## Review — `hardening_3`, round 3

Scope: working tree vs `HEAD` (`fa288b02c`), excluding the `ad-hoc-class-*` plan/review files and `internal_docs/typescript_go_architecture_transfer_guardrails.md`.

### Re-audit of the four round-2 findings

1. **Resolved.** `crates/sifr_lowering/src/lower/rust_interop_tests.rs:257-260` now asserts `"invalid Rust async contract: \`@rust.async(...)\` requires \`async def\`"`, matching `rust_interop.rs:117`. Ran `cargo test -p sifr_lowering --lib`: **831 passed, 0 failed, 1 ignored**.
2. **Resolved.** Evidence parsing moved to `_rust_test_evidence.py`. I verified independently in a temp tree that multi-line `#[cfg(all(\n feature = "special",\n unix\n))]`, file-level `#![cfg(feature = "special")]`, enclosing inline `mod` gates, and external `mod x;` gates are all collected; the two round-2 demo shapes now fail closed. Same-line and macro-generated forms yield `found 0` (conservative, not a hole).
3. **Partially resolved → finding 2 below.** `--skip` now matches the derived module path, but the derivation is wrong for `#[path]`-remapped modules.
4. **Resolved.** `_provenance_checks.py:389-398` requires `executes_cargo_probe` for positive `cargo-probe` bindings outside the generated-build suites, and the marker is an explicit `#[doc = "sifr-evidence: executes-cargo-probe"]`. Exactly three tests carry it (`rust_interop_contract_tests.rs:99`, `:261`, `rust_interop_async_contract_tests.rs:104`); each writes a real backend crate and drives `apply_package_rust_interop_metadata`. Name substrings no longer authorize anything — `..._reaches_probe_plan` and `..._clears_async_method_send_probe` remain bound only to contract-only rows.

Independently re-verified: 34 manifests at schema v2; 47 passing sides bound, 21 planned sides unbound and structurally forbidden from carrying `validation`; every bound side's README repeats test name + file + suite + profile exactly (script-checked, 0 mismatches); 41 of 47 derived cargo paths match `cargo test --list` exactly. Local bridge tests pass (`2 passed` in 15.7 s) and the positive asserts the real digest `[20, 38, 50, 184, 100, 68, 224, 154]` with an `assert_ne!` guard on the negative's manifest mutation. Area run: 7 variants / 0 failures. `check_fixture_matrix --self-test` `cases=53`, compatibility `cases=4`, tiers `cases=6`, stale-drafts ok, `sifr_verify --self-test` all green (incl. the Rust-interop profile-execution self-test), `sifr_driver --lib` 387 passed / 40 ignored, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, file-size guardrail (2826 files), `git diff --check` all pass. The `map_error` simplification of the `local_blake3_bridge` scenario is not a coverage loss — `map_error` remains covered by `panic_boundary`, `panic_boundary_wrapper_emission`, and `examples/blake3.sifr`.

### 1. MEDIUM — the `panic_boundary` negative binds a test that does not validate the claimed direction

`verification/areas/rust_interop/fixtures/panic_boundary/fixture.json:11-17` binds `panic_payload_not_exposed` to `catch_rust_panic_redacts_payload_details` (`crates/sifr_runtime/src/interop.rs:466`). That evidence side declares `expected_result: "diagnostic"`, `expected_diagnostic: "SIFR-RUST-PANIC-0001"`, and its source (`negative/panic_payload_not_exposed.sifr`) is a `Result[uint32, UserError]` return with **no `RustPanicError` surface and no panic policy** — a compile-time rejection. The bound test never compiles that shape and never emits `SIFR-RUST-PANIC-0001`; it asserts `catch_rust_panic(|| panic!("secret backend token")).message() == "Rust bridge panicked"`, a runtime redaction property.

The exactly-correct test already exists and is unbound: `package_rust_interop_result_requires_panic_surface` (`crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs:13-31`) asserts `diagnostics[0].code == "SIFR-RUST-PANIC-0001"` and `message.contains("RustPanicError")`. The fixture README even lists it first as the coverage for this case (`fixtures/panic_boundary/README.md:7`) while the structured record points elsewhere — so the README and the authoritative binding disagree about what proves the claim.

This is the milestone's own acceptance criterion ("every passing evidence direction … resolves to one exact Rust test") satisfied formally but not substantively; the validator cannot catch it because it only checks existence, ownership, and execution-kind source class. Fix: rebind the negative to `package_rust_interop_result_requires_panic_surface` (`sifr_driver_lib`, `create-pr`) and update the README provenance block. Note this frees `catch_rust_panic_redacts_payload_details`, which is currently reserved by the one-test-one-side rule.

### 2. LOW — `--skip` module-path matching is wrong for `#[path]`-remapped modules

`verification/areas/rust_interop/checks/_rust_test_evidence.py:145-161` derives the cargo test path purely from the file path under `src/`, ignoring `#[path = "..."] mod ...;` remapping. `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs` is declared as `mod rust_interop_build_tests;` inside `package_project_build_check.rs` (`crates/sifr_driver/src/tests/package_project_build_check.rs:315-316`), so its real cargo path is

```
tests::package_project_build_check::rust_interop_build_tests::test_build_local_bridge_blake3_positive_cargo_probe
```

(confirmed from `cargo test -p sifr_driver --lib -- --list`), while the validator derives `tests::package_rust_interop_build_tests::…`. All **6** generated-build bindings (`local_bridge_blake3`, `same_workspace_crate`, `shared_bridge_crate`, both sides each) are affected; the other 41 derive correctly. Consequence: a suite command carrying `--skip package_project_build_check` would silently exclude every tier-1 generated-build binding while the validator reports success — the exact hole round-2 finding 3 was meant to close, still open for these six. Latent today (the only filter in use is `test_e2e_pass`). Fix: resolve `#[path]` declarations when deriving the module path (or match the skip filter against the enclosing declared module chain), and add a `#[path]`-remapped mutation case.

### Non-blocking observations

- `_rust_test_evidence.py:279-282`: a Rust char literal `'"'` puts the string scanner into `in_string` state and blanks the rest of the file, so any test after it reports `found 0`. Fails closed, and no bound file contains that literal today.
- `_rust_test_evidence.py:184-189`: `#[cfg_attr(not(feature = "x"), ignore)]` is read as *requiring* `x` and as not-ignored — inverted, but conservative in the default (feature-off) direction. No bound test uses `cfg_attr`.
- `cargo clippy --workspace --all-targets -- -D warnings` fails with 28 pre-existing errors in unmodified `sifr_lowering` test files (`lower/compiler_intrinsics_tests.rs:24`, `lower/python_buffer_contract_tests.rs:742`, `name_resolution_snapshot_tests.rs:455`, …), all from commit `e219346a7f`. The documented gate is `cargo clippy --workspace -- -D warnings`, which passes; this is out of scope for the milestone but contradicts the round-2 note that `--all-targets` was green.
- Near-cap hand-maintained files: `rust_interop.rs` 896, `rust_interop_contract_tests.rs` 872, `check_fixture_matrix.py` 859, `_provenance_checks.py` 778. Under the 900 cap, little headroom for `hardening_4`.
- I did not run the full `scripts/run_all_tests.sh --profile create-pr` / merge lanes; I ran the area, all four area self-tests, the runner self-test, both affected crate suites, the ignored generated-build tests, and the four common gates.

Everything else in `hardening_3` is genuinely met: schema-v2 migration across all 34 manifests, two-sided distinct provenance for every claimed-support row, README removed as validator input, no planned evidence bound, the honest contract-only narrowing of `async_runtime_core`/`callback_subscription_core` with matching matrix/README/architecture text, real Cargo probing for package bridge roots, and tier-1 rows carrying executed build evidence.

Actionable findings: 2. CHANGES REQUIRED.
