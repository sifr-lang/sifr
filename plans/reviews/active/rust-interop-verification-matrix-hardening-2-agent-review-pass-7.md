## Review pass 7: `hardening_2` at `891fe54dc` (diff `0469fe4563..891fe54dc`)

Scope: re-review after the pass-6 MEDIUM finding (mutation self-tests ungated) was
addressed by `891fe54dc` "Gate Rust interop mutation self-tests".

### The pass-6 finding is fixed, and the fix is a real gate

`verification/runner/sifr_verify/area_adapter.py:44-45,181-184,190-242` adds
`area-check-self-test`, which reuses `run_area_check_case` and appends `--self-test`
to the argv. `verification/areas/rust_interop/manifest.json:26-32,46-52,66-72` adds one
self-test case to each of the `matrix`, `tiers`, and `compatibility-matrix` suites.

Verified by execution (`python -m sifr_verify areas run --area rust_interop`):
`variants=7, failures=0`, with all three self-tests running and reporting
`cases=27`, `cases=6`, `cases=3` respectively.

Verified the gate is not decorative — each check was mutated in place, the ordinary
data path still passed, and the self-test caught it:

| Mutation | plain run | `--self-test` |
| --- | --- | --- |
| drop tier/kind branch (`check_fixture_matrix.py:248-253`) | exit 0 | exit 1, `tier 0/cargo-probe allowed=False failures=[]` |
| drop `_expect_equal(... "diagnostic_crate_rationale")` (`check_compatibility_matrix.py:125`) | exit 0 | exit 1, `missing rationale passed` |
| drop tier-mismatch branch (`check_tiers.py:58-61`) | exit 0 | exit 1, `matrix mismatch did not report ...` |

Tree restored to `891fe54dc` afterwards (`git status --porcelain` clean).

### Result metadata / argv are accurate

`target/verification/areas/rust-interop-results.json` records each self-test case with
`command`/`label` = `area-check-self-test` and
`argv = [<python>, <entry>, "--self-test"]`; the four ordinary cases keep
`command`/`label` = `area-check` and the two-element argv. Timing lines are
`case=<suite>/<case-id>/<command>`, so existing `area-check` timing tokens are
unchanged — no timing-baseline drift, and no timing registry references
`rust_interop` case tokens.

### Executed in every profile that selects the suites

`required_rust_interop_suites()` (`profiles.py:189-203`) derives the required set from
the manifest suite names, and `create-pr`, `merge`, `nightly`, and `release` each select
all four suites (`selected_areas[].suites`); `python-interop-live` is
`selected-areas-only` and does not select `rust_interop`. Because the self-tests were
added as *cases inside existing suites*, no profile edit was needed and all three run in
all four authoritative profiles. `python -m sifr_verify profiles check` and
`--self-test` (8 self-tests, including "Rust interop profile execution self-test") pass.

### Maintainability / blast radius

- `area-check-self-test` mirrors the established `python-script-self-test` /
  `budget-self-test` / `benchmark-self-test` naming in
  `verification/areas/performance/manifest.json`, so the convention is consistent.
- `verification/schemas/area.schema.json` keeps `command` a free-form string, so no
  schema change was needed; `discover_areas` validation passes.
- `validate_unique_baseline_artifact_paths` (`area_adapter.py:447-453`) correctly
  excludes the new command alongside `area-check`/`validation-suite`; the self-test
  cases produce no baseline artifacts, so the shared `entry` with the sibling
  `area-check` case cannot collide.
- No other area manifest uses the new command, and the only cross-area consumer that
  filters on `"area-check"` (`verification/areas/diagnostics/checks/code_baseline_coverage.py:71`)
  reads only the diagnostics manifest. No blast radius.

### Rest of the milestone diff — no regressions found

- `cargo test -p sifr_driver --lib rust_interop_build_tests -- --ignored --test-threads=1`
  → **4 passed, 0 failed** (36.0s) at this tip.
- Those ignored tests are gated: `sifr_driver_generated_builds`
  (`cargo test -p sifr_driver --lib -- --ignored --test-threads=1`) is
  `modes: ["full"], status: "blocking", executed_in_merge: true` in `merge.json` /
  `nightly.json` / `release.json`, so the tier-1 `cargo-probe` upgrades of
  `same_workspace_crate` and `shared_bridge_crate` are backed by an executed merge gate.
- Tier *assignments* are byte-identical to `0469fe4563` (only descriptions changed) —
  no fixture was relocated to a laxer tier to satisfy `ALLOWED_EXECUTION_KINDS`.
- `cargo fmt --check` clean; `cargo clippy --workspace` (the CI invocation) clean;
  `scripts/check_file_size_guardrails.py` PASS (limit 900);
  `scripts/check_sifr_driver_maintainability_guardrails.py` PASS.
- `.gitignore` un-ignore narrowing still yields tracked-lock count == on-disk count == 11.
- The negative-overlay files that `_scenario_checks._read_toml` skips silently when
  absent are all `include_str!` inputs in
  `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:3-26`, so deleting
  any of them fails compilation of `sifr_driver` in every profile — the silent-skip path
  is fully covered by a stronger gate.
- Docs still match behavior: `docs/rust-interop.mdx:49-62`,
  `internal_docs/rust_interop_architecture.md:1006-1022`, and the area README
  §Tier And Execution Semantics all state exactly `ALLOWED_EXECUTION_KINDS`.

### Non-blocking observations (not findings)

1. Nothing pins the *presence* of the three `-self-test` manifest cases:
   `required_rust_interop_suites()` and `validate_rust_interop_result`
   (`profile_runner.py:115-160`) both check suite names/aggregates only, so deleting a
   case from `manifest.json` would silently drop that gate. This is the same class of
   risk as deleting any test, and matches the precedent in the performance area; noted
   only for awareness.
2. `verification/areas/rust_interop/README.md:78-88` §Suites describes the four suites
   but does not mention that three of them also execute a `--self-test` mutation case.
   Nothing in the README is inaccurate; adding a sentence would make the enforcement
   discoverable.

### Verdict

**Actionable findings: None. APPROVED.**
