

## Code Review: Phase 37 Follow-up 2 — Organization Demo Repository Subrepos

### Findings

#### 1. Submodule correctness — PASS

- `.gitmodules` (lines 13-32): All 5 demo repos tracked with correct `path`, `url`, and `branch=main`.
- Index entries confirmed as mode `160000` gitlinks (submodules, not trees/blobs): `git ls-files --stage` shows `d9619e5... sifr-demo-json`, `b5deef7... sifr-demo-http`, `3ed21a8... sifr-demo-test-support`, `a3e9f3b... sifr-demo-app`, `ed15a95... sifr-demo-workspace`.
- Each submodule directory has a `gitdir` pointer: `.git` file correctly points to `.git/modules/...`.
- Remote commit hashes match what the context described: json `20a3d8ef`, http `9289f7fa`, test-support `b66077a6`, app `49db0618`, workspace `ed15a955`.

#### 2. Guardrails script — PASS with a gap (acceptable)

`scripts/check_package_manager_guardrails.py` validates:
- `.gitmodules` presence and entries with URL/branch checking (lines 126-147, 268-288).
- `checkout_model = git_submodule` in manifest (line 194).
- Each demo repo is listed in `.gitmodules` with correct URL and `branch=main`.
- Required paths exist per repo.
- Pure markers in json, test-support, and workspace members (line 322-328).
- App Cargo.toml has both json versions, http, test-support, aliases, and tag pins (lines 347-357).
- App Cargo.lock pins `v0.2.0` alias (line 358).
- App `migrate.sifr` imports both aliases (lines 360-361).
- Http Cargo.toml depends on tagged json and reqwest with both in sifr.toml (lines 335-338).
- Workspace structure has correct `default-members`, `exclude`, and workspace dependencies (lines 366-377).

**Gap**: `check_demo_repository_shape` does not call `check_pure_marker` on `sifr-demo-http`'s `src/lib.rs`. However, `check_rust_backed_http_template` at lines 339-340 requires `reqwest::` in `src/lib.rs`, which implicitly requires Rust code. The http repo's `src/lib.rs` contains `pub fn trusted_backend_name()` that exercises reqwest (satisfying the requirement), but this specific shape is validated by `cargo check` rather than the guardrails script.

#### 3. Cargo standalone workspace issue — FIXED

- All four standalone repos have `[workspace]` (confirmed by grep count = 1 each).
- `cargo metadata --locked` in sifr-demo-json: resolves only json, workspace_members is a single-element list — correct.
- `cargo metadata --locked` in sifr-demo-app: resolves 109 packages, `sifr-demo-app` is the root package, no parent workspace contamination.
- `cargo metadata --locked` in sifr-demo-http: shows only http + json v0.1.0 (no v0.2.0) — correct.

#### 4. Git tag/lock pin internal consistency — CONSISTENT

Verified chain:
- `sifr-demo-http` → `sifr-demo-json` `v0.1.0` at `20a3d8ef` (Cargo.lock: `git+...sifr-demo-json?tag=v0.1.0#20a3d8ef...`)
- `sifr-demo-app` → `sifr-demo-http` `v0.1.0` at `9289f7fa` (Cargo.lock: `git+...sifr-demo-http?tag=v0.1.0#9289f7fa...`)
- `sifr-demo-app` → `sifr-demo-json` `v0.1.0` at `20a3d8ef` (Cargo.lock: `git+...sifr-demo-json?tag=v0.1.0#20a3d8ef...`)
- `sifr-demo-app` → `sifr-demo-json` `v0.2.0` at `b6733d1f` (Cargo.lock: `git+...sifr-demo-json?tag=v0.2.0#b6733d1f...`)
- `sifr-demo-app` → `sifr-demo-test-support` (Cargo.lock confirms presence)
- `sifr-demo-app`'s `migrate.sifr` imports both `demo_json_v1` and `demo_json_v2`

Tags exist and match in all remote repos:
- json: `v0.1.0` → `20a3d8ef`, `v0.2.0` → `b6733d1f`
- http: `v0.1.0` → `9289f7fa`
- app: `v0.1.0` → `49db0618`
- test-support: `v0.1.0` → `b66077a6`

#### 5. Docs/test naming — ACCURATE

- `internal_docs/phases/37_package_management.md` lines 14-15: follow-up item 2 marked unchecked, correctly representing the pending PR state.
- `crates/sifr_package/TRACEABILITY.md` line 49: correctly references subrepos as `ported` coverage.
- `issues/phase37-package-management-execution.md` line 631-635: closeout note correctly says "checked out through git submodules".
- `verification/package_management/demo_repositories/README.md` line 3: "git submodule checkouts".
- Test name `phase37_demo_subrepos_cover_required_org_repos` is accurate and passes.

---

### Verdict

All 5 review dimensions pass. The guardrails script gap (no explicit Rust code check for http's lib.rs) is covered by `cargo check` in the validation suite and is implicit in the reqwest requirement. No blockers found.

**READY**
