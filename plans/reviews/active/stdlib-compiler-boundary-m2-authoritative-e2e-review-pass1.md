# M2 review pass 1 — Authoritative E2E Dependency Planning

## Verdict: **SATISFIED**

No blocking findings. Every M2 acceptance criterion is met by the diff, the
inventory document, and the referenced validation evidence. The E2E harness
now derives dependencies solely from typed compiler metadata plus the
production `SysrootDependencyPlan`; generated-Rust text scanning and the
handwritten stdlib/runtime dependency maps are entirely gone. Three findings
below are non-blocking readability/test-strengthening notes.

## Scope reviewed

- Rearchitecture plan §M2 (`plans/issues/active/ad-hoc-stdlib-compiler-boundary-rearchitecture.md:282-315`).
- Checked former-rule inventory (`plans/issues/active/ad-hoc-stdlib-compiler-boundary-m2-inventory.md`).
- Public/typed dependency API in `crates/sifr_driver/src/build/cargo_manifest.rs`
  and its re-exports (`crates/sifr_driver/src/build/mod.rs`,
  `crates/sifr_driver/src/lib.rs`).
- E2E harness rewrites: `harness_model.rs`, `fixture_compilation.rs`,
  `batch_execution.rs`, `harness_behavior_tests.rs`, `mod.rs`.
- New authority test suite `crates/sifr/tests/e2e_support/dependency_plan_authority_tests.rs`.
- Milestone demo `demos/authoritative_dependency_plan_demo.sifr`.
- Deleted files (7): `fixture_cargo_toml.rs`, `fixture_dependency_paths.rs`,
  `fs_dependency_inference_tests.rs`, `network_http_dependency_rules_tests.rs`,
  `runtime_observability_dependency_tests.rs`,
  `stateless_sysroot_cargo_toml_tests.rs`,
  `structured_data_cargo_toml_tests.rs`.
- Cross-checks against `sifr_stdlib_manifest::SysrootDependencyPlan`,
  `features_for_stdlib_module`, `feature_for_codegen_requirement`,
  `planned_sifr_stdlib_features`, and `retained_dependency_specs`.
- `sifr_driver` clippy (workspace lints, `-D warnings`) — clean on the changed
  crate.

## Acceptance-criteria audit

- **Typed compiler feature metadata preserved in `CompiledCase`.**
  `harness_model.rs:63-71` stores `used_stdlib_modules: HashSet<String>` and
  `required_features: HashSet<StdlibFeature>` verbatim; the old
  `required_crates: BTreeSet<String>` name-based collapse in
  `fixture_compilation.rs` is gone (`fixture_compilation.rs:36-52,53-79`).
  `sifr_driver::CompileResultFull::Success` already exposes both fields
  (`crates/sifr_driver/src/diagnostics.rs:18-29`); nothing intermediate rewrites
  the identity.
- **Production `SysrootDependencyPlan` resolved and stored per fixture.**
  `CompiledCase.dependency_plan` is populated in `compile_fixture` via
  `sifr_driver::try_generate_standalone_dependency_plan(used_stdlib_modules,
  required_features)` (`fixture_compilation.rs:196-224`). That function is the
  new authoritative M2 entrypoint (`cargo_manifest.rs:26-36`) and delegates to
  `try_generate_sysroot_dependency_plan` with `InteropBuildPlan::default()` and
  `CargoVendorMode::SysrootOnly`, matching what `sifr build`/`sifr run` use for
  pure-Sifr binaries.
- **Batch groups and caches key on resolved plan identity.**
  `DependencyFingerprint` is now `{ dependency_inputs, resolved_plan }`
  (`harness_model.rs:73-77, 388-399`), sourced from
  `SysrootDependencyPlan::dependency_input_fingerprint()` and
  `SysrootDependencyPlan::cache_fingerprint`
  (`crates/sifr_stdlib_manifest/src/features/dependency_plan.rs:87-125`).
  `BatchGroup` also carries the plan (`harness_model.rs:79-88`).
  `build_group_sources` rejects mixed plans:
  `"batch group contains non-identical production dependency plans"`
  (`fixture_compilation.rs:238-256`). `E2E_CACHE_SCHEMA_VERSION` bumps 1→2
  (`harness_model.rs:7`), and `cache_key_for_group` mixes both the schema and
  the new fingerprint signature (`fixture_compilation.rs:535-555`), so pre-M2
  cache entries invalidate cleanly.
- **Grouped Cargo generated from the plan, not module/crate switches.**
  Batch: `sifr_driver::generate_dependency_cargo_toml(&group.package_name,
  &group.dependency_plan)` (`batch_execution.rs:55-58`). Standalone-with-deps:
  same public API (`batch_execution.rs:359`). Both call
  `sysroot_cargo_config_args(&plan)` **before** the `build` subcommand
  (`batch_execution.rs:69-72, 367-369`), which is the correct cargo argument
  position for `--config` overrides. The switch-style renderer
  (`fixture_cargo_toml.rs`, 355 lines) is deleted outright.
- **`infer_dependencies` and every generated-Rust dependency scan deleted.**
  `grep -R 'infer_dependencies\|normalize_dependency_set\|join_sorted' crates
  scripts verification` returns zero hits. `harness_model.rs` no longer exposes
  any scanner. The old `_union_stdlib`/`_union_crates` union-across-cases
  computation in `build_group_sources` and the parallel union in
  `build_batch_group` are removed (`fixture_compilation.rs:257-289`,
  `batch_execution.rs:54-73`).
- **Duplicate E2E stdlib/runtime feature and dependency maps deleted.**
  Seven files (~1,000 lines) removed. The only remaining E2E-side type surface
  is the shared re-export `pub(crate) use sifr_stdlib_manifest::{StdlibFeature,
  SysrootDependencyPlan};` (`mod.rs:1`); no fixture-side feature/crate table
  exists.
- **Inventory covers every former inference rule.**
  `plans/issues/active/ad-hoc-stdlib-compiler-boundary-m2-inventory.md`
  enumerates 6 module rules, 1 runtime-crate rule, and 29 direct-crate rules
  (35 unique former rules, matching §M2). The bounded corpus matches the plan
  verbatim: `_bigint`, `_sifr.fs`, `_sifr.net`, `_sifr.tls`, `_sifr.http`,
  `_sifr.signal`; `sifr_runtime`; and `regex`, `rand`, `rand_distr`, `chrono`,
  `md5`, `uuid`, `toml`, `flate2`, `zip`, `base64`, `sha1`, `sha2`, `blake2`,
  `rust_decimal`, `bigdecimal`, `tracing`, `metrics`, `postcard`, `url`,
  `percent-encoding`, `http`, `bytes`, `h2`, `http-body`, `http-body-util`,
  `hyper`, `hyper-util`, `tower-service`, `cookie`. Each row cites the typed
  production authority (`features_for_stdlib_module`,
  `feature_for_codegen_requirement`, `planned_sifr_stdlib_features`, or a
  structural codegen emission) and the checked production resolution (crate
  path, transitive `sifr_stdlib` feature, or documented obsolete substring
  match). `dependency_plan_authority_tests.rs:4-64` mirrors the tables as
  `FORMER_MODULE_RULES` (six entries incl. `_bigint`) and `FORMER_DIRECT_RULES`
  (29 entries), and iterates both plus the runtime-crate rule with `assert_eq!`
  on the typed authority and a plan-resolution assertion.
- **Sysroot Cargo config/vendor mode used for grouped builds.**
  `build_batch_group` and `build_and_run_capture_with_deps` both call
  `sifr_driver::sysroot_cargo_config_args(&plan)` prior to the `build`
  subcommand (`batch_execution.rs:69, 367`). The helper only emits `--config`
  args in `SysrootOnly` mode (`cargo_manifest.rs:56-69`); package-owned mode
  produces zero args, matching prior `sifr build` behavior.
- **Missing-metadata regression exists and fails as expected.**
  `dependency_plan_authority_tests.rs:181-194`:
  ```
  let error = build_and_run_capture_with_deps(
      "use num_bigint::BigInt;\nfn main() { let _ = BigInt::from(1); }",
      "missing_dependency_metadata",
      &HashSet::new(),
      &HashSet::new(),
  ).expect_err("...");
  assert!(error.contains("Rust compilation failed"));
  assert!(error.contains("num_bigint"));
  ```
  With empty typed inputs the standalone plan has no crates or retained deps,
  so the generated Cargo.toml cannot resolve `num_bigint` — Rust build fails,
  and there is no code path that repairs it from the source text.
- **Cross-fixture regression exists.**
  `dependency_plan_authority_tests.rs:196-217` asserts that two cases with
  non-identical plans (`empty` vs `NumBigint`-required) both (a) fail
  `build_group_sources` with `"non-identical production dependency plans"` and
  (b) fall into two separate `plan_batches` buckets. This confirms
  bucket-and-build-time isolation and eliminates the previous unioning path.
- **Cache identity changes with the plan.**
  `dependency_plan_authority_tests.rs:219-240` builds two groups with
  otherwise-identical fixture bytes but different `required_features`, and
  asserts both `dependency_fingerprint()` and `cache_key_for_group(...)`
  differ. Because `resolved_plan` in the fingerprint is
  `plan.cache_fingerprint` — which includes sysroot content SHA, toolchain id,
  crate feature sets, retained direct deps, and vendor mode
  (`dependency_plan.rs:218-247`) — any production-plan change propagates to the
  cache key.
- **Public API design.**
  Two new `pub` entrypoints in `sifr_driver`
  (`crates/sifr_driver/src/lib.rs:22-25`): `generate_dependency_cargo_toml`
  (project-name + plan; wraps the interop-aware renderer with
  `InteropBuildPlan::default()`) and `try_generate_standalone_dependency_plan`
  (typed inputs → plan). The interop-carrying helper is renamed
  `generate_dependency_cargo_toml_with_interop` and remains `pub(crate)`; only
  the standalone/vendor-config surfaces are made public. `sysroot_cargo_config_args`
  is elevated to `pub` so the harness can apply the same `--config` line the
  production build uses. This keeps the interop-plan path private to the
  driver while exposing precisely the surfaces the E2E harness needs.
- **Safety, panics, unsafe.** No new `unsafe`, `unwrap`, `expect`, or panics
  in generated-project paths. `expect` calls in `dependency_plan_authority_tests.rs`
  are inside `#[test]` fns (test-code invariants), which is acceptable.
- **File-size guardrail.** Largest touched hand-maintained files:
  `harness_behavior_tests.rs` 765, `batch_execution.rs` 721, `harness_model.rs`
  707, `fixture_compilation.rs` 598, `cargo_manifest.rs` 489, and
  `dependency_plan_authority_tests.rs` 240 lines — all below the 900-line cap.
- **Milestone demo.** `demos/authoritative_dependency_plan_demo.sifr`
  exercises `sifr.base64`, `sifr.hashlib`, and `sifr.io` — three independent
  typed-feature paths — through one production plan; task notes it passes.

## Findings (non-blocking)

### 1. `compiled_fixture_plan_matches_standalone_production_plan` is tautological

File: `crates/sifr/tests/e2e_support/dependency_plan_authority_tests.rs:159-179`.

The test compiles `runtime_diagnostics_tracing.sifr` via `compile_fixture`,
then re-invokes `try_generate_standalone_dependency_plan` with the same
`(used_stdlib_modules, required_features)` inputs and asserts equality.
Because `compile_fixture` itself already stores the result of
`try_generate_standalone_dependency_plan(used_stdlib_modules,
required_features)` in `CompiledCase.dependency_plan`
(`fixture_compilation.rs:196-224`), the comparison reduces to
`try_generate_standalone_dependency_plan(inputs) ==
try_generate_standalone_dependency_plan(inputs)`, i.e. determinism only. It
does not demonstrate that the harness plan matches what
`sifr build`/`materialize_binary_project_with_report` would produce for the
same source (that path invokes `try_generate_sysroot_dependency_plan` with the
generated project's `InteropBuildPlan`, `materialize.rs:33-39`).

For all current E2E pass fixtures the two paths are in fact equivalent —
`grep -r '@rust(' crates/sifr/tests/e2e/pass/` returns zero hits, so the
interop plan is `InteropBuildPlan::default()` and the two production
entrypoints produce identical plans. The 648/648 cache-disabled pass suite is
the operational proof. Nevertheless, this specific test does not carry that
weight on its own.

Suggested strengthening (non-blocking, could land in a follow-up): compare
against a plan built via `try_generate_sysroot_dependency_plan(…,
&InteropBuildPlan::default(), CargoVendorMode::SysrootOnly)` from the
`sifr_driver::build` module, or invoke `sifr build` and diff its recorded
`BuildSysrootReport` against the harness `dependency_plan`.

### 2. Leftover `required_crates` variable name in e2e_entrypoints.rs

File: `crates/sifr/tests/e2e_support/e2e_entrypoints.rs:52,77,384,401`.

`compile_source_with_metadata` now returns `HashSet<StdlibFeature>` for the
third tuple element, but the destructured name in `test_e2e_pass` and
`test_e2e_runtime_fail` is still `required_crates`. The type is correct and
the values flow to `build_and_run_with_deps`/`build_and_run_capture_with_deps`
whose fourth parameter is `&HashSet<StdlibFeature>`
(`batch_execution.rs:337, 401`), so runtime behavior is right. It is only a
readability lag from the pre-M2 name. Renaming to `required_features` would
match the rest of the codebase.

### 3. `_bigint` inventory assertion checks feature count, not identity

File: `crates/sifr/tests/e2e_support/dependency_plan_authority_tests.rs:102-121`.

For `_bigint` the test enters a branch that constructs
`(HashSet::new(), expected_features.iter().copied().collect())`, then asserts
`plan.required_features.len() == expected_features.len()`. Since
`expected_features` is copied straight into `features`, this is trivially
true and would still pass if `expected_features` were replaced with an
unrelated pair of `StdlibFeature` variants. The neighboring
`assert_eq!(features_for_stdlib_module(module), *expected_features, "{module}")`
already fixes the module→features table (line 97-101), so the resolved-plan
assertion is defense-in-depth rather than the primary check. Suggested
strengthening: iterate `expected_features` and assert
`plan.required_features.contains(feature)` for each, matching the
`FORMER_DIRECT_RULES` pattern at line 136.

## Validation gaps

- No end-to-end comparison against a `sifr build`-produced Cargo manifest (see
  Finding 1). The current gate is transitive — 648/648 cache-disabled pass
  fixtures build under the plan-derived Cargo.toml — but no test snapshots the
  two plans side by side.
- The plan lists 143 batch groups for 648 pass fixtures with 0 cache hits;
  the group count is not asserted anywhere, so a future regression that
  accidentally over-groups (union-style) would only be caught if it happens
  to build a mixed-plan group and hit the new
  `"non-identical production dependency plans"` guard. That guard is a strict
  correctness barrier, not a coverage metric — acceptable, but worth calling
  out.
- No self-test yet covers the `sifr_cargo_config_args` in-flight arg placement
  (i.e. that `--config` precedes the `build` subcommand). The driver-side
  tests in `cargo_manifest.rs:302-342` cover the produced values, not their
  cargo invocation position. Runtime evidence from the 648/648 sysroot-only
  suite indirectly confirms placement; a unit test that asserts the argv
  ordering would harden it.

None of the above are blocking. Each is a minor test-strengthening
opportunity for a follow-up commit or a later milestone.

## Rust 1.94 workspace clippy

Ran `cargo clippy -p sifr_driver --no-deps -- -D warnings`: no warnings on the
changed crate. Workspace-wide clippy still reports pre-existing lints on
unrelated code as noted in the task; none are attributable to M2-changed lines.

## Final verdict

**SATISFIED.**
