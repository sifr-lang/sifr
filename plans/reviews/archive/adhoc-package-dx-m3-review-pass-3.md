

## Review Complete: milestone_adhoc_pkg_3 — Third Pass

**VERDICT: `READY`**

All three review passes complete:

| Pass | Finding | Status |
|------|---------|--------|
| Pass 1 | `cmd_run_file` dropped `app_args` | FIXED — `cmd_run_file` now forwards args |
| Pass 2 | Alignment matrix had `--features` in `aligned_flags` | FIXED — moved to `intentional_exclusions` |
| Pass 3 | Root `sifr.toml` missing `sifr-version` | FIXED — field added |
| Pass 3 | Multi-root explicit-file validation incorrect | FIXED — `validate_explicit_file` uses `.any()` over all roots |
| Pass 3 | App target discovery needed separation (guardrail) | FIXED — `ops/session_targets.rs` extracted |

**Key changes after pass 2:**

1. **`sifr.toml`** — `sifr-version = ">=0.3,<0.4"` added to satisfy manifest requirement
2. **`ops/session.rs`** — `validate_explicit_file` now checks all `source_roots` (`.any()`), not just the first
3. **`ops/session_targets.rs`** — New module extracted from `session.rs` to satisfy no-monolithic-files guardrail
4. **New test** — `package_session_accepts_explicit_file_under_any_legacy_source_root` uses the root manifest's multi-root config to verify the fix

**Validation:** All checks pass (60 tests, clippy, guardrails, fmt, docs, quick profile).

Review saved to: `reviews/adhoc-package-dx-m3-review-pass-3.md`
 The test `package_session_accepts_explicit_file_under_any_legacy_source_root` uses this exact manifest to verify that explicit files under *any* configured source root pass validation.

---

#### `crates/sifr_package/src/ops/session.rs` — `validate_explicit_file` and multi-root tracking

`PackageSession` now stores `source_roots: Vec<PathBuf>` (line 16) and `validate_explicit_file` (lines 251–271) uses `.any()` over all roots:

```rust
if self.source_roots.iter().any(|source_root| path_is_under(file, source_root)) {
    Ok(())
} else {
    let source_root = self.source_roots.first().unwrap_or(&self.workspace_root);
    Err(PackageDiagnostic::explicit_file_outside_source_root(file, source_root))
}
```

**Verdict:** Correct. A file under `audits/leetcode/src` or `.` (the current directory) is accepted. Only files outside *all* configured roots are rejected.

---

#### `crates/sifr_package/src/ops/session_targets.rs` (new file)

App target discovery was extracted from `session.rs` to satisfy package guardrails (no monolithic files). The module provides `discover_app_targets(source_roots, package_name)` and handles:

- `src/main.sifr` → named target
- `src/bin/*.sifr` → flattened names, `/` as path separator
- `src/bin/tools/migrate.sifr` → target named `tools/migrate`
- Invalid name rejection via `SIFR-PACKAGE-0606`

**Verdict:** Correct. No logic change — just a file extraction that satisfies the guardrail.

---

#### `crates/sifr_package/src/milestone_adhoc_pkg_3_tests.rs` — new test

```rust
#[test]
fn package_session_accepts_explicit_file_under_any_legacy_source_root() {
    // Uses roots = ["audits/leetcode/src", "."]
    // Writes demos/app.sifr (under the "." source root)
    // plan_check(Some(&path.join("demos/app.sifr")), ...) must not error
}
```

**Verdict:** Correct. The test explicitly exercises the multi-root validation path. The manifest uses legacy `[source].roots` (not production `[source].root`), which is appropriate since this is a legacy fixture, not a production package.

---

### Verification Checklist

| Check | Result |
|-------|--------|
| `cargo test -p sifr_package` | PASS, 60 tests |
| `cargo test -p sifr_package package_session -- --test-threads=1` | PASS, 7 tests |
| `cargo test -p sifr_package cargo_failure_redaction` | PASS |
| `cargo test -p sifr_package manifest_parses_scripts` | PASS |
| `cargo test -p sifr --bin sifr package_cli -- --test-threads=1` | PASS, 5 tests |
| `cargo build -p sifr` | PASS |
| `cargo clippy -p sifr_package -- -D warnings` | PASS |
| `cargo clippy -p sifr -- -D warnings` | PASS |
| `cargo run -q -p sifr -- check demos/mode_consistency/main.sifr` | PASS |
| `python3 scripts/check_package_manager_guardrails.py` | PASS |
| `python3 scripts/check_diagnostic_docs_sync.py` | PASS |
| `python3 scripts/check_diagnostic_code_coverage.py` | PASS |
| `cargo run -q -p sifr_diagnostics --bin gen-error-docs -- --check` | PASS |
| `cargo fmt --check` | PASS |
| `scripts/run_all_tests.sh --profile quick` | PASS |
| `docs/errors/SIFR-PACKAGE-0105.md` | Deleted (retired) |
| New doc pages | All present: 0605, 0606, 0710, 0714 |
| Root `sifr.toml` `sifr-version` | Present |

---

### Final Review of Phase Contract

| Contract Item | Implementation | Status |
|---|---|---|
| `sifr.toml` requires `package.sifr-version` | `manifest/sifr.rs:92-99` validates and requires the field | READY |
| Explicit files validated against all source roots | `validate_explicit_file` uses `.any()` over `source_roots` | READY |
| Multi-root manifests work with explicit file validation | Test `package_session_accepts_explicit_file_under_any_legacy_source_root` | READY |
| App target discovery is separate module | `ops/session_targets.rs` with `discover_app_targets` | READY |
| Script expansion, ambiguity detection, recursion guard | All present and tested | READY |
| Cargo failure redactor (0105 retired → 0101) | Test `cargo_failure_redaction_preserves_public_context_and_retires_0105` | READY |
| CLI wired: locked/offline/frozen, --bin, --script, --explain, --message-format | All present and tested | READY |
| Guardrails pass | `check_package_manager_guardrails.py` → PASS | READY |
| Alignment matrix reflects M3 scope | `run --features` in `intentional_exclusions`, `--bin/--locked/--offline/--frozen/--` in `aligned_flags` | READY |

---

### Summary

| Area | Status |
|---|---|
| Root `sifr.toml` `sifr-version` field | READY |
| `PackageSession` tracks all `source_roots` | READY |
| `validate_explicit_file` accepts files under any configured root | READY |
| `ops/session_targets.rs` module separation (guardrail) | READY |
| New multi-root explicit file test | READY |
| All prior blocking findings (pass 1: app args; pass 2: alignment matrix) | RESOLVED |
| Full validation suite | PASS |

---

**VERDICT:** `READY`

This milestone is complete. All pass-1 and pass-2 blocking findings have been resolved, the multi-root validation fix has been implemented and tested, and all guardrails pass.
