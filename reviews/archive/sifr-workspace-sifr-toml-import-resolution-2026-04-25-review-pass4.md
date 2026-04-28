# External Review (pass 4): Sifr Workspace Resolution Via `sifr.toml`

Reviewer: external review pass
Review date: 2026-04-25
Inputs reviewed:

- `issues/sifr-workspace-sifr-toml-import-resolution-2026-04-25.md` (source issue, native `sifr.toml` rewrite)
- `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md` (phase plan, native `sifr.toml` rewrite)
- `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md` (execution checklist, refreshed)
- `internal_docs/roadmap.md` (Phase 31.6 row)
- `reviews/sifr-workspace-pyproject-import-resolution-2026-04-25-review-pass2.md` (NOT READY)
- `reviews/sifr-workspace-pyproject-import-resolution-2026-04-25-review-pass3.md` (READY for prior pyproject-targeted plan)
- Spot checks against the live tree under `crates/sifr_driver/src/`, `crates/sifr/tests/verification/project/`, `verification/suites/manifest.json`, `audits/leetcode/helpers/`

The review below is organized as: 1) blocking findings, 2) nonblocking improvements, 3) final verdict.

---

## 1. Blocking Findings

None.

The pass-2 blockers (B1 dotted-module materialization, B2 verification-suite fixture placement) remain resolved under the native `sifr.toml` rewrite, and the target-switch from `pyproject.toml` / `[tool.sifr]` to native `sifr.toml` is internally consistent across all four active docs.

### Pass-2 B1 (dotted materialization) — still resolved

The phase plan's "Dotted Module Materialization Model" section ([phase plan lines 132-160](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:132)) survives the rewrite intact:

- canonical Sifr module ID stays the dotted import string;
- `ProjectLowering`, compile order, export collection, and `support_modules` cache keys remain keyed by the dotted ID;
- the shared Rust module layout helper is fixed at [`crates/sifr_driver/src/project/rust_module_layout.rs`](crates/sifr_driver/src/project/rust_module_layout.rs) (file does not yet exist on disk — to be added in WS3, as the phase plan now commits);
- `helpers.list_node` materializes as `src/helpers/mod.rs` plus `src/helpers/list_node.rs`, with `src/main.rs` declaring only `mod helpers;`;
- end-to-end regression through `check`, `emit`, `build`, `run` is required ([phase plan lines 156-160](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:156));
- namespace/file collisions are rejected by `SIFR-WORKSPACE-0103`.

WS3 lands the helper plus unit tests, WS4 wires it into materialization and adds the cache regression — split is preserved from the pass-3 baseline.

### Pass-2 B2 (verification-suite fixture placement) — still resolved

WS5 ([phase plan lines 318-356](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:318)) places workspace fixtures under `crates/sifr/tests/verification/project/<case_id>/` and registers them in [`verification/suites/manifest.json`](verification/suites/manifest.json), with an explicit prohibition on the flat `crates/sifr/tests/e2e/{pass,fail}` harness ([phase plan line 345](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:345)). The four cases (workspace pass, ambiguous, malformed `sifr.toml`, unresolved) are listed and the acceptance criteria pin a project-suite invocation. Source issue lines 163-174 mirror the same home.

### Native `sifr.toml` switch — internally consistent

- Source issue title, problem framing, suggested solution, validation rules, resolver order, diagnostics, test plan, and required artifacts all reference `sifr.toml` directly with no residual `pyproject.toml` or `[tool.sifr]` mention.
- Phase plan title, purpose, problem statement, target configuration, manifest model, validation contract, workstream scope, diagnostics, and quality contract all reference `sifr.toml` directly. Pyproject/`[tool.sifr]` are mentioned only in two non-goal/forward-compat lines ([phase plan lines 19, 54, 129, 330](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:19)) that explicitly defer them.
- No fallback, parser branch, resolver fork, or compatibility adapter for `pyproject.toml` is implied or scheduled.
- Diagnostic codes are consistently `SIFR-WORKSPACE-XXXX` with the documentation URL form required by the roadmap global rule.
- Validation contract treats malformed `sifr.toml` as a hard error and an empty `sifr.toml` as a valid workspace defaulting to `roots = ["."]`. WS1 makes `resolve_compilation_mode` error-carrying ([phase plan line 212](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:212)) so a malformed manifest cannot silently fall back to single-file mode — the no-fallback rule from pass-2 N17 is preserved.
- Resolver order (stdlib → entry-sibling → workspace sources in declaration order) and entry-sibling-always-wins ambiguity rule are stated identically in the source issue and the phase plan.
- The pilot source-root list is locked to `roots = ["audits/leetcode", "."]` ([phase plan lines 343-344](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:343)).
- Pilot artifact filenames (`leetcode_pair_diff_scan_<YYYYMMDD>.json`, `full_corpus_current_results_<YYYYMMDD>_workspace_pilot.json`) are pinned in both source issue and phase plan.
- Cache regression covers all three required cases (content change, inert reorder, shadowing reorder) at [phase plan lines 313-315](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:313).
- Implementation checklist now includes the WS1 error-carrying compilation-mode item and the WS2 test-runner adoption item that pass-3 N4 flagged ([phase plan lines 433, 435](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:433)).
- Roadmap row 31.6 ([`internal_docs/roadmap.md` line 56](internal_docs/roadmap.md:56)) links the renamed source issue and execution files, status `ready_to_implement`, title `Ad Hoc Sifr Workspace Resolution Via sifr.toml`. Old pyproject-named files are not referenced.
- Execution checklist sets per-wave validation rows for WS0–WS5 and keeps the WS6 final-gate validation list intact; per-wave evidence requirement from pass-2 N15 holds.

The phase remains implementation-ready under the native target.

---

## 2. Nonblocking Improvements

### N1. Phase plan "Proposed native shape" lists tables this slice does not validate

The phase plan's "Native `sifr.toml` Manifest Model" section ([phase plan lines 96-130](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:96)) shows a full proposed manifest including `[workspace]`, `[[bin]]`, `[dependencies]`, and `[profile.dev]`. The validation contract that follows ([phase plan lines 73-81](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:73)) only validates `[package]` and `[source]`, and line 130 says reserved tables "must not affect module resolution". What the contract does not say: if a user actually writes one of those tables today, does the parser

- accept and silently ignore unknown top-level tables (typical `serde` default with `#[serde(deny_unknown_fields)] = false`),
- validate the tables syntactically but reject unknown keys inside them, or
- hard-reject any table outside `[package]` and `[source]` until the dedicated package-management phase lands?

Pick one in WS0 before the parser is written, otherwise the first WS0 PR will rediscover the question. Recommend "accept and silently ignore unknown top-level tables and unknown nested keys, with a forward-compat note that future phases may tighten this", so users can write design-runway-shaped manifests today without forking parser behavior later.

### N2. Source issue diagnostic phrasing has no diagnostic codes / URLs

The source issue's Diagnostics section ([source issue lines 124-135](issues/sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:124)) lists user-facing wording but no `SIFR-WORKSPACE-XXXX` codes or `sifr.sh/docs/errors/<CODE>` URLs. The phase plan WS3 ([phase plan lines 268-275](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:268)) assigns codes and URLs cleanly. Mirror those into the source issue (or add a one-line note that the phase plan owns the canonical diagnostic codes) so a reviewer reading only the source issue does not assume codes are unspecified and break the roadmap global rule at line 28.

### N3. Source issue's verification fixture tree differs in shape from WS5 pilot

The source issue's verification-suite fixture sketch at [source issue lines 165-170](issues/sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:165) places `helpers/list_node.sifr` at the workspace root, while the WS5 LeetCode pilot lives at `audits/leetcode/helpers/list_node.sifr`. The two are intentionally different scenarios (synthetic verification fixture vs. real LeetCode pilot), and both compose with `roots = ["audits/leetcode", "."]`, but a one-line note in the source issue calling that out would prevent a reviewer from flagging it as a mismatch.

### N4. WS5 `sifr test` deferral is in the phase plan but not the source issue

[Phase plan line 346](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:346) explicitly defers `sifr test` workspace-awareness to a later frontend-mode-parity follow-up. The source issue Non-Goals ([source issue lines 44-51](issues/sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:44)) does not mention `sifr test` at all. Add a one-line non-goal so a reader of only the source issue knows the gap and does not file a frontend-parity regression against this slice.

### N5. Phase plan "Review status" line is stale

[Phase plan line 7](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:7) currently reads `Review status: pending refresh after switching implementation target to native sifr.toml`. Once this pass-4 review returns READY, flip that line to reference the pass-4 review path so the doc does not look mid-flight to the implementing engineer.

### N6. Layout helper home picked but not reaffirmed in WS4 / test-runner notes

The phase plan fixes the layout helper at `crates/sifr_driver/src/project/rust_module_layout.rs` ([phase plan line 140](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:140)) and WS3 owns it ([phase plan line 263](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:263)). WS4 ([phase plan line 302](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:302)) and WS3's test-runner note ([phase plan lines 152-153](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:152)) refer to "the shared layout helper" without re-citing the path. Inline the path one more time in the WS4 scope so the WS4 PR does not rediscover the placement question. Pure hygiene.

### N7. Execution checklist per-wave validation selectors are still tentative

Execution checklist per-wave evidence rows ([execution lines 35-102](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:35)) reference selectors like `cargo test -p sifr_driver workspace -- --nocapture` and `cargo test -p sifr_driver discovery -- --nocapture`. The actual Rust module names (`workspace`, `discovery`, `project_build_check`, etc.) are decided by WS0 / WS2 / WS4 implementation. The selectors will need a once-over after WS0 / WS2 / WS4 land so the execution checklist does not drift; flag this as a per-PR sanity check rather than a doc edit.

### N8. Risks section discusses ancestor-`sifr.toml` blast radius but should pin the test

[Phase plan / source issue Risks](issues/sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:215) state that a malformed ancestor `sifr.toml` deliberately fails Sifr invocations below it (acceptable because `sifr.toml` is Sifr-owned). WS0 acceptance ([phase plan lines 188-193](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:188)) covers the "malformed `sifr.toml` is a hard error when encountered before a closer valid workspace is found" case. Add an explicit nested-ancestor unit test where a closer valid `sifr.toml` is present and a farther-ancestor malformed `sifr.toml` is ignored — the wording is in the phase plan but the WS0 test list does not name that exact scenario distinctly. One-liner.

---

## 3. Verdict

READY
