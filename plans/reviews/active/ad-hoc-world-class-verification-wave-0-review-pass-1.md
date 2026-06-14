# Wave 0 Review

## Blocking findings

**1. `sifr_verify doctor` / `scripts/verification_doctor.sh` is missing.**
Wave 0 explicitly tasks this ("Add `scripts/verification_doctor.sh` or `uv run --project verification --locked python -m sifr_verify doctor` to check local prerequisites: Rust toolchain, Python version, uv lock state, Cargo offline setup, required sanitizer tools where applicable, and supported host metadata."). A grep across `verification/runner/sifr_verify/` and `scripts/` finds no implementation. The local-first contract loses its setup-failure boundary without it.

**2. All four profiles declare `selected_areas: core_language → suites: ["e2e-pass"]`, but `core_language/manifest.json` has no `e2e-pass` suite.**
The manifest only ships `integer_dtype_contract`, `phase24_hir_analysis`, `phase25_cfg_flow`, `audit-fixtures`. The e2e pass corpus is actually executed by `profile_runner.run_e2e_pass_suite` via the bash script using `legacy_facade.e2e.fixture_manifest`, *not* through the area runner. Result: the profile-data path declares coverage that never goes through the suite it names. This directly contradicts the Wave 0 exit criterion "A reviewer can answer 'what blocks merges for this compiler guarantee?' without reading Python runner code." Either add a real `e2e-pass` suite to the `core_language` manifest, or remove the lie from the profile data.

**3. `selected_areas` is not validated against existing areas/suites.**
`profiles.py:load_profile` and `coverage_matrix.py:validate_profile_policy` never cross-check `selected_areas[*].area`/`selected_areas[*].suites` against the area manifests. Typos and references to nonexistent suites pass validation silently — which is how #2 went undetected. Wave 0 lists "advisory-mode check that fails schema errors and unknown statuses." Unknown suite ids fall in the same family and should fail here.

## Schema / enforcement gaps

**4. Cargo hermetic contract is declared but not enforced.**
Profiles set `cargo_policy.locked: true, offline: true`, but `profile_runner.run_crate_tests` issues `cargo test -p sifr_*` without `--locked` and without `CARGO_NET_OFFLINE=true` in the env. Wave 0's profile_policy.md explicitly states "Cargo profile execution is locked and offline." The runner does pass `--locked` to `uv` invocations but not to cargo. Encoded ≠ enforced. The plan's "Required workflow" treats local validation as authoritative, so this is the bit that actually matters.

**5. `compare_plans` and `build_profile_plan` omit `execution_sandbox` and `reference_host`.**
`profiles.build_profile_plan` returns only `network_policy, cargo_policy, reference_host, budgets, selected_areas, legacy_facade, e2e` — and `compare_plans` only compares `["profile", "selected_areas", "legacy_facade", "e2e", "network_policy", "cargo_policy"]`. `execution_sandbox` is missing from both the emitted plan and the comparator. CI can drift on sandbox settings (tempdir_only, external_network, stdout_stderr_byte_limit, subprocess_cleanup) without the parity check noticing. Wave 0's "Add a local/CI plan-equivalence skeleton" is the place to surface that.

## Data quality (non-blocking)

**6. `first_party_crate_tests` and `cargo_features_targets` rows are bound to `guarantee_id: hir-name-type-flow-contract`** (compiler_surface_matrix.json:434, 448). These are workspace/cargo infrastructure surfaces, not HIR/name/type/flow concerns. The validator only checks the guarantee_id resolves, but the mapping is semantically wrong. Consider a dedicated infrastructure guarantee (e.g., bind to the project-workspace-package-contract) or document the intent.

**7. `distribution_release_full` row has `merge_suite: distribution_release:representative`** — identical to the `distribution_release_representative` row's merge_suite. Two rows pointing at the same merge evidence, with only the row name implying a difference. Either reuse one row or differentiate the merge_suite.

**8. `owners.json` carries duplicate identities for the same team** (`algorithmic/compatibility` vs `algorithmic-compatibility`; `runtime/platform` vs `runtime-platform`; `compiler/codegen` vs `codegen`; etc.). The plan's "team-style owners" list (`compiler-verification`, `codegen`, `runtime-platform`, ...) is the canonical form, yet the slash variants are what's used by the surface matrix and area manifests. Pick one form and remove the orphan duplicates, or the registry will accumulate aliasing drift.

**9. `owners.json` has no schema in `verification/schemas/`.** Its shape is enforced only by ad-hoc parsing inside `coverage_matrix.py:load_owner_ids`. The plan calls it "the authoritative owner registry." A schema file alongside `area.schema.json` / `profile.schema.json` would prevent silent shape drift.

**10. `ALLOWED_WAVES = 1..9` and `ALLOWED_SUBWAVES` include `"0".."9"+"final"`, but the plan defines no subwave `"0"` for Wave 5/6** — only the named ones (5.1..5.8, 6.0..6.1). `cpython_hand_seeded_differential` uses `closes_in_subwave: "0"` which the plan refers to as `6.0` and so is intentional, but Wave 5 has no 5.0. Tighter: encode the wave→subwave map (only the (wave, subwave) pairs the plan defines) rather than a flat allowed list.

## Pass-through observations

- `validate_owner_registry_covers_area_manifests` correctly covers all 15 area manifests, and every existing area owner is in `owners.json`.
- `SIFR_COVERAGE_MATRIX_STRICT=1` correctly fails on temporary rows. ✓
- Stable-guarantee → matrix-row coverage is enforced (`stable guarantee has no matrix row`). ✓
- `red-blocker` requires `current_failure_count`, `command`, `triage_file`, `closes_in_wave`, `issue`, `expiry`. ✓
- `not-applicable` is rejected for stable guarantees. ✓
- `broad-only` is rejected for stable guarantees. ✓
- Expired temporary rows fail validation. ✓
- 22 temporary rows match the runner's reported count.

The shape of Wave 0 is correct; the misses are concentrated in three places: a missing doctor command, profile data declaring a suite that doesn't exist (with no validator to catch it), and metadata declared but not actually enforced (cargo offline, sandbox parity).
