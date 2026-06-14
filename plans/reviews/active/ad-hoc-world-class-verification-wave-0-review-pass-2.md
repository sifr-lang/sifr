Now I have what I need. Let me record my findings.

## Wave 0 review — pass 2

### Pass-1 blockers — verification

**1. Doctor command — RESOLVED.** `verification/runner/sifr_verify/doctor.py` ships python/cargo/rustc/uv tool checks, `uv lock --check`, `cargo metadata --locked` with `CARGO_NET_OFFLINE=true`, optional sanitizer tools (skip), and host metadata. Routed at `__main__.py:38-39`; documented at `verification/README.md:13,35-38` and `verification/policy/profile_policy.md:26-27`.

**2. core_language:e2e-pass declared but missing — RESOLVED.** No profile selects a `core_language:e2e-pass` suite. `merge`/`nightly`/`release` select only the three real suites (`integer_dtype_contract`, `phase24_hir_analysis`, `phase25_cfg_flow`); `create-pr` omits `core_language` entirely. The e2e pass corpus continues to run through `legacy_facade.e2e.fixture_manifest`.

**3. Selected-area/suite validation — RESOLVED.** Enforced in two places:
- `profiles.py:98-120` — fail-fast at load time (load_profile → validate_selected_area_suites). Unknown area or unknown `area:suite` raises `ProfileError`.
- `coverage_matrix/checks/coverage_matrix.py:268-357` — advisory-mode validator with the same cross-check.

**4. Cargo hermetic contract — MOSTLY RESOLVED, ONE GAP.**
- `profile_runner.cargo_command()` (lines 86-91) consistently injects `--locked` (including before `--`).
- `ProfileRunner.__init__` (lines 127-130) sets `os.environ["CARGO_NET_OFFLINE"] = "true"` when `cargo_policy.offline=true`, so unparameterized `subprocess.run` calls inherit it.
- `area_adapter.py:233,450`, `audit_fixtures.py:143`, `hardening/core.py:155` all add `--locked`.

  **Remaining gap (blocking):** `verification/runner/e2e/run_e2e_pass.sh:198` invokes
  ```
  cargo test -p sifr --test e2e test_e2e_pass -- --nocapture
  ```
  without `--locked`. This is reached from `profile_runner.run_e2e_pass_suite` and is therefore on the profile-execution path. CARGO_NET_OFFLINE is inherited via the `os.environ` mutation, but `--locked` is not enforced — cargo will silently regenerate `Cargo.lock` if it drifts. `profile_policy.md:20-21` states "Cargo profile execution is locked and offline"; the e2e pass invocation violates the "locked" half.

**5. Plan emission/compare omitted sandbox/host — RESOLVED.** `profiles.build_profile_plan` (lines 218-225) emits `network_policy`, `cargo_policy`, `reference_host`, and `execution_sandbox`; `compare_plans` (lines 258-267) compares all four.

### Additional pass-1 items

- **Owner-registry schema** (#9) — RESOLVED. `verification/schemas/owners.schema.json` added; `load_owner_ids` (`coverage_matrix.py:89-99`) validates against it.
- **Duplicate owner aliases** (#8) — RESOLVED. `owners.json` keeps only one canonical id per team; surface-matrix and area manifests now all resolve.
- **Mis-mapped infra surfaces** (#6) — RESOLVED. `first_party_crate_tests` and `cargo_features_targets` now bind to `project-workspace-package-contract` (lines 435, 449).
- **Wave→subwave map** (#10) — RESOLVED. `ALLOWED_SUBWAVES_BY_WAVE` is a wave-keyed map; subwave validation falls back to `set()` for waves with no defined subwaves.

### Remaining blocker for Wave 0 exit

- `verification/runner/e2e/run_e2e_pass.sh:198` must add `--locked` to the `cargo test` invocation. Inheriting `CARGO_NET_OFFLINE` from `os.environ` is half the contract; the other half (`--locked`) is encoded but not enforced for the e2e pass suite, which is the slowest and most reproducibility-sensitive step.

### Non-blocking recommendations

- **Pass-1 #7 not addressed:** `distribution_release_full` (line 422-431) still shares `merge_suite: distribution_release:representative` with `distribution_release_representative` (line 411-421). Two rows pointing at the same merge evidence remains misleading; pick a row to differentiate or collapse them.
- `scripts/check_codegen_rawcode_gate.sh:15` (`cargo test -p sifr_codegen preamble_rawcode_is_zero`) also lacks `--locked`. Not currently invoked from any profile path, but if it gets wired in later it inherits the same flaw.
- `area_adapter.py:244-251` `subprocess.run` to `cargo test` builds its env from `_contract_matrix_env()` (which copies `os.environ`). When the area runner is invoked directly (not via `ProfileRunner`), `CARGO_NET_OFFLINE` is not set. Consider centralizing the cargo-env contract in one helper so direct `areas run` invocations honor it.
- `__main__.py:14-22` exposes both legacy top-level flags (`--self-test`, `--list-areas`, `--profile`) and subcommand routing (`profiles`, `areas`, `doctor`, `reports`). Worth pruning before Wave 0 closes so the public surface is just the subcommand form.
- `ALLOWED_WAVES` still permits 1–9 in flat form even though only 2, 5, 6, 9 have defined subwaves; a row with `closes_in_wave: "3"` and no subwave silently passes. If the plan defines coverage waves precisely, encode the same in the validator.

### Bottom line

Four of the five pass-1 blockers are fully resolved; the cargo hermetic contract is closed everywhere except `run_e2e_pass.sh:198`. Adding `--locked` there is the last actionable Wave 0 exit item.
