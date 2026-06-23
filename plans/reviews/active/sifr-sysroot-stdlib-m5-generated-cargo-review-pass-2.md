I have enough to produce the review. Below are findings, severity-ordered, with file:line references.

---

# M5 Review Pass-2 — Findings

## Verdict
**Satisfied** for M5 PR readiness, with one non-blocking validation gap I recommend addressing before merge. All pass-1 high/medium blockers are properly fixed; the remaining gaps are either forward-looking (M9 prep) or testable-coverage gaps that are mitigated by code structure.

---

## Pass-1 blocker resolution

| # | Pass-1 finding | Status | Evidence |
|---|---|---|---|
| 1 | Workspace `.cargo/config.toml` leaks vendor replacement globally | ✅ Fixed | `.cargo/config.toml:1-3` is now a 2-line placeholder comment. Vendor config is invocation-scoped via `--config` in `crates/sifr_driver/src/build/cargo_manifest.rs:34-47`, and only emitted when `cargo_vendor_mode == SysrootOnly` (line 35). Asserted negatively in `crates/sifr_driver/src/tests/package_project_build_check.rs:331-334`. |
| 2 | `generated_cargo_dependencies` falls back to compile-time source paths | ✅ Fixed | `crates/sifr_stdlib_model/src/features/dependency_plan.rs` no longer contains `legacy_development_*` helpers. Public API is fallible (`try_generated_cargo_dependencies:163-173`, `try_sysroot_dependency_plan:112-124`). Only remaining `.expect` callsite is `crates/sifr_codegen/src/lib_project_codegen.rs:130` — a `#[allow(clippy::expect_used)]` legacy helper used only by `lib_codegen_tests`. |
| 3 | Vendor mode ignores package-project mode | ✅ Fixed | `BuildCompilationMode` is threaded via `requested_vendor_mode_for_build` in `crates/sifr_driver/src/build/entrypoint.rs:329-336`. `cargo_manifest.rs:24-30` combines requested mode and rust-interop path-dep presence. Tests at `cargo_manifest.rs:166-190`. |
| 4 | Missing M5 validation fixtures | ⚠️ Mostly added (see gaps below) | Rendered Cargo.toml snapshot at `cargo_manifest.rs:193-227`; missing-vendor boundary diagnostic at `crates/sifr/tests/sysroot_cli.rs:101-123`; invocation-scoped config at `cargo_manifest.rs:126-154`; package-mode no-config-copy at `package_project_build_check.rs:329-334`; build report sysroot identity at `crates/sifr/tests/build_output_behavior.rs:113-116`; rendered-Cargo.toml cache discriminator at `materialize.rs:505-513`. |
| 5 | Test runner inherits `CARGO_TARGET_DIR` | ✅ Fixed | `crates/sifr_driver/src/test_runner/execution.rs:140` calls `env_remove("CARGO_TARGET_DIR")`. |
| 6 | Retained direct deps lack per-feature attribution | ⚠️ Not fixed (deferred) | `dependency_plan.rs:199-243` still emits flat `Vec<String>`. Acceptable for M5 — no leaves have migrated yet; no functional regression. Becomes blocking at the first leaf migration milestone. |
| 7 | `toml` workspace dep missing `preserve_order` | ✅ Fixed | `Cargo.toml:131` declares `features = ["preserve_order"]`. |
| 8 | `package_name()` and `fingerprint_key()` identical | ⚠️ Deferred | `dependency_plan.rs:18-34`. Explicitly deferred to M6/M8 per user note; no current consumer relies on divergence. |
| 9 | TOML quoting covers only `\` and `"` | ✅ Fixed | Both call sites use `toml::Value::String(...).to_string()` (`dependency_plan.rs:300-302`, `cargo_manifest.rs:116-118`). Test at `cargo_manifest.rs:144-154` and `dependency_plan.rs:310-322`. |
| 10 | `materialize_binary_project_at_path` resolves plan twice | ✅ Fixed | `materialize.rs:33-45` and `52-91` resolve once and thread `dependency_plan` into `materialize_binary_project_at_path`. |

---

## Remaining gaps

### Non-blocking but recommended before merge

**G1. No positive package-mode test for crates.io user-registry resolution.** The leak source from pass-1 #1 is structurally gone (the workspace config is a no-op comment), but there is no positive test that compiles a generated package project with a real or stub crates.io dependency and confirms cargo resolves it through normal channels. The closest test is `test_build_cached_package_project_links_direct_rust_interop_dependency` (`package_project_build_check.rs:354-392`), which only exercises a `DirectCargoDependency` path-dep. A regression that reintroduces `replace-with` into `.cargo/config.toml` would not be caught by any current test. **Fix**: a single fixture that registers a vendored stub crate via package metadata and asserts the build succeeds without invoking `sifr-vendor`.

**G2. No offline/frozen package-mode fixture.** Issue acceptance line 383-384 requires an offline/frozen package fixture proving combined-graph success *or* a clear diagnostic. Neither exists. The architecture doc at `internal_docs/sifr_sysroot_and_stdlib_architecture.md:334-339` explicitly permits initial "clear failure path" — so this can be a negative diagnostic test only.

**G3. No "migrated leaf emits no direct third-party dep" snapshot.** Issue acceptance line 385-386. Cannot land until #6 attribution exists; appropriate to defer with #6 to the first leaf-migration milestone.

### Tracked debt (not M5-blocking)

- **Direct-dep retention attribution (#6).** `retained_direct_dependencies` (`dependency_plan.rs:199-243`) returns a flat `Vec<String>` with no `(feature, reason)` mapping. Add a `RetainedDependency { spec, retained_for: BTreeSet<StdlibFeature>, reason: &'static str }` shape before M9 begins migrating leaves; otherwise the spec's "with a deletion milestone and validation evidence" requirement has nothing to anchor to.
- **`package_name`/`fingerprint_key` collapsibility (#8).** Either consolidate or add a `// diverges at M6/M8 …` comment so future readers don't collapse them.

### Minor observations (no action needed)

- `cargo_manifest.rs:81-98` short-circuits on `DirectCargoDependency` only; `PackageBridge`/`SelfMethod` correctly skipped. The OR with `requested_vendor_mode == PackageOwned` (line 24-30) handles the case where user crates.io deps come via Cargo metadata rather than direct interop.
- `binary_project_cache_key` (`materialize.rs:385-413`) includes the full rendered Cargo.toml *and* the dependency-plan fingerprint, which is over-keyed but safe — vendor-mode flips invalidate cache through two independent channels.
- `BuildSysrootReport` carries identity (`report.rs:46-77`) and is exercised through real CLI flow in `build_output_behavior.rs:110-116` (asserts `sysroot:`, `toolchain:`, `digest:` lines). The unit test in `crates/sifr/src/build_output.rs:140-191` uses a hand-built report; both surfaces are covered.
- Test-runner cargo plan (`test_runner/artifacts.rs:52-69`) hardcodes `CargoVendorMode::SysrootOnly`. Correct for the current test-runner scope, which compiles stdlib-only test code; will need a request-mode-aware variant when package tests pull in user deps.

---

## Validation to rerun before PR/merge (given local Cargo contention caveat)

You should rerun, in this order, once contention clears:

1. **Focused first** — these exercise the M5 surface area directly and are quick:
   - `cargo test -p sifr_stdlib_model` (covers `dependency_plan`, `features_tests`, `*_dependency_snapshots`)
   - `cargo test -p sifr_driver --lib cargo_manifest::` and `materialize::` (vendor-mode and TOML rendering)
   - `cargo test -p sifr_driver --test sifr_driver -- tests::package_project_build_check::test_build_cached_package_project_materializes_namespace_roots tests::package_project_build_check::test_build_cached_package_project_links_direct_rust_interop_dependency`
   - `cargo test -p sifr --test sysroot_cli` and `--test build_output_behavior`
   - `cargo test -p sifr_codegen -- intrinsics::registry_core_tests::runtime_module_dependency_metadata_includes_observability_facades`

2. **Authoritative gate** before opening the PR — `scripts/run_all_tests.sh --profile create-pr`. CLAUDE.md is explicit that this is the merge gate and CI mirrors it exactly.

3. **Local clippy/fmt** if not already in step 2: `cargo clippy --workspace -- -D warnings` and `cargo fmt --check`.

If the create-pr profile run completes cleanly, M5 is ready to merge. G1/G2 can either land in this PR (preferred — both are <50 LOC each) or be tracked as M5-tail follow-ups in `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md`.
