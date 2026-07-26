I'll hold for the lane. Meanwhile, the review is complete — findings below.

## Review: `hardening_2` (Rust-interop verification matrix hardening)

### What I verified as genuinely working

- **Frozen tier/execution-kind table**: `ALLOWED_EXECUTION_KINDS` in `check_fixture_matrix.py:123-129` matches the plan's table exactly. The self-test enumerates all 5×4 tier/kind combinations with a *two-sided* assertion (`is_allowed == has_pair_failure`), so it would catch both over- and under-rejection. 25 cases, passes.
- **`check_tiers.py --self-test`** does real temporary mutation: it clones the checked-in matrix/TOML, renders mutated TOML to files under `tempfile.TemporaryDirectory`, and re-runs `_load_and_validate` on those paths. Covers missing assignment, duplicate assignment, matrix mismatch, invalid tier name, empty fixture list. It no longer silently runs the checked-in path.
- **`diagnostic_crate_rationale`** is present and byte-identical across all three copies for `direct_crate_negative_type` and `blocking_diagnostics`; `_validate_manifest_alignment` and `check_compatibility_matrix.py:125` cross-validate all three; `linked`/`executed` are pinned to `false`; rationale on a non-diagnostic row is rejected.
- **Tier-1 rows are real cargo probes.** I ran them: `cargo test -p sifr_driver --lib rust_interop_build_tests -- --ignored --test-threads=1` → 4 passed in 18.6s. The positive tests generate a package, link a real path dependency, run the binary and assert observed values; I independently recomputed FNV-1a of `"sifr-rust-interop"` = `1451903697411170458` and the hex of `"sifr"` = `73696672`, both matching. The `#[ignore]` is legitimate: `sifr_driver_generated_builds` is `blocking` with `--ignored` and `full` mode, and merge/nightly/release all set `"crate_tests": "full"`.
- Gates I ran clean: area run (4 suites, 0 failures), all four `--self-test` entrypoints, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, `check_file_size_guardrails.py` (`check_fixture_matrix.py` at 757/900), `git diff --check`.

### Actionable findings

**1. MEDIUM — the same-workspace negative test proves "unknown crate", not "undeclared workspace crate"**
`crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:113` builds the graph as `package_entrypoint_for(&dir, &app, &[&app], &[])` — the `workspace_hash` backend crate is **absent from the workspace entirely**, and the source targets a name (`undeclared_workspace_hash`) that exists nowhere. `backend_for_root` (`crates/sifr_driver/src/build/rust_interop/target_resolution.rs:27-37`) only searches declared backend edges, so this is the same "unresolved root" path already covered by `dotted_path_resolution` / `package_rust_interop_rejects_unknown_target_root`. The fixture's own first line claims "The compiler does not invent workspace fallback resolution" — that claim requires the crate to *be* in the workspace and still be rejected. Fix: include the backend in `packages` (`&[&app, &backend]`) while omitting the edge, and keep the root name `workspace_hash`.

**2. MEDIUM — the `positive` evidence records were relabeled `cargo-probe` but nothing builds them**
`fixtures/same_workspace_crate/positive/declared_path_dependency_resolves.sifr:4` and `fixtures/shared_bridge_crate/positive/stable_runtime_types_only.sifr:4` now carry `# execution-kind: cargo-probe` with `status: passing`, but the new tests `include_str!` the *scenario example* `src/main.sifr`, not these files (`package_rust_interop_build_tests.rs:6-8, 15-17`). The negative side does bind its evidence file (lines 9-11, 18-20), so the sides are asymmetric: the positive evidence records are still only text-linted. This is precisely the "claim stronger than executed evidence" the objective forbids. Either build the positive evidence files (they need a `main()`), or point the evidence records at the scenario example path.
Sub-note: `verify_workspace_hash_crate` is never called from `main()`, so `workspace_hash_pair` is linked but never executed — fine for a build probe, but the row shouldn't be read as runtime coverage of both bindings.

**3. LOW–MEDIUM — `diagnostic_family` now contradicts the fixture's only diagnostic**
`fixtures/same_workspace_crate/fixture.json:3` still says `"diagnostic_family": "SIFR-RUST-CARGO-0001"` while the negative evidence moved to `SIFR-RUST-RESOLVE-0001` (line 6). Every other fixture keeps the two aligned (`bridge_version_mismatch`, `cargo_locked_offline`), and `check_fixture_matrix.py:460` only checks the family is a *reserved* code, so nothing catches the drift. Update it to `SIFR-RUST-RESOLVE-0001`, and consider having the validator require `diagnostic_family` to equal the negative `expected_diagnostic` when the negative side is a diagnostic.

**4. LOW — the compatibility-matrix self-test has no positive control and only two cases**
`check_compatibility_matrix.py:169-226` asserts a substring appears for "missing rationale" and "mismatched rationale". Both are one-sided: an `_expect_equal` that failed unconditionally would still pass this self-test. Add a control row (identical rationale → zero failures). The same gap exists in `check_tiers.py:_run_self_test` — no case asserts the unmutated data yields no failures.

**5. LOW — malformed rationale is unvalidated when a diagnostic row lists no crates**
`check_fixture_matrix.py:264-266`: `_validate_diagnostic_crate_rationale` is only invoked `if has_crates`. A `compiler-diagnostic` row with `required_crates: []` and `diagnostic_crate_rationale: {"junk": 1}` passes all three files. Validate the shape whenever the field is present on a diagnostic row.

**6. LOW — the READMEs name `#[ignore]`d tests with no reproduction command**
`fixtures/same_workspace_crate/README.md:6-13` and `fixtures/shared_bridge_crate/README.md:7-15` name the four test functions, but `cargo test -p sifr_driver` silently skips all of them, and `verification/areas/rust_interop/README.md:95-101` documents create-PR as the profile that runs this area — which uses `"crate_tests": "smoke"` (`verification/profiles/create-pr.json:253`) and therefore does **not** execute this evidence. Record the actual reproduction (`cargo test -p sifr_driver --lib <name> -- --ignored`) and the owning suite/profile (`sifr_driver_generated_builds`, merge/nightly/release full mode) so the cargo-probe claim is traceable before `hardening_3` formalizes provenance.

**7. LOW — the test re-authors `sifr.toml` instead of consuming the fixture's**
`package_rust_interop_build_tests.rs:22-46` hand-writes a manifest that is content-identical to `fixtures/*/examples/*/sifr.toml` (including the same package names and trust targets). The scenario checks lint the checked-in manifest; the build proves a hand-copied duplicate. A future trust-target or `[rust]` change to the fixture manifest would leave the build evidence silently stale. `include_str!` the fixture `sifr.toml` instead.

### Informational (not `hardening_2` scope)

`check_stale_drafts.py --self-test` still prints `rust interop stale draft scan ok` — it ignores the flag and runs the ordinary checked-in-data path, the exact anti-pattern `hardening_2` fixed for `check_tiers.py`. It is listed in the issue's Required Validation block, so it currently gives false self-test signal. `hardening_4` owns that file; worth an explicit line in that item.

### Exit-criteria status

`hardening_2`'s bullets are all present, and the docs (`docs/rust-interop.mdx:49-62`, `internal_docs/rust_interop_architecture.md:1004-1023`, area README §Tier And Execution Semantics, tier descriptions) consistently state the frozen table and that contract-only never satisfies a runtime claim. The mutation-coverage requirement ("reject every disallowed pair, missing/mismatched rationale, a rationale on a non-diagnostic row, and a tier-1 row downgraded to contract-only") is met. **Not approved as-is**: findings 1 and 2 mean two of the four tier-1 evidence directions do not execute what their labels claim, which is the issue's central invariant.
