# M3 Workspace Session Data Model — Review

## 1. Blocking findings

**None.** The seven M3 acceptance criteria are all met. The M1 guardrail, file-size guardrail, `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_frontend workspace_session`, and `cargo test -p sifr_frontend` all pass on the working tree. M3 is purely additive: no caller of `FrontendContext::load_project`, `FrontendContext::load_single_file`, or `FrontendContext::load_project_with_provider` was changed (`crates/sifr_analysis/src/host/implementation.rs:36/41`, `crates/sifr_frontend/src/query_diagnostics.rs:537/553/595/642`, `crates/sifr_frontend/src/bin/frontend_query_bench.rs:296/308`, `crates/sifr_lint/src/engine.rs:252`, `crates/sifr_lint/src/rules/large_parameter_list.rs:96` — all unchanged), so CLI and existing frontend behavior are preserved by construction. `sifr_analysis::AnalysisSnapshot` is also unchanged (M4's migration target), so analysis queries remain on the existing path.

## 2. Non-blocking findings

### 2.1 Duplicate M3 row in the planning issue
`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:12-13` adds two near-identical M3 rows in a single diff. Deduplicate before PR — the second row adds no new info.

### 2.2 M3 doc validation list disagrees with the execution tracker
- `internal_docs/typescript_go_architecture_transfer_m3_workspace_session.md:49-59` lists 7 items including `cargo test -p sifr_analysis`, `cargo test -p sifr_lsp`, the M1 guardrail script, and the package-manager guardrail.
- The execution tracker (`issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md:30-32`, mirrored in `ad-hoc-typescript-go-compiler-architecture-transfer.md:28-32`) lists only the 5 the user reported. Pick one source of truth before PR. The M3 doc's "Validation" section correctly notes that the full quick gate remains required, so either bring the tracker up to the doc's list or trim the doc to the focused subset.

### 2.3 M3 doc does not list the spec's other minimum-validation items
The phase spec (`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:905-914`) requires `git diff --check` and `cargo test -p sifr -- --skip test_e2e_pass`. The M3 doc's "Validation" section omits both; the tracker omits them too. Worth running both before PR; trivial to add.

### 2.4 `upsert_overlay` does not auto-reload
`crates/sifr_frontend/src/workspace_session.rs:166-178` records the overlay and bumps the revision, but does not rebuild the `FrontendContext`. Callers must call `reload()` to see the change reflected in the snapshot's `source_map` / `module_graph`. The existing tests do this correctly (`workspace_session.rs:271`), but the M3 doc's "Session Model" section should call this contract out so a future caller does not assume the snapshot reflects the latest `upsert_overlay`.

### 2.5 `snapshot()` takes `&mut self` to bump `next_snapshot_id`
`crates/sifr_frontend/src/workspace_session.rs:188-204` requires `&mut self` because of `self.next_snapshot_id += 1`. This is the right choice for M3's serialized model (no concurrent access), but the M3 doc does not note the constraint. M11 (scheduler) will need to revisit this (either `Cell<u64>` / `AtomicU64` or a `&mut self` boundary). A one-line note in the M3 doc would prevent surprise.

### 2.6 `open_single_file` and `open_project` are asymmetric
`open_project` calls `reload()` (`workspace_session.rs:87-91`); `open_single_file` directly assigns `self.context` via `FrontendContext::load_single_file` (`workspace_session.rs:108-113`). Both work, but the asymmetry means a future contributor adding a side effect to `reload()` (e.g., dependency capture) will silently skip it for single-file. Consider routing through `reload()` for both, or documenting why single-file skips it.

### 2.7 Single-file `SourceDependency` list is always empty
`workspace_session.rs:159` clears `source_dependencies` on every single-file `reload()`. The spec says "session owns... provider dependency records" — for project targets this is satisfied, for single-file it is an empty `Vec`. This is the correct behavior (the single-file path doesn't generate tracked reads), but the M3 doc should state it so M6 doesn't assume single-file sessions have dependencies to consume.

### 2.8 `WorkspaceSnapshot` has `Option<SourceMapView>` / `Option<ModuleGraphView>`
`workspace_session.rs:66-67` wraps both in `Option<...>` because `self.context` is `Option<FrontendContext>`. The public constructors (`open_project`, `open_single_file`) always populate `context`, so the `Option` is purely defensive against future constructors. Either drop the `Option` and add a `debug_assert!(self.context.is_some())` in `snapshot()`, or document that `None` is reserved for un-reloaded sessions.

### 2.9 `WorkspaceSession::context()` is public
`workspace_session.rs:221-224` exposes a read-only handle to the underlying `FrontendContext`. This makes M3 inspectable but bypasses the snapshot model (callers can mutate the session and observe the live context). M4 will likely need to restrict this when snapshots become the canonical handle. Worth a TODO or note in the M3 doc.

### 2.10 `single_file_input` is a slight misnomer
`workspace_session.rs:226-240` builds a `FrontendInput` from session state, not "the single-file input". Consider `build_single_file_input` for clarity.

### 2.11 `TempProject` doesn't include `process::id()` in the path
`workspace_session.rs:331-339` uses `as_nanos()` for uniqueness. The M2 test (`source_provider.rs:431-438`) includes both PID and nanos, which is more defensive against concurrent CI. Astronomically unlikely to collide, but the M2 pattern is the safer template.

## 3. Missing validation or test coverage

All observable in the working tree, all small.

### 3.1 No test exercises `remove_overlay`
`workspace_session.rs:180-186` adds the method, but no test asserts that an upserted overlay can be removed and is then absent from a subsequent snapshot. A trivial addition: upsert, snapshot, remove, snapshot, assert the second snapshot's `overlays` is empty.

### 3.2 No test asserts `compiler_options` or `package_config_identity` on the snapshot
The snapshot exposes both (`workspace_session.rs:68-69`), but the two existing tests don't assert their values. A one-line assertion that `snapshot.package_config_identity.workspace_root == Some(root.root)` and `snapshot.compiler_options.mode == FrontendMode::ProjectEntrypoint` would prove the snapshot freezes them.

### 3.3 No test asserts `snapshot.revision` matches `session.revision()`
`workspace_session.rs:194` freezes `revision: WorkspaceRevision` into the snapshot, and `revision()` (`:206-209`) exposes the session's revision. No test asserts the two are equal, nor that a reload bumps the revision. Worth one assertion.

### 3.4 No test asserts the revision bump on `upsert_overlay` / `remove_overlay`
`workspace_session.rs:177, 183` both increment `self.revision.0`. No test covers the increment contract. A test that asserts `session.revision().as_u64()` is `1` after one `upsert_overlay` and `2` after one `reload()` would prove it.

### 3.5 No test for the `cache_registry` snapshot field
Currently always `(0, 0)` since the field is a placeholder. A test that asserts `snapshot.cache_registry.parse_generation == 0` and `hir_generation == 0` would document the placeholder contract, even if trivial.

### 3.6 M3 doc's validation list includes `cargo test -p sifr_analysis` and `cargo test -p sifr_lsp`
The user-reported validation did not run them. Both crates are downstream consumers of `sifr_frontend`, so a regression there would be silent. Recommend running before PR.

## 4. Approval verdict

**M3 is APPROVED for PR** after the listed validation. The data model matches the planning issue's M3 scope (`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:587-604`): `WorkspaceSession` owns overlays, tracked dependencies, the last `FrontendContext`, revision + snapshot-id counters, compiler options, package/config identity, and cache-registry handles; `WorkspaceSnapshot` freezes a `Clone + Debug + PartialEq + Eq` inspectable value; CLI, analysis, and LSP are unchanged. The two new tests pass, clippy is clean, fmt is clean, the M1 guardrail still passes (it checks for the literal strings `WorkspaceSession` and `WorkspaceSnapshot` in the M1 doc, which are present), and the file-size guardrail reports PASS at 2021 files (the new `workspace_session.rs` is 355 lines, well under the 900-line cap). M1 doc's "Future Milestone Update Obligations" (`internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:170-172`) requires M3 to move overlay lifecycle and tracked dependency records into `WorkspaceSession` snapshots — that obligation is satisfied.

Before PR, the two cheap fixes that will save a reviewer pass:
- Deduplicate the M3 row in `ad-hoc-typescript-go-compiler-architecture-transfer.md:12-13`.
- Reconcile the M3 doc's validation list with the execution tracker (or vice versa).

Optional but valuable: add the small tests in §3.1-3.4, and run `cargo test -p sifr_analysis` and `cargo test -p sifr_lsp` as the M3 doc already calls for. Everything else in §2/§3 is follow-up-worthy but not required to merge.
