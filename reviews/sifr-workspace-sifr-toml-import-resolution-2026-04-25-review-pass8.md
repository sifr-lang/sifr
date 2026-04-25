# External Review (pass 8): Sifr Workspace Resolution Via `sifr.toml` (post-merge audit of pass-7 fixes)

Verdict: READY

Reviewer: external review pass 8 (post-merge audit after PR #1647 landed)
Review date: 2026-04-25
Branch reviewed: local `main` clean, even with `origin/main` at `2f284f77` (`Fix Sifr workspace closure review blockers (#1647)`)
Inputs reviewed:

- `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass5.md` (the corrected NOT READY pass-5)
- `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass7.md` (READY on the working branch)
- `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md` (phase plan, status `closed`)
- `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md` (execution checklist, status `closed`)
- Post-merge state of `crates/sifr_driver/src/project/{mod.rs,assembly.rs,rust_module_layout.rs}`, `crates/sifr_driver/src/test_runner/{artifacts.rs,execution.rs}`, `crates/sifr_driver/src/tests/{project_graph.rs,test_runner.rs}`, `scripts/run_all_tests.sh`, `internal_docs/roadmap.md`
- Local validation re-run on the merged tip: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_driver --lib`, targeted re-runs of the B1/B3 sentinels, plus the LeetCode workspace pilot via `cargo run -q -p sifr -- {check,emit} audits/leetcode/0021_merge_two_sorted_lists.sifr`

Scope of this review: confirm that pass-5 blockers B1, B2, and B3 remain fixed on `main` (not just on the feature branch reviewed in pass 7), that the validation gate in `scripts/run_all_tests.sh` actually exercises `sifr_driver` library tests, that the execution checklist and review artifacts are internally consistent post-merge, and that no new blocking issues surfaced after the merge of PR #1647.

---

## 1. Blocking Findings

None. No new blockers surfaced post-merge; B1, B2, and B3 from the corrected pass-5 review remain resolved on `main` at `2f284f77`.

---

## 2. Verification of pass-5 blockers on merged main

### B1 — Resolved (confirmed)

[crates/sifr_driver/src/tests/project_graph.rs:312-334](crates/sifr_driver/src/tests/project_graph.rs:312) — `test_assemble_project_main_rs_is_deterministic_against_hashmap_order` now asserts `"mod consumer;\nmod provider;\n\nfn main() {}\n"` (alphabetic top-level-namespace order produced by `top_level_module_declarations`). The sibling test `test_assemble_project_main_rs_declares_dotted_modules_by_top_level_namespace` at [project_graph.rs:336-350](crates/sifr_driver/src/tests/project_graph.rs:336) is intact and still guards the dotted-namespace dedupe behavior. Targeted re-run on `main`:

```
cargo test -p sifr_driver --lib test_assemble_project_main_rs_is_deterministic_against_hashmap_order
=> 1 passed; 0 failed
```

The full lib suite is now 97 passed / 0 failed, up from the 94 passed / 1 failed observed at b60ff461 in pass 5.

### B2 — Resolved (confirmed)

[scripts/run_all_tests.sh:102-103](scripts/run_all_tests.sh:102) adds `cargo test -p sifr_driver --lib` after the existing `cargo test -p sifr -- --skip test_e2e_pass` step. This runs ahead of the contract matrix, e2e pass suite, and hardening, so a regression on `sifr_driver` lib tests fails the lane before any heavier work runs. The execution checklist [issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:189-194](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:189) records both the direct `cargo test -p sifr_driver --lib` evidence (97 passed, 0 failed) and the quick/full lane re-runs that include this step. Pass-5 N7 (substring-filter `cargo test -p sifr_driver workspace`) is also addressed because the broader `--lib` gate now stands on its own.

### B3 — Resolved (confirmed)

The test-runner Rust materialization path on `main` uses the shared module-layout helpers exactly as the WS3/WS4 plan prescribes:

- [crates/sifr_driver/src/test_runner/artifacts.rs:5-20](crates/sifr_driver/src/test_runner/artifacts.rs:5) emits top-level `mod` declarations through `top_level_module_declarations`, which dedupes dotted IDs (e.g. `helpers.list_node` collapses to `mod helpers;`).
- [crates/sifr_driver/src/test_runner/execution.rs:54-79](crates/sifr_driver/src/test_runner/execution.rs:54) writes each support module file via `rust_module_file_path`, creating parents as needed; `helpers.list_node` lands at `src/helpers/list_node.rs` instead of a literal `helpers.list_node.rs`.
- [crates/sifr_driver/src/test_runner/execution.rs:81-109](crates/sifr_driver/src/test_runner/execution.rs:81) writes per-namespace `mod.rs` files through `namespace_module_files`, declaring each direct child as `pub mod <child>;`, mirroring the project-build assembly path at [crates/sifr_driver/src/build/materialize.rs:88-110](crates/sifr_driver/src/build/materialize.rs:88).
- [crates/sifr_driver/src/project/mod.rs:17-19](crates/sifr_driver/src/project/mod.rs:17) re-exports `top_level_module_declarations`, `namespace_module_files`, and `rust_module_file_path` together, keeping the helper surface unified across project-build and test-runner consumers.

Coverage on `main`:

- Unit-level: [crates/sifr_driver/src/tests/test_runner.rs:402-418](crates/sifr_driver/src/tests/test_runner.rs:402) (`test_compose_test_runner_lib_declares_dotted_modules_by_namespace`) asserts the lib emits `mod helpers;`, `mod math;`, and never `mod helpers.list_node;` for a mixed support-module set.
- End-to-end: [crates/sifr_driver/src/tests/test_runner.rs:49-85](crates/sifr_driver/src/tests/test_runner.rs:49) (`test_run_tests_resolves_dotted_local_support_modules`) writes a real `helpers/list_node.sifr` import, runs the test runner, and asserts success. Targeted re-run on `main`:

```
cargo test -p sifr_driver --lib test_run_tests_resolves_dotted_local_support_modules
=> 1 passed; 0 failed
```

Without the materialization fix, this end-to-end test would have failed on either the invalid `mod helpers.list_node;` declaration or a missing-file error during `cargo test`. Both pass on the merged tip.

---

## 3. Validation gate audit

`scripts/run_all_tests.sh` on `main` runs (in order, before any e2e or hardening): HIR maintainability guardrails, sifr_driver maintainability guardrails, `cargo test -p sifr -- --skip test_e2e_pass`, and `cargo test -p sifr_driver --lib` ([scripts/run_all_tests.sh:93-103](scripts/run_all_tests.sh:93)). The `cargo test -p sifr_driver --lib` step is unconditional — it runs in every lane (`quick`, `pr`, `nightly`, `release`) because it sits above the `if [[ -n "${CONTRACT_SUITES}" ]]` and `if [[ "${RUN_HARDENING}" == "1" ]]` gates. That matches the requirement from pass-5 B2.

Local re-run on the merged tip:

| Check | Result |
| --- | --- |
| `cargo fmt --check` | pass |
| `cargo clippy --workspace -- -D warnings` | pass |
| `cargo test -p sifr_driver --lib` | 97 passed, 0 failed |
| `cargo test -p sifr_driver --lib test_assemble_project_main_rs_is_deterministic_against_hashmap_order` | 1 passed |
| `cargo test -p sifr_driver --lib test_run_tests_resolves_dotted_local_support_modules` | 1 passed |
| `cargo run -q -p sifr -- check audits/leetcode/0021_merge_two_sorted_lists.sifr` | "no errors found" |
| `cargo run -q -p sifr -- emit audits/leetcode/0021_merge_two_sorted_lists.sifr` | top-level `mod helpers;` plus nested `// src/helpers/list_node.rs`, no flat `mod helpers.list_node;` |

I did not re-run the full `scripts/run_all_tests.sh` lane in this pass; the execution checklist already records both `--profile quick` (87.03s, 0 failures, includes the new `cargo test -p sifr_driver --lib`) and full PR profile (108.06s, hardening `variants = 28`, `failures = 0`, `blocking_failures = 0`) at [execution lines 193-194](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:193). The lane script on `main` is the one those measurements were taken against.

---

## 4. Internal consistency of execution checklist and review artifacts

- The execution checklist's `External Reviews` section ([execution lines 196-204](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:196)) accurately documents the historical pass-1 → pass-7 sequence, including the pass-5 supersedence note and the pass-6 supersedence call-out. Once this pass-8 artifact lands, the checklist should be amended in a follow-up edit to add the pass-8 row; absence of that row today is not a blocker because pass-8 had not run yet at merge time.
- The execution checklist's `Validation Evidence` section ([execution lines 175-194](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:175)) cleanly separates the original closure validation (lines 177-188) from the post-pass-5 blocker-fix validation (lines 189-194). The pass-5 NOT READY artifact is preserved in `reviews/`, so the audit trail (pass 5 NOT READY → fix → pass 7 READY → merge → pass 8 post-merge audit) is intact and traceable.
- Phase plan status is `closed` and matches [internal_docs/roadmap.md:56](internal_docs/roadmap.md:56) row 31.6 (`closed`). The WS3 affected-files note about the test-runner using the dotted helpers ([phase plan lines 152-153](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:152)) and the WS4 implementation note ([phase plan line 310](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:310)) are both honored on `main` (see B3 above).
- `audits/leetcode/0021_merge_two_sorted_lists.sifr` and the workspace `sifr.toml` continue to compile, run, and emit through the workspace pipeline — the pilot has not regressed.

---

## 5. Other observations (non-blocking)

These are pass-5 and pass-7 follow-up hygiene items that are still applicable on `main`. None gate closure; they are listed so the next hygiene tracker can pick them up.

- N1 (pass 5): WS4 has only one of three required cache-regression assertions in [crates/sifr_driver/src/tests/project_build_check.rs:145-170](crates/sifr_driver/src/tests/project_build_check.rs:145). The "inert source-root reorder leaves cache key unchanged" and "shadowing source-root reorder changes cache key" sentinels are still missing.
- N2 (pass 5): SIFR-WORKSPACE-0002/0003/0004 lack verification-suite snapshots in `human`, `json`, `compact`. They have unit-message coverage at [crates/sifr_driver/src/workspace/tests.rs:162-190](crates/sifr_driver/src/workspace/tests.rs:162) but no project-suite fixture.
- N3 (pass 5): Diagnostic codes are still recovered by string-prefix match in [crates/sifr_driver/src/diagnostics.rs:96-128](crates/sifr_driver/src/diagnostics.rs:96), so a future reword of the workspace error wording silently downgrades the diagnostic to `SIFR-BUILD-0001`.
- N4 (pass 5): The CLI human-format label for `SIFR-WORKSPACE-*` is still `error:` instead of `build error:` ([crates/sifr/src/main.rs:371-386](crates/sifr/src/main.rs:371)). Cosmetic.
- N5 (pass 5): Unresolved-import diagnostics still print a literal `./` segment when source root is `.` (see [crates/sifr/tests/verification/project/workspace_unresolved_import/baselines/check-human.stderr.txt](crates/sifr/tests/verification/project/workspace_unresolved_import/baselines/check-human.stderr.txt)).
- N6 (pass 5): Dead branch in `unresolved_import_message` at [crates/sifr_driver/src/project/discovery.rs:283-286](crates/sifr_driver/src/project/discovery.rs:283).
- N8 (pass 5): [internal_docs/architecture.md:592](internal_docs/architecture.md:592) still describes `[dependencies]` in `sifr.toml` as live behavior even though the phase explicitly defers dependency semantics ([internal_docs/sifr_workspace_design.md](internal_docs/sifr_workspace_design.md)).
- Pass-7 latent shadow note: if both a bare `helpers` and a dotted `helpers.list_node` ever appear in `support_module_names`, [crates/sifr_driver/src/test_runner/execution.rs:54-79](crates/sifr_driver/src/test_runner/execution.rs:54) would emit `src/helpers.rs` while [test_runner/execution.rs:81-109](crates/sifr_driver/src/test_runner/execution.rs:81) emits `src/helpers/mod.rs`, triggering a same-named-module conflict from rustc. This mirrors an equivalent risk on the project-assembly path and is not a regression introduced by the WS3/WS4/PR-#1647 work; track in the same hygiene follow-up.

---

## 6. Verdict

READY. No blockers remain on `main` at `2f284f77`. B1, B2, and B3 from the corrected pass-5 review are resolved post-merge, the validation gate now exercises `sifr_driver` library tests on every lane, and the execution checklist plus review artifacts are internally consistent (post-merge audit trail: pass 5 NOT READY → PR #1647 fix → pass 7 READY → pass 8 post-merge audit). No further review rounds are required for the Sifr workspace resolution phase closure; the non-blocking notes above belong in a downstream hygiene tracker rather than another review pass on this phase.
