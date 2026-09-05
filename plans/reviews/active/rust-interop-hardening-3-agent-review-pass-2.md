## Review — `hardening_3`, round 2

Scope: uncommitted working tree vs `HEAD` (`fa288b02c`, hardening_2 merged) — 85 files plus untracked `_provenance_checks.py`, `rust_interop_evidence_contract_tests.rs`, `rust_interop_probe_manifest.rs`. The `ad-hoc-class-*` plan/review files were excluded.

### Re-audit of round-1 findings — all seven resolved

1. **Resolved.** `async_runtime_core` is now tier-2 `contract-only` in the fixture matrix (`data/rust_interop_fixture_matrix.json:187`), compatibility matrix, `fixture.json`, both evidence source headers, README, and both architecture docs; the negative binds `rust_interop_hidden_blocking_async_resource_evidence_is_rejected` (`crates/sifr_lowering/src/lower/rust_interop_tests.rs:334`), which mirrors the fixture's actual rejected declaration (`sifr_stdlib.async_core.hidden_blocking_wait`, `@blocking_io`) and asserts the exact message rather than only the code. `_validate_execution_kind_source` (`_provenance_checks.py:554`) now forces `runtime-observed` onto `crates/sifr_runtime/**` and blocks runtime tests from certifying `compiler-diagnostic` rows.
2. **Resolved.** `local_bridge_blake3` binds two new distinct ignored generated-build tests (`crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:123`, `:143`) at `sifr_driver_generated_builds`/`merge`; the compiler now genuinely Cargo-probes package bridge roots (`build/rust_interop.rs:301-313` → `build/rust_interop/probe_planning.rs:57-98`, with `source_prefix` threaded through `rust_interop_probe.rs:239-248`). I ran both: `2 passed` in 15.5 s. `sifr check` on the pristine scenario is clean.
3. **Resolved.** `callback_subscription_core` is `contract-only`, bound to two distinct driver callback-contract tests; no `sifr_runtime_python` suite exists anywhere in `verification/profiles/*.json` (verified by grep), so no pyo3 build entered the PR lane.
4. **Resolved for in-file single-line gates.** `_rust_test_definitions` collects `#[cfg(feature = …)]` on the test and enclosing inline `mod`, `_external_module_features` walks `mod x;` declarations, and `_enabled_cargo_features` resolves `--features`/`--no-default-features`/`--all-features` through the manifest feature closure; two mutation cases cover it. (Residual gap → finding 2.)
5. **Resolved.** `_validate_command_filters` (`:522`) rejects `--skip` filters, with a mutation case. (Residual gap → finding 3.)
6. **Resolved.** Mutation coverage now includes weakest-profile, package ownership, `step`, and field-set, plus a positive ignored-binding control; self-test reports `cases=49`.
7. **Resolved.** The conditional-expression trap is gone, replaced by `_suite_selected` (`:215`) shared by `_validate_suite_selection` and `_weakest_executing_profile`.

Independently re-verified: 34 manifests are schema v2, 47 bound / 11 unbound sides, every bound side's README repeats test name + file + suite + profile exactly (script-checked), no stale reference to the renamed `package_rust_interop_async_current_thread_allows_non_send_future` remains, and the three cargo-probe positives bound outside the generated-build suites (`..._direct_probe_accepts_bridge_signature`, `..._opaque_probe_accepts_declared_send_sync_copy`, `..._async_probe_current_thread_allows_non_send_future`) each write a real crate and invoke rustc. Fixture/compatibility/tier self-tests, the full area (7 variants, 0 failures), `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, file-size guardrail, `git diff --check`, `sifr_driver --lib` (387 passed / 40 ignored) and `-p sifr` (`--skip test_e2e_pass`, all green) all pass.

### 1. HIGH — a blocking-suite test is broken by this change, so the exit gate cannot pass

`crates/sifr_lowering/src/lower/rust_interop_tests.rs:255-260`. This patch strengthened `rust_interop_rejects_async_decorator_on_sync_function` to assert `error.message == "invalid Rust async contract: Rust async interop cannot be combined with blocking or CPU-heavy classification"`, but that input (`@rust.async` on a **sync** `def`) emits `"invalid Rust async contract: `@rust.async(...)` requires `async def`"` (`crates/sifr_lowering/src/lower/rust_interop.rs:117`); the pasted message belongs to `rust_interop.rs:130`. Result:

```
cargo test -q -p sifr_lowering --lib -- --skip test_e2e_pass
test result: FAILED. 830 passed; 1 failed; 1 ignored
---- lower::rust_interop_tests::rust_interop_rejects_async_decorator_on_sync_function
panicked at crates/sifr_lowering/src/lower/rust_interop_tests.rs:257:5
```

`sifr_lowering` is a blocking suite in all four profiles (`verification/profiles/create-pr.json`, modes `smoke`+`full`), so `scripts/run_all_tests.sh --profile create-pr` and the merge gate both fail, and the milestone's required validation list is not satisfied. The test is also **not** bound to any fixture evidence, so the edit was unnecessary scope; and the focused-evidence list supplied for this review omits the one suite this patch touched outside `sifr_driver`, which is why the break went unreported. Fix: assert the `requires `async def`` message (or revert that hunk), then run the `sifr_lowering` suite and the create-PR lane before re-claiming evidence.

### 2. LOW-MEDIUM — feature-gate detection still misses multi-line and file-level `cfg`

`_provenance_checks.py:296-306` only accumulates attribute lines whose stripped text starts with `#[`, and any non-`#[`/non-`fn`/non-`mod` line clears the accumulator. Demonstrated against the module's own helpers in a temp tree:

- `#[cfg(all(\n feature = "special",\n unix\n))]` above `#[test] fn gated_multiline()` → accepted, `failures == []`.
- `#![cfg(feature = "special")]` at file top above `#[test] fn inner_gated()` → accepted, `failures == []`.

Either form lets a binding certify a test that no blocking suite compiles — the exact hole round-1 finding 4 closed for the single-line form. Latent today (no bound file uses either shape; `crates/sifr_runtime/src/interop.rs:488`'s `net`-gated test is not bound). Fix: join continuation lines into one logical attribute, treat `#![cfg(...)]` as a file-level gate, and add both as mutation cases.

### 3. LOW — `--skip` matching ignores module paths

`_provenance_checks.py:534-542` compares each `--skip` value against `test_name` only, but cargo's filter matches the full test path (`lower::rust_interop_tests::foo`). A suite command carrying `--skip rust_interop_tests` would silently exclude every bound test in that module while the validator reports success. Latent (today's filters are `test_e2e_pass`). Fix: match the skip filter against `<module path derived from test_file>::<test_name>` as well.

### 4. LOW — the positive cargo-probe "probe marker" is a name substring

`_provenance_checks.py:560-569` accepts any positive `cargo-probe` binding whose `test_name` contains `probe`. Two currently bound tests prove the substring does not imply probing: `package_rust_interop_view_send_sync_metadata_reaches_probe_plan` and `package_rust_interop_opaque_current_thread_clears_async_method_send_probe` only inspect `interop.rust.probe_plan`. Both are bound to `contract-only` rows today, so no row is currently mis-certified, but the rule admits a plan-only test for a tier-1/3 build claim. Fix: gate on an explicit marker the test itself carries (e.g. a `probe_executes` attribute list or restriction to the generated-build suites) rather than a name substring.

### Non-blocking observations

- The `local_bridge_blake3` positive asserts only `run_built_package(...) == ""`; the sibling tier-1 rows assert real digests (`"1451903697411170458"`, `"4e138d18e63ba405"`). Printing the hash would make the bridge's value observable at no extra cost. Relatedly, the rewritten scenario `examples/local_blake3_bridge/src/main.sifr` is overwritten by `install_evidence_source` and so never compiled by a test (siblings build both roots); I verified manually that it checks clean.
- The negative test's `manifest.replace(...)` of `rust-no-panic` (`package_rust_interop_build_tests.rs:155`) is a silent no-op if that line ever changes; asserting the replacement occurred would be cheap.
- Near-cap hand-maintained files after the decomposition: `rust_interop_tests.rs` 899, `rust_interop.rs` 896, `_provenance_checks.py` 883, `rust_interop_contract_tests.rs` 870, `check_fixture_matrix.py` 859 — the guardrail passes, but `hardening_4` has almost no headroom in `check_fixture_matrix.py`/`_provenance_checks.py`.

Everything else in the milestone — schema v2 migration, two-sided provenance for every claimed row, README removed as validator input, no planned evidence bound, honest narrowing of the two tier-2 rows with matching docs, and the tier-1 rows carrying real build evidence — is genuinely met. Only finding 1 blocks.

Actionable findings: 4. CHANGES REQUIRED.
