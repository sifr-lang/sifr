

Based on my thorough review of the working tree, here is my assessment:

---

## Review: milestone_adhoc_pkg_8 — Cargo-compatible workspace run ergonomics

### Files Reviewed- `crates/sifr/src/workspace_run_selection.rs` (122 lines) — new
- `crates/sifr_package/src/ops/workspace_session.rs` (41 lines) — new
- `crates/sifr/src/cli_model_and_entrypoint.rs` — additions for Run command wiring
- `crates/sifr/src/diagnostic_rendering_and_run.rs` — `cmd_run` integration- `crates/sifr/src/check_and_package_commands.rs` — `package_session_for_cwd` to shared module
- `crates/sifr/src/mode_resolution_tests.rs` — test coverage at line 376-449
- `crates/sifr_package/src/ops/session.rs` — `PackageSession::discover`, `plan_run`, script handling
- `crates/sifr_package/src/diag/package.rs` — `workspace_run_ambiguous` diagnostic
- `crates/sifr_package/src/cargo/metadata.rs` — `workspace_default_members` field- `docs/package_management.md` — updated with demo/examples

---

### 1. Blocking Findings
**None.** No correctness issues found. The implementation correctly:

- Preserves workspace root throughout the selection chain (uses `self.workspace_root` from discovered session)
- Resolves source roots from the **selected** package's `sifr.toml` (via `from_package_metadata`)
- Filters to Sifr-source packages only (skips `BackendRust`) when resolving default members
- Short-circuits run delegation to Cargo when `--bin` or script creates a fully-qualified plan
- Has no risk of crossing Sifr packages in the same workspace run unless deliberately selected by name---

### 2. Non-blocking Concerns

**A. Test gap — ambiguity diagnostic coverage**`mode_resolution_tests.rs:376-449` tests4 scenarios (explicit package, `--bin`, `--script`, default-with-exact-members). There is no dedicated test for the **zero-candidates** branch of `default_workspace_run_session` (lines 70-74) or the **multi-candidate ambiguity** path where the diagnostic on line 109 fires. This could be validated silently if behavior regresses.

**B. Guardrail: missing `workspace = true`**  
The demo `packages/app/Cargo.toml` omits `workspace = true`, which is conventional for workspace members. The guardrail in `check_workspace_template` (line 423-424) does not enforce this. Not a functional issue; purely cosmetic.

**C. No dedicated milestone test file**  
There is no `milestone_adhoc_pkg_8_tests.rs`. This is acceptable since tests exist in `mode_resolution_tests.rs` and `sifr_package` integration tests, but the pattern of other milestones suggests one may be expected.

---

### 3. Validation Gaps

**Minimal validation gaps** — The user reports the following passed:
- Unit tests: `cargo test -p sifr_package package_session` and `sifr --bin sifr package_cli`
- Guardrail: `python3 scripts/check_package_manager_guardrails.py` ✓- E2E from demo workspace: three command forms (`-p`, `--bin`, `--script`)

**Unverified but low-risk:**
- The `[package].default-run` path (`has_default_runnable_app()` → `manifest.default_run`) — no explicit integration test, but the code path is well-structured.
- Error path when no candidate matches (diagnostic `workspace_run_ambiguous` with `candidates.is_empty()`) — branch exists in `package.rs:277-294` but not exercised.

---

### 4. File-Size Compliance

| File | Lines | Limit | Status |
|------|-------|-------|--------|
| `sifr/src/workspace_run_selection.rs` | 122 | N/A (new) | OK |
| `sifr_package/src/ops/workspace_session.rs` | 41 | N/A (new) | OK |
| `sifr_package/src/ops/session.rs` | 401 | 420 | OK |
| `sifr/src/cli_model_and_entrypoint.rs` | 880 | 900 (hand-maintained cap) | OK |
| Guardrail script | — | — | PASS |

---

### 5. Final Verdict

**READY**

The implementation is correct by inspection:

1. **`sifr run -p <package>`** creates a session via `from_package_metadata` that correctly isolates source roots, manifest path, and scripts to the selected package while retaining the workspace root for Cargo delegation.

2. **Default-members behavior** honors `workspace.default-members` and falls back to all members; emits `SIFR-PACKAGE-0605` with a clear diagnostic when multiple candidates exist or none are runnable Sifr apps.

3. **Manifest-less mode preserved**: `run_target_is_explicit_path()` filters targets that look like `.sifr` files or contain path separators, passing them through without workspace graph lookup.

4. **Scripts** pass the selected `PackageSession` (with its correct package context) alongside the plan.

5. **Guardrails**: package-manager maintainability checks pass, no module exceeds limits, no banned Cargo terms outside the adapter layer.

The non-blocking concerns are minor test gaps and cosmetic guardrail items; they do not affect correctness or safety. The implementation is ready to ship pending the user's confirmation that the three e2e validations produce the expected output (`200`).
