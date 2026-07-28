# Rust Interop Runtime and Ecosystem Certification Follow-Up

## Status

Active follow-up created by Phase 39 closeout.

The verification-hardening dependency is complete through
[`hardening_4`](../archive/rust-interop-verification-matrix-hardening.md#hardening_4-replace-lexical-rejection-context):
[PRs #3018](https://github.com/sifr-lang/sifr/pull/3018),
[#3019](https://github.com/sifr-lang/sifr/pull/3019),
[#3020](https://github.com/sifr-lang/sifr/pull/3020),
[#3022](https://github.com/sifr-lang/sifr/pull/3022), and
[#3023](https://github.com/sifr-lang/sifr/pull/3023) are merged.
`certification_0` may start. The row implementation sequence remains blocked
until `certification_0` completes the remaining pre-row entry criteria below.

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
| `certification_6` | in progress | callback subscription lifecycle matrix starts after `certification_5` |
| `certification_7` | blocked | starts after `certification_6` merges |
| `certification_8` | blocked | starts after `certification_7` merges |
| `certification_9` | blocked | starts after `certification_8` merges |
| `certification_10` | blocked | starts after `certification_9` merges |
| `certification_11` | blocked | starts after `certification_10` merges |
| `certification_12` | blocked | starts after `certification_11` merges |
| `certification_13` | blocked | starts after `certification_12` merges |
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
- [ ] Run focused and authoritative local gates, Opus review rounds to
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
- the mandatory negative generated-build test rejects a named nested handler
  retaining `NonSend` state with `SIFR-RUST-CB-0001` before Cargo probing;
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

### certification_9 through certification_13: Cargo and Ecosystem

Implement the rows in numeric order as separate PRs. `certification_11` must
land before the tier-4 ecosystem rows so their locked/offline cache behavior is
part of the evidence rather than an assumption.

Native/proc-macro trust tests must prove rejection happens before the
untrusted script or macro can create a sentinel file. Ecosystem tests must use
checked-in lockfile resolution and temp packages; fetching during a validation
lane is a failure.

### certification_14: Track A Closeout and Stable Gate

This PR starts only when `certification_1` through `certification_13` are
merged.

- Re-run the row/fixture inventory and update documented counts.
- Replace the completion-time backstop in
  `scripts/check_sysroot_stdlib_resource_certification_gate.py` that currently
  requires at least one future-owned row, and update the guard's
  `--self-test` completed-matrix assertion in the same PR. Keep its supported
  stdlib-core invariants.
- Change `check_compatibility_matrix.py` so an unused
  `future-owned-by-separate-phase` category is valid after all current
  deferrals resolve; still reject unknown categories and invalid rows.
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
