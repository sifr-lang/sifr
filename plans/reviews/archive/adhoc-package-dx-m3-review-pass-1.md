

## Code Review: milestone_adhoc_pkg_3

I've reviewed the diff across all changed files. Overall the implementation is solid. However, I found one **blocking issue** and two **non-blocking observations**.

---

### BLOCKING: `sifr run` does not handle manifest-less explicit files correctly

**Location:** `crates/sifr/src/main.rs:863-869`

When `cmd_run` receives an explicit `.sifr` file (via positional `target` that looks like a sifr path or contains path separators), `session.plan_run()` validates the file against the package source root. If there is no `sifr.toml`, the session's `manifest_less_mode` is `true` and `validate_explicit_file()` short-circuits to `Ok(())` — so the file passes validation. But then `plan_run()` returns a `PackageCommandPlan` with `cargo: None` and `run_target: Some(ResolvedRunTarget::File(_))`. This is correct.

**The actual problem** is in `cmd_run`:

```rust
if let Some(
    sifr_package::ResolvedRunTarget::File(path)
    | sifr_package::ResolvedRunTarget::App { path, .. },
) = plan.run_target
{
    return cmd_run_file(&path, app_args, diagnostic_format);
}
```

When there's a manifest-less session with an explicit file, `plan.run_target` is `Some(File(...))`, so we call `cmd_run_file`. But `cmd_run_file` ignores `app_args` entirely in its signature — it only uses `file` and `diagnostic_format`. The `app_args` (everything after `--`) are never forwarded.

**Concrete scenario:**
```bash
sifr run src/tool/task.sifr -- --verbose
# app_args = ["--verbose"]
# plan.run_target = Some(ResolvedRunTarget::File("src/tool/task.sifr"))
# cmd_run_file(path, app_args, ...) is called
# but cmd_run_file signature: fn cmd_run_file(file: &Path, diagnostic_format: DiagnosticFormat)
# and inside: artifact.binary_path() is executed with NO args forwarded to the process
```

**Fix:** `cmd_run_file` must accept and forward `app_args`. The call at line 868 needs to pass `app_args` to `cmd_run_file`, and `cmd_run_file` (line 924) must pass them to `std::process::Command::new(artifact.binary_path()).args(app_args).output()`.

---

### Observation 1: `cargo run` passes `--bin` but ignores `--features` in app target plan

**Location:** `crates/sifr_package/src/ops/session.rs:204-219` (`app_target_plan`)

The plan unconditionally uses `CargoFeatureSelection::default()` and only passes `bin` and `app_args`:

```rust
cargo: Some(CargoCommandPlan::run(
    self.workspace_root.clone(),
    self.lock_mode,
    &CargoFeatureSelection::default(),  // no features
    Some(&target.name),
    app_args,
))
```

The CLI `sifr run --bin admin --features my-feature` would parse `--features` into `Args` but there is no `features` field in `Commands::Run`. The alignment matrix marks `run --features` as "aligned" but it's not wired.

**Not blocking** for M3 since the alignment matrix says `check` and `run --features` are `partial_m3` — features passthrough is deferred to future work. However, the matrix should not list aligned_flags that aren't wired.

---

### Observation 2: `check_package_manager_guardrails.py` requires `cargo_cli_alignment_matrix.json` but the matrix marks `run --features` aligned

**Location:** `scripts/check_package_manager_guardrails.py:18-30` + `verification/package_management/cargo_cli_alignment_matrix.json`

The guardrail checks that the alignment matrix exists. The matrix is well-structured and the `policy.sifr_only_flags` listing (`run --script`, `run explicit .sifr path`, etc.) is accurate. But the `run` command's `aligned_flags` includes `--features` which is not wired in the CLI. This creates a drift between the matrix and the implemented surface.

The matrix's `partial_m3` status with `intentional_exclusions` acknowledges that features passthrough is deferred — but listing `--features` in `aligned_flags` implies it works. The intent is clear from the comment; however, future reviewers may be confused.

**Not blocking** but the matrix comment could be more explicit that `--features` is **not** wired in M3 despite being in `aligned_flags`.

---

### Summary

| Area | Status |
|------|--------|
| Package session discovery, fetch, tree, check plans | READY |
| Script expansion and recursion rejection | READY |
| CLI flag wiring (locked/offline/frozen, --bin, --script, --explain, --message-format) | READY |
| Cargo failure redactor (0105 retired, maps to 0101) | READY |
| New diagnostics (0605, 0606, 0710, 0714) | READY |
| Manifest parsing for [scripts], [dependencies], [dev-dependencies] | READY |
| Clippy cleanup (build.rs, namespace_api pass-by-ref) | READY |
| Test coverage for session planning | READY |
| **App args forwarding in manifest-less explicit file run** | **BLOCKING** — `cmd_run_file` drops `app_args` |

---

**VERDICT:** `CHANGES_REQUESTED`

Fix the `cmd_run_file` signature to accept and forward `app_args`, and update the call site at `crates/sifr/src/main.rs:868` to pass them. This is a one-file change (main.rs). All other areas are ready.
