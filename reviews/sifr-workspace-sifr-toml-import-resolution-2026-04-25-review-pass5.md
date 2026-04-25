# External Review (pass 5): Sifr Workspace Resolution Via `sifr.toml`

Reviewer: external review pass
Review date: 2026-04-25
Inputs reviewed:

- `issues/sifr-workspace-sifr-toml-import-resolution-2026-04-25.md` (source issue, status open)
- `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md` (phase plan, status closed)
- `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md` (execution checklist, status closed, all WS0-WS6 merged)
- `internal_docs/roadmap.md` (Phase 31.6 row, status `closed`)
- `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass4.md` (READY)
- Spot checks against the merged tree:
  - `crates/sifr_driver/src/workspace/{mod.rs,tests.rs}`
  - `crates/sifr_driver/src/project/{discovery.rs,rust_module_layout.rs,assembly.rs}`
  - `crates/sifr_driver/src/diagnostics.rs` (`SIFR-WORKSPACE-XXXX` mapping)
  - `crates/sifr_driver/src/build/entrypoint.rs` (workspace-aware cache tests)
  - `crates/sifr_driver/src/tests/project_build_check.rs` (`test_cached_project_invalidates_when_workspace_helper_changes`)
  - `crates/sifr/tests/verification/project/` (six fixtures including the four workspace cases)
  - `sifr.toml` at repo root, `audits/leetcode/helpers/list_node.sifr`, `audits/leetcode/0021_merge_two_sorted_lists.sifr`
  - `verification/leetcode/leetcode_pair_diff_scan_20260425.json`, `verification/leetcode/full_corpus_current_results_20260425_workspace_pilot.json`, `verification/leetcode/full_corpus_current_results_20260425_workspace_closure.json`

This is a post-merge confirmation review. Pass 4 returned READY; the phase landed on 2026-04-25 across PRs #1639-#1645 and the roadmap row is `closed`. The review below verifies that the merged tree honors the design and that pass-4 nonblockers were either resolved before merge or carried forward as intentional follow-ups. Structure: 1) blocking findings, 2) post-merge confirmations, 3) nonblocking observations carried forward, 4) verdict.

---

## 1. Blocking Findings

None.

The pass-2 blockers (B1 dotted-module materialization, B2 verification-suite fixture placement) are resolved in the merged tree. The native `sifr.toml` target is internally consistent across the source issue, phase plan, execution checklist, and roadmap. No regression of pass-3 or pass-4 contract claims was observed.

---

## 2. Post-Merge Confirmations

### Implementation matches the design

- Workspace discovery module exists at `crates/sifr_driver/src/workspace/` with `mod.rs` and `tests.rs`, split per pass-2 and WS6 maintainability guardrail. `mod.rs` is 182 lines and `tests.rs` is 238 lines.
- `crates/sifr_driver/src/project/rust_module_layout.rs` exists at the path fixed by the phase plan (pass-4 N6).
- `ModuleResolver` and `ResolvedModule` live in `crates/sifr_driver/src/project/discovery.rs` and consume the workspace context per WS2 / WS3 design.
- Diagnostic codes `SIFR-WORKSPACE-0001` through `0004` (parse/config) and `0101` through `0103` (resolution) are emitted from `crates/sifr_driver/src/diagnostics.rs:96-128` with the documented `https://sifr.dev/docs/errors/<CODE>` URL form.
- Cache-key regression for workspace helper content lives at `crates/sifr_driver/src/tests/project_build_check.rs:146` (`test_cached_project_invalidates_when_workspace_helper_changes`), and adjacent tests in `crates/sifr_driver/src/build/entrypoint.rs:436,476` cover the cache-reuse and cache-invalidation pair required by WS4.
- Pass-4 N8 nested-ancestor test exists: `crates/sifr_driver/src/workspace/tests.rs:227` — `test_closer_valid_manifest_ignores_farther_malformed_manifest`.
- Pass-4 N1 decision (accept and silently ignore unknown top-level tables and unknown nested keys) is locked in by `test_unknown_tables_and_keys_are_ignored` at `crates/sifr_driver/src/workspace/tests.rs:147`.

### Verification fixtures and pilot are in place

- `crates/sifr/tests/verification/project/` contains `workspace_dotted_helper_run`, `workspace_ambiguous_import`, `workspace_malformed_manifest`, and `workspace_unresolved_import` matching the WS5 case list.
- Repo-root `sifr.toml` declares `[source].roots = ["audits/leetcode", "."]` per the locked pilot configuration.
- `audits/leetcode/helpers/list_node.sifr` exists with the canonical `ListNode` model plus `nodeVal`, `nodeNext`, `hasNode`, `makeListNode`, and `listNodeToString` helpers.
- `audits/leetcode/0021_merge_two_sorted_lists.sifr` imports `from helpers.list_node import ...` (one matching import line).
- Pair scan `verification/leetcode/leetcode_pair_diff_scan_20260425.json` and the workspace-pilot/closure full corpus runs are both present, matching the execution checklist evidence (`PASS = 208`, `NO_ORACLE = 203`, no `CHECK_ERROR`/`RUN_ERROR`/`TIMEOUT`).

### Pass-4 nonblockers — disposition

- N1 (unknown tables/keys policy): resolved. Phase plan line 78 codifies "accepted and ignored in this slice" and `test_unknown_tables_and_keys_are_ignored` enforces it.
- N2 (source issue lacks codes/URLs): resolved by the pointer line at source issue line 139 routing canonical diagnostic codes to the phase plan WS3 contract.
- N3 (synthetic vs. pilot fixture tree mismatch): resolved by the explanatory line at source issue line 176.
- N4 (`sifr test` deferral missing from source issue Non-Goals): resolved by source issue line 52.
- N5 (stale "Review status" line on the phase plan): resolved by phase plan line 7 referencing pass-4 READY plus the WS6 local-gate date.
- N6 (layout helper home reaffirmed in WS4 scope): resolved. Phase plan WS4 scope cites `crates/sifr_driver/src/project/rust_module_layout.rs` directly.
- N7 (per-wave validation selectors firmness): resolved as a per-PR sanity check; the execution checklist now records the exact selectors that ran (e.g., `cargo test -p sifr_driver workspace -- --nocapture`, `cargo test -p sifr_driver discovery -- --nocapture`, `cargo test -p sifr_driver project_build_check -- --nocapture`).
- N8 (named nested-ancestor test): resolved (see confirmation above).

All eight pass-4 nonblockers are closed before merge or carried forward as expected.

---

## 3. Nonblocking Observations Carried Forward

These are post-merge observations for follow-up phases. They do not affect the closure of Phase 31.6.

### O1. Workspace diagnostic codes are derived by message-prefix matching

`crates/sifr_driver/src/diagnostics.rs:96-128` maps `CompileError` instances to `SIFR-WORKSPACE-XXXX` codes by `message.starts_with(...)` and `message.contains(...)` against the user-facing wording. This works today because every workspace error site emits the exact prefix the mapper looks for, and the unit tests in `crates/sifr_driver/src/tests/diagnostics.rs` lock the mapping. The fragility: a future copy edit to a diagnostic message string can silently drop the workspace code and fall through to the generic `SIFR-BUILD-0001`, with no compile-time signal. A future hardening phase should consider attaching the diagnostic code at the `CompileError` construction site (e.g., a `code: Option<&'static str>` field on `CompileError`) so the mapping is structural rather than textual. Not a regression; the locked tests catch the current strings.

### O2. `[package].name` in repo-root `sifr.toml` differs from the pilot example in the phase plan

`sifr.toml:2` sets `name = "sifr-workspace"`, while the phase plan "Target Configuration For This Slice" (lines 64) uses `name = "leetcode-fixtures"` as the example. This is fine — `package.name` has no semantic effect in this slice — but the design note `internal_docs/sifr_workspace_design.md` (and any future doc snapshots) should make clear that the actual repo-root manifest uses `sifr-workspace`, so future readers do not file a doc-vs-state mismatch. Pure hygiene; not a regression.

### O3. `sifr test` workspace-awareness remains the largest open follow-up

Both the source issue (line 52) and the phase plan (line 348) explicitly defer `sifr test` workspace-awareness to a later frontend-mode-parity follow-up. The execution checklist does not yet point to a created issue for that follow-up. Once a follow-up issue is filed, the source issue's "Non-Goals" line and `internal_docs/sifr_workspace_design.md` should backfill the issue link so the deferral is traceable. Not blocking; tracking improvement.

### O4. Reserved Cargo-inspired tables are accepted-and-ignored without a forward-compat warning

Per the WS0 policy, unknown top-level tables (`[workspace]`, `[[bin]]`, `[dependencies]`, `[profile.dev]`) are accepted and silently ignored. This is the right call for this slice. As soon as a dedicated package-management phase begins, these tables will start to take effect, and users who copied the "Proposed native shape" example from phase plan lines 99-122 into their own `sifr.toml` may see behavior change without warning. Consider, in the package-management phase entry criteria, an explicit migration note plus a one-cycle deprecation/info diagnostic when reserved tables are first encountered with non-trivial content. Not blocking; forward-looking.

### O5. The execution checklist's historical reviews list is informative but has no link to pass-5

`issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:190-195` lists historical pass 1 through pass 4. Add a pass-5 line once this artifact is filed, so the execution checklist and the reviews directory stay in sync. Pure bookkeeping.

---

## 4. Verdict

READY (post-merge confirmation).

Phase 31.6 is closed across the source issue, phase plan, execution checklist, and roadmap. The merged tree implements the native `sifr.toml` workspace concept, dotted module materialization, deterministic diagnostics with stable codes/URLs, cache invalidation for workspace helpers, verification-suite coverage, and the LeetCode helper pilot exactly as designed. All pass-4 nonblockers are resolved or recorded. No blocking findings; the observations in section 3 are forward-looking improvements for follow-up phases and do not require reopening Phase 31.6.
