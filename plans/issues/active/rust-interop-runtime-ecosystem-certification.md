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
| `certification_1` | in progress | executable bridge-type roundtrips and nested dict conversion |
| `certification_2` | blocked | starts after `certification_1` merges |
| `certification_3` | blocked | starts after `certification_2` merges |
| `certification_4` | blocked | starts after `certification_3` merges |
| `certification_5` | blocked | starts after `certification_4` merges |
| `certification_6` | blocked | starts after `certification_5` merges |
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
- [ ] Run focused and authoritative local gates, Opus review rounds to
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

`certification_3` may use bridge-version 1 call-scoped callbacks. Any callback
behavior that truly requires the bridge-version 2 structural call contract
must be split into the later package-resource item rather than silently
expanding this row.

### certification_4 through certification_8: Runtime Resources and Views

Implement the rows in numeric order as separate PRs. Shared loopback harness
code may be introduced by `certification_4` in a focused verification helper
module and reused afterward; service-specific behavior and evidence remain in
the owning row.

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
