# M2 review pass 2 — Authoritative E2E Dependency Planning

## Verdict: **NOT SATISFIED**

The repair itself is architecturally sound: the E2E harness now consumes the
same `SysrootDependencyPlan` a `sifr build` would produce, including the
resolved private-stdlib interop crates that pass 1 correctly flagged as
missing from raw compile metadata. The strengthened parity test is real. All
three pass 1 non-blocking suggestions are addressed. However, the repair
introduces a hand-maintained file-size guardrail violation on
`crates/sifr_driver/src/build/entrypoint.rs`, which is one of the explicit
merge-gate invariants named in `AGENTS.md`. That guardrail is not a
suggestion; a merged M2 must pass it because the local `run_all_tests.sh
--profile create-pr` invocation runs it.

## Scope reviewed

- Full working-tree diff versus HEAD (23 files, +282 / −1281).
- Rearchitecture plan `plans/issues/active/ad-hoc-stdlib-compiler-boundary-rearchitecture.md`
  §M2 (lines 282–315) and the M2 inventory
  `plans/issues/active/ad-hoc-stdlib-compiler-boundary-m2-inventory.md`.
- Pass 1 review verdict and non-blocking findings at
  `plans/reviews/active/stdlib-compiler-boundary-m2-authoritative-e2e-review-pass1.md`.
- Repaired dependency-planning API surface:
  `crates/sifr_driver/src/build/cargo_manifest.rs`,
  `.../build/materialize.rs`, `.../build/entrypoint.rs`,
  `.../build/mod.rs`, `.../lib.rs`,
  `.../frontend/api.rs`, `.../diagnostics.rs`,
  `.../test_runner/artifacts.rs`, `.../build/sysroot_interop_tests.rs`.
- E2E harness: `crates/sifr/tests/e2e_support/{mod,harness_model,fixture_compilation,batch_execution,e2e_entrypoints,harness_behavior_tests,dependency_plan_authority_tests}.rs`.
- Cross-checks against `sifr_stdlib_manifest::{try_sysroot_dependency_plan,
  SysrootDependencyPlan, CargoVendorMode, SysrootCrate}` and
  `sifr_codegen::InteropBuildPlan::cache_key_fragment`.
- Milestone demo `demos/authoritative_dependency_plan_demo.sifr`.
- Local `python3 scripts/check_file_size_guardrails.py` run (see Finding 1).

## Findings

### 1. `crates/sifr_driver/src/build/entrypoint.rs` violates the 900-line guardrail (BLOCKING)

`crates/sifr_driver/src/build/entrypoint.rs` is 928 lines after this patch
(863 lines at HEAD). The M2 diff adds ~65 lines to
`compile_single_file_entrypoint_with_metadata_and_options` for the resolved
`InteropBuildPlan` cache. Ran locally:

```
$ python3 scripts/check_file_size_guardrails.py
file-size guardrails: FAIL
- crates/sifr_driver/src/build/entrypoint.rs: 928 lines (limit 900, category rust)
```

`AGENTS.md` treats this cap as a hard invariant:

> **File-size guardrail**: Hand-maintained first-party source files must stay
> under **900 lines**. […] Run the file-size guardrail before considering
> work complete. If a touched file exceeds the cap, refactor it by
> responsibility rather than adding more code to an oversized module.

Local validation (`scripts/run_all_tests.sh --profile create-pr`) runs
`check_file_size_guardrails.py` per the plan's Validation Matrix
(`plans/issues/active/ad-hoc-stdlib-compiler-boundary-rearchitecture.md:498`).
The current state fails that gate.

The additions form a well-scoped responsibility unit — sysroot-identity
cache key derivation plus the `RESOLVED_STDLIB_INTEROP` static — that can be
split into a small submodule (e.g., `build/stdlib_interop_cache.rs`) that
exposes a `resolve_or_cache(codegen_result, stdlib_interop,
rust_interop_context) → GeneratedBinaryProject` seam. `entrypoint.rs`
then keeps only the entrypoint plumbing. Refactoring the module by
responsibility is the guardrail's stated remediation, not increasing the
line cap.

### 2. Resolved-interop cache omits `LoweringOptions` from the cache key (non-blocking, latent)

`crates/sifr_driver/src/build/entrypoint.rs:161-221` caches the resolved
`InteropBuildPlan` keyed on `sysroot_identity` + `stdlib_interop.plan.cache_key_fragment()`.
The `compile_single_file_entrypoint_with_metadata_and_options`
`lowering_options` parameter is not part of that key. Today no lowering
option affects the resolved interop (options steer HIR lowering and Python
runtime plumbing rather than `@rust` decorator surface), so the omission
is functionally correct against the current corpus. If a future
`LoweringOptions` flag ever alters the emitted declarations the cache
would silently return stale results. Consider including a hash of
`lowering_options` in the key or documenting the invariant next to the
cache.

### 3. Cache mutex is held across resolution (non-blocking, minor perf smell)

`crates/sifr_driver/src/build/entrypoint.rs:199-221` acquires the
`RESOLVED_STDLIB_INTEROP` mutex before the miss path and holds it while
running `attach_stdlib_rust_interop` and `apply_package_rust_interop_metadata`.
Concurrent cache misses with distinct keys serialize behind a single
critical section. The E2E workload almost always hits after the first miss
(all 648 fixtures share one sysroot), so wall-clock impact is negligible;
the observed `compile=14.4s / total=218.6s` in the task's validation notes
matches this expectation. Non-blocking, but a `MutexGuard`-drop-before-work
pattern (compute the plan outside the lock, insert only if key is still
absent) removes the smell without altering semantics.

## Acceptance-criteria audit

- **Missing metadata is a real Rust build failure, not a scanner repair.**
  `dependency_plan_authority_tests.rs:222-236` (`missing_metadata_is_not_repaired_from_generated_rust`)
  passes empty `(HashSet, HashSet)` metadata plus the interop default and
  asserts the Cargo build fails with `Rust compilation failed` and
  `num_bigint` in the message. No harness path scans the Rust source.
- **Production `SysrootDependencyPlan` with resolved private-stdlib interop
  is authoritative.** `crates/sifr_driver/src/build/cargo_manifest.rs:26-55`
  centralises `try_generate_standalone_dependency_plan` → `try_generate_sysroot_dependency_plan`
  → `add_sysroot_interop_crates`. The interop-carrying variant is what
  `materialize.rs:33-38` and `materialize.rs:66-72` use for production
  builds and cache misses. E2E callers pass the same resolved
  `InteropBuildPlan` returned by `compile_source_with_metadata`
  (`fixture_compilation.rs:34-57, 209-224`,
  `batch_execution.rs:333-350`, `e2e_entrypoints.rs:52-78, 385-403`).
- **Real production build-report parity is asserted per fixture.**
  `dependency_plan_authority_tests.rs:173-220`
  (`compiled_fixture_plan_matches_production_build_report`) compiles
  `runtime_diagnostics_tracing.sifr`, then invokes the actual
  `sifr_driver::build_single_file_report(...)` and asserts equality of
  `dependency_input_fingerprint()`, `cache_fingerprint`, `sysroot_root`,
  `toolchain_id`, and `sysroot_content_sha256`. The fixture is
  representative because its typed feature set forces `SifrRuntime`,
  `Tokio`, `Tracing`, and the sysroot `sifr_stdlib` crate — the exact
  kind of graph the pass 1 review said needed independent verification.
  This resolves pass 1 finding 1.
- **Grouping, cache, and vendor mode key on the resolved plan.**
  `harness_model.rs:73-77, 388-399` — `DependencyFingerprint = {
  dependency_inputs, resolved_plan }`, sourced from
  `plan.dependency_input_fingerprint()` and `plan.cache_fingerprint`.
  `fixture_compilation.rs:250-270` — `build_group_sources` refuses to
  merge cases whose `dependency_plan` values differ verbatim
  (`"non-identical production dependency plans"`).
  `fixture_compilation.rs:372-417` — `plan_batches` buckets by
  `DependencyFingerprint`. `E2E_CACHE_SCHEMA_VERSION = 2`
  (`harness_model.rs:7`) and `cache_key_for_group`
  (`fixture_compilation.rs:556-576`) mix the schema and full fingerprint
  signature — any change in the resolved plan invalidates cached artifacts.
  `batch_execution.rs:69-72, 371` applies
  `sifr_driver::sysroot_cargo_config_args(&plan)` before `build`, which
  is the position cargo requires. Vendor mode toggling is verified by
  `sysroot_cargo_config_args_leave_package_owned_mode_alone` and
  `dependency_plan_honors_sysroot_only_request`
  (`crates/sifr_driver/src/build/cargo_manifest.rs:336-368`).
- **BigInt inventory now asserts feature membership.**
  `dependency_plan_authority_tests.rs:96-131` iterates
  `FORMER_MODULE_RULES`, calls `features_for_stdlib_module(module)`
  against the row directly, and for `_bigint` asserts
  `plan.required_features.len() == expected_features.len()` AND
  `plan.required_features.contains(feature)` for every declared feature.
  Resolves pass 1 finding 3.
- **`required_features` naming is consistent.**
  `e2e_entrypoints.rs:52,77,385,402` renames the destructured tuple
  entries to `required_features`, matching the field type
  (`HashSet<StdlibFeature>`) and the harness. `grep -rn "required_crates"
  crates/` returns only unrelated `verification/areas/rust_interop`
  fixture-matrix JSON entries. Resolves pass 1 finding 2.
- **Duplicate E2E stdlib/runtime maps and inference rules deleted.**
  Verified via `git diff --stat`:
  `fixture_cargo_toml.rs` (355), `fixture_dependency_paths.rs` (186),
  `fs_dependency_inference_tests.rs` (13),
  `network_http_dependency_rules_tests.rs` (157),
  `runtime_observability_dependency_tests.rs` (13),
  `stateless_sysroot_cargo_toml_tests.rs` (118), and
  `structured_data_cargo_toml_tests.rs` (25) are all removed
  outright. `grep -R 'infer_dependencies\|normalize_dependency_set' crates
  scripts verification` returns zero hits. The remaining E2E-side
  imports collapse to `pub(crate) use
  sifr_stdlib_manifest::{StdlibFeature, SysrootDependencyPlan};`
  (`mod.rs:1`).
- **Inventory covers every former rule.** The 6 module rules, 1 runtime
  rule, and 29 direct-crate rules in
  `ad-hoc-stdlib-compiler-boundary-m2-inventory.md` correspond 1:1 to
  the enum tables in `dependency_plan_authority_tests.rs:4-64`. Every
  direct-crate row is asserted through
  `feature_for_codegen_requirement(requirement) == Some(feature)` and
  `plan.required_features.contains(feature)`. The inventory also
  documents the production-parity and cache-per-sysroot-identity
  invariants introduced by this pass.
- **`InteropBuildPlan` re-export API.**
  `crates/sifr_driver/src/lib.rs:22-25,35` re-exports
  `generate_dependency_cargo_toml`,
  `try_generate_standalone_dependency_plan`,
  `sysroot_cargo_config_args`, and (new)
  `sifr_codegen::InteropBuildPlan` as public. The interop-carrying
  variants stay `pub(crate)` (`build/mod.rs:52-54`). E2E callers use
  exactly the public surface (`sifr_driver::InteropBuildPlan::default()`,
  `try_generate_standalone_dependency_plan(_, _, &interop)`) — no test
  reaches into the private helpers.
- **`CompileResultFull::Success` carries the resolved interop.**
  `crates/sifr_driver/src/diagnostics.rs:18-29` adds `interop:
  sifr_codegen::InteropBuildPlan`. `frontend/api.rs:48-62` pipes
  `codegen_result.interop` through, where `codegen_result` is now
  `CompiledSingleFileMetadata` whose `interop` field holds the
  post-`apply_package_rust_interop_metadata` plan
  (`entrypoint.rs:206-228`).
- **Test-runner Cargo generation still resolves.**
  `crates/sifr_driver/src/test_runner/artifacts.rs:1-70` renames
  `generate_dependency_cargo_toml_for_cache_key` to
  `generate_dependency_cargo_toml_with_interop` and preserves
  `CargoVendorMode::SysrootOnly` for the sifr_tests project.
- **Rust source behavior — resolved-plan interop propagates without a
  parallel resolution path.** The cache path in
  `entrypoint.rs:194-221` gates on `codegen_result.interop ==
  InteropBuildPlan::default()`, so any user `@rust` bridge in the
  fixture source (which populates
  `codegen_result.interop.rust.declarations`) always falls through to
  the fresh
  `attach_stdlib_rust_interop`+`apply_package_rust_interop_metadata`
  path. User interop behavior is preserved because the cache is
  transparent for it.
- **Cache correctness against the sysroot.** The cache key includes
  `sysroot.root`, `sysroot.toolchain_id()`, and
  `sysroot.manifest.sysroot_content_sha256`. Because the sysroot
  content hash covers the private-stdlib source that seeds the
  interop plan, and because
  `stdlib_interop.plan.cache_key_fragment()` records the plan itself
  (declarations, targets, contracts, probes, cargo inputs — see
  `crates/sifr_codegen/src/rust_interop_plan.rs:13-70`), collisions
  require two sysroots with identical content sha AND identical
  interop plan, i.e. the same effective sysroot. The stdlib-interop
  cache is process-global and unbounded; for the intended callers
  (compiler CLI, E2E test binary) that is acceptable, but the pattern
  should be reviewed if the driver is ever embedded in a long-lived
  daemon that hot-swaps sysroots.
- **Concurrency.** The cache uses a single `std::sync::Mutex`; poison
  is handled with `unwrap_or_else(std::sync::PoisonError::into_inner)`
  (no new panic). No lock is held across a `Result`-returning path
  that could unwind and leave inconsistent state — the insert is the
  last mutation. See Finding 3 for the minor performance smell.
- **Error paths.** `try_generate_standalone_dependency_plan` continues
  to surface `SysrootError` through `.boundary_message()` at both call
  sites (`batch_execution.rs:342-350`,
  `fixture_compilation.rs:214-224`). No new user-project unwrap or
  expect was introduced (`git diff` for `entrypoint.rs` shows the sole
  addition is the poison-recovery). `expect` calls in the new
  authority tests are inside `#[test]` fns and are acceptable.
- **Milestone demo.**
  `demos/authoritative_dependency_plan_demo.sifr` exercises
  `sifr.base64`, `sifr.hashlib`, and `sifr.io` in one binary; the task
  reports it builds and runs under the plan-derived Cargo.toml.

## Validation gaps

- Local file-size guardrail fails (Finding 1). The reported
  validation set does not include
  `python3 scripts/check_file_size_guardrails.py`, so the failure
  would only surface at `scripts/run_all_tests.sh --profile
  create-pr`. The task description marks file sizes as an explicit
  verification item; that check regresses.
- The parity assertion is exercised on exactly one fixture
  (`runtime_diagnostics_tracing.sifr`). The 648/648 cache-disabled
  pass suite is the transitive proof that every fixture builds under
  its production-derived Cargo.toml, but no test enumerates the
  broader corpus for byte-exact fingerprint equality. This mirrors
  pass 1's remaining transitive-only gap and is acceptable given the
  strengthened per-fixture check plus the cross-fixture regression
  and cache-identity regressions.
- No self-test asserts the argv order of
  `sifr_cargo_config_args(&plan)` when invoked (i.e., that
  `--config` precedes the `build` subcommand). Runtime evidence from
  the 648/648 sysroot-only suite continues to confirm placement.

## File-size and lint checks

- `python3 scripts/check_file_size_guardrails.py` — **FAIL**
  (`crates/sifr_driver/src/build/entrypoint.rs: 928 lines`).
- `python3 scripts/check_hir_maintainability_guardrails.py` — PASS.
- `wc -l` on touched hand-maintained files (all others under cap):
  `harness_behavior_tests.rs` 767, `batch_execution.rs` 731,
  `harness_model.rs` 722, `e2e_entrypoints.rs` 700,
  `materialize.rs` 658, `fixture_compilation.rs` 619,
  `cargo_manifest.rs` 490, `dependency_plan_authority_tests.rs`
  282.

## Final verdict

**NOT SATISFIED.**

The M2 architectural repair is correct: resolved private-stdlib interop
crates are threaded into the production `SysrootDependencyPlan`, the E2E
harness consumes that exact plan, the harness↔production parity is now
asserted against a real `build_single_file_report`, and every pass 1
non-blocking finding is closed. What blocks acceptance is the concrete
file-size guardrail violation on
`crates/sifr_driver/src/build/entrypoint.rs`. Under `AGENTS.md`, an M2 PR
whose local validation gate fails cannot merge; the fix is to extract the
new resolved-interop cache into its own responsibility-scoped module so
`entrypoint.rs` returns under 900 lines. Findings 2 and 3 are
non-blocking follow-ups the fix can address in the same commit or defer.
