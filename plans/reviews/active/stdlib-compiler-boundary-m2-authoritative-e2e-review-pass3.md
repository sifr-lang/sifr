# M2 review pass 3 — Authoritative E2E Dependency Planning

## Verdict: **SATISFIED**

Pass 2's blocking finding (`entrypoint.rs` at 928 lines) is resolved by
extracting the resolved-interop cache into
`crates/sifr_driver/src/build/single_file_interop_cache.rs` (99 lines) and
leaving `entrypoint.rs` at 867 lines. Both pass 2 non-blocking follow-ups
(LoweringOptions-shaped cache-key concern and mutex-held-across-resolution)
are also addressed by the new module's design. The M2 architectural work
(typed compiler metadata → production `SysrootDependencyPlan` → grouped
Cargo → per-plan cache identity, with deletion of every generated-Rust
scanner) remains intact after the extraction, and the pass 2 production
build-report parity test is preserved.

## Scope reviewed

- Full working-tree diff versus HEAD (23 files, +282 / −1281):
  `git diff --stat HEAD` verified.
- New file: `crates/sifr_driver/src/build/single_file_interop_cache.rs`
  (99 lines).
- Refactored file: `crates/sifr_driver/src/build/entrypoint.rs`
  (867 lines) — imports `resolve_single_file_metadata` and
  `CompiledSingleFileMetadata` from the new module.
- Repaired dependency-planning API surface:
  `build/cargo_manifest.rs`, `build/materialize.rs`, `build/mod.rs`,
  `build/sysroot_interop_tests.rs`, `frontend/api.rs`,
  `diagnostics.rs`, `lib.rs`, `test_runner/artifacts.rs`.
- E2E harness:
  `crates/sifr/tests/e2e_support/{mod,harness_model,fixture_compilation,batch_execution,e2e_entrypoints,harness_behavior_tests,dependency_plan_authority_tests}.rs`.
- Cross-checks: `sifr_stdlib_manifest::{try_sysroot_dependency_plan,
  SysrootDependencyPlan, CargoVendorMode, SysrootCrate}`,
  `sifr_codegen::InteropBuildPlan::cache_key_fragment`,
  `crates/sifr_driver/src/stdlib/bootstrap.rs`,
  `crates/sifr_driver/src/build/rust_interop.rs`,
  `crates/sifr_driver/src/build/sysroot_interop.rs`.
- M2 acceptance criteria in
  `plans/issues/active/ad-hoc-stdlib-compiler-boundary-rearchitecture.md:282-315`.
- Inventory `plans/issues/active/ad-hoc-stdlib-compiler-boundary-m2-inventory.md`.
- Pass 1 verdict (`stdlib-compiler-boundary-m2-authoritative-e2e-review-pass1.md`),
  pass 2 verdict (`stdlib-compiler-boundary-m2-authoritative-e2e-review-pass2.md`),
  and the fixes claimed by pass 3.
- Milestone demo `demos/authoritative_dependency_plan_demo.sifr`.
- Local `python3 scripts/check_file_size_guardrails.py` and
  `scripts/check_hir_maintainability_guardrails.py`.

## Pass 2 blockers and follow-ups — final disposition

### Pass 2 Finding 1 (BLOCKER) — file-size guardrail on `entrypoint.rs` — **RESOLVED**

The cache logic that previously bloated
`compile_single_file_entrypoint_with_metadata_and_options` is extracted
into `crates/sifr_driver/src/build/single_file_interop_cache.rs`:

- `CompiledSingleFileMetadata` (the four-field return type) moves out of
  `entrypoint.rs`.
- `resolve_single_file_metadata(codegen_result, rust_interop_context,
  stdlib_interop)` becomes the single seam
  `compile_single_file_entrypoint_with_metadata_and_options` invokes
  after codegen (`entrypoint.rs:154-168`).
- The static `RESOLVED_STDLIB_INTEROP`, cache-key derivation, and cache
  eligibility gate all live in the new module.

Local `python3 scripts/check_file_size_guardrails.py` reports:

```
file-size guardrails: PASS (2485 files, limit 900 lines)
```

The largest touched hand-maintained files fit under the cap:
`harness_behavior_tests.rs` 767, `batch_execution.rs` 731,
`harness_model.rs` 722, `e2e_entrypoints.rs` 700, `materialize.rs` 658,
`fixture_compilation.rs` 619, `cargo_manifest.rs` 490,
`dependency_plan_authority_tests.rs` 282, `entrypoint.rs` 867,
`single_file_interop_cache.rs` 99. The extraction is
responsibility-scoped (sysroot-identity keyed cache of resolved
interop), which matches the AGENTS.md remediation directive rather
than mechanical splitting.

### Pass 2 Finding 2 — `LoweringOptions` in cache key — **RESOLVED (by eligibility gate)**

Cache participation now requires `codegen_result.interop ==
InteropBuildPlan::default()`
(`single_file_interop_cache.rs:35-39`). Any user `@rust` bridge — which
is what would carry a `LoweringOptions`-shaped interop signal into the
plan — populates `codegen_result.interop.rust.declarations` in
codegen and bypasses the cache, falling through to a fresh
`resolve_interop`. Fixtures that satisfy the eligibility gate have
empty codegen-side interop and their resolved plan depends only on
the sysroot's `StdlibRustInterop` (produced by the process-wide
`STDLIB_COMPILED_CACHE` in `stdlib/bootstrap.rs`), so `LoweringOptions`
cannot change the result. This is now enforced by construction rather
than by the caller documenting it.

### Pass 2 Finding 3 — mutex held across resolution — **RESOLVED**

`resolve_cached_stdlib_interop` (`single_file_interop_cache.rs:50-69`)
now uses a two-level structure:

```rust
let cell = {
    let cache = RESOLVED_STDLIB_INTEROP.get_or_init(|| Mutex::new(HashMap::new()));
    let mut entries = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Arc::clone(entries.entry(key).or_insert_with(|| Arc::new(OnceLock::new())))
};
cell.get_or_init(|| resolve_interop(codegen_result, rust_interop_context, stdlib_interop))
    .clone()
```

The global `Mutex<HashMap<String, Arc<OnceLock<Result<...>>>>>` is held
only long enough to look up or insert the `Arc<OnceLock<...>>`. Actual
resolution runs outside the lock via `OnceLock::get_or_init`.
Concurrent callers with distinct keys never serialize; concurrent
callers with the same key wait on the per-key `OnceLock`, which is the
correct behavior (one resolution shared by all waiters). Poison is
still handled via `unwrap_or_else(std::sync::PoisonError::into_inner)`.
No new panic surface.

## Acceptance-criteria audit

- **Missing metadata is a real Rust build failure, not a scanner repair.**
  `dependency_plan_authority_tests.rs:222-236` passes empty typed
  metadata plus `InteropBuildPlan::default()` and asserts the Cargo
  build fails with `Rust compilation failed` and `num_bigint`.
- **Production `SysrootDependencyPlan` with resolved private-stdlib
  interop is authoritative.**
  `cargo_manifest.rs:26-55` centralises
  `try_generate_standalone_dependency_plan` →
  `try_generate_sysroot_dependency_plan` →
  `add_sysroot_interop_crates`. The interop-carrying variant is what
  `materialize.rs:172-178,412-421` uses for production builds and cache
  keys. E2E callers pass the resolved `InteropBuildPlan` returned by
  `compile_source_with_metadata`
  (`fixture_compilation.rs:34-57, 209-224`,
  `batch_execution.rs:333-350`,
  `e2e_entrypoints.rs:52-78, 385-403`).
- **Real production build-report parity is asserted per fixture.**
  `dependency_plan_authority_tests.rs:173-220`
  (`compiled_fixture_plan_matches_production_build_report`) still
  compiles `runtime_diagnostics_tracing.sifr` and invokes the real
  `sifr_driver::build_single_file_report(...)`, asserting equality of
  `dependency_input_fingerprint()`, `cache_fingerprint`, `sysroot_root`,
  `toolchain_id`, and `sysroot_content_sha256`. The extraction leaves
  this test's call sites (`sifr_driver::build_single_file_report`,
  `compile_fixture`) untouched, so the parity assertion still applies
  after the refactor.
- **Grouping, cache, and vendor mode key on the resolved plan.**
  `harness_model.rs:73-77, 388-399` —
  `DependencyFingerprint = { dependency_inputs, resolved_plan }` from
  `plan.dependency_input_fingerprint()` and `plan.cache_fingerprint`.
  `fixture_compilation.rs:250-270` — `build_group_sources` refuses to
  merge cases whose `dependency_plan` values differ
  (`"non-identical production dependency plans"`).
  `fixture_compilation.rs:372-417` — `plan_batches` buckets by
  `DependencyFingerprint`. `E2E_CACHE_SCHEMA_VERSION = 2`
  (`harness_model.rs:7`) and `cache_key_for_group`
  (`fixture_compilation.rs:556-576`) mix schema + full fingerprint
  signature. `batch_execution.rs:69-72, 371` applies
  `sifr_driver::sysroot_cargo_config_args(&plan)` before `build`,
  which is the position cargo requires.
  `dependency_plan_authority_tests.rs:261-282` asserts that changing
  the required features changes both `dependency_fingerprint()` and
  `cache_key_for_group(...)`.
- **BigInt inventory asserts feature membership.**
  `dependency_plan_authority_tests.rs:96-131` iterates
  `FORMER_MODULE_RULES`, calls `features_for_stdlib_module(module)`
  against the row, and for `_bigint` iterates
  `expected_features` and asserts
  `plan.required_features.contains(feature)` for each.
- **`required_features` naming is consistent.**
  `e2e_entrypoints.rs:52,77,385,402` destructures `required_features`
  from `compile_source_with_metadata`. `grep -rn "required_crates"
  crates/` returns no source hits.
- **Duplicate E2E stdlib/runtime maps and inference rules deleted.**
  Verified via `git diff --stat`: seven files removed (~1,000 lines).
  `grep -R 'infer_dependencies\|normalize_dependency_set' crates
  scripts verification` returns zero hits. The remaining E2E-side type
  surface is `pub(crate) use sifr_stdlib_manifest::{StdlibFeature,
  SysrootDependencyPlan};` (`mod.rs:1`).
- **Inventory covers every former rule.**
  `ad-hoc-stdlib-compiler-boundary-m2-inventory.md` lists 6 module
  rules, 1 runtime rule, and 29 direct-crate rules. Each row cites a
  typed production authority (`features_for_stdlib_module`,
  `feature_for_codegen_requirement`, `planned_sifr_stdlib_features`,
  structural codegen emission, or documented obsolescence).
  `dependency_plan_authority_tests.rs:4-64` mirrors the tables and
  iterates them with `feature_for_codegen_requirement` and
  `plan.required_features.contains(...)` assertions.
- **Public API design.**
  `crates/sifr_driver/src/lib.rs:22-25,35` exports
  `generate_dependency_cargo_toml`,
  `try_generate_standalone_dependency_plan`,
  `sysroot_cargo_config_args`, and
  `sifr_codegen::InteropBuildPlan`. The interop-carrying variants
  remain `pub(crate)` (`build/mod.rs:44-49`). Every E2E call uses only
  the public surface.
- **`CompileResultFull::Success` carries the resolved interop.**
  `diagnostics.rs:18-30` adds `interop: sifr_codegen::InteropBuildPlan`.
  `frontend/api.rs:48-62` pipes `codegen_result.interop` through, and
  `codegen_result` is now the extracted `CompiledSingleFileMetadata`
  whose `interop` field is populated by `resolve_single_file_metadata`.
- **Test-runner Cargo generation resolves.**
  `test_runner/artifacts.rs:34-69` renames
  `generate_dependency_cargo_toml_for_cache_key` to
  `generate_dependency_cargo_toml_with_interop` and preserves
  `CargoVendorMode::SysrootOnly`.
- **Safety, panics, unsafe.** No new `unsafe`, `.unwrap()`, or `.expect()`
  in generated-project runtime paths. `expect` calls in the new tests
  are inside `#[test]` fns (test-code invariants), which is acceptable.

## Correctness of the extracted cache (independent audit)

### Source and metadata preservation across cache hits/misses

`resolve_single_file_metadata` extracts `lowering_stats`, `rust_source`,
`used_stdlib_modules`, and `required_features` from `codegen_result`
BEFORE the cache branch (`single_file_interop_cache.rs:30-33`). Only the
`interop` field is derived through the cache. On a cache hit, the
returned `CompiledSingleFileMetadata` combines the current call's
source/metadata with the cached resolved interop; on a miss, the
freshly-resolved interop is stored and returned identically. This is
the correct semantics: source-dependent fields must reflect the caller's
input; sysroot-dependent fields can be shared. Verified by reading the
function.

### Cache eligibility bounds

The cache is entered only when `codegen_result.interop ==
InteropBuildPlan::default()`
(`single_file_interop_cache.rs:35-39`). `codegen_result.interop` is
populated by `sifr_codegen::generate_rust_with_stdlib` from
`RustInteropDeclaration`s attached to `HirFunction`/`HirClass`. Any
user `@rust` bridge therefore forces the else branch and runs a fresh
`resolve_interop`. The eligibility gate is stronger than
"pure-Sifr fixture": it excludes any codegen output that carries user
interop declarations, which is what the LoweringOptions concern would
require to matter.

### Cache key completeness

`stdlib_interop_cache_key` (`single_file_interop_cache.rs:83-99`)
composes:

- `root=<sysroot.root>` (path, differs per install)
- `toolchain=<sysroot.toolchain_id()>`
- `content=<sysroot.manifest.sysroot_content_sha256>` (SHA-256 of
  sysroot content)
- `<stdlib_interop.plan.cache_key_fragment()>` (full plan digest:
  declarations, resolved targets, generated bridge modules,
  bridge_contracts, trust requirements, probes, bridge_sources,
  cargo_inputs — see `rust_interop_plan.rs:15-70`)

The `<no-sysroot>` fallback for the `stdlib_interop.sysroot.is_none()`
case would collapse all sysroot-less states onto one key, but that
branch is only reachable when the stdlib has no interop declarations to
resolve — in which case `attach_stdlib_rust_interop` short-circuits
before `apply_package_rust_interop_metadata`
(`sysroot_interop.rs:33-35`), producing the default plan. Collisions
under that branch are therefore semantically vacuous.

Collision between two live sysroots requires identical root path,
toolchain id, content SHA-256, and identical pre-resolution plan digest
simultaneously — cryptographically improbable for distinct sysroots.

### Concurrency

The two-level `Mutex<HashMap<..., Arc<OnceLock<...>>>>` pattern serializes
only the map lookup/insert, not resolution. `OnceLock::get_or_init`
inside a per-key `Arc` ensures a single resolution per key while
letting concurrent distinct-key resolutions proceed in parallel. This
is the standard pattern for deduplicated lazy initialization and matches
what pass 2 asked for. Poison recovery uses
`unwrap_or_else(std::sync::PoisonError::into_inner)` (no panic).

### Error caching

If `resolve_interop` returns `Err(Vec<RenderedDiagnostic>)`, the error
is stored in the `OnceLock` and cloned to subsequent callers with the
same key (`ResolvedInterop = Result<InteropBuildPlan,
Vec<RenderedDiagnostic>>`, `single_file_interop_cache.rs:19-20`). This
is deterministic: same sysroot content → same errors. Since the E2E
binary and `sifr` CLI are short-lived and resolve one sysroot per run,
a cached failure surfaces the same diagnostic every call rather than
flapping. For a long-lived daemon that hot-swaps sysroots, cache
growth is unbounded and cached errors would persist through a sysroot
being fixed under the same identity — but that is not a target
consumer today, and the earlier
`STDLIB_COMPILED_CACHE`/`get_or_init_stdlib_cache` in
`stdlib/bootstrap.rs` already exhibits the same lifetime shape.

### Production build parity

The E2E harness uses `try_generate_standalone_dependency_plan(modules,
features, interop)` and `generate_dependency_cargo_toml(project_name,
&plan)`. The production `sifr build` / `sifr run` paths use
`materialize_binary_project_with_report` →
`try_generate_sysroot_dependency_plan(..., SysrootOnly)` →
`generate_dependency_cargo_toml_with_interop(project_name, &plan,
&generated_project.interop)`
(`materialize.rs:150-198`). Two structural gaps between the paths:

1. `generate_dependency_cargo_toml` renders with
   `InteropBuildPlan::default()`, so
   `rust_interop_path_dependencies(interop)` returns an empty map.
   The production `_with_interop` variant includes direct/package
   bridge path dependencies. Sysroot interop crates are added inside
   `add_sysroot_interop_crates` in both paths, so they appear in both
   Cargo files. Divergence is limited to `DirectCargoDependency` /
   `PackageBridge` interop, i.e., user `@rust` fixtures. `grep -r
   '@rust(' crates/sifr/tests/e2e/pass/` continues to return zero
   hits, so the current corpus does not exercise the gap.
2. The E2E harness uses `SysrootOnly` universally; the production path
   uses `SysrootOnly` for single-file/project builds and
   `PackageOwned` for package-project builds
   (`entrypoint.rs:334-341`). Pass fixtures are single-file; package
   projects are covered by unit tests
   (`sysroot_interop_dependency_plan_keeps_sysroot_vendor_mode`).

Neither gap affects M2's acceptance: fixture-level parity is asserted
against the real build-report fingerprint, and the pass corpus is
demonstrably pure-Sifr. Pass 1's transitive proof (648/648 fixtures
build under harness Cargo) plus the pass 2 fingerprint parity check
remain sufficient for the current corpus. Recording these gaps for
future scope: when fixtures start using `@rust(package=...)`, the
harness Cargo path will need to route through the interop-carrying
renderer, and the parity test will need to exercise a fixture that
carries a non-empty interop plan.

### Plan identity / Cargo / vendor behavior

- Grouped Cargo is generated exclusively from the production plan via
  `sifr_driver::generate_dependency_cargo_toml(&group.package_name,
  &group.dependency_plan)` (`batch_execution.rs:55-58`). No
  module/crate string switches remain (deleted
  `fixture_cargo_toml.rs`, 355 lines).
- `sysroot_cargo_config_args(&plan)` returns args only for
  `SysrootOnly` mode
  (`cargo_manifest.rs:57-70`); `PackageOwned` returns empty. The E2E
  harness always applies `SysrootOnly` (its plans are built with
  `try_generate_standalone_dependency_plan`).
- `add_sysroot_interop_crates` extends `dependency_plan.cache_fingerprint`
  when it inserts sysroot crates
  (`cargo_manifest.rs:220-244`), so plans that pick up different
  interop crate sets have distinct fingerprints and cannot alias in
  the harness cache.

## Prior findings status

- **Pass 1 Finding 1** (tautological parity check) — RESOLVED by
  pass 2's `compiled_fixture_plan_matches_production_build_report`,
  preserved after extraction.
- **Pass 1 Finding 2** (`required_crates` leftover naming) — RESOLVED.
- **Pass 1 Finding 3** (`_bigint` count-only assertion) — RESOLVED;
  membership check present.
- **Pass 2 Finding 1** (file-size guardrail) — RESOLVED (see above).
- **Pass 2 Finding 2** (LoweringOptions cache-key concern) — RESOLVED
  by eligibility gate.
- **Pass 2 Finding 3** (mutex-held-across-resolution) — RESOLVED by
  per-key `OnceLock`.

## Validation

- `python3 scripts/check_file_size_guardrails.py` — PASS
  (2485 files, limit 900).
- `python3 scripts/check_hir_maintainability_guardrails.py` — PASS.
- Task-reported (accepted as evidence):
  - `dependency_plan_authority_tests` authority suite: 7/7 after
    extraction.
  - Compile-metadata regression: PASS.
  - Cache-disabled E2E pass suite after interop repair: 648/648 with
    143 groups and 0 hits.
  - Non-pass suite: 36/36.
  - `sifr_driver` library nonignored tests: 325/325.
  - `demos/authoritative_dependency_plan_demo.sifr`: builds and runs
    under plan-derived Cargo.toml.

## Non-blocking observations for a future milestone

1. Harness Cargo generation goes through
   `generate_dependency_cargo_toml` with
   `InteropBuildPlan::default()`, so direct/package-bridge interop
   path dependencies would be missing if a pass fixture ever declared
   a user `@rust(package=...)` bridge. Not exercised today; if it is
   ever added to the pass corpus, route the harness through the
   interop-carrying renderer (or accept that only sysroot-owned
   interop is supported for grouped E2E). Non-blocking.
2. `RESOLVED_STDLIB_INTEROP` is unbounded process-wide. Fine for
   `sifr` CLI and the E2E binary; a long-lived daemon that swaps
   sysroots would want a bounded map. Non-blocking.
3. The pass 2 production build-report parity test still exercises
   exactly one fixture. The 648/648 cache-disabled pass suite is the
   transitive proof for the corpus. Broadening the fingerprint parity
   test to a small representative panel (e.g., a `_sifr.fs`, a
   `_sifr.net`, and a decimal-heavy fixture) would harden the
   invariant. Non-blocking.

## Final verdict

**SATISFIED.**
