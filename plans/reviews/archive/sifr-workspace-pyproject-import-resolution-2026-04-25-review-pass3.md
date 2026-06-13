# External Review (pass 3): Sifr Workspace Resolution Via `pyproject.toml`

Reviewer: external review pass
Review date: 2026-04-25
Inputs reviewed:

- `issues/sifr-workspace-pyproject-import-resolution-2026-04-25.md` (source issue, revised)
- `issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md` (phase plan, revised)
- `issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25-execution.md` (execution checklist, revised)
- `internal_docs/roadmap.md` (Phase 31.6 row)
- `reviews/sifr-workspace-pyproject-import-resolution-2026-04-25-review-pass2.md` (prior NOT READY findings)
- Spot checks against `crates/sifr_driver/src/project/assembly.rs`, `crates/sifr_driver/src/build/{materialize,project_codegen,entrypoint,api}.rs`, `crates/sifr_driver/src/test_runner/{orchestrator,artifacts,execution}.rs`, `crates/sifr/tests/verification/project/`, `verification/suites/manifest.json`, `audits/leetcode/helpers/`

The review below is organized as: 1) blocking findings, 2) nonblocking improvements, 3) final verdict.

---

## 1. Blocking Findings

### Pass-2 B1 — Resolved

The phase plan now contains an explicit "Dotted Module Materialization Model" section ([phase plan lines 128-156](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:128)) that:

- commits to the nested Rust module tree option (option 1 from pass-2): `helpers.list_node` materializes as `src/helpers/mod.rs` (containing `pub mod list_node;`) plus `src/helpers/list_node.rs`, with `src/main.rs` declaring only the top-level `mod helpers;`;
- keeps the canonical Sifr module ID as the dotted import string and routes `ProjectLowering`, compile order, export collection, and `support_modules` cache keys through that ID;
- names the affected files: [`crates/sifr_driver/src/project/assembly.rs`](crates/sifr_driver/src/project/assembly.rs), [`crates/sifr_driver/src/build/materialize.rs`](crates/sifr_driver/src/build/materialize.rs), [`crates/sifr_driver/src/build/project_codegen.rs`](crates/sifr_driver/src/build/project_codegen.rs), [`crates/sifr_driver/src/test_runner/artifacts.rs`](crates/sifr_driver/src/test_runner/artifacts.rs), and [`crates/sifr_driver/src/test_runner/execution.rs`](crates/sifr_driver/src/test_runner/execution.rs) (each verified to exist on disk);
- requires a regression that exercises `from helpers.list_node import ListNode` end-to-end through `check`, `emit`, `build`, `run` and asserts the on-disk Rust shape (`mod helpers;`, `src/helpers/mod.rs`, `src/helpers/list_node.rs`);
- requires a deterministic diagnostic when both `helpers.sifr` and `helpers/list_node.sifr` resolve in the same graph (allocated as `SIFR-WORKSPACE-0103`);
- splits the work cleanly across waves: WS3 lands the shared layout helper plus unit tests; WS4 wires the helper into `assemble_project_main_rs` and `materialize.rs` and adds the cache regression.

The materialization decision is now committed and testable. B1 is resolved.

### Pass-2 B2 — Resolved

WS5 ([phase plan lines 313-349](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:313)) now scopes workspace fixtures to `crates/sifr/tests/verification/project/<case_id>/` and registers them in [`verification/suites/manifest.json`](verification/suites/manifest.json) — the same home as the existing `multi_module_run` and `missing_import_reports_error` cases. The implementation note at [phase plan line 339](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:339) explicitly forbids the flat `crates/sifr/tests/e2e/pass` and `fail` directories with the harness rationale spelled out. Acceptance criteria pin a concrete invocation (`scripts/run_verification_suites.py --suite project` or repo-equivalent). The four cases (workspace pass, ambiguous, malformed pyproject, unresolved) are listed. B2 is resolved.

No new blocking findings.

---

## 2. Nonblocking Improvements

### N1. `SIFR-WORKSPACE-0103` wording ambiguity

The namespace/file collision diagnostic ([phase plan line 271](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:271)) reads `module '<module>' conflicts with namespace module '<parent>'`. In the canonical example — `helpers.sifr` plus `helpers/list_node.sifr` — calling `helpers` the "namespace module" reads backward (the file is the leaf module; the directory is the namespace). Reword to make both sides concrete, e.g. `module '<dotted_id>' resolves to file '<path_to_leaf>' but parent name '<parent>' is also a module file '<path_to_collision>'; package directories are not supported in this phase`. Implementer can pick the exact phrasing as long as both the file and the colliding parent path appear in the diagnostic.

### N2. Pin pilot artifact paths in WS5

WS5 says "regenerate pair scan and full corpus artifacts" but does not name the output files. The source issue ([lines 233-234](issues/sifr-workspace-pyproject-import-resolution-2026-04-25.md:233)) names `verification/leetcode/leetcode_pair_diff_scan_<YYYYMMDD>.json` and `verification/leetcode/full_corpus_current_results_<YYYYMMDD>_workspace_pilot.json`. Copy those names into WS5 so the pilot PR cannot drift on artifact location.

### N3. WS3 snapshot harness should be named explicitly

WS3 acceptance criteria ([phase plan line 284](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:284)) calls for "Failure snapshots cover every new user-facing diagnostic, including diagnostic code and URL". The repo uses both `insta` and the verification-suites diagnostic-format snapshots; nominate which harness owns these snapshots so the WS3 reviewer is not left guessing. The verification-suites manifest already supports `diagnostic_formats: ["human", "json", "compact"]` for the new fail cases — that path covers the URL+code surfacing automatically and is the simpler choice.

### N4. Implementation checklist is missing two items already required by the body

The "Implementation Checklist" at the bottom of the phase plan ([lines 422-437](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:422)) does not have explicit lines for:

- "WS1: `resolve_compilation_mode` returns an error-carrying type so workspace parse failures cannot be silently swallowed" (the no-fallback hardening from pass-2 N17, captured in the WS1 body but not in the checklist), and
- "WS2: `test_runner/orchestrator.rs` adopts the entry-parent-only `ModuleResolver` with no scope change" (captured in the WS2 body but not in the checklist).

Add two lines so the per-wave validation in the execution file has matching checklist items in the phase plan.

### N5. Layout helper ownership and home

The plan refers to "a shared Rust module layout helper in the driver" ([phase plan line 137](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:137)) but does not name a file. Suggest fixing the home now (e.g., `crates/sifr_driver/src/build/rust_module_layout.rs` or under the new `crates/sifr_driver/src/workspace/` module) so that the WS3 PR does not relitigate placement. The helper is consumed from both `build/*` and `test_runner/*`, so a shared `build/` location is the natural fit.

### N6. WS4 must also assert the cache key changes when sources reorder *changes* resolved content

WS4 acceptance ([phase plan lines 309-311](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:309)) covers two cases:

- dotted helper content change → cache key changes;
- inert source-root reorder (no resolved content change) → cache key stable.

Add a third assertion: a source-root reorder that *changes which file resolves* (e.g., a shadowing helper in a higher-priority root that supplants a lower-priority one) → cache key changes. Otherwise the resolver-determined identity is not actually proven to flow into the cache key. This is a one-liner in the existing test plan.

### N7. Document `audits/leetcode/helpers/` precondition once more

The phase plan correctly notes the directory may exist locally ([line 341](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:341)). It does, in fact, exist and is empty (verified). Promote that note into the WS5 "Planned Scope" line in the execution file so the pilot PR reviewer does not see "create directory" and second-guess the diff.

### N8. Roadmap link target

[`internal_docs/roadmap.md` line 56](internal_docs/roadmap.md:56) lists the 31.6 row as `ready_to_implement` and links the phase + execution files. Once WS6 closes, that row plus the dependency Mermaid will need touching. Non-blocking; record it now in the WS6 closure checklist so it is not missed at the end.

### N9. Validation invocation parity

The execution checklist's per-wave evidence rows ([execution lines 35-102](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25-execution.md:35)) reference selectors like `cargo test -p sifr_driver workspace -- --nocapture`. The phase plan acceptance criteria use the same form. If the eventual targeted selector ends up being slightly different (for example `cargo test -p sifr_driver -- workspace::` or the workspace module ends up named differently), keep the execution selector in lockstep with the phase plan — flag this as a per-PR sanity check rather than a doc edit, since the actual module path is decided by WS0.

### N10. PR sequencing exposure window is acceptable but worth one explicit line

The locked sequencing is `WS0 → WS2 → WS3 → WS1 → WS4 → WS5 → WS6` ([phase plan lines 383-389](issues/ad-hoc-sifr-workspace-pyproject-import-resolution-2026-04-25.md:383)). Between WS1 merging and WS4 merging, a hypothetical user adding `[tool.sifr]` and a workspace-source import would hit a clean unresolved-import diagnostic (because WS4 has not yet plumbed `WorkspaceRoot` into `RootedEntrypoint`/`api.rs`), not silent miscompilation. That is the expected and acceptable behavior, but worth one line in the PR sequencing section confirming it so a future reader does not re-derive the question.

---

## 3. Verdict

READY

Rationale: the two pass-2 blockers (B1 dotted-module materialization, B2 verification-suite fixture home) are both committed in writing with concrete file lists, regression assertions, and acceptance criteria that an implementer can land against without reopening design discussions. The pass-2 nonblocking guidance (N1 example cleanup, N2 optional `name`, N3 diagnostic codes/URLs, N4 native-manifest scope, N5 PR sequencing lock, N6 test-runner adoption, N7 cache regression detail, N8 `sifr test` deferral note, N9 TOML version pin, N10 path validation tests, N11 malformed-ancestor pyproject policy, N12 stale trie claim, N13 empty helpers dir, N14 source list lock, N15 per-wave validation, N16 manifest naming, N17 no-fallback hardening) is folded in. The remaining items above are all minor wording, checklist-completeness, and PR-prep hygiene — none of them require another review pass to clear.

Recommended path forward: fold N1, N2, N4, N5, N6, and N10 into the phase plan as a quick polish edit; record N3, N7, N8, N9 as PR-time notes; proceed with WS0.
