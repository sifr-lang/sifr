# M9 Wave 1 — `_sifr.platform` / `_sifr.html` migration review (pass 1b)

Scope: read-only review of the uncommitted diff that migrates `_sifr.platform`
and `_sifr.html` from compiler intrinsics to `sifr_stdlib`-backed private Rust
interop declarations, plus the supporting feature gating, bootstrap, codegen,
and probe plumbing.

## Findings

### 1. Stale `_sifr.platform` / `_sifr.html` entries in `get_intrinsic_module` — non-blocking, recommended cleanup
- File: `crates/sifr_stdlib_model/src/lib.rs:149,161` (registrations) and
  `crates/sifr_stdlib_model/src/platform_misc.rs:28-64,143-159`
  (`intrinsic_platform()` / `intrinsic_html()` builders).
- After this wave, the compiled `_sifr.platform` and `_sifr.html` modules
  fully own these function signatures via `def ... @rust(...)` in
  `stdlib/_sifr/{platform,html}.sifr`. The bootstrap deliberately prefers the
  compiled exports first (`crates/sifr_driver/src/stdlib/bootstrap.rs:167-216`,
  `has_compiled_exports` branch), so the legacy `intrinsic_platform()` /
  `intrinsic_html()` payloads are now dead in the canonical control flow.
- Risk if left as-is: drift between the legacy hand-written `IntrinsicModule`
  type signatures and the Sifr/Rust source of truth — e.g. if a future change
  edits `platform_node` in `_sifr/platform.sifr` (or relaxes its convention),
  the legacy entry would silently disagree. The fallback branch
  (`bootstrap.rs:191-216`) only fires when compiled exports are missing, but a
  future bug in source-loading or ordering could cause it to fire and surface
  the stale legacy signature.
- Suggested follow-up (not blocking): remove the migrated entries from
  `get_intrinsic_module` and delete the unused builder functions, or leave a
  TODO calling out the dual-registration explicitly. Either way, capture this
  in the M9 plan so subsequent waves either pattern-match this cleanup or
  document why the entries are retained.

### 2. `dependency_features` assumes "feature = `segments[1]`" — works for the current shape, should be hardened or documented
- File: `crates/sifr_driver/src/build/rust_interop_probe.rs:166-194`.
- The helper enables `sifr_stdlib` Cargo features by reading the second
  segment of the canonical target path (`sifr_stdlib.<feature>.<func>`), then
  checking the backend manifest for a matching `[features]` key.
- Today, the two migrated leaves and the existing planned dependency mapping
  (`crates/sifr_stdlib_model/src/features/generated_stdlib_features.rs:28-39`)
  all follow that shape, and `crate_feature_exists` correctly suppresses the
  feature on temp crates that lack it.
- Future risk: if a later wave introduces a `@rust(sifr_stdlib.<func>)` (no
  feature segment), or nests targets under a non-feature submodule, the
  probe will silently omit feature gating and might fall back to a stub-less
  build. The test
  `sysroot_stdlib_probe_features_follow_target_module_segment` only
  exercises the happy `sifr_stdlib.platform.platform_system` path; it does
  not assert the "no-feature segment" or "segment not declared as a feature"
  outcomes.
- Suggested follow-up (not blocking): document the assumption in a doc
  comment on `dependency_features` and add a probe test asserting that an
  unrelated second segment yields an empty feature list — both behaviours
  the migration relies on for temp/probe crates without those features.

### 3. Full pre-PR validation suite not yet run — process gap per `AGENTS.md`
- `AGENTS.md` mandates running `scripts/run_all_tests.sh --profile create-pr`
  before considering a change done. The task statement enumerates only
  focused `sifr_driver`/`sifr_codegen`/`sifr_stdlib_model` tests plus
  emit/run demos.
- Not a code-level finding; flagged so the create-PR profile is run before
  the PR is opened. The new prescan flow for `_sifr.*` imports
  (`crates/sifr_codegen/src/module_prescan.rs:13-52`) is reachable from
  every public stdlib module that imports `from _sifr.*`, not just the two
  migrated leaves, so the wider suite is the appropriate gate.

### 4. Test coverage for the bootstrap rewrite is shallow on the new branches — non-blocking
- File: `crates/sifr_driver/src/stdlib/bootstrap_tests.rs` is just an
  extraction of the pre-existing tests from `bootstrap.rs`; no new test
  exercises the new `has_compiled_exports` branch at
  `bootstrap.rs:167-216` or the new "skip only when private & empty" gate at
  `bootstrap.rs:110-117`.
- `stateless_private_codegen_tests.rs` does cover the end-to-end codegen
  outcome (private code references `sifr_stdlib::...`, intrinsic names are
  empty, transitive deps include `_sifr.*`). That is reasonable end-to-end
  coverage for this wave, but a focused unit test for the bootstrap branch
  selection would make the new control flow easier to maintain as later
  waves migrate more leaves.

## Notes (informational; not findings)

- `crates/sifr_stdlib_model/src/sources.rs:359-381` reorders sources so
  `PRIVATE_STDLIB_MODULES` are loaded *before* the public set. This is
  load-bearing for the new `has_compiled_exports` lookup in `bootstrap.rs`
  and is correct, but it is a subtle semantic change. No existing test
  asserts the previous (public-first) order, and downstream consumers
  (`crates/sifr_driver/src/stdlib/tooling.rs:60-85`) treat the list
  order-independently, so this should be safe.
- `platform_release` / `platform_version` shell out to `uname -r` / `-v`
  via `std::process::Command::output().ok()`. This preserves the previous
  intrinsic's behavior (errors are swallowed, empty output falls back to
  `std::env::consts::OS`). Compatible with `panic=trusted_no_panic` only
  because the subprocess error path is fully eaten; worth noting that the
  trust label is somewhat aspirational for these two leaves, but no
  regression vs. pre-migration behavior.
- The intrinsic registry deletes
  (`crates/sifr_codegen/src/intrinsics/registry/{html,platform}.rs`,
  removal of the `mod html;` / `mod platform;` entries and lowering arms
  in `registry.rs`) are complete — no stale references in the intrinsics
  subtree.
- `dependency_line` does not add `default-features = false` to probe
  manifests, while user-facing generated `Cargo.toml`s do
  (`crates/sifr_stdlib_model/src/features/dependency_plan.rs:60-76`).
  Today this is harmless because `sifr_stdlib`'s `default = []`, but it
  is a divergence between probe and final-crate dependency shapes worth
  remembering as features grow.
- Public/private routing verified end-to-end:
  - `stdlib/sifr/{platform,html}.sifr` continue to import the leaf names
    from `_sifr.*` and re-export through public functions.
  - `re_export_stdlib_imports` copies the leaf functions into
    `stdlib_defs.functions["sifr.platform"]` /
    `stdlib_defs.functions["sifr.html"]`, so user code can do
    `from sifr.platform import platform_system, platform_arch, ...` as
    demonstrated in `demos/platform/main.sifr` without those names ever
    being registered as intrinsics
    (`stdlib_code.intrinsic_names["sifr.platform"]` is empty after the
    migration, per `stateless_private_codegen_tests`).
- `feature_contract.rs` and `Cargo.toml` consistently declare `html` and
  `platform` as leaf features with empty transitive deps, matching the
  pure-`std` implementations.
- The `module_prescan.rs` rewrite is safe for non-migrated `_sifr.*`
  modules: the `else` branch at lines 43-51 still treats imports as
  intrinsic whenever the imported module has no entry in
  `stdlib_intrinsic_names` (the case for every still-empty `_sifr.*`
  placeholder), preserving prior behaviour for them.

VERDICT: PASS
