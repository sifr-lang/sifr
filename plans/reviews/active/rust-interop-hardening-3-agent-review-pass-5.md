## Review — `hardening_3`, round 5

Scope: working tree vs `HEAD` (`fa288b02c`), excluding the `ad-hoc-class-*` plan/review files, `internal_docs/typescript_go_architecture_transfer_guardrails.md`, and phase-40 artifacts. No files modified.

### Re-audit of the three round-4 findings — all resolved

**Round-4 finding 1 (opaque_handle_tokenizer) — resolved.** `fixtures/opaque_handle_tokenizer/fixture.json:3,6` now declare `SIFR-RUST-TYPE-0001`, the source header at `negative/unsatisfied_send_or_copy_rejected.sifr:6` matches, and `README.md:10-14` explains the code correctly. The bound test `rust_interop_contract_tests.rs:331-332` asserts exactly `SIFR-RUST-TYPE-0001` + `"Rust bridge probe failed"`. `direct_negative_type_stops_before_cargo_probe_execution` (`rust_interop_evidence_contract_tests.rs:50`) now asserts the code itself — the round-4 non-blocking observation is closed too.

**Round-4 finding 2 (contract-only source class) — resolved in the gate.** `_provenance_checks.py:400-405` now rejects `contract-only` alongside `compiler-diagnostic` for `crates/sifr_runtime/` bindings; I confirmed by direct invocation that both kinds are rejected on both sides. Mutation coverage at `:732-747`.

**Round-4 finding 3 (unprovable diagnostic on runtime rows) — resolved.** Outcome semantics moved to `_evidence_expectations.py`; `close_after_use` and `opaque_resource_core` negatives now declare `runtime-error-state` + `closed`/`poisoned` with matching source headers, and their READMEs say "runtime `HandleStateError` observation rather than compiler diagnostics." The bound tests (`sifr_runtime/src/interop.rs:451-463`, `:480-486`) assert exactly those states. Mutation cases at `_evidence_expectations.py:107-165` cover runtime-diagnostic, non-runtime error state, missing/wrong/cross-kind fields.

### Independently re-verified

68 evidence directions; 47 passing with distinct provenance (0 shared tests), 21 planned with no `validation`. I re-derived every bound test body and cross-checked the declared `expected_diagnostic` against the assertion: **all 15 diagnostic bindings match** (9 via literal, 6 via `DiagnosticCode::` constants — `RUST_ASYNC_CONTRACT`, `RUST_CONFIG_MALFORMED_DECORATOR` via `assert_malformed`, `RUST_RESOLVE_TARGET_ROOT` — all confirmed against `registry.rs:64-70`). All 47 README canonical-provenance sentences match `fixture.json` byte-for-byte (script-checked, 0 mismatches).

Gates run: fixture matrix **cases=63** / `fixtures=34`; compatibility `cases=4` / `rows=34`; tiers `cases=6`; full area **7 variants, 0 failures**; `sifr_verify --self-test` all pass; `sifr_driver --lib` 387/0; `sifr_runtime` 55/0; `sifr_lowering` 831/0; the ignored generated-build suite **7 passed / 0 failed in 62s** (real Cargo builds for all three tier-1 rows plus the crc32 link test); `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `git diff --check`, file-size guardrail (2827 files) all pass. (`clippy --all-targets` fails in `sifr_codegen` — untouched by this change, pre-existing, out of scope.)

---

### 1. BLOCKER — the round-4 finding-2 fix breaks `scripts/run_all_tests.sh --profile create-pr`

`verification/areas/rust_interop/checks/_provenance_checks.py:732,734,735,745` introduce the identifiers `contract_source_failures` and the fixture-id string `"contract_source"`. These match the banned pattern `\bcontract_[a-z][a-z0-9_]*\b` in `verification/areas/coverage_matrix/checks/verification_taxonomy.py:127+`, producing four errors:

```
verification-taxonomy error: verification/areas/rust_interop/checks/_provenance_checks.py:732: line contains delivery-plan taxonomy: contract_source_failures: list[str] = []
... :734, :735, :745
```

`coverage_matrix_checks` is the **first blocking step** of the create-PR lane, so the lane aborts there:

```
[sifr-lane-step] name=coverage_matrix_checks elapsed_ms=11070 status=fail
slowest_step=coverage_matrix_checks 11070ms status=fail
```

`rust_interop_checks` never executes. This fails the milestone's own Required Validation list and contradicts `hardening_1`'s standing exit gate (`must print name=rust_interop_checks ... status=pass`). Round 4 reported this lane green; the regression was introduced by the round-4 corrections themselves and the lane was evidently not re-run afterward.

Fix: rename the four identifiers to spellings that don't match `contract_[a-z]` (e.g. `runtime_source_contract_failures`, fixture id `"contract-only-source"` — the hyphen does not trip the pattern), then re-run the full create-PR lane.

### 2. MEDIUM — `cargo-probe` negative evidence may still bind a `crates/sifr_runtime/` test

`_provenance_checks.py:384-405` is an `elif` chain. For `execution_kind == "cargo-probe"` with `side == "negative"`, none of the three branches fire, so no source-class rule applies. Verified directly:

```
cargo-probe   positive  runtime-test-source -> ['... must use an explicit probe test']
cargo-probe   negative  runtime-test-source -> ACCEPTED
```

That means any of the eight `cargo-probe` negative rows (`bridge_type_matrix`, `direct_crate_crc32`, `direct_crate_matrix`, `async_ecosystem_matrix`, `opaque_handle_tokenizer`, `local_bridge_blake3`, `same_workspace_crate`, `shared_bridge_crate`) could be rebound to, say, `sifr_runtime/src/interop.rs::catch_rust_panic_redacts_payload_details` and pass every gate — a unit test that proves nothing about Cargo probing certifying a tier-1/tier-3 build claim. This is the exact hole round 4 closed for `contract-only`, left open on the one remaining side. No current row hits it, so the fix is a pure guard with no data churn:

```python
if execution_kind == "runtime-observed":
    if not test_file.startswith("crates/sifr_runtime/"): fail(...)
else:
    if test_file.startswith("crates/sifr_runtime/"): fail(...)
    if execution_kind == "cargo-probe" and side == "positive" and not (...): fail(...)
```

plus one mutation case. This also shrinks the function rather than growing it (`_provenance_checks.py` is at 834/900).

### 3. MEDIUM — nothing mechanically checks that a bound test asserts the declared outcome

`_evidence_expectations.py:42-78` validates `expected_diagnostic`/`expected_runtime_state` against the reserved-code set, the source header, and the execution kind — but never against the Rust test the record binds. That gap is the direct cause of round-3 finding 1 (`panic_boundary` bound to a test asserting a different thing) and round-4 finding 1 (`opaque_handle_tokenizer` declaring `HANDLE-0001` while its test asserts `TYPE-0001`). Both were corrected in data only; a third instance of the same class would again pass every gate and require a human to catch it.

I verified all 47 bindings are correct today, so this too is a pure guard. It is mechanizable at modest cost: parse `crates/sifr_diagnostics/src/codes/registry.rs` for `pub const NAME: Self = Self::new("SIFR-...")`, extract the bound test body plus any locally-called helper (only `assert_malformed` and `unsupported_container_diagnostics` are needed today), and require the declared code — as literal or constant — to appear; likewise require `HandleStateError::Closed`/`Poisoned` for the two `runtime-error-state` rows. Given both target files are near the 900-line cap, this belongs in a new focused module alongside `_evidence_expectations.py`.

---

### Non-blocking observations

- `crates/sifr_driver/src/build/rust_interop.rs:302-312` still embeds `let ... else { return }` inside a struct-literal field initializer. Correct and fails closed (`probe_planning.rs:60-72` pushes `SIFR-RUST-CARGO-*` before returning), but hoisting it above the `match` would read better.
- `close_after_use` and `opaque_resource_core` retain `diagnostic_family: "SIFR-RUST-HANDLE-0001"` while neither side claims a diagnostic. The field is a row-family label, not an evidence claim, and `_validate_diagnostic_family_alignment` correctly skips non-diagnostic negatives — but it is now unverified on those two rows.
- `fixtures/panic_boundary/positive/result_declares_rust_panic_error_or_map_error.sifr:9` and `fixtures/local_bridge_blake3/examples/blake3.sifr:9` (dangling `bridge.blake3.map_panic` reference) are unchanged from round 4; both remain illustrative-only.
- `_provenance_checks.py:348-371` still validates `--skip` filters only; positive libtest filters and `--exact` are unhandled, with zero current instances.
- Near-cap hand-maintained files: `rust_interop.rs` 896, `rust_interop_contract_tests.rs` 872, `check_fixture_matrix.py` 861, `_provenance_checks.py` 834.

Everything else in `hardening_3` is met: schema-v2 across all 34 manifests, two-sided distinct executable provenance for all 47 passing directions, README removed as validator input while staying consistent, no planned evidence falsely bound, outcome semantics that a runtime-observed row can actually prove, and tier-1 rows carrying real executed Cargo-build evidence.

Actionable findings: 3. CHANGES REQUIRED.
