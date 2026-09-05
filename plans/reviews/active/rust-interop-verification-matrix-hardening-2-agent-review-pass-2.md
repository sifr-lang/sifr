# Review Pass 2: `hardening_2` (Rust-interop verification matrix hardening)

Scope: current working diff for
`plans/issues/active/rust-interop-verification-matrix-hardening.md` item
`hardening_2`, re-reviewed after the pass-1 fixes. The
`ad-hoc-class-field-mutating-receiver-place-semantics` issue and its review
files are out of scope and were ignored.

## Pass-1 findings: all seven resolved

1. **Undeclared workspace-member negative (MEDIUM) — resolved.**
   `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:101` now
   builds the graph as `package_entrypoint_for(&dir, &app, &[&app, &backend], &[])`.
   `package_graph` emits every listed package into `workspace_members`
   (`package_project_build_check.rs:214`), so `workspace_hash` really is a
   workspace member; only the dependency edge is missing. The negative source
   targets the real root `workspace_hash.hash`
   (`negative/undeclared_workspace_crate_rejected.sifr:7`), so the test now
   proves "present in the workspace, undeclared, still rejected" rather than
   "unknown crate".
2. **Positive evidence built (MEDIUM) — resolved.** Both positive evidence
   files are `include_str!`-bound and executed:
   `SAME_WORKSPACE_POSITIVE` (line 9) builds and runs, asserting
   `1451903697411170458`; `SHARED_BRIDGE_POSITIVE` (line 21) builds and runs,
   asserting `17`. Both sides of both fixtures are now symmetric.
3. **Diagnostic family alignment (LOW–MEDIUM) — resolved.**
   `same_workspace_crate/fixture.json:3` is `SIFR-RUST-RESOLVE-0001`, and the
   drift class is now mechanically closed:
   `_validate_diagnostic_family_alignment` (`check_fixture_matrix.py:566`)
   requires `diagnostic_family == negative.expected_diagnostic`, with a
   self-test case.
4. **Positive controls (LOW) — resolved.** `check_compatibility_matrix.py:169`
   and `check_tiers.py:_run_self_test` both assert unmutated data yields zero
   failures before running mutations.
5. **Rationale validated without crates (LOW) — resolved.**
   `_validate_execution_semantics` now validates the rationale shape whenever
   the field is present on a diagnostic row (`check_fixture_matrix.py:265`),
   with a `diagnostic_empty_crates_malformed` self-test case.
6. **Ignored-test reproduction and profile ownership (LOW) — resolved.** Both
   fixture READMEs record the exact
   `cargo test -p sifr_driver --lib <name> -- --ignored --test-threads=1`
   commands and name `sifr_driver_generated_builds`; that suite is blocking,
   `modes: ["full"]`, and includes `--ignored`
   (`verification/profiles/merge.json:67`, `nightly.json:69`,
   `release.json:68`), matching the stated merge/nightly/release ownership.
7. **Canonical manifest consumed (LOW) — resolved.** `install_fixture_manifest`
   writes the checked-in `sifr.toml` from each fixture scenario
   (`package_rust_interop_build_tests.rs:6,18,28`); the hand-written duplicate
   is gone.

## Gates run in this pass

- area run: 4 suites, 0 failures (34 fixtures, 34 rows).
- all four `--self-test` entrypoints: fixture matrix 27 cases, compatibility 3,
  tiers 6, stale drafts (see informational note).
- `cargo test -p sifr_driver --lib rust_interop_build_tests -- --ignored --test-threads=1`
  → 4 passed in 19.8s.
- `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`,
  `check_file_size_guardrails.py` (PASS; `check_fixture_matrix.py` 813/900),
  `git diff --check`.
- Not re-run in this pass: `scripts/run_all_tests.sh --profile create-pr` and
  the default merge gate.
- Pre-existing and unrelated: `cargo clippy --workspace --all-targets` fails in
  `sifr_lowering`/`sifr_driver` *test* code (e.g.
  `crates/sifr_lowering/src/lower/compiler_intrinsics_tests.rs:24`). No lint is
  attributable to this diff; the repo's canonical clippy command is clean.

## Exit criteria for `hardening_2`

All met: frozen table encoded and two-sidedly self-tested; real
`check_tiers.py --self-test` on temporary mutated data; three-copy
`diagnostic_crate_rationale` cross-validated; both diagnostic rows migrated;
`same_workspace_crate` and `shared_bridge_crate` are executed `cargo-probe`
rows; matrices, manifests, evidence headers, tier descriptions, READMEs,
architecture, and public docs updated in the same change. The required mutation
coverage (disallowed pair, missing/mismatched rationale, rationale on a
non-diagnostic row, tier-1 downgrade) is present and passing.

## Remaining actionable findings (all LOW, none blocking)

1. **README misattributes which source is compiled.**
   `fixtures/same_workspace_crate/README.md:7-9` and
   `fixtures/shared_bridge_crate/README.md:8-10` say the test "compiles the
   checked-in scenario source". The tests compile the *evidence* files
   (`positive/declared_path_dependency_resolves.sifr`,
   `positive/stable_runtime_types_only.sifr`). In this area "scenario example"
   is a term of art for `examples/<name>/`, and that source is never compiled —
   `_scenario_checks.py` only token-lints it. Say "checked-in positive evidence
   source" instead. Related: both scenario examples now carry
   `# execution-kind: cargo-probe` and newly added `main()` bodies with
   assertions (`examples/workspace_hash_crate/src/main.sifr:3,18`;
   `examples/shared_hash_bridge/src/main.sifr:3,18-20`) that nothing executes;
   `workspace_hash.hash_pair` and `digest_hex` are therefore still unprobed
   bindings.
2. **`cargo-probe` negative sides do not build.** Both negative tests call
   `check_package_project` only (`package_rust_interop_build_tests.rs:103,156`),
   yet the row-level label is `cargo-probe` and
   `docs/rust-interop.mdx:56` defines it as "builds generated/package Rust code
   and observes the declared positive or negative result". The label is
   per-fixture, so the plan's frozen table leaves no alternative; the honest fix
   is doc wording — state that a `cargo-probe` negative direction may be
   observed as a rejection before Cargo runs (the fixture READMEs already say
   this in prose). Same sentence in
   `verification/areas/rust_interop/README.md` §Tier And Execution Semantics.
3. **The rejected shared-crate source is authored inside the test.**
   `package_rust_interop_build_tests.rs:141-146` hand-writes the
   `use crate::__sifr_bridge::app::GeneratedPrivate;` backend. The
   boundary-violating artifact therefore exists nowhere in the fixture, is
   invisible to the area checks, and cannot be drift-checked the way the
   positive backend now is. Consider a checked-in
   `examples/shared_hash_bridge/rust/.../negative_lib.rs` (or equivalent) that
   the test `include_str!`s.
4. **Scenario manifest trusts a binding the scenario never declares.**
   `examples/shared_hash_bridge/sifr.toml:20` adds
   `sifr_shared_hash_bridge.generated_private_type` to `rust-no-panic` solely so
   the negative Rust test can reuse the manifest. The canonical scenario now
   grants no-panic trust to a target that is by construction rejected and that
   `src/main.sifr` does not bind.
5. **Only `sifr.toml` is consumed; the Cargo layout is not.** The tests
   synthesize cargo metadata (`package_graph`), so the fixture's
   `Cargo.toml` claims (`path = "rust/workspace_hash"`, `members = [`, asserted
   as tokens in `_scenario_checks.py:41`) remain lint-only. Drift there would
   not break the build evidence.

## Informational (not `hardening_2` scope)

`check_stale_drafts.py --self-test` still prints `rust interop stale draft scan
ok` and ignores the flag, so it gives false self-test signal while listed in
this issue's Required Validation block. `hardening_4` owns the file; pass-1
suggested adding an explicit line to that item, and the issue text has not been
updated.

## Verdict

`hardening_2` is functionally complete and its exit criteria are satisfied. The
two MEDIUM pass-1 defects — the central "evidence must execute what its label
claims" invariant — are genuinely fixed and independently re-executed. The five
remaining findings are wording/hygiene-level; finding 1 is worth fixing before
merge because it is a provenance claim that `hardening_3` will consume.
