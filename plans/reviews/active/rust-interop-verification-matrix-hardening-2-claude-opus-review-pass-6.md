## Review: `7b95b92771` (`hardening_2`, rebased onto `0469fe4563`)

### Rebase / conflict resolution — clean
Only `.gitignore` overlapped PR #3019. Pre-rebase tip `7fe0c8f175` vs `7b95b92771` differ in exactly one file: `verification/areas/rust_interop/fixtures/cargo_locked_offline/examples/locked_offline_cache/Cargo.lock`, and it is now byte-identical to `0469fe4563`'s version (the branch's hand-written variant was correctly dropped). `#3019`'s `profile_runner.py` / `selftest.py` changes are intact. The `.gitignore` narrowing (`fixtures/**/Cargo.lock` → `fixtures/*/examples/*/Cargo.lock` + `fixtures/*/negative/Cargo.lock`) still un-ignores all 11 checked-in locks including #3019's, and still ignores `.../workspace_hash_crate/sifr_output/Cargo.lock` (`.gitignore:27`). Tracked set == on-disk set == 11.

### Verified in this pass
- `cargo test -p sifr_driver --lib rust_interop_build_tests -- --ignored --test-threads=1` → **4 passed, 0 failed** (35.79s). Both tier-1 positives build *and run* real Cargo graphs (`cargo metadata --format-version=1 --locked --offline` → `derive_package_graph`) and assert real FNV-1a values; both negatives run against genuine graphs (in-workspace-but-undeclared member; overwritten `rust/.../lib.rs` importing `crate::__sifr_bridge`) and observe `SIFR-RUST-RESOLVE-0001`.
- Area run: `variants=4, failures=0`. `cargo fmt --check` clean. `git status --porcelain --ignored` over the area: nothing beyond `__pycache__/`.
- `--self-test` all green and all exercise real validators: fixture matrix 27 cases (full 5×4 tier/kind sweep two-sidedly via `_validate_execution_semantics`), tiers 6 cases (mutated TOML/JSON rendered to `tempfile.TemporaryDirectory` and re-read through `_load_and_validate`, with an unmutated control), compatibility 3 cases (through `_validate_row`). Unknown args now exit 2 instead of silently running the data path.
- File sizes: largest touched is `check_fixture_matrix.py` at 813 lines; all under 900.
- Docs match behavior: `docs/rust-interop.mdx:49-62`, `internal_docs/rust_interop_architecture.md:1003-1022`, `verification/areas/rust_interop/README.md` §Tier And Execution Semantics all state exactly `ALLOWED_EXECUTION_KINDS` (`check_fixture_matrix.py:125-131`). Both fixture READMEs name the real test functions and correctly attribute them to merge/nightly/release full mode — `create-pr.json:253` sets `crate_tests: "smoke"` and the suite is `modes: ["full"]`, so the READMEs are not overclaiming.

### Actionable findings

**1. MEDIUM — the new mutation self-tests are never executed by any profile, so the enforcement they protect is ungated.**

`verification/areas/rust_interop/manifest.json:22,35,48,61` register all four checks with `"command": "area-check"`, and `run_area_check_case` (`verification/runner/sifr_verify/area_adapter.py:191-196`) invokes `[sys.executable, str(entry)]` with **no arguments**. So `_run_self_test` in `check_fixture_matrix.py:298`, `check_tiers.py:74`, and `check_compatibility_matrix.py:169` only ever runs when a human types `--self-test`.

Failure scenario: delete the `elif tier in ALLOWED_EXECUTION_KINDS ...` branch at `check_fixture_matrix.py:246-252` (or the `diagnostic_crate_rationale` line at `check_compatibility_matrix.py:125`). The checked-in data still validates, so `rust_interop_checks` passes in create-pr/merge/nightly/release and `scripts/run_all_tests.sh` is fully green — a silent removal of the exact semantics `hardening_2` exists to freeze. The issue's "Mutation tests must reject every disallowed pair… and a tier-1 row downgraded to contract-only" (`plans/issues/active/rust-interop-verification-matrix-hardening.md:178-180`) is met in code but not in any gate.

This deviates from established precedent in the same tree: `verification/areas/performance/manifest.json:32-34,59-61,73-75` register `*-self-test` cases dispatched by `verification/areas/performance/runner.py:211-220` as `[sys.executable, entry, "--self-test"]`, and `verification/areas/coverage_matrix/manifest.json:65-66` does the same.

Fix: add self-test cases inside the existing `matrix`, `tiers`, and `compatibility-matrix` suites with a `--self-test`-passing command kind (per the performance-area pattern). Suite *names* are unchanged, so `required_rust_interop_suites()` (`verification/runner/sifr_verify/profiles.py:191-203`) and the four profile `selected_areas` blocks need no edits.

Not raised: `check_stale_drafts.py` still ignores `--self-test` and falls through to the ordinary scan — already explicitly owned by `hardening_4` at `plans/issues/active/rust-interop-verification-matrix-hardening.md:210-211`.

### Verdict
Rebase is regression-free; tier/execution evidence is real, executed, and enforced against the checked-in data; clean-checkout lock behavior is sound; negative probes use genuine Cargo graphs; files are maintainable; docs match behavior. **One MEDIUM finding above** — the self-tests need manifest wiring to be a gate rather than a convention. Everything else in the `hardening_2` exit criteria is met.
