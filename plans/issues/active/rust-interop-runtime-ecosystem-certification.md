# Rust Interop Runtime and Ecosystem Certification Follow-Up

## Status

Active Track A follow-up created by Phase 39 closeout. Certifications 1
through 10 are merged; `certification_11` is in progress and later items remain
strictly sequential.

The verification-hardening dependency is complete through
[`hardening_4`](../archive/rust-interop-verification-matrix-hardening.md#hardening_4-replace-lexical-rejection-context):
[PRs #3018](https://github.com/sifr-lang/sifr/pull/3018),
[#3019](https://github.com/sifr-lang/sifr/pull/3019),
[#3020](https://github.com/sifr-lang/sifr/pull/3020),
[#3022](https://github.com/sifr-lang/sifr/pull/3022), and
[#3023](https://github.com/sifr-lang/sifr/pull/3023) are merged. The
`certification_0` prerequisite and certifications 1 through 10 are also merged.

This issue has two tracks:

- Track A certifies every currently deferred runtime/ecosystem surface needed
  for honest stable-channel claims.
- Track B is a dormant downstream certification item for the general external
  package-resource substrate that Native Pydantic-Sifr `milestone_ps_2` will
  release. Track B is not a Phase 40 blocker while that surface does not exist
  and is not advertised.

Each item below is one PR. An item starts only after its listed dependencies
have merged. Every PR follows the repository workflow: define its checklist,
implement and validate locally, open the PR, review to satisfaction, merge,
then update this issue and durable documentation before starting the next
item. Do not combine independent items to shorten the sequence.

## Objective

Resolve all eleven current `future-owned-by-separate-phase` compatibility rows
and the two unmodeled runtime deferrals identified below into either:

- passing `supported` or `supported-through-bridge` evidence for the row's
  stated `execution_kind`; or
- an explicit `unsupported-by-design` contract backed by passing positive and
  negative compiler-diagnostic evidence.

No row may be promoted by editing status strings or README prose alone.

`unsupported-by-design` is not an evidence waiver. Choosing it requires an
architecture decision, a compiler-diagnostic capability/fixture shape, both
evidence directions passing, public rejection documentation, and no implicit
fallback.

## Entry Criteria

Track A implementation starts only after
[`rust-interop-verification-matrix-hardening.md`](../archive/rust-interop-verification-matrix-hardening.md)
items `hardening_1` through `hardening_4` have merged. Those items make the
Rust-interop area authoritative, define tier/execution-kind rules, bind
support claims to executable local-lane evidence, and replace ambiguous
stale-syntax exemptions. This dependency is satisfied; the current baseline
is 34 fixture rows, 34 compatibility rows, 34 schema-v2 fixture manifests, 47
passing evidence directions, and 21 planned directions.

Before the first Track A row PR:

- add distinct `zero_copy_runtime_matrix` and
  `advanced_data_runtime_matrix` rows as described in
  `certification_0`; the existing contract-only rows remain supported only for
  their stated compile-time contracts;
- record the current compatibility and fixture counts in this issue;
- make the Phase 40 stable-claim gate consume structured compatibility data,
  not prose; and
- confirm every required Cargo crate/version is present in the checked-in
  lockfile and cacheable by the repository's locked/offline setup.

## Canonical Evidence and Promotion Rules

Every row PR must update, in one change:

- `verification/areas/rust_interop/data/rust_interop_fixture_matrix.json`;
- `verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`;
- `verification/areas/rust_interop/data/rust_interop_tiers.toml` when a row is
  added or re-tiered;
- `verification/areas/rust_interop/fixtures/<row>/fixture.json`, positive and
  negative `.sifr` sources, package/scenario examples, and README;
- the focused Rust tests named by both evidence provenance records;
- `verification/areas/rust_interop/checks/check_fixture_matrix.py` inventories
  and self-tests when rows or crates change;
- `internal_docs/rust_interop_architecture.md` and the public Rust-interop
  compatibility docs; and
- this issue's status table, with the merged PR link and review artifact.

For `supported` or `supported-through-bridge`:

1. Both matrix evidence statuses are `passing`.
2. Both fixture evidence records contain the structured executable provenance
   required by the hardening issue.
3. The named test is executed by the declared mandatory local profile's
   `crate_tests` step.
4. `cargo-probe` evidence builds generated/package code with the pinned crate
   and observes the declared build or value result.
5. `runtime-observed` evidence executes generated compiler/runtime code and
   observes values, lifecycle, cleanup, cancellation, or failure behavior; a
   source-shape or emitted-token assertion is insufficient.
6. Positive and negative behavior is hermetic: temporary directories,
   loopback-only services, explicit subprocess cleanup, no external network,
   no developer-machine service dependency, and deterministic timeouts.

For `unsupported-by-design`, the same PR must:

1. merge the rejection decision into the architecture;
2. change the row capability and `execution_kind` to the actual
   compiler-diagnostic contract;
3. replace any runtime/build placeholder with concrete accepted-rejection and
   cannot-bypass diagnostic fixtures;
4. set both evidence statuses to `passing`;
5. document the unsupported surface publicly; and
6. prove no fallback, hidden adapter, build-script execution, network access,
   or panic path is introduced.

If a row cannot meet either rule, it stays
`future-owned-by-separate-phase`; time pressure is not a promotion criterion.

## Track A Row Contract

The table freezes the PR boundary, exact evidence IDs, expected promotion, and
special acceptance behavior. Existing feature pins in the fixture matrix are
normative and must not be broadened.

| Item | Row / tier / execution | Positive evidence | Negative evidence | Expected result and special gate |
| --- | --- | --- | --- | --- |
| `certification_1` | `bridge_type_matrix` / 1 / `cargo-probe` | `supported_type_roundtrips` | existing `unsupported_container_rejections` | `supported-through-bridge`; roundtrip each `serde`, `serde_json`, `thiserror`, `bytes`, and `indexmap` value through generated package glue, including nested/error values |
| `certification_2` | `panic_boundary_wrapper_emission` / 2 / `runtime-observed` | `generated_wrapper_maps_panic_to_declared_error` | `invalid_map_error_signature_rejected` | `supported`; execute generated catch/map glue, mapper-signature rejection, mapper panic fallback to redacted original `RustPanicError`, and unrepresentable-fallback rejection |
| `certification_3` | `callbacks_call_scoped` / 2 / `runtime-observed` | `callback_valid_during_call` | `callback_storage_rejected` | `supported-through-bridge`; execute invocation, cleanup, panic mapping, and storage/escape rejection |
| `certification_4` | `async_runtime_reqwest` / 2 / `runtime-observed` | `async_reqwest_loopback` | `hidden_block_on_rejected` | `supported-through-bridge`; use an in-process HTTP loopback server, prove borrowed-input ownership, cancellation/drop, runtime reuse, timeout cleanup, and no nested/blocking runtime |
| `certification_5` | `opaque_resource_matrix` / 2 / `runtime-observed` | `resource_close_aclose_matrix` | `invalid_resource_aliasing` | `supported-through-bridge`; cover `reqwest` with HTTP loopback, `rusqlite` with a temporary database, and `redis`/`tokio-postgres` with deterministic loopback protocol harnesses; prove close/aclose, double-close stability, alias/use-after-close rejection, poison redaction, and subprocess/task cleanup |
| `certification_6` | `callback_subscription_ecosystem` / 2 / `runtime-observed` | `subscription_cancel_shutdown` | `invalid_thread_capture_rejected` | `supported-through-bridge`; use loopback WebSocket/RESP and a temporary watched directory for `tokio-tungstenite`, Redis pub/sub, and `notify`; prove backpressure/overflow policy, cancellation, close, shutdown, thread capture, and callback panic mapping |
| `certification_7` | `zero_copy_runtime_matrix` / 2 / `runtime-observed` | `crate_backed_view_lifecycle` | `borrow_escape_and_invalid_mutability_rejected` | new `supported-through-bridge` row for `bytes`, `memmap2`, `bytemuck`, and `zerocopy`; observe alias-preserving views, owner lifetime, mutation exclusivity, Send/Sync obligations, release, and async-suspension rejection |
| `certification_8` | `advanced_data_runtime_matrix` / 4 / `runtime-observed` | `crate_backed_arrow_tensor_roundtrips` | `schema_shape_device_mismatch_rejected` | new `supported-through-bridge` row for `arrow`, `datafusion`, `polars`, `ndarray`, and CPU-only `candle`; observe schema/dtype/rank/shape/layout/stride identity, DLPack transfer, owner cleanup, mismatch rejection, and no implicit copy |
| `certification_9` | `native_build_script` / 3 / `cargo-probe` | `trusted_build_script_native_evidence` | `untrusted_native_link_rejected` | `supported`; build `cc`, `bindgen`, `cxx`, and `zstd` in a temp package, verify declared native trust and deterministic artifacts, and reject undeclared native/build-script execution before it runs |
| `certification_10` | `proc_macro_trust` / 3 / `cargo-probe` | `trusted_proc_macro` | `untrusted_proc_macro_rejected_pre_execution` | `supported`; build `serde_derive` and deterministic `prost-build` output, verify trust/cache identity, and reject undeclared execution before it runs |
| `certification_11` | `cargo_locked_offline` / 3 / `cargo-probe` | `locked_offline_cache_hit` | `lockfile_feature_drift_rejected` | `supported`; execute Sifr check/build/run with `--locked`, `--offline`, and `--frozen`, prove a warm cache hit, and reject lockfile, checksum, feature, and frozen-mode drift without network |
| `certification_12` | `ecosystem_cli_certification` / 4 / `cargo-probe` | `cli_tooling_probe_coverage` | `unsupported_anyhow_surface` | `supported-through-bridge`; build and run `clap`, `tracing`, `tracing-subscriber[env-filter]`, and bridge-safe `anyhow` adapters; reject direct unsupported `anyhow` crossings |
| `certification_13` | `ecosystem_backend_certification` / 4 / `cargo-probe` | `backend_probe_coverage` | `sqlx_without_offline_artifacts` | `supported-through-bridge`; build and execute hermetic `axum`/`tower-http` loopback plus `sqlx` offline metadata paths; reject missing/stale SQLx offline artifacts before network |

## Implementation Progress

| Item | Status | Evidence |
| --- | --- | --- |
| `certification_0` | merged | [PR #3026](https://github.com/sifr-lang/sifr/pull/3026) |
| `certification_1` | merged | [PR #3027](https://github.com/sifr-lang/sifr/pull/3027); executable recursive bridge-type roundtrips |
| `certification_2` | merged | [PR #3031](https://github.com/sifr-lang/sifr/pull/3031); generated panic wrapper emission and mapper fallback |
| `certification_3` | merged | [PR #3033](https://github.com/sifr-lang/sifr/pull/3033); generated call-scoped callback invocation and lifetime rejection |
| `certification_4` | merged | [PR #3036](https://github.com/sifr-lang/sifr/pull/3036); async reqwest loopback, runtime reuse, cancellation/drop, timeout cleanup, and hidden blocking rejection |
| `certification_5` | merged | [PR #3042](https://github.com/sifr-lang/sifr/pull/3042); opaque resource lifecycle matrix with HTTP/Redis/PostgreSQL loopbacks and a temporary SQLite database |
| `certification_6` | merged | [PR #3046](https://github.com/sifr-lang/sifr/pull/3046); retained callback subscription lifecycle and capture contract |
| `certification_7` | merged; retrospective performance rerun pending | [PR #3053](https://github.com/sifr-lang/sifr/pull/3053); crate-backed zero-copy lifecycle and compiler rejection contract |
| `certification_8` | merged | [PR #3067](https://github.com/sifr-lang/sifr/pull/3067); crate-backed Arrow/tensor generated package and compiler mismatch rejection |
| `certification_9` | merged | [PR #3069](https://github.com/sifr-lang/sifr/pull/3069); exact-pinned native build-script package, deterministic artifacts, and fail-closed direct/transitive trust rejection |
| `certification_10` | merged | [PR #3071](https://github.com/sifr-lang/sifr/pull/3071); exact-pinned proc-macro/codegen package, deterministic prost output, and package-wide pre-execution trust rejection |
| `certification_11` | merged | [PR #3075](https://github.com/sifr-lang/sifr/pull/3075); locked/offline/frozen Sifr command propagation, cache reuse, and deterministic drift rejection |
| `certification_12` | merged | [PR #3076](https://github.com/sifr-lang/sifr/pull/3076); exact-pinned CLI/tooling execution and bridge-safe `anyhow` boundary certification |
| `certification_13` | in progress | exact-pinned backend loopback execution and fail-closed SQLx offline metadata certification |
| `certification_14` | blocked | starts after `certification_13` merges |
| `certification_pkg_resource_core` | dormant | starts only after Native Pydantic-Sifr `milestone_ps_2` releases bridge version 2 |

## Ordered Track A Items

### certification_0: Model Runtime Deferrals and Stable Claims

This PR lands after hardening items `hardening_1` through `hardening_4` and
before any row implementation.

Implementation checklist:

- [x] Add the two future-owned runtime rows to both matrices, tier metadata,
  fixture manifests/sources/examples, validator inventories, and docs without
  weakening the five existing contract-only claims.
- [x] Add structured stable claims and a validator whose self-tests reject
  unknown/future-owned claims, execution-scope drift, public-doc omissions,
  and contract-only runtime overclaims.
- [x] Pin every declared ecosystem crate and frozen feature policy in the root
  workspace lock, prove the inventory resolves with locked/offline Cargo, and
  add mutation coverage for lock/inventory drift.
- [x] Recompute and record the post-item row, manifest, evidence, category,
  execution-kind, and locked-crate counts.
- [x] Run the focused provenance/checker gates, create-PR profile, full merge
  profile, Clippy, rustfmt, maintainability, file-size, and diff-hygiene gates.
- [x] Run Opus review rounds to satisfaction, merge the PR, and unblock only
  `certification_1`.

Post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 47 passing and 25 planned evidence directions;
- categories: 17 `supported`, 5 `supported-through-bridge`, 1
  `unsupported-by-design`, and 13 `future-owned-by-separate-phase`;
- execution kinds: 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required crate aliases, each exact-pinned at the catalog dependency
  boundary and present in the checked-in root lockfile; transitive family
  crates remain selected by that exact locked graph;
- 23 structured stable claims; and
- 5 registered area suites with 10 cases, including `stable-candidate` in
  create-PR, merge, nightly, and release.

Validation evidence on 2026-07-26 and 2026-07-27:

- the final create-PR profile passed all 23 steps; its complete Rust-interop
  step measured 4,732 ms after one 578 ms locked-cache setup;
- the merge profile passed all 23 lane steps, including all 10 Rust-interop
  variants, performance, full E2E, sanitizers, release-smoke/equivalence,
  diagnostics, and ecosystem hardening; its Rust-interop step measured 4,394
  ms after one 658 ms locked-cache setup;
- exact-state nightly and release runs passed their complete Rust-interop and
  stable-candidate coverage in 4,161 ms and 3,880 ms respectively; and
- both extended profiles later exposed the same 20 pre-existing algorithmic
  full-corpus failures among 412 variants. All failures are algorithmic-corpus
  cases, no algorithmic fixture or compiler behavior is changed here, and the
  owning issue lands separately; they do not block this item or Phase 40.

The catalog's reachable graph necessarily advances ten versions already used
elsewhere in the workspace: `hashbrown` 0.17.0 to 0.17.1, `rand` 0.10.1 to
0.10.2, `js-sys` 0.3.95 to 0.3.103, `wasm-bindgen` 0.2.118 to 0.2.126 and its
three macro/support crates, plus `futures-core`, `futures-channel`, and
`futures-sink` 0.3.32 to 0.3.33. Each advance is forced by an exact catalog
dependency family; the minimal lock update removes no existing package and
adds no package unreachable from `sifr_rust_interop_catalog`.

- Phase 40 `milestone_40_0` consumed the hardening work while this item was in
  flight. `milestone_40_1` consumes this item's registered stable-candidate
  claim check before qualification.
- Add `zero_copy_runtime_matrix` and `advanced_data_runtime_matrix` to both
  matrices, tier metadata, fixture directories, validator inventories, and
  architecture/public docs with both evidence directions `planned`.
- Keep `zero_copy_bytes`, `zero_copy_view_matrix`, `arrow_record_batch`,
  `tensor_dlpack_bridge`, and `advanced_data_matrix` supported only for their
  existing `contract-only` execution kind. Their notes and public claims must
  say `contract-only`; they are not runtime evidence.
- Add `verification/areas/rust_interop/data/stable_support_claims.json` and a
  Rust-interop area check that validates every advertised stable claim names a
  compatibility row, uses the same execution scope, and is never
  `future-owned-by-separate-phase`.
- Treat `stable_support_claims.json` as the compatibility-derived input that
  Phase 40 digests into its canonical `stable-release-plan.json`; it is not a
  second release-plan authority. The governed Phase 40 release plan remains
  authoritative for a concrete stable candidate.
- This item registers `stable-candidate` in create-PR, merge, nightly, and
  release together. Phase 40 confirms and consumes that registered suite.
  Rust-interop profile validation requires every registered area suite in
  every authoritative profile; a release-only selection would make all four
  profiles invalid.
- Add a stable-candidate mode that fails when public stable docs advertise a
  row absent from the claims file or advertise runtime support through a
  contract-only row.
- Confirm Phase 40 `milestone_40_1` preserves the stable-candidate registration
  in all four authoritative profiles and consumes its result during
  qualification, and the `milestone_40_4` documentation gate executes it. Do
  not make all development builds fail merely because honest future-owned rows
  remain unadvertised.

Exit gate:

- the two deferrals are mechanically visible as future-owned runtime rows;
- stable claims preserve the narrower contract-only rows without overclaim;
- the normal Rust-interop area and a deliberately invalid stable-claims
  self-test pass/fail as expected; and
- all four local profiles report the Rust-interop checks step as executed.

### certification_1 through certification_3: Bridge and Pydantic Blockers

Implement `certification_1`, `certification_2`, then `certification_3` as
separate PRs. `certification_2` and `certification_3` are prerequisites for
Native Pydantic-Sifr `milestone_ps_3`; their merged status must be recorded in
both active issues.

#### certification_1: Bridge Type Roundtrips

Implementation checklist:

- [x] Add a locked, offline package scenario using exact-pinned `serde`,
  `serde_json`, `thiserror`, `bytes`, and `indexmap`.
- [x] Build and execute generated Sifr package glue over nested values, the
  mapped error path, bytes, and nested dictionaries without claiming key
  iteration order.
- [x] Fix generated recursive `list`, `dict`, exact-`int`, and `Option`
  lowering in both directions, including escaped user identifiers, so runtime
  values match the probed bridge types without raw Rust build failures.
- [x] Bind positive evidence to the mandatory ignored generated-build suite and
  retain the existing unsupported-container diagnostic evidence.
- [x] Promote only `bridge_type_matrix` in both matrices, structured stable
  claims, public docs, architecture docs, fixture provenance, and counts.
- [x] Run focused and authoritative local gates, Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_2`.

Post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 48 passing and 24 planned evidence directions;
- categories: 17 `supported`, 6 `supported-through-bridge`, 1
  `unsupported-by-design`, and 12 `future-owned-by-separate-phase`;
- execution kinds: 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
  and
- 24 structured stable claims.

Review and gate evidence:

- Opus rounds 1 through 3 requested corrections for ordering claims, lock
  hermeticity, recursive composite conversion, inventory counts, and escaped
  user identifiers; [round 4](../../reviews/active/rust-interop-certification-1-review-round4.md)
  and the final PR-level
  [round 5](../../reviews/active/rust-interop-certification-1-review-round5.md)
  are `SATISFIED`.
- The authoritative `create-pr` profile passed on the warm rerun, including
  Rust interop `10/10` and E2E `131/131`; the first attempt was functionally
  green but exceeded the Python-interop step budget.
- The authoritative `merge` profile passed all 24 lane steps, including the
  unchanged performance budgets, Rust interop `10/10`, generated-build
  evidence, E2E, and 261 hardening variants with zero failures; only the two
  governed ASan capability skips remained.
- Final exact-head
  [round 6](../../reviews/active/rust-interop-certification-1-review-round6.md)
  is `SATISFIED`.
- [PR #3027](https://github.com/sifr-lang/sifr/pull/3027) merged as
  `53fa84b708`; its final full merge profile passed every lane step, including
  Rust interop `10/10`, the mandatory 41-test generated-build suite, E2E
  `674/674`, and 261 hardening variants.

#### certification_2: Panic Boundary Wrapper Emission

Implementation checklist:

- [x] Extend Result bridge contracts so `E | RustPanicError` keeps `E` as the
  Rust target error representation while reserving `RustPanicError` for
  generated wrapper failures.
- [x] Emit sync generated wrappers that catch Rust target panics, redact panic
  payloads, map ordinary bridge errors, and place the original panic in the
  declared Sifr error union.
- [x] Implement `panic=map_error(path)` through a second protected mapper call;
  mapper panics must fall back to the original redacted `RustPanicError`.
- [x] Reject mapper signatures that are not
  `fn(RustPanicErrorBridge) -> E` with `E: Display`, and reject declarations
  whose error channel cannot represent the mapper-panic fallback.
- [x] Add a locked/offline runtime package scenario plus direct negative
  evidence for invalid mapper signatures and unrepresentable fallback
  declarations.
- [x] Bind both evidence directions to mandatory merge-lane tests, promote only
  `panic_boundary_wrapper_emission`, and update structured claims, public and
  architecture docs, provenance, and inventory counts.
- [x] Run focused and authoritative local gates, Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_3`.

Post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 50 passing and 22 planned evidence directions;
- categories: 18 `supported`, 6 `supported-through-bridge`, 1
  `unsupported-by-design`, and 11 `future-owned-by-separate-phase`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
  and
- 25 structured stable claims.

Focused implementation evidence:

- the locked `panic_wrapper_runtime` package executes success, ordinary-error,
  mapped-panic, mapper-panic fallback, and directly declared
  `RustPanicError` paths through generated release code;
- the paired generated-build negative test rejects an invalid
  `RustPanicErrorBridge` mapper signature, while the same mandatory driver
  suite directly rejects an error channel without a representable fallback
  using `SIFR-RUST-PANIC-0001`; and
- the complete Rust-interop area passes all 10 variants, with matrix self-test
  coverage increased to 95 mutation cases.

Review and gate evidence:

- Opus review rounds
  [1](../../reviews/active/rust-interop-certification-2-review-round1.md),
  [2](../../reviews/active/rust-interop-certification-2-review-round2.md),
  [3](../../reviews/active/rust-interop-certification-2-review-round3.md),
  [4](../../reviews/active/rust-interop-certification-2-review-round4.md),
  [5](../../reviews/active/rust-interop-certification-2-review-round5.md), and
  [6](../../reviews/active/rust-interop-certification-2-review-round6.md)
  requested revisions that were closed in sequence.
- Exact working-tree
  [round 7](../../reviews/active/rust-interop-certification-2-review-round7.md)
  is `SATISFIED` with no milestone blockers.
- Exact committed PR-head
  [round 8](../../reviews/active/rust-interop-certification-2-review-round8.md)
  is `SATISFIED` with no PR blockers.
- Final merge-readiness
  [round 9](../../reviews/active/rust-interop-certification-2-review-round9.md)
  is `SATISFIED`. The merge lane passed every functional step and stopped only
  at three unchanged `check`-mode performance medians under sustained unrelated
  host load. The same five-sample runner reproduced all three misses with a
  retained compiler binary that predates both `certification_1` merge and all
  `certification_2` commits; that control was slower than the PR head on the
  arithmetic case. Opus independently verified that the fixtures contain no
  Rust interop, terminate before codegen/bridge planning, and demonstrate
  environmental timing drift rather than a PR-attributable regression.
- `scripts/run_all_tests.sh --profile create-pr` passes every blocking lane:
  Rust interop `10/10`, Python interop `19/19`, runtime platform 28 variants,
  all crate smoke suites, generated-code quality `5/5`, and create-PR E2E
  `131/131`.
- [PR #3031](https://github.com/sifr-lang/sifr/pull/3031) merged as
  `d6f41ac499`; exact-new-head Opus confirmation was `SATISFIED`.

#### certification_3: Call-Scoped Callback Runtime

Implementation checklist:

- [x] Treat a plain top-level `Callable[[...], R]` Rust interop parameter as
  call-scoped while preserving `@rust.callback(...)` as the separate
  thread-safe policy contract.
- [x] Add a borrowed `CallScopedCallbackBridge<'call, Args, Output>` that owns
  no callback and is deliberately neither `Send` nor `Sync`.
- [x] Emit generated callback adapters that convert callback arguments and
  success values, map declared callback errors through display strings, and
  execute inside the already-certified outer panic boundary.
- [x] Probe the concrete callback bridge signature and report
  `SIFR-RUST-CB-0001` when Rust attempts to store, return, or move the borrowed
  callback across a thread boundary.
- [x] Add the locked/offline `call_scoped_callback_runtime` package plus
  mandatory generated-build evidence for invocation, ordinary callback errors,
  redacted callback panic mapping, and storage/return/thread escape rejection.
- [x] Promote only `callbacks_call_scoped` in both matrices, structured stable
  claims, public and architecture docs, fixture provenance, and counts.
- [x] Run focused and authoritative local gates, Opus review rounds to
  satisfaction, merge the PR, record the Native Pydantic-Sifr prerequisite,
  and unblock only `certification_4`.

Post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 52 passing and 20 planned evidence directions;
- categories: 18 `supported`, 7 `supported-through-bridge`, 1
  `unsupported-by-design`, and 10 `future-owned-by-separate-phase`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
  and
- 26 structured stable claims.

Focused implementation evidence:

- the locked `call_scoped_callback_runtime` package executes two successful
  callback invocations, preserves an ordinary callback error, maps a callback
  panic to redacted `RustPanicError`, and emits no panic payload to stderr;
- the paired generated-build negative test installs storage, returned deferred
  invocation, and unmanaged-thread variants; each assertion is pinned to the
  concrete rustc lifetime or thread-trait failure and reports
  `SIFR-RUST-CB-0001` before the package can run.
- focused callback/codegen/driver tests, Clippy, fixture-matrix checks,
  maintainability guardrails, file-size guardrails, and generated positive and
  negative package builds pass; working-tree Opus review round 5 and exact PR
  [#3033](https://github.com/sifr-lang/sifr/pull/3033) review round 2 report
  `SATISFIED`;
- the warmed `create-pr` lane passes every step through Python interop. Its
  Rust-interop step is blocked only by the preserved parallel-worktree
  promotion of `opaque_resource_matrix` while that row's evidence is still
  planned; the certification-3 fixture-matrix and tier suites pass, and this PR
  excludes that unrelated hunk. The earlier Python doctor timeout was
  reproduced as a passing focused check and passed inside the warmed full
  rerun.

`certification_3` may use bridge-version 1 call-scoped callbacks. Any callback
behavior that truly requires the bridge-version 2 structural call contract
must be split into the later package-resource item rather than silently
expanding this row.

### certification_4 through certification_8: Runtime Resources and Views

Implement the rows in numeric order as separate PRs. Shared loopback harness
code may be introduced by `certification_4` in a focused verification helper
module and reused afterward; service-specific behavior and evidence remain in
the owning row.

#### certification_4: Async Reqwest Runtime

Implementation checklist:

- [x] Make async signature probes invoke the target with typed, non-executed
  arguments so futures borrowing bridge inputs can be checked without erasing
  their lifetime family.
- [x] Reject package-local async bridge source that calls `block_on` or
  constructs a Tokio runtime, while ignoring comments and literals and keeping
  the rejection on `SIFR-RUST-ASYNC-0001`.
- [x] Add a locked/offline `reqwest_loopback_runtime` package that binds an
  ephemeral in-process HTTP server before spawning it, bounds accept/read/write
  and client operations, and never uses external DNS or services.
- [x] Execute borrowed-input request/response behavior twice on the generated
  Tokio runtime and prove current-thread runtime reuse.
- [x] Cancel a delayed reqwest future through a Sifr timeout and observe
  request/server guard drop plus zero active work after bounded cleanup.
- [x] Bind both evidence directions to mandatory generated-build tests, promote
  only `async_runtime_reqwest`, update structured claims/docs/provenance/counts,
  and preserve all later future-owned rows.
- [x] Run focused and authoritative local gates, Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_5`.

Post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 54 passing and 18 planned evidence directions;
- categories: 18 `supported`, 8 `supported-through-bridge`, 1
  `unsupported-by-design`, and 9 `future-owned-by-separate-phase`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
- 60 package examples and 14 scenario examples; and
- 27 structured stable claims.

Focused implementation evidence:

- the locked runtime package executes two borrowed-input reqwest calls on one
  generated current-thread Tokio runtime, cancels a third delayed call, and
  observes `completed=2`, `cancelled=1`, `runtime_calls=3`,
  `runtime_reused=true`, and zero active request/server work;
- package-local ordinary Rust source is audited before Cargo probing for
  nested Tokio runtime construction and blocking operations, while documented
  cross-file-glob and macro-expanded gaps remain governed by the package trust
  contract;
- a transitive native link is accepted only when declared by the bridge
  manifest, and the paired generated-build test rejects the undeclared link;
- all three generated-build tests, focused Rust interop tests, Clippy,
  formatting, matrix self-tests, maintainability guardrails, and file-size
  guardrails pass; working-tree Opus review
  [round 13](../../reviews/active/rust-interop-certification-4-review-round13.md)
  reports `SATISFIED`, the
  [exact-PR review](https://github.com/sifr-lang/sifr/pull/3036#issuecomment-5094639600)
  also reports `SATISFIED`, and
  [PR #3036](https://github.com/sifr-lang/sifr/pull/3036) is merged;
- the Rust interop area facade's only two failures are caused by the preserved
  parallel-worktree `opaque_resource_matrix` promotion while that row's
  evidence remains planned; shadow-copy checks excluding that one unrelated
  hunk pass, and this PR excludes it.

All harnesses must:

- bind only ephemeral loopback ports;
- start after the test owns cleanup handles;
- use bounded readiness and operation timeouts;
- terminate tasks/processes on success, failure, and cancellation;
- avoid external DNS/network and machine-installed services; and
- be exercised under the profile sandbox and offline Cargo policy.

Redis and PostgreSQL harnesses emulate only the handshake and request/response
frames exercised by the certified operations, plus the malformed/early-close
frames required by negative evidence. General Redis or PostgreSQL server
compliance is out of scope and must not expand these PRs.

#### certification_5: Opaque Resource Lifecycle Matrix

Implementation checklist:

- [x] Add a locked/offline `resource_lifecycle_runtime` package whose generated
  bridge signatures construct and consume `Handle<ResourceMatrix>` on the
  generated current-thread Tokio runtime.
- [x] Exercise `reqwest` through an ephemeral HTTP loopback, bundled
  `rusqlite` through a unique temporary database, and `redis` plus
  `tokio-postgres` through minimal deterministic loopback protocol servers.
- [x] Bind listeners and cleanup handles before task start, bound every
  operation and join, remove the temporary database, and require zero active
  harness-owned tracked tasks after close.
- [x] Execute malformed Redis RESP and PostgreSQL early-close paths and keep
  their scope limited to the frames used by this certification.
- [x] Prove shared-alias use-after-close rejection, stable double close, and
  exact panic-payload redaction through the runtime handle substrate.
- [x] Bind both evidence directions to distinct mandatory generated-build
  tests, promote only `opaque_resource_matrix`, and update structured stable
  claims, public/internal docs, provenance, counts, and validator self-tests.
- [x] Run focused and authoritative local gates, Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_6`.

Post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 56 passing and 16 planned evidence directions;
- categories: 18 `supported`, 9 `supported-through-bridge`, 1
  `unsupported-by-design`, and 8 `future-owned-by-separate-phase`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
- 60 package examples and 15 scenario examples; and
- 28 structured stable claims.

Focused implementation evidence:

- `cargo test -p sifr_driver --lib -- --ignored --test-threads=1
  test_build_opaque_resource` executed both mandatory generated-build runtime
  paths: 2 passed, 0 failed, in 77.58 seconds. This focused run complements the
  create-PR smoke profile, which compiles but intentionally does not execute
  ignored generated-build tests;
- `scripts/run_all_tests.sh --profile create-pr` passed every lane, including
  131/131 representative E2E fixtures, the 10/10 Rust interop area matrix,
  compiler/codegen/driver crate suites, all guardrails, and zero blocking
  hardening failures;
- `scripts/run_all_tests.sh` passed the authoritative merge profile after
  integrating exact `origin/main` base `f9837adb10`: 57/57 distribution
  variants (including the protected stable-release drill), 50/50 ignored
  generated-build tests (including both opaque-resource runtime paths),
  674/674 E2E pass fixtures, 261/261 hardening variants, and every blocking
  verification lane completed with zero failures. The report recorded only
  non-blocking cold-cache timing advisories after the regenerable
  generated-artifact cache was cleared;
- generated opaque-handle glue executes HTTP, SQLite, Redis, and PostgreSQL
  operations through a borrowed signature and closes the handle through the
  declared owned `close=async_close` member routed to
  `bridge.resources.aclose`; the operation summary is
  `http=echo:reqwest;sqlite=sqlite;redis=PONG;postgres=1`;
- the compiler carries the selected close receiver's ownership in Rust
  declaration metadata, preserves ordinary non-opaque bridge-method call
  shapes, and rejects mismatched, borrowed, or duplicate opaque closes before
  rustc;
- malformed Redis RESP and PostgreSQL early-close probes are rejected before
  cleanup, and the normal servers implement only the handshake and operation
  frames exercised by this scenario; Redis library-metadata `CLIENT SETINFO`
  is explicitly disabled rather than included in the certified frame set;
- the negative generated path operates on the four-resource identity, closes
  the original, and observes `resource-state=closed` when a real operation is
  attempted through its bridge-local shared alias; no Sifr-level clone policy
  is declared or claimed. The positive path observes `closed` then
  `already-closed`, exact `Rust bridge panicked` guard redaction, temporary
  database removal, and zero active harness-owned tracked tasks;
- `rusqlite` is exact-pinned at `0.39.0` because `0.40.1` selects
  `libsqlite3-sys 0.38.1`, whose build script uses unstable `cfg_select!` and
  fails on stable Rust toolchains (including stable 1.94);
  the downgraded locked graph retains the required bundled SQLite feature, and
  the already-supported blocking-diagnostics row is revalidated against that
  same exact root lock graph;
- working-tree review rounds 1–17 are recorded under `plans/reviews/active/`;
  [round 17](../../reviews/active/rust-interop-certification-5-review-round17.md)
  independently closes all prior findings and reports `SATISFIED`, and
  [PR #3042](https://github.com/sifr-lang/sifr/pull/3042) is the Certification
  5 merge.

#### certification_6: Callback Subscription Ecosystem

Implementation checklist:

- [x] Replace the thread-safe callback marker with a typed generated/runtime
  bridge that owns a `Send + Sync + 'static` handler, preserves the declared
  callback argument/result contract, contains callback panics, and exposes the
  exact backpressure, overflow, and shutdown policy to the package bridge.
- [x] Require owned thread-safe callback parameters and a fallible opaque
  subscription result, and reject named or nested handlers whose captures
  cannot be proven sendable and share-safe with `SIFR-RUST-CB-0001`.
- [x] Add a locked/offline `subscription_lifecycle_runtime` package using raw
  loopback WebSocket framing through `tokio-tungstenite`, a minimal Redis
  pub/sub RESP harness, and a unique temporary watched directory through
  `notify`.
- [x] Prove bounded overflow-as-error behavior, callback error propagation,
  exact panic-payload redaction, foreign-thread notification entry, explicit
  cancellation, drain shutdown, consuming async close, bounded task joins,
  temporary-directory removal, and zero active harness-owned work.
- [x] Bind both evidence directions to distinct mandatory generated-build
  tests, promote only `callback_subscription_ecosystem`, and update structured
  claims, public/internal docs, provenance, counts, and validator self-tests.
- [x] Run focused and authoritative local gates, Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_7`.

Post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 58 passing and 14 planned evidence directions;
- categories: 18 `supported`, 10 `supported-through-bridge`, 1
  `unsupported-by-design`, and 7 `future-owned-by-separate-phase`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
- 60 package examples and 16 scenario examples; and
- 29 structured stable claims.

Focused implementation evidence:

- the mandatory positive generated-build test executes typed retained callbacks
  through raw loopback WebSocket frames, Redis Pub/Sub RESP, and a real
  filesystem watcher, producing the exact lifecycle summary with bounded
  overflow, ordinary callback error, redacted panic, foreign-thread entry,
  queue drain, cancellation, consuming async close, zero active work, and
  temporary-directory removal;
- the mandatory negative generated-build test rejects both a named nested
  handler retaining `NonSend` state and one retaining a callable whose
  captures are unknown with `SIFR-RUST-CB-0001`, and rejects second attachment
  of a consumed nested handler with `SIFR-OWN-0001`; direct and
  sibling-transitive mutating captures also fail the retained `Fn` contract
  with `SIFR-RUST-CB-0001`, before Cargo probing;
- the package lock is an exact subset of the root lock graph, and its standalone
  Rust library passes locked/offline Cargo check and Clippy;
- fixture, compatibility, tier, scenario, stable-claim, and provenance checks
  bind both directions to distinct merge-profile test names and mutation-test
  the locked dependency policy, callback policy, foreign-thread observation,
  and subscription cleanup guardrails.
- Opus review round 1 found eight actionable gaps. The follow-up makes callback
  policy parsing canonical in the IR and rejects malformed policy before
  codegen, rejects retained callbacks under explicit or profile-level abort
  strategy, covers function and method attachment captures plus generated
  method bounds, drives queue overflow and close-time drain from the carried
  policy, cancels a real scheduled callback delivery before invocation, and
  splits callback bridges out of the saturated runtime module. The mandatory
  positive and negative generated-build tests pass together after these fixes.
- [Opus review round 2](../../reviews/active/rust-interop-certification-6-review-round-2.md)
  confirmed all round-1 findings closed and found four deeper attachment
  gaps. The follow-up emits verified nested retained handlers as owning
  `move` closures, traverses sibling nested-function captures transitively,
  transports retained-callback parameter indices through direct imports,
  aliases, re-exports, and imported methods, and enforces abort strategy before
  bridge-signature lookup. Focused regressions cover each path, and the
  mandatory positive generated package now builds and executes a nested
  handler with a verified local capture.
- [Opus review round 3](../../reviews/active/rust-interop-certification-6-review-round-3.md)
  confirmed the round-2 attachment, metadata, and panic-strategy findings
  closed, then exposed raw-rustc failures for blanket-moving non-`Copy`
  captures and accepting callable values with unknown captures. The follow-up
  clones verified non-`Copy` captures inside an isolated closure-construction
  block, preserves the enclosing binding after attachment and across loop
  iterations, rejects unprovable callable captures with
  `SIFR-RUST-CB-0001`, and teaches structured loop bodies to emit retained
  nested handlers. Focused lowering, codegen, and both mandatory generated
  package directions cover the corrected contract.
- [Opus review round 4](../../reviews/active/rust-interop-certification-6-review-round-4.md)
  confirmed every earlier high-severity raw-rustc gap closed, then found that
  reusing the generated handler binding itself could still reach rustc
  `E0382`, and that declaration-time capture snapshot semantics were unstated.
  The follow-up makes successful retained attachment an explicit ownership
  move, diagnoses second attachment and direct invocation with
  `SIFR-OWN-0001`, diagnoses outer-loop reuse with `SIFR-OWN-0004`, and pins
  declaration-time snapshots in codegen and the runtime-observed package.
- [Opus review round 5](../../reviews/active/rust-interop-certification-6-review-round-5.md)
  independently closed every prior finding, then found false rejection of
  attribute/method-derived locals whose capture type remained inference-time
  `Unknown`, plus an `FnMut` escape through direct or transitive `nonlocal`
  mutation. The follow-up refreshes capture types from lowered lexical
  bindings, gives genuinely unresolved captures an explicit contract
  diagnostic, records mutated nested captures, and rejects direct and
  transitive `FnMut` handlers before Cargo. Both mandatory generated-package
  directions exercise the corrected contract.
- [Opus review round 6](../../reviews/active/rust-interop-certification-6-review-round-6.md)
  confirmed every round-1 through round-5 remediation, then found that the
  test-body decomposition broke negative-evidence provenance and that
  assignment-target-only capture mutation could still reach raw rustc
  `E0525`. The follow-up restores diagnostic assertions to the manifest-bound
  generated-build test, discovers captures in assignment and deletion targets,
  analyzes mutation over actual captured bindings through structured control
  flow and sibling functions, covers collection-mutating methods, and prefers
  lowered lexical types over builtin-name inference. The unresolved-type
  branch now has a direct regression.
- [Opus review round 7](../../reviews/active/rust-interop-certification-6-review-round-7.md)
  confirmed all earlier findings and the real area gates, then found that a
  function nested inside the retained handler could hide both `FnMut` and
  `NonSend` capture use. It also found name-only `write` classification falsely
  rejected `RwLock.write()`, and exposed silent shadowing for a `nonlocal`
  walrus. The follow-up propagates free captures through arbitrarily nested
  helper scopes, walks their mutations with parameter/local shadowing, uses
  receiver types for collection mutation, adds generated positive `RwLock`
  evidence and four generated negative nested-helper directions, and rejects
  `nonlocal` walrus with `SIFR-FLOW-0003`.
- [Opus review round 8](../../reviews/active/rust-interop-certification-6-review-round-8.md)
  revalidated every earlier finding and both mandatory packages, then found
  capture and mutation traversal gaps in interpolated strings, lambdas, slice
  bounds, starred expressions, and comprehensions, plus incomplete nested
  parameter shadowing. The follow-up makes both expression walkers exhaustive,
  preserves comprehension/lambda lexical scope, strips every parameter kind,
  adds focused regressions for each escape, and extends the generated positive
  f-string clone evidence and negative hidden-capture diagnostics.
- [Opus review round 9](../../reviews/active/rust-interop-certification-6-review-round-9.md)
  independently reproduced every prior finding and expression escape, passed
  both mandatory generated packages, 1,901 affected tests, the full
  Rust-interop area, Clippy, formatting, and all guardrails, recomputed the
  complete inventory, and reported `SATISFIED` with no actionable finding.
- [Final PR-head review](../../reviews/active/rust-interop-certification-6-review-round-10.md)
  verified the exact [PR #3046](https://github.com/sifr-lang/sifr/pull/3046)
  head, the authoritative create-PR report, all mandatory and focused
  evidence, and the complete inventory, and reported `SATISFIED`.

#### certification_7: Crate-Backed Zero-Copy Runtime

Implementation checklist:

- [x] Bind opaque crate-backed `@rust.zero_copy(view=...)` to the exact Rust
  handle type carried by the function return, preserve contract-only generated
  record views, and reject annotation/return mismatches with
  `SIFR-RUST-ZC-0001`.
- [x] Carry the paired `@rust.view(...)` Send/Sync obligations onto the direct
  zero-copy type probe, treat `view=` as a type rather than a value, and map
  failed obligations to the zero-copy diagnostic family.
- [x] Add a locked/offline `crate_backed_view_runtime` generated package using
  exact root-lock versions of `bytes`, `memmap2`, `bytemuck`, and `zerocopy`
  with only safe Rust.
- [x] Observe moved-allocation identity for the owned buffer received by the
  bridge and retained owner lifetime for `bytes::Bytes`, exclusive mutation
  followed by read-only sealing for
  `memmap2`, pointer-identical `bytemuck` and `zerocopy` views, and consuming
  release with exactly one drop and zero active views.
- [x] Reject mutable views from shared owners, returned call-lifetime escape,
  and owner-lifetime async suspension before Cargo; independently mutate the
  package view to non-Send/non-Sync and require the direct probe to reject it.
- [x] Bind both evidence directions to distinct mandatory generated-build
  tests, promote only `zero_copy_runtime_matrix`, and update structured
  claims, public/internal docs, provenance, counts, and validator self-tests.
- [ ] Run focused and authoritative local gates, Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_8`.

Post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 60 passing and 12 planned evidence directions;
- categories: 18 `supported`, 11 `supported-through-bridge`, 1
  `unsupported-by-design`, and 6 `future-owned-by-separate-phase`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
- 60 package examples and 17 scenario examples; and
- 30 structured stable claims.

Focused implementation evidence:

- [Opus review round 1](../../reviews/active/rust-interop-certification-7-review-round-1.md)
  independently passed the mandatory packages, the full Rust-interop area,
  Clippy, formatting, and maintainability checks, then found six actionable
  gaps. The remediation replaces substring matching with exact Ok-slot opaque
  handle identity and prefix/container regressions, classifies view-trait
  failures without renderer-dependent raw rustc leakage, unit-tests all four
  Send/Sync probe forms, freezes `zerocopy[derive]` across both zero-copy rows
  and the catalog, reinterprets a mutated sealed mmap through bytemuck and
  zerocopy, expands scenario mutations, and decomposes scenario/probe tests so
  all maintained files remain below the 900-line cap.
- [Opus review round 2](../../reviews/active/rust-interop-certification-7-review-round-2.md)
  confirmed every round-1 finding closed and all focused gates green, then
  found one diagnostic-ordering regression and two robustness gaps. The
  follow-up preserves the bridge-type diagnostic for propagated unsupported
  Result slots, asserts the actual Send/Sync probe invocations, and moves
  callback/resource token inventories into their scenario-owned modules to
  restore durable file-size headroom.
- [Opus review round 3](../../reviews/active/rust-interop-certification-7-review-round-3.md)
  confirmed all earlier findings closed, then exposed full-driver regressions
  from applying opaque-handle identity to contract-only generated records, a
  new Clippy violation, and duplicated canonical target rendering. The
  follow-up scopes exact identity to opaque crate-backed handles while
  preserving generated-record metadata validation, reuses codegen's canonical
  handle renderer, adds a generated-record regression, and uses the
  Clippy-approved diagnostic note branch.
- [Opus review round 4](../../reviews/active/rust-interop-certification-7-review-round-4.md)
  reproduced the full driver and area gates and confirmed all earlier findings
  closed, then found two supported positive fixture sources still returned
  `bytes` instead of their declared opaque views and that the driver locally
  parsed codegen's generated bridge paths. The follow-up makes the manifest-
  bound tests lower and validate those exact checked-in fixtures, gives both
  fixtures opaque handle returns, and moves canonical/legacy generated-path
  recognition and its malformed-path regressions into codegen.
- [Opus review round 5](../../reviews/active/rust-interop-certification-7-review-round-5.md)
  confirmed the positive fixture and shared codegen-policy fixes, then found
  both contract-only negative provenance tests still used synthetic sources;
  the bytes fixture also omitted the copy-fallback contract it advertised.
  The follow-up gives that fixture a complete paired opaque view with explicit
  `copy_fallback=True` and makes both manifest-bound tests lower and validate
  their exact checked-in negative sources.
- [Opus review round 6](../../reviews/active/rust-interop-certification-7-review-round-6.md)
  confirmed direct provenance for all four contract-only directions and every
  earlier remediation, then found the copy-fallback test could not distinguish
  that key from any other unsupported key. The follow-up includes the exact
  rejected key in zero-copy and view diagnostics and pins the `copy_fallback`
  and legacy `mutable` assertions to their source tokens.
- [Opus review round 7](../../reviews/active/rust-interop-certification-7-review-round-7.md)
  independently reproduced the full driver, all three mandatory generated
  builds, Clippy, the Rust-interop area, guardrails, counts, and safe-Rust
  audit; confirmed every round-1 through round-6 remediation and unrelated-
  path preservation; and reported `SATISFIED` with no actionable finding.
- [Integrated-head Opus review round 8](../../reviews/active/rust-interop-certification-7-review-round-8.md)
  verified the current-main merge changed no Rust or Rust-interop file, the
  authoritative create-PR failure's three transfer-inventory anchors now match
  the exact probe reads and both transfer gates pass, all earlier findings
  remain closed, and the exact head is `SATISFIED`.
- [Integrated-head Opus review round 9](../../reviews/active/rust-interop-certification-7-review-round-9.md)
  confirmed the implementation remained sound after the next Phase 40
  integration, then found one low-severity import-order/spacing regression and
  required fresh exact-head lane evidence before closure. The follow-up
  restores the scenario-check module boundary and discards every stale lane
  report after the shared target was cleaned.
- [Integrated-head Opus review round 10](../../reviews/active/rust-interop-certification-7-review-round-10.md)
  independently rebuilt all three mandatory zero-copy packages, passed the
  full driver and codegen Rust-interop suites, the complete Rust-interop area,
  Clippy, formatting, file-size and maintainability guardrails, confirmed all
  earlier findings closed, and reported the implementation `SATISFIED`.
- The exact [PR #3053](https://github.com/sifr-lang/sifr/pull/3053) head passed
  the create-PR profile on 2026-07-29: all blocking steps were green, including
  all 10 Rust-interop variants, the smoke performance budget, 428 passing
  driver tests with 55 intentional generated-build ignores, and all 131 E2E
  fixtures. The warm wall-time advisory reflects cold artifact groups and a
  parallel release corpus; no blocking step exceeded its budget.
- [Final PR-head review](../../reviews/active/rust-interop-certification-7-review-round-11.md)
  re-derived the implementation and every prior finding against the exact
  published PR head, independently verified all inventories and create-PR
  evidence, found no blocking issue, and reported `SATISFIED`. Its one
  low-severity parser-robustness note affects only the wording of an
  already-invalid tuple-handle rejection and cannot accept an unsupported
  surface or introduce a panic path.

#### certification_8: Crate-Backed Advanced Data Runtime

Implementation checklist:

- [x] Add a locked/offline generated package with shared
  `sifr_arrow_bridge` and `sifr_tensor_bridge` crates using the exact root-lock
  versions and default-feature policies for Arrow, DataFusion, Polars,
  ndarray, and CPU-only Candle.
- [x] Make package-scoped native-link trust apply to direct/shared-crate Rust
  bindings as well as package-local `bridge.*` targets, with a focused
  resolver regression and an exact post-build native-link allowlist.
- [x] Move owned generated-package vectors into Arrow, ndarray, and Candle
  without allocation changes; register the Arrow record batch with
  DataFusion; and observe matching Polars schema, dtype, row count, rank,
  shape, layout, strides, and CPU device.
- [x] Consume the ndarray owner into a safe one-shot DLPack-style managed
  capsule without copying, then prove consuming close releases exactly one
  owner and leaves zero active owners.
- [x] Reject schema-root, rank/shape, and non-CPU device mismatches before
  Cargo through the checked-in negative fixture.
- [x] Bind both evidence directions to distinct mandatory generated-build
  tests, promote only `advanced_data_runtime_matrix`, retain all three
  narrower contract-only rows, and update structured claims, public/internal
  docs, provenance, counts, and mutation-tested scenario policy.
- [x] Run focused and authoritative local gates, Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_9`.

Review and validation notes:

- [Round 1](../../reviews/active/rust-interop-certification-8-review-round-1.md)
  found three blocking evidence gaps: an arm64-only native-link allowlist,
  independently constructed Polars values, and cleanup sampled only after
  close. The implementation now declares the exact arm64/x86_64 locked-graph
  envelope, derives Polars input from the crossed Arrow buffer with an
  explicitly reported copy, and asserts owners active before close plus
  released exactly once afterward.
- [Round 2](../../reviews/active/rust-interop-certification-8-review-round-2.md)
  independently confirmed those three fixes and found one stale scenario
  README count after the native-link envelope expanded from five emitted
  host-local names to seven cross-host entries.
- [Round 3](../../reviews/active/rust-interop-certification-8-review-round-3.md)
  re-derived the full milestone, the pinned build-script architecture outputs,
  and every evidence claim after that correction, found no actionable issue,
  and reported `SATISFIED`.
- [Round 4](../../reviews/active/rust-interop-certification-8-review-round-4.md)
  audited the root-lock parse cache added to keep the Rust-interop validation
  step within its blocking budget, confirmed scenario locks remain freshly
  mutation-tested and subset enforcement is unchanged, rechecked the complete
  milestone, and reported `SATISFIED`.
- [Published-head review](../../reviews/active/rust-interop-certification-8-review-round-5.md)
  independently re-derived the native-link envelope, lifecycle and no-copy
  evidence, inventories, claims, docs, and provenance against the exact
  [PR #3067](https://github.com/sifr-lang/sifr/pull/3067) head; reran both
  mandatory generated builds and all focused gates; and reported `SATISFIED`
  with no actionable finding.
- [Exact PR-head round 5](../../reviews/active/rust-interop-certification-8-review-round-5.md)
  independently rebuilt both mandatory generated packages, reran the complete
  Rust-interop area, all driver tests, Clippy, formatting, and guardrails,
  re-derived the native-link envelope and all inventories, and reported
  `SATISFIED` with no actionable finding against PR #3067 head
  `3bd82793a9652b30f23c08c4f54d11c5aa0e298a`.
- [Merge-readiness round 6](../../reviews/active/rust-interop-certification-8-review-round-6.md)
  audited the repeated full merge-lane performance-only failures, proved the
  four affected benchmark fixtures cannot reach package Rust-interop or the
  changed native-link trust path, identified pre-existing main-branch LSP
  baseline drift, matched the established `certification_2` environmental
  timing precedent, and reported `SATISFIED TO MERGE` with no PR-attributable
  blocker.
- Focused revalidation passes the positive generated-package runtime test, the
  exact three-diagnostic negative test, locked/offline scenario Clippy, 429
  non-generated driver tests, all 10 Rust-interop area variants, 152 fixture
  mutation cases, and the file-size/HIR/diff guardrails.
- The authoritative create-PR profile passes all blocking steps, including the
  Rust-interop check at 6.8 seconds against its 10-second budget, 429 driver
  tests, and all 131 E2E fixtures. Its total warm wall-time advisory reflects
  cold artifact groups and shared-host contention; every blocking step stayed
  within budget.
- Repeated full merge profiles pass every functional step: Python interop
  `25/25`, Rust interop `10/10`, developer tooling `32/32`, all guardrails,
  and the representative benchmark runner itself. Their only blocking result
  is the budget comparison for unchanged `check-project-004-project-graph`,
  `check-single-file-001-arithmetic`,
  `diagnostic-non-regression-002-json-diagnostic-schema`, and intermittently
  `lsp-query-003-diagnostics` under concurrent shared-host validation. Round 6
  statically confirms none can execute this PR's package-only Rust-interop
  trust path; this is accepted under the same environmental-drift policy used
  for `certification_2`. Repository-wide baseline recalibration remains a
  `certification_14` retrospective item and does not block `certification_9`.

Expected post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 62 passing and 10 planned evidence directions;
- categories: 18 `supported`, 12 `supported-through-bridge`, 1
  `unsupported-by-design`, and 5 `future-owned-by-separate-phase`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
- 60 package examples and 18 scenario examples; and
- 31 structured stable claims.

### certification_9 through certification_13: Cargo and Ecosystem

Implement the rows in numeric order as separate PRs. `certification_11` must
land before the tier-4 ecosystem rows so their locked/offline cache behavior is
part of the evidence rather than an assumption.

Native/proc-macro trust tests must prove rejection happens before the
untrusted script or macro can create a sentinel file. Ecosystem tests must use
checked-in lockfile resolution and temp packages; fetching during a validation
lane is a failure.

#### certification_9: Native Build-Script Trust

Implementation checklist:

- [x] Replace the planning scaffold's local `0.1.0` stand-ins with a
  locked/offline generated package whose direct wrapper crates compile exact
  root-lock `cc = 1.2.63`, `bindgen = 0.72.1`, `cxx = 1.0.198`, and
  `zstd = 0.13.3` dependencies.
- [x] Make each direct wrapper carry an actual build script so Cargo metadata
  exposes the pre-execution trust requirement; exercise cc compilation,
  bindgen generation, cxx bridge expansion, and zstd compression through safe
  generated-package calls.
- [x] Emit and runtime-observe deterministic versioned artifacts from every
  build script, compare fresh-build evidence, and bind the complete package to
  the checked-in lockfile with exact feature/default-feature policy.
- [x] Declare the exact direct and transitive native-link envelope, prove the
  post-build allowlist accepts only that envelope, and keep all build-script,
  native-link, and artifact paths hermetic to temporary package/target
  directories.
- [x] Turn the checked-in negative evidence into a pre-Cargo trust rejection:
  remove a required declared native link/build-script permission, arm an
  untrusted build script with a sentinel write, and prove
  `SIFR-RUST-TRUST-0001` is emitted while the sentinel remains absent.
- [x] Bind positive and negative evidence to distinct mandatory generated
  package tests, promote only `native_build_script`, and update structured
  claims, public/internal docs, provenance, counts, and mutation-tested
  scenario policy.
- [x] Run focused and authoritative local gates, Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_10`.

Expected post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 64 passing and 8 planned evidence directions;
- categories: 19 `supported`, 12 `supported-through-bridge`, 1
  `unsupported-by-design`, and 4 `future-owned-by-separate-phase`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
- 60 package examples and 18 scenario examples; and
- 32 structured stable claims.

Review and validation notes:

- [Opus round 1](../../reviews/active/rust-interop-certification-9-review-round-1.md)
  independently reproduced the locked graph, deterministic artifacts,
  sentinel effectiveness, native-link envelope, area inventory, and safety
  constraints. It reported two medium and nine low findings before returning
  `NOT SATISFIED`.
- The round-1 fixes add a post-build rejection for undeclared transitive
  `zstd` evidence, an armed-script positive control, kind-specific
  pre-execution diagnostics, a real zstd encode/decode roundtrip, version
  literal mutation coverage, and checked `probe.c` identity. Documentation
  now scopes the claim to the Apple/GNU arm64/x86_64 host envelope and records
  the C/C++ compiler plus libclang prerequisites.
- [Opus round 2](../../reviews/active/rust-interop-certification-9-review-round-2.md)
  independently reran both mandatory tests, all area inventories, pin
  agreement, suite binding, lint, formatting, and guardrails; re-inspected all
  eleven round-1 findings; and reported `SATISFIED` with no actionable
  findings.
- [Exact-head round 3](../../reviews/active/rust-interop-certification-9-review-round-3.md)
  audited PR #3069 head `b5497901d4d7c7d90a65d03402708f6642e913ea`,
  validated the committed matrix blob independently, confirmed the unrelated
  backend hunk is absent, re-derived every inventory, and reported
  `SATISFIED` with no actionable findings.
- [Merge-readiness round 4](../../reviews/active/rust-interop-certification-9-review-round-4.md)
  verified the doc-only round-3 recording commit at PR head
  `1d66d90b0014c7218ebe1eac9b46f5a6dd37a772`, reconfirmed the backend hunk
  remains excluded and the PR is mergeable/clean, and reported `SATISFIED`
  with no actionable findings.
- Focused revalidation passes both mandatory generated-package tests: the
  positive proof executes two fresh byte-identical locked/offline/frozen
  builds plus generated Sifr check/build/run in 63.74 seconds, while the
  negative proof executes the sentinel control, both pre-execution trust
  removals, and the transitive post-build rejection in 27.66 seconds.
- Fixture-matrix self-tests pass all 166 mutation cases. Workspace Clippy,
  scenario Clippy, rustfmt, 429 non-generated driver tests, HIR
  maintainability, file-size, and diff-hygiene gates passed before the
  round-1 fixes and are rerun as part of the final authoritative gate.
- The shared worktree's full Rust-interop area currently has one unrelated
  failure: a parallel `ecosystem_backend_certification` category edit promotes
  that row while its evidence remains planned. Certification 9 does not stage
  or claim that hunk.
- The authoritative create-PR profile passed every step before Rust interop,
  including all 19 Python-interop variants. Rust interop passed 9 of 10
  variants and stopped only on that same unstaged backend category/evidence
  mismatch; all certification-9 matrix, mutation, tier, stale-draft, and
  stable-candidate variants passed.
- PR #3069 merged on 2026-07-29 as
  `afd25c3920a646fb0eea273c6899010baa7e94b7`; only
  `certification_10` is unblocked.

#### certification_10: Proc-Macro and Codegen Trust

Implementation checklist:

- [x] Replace the local `0.1.0` stand-ins with direct wrapper crates that
  compile exact root-lock `serde_derive = 1.0.228` and
  `prost-build = 0.14.4` under locked/offline/frozen Cargo.
- [x] Execute a real wrapper derive macro in generated package code, expose a
  deterministic marker through safe bridge glue, and make the macro capable
  of writing an explicitly armed sentinel for the negative proof.
- [x] Run `prost-build` over an in-memory descriptor set without `protoc`,
  keep generated files under `OUT_DIR`, compare two fresh builds
  byte-for-byte, and runtime-observe the generated schema artifact.
- [x] Close the package-wide trust gap so every direct proc-macro,
  build-script, and native-link dependency that Cargo can execute is validated
  before any direct probe or final package build, including package-local
  bridge targets.
- [x] Independently remove proc-macro and build-script permissions, prove
  kind-specific `SIFR-RUST-TRUST-0001` diagnostics before their armed
  sentinels, and include positive controls proving both sentinels execute when
  trust is present.
- [x] Bind both evidence directions to mandatory generated-package tests,
  promote only `proc_macro_trust`, and update structured claims,
  public/internal docs, provenance, counts, cache-identity assertions, and
  mutation-tested scenario policy.
- [x] Extract scenario dispatch before growing the current 891-line module,
  run focused and authoritative local gates, complete Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_11`.

Expected post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 66 passing and 6 planned evidence directions;
- categories: 20 `supported`, 12 `supported-through-bridge`, 1
  `unsupported-by-design`, and 3 `future-owned-by-separate-phase`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
- 60 package examples and 18 scenario examples; and
- 33 structured stable claims.

Validation evidence to date:

- The positive mandatory test passes two fresh locked/offline/frozen builds,
  compares byte-identical generated Rust and version evidence, then checks,
  builds, and runs the generated Sifr package in 32.89 seconds after the
  round-1 fixes.
- The negative mandatory test passes its armed-sentinel control and both
  independent pre-execution trust removals, and now also proves the checked-in
  negative fixture is valid with trust present, in 16.83 seconds.
- All 432 non-generated driver tests pass; the seven focused trust tests,
  including direct/local proc-macro coverage, declaration attribution, and
  proc-macro trust cache identity, also pass.
- The fixture inventory passes with 36 rows, 44 crate aliases, 60 package
  examples, and 18 scenarios; all 184 mutation cases pass. Tier,
  compatibility, stale-draft, stable-candidate, formatting, HIR
  maintainability, scenario Clippy, and production `sifr_driver` Clippy gates
  pass.
- The full shared-worktree Rust-interop area passes 9 of 10 variants. Its sole
  failure is the preserved unrelated
  `ecosystem_backend_certification` category edit, which promotes that row
  while both evidence directions remain planned. This item does not stage or
  claim that parallel-agent hunk.
- Workspace/all-target Clippy is currently blocked by unrelated parallel
  changes in `sifr_stdlib_manifest`, `sifr_ipc`, `sifr_lowering`, and existing
  `sifr_driver` test-only lint findings. The production `sifr_driver` library
  and this scenario pass `-D warnings`.
- [Opus round 1](../../reviews/active/rust-interop-certification-10-review-round-1.md)
  independently reproduced exact pins, both mandatory tests, deterministic
  codegen, sentinel behavior, package-wide trust, inventories, claims, and
  guardrails. It reported one medium and five low findings before returning
  `NOT SATISFIED`.
- The round-1 fixes make the negative fixture type-correct with trust present
  and add both an executable valid control and a mutation guard. Trust
  diagnostics now select a declaration that actually targets the dependency,
  name the exact `[trust]` allow-list in user-visible guidance, and retain both
  direct-root and local-bridge proc-macro unit coverage. Evidence now labels
  upstream `serde_derive` compilation separately from execution of the
  wrapper's `SifrGenerated` macro.
- [Opus round 2](../../reviews/active/rust-interop-certification-10-review-round-2.md)
  independently re-ran both mandatory tests, all 432 non-generated driver
  tests, the 184 mutation cases, inventories, claims, lint, formatting, and
  guardrails; re-inspected every round-1 finding; and returned `SATISFIED`
  with no actionable findings.
- The authoritative create-PR profile passed every step before Rust interop,
  including all 19 Python-interop variants. Rust interop passed 9 of 10
  variants and stopped only on the same unstaged parallel
  `ecosystem_backend_certification` category/planned-evidence mismatch; all
  certification-10 matrix, mutation, tier, stale-draft, and stable-candidate
  variants passed.
- [Exact-head round 3](../../reviews/active/rust-interop-certification-10-review-round-3.md)
  audited PR #3071 head
  `4e73e3cddbe6b4ef5875bd2ea697713f4730a866`, validated the committed
  compatibility matrix in an exported tree with the backend hunk absent,
  reran the mandatory and focused gates, and reported `SATISFIED` with no
  actionable findings.
- [Merge-readiness round 4](../../reviews/active/rust-interop-certification-10-review-round-4.md)
  found no repository defect at head
  `60512845062501d85b6a908c50b3ca9d97cecea1`, but correctly returned
  `NOT SATISFIED` because PR #3071 was still a draft.
- After the PR was marked ready and its body updated,
  [round 5](../../reviews/active/rust-interop-certification-10-review-round-5.md)
  reconfirmed the unchanged head, clean merge state, committed matrix
  exclusion, and review history, then returned `SATISFIED` with no actionable
  findings.
- PR #3071 merged on 2026-07-29 as
  `3c9601d268747b4543fbdca864f6a8ba50c44656`; only
  `certification_11` is unblocked.

#### certification_11: Locked, Offline, and Frozen Cargo

Implementation checklist:

- [x] Replace the planning scaffold with a generated package whose dependency,
  feature policy, and checked-in `Cargo.lock` are exact and root-lock-backed.
- [x] Exercise Sifr `check`, `build`, and `run` with `--locked`, `--offline`,
  and `--frozen`, proving every package/Cargo subprocess preserves the
  requested resolution mode.
- [x] Prove a cold prepared build followed by a network-disabled warm cache hit
  reuses the same deterministic package and artifact identity without registry
  access or lockfile writes.
- [x] Independently reject missing/stale lock entries, checksum/source drift,
  feature drift, and frozen-mode mutations with stable
  `SIFR-RUST-CARGO-0001` diagnostics and no silent resolver fallback.
- [x] Bind both evidence directions to mandatory generated-package tests,
  promote only `cargo_locked_offline`, and update structured claims,
  public/internal docs, provenance, counts, and cache-identity assertions.
- [x] Add exact scenario-policy and mutation coverage for flags, lock identity,
  feature policy, network denial, cache evidence, and each negative drift
  direction without growing a maintained module past the file-size cap.
- [x] Run focused and authoritative local gates, complete Opus review rounds to
  satisfaction, merge the PR, and unblock only `certification_12`.

Validation evidence to date:

- The mandatory positive test executes a cold prepared frozen build, a warm
  cache hit with the identical binary path, and direct `sifr check`, `sifr
  build`, and `sifr run` operations under frozen, locked, and offline modes.
  The exact-pinned `indexmap 2.14.0` bridge returns its version marker and
  deterministic hash. An exact Cargo-argv sink observes package metadata,
  generated resolution, Rust probes, and final builds; the final current-code
  run passed in 65.01 seconds.
- The mandatory negative test runs five independent no-network mutations
  through the Sifr check command: missing lock, stale selected version,
  checksum drift, source drift, and real registry feature drift. Every case
  reports the observed `SIFR-RUST-CARGO-0001` diagnostic with a distinct reason,
  leaves the lock absent or byte-identical, and the focused test passed in 0.36
  seconds. The Rust declaration lives in an imported non-entry module, so the
  diagnostic is classified from parsed package state rather than source text.
- Package-owned signature probes now preserve package Cargo source ownership
  instead of forcing the sysroot vendor used by `SysrootOnly` builds. Generated
  locks are prepared from authoritative package/sysroot locks, registry
  identities and checksums are validated, constrained probes use frozen
  strength, normal and constrained probe cache entries are isolated, and
  prepared-lock cache publication is atomic. Package-lock pins take precedence
  over sysroot pins, and prepared-lock cache identities normalize ephemeral
  path roots while retaining manifest and authority digests.
- The fixture inventory passes with 36 rows, 44 crate aliases, 61 package
  examples, and 18 scenarios. All 190 meaningful mutation cases, tier checks,
  and 34 structured stable claims pass. On the clean committed matrix baseline this
  item yields 68 passing and 4 planned evidence directions with 21 supported,
  12 bridge-supported, 1 unsupported-by-design, and 2 future-owned rows.
- The shared worktree compatibility command still stops only on the preserved
  parallel-agent `ecosystem_backend_certification` category edit, whose two
  evidence directions remain planned. This item will not stage or claim that
  unrelated hunk.
- The exact lock-argument, combined locked/offline normalization, Cargo failure
  classification, package-owned source selection, formatting, stable-claim,
  and 900-line file-size guards pass.
- [Opus round 1](../../reviews/active/rust-interop-certification-11-review-round-1.md)
  reproduced the mandatory paths and core architecture, then returned
  `NOT SATISFIED` for lint failures, entry-file lexical classification, and a
  negative test that asserted a directly invoked classifier rather than the
  command's emitted diagnostic. Its non-blocking findings covered exact argv
  evidence, package/sysroot authority precedence, grouped drift reasons,
  manifestless fallback, file-size headroom, and prepared-lock cache growth.
- The round-1 corrections make all three commands reject constrained
  manifestless operation; capture actual CLI diagnostics and every relevant
  Cargo subprocess; prioritize and test the package lock; distinguish all five
  drift reasons without retrying or mutating Cargo resolution; normalize
  prepared-cache path identities; and split command, diagnostic, entrypoint,
  probe, and generated-package test responsibilities below the 900-line cap.
  Production `sifr`, `sifr_package`, and `sifr_driver` Clippy, 142 package
  tests, 435 non-generated driver tests, rustfmt, HIR maintainability,
  TypeScript-Go transfer inventory, file-size, and diff-hygiene gates pass.
- [Opus round 2](../../reviews/active/rust-interop-certification-11-review-round-2.md)
  independently re-ran both mandatory tests, package/driver tests, workspace
  and production Clippy, every Rust-interop inventory, formatting, transfer,
  maintainability, file-size, and diff-hygiene gates. It re-inspected B1-B3
  and N1-N8 individually, validated the cert-11 matrix with only the unrelated
  backend hunk removed, recomputed the documented counts exactly, and returned
  `SATISFIED` with no blocking findings.
- [Exact-head round 3](../../reviews/active/rust-interop-certification-11-review-round-3.md)
  audited PR #3075 head
  `4c1fdeae6e774460ab4c4cb3ddbe19c1016c1471`, proved the unrelated backend
  promotion is absent from the commit, re-ran the mandatory and focused gates,
  validated every provenance/link and the exported committed matrix, and
  returned `SATISFIED` with no blocking findings.
- The final uncontended `scripts/run_all_tests.sh --profile create-pr` run
  passed every lane except the live-worktree Rust-interop compatibility check.
  That check stopped only on the preserved unstaged parallel-agent promotion
  of `ecosystem_backend_certification` while its evidence remains planned.
  The exported committed PR head passes the same checker with the documented
  68 passing / 4 planned evidence directions and exact category counts.
- [Merge-readiness round 4](../../reviews/active/rust-interop-certification-11-review-round-4.md)
  independently audited ready-for-review PR #3075 at
  `68c5f1a43091ddac40aa230d76d76bca308dc7fc`, reproduced the mandatory
  positive/negative tests, package and driver tests, workspace Clippy,
  committed-matrix counts, file-size and guardrail checks, and additionally
  proved stale-lock rejection for `check`, `build`, and `run`. It returned
  `SATISFIED` with no blocking findings and confirmed the sole live-tree gate
  failure is absent from the PR.
- A final exact-head confirmation reviewed published head
  `4452643a94deb28068ea994780878f540b2e88bf`, confirmed the prior
  `SATISFIED` merge-readiness verdict still applied, and recommended merge.
  PR #3075 merged on 2026-07-30 as
  `d5a4b294d3d8f88ea332733d74e9505abaedad5d`; only
  `certification_12` is unblocked.

#### certification_12: CLI and Tooling Ecosystem Bridge

Implementation checklist:

- [x] Replace the planning-only shadow crates with an exact-pinned generated
  package that compiles the real `clap`, `tracing`,
  `tracing-subscriber 0.3.23` with `env-filter`, and `anyhow 1.0.102`
  dependency graph under the package lock.
- [x] Execute a deterministic CLI parse and filtered tracing event through a
  package-local Rust bridge, and prove internal `anyhow::Error` context is
  collapsed into a declared Sifr-facing error before crossing the bridge.
- [x] Add a direct Rust surface whose `anyhow::Error` result cannot be
  represented by the Sifr bridge, then prove the real compiler reports the
  stable `SIFR-RUST-TYPE-0001` diagnostic while the explicit adapter remains
  accepted.
- [x] Bind both evidence directions to mandatory generated-package tests,
  promote only `ecosystem_cli_certification` to
  `supported-through-bridge`, and update scenario policy, structured claims,
  public/internal docs, provenance, and exact inventory counts.
- [x] Add validator self-test mutations for exact versions, the
  `env-filter` feature, bridge dependency ownership, positive/negative
  provenance, and the supported-through-bridge contract without weakening the
  preserved backend row.
- [x] Run focused and authoritative local gates, complete Opus review rounds
  to satisfaction, merge the PR, and unblock only `certification_13`.

Validation evidence to date:

- The mandatory positive test builds the authoritative package lock, confirms
  the exact Cargo feature graph, and executes real `clap 4.6.1` parsing plus a
  captured `tracing 0.1.44` event under
  `tracing-subscriber 0.3.23` `env-filter`. Its stable marker also proves the
  internal `anyhow 1.0.102` path returns only the declared `CliError`; the
  focused run passed.
- The mandatory negative test first proves the explicit adapter is accepted,
  then targets a sibling function returning `anyhow::Error` directly. The
  actual rustc signature evidence names `anyhow::Error` and the compiler
  reports `SIFR-RUST-TYPE-0001`, with no trust diagnostic masking the type
  contract; the focused run passed.
- The generated scenario builds with `cargo build --workspace --locked
  --offline --frozen`. Its lock is a root-lock subset and contains the exact
  selected transitive identities rather than locally newest compatible
  versions.
- The fixture inventory passes with 36 rows, 44 crate aliases, 61 package
  examples, and 18 scenarios. All 209 meaningful scenario/matrix mutation
  cases pass. On the exact staged tree this item yields 70 passing and 2
  planned evidence directions with 21 supported, 13 bridge-supported, 1
  unsupported-by-design, and 1 future-owned row; all four compatibility
  categories remain represented.
- The preserved unstaged `ecosystem_backend_certification` promotion still
  fails only the live compatibility command because its evidence remains
  planned. The staged certification-12 tree retains that row as future-owned
  and passes the fixture, compatibility, tier, and 35-claim stable-support
  checkers.
- [Opus round 1](../../reviews/active/rust-interop-certification-12-review-round-1.md)
  reproduced the full CLI/tooling contract and both mandatory tests, then
  returned `NOT SATISFIED` because the generic fixture checker had grown from
  899 to 904 lines. The correction moved fixture-specific binding-token policy
  into `_binding_helpers.py`, restored the checker to the hard cap, and
  added load-bearing exclusion plus direct-binding policy mutations.
- [Opus round 2](../../reviews/active/rust-interop-certification-12-review-round-2.md)
  audited integrated head
  `e2c321a788142bdf0da02967efee076c985a3d7c`, proved the certification patch
  survived the current-main merge byte-for-byte, re-ran both mandatory tests
  and the exact exported Rust-interop area, and returned `SATISFIED` with no
  blocking findings. Its filter-durability observation is closed by requiring
  and mutation-testing the excluded-target emission itself.
- [Published-head round 3](../../reviews/active/rust-interop-certification-12-review-round-3.md)
  reproduced every certification-12 gate at PR #3076 head
  `3867b21d56dc961b944c9259c632de2fc1d9d3c4`, then returned
  `NOT SATISFIED` because the carried certification-11 round-5 file contained
  only a truncated conversational tail rather than a self-contained review.
  The stub artifact and its link are removed; the historical confirmation is
  retained as plain prose instead of overstated provenance.
- [Published-head round 4](../../reviews/active/rust-interop-certification-12-review-round-4.md)
  verified the provenance repair at PR head
  `eca5abb7d9fca587ad6f31b3310f3e470db693d5`, re-ran the exact committed
  Rust-interop area and guardrails, confirmed the parallel backend edit remains
  absent, and returned `SATISFIED` with no blocking findings.
- The shared-worktree `create-pr` lane stops at the resource-certification
  backstop because the preserved unstaged backend promotion removes the last
  future-owned row. An exact-head archive passes that backstop and every
  certification/core guardrail; its cold diagnostic run exceeded the
  archive-path timing budget, while the warm rerun passed diagnostics in
  7.28 seconds. The archive-only Python doctor then exceeded its subprocess
  limit because its generated package lived under the temporary source root;
  the same doctor passed in the main workspace with deferred=1, resolved=3,
  parity=5, and zero mutations. No failing command touches certification-12
  source, and the exact-head Rust-interop area remains 10/10.
- [Final merge-readiness round 5](../../reviews/active/rust-interop-certification-12-review-round-5.md)
  independently exported and audited published PR head
  `96ab24c553d2afc05d24686b591bedd9f6289858`, reproduced the complete 10/10
  Rust-interop area, mandatory tests, workspace Clippy, committed inventory,
  guardrails, and shared-worktree gate attribution, and returned `SATISFIED`
  with no blockers. PR #3076 merged on 2026-07-30 as
  `ea119724e325b3900ccca81db766114d76eb4efd`; only `certification_13` is
  unblocked.

#### certification_13: Backend and Service Ecosystem Bridge

Implementation checklist:

- [x] Replace the planning-only shadow crates with an exact-pinned generated
  package that compiles real `axum 0.8.9`, `tower-http 0.7.0`, and
  `sqlx 0.8.6` with only the frozen
  `runtime-tokio-rustls`/`postgres`/`macros` SQLx feature policy under the
  checked-in package lock.
- [x] Execute a hermetic `127.0.0.1:0` Axum service through a real
  `tower-http` middleware layer, observe the response and middleware evidence,
  and shut the listener/task down deterministically without external network
  access.
- [x] Compile a real SQLx query macro from checked-in `.sqlx/` metadata under
  `SQLX_OFFLINE=true`, bind the query identity and metadata hash into runtime
  evidence, and prove neither `DATABASE_URL` nor a live database is required.
- [x] Turn `sqlx_without_offline_artifacts` into a mandatory generated-package
  diagnostic: independently remove and stale-mutate the checked-in query
  metadata, require stable `SIFR-RUST-CARGO-0001`, and prove rejection occurs
  with database/network access disabled.
- [x] Bind positive and negative evidence to distinct mandatory driver tests,
  promote only `ecosystem_backend_certification` to
  `supported-through-bridge`, and update scenario policy, structured claims,
  public/internal docs, provenance, and exact inventory counts.
- [x] Add validator self-test mutations for exact versions and features,
  bridge ownership, loopback/middleware execution, offline environment,
  metadata identity, both negative directions, evidence provenance, and the
  supported-through-bridge contract without weakening earlier rows.
- [ ] Run focused and authoritative local gates, complete Opus review rounds
  to satisfaction, merge the PR, and unblock only `certification_14`.

Expected post-item inventory:

- 36 fixture-matrix rows, 36 compatibility rows, and 36 schema-v2 fixture
  manifests;
- 72 passing and 0 planned evidence directions;
- categories: 21 `supported`, 14 `supported-through-bridge`, and 1
  `unsupported-by-design`;
- execution kinds remain 13 `cargo-probe`, 4 `compiler-diagnostic`, 10
  `contract-only`, and 9 `runtime-observed`;
- 44 required exact-pinned crate aliases in the checked-in root lock graph;
- 61 package examples and 18 scenario examples; and
- 36 structured stable claims.

Validation evidence to date:

- The mandatory positive test builds the root-lock-backed exact dependency
  graph, runs generated Sifr package glue, binds Axum to `127.0.0.1:0`,
  observes a `tower-http 0.7.0` response header and HTTP 200 body, expands the
  SQLx query metadata for value 13, and completes graceful shutdown. The
  current cold generated-package run passed in 74.81 seconds.
- The mandatory negative test first accepts the checked-in metadata, then
  removes and stale-mutates it on the same package root. A pre-Cargo verifier
  prevents stable Cargo's incomplete `.sqlx/` input tracking from hiding
  either mutation behind a warm probe cache. Both cases report
  `SIFR-RUST-CARGO-0001`; `SQLX_OFFLINE=true` is forced, `DATABASE_URL` is
  removed from compiler Cargo commands, and the armed database listener
  observes no connection. The current run passed in 63.34 seconds.
- The direct scenario package and the full workspace both build with
  `--locked --offline --frozen`. The scenario lock is a root-lock subset and
  resolves real `axum 0.8.9`, `tower-http 0.7.0`, `sqlx 0.8.6`, and the frozen
  SQLx feature set rather than local shadow crates.
- The fixture inventory passes with 36 rows, 44 crate aliases, 61 package
  examples, and 18 scenarios. All 228 mutation cases pass. Compatibility has
  72 passing and zero planned evidence directions across 21 supported, 14
  bridge-supported, and 1 unsupported-by-design row; 36 stable claims pass
  and the entire Rust-interop area is green at 10/10 variants.
- Because this is the final future-owned row, the compatibility checker now
  permits the declared `future-owned-by-separate-phase` category to be empty
  while still requiring all three active categories and rejecting unknown
  categories. Its completed-matrix and missing-active-category self-tests
  pass. The completion-time resource gate now also accepts zero deferrals
  while retaining the rule that only passing supported stdlib-core rows may
  authorize retained compiler surfaces.
- All 440 non-generated driver tests pass with 65 generated-build tests
  intentionally ignored. Production `sifr_driver` Clippy, workspace rustfmt,
  TypeScript-Go transfer inventory, HIR/driver maintainability, 900-line
  file-size, and diff-hygiene guardrails pass.
- The authoritative create-PR lane passed every core, Python-interop, and
  Rust-interop check, then recorded one shared-worktree LSP smoke timeout after
  23 successful protocol requests. The exact six-variant `lsp-smoke` suite
  passed immediately on isolation, including protocol shutdown, marker corpus,
  transcript replay, and all self-tests. Workspace Clippy passes with warnings
  denied.

### certification_14: Track A Closeout and Stable Gate

This PR starts only when `certification_1` through `certification_13` are
merged.

- Re-run the row/fixture inventory and update documented counts.
- Confirm the completion-time backstop transition in
  `scripts/check_sysroot_stdlib_resource_certification_gate.py` that currently
  accepts zero future-owned rows, and re-run the guard's completed-matrix
  `--self-test`. Keep its supported stdlib-core invariants.
- Confirm `check_compatibility_matrix.py` permits an unused
  `future-owned-by-separate-phase` category after all current deferrals
  resolve while still rejecting unknown categories and invalid rows.
- Remove stale `future_owner` fields from promoted rows and confirm no planned
  evidence status remains in Track A rows.
- Run the Phase 40 stable-candidate check and verify public docs advertise only
  the structured stable claims with their exact execution scope.
- Convert the completed stdlib handoff below to durable historical wording,
  update Phase 39/Phase 40/roadmap status links, and record the final Opus
  review.

Track A is complete only when its inventory, all validator self-tests, the
entire Rust-interop area, create-PR lane, merge lane, and stable-candidate gate
pass locally.

## Completed Stdlib Native-Boundary Handoff

The archived stdlib native-boundary phase already split broad ecosystem rows
rather than claiming them wholesale:

- `opaque_resource_matrix` was split into supported stdlib
  `opaque_resource_core` plus the retained ecosystem resource row.
- `async_runtime_reqwest` was split into the supported contract-only
  `async_runtime_core` row plus the retained reqwest loopback row. Runtime task
  cancellation/drop remains part of the downstream row's certification.
- the former subscription matrix was split into the supported contract-only
  `callback_subscription_core` row plus
  `callback_subscription_ecosystem`; subscription cancellation/shutdown
  execution remains downstream.
- `callbacks_call_scoped` and `panic_boundary_wrapper_emission` were not
  claimed by stdlib migration and remain owned here.

The supported core rows remain regression constraints. Track A must not fold
service loopbacks or external-package claims into them.

## Track B: Native Pydantic-Sifr Package Resource Certification

Native Pydantic-Sifr consumes three certification-owned rows:

- `opaque_resource_package_core`;
- `callbacks_call_scoped`; and
- `panic_boundary_wrapper_emission`.

The latter two land through `certification_2` and `certification_3`.
`opaque_resource_package_core` is intentionally not pre-created as a
README-only future row because its general substrate does not exist until
Native Pydantic-Sifr `milestone_ps_2` releases bridge version 2.

### certification_pkg_resource_core

Dependency order:

1. Native Pydantic-Sifr `milestone_ps_2` merges and releases the general
   bridge-version 2 package-resource substrate.
2. This sequential certification item creates and certifies the general
   synthetic-package row.
3. Native Pydantic-Sifr `milestone_ps_3` may start only after this item,
   `certification_2`, and `certification_3` are merged and released.

This one PR:

- creates `opaque_resource_package_core` as tier 2,
  `runtime-observed`, with no service-specific crate;
- uses positive evidence `package_resource_construct_use_close` to execute
  sealed construct/use/close through a synthetic external Rust-backed Sifr
  package;
- uses negative evidence `package_resource_alias_use_after_close_rejected` to
  prove alias, use-after-close, double-close, poison, and cross-package
  construction rejection;
- binds both evidence directions to executable mandatory-lane tests;
- updates the matrices, tiers, `REQUIRED_FIXTURES`, fixture files,
  architecture/public docs, stable-claims data if the surface is advertised,
  and current counts; and
- runs the real compiler/runtime with the released bridge-version 2 substrate,
  not a Pydantic-specific adapter or private bypass.

Exit gate: the synthetic package executes, both evidence directions pass, all
Rust-interop checks and their self-tests pass, the full area runner passes, and
`scripts/run_all_tests.sh --profile create-pr` plus
`scripts/run_all_tests.sh` pass locally. Only then may Native Pydantic-Sifr
`milestone_ps_3` begin.

## Validation Required for Every Item

Focused commands vary by row and must be written into the item checklist before
implementation. The minimum common gate is:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area rust_interop
python3 verification/areas/rust_interop/checks/check_fixture_matrix.py --self-test
python3 verification/areas/rust_interop/checks/check_compatibility_matrix.py --self-test
python3 verification/areas/rust_interop/checks/check_tiers.py --self-test
scripts/run_all_tests.sh --profile create-pr
scripts/run_all_tests.sh
cargo clippy --workspace -- -D warnings
cargo fmt --check
python3 scripts/check_hir_maintainability_guardrails.py
python3 scripts/check_file_size_guardrails.py
git diff --check
```

The focused provenance test for each changed evidence side must also be run
directly with the exact Cargo command/test filter recorded in `fixture.json`.
Do not wait on CI; the local gates are authoritative.

## Stable Release Constraint

Phase 40 may advertise only rows accepted by
`stable_support_claims.json` and the stable-candidate check. Honest
future-owned rows may remain visible as unsupported future work, but no stable
docs, installer metadata, release checklist, or package authoring guide may
promote them.

Track A is the work that makes the currently deferred ecosystem eligible for
stable claims. Track B is independently gated by the later bridge-version 2
release and does not delay Phase 40 while absent and unadvertised.
