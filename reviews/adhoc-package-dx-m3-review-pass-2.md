## Code Review: milestone_adhoc_pkg_3 — Second Pass

**Reviewer:** Claude (second pass)
**Date:** 2026-05-23
**Prior review:** `reviews/adhoc-package-dx-m3-review-pass-1.md` reported `CHANGES_REQUESTED` for two findings.

---

### Re-check of Prior Blocking Finding: App Args Forwarding

**Finding:** `cmd_run_file` was reported to drop `app_args` entirely.

**Current state (`crates/sifr/src/main.rs:868`):**
```rust
return cmd_run_file(&path, app_args, diagnostic_format);
```

**`cmd_run_file` signature and implementation (lines 913-927):**
```rust
fn cmd_run_file(file: &Path, app_args: &[String], diagnostic_format: DiagnosticFormat) -> i32 {
    // ...
    let output = std::process::Command::new(artifact.binary_path())
        .args(app_args)
        .output()
        // ...
```

**Verdict:** The prior blocking finding was based on an older version of the code. The current implementation correctly accepts `app_args` as a parameter and forwards it to the subprocess. The call site at line 868 correctly passes `app_args`. **Not blocking.**

---

### Re-check of Prior Observation: Alignment Matrix Feature Flags

**Finding:** `cargo_cli_alignment_matrix.json` listed `--features` in `aligned_flags` for `run`, but feature passthrough was not wired.

**Current state (`verification/package_management/cargo_cli_alignment_matrix.json`):**
```json
{
  "name": "run",
  "status": "partial_m3",
  "aligned_flags": ["--bin", "--locked", "--offline", "--frozen", "--"],
  "intentional_exclusions": ["Feature selection and Cargo common display/config flags pending full delegated-command passthrough."]
}
```

**Verdict:** `--features` is no longer in `aligned_flags`. Feature selection is explicitly listed as an intentional exclusion. The matrix is accurate for M3 scope. **Resolved.**

---

### Full Diff Review

#### `crates/sifr/src/main.rs`
- **CLI redesign:** `Run`, `Check`, `Tree`, `Fetch` commands now use package-aware resolution instead of positional file paths.
- **`--explain` flag:** New top-level flag for diagnostic code explanations without running operations.
- **Lock flags:** `--locked`, `--offline`, `--frozen` wired consistently across commands.
- **`--bin`/`--script`/`--args`:** Properly parsed and forwarded through `PackageRunRequest` to session planning.
- **`--message-format`:** Properly forwarded to Cargo for package checks via `extend_forwarded_args`.

#### `crates/sifr_package/src/ops/session.rs` (new file)
- `PackageSession::discover`: Correct manifest discovery with manifest-less fallback.
- `plan_run`: Correct priority order — script → bin → explicit path → default target.
- `app_target_plan`: Correctly uses `CargoFeatureSelection::default()` (features deferred to future work, per matrix).
- `validate_explicit_file`: Correctly short-circuits for manifest-less mode; validates paths under source root when manifest exists.
- `plan_script`: Correct recursion guard with `script_depth` tracking. Detection correctly identifies `run` → script chain.
- `discover_app_targets`: Clean layout discovery with nested bin directory support.

#### `crates/sifr_package/src/cargo/commands.rs`
- `CargoCommandPlan::run`: Correctly appends `--` separator before app args.
- `CargoCommandPlan::tree`: Correctly forwards trailing args.
- `CargoCommandPlan::check`: Correctly handles feature selection.

#### `crates/sifr_package/src/manifest/package_sections.rs` (new file)
- `SifrScript` and `SifrDependency` types are correct.
- `parse_scripts`: Correct TOML table parsing with required `command` field.
- `parse_dependencies`: Correctly handles both version strings and dependency tables.

#### `crates/sifr_package/src/manifest/sifr.rs`
- New fields: `default_run`, `scripts`, `dependencies`, `dev_dependencies`.
- Correctly parsed from TOML tables.

#### `crates/sifr_diagnostics/src/codes.rs`
- `SIFR-PACKAGE-0105` retired: removed from active codes, added as reserved with retirement notice.
- New codes: 0605, 0606, 0710, 0714 — all properly registered with test fixtures.

#### `crates/sifr/build.rs`
- Graceful fallback to `"0.0.0"` when version env vars unavailable.

#### `crates/sifr_package/src/diag/package.rs`
- New diagnostic constructors: `run_target_ambiguous`, `invalid_app_target_name`, `explicit_file_outside_source_root`, `script_recursion` — all correctly implemented with actionable help messages.

#### `crates/sifr_package/src/ops/plan.rs`
- New fields on `OperationPlan`: `requires_network`, `writes_projection`, `manifest_less_mode`.
- `violates_lock_mode` correctly extends to network and projection constraints.

#### Test coverage (`milestone_adhoc_pkg_3_tests.rs`)
- `package_session_plans_fetch_tree_and_package_check`: Validates fetch/tree/check plan generation.
- `package_session_resolves_default_script_and_explicit_bin`: Validates default target resolution and script origin.
- `package_session_reports_script_target_ambiguity`: Validates 0605 error for ambiguous selectors.
- `package_session_rejects_invalid_nested_target_name`: Validates 0606 error for bad target names.
- `package_session_rejects_explicit_file_outside_source_root`: Validates 0710 error for out-of-root files.
- `package_session_rejects_nested_script_expansion`: Validates 0714 error for script recursion.

---

### Validation Results

All prior validations confirmed:
- `python3 scripts/check_package_manager_guardrails.py` → PASS
- `cargo test -p sifr_package` → PASS
- `cargo test -p sifr --bin sifr package_cli -- --test-threads=1` → PASS
- `cargo build -p sifr` → PASS
- `cargo clippy -p sifr_package -- -D warnings` → PASS
- `cargo clippy -p sifr -- -D warnings` → PASS
- Docs sync/coverage/gen-error-docs/fmt → PASS
- Manual CLI smoke (`run --script`, `fetch`, `tree`, `check --message-format json`) → PASS

---

### Summary

| Area | Status |
|------|--------|
| App args forwarding in manifest-less explicit file run | READY (was blocking in pass 1, now fixed) |
| Cargo CLI alignment matrix accuracy | READY (features correctly excluded) |
| Package session discovery and resolution | READY |
| Script expansion and recursion rejection | READY |
| CLI flag wiring (locked/offline/frozen, --bin, --script, --explain, --message-format) | READY |
| Cargo failure redactor (0105 retired, maps to 0101) | READY |
| New diagnostics (0605, 0606, 0710, 0714) | READY |
| Manifest parsing for scripts, dependencies, dev-dependencies | READY |
| OperationPlan extension for network/projection constraints | READY |
| Test coverage for session planning | READY |
| Clippy cleanup (build.rs, namespace_api pass-by-ref) | READY |
| Error documentation generation | READY |

---

**VERDICT:** `READY`

This milestone is ready for PR. Both prior blocking findings have been resolved — the current code correctly forwards app args, and the alignment matrix accurately reflects M3 scope.