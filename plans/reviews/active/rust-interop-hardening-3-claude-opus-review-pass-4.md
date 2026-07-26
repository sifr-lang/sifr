## Review — `hardening_3`, round 4

Scope: working tree vs `HEAD` (`fa288b02c`), excluding the `ad-hoc-class-*` plan/review files and `internal_docs/typescript_go_architecture_transfer_guardrails.md`. No files were modified.

### Re-audit of the two round-3 findings

**Round-3 finding 1 (panic_boundary negative) — resolved.** `fixtures/panic_boundary/fixture.json:11-17` now binds `panic_payload_not_exposed` to `package_rust_interop_result_requires_panic_surface` (`rust_interop_panic_contract_tests.rs:13-31`), which asserts `diagnostics[0].code == "SIFR-RUST-PANIC-0001"` and `message.contains("RustPanicError")` — the exact claimed direction. `fixtures/panic_boundary/README.md:19-20` repeats the binding verbatim (`sifr_driver_lib`, `create-pr`). Ran the exact test: `1 passed`. `catch_rust_panic_redacts_payload_details` is now referenced only in explanatory prose, not as a binding.

**Round-3 finding 2 (`#[path]` module-path derivation) — resolved.** `_rust_test_evidence.py:180-198` resolves `#[path]` declarations recursively with a `seen` cycle guard, and `_path_module_declarations` (`:213-253`) now carries the declaration's cfg features so remapped gates are inherited. I re-derived all 47 bound paths through the real `rust_test_path()` and diffed them against `cargo test --list` across `sifr_driver`/`sifr_runtime`/`sifr_lowering`/`sifr_codegen`/`sifr_package`/`sifr_hir`/`sifr`: **47 derived, 0 missing** (was 41 mismatched). Spot-checked the hard cases directly — `tests/package_rust_interop_build_tests.rs → tests::package_project_build_check::rust_interop_build_tests`, `build/rust_interop/probe_planning.rs → build::rust_interop::probe_planning`, `stdlib/bootstrap_tests.rs → stdlib::bootstrap::tests`. Both new self-test cases exist (`_provenance_checks.py:638-658`).

### Independently re-verified

47 bindings, all distinct (0 shared tests); 34 manifests at schema v2; every non-passing side carries no `validation`; every passing side carries one; every `validation` object has exactly the five allowed fields; all 47 READMEs repeat test name + file + suite + profile exactly (script-checked, 0 mismatches). Execution-kind source classes are coherent: all passing `runtime-observed` rows bind `sifr_runtime`, all `cargo-probe` positives bind either a generated-build suite or one of the three `#[doc = "sifr-evidence: executes-cargo-probe"]` tests. Generated-build bindings correctly resolve to `merge` (the suite is `full`-mode only).

Gates run: `check_fixture_matrix --self-test` **cases=55**; `check_compatibility_matrix --self-test` cases=4; tiers cases=6; stale-drafts ok; full area **7 variants / 0 failures**; `sifr_driver --lib` **387 passed / 0 failed / 40 ignored**; `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, file-size guardrail (2826 files), HIR guardrails, `git diff --check` all pass. Parser checks are fast (full fixture check 0.9 s). **`scripts/run_all_tests.sh --profile create-pr` ran to completion: `EXIT=0`, with `name=rust_interop_checks elapsed_ms=1774 budget_ms=5000 status=pass`** — the milestone's exit gate.

I also tested the `local_bridge_blake3` scenario simplification directly: copying both the `HEAD` (`map_error`/`Result`) and current (`trusted_no_panic`) scenarios to `/tmp` and running `sifr bridge check` — **both pass**, so the rewrite was not needed to make the new package-bridge Cargo probe succeed. It is a genuine strengthening (projection-level → real build + run asserting the digest `[20, 38, 50, 184, 100, 68, 224, 154]`, with an `assert_ne!` guard on the negative's manifest mutation), not a workaround.

---

### 1. MEDIUM — `opaque_handle_tokenizer`'s negative claims a diagnostic its bound test disproves

`fixtures/opaque_handle_tokenizer/fixture.json:6` declares `expected_diagnostic: "SIFR-RUST-HANDLE-0001"` (matching `diagnostic_family` at `:35` and the source header `negative/unsatisfied_send_or_copy_rejected.sifr:6`). The bound test (`fixture.json:16` → `rust_interop_contract_tests.rs:333-334`) asserts the opposite:

```rust
assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
assert!(diagnostics[0].message.contains("Rust bridge probe failed"));
```

That is correct behavior: per `crates/sifr_diagnostics/src/codes/registry/registry_entries/rust_interop.rs:41-61`, `SIFR-RUST-HANDLE-0001` is "Rust opaque handle contract is invalid" (emitted for close-policy violations — see `rust_interop_contract_tests.rs:404`, `:446`, `:473`), while an unsatisfied `Send`/`Copy` probe obligation is `SIFR-RUST-TYPE-0001`. The fixture's negative source satisfies the close contract (`close=close` plus a `close(own self)` method), so the shape it declares can only produce `TYPE-0001`.

So the evidence record's declared diagnostic is false, and hardening_3 has now formally certified it: the validator only cross-checks `diagnostic_family` against `expected_diagnostic` (both wrong together) and never compares either against what the bound test asserts. This is the same defect class as round-3 finding 1 — the milestone criterion satisfied formally, not substantively.

Fix: correct the negative's `expected_diagnostic` (and reconcile `diagnostic_family`/the `.sifr` header/README), or rebind the negative to a shape that genuinely emits `SIFR-RUST-HANDLE-0001`.

### 2. MEDIUM — the round-3 finding-1 hole was closed in data only; the validator still permits it

`_provenance_checks.py:374-405` constrains `runtime-observed` (must be a `crates/sifr_runtime/` test), positive `cargo-probe` (must carry the probe marker), and `compiler-diagnostic` (must **not** be a runtime test). `contract-only` has no source-class rule at all — which is exactly why round 3's `panic_boundary` negative could bind `crates/sifr_runtime/src/interop.rs::catch_rust_panic_redacts_payload_details` and pass every check.

The binding was corrected but the gate was not, so the identical mistake re-lands silently on any of the ten contract-only rows. No contract-only row binds a runtime test today, so adding the symmetric rule (`contract-only` evidence must not resolve to `crates/sifr_runtime/`) is a pure guard with no data churn, plus one mutation self-test. The milestone's stated purpose is mechanical enforcement of exactly this ("README-only evidence, broad module names, missing/ignored tests … are rejected"); leaving the hole open makes the round-3 fix a one-off data patch rather than a root-cause fix.

### 3. LOW — two runtime-observed rows declare a compile-time-only diagnostic that cannot be proven

`fixtures/close_after_use/fixture.json:6-7` and `fixtures/opaque_resource_core/fixture.json:6-7` both declare `expected_result: "diagnostic"` with `expected_diagnostic: "SIFR-RUST-HANDLE-0001"` on `execution_kind: "runtime-observed"` rows. `SIFR-RUST-HANDLE-0001` is owned by `sifr_driver::build::rust_interop` (registry entry cited above), but `_provenance_checks.py:384-389` *requires* runtime-observed evidence to bind a `crates/sifr_runtime/` test. The bound tests (`interop.rs:480-486`, `:451-463`) assert `HandleStateError::Closed` / `Poisoned` and emit no diagnostic at all.

The combination is structurally unsatisfiable under the milestone's own rules: the declared claim can never be validated by the test the rules force you to bind. The substantive runtime behavior *is* proven; only the declared expectation is mismodeled. Fix: express these as runtime error-state expectations rather than `expected_result: "diagnostic"` + a driver diagnostic code, and reject that pair in `check_fixture_matrix.py`.

---

### Non-blocking observations

- `fixtures/direct_crate_negative_type/fixture.json:11` binds the negative to `direct_negative_type_stops_before_cargo_probe_execution` (`rust_interop_evidence_contract_tests.rs:45-52`), which asserts `diagnostics.len() == 1` and absence of `"Rust bridge probe failed"` but never asserts `SIFR-RUST-TYPE-0001` itself. The sibling positive binding does, so the family is covered — but the negative side alone would not notice a code change.
- `fixtures/panic_boundary/positive/result_declares_rust_panic_error_or_map_error.sifr:9` uses `panic=map_error(...)`; the bound test proves only the `RustPanicError`-surface disjunct. Acceptable for an "or" claim (and `..._accepts_map_error_surface` still executes in the same suite), but the fixture source and the binding exercise different shapes.
- `fixtures/local_bridge_blake3/examples/blake3.sifr:9` still references `bridge.blake3.map_panic`, which this change deleted from the scenario's `src/bridges/blake3.rs`. Package examples aren't compiled, so this is a dangling illustrative reference only.
- README canonical-provenance blocks match all 47 bindings today, but nothing enforces that (by design — README was removed as validator input). A cheap `--self-test`-only consistency check would prevent the exact README/binding disagreement round 3 caught.
- `_provenance_checks.py:348-371` validates `--skip` filters only. Positive libtest name filters and `--exact` are unhandled; no suite command uses either today, so this is latent with zero current instances.
- `crates/sifr_driver/src/build/rust_interop.rs:302-312` embeds a `let ... else { return }` inside a struct-literal field initializer. It is correct and fails closed (diagnostic pushed before the return), but hoisting it above the `match` would read better.
- Round-3's `_rust_test_evidence.py` notes still hold: a Rust `'"'` char literal blanks the remainder of a file (`:376-379`), and `cfg_attr(not(feature = ...), ignore)` is read inverted (`:281-286`). Both fail closed; no bound file hits either.
- Near-cap hand-maintained files: `rust_interop.rs` 896, `rust_interop_contract_tests.rs` 872, `check_fixture_matrix.py` 859, `_provenance_checks.py` 817. Findings 2 and 3 add lines to `_provenance_checks.py` and `check_fixture_matrix.py` — both need a split rather than growth.

Everything else in `hardening_3` is met: schema-v2 migration across all 34 manifests, two-sided distinct provenance for every claimed-support row, README removed as validator input, no planned evidence bound, the honest contract-only narrowing of `async_runtime_core`/`callback_subscription_core` with matching matrix/README/architecture text, real Cargo probing for package bridge roots (conservative: missing sysroot runtime crate emits `SIFR-RUST-CARGO-*` and aborts resolution), and tier-1 rows carrying executed build evidence.

Actionable findings: 3. CHANGES REQUIRED.
