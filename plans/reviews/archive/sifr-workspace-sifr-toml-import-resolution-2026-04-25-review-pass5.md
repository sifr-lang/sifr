# External Review (pass 5): Sifr Workspace Resolution Via `sifr.toml` (closure)

Verdict: NOT READY

Reviewer: external review pass 5 (post-merge closure audit)
Review date: 2026-04-25
Inputs reviewed:

- `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md` (phase plan, status: closed)
- `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md` (execution checklist, status: closed)
- `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass4.md` (pre-merge READY)
- Local `main` at `b60ff461` representing merged PRs [#1639–#1645](https://github.com/sifr-lang/sifr/pulls)
- Spot checks against `crates/sifr/src/main.rs`, `crates/sifr_driver/src/workspace/`, `crates/sifr_driver/src/project/`, `crates/sifr_driver/src/build/`, `crates/sifr_driver/src/test_runner/`, `crates/sifr_driver/src/tests/`, `crates/sifr/tests/verification/project/`, `verification/suites/manifest.json`, `audits/leetcode/`, `internal_docs/sifr_workspace_design.md`, `internal_docs/architecture.md`, `internal_docs/roadmap.md`, `verification/leetcode/`, `scripts/run_all_tests.sh`
- Local validation re-run: `cargo build --release -p sifr`, `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_driver workspace`, `cargo test -p sifr_driver --lib`, `cargo run -p sifr -- {check,run,emit} audits/leetcode/0021_merge_two_sorted_lists.sifr`

Note: a prior pass-5 artifact existed at this path that returned READY. This pass-5 supersedes it after running the full `sifr_driver` lib test suite, which the previous pass-5 did not. Findings below are reproducible on `main` at b60ff461.

---

## 1. Blocking Findings

### B1. `sifr_driver` lib test deterministically fails on merged main

[crates/sifr_driver/src/tests/project_graph.rs:312-334](crates/sifr_driver/src/tests/project_graph.rs:312) — `test_assemble_project_main_rs_is_deterministic_against_hashmap_order` deterministically fails on `main` at b60ff461:

```
left:  "mod consumer;\nmod provider;\n\nfn main() {}\n"
right: "mod provider;\nmod consumer;\n\nfn main() {}\n"
panicked at crates/sifr_driver/src/tests/project_graph.rs:333:5
test result: FAILED. 94 passed; 1 failed; 0 ignored
```

Root cause: WS3 (commit `b30d511d`, "Resolve workspace modules from source roots") changed [crates/sifr_driver/src/project/assembly.rs:16-34](crates/sifr_driver/src/project/assembly.rs:16) to declare top-level Rust modules through [`top_level_module_declarations`](crates/sifr_driver/src/project/rust_module_layout.rs:19-27), which orders names through a `BTreeSet` (alphabetic). The pre-existing assertion at line 333 still expects the legacy `compile_order` order (`provider, consumer`). The same diff added a sibling `test_assemble_project_main_rs_declares_dotted_modules_by_top_level_namespace` test but did not update the failing one.

Beyond the broken assertion, this test was the primary regression sentinel for "main.rs declarations are deterministic against HashMap order" — leaving the wrong expectation removes that protection until it is fixed.

### B2. Local validation gate does not exercise `sifr_driver` library tests, masking B1

[scripts/run_all_tests.sh:99-100](scripts/run_all_tests.sh:99) runs only `cargo test -p sifr -- --skip test_e2e_pass`. `cargo test -p sifr` does not execute the `sifr_driver` crate's library tests. The execution checklist's WS6 row at [line 181](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:181) ran `cargo test -p sifr_driver workspace -- --nocapture`, which substring-filters by `workspace` and silently excludes `tests::project_graph::*`. As a result, the WS6 evidence rows that claim "0 failures" ([execution lines 185-186](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:185)) do not actually exercise the bulk of `sifr_driver`'s lib tests.

This is the validation gap that allowed B1 to ship green. Closing the phase requires either widening `run_all_tests.sh` to run `cargo test -p sifr_driver` (or `cargo test --workspace`) and re-running the WS6 evidence row, or otherwise wiring `sifr_driver` lib tests into the merge-gate; otherwise the same class of regression will recur.

### B3. Unimplemented WS4 scope: test-runner Rust materialization does not use the dotted module layout helper

WS4 scope ([phase plan line 310](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:310)) and WS3 affected-files list ([lines 152-153](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:152)) explicitly require:

> `crates/sifr_driver/src/test_runner/artifacts.rs` and `crates/sifr_driver/src/test_runner/execution.rs`: use the same module layout helper for support modules that contain dotted IDs

Neither file was updated:

- [crates/sifr_driver/src/test_runner/artifacts.rs:9-13](crates/sifr_driver/src/test_runner/artifacts.rs:9) emits `mod {module_name};` directly for each `support_module_names` entry. For a dotted ID like `helpers.list_node` this writes `mod helpers.list_node;` — invalid Rust.
- [crates/sifr_driver/src/test_runner/execution.rs:53-64](crates/sifr_driver/src/test_runner/execution.rs:53) writes each support module to `src_dir.join(format!("{module_name}.rs"))`, which for the same dotted ID produces a literal `helpers.list_node.rs` flat file rather than `helpers/list_node.rs` plus `helpers/mod.rs`.

This is a real (not theoretical) regression because [crates/sifr_driver/src/project/discovery.rs:62-65](crates/sifr_driver/src/project/discovery.rs:62) and [discovery.rs:264-271](crates/sifr_driver/src/project/discovery.rs:264) now convert dots to nested paths even for the entry-parent-only resolver used by [test_runner/orchestrator.rs:48](crates/sifr_driver/src/test_runner/orchestrator.rs:48). Pre-WS3, `helpers.list_node` mapped to a non-existent flat `helpers.list_node.sifr` file and never reached the support-module list; post-WS3, a test directory containing `test_*.sifr` that imports `from helpers.list_node import …` plus a sibling `helpers/list_node.sifr` will resolve through the orchestrator and break the test runner. No existing test exercises that scenario, which is why CI did not flag it, but the scope item is explicit and the bug latent.

This must either be implemented (using `top_level_module_declarations` / `namespace_module_files` / `rust_module_file_path`) or the phase plan must be amended to remove the scope and the test-runner orchestrator must reject dotted support module IDs with a clear diagnostic until support lands. The current "neither" state contradicts the merged plan.

---

## 2. Non-blocking Notes

### N1. WS4 cache-regression coverage is one of three required cases

WS4 acceptance criteria ([phase plan lines 313-318](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:313)) call out three cache regression assertions:

1. dotted helper content change → cache key changes
2. inert source-root reorder → cache key unchanged
3. shadowing source-root reorder → cache key changes

Only (1) is implemented at [crates/sifr_driver/src/tests/project_build_check.rs:145-170](crates/sifr_driver/src/tests/project_build_check.rs:145) (`test_cached_project_invalidates_when_workspace_helper_changes`). (2) and (3) have no test. The current cache-key implementation at [crates/sifr_driver/src/build/materialize.rs:144-168](crates/sifr_driver/src/build/materialize.rs:144) is content-keyed (over `support_modules` BTreeMap, `main_rs`, Cargo.toml, stdlib set, crates set), so the desired behavior is plausibly satisfied — but no regression sentinel exists, which is what the acceptance criterion was protecting.

### N2. Verification-suite diagnostic coverage is incomplete for SIFR-WORKSPACE-0002/0003/0004

WS3 acceptance ([phase plan line 290](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md:290)):

> Verification-suite diagnostic snapshots cover every new user-facing diagnostic in `human`, `json`, and `compact` formats, including diagnostic code and URL.

The merged project suite covers SIFR-WORKSPACE-0001 ([workspace_malformed_manifest](crates/sifr/tests/verification/project/workspace_malformed_manifest)), 0101 ([workspace_unresolved_import](crates/sifr/tests/verification/project/workspace_unresolved_import)), and 0102 ([workspace_ambiguous_import](crates/sifr/tests/verification/project/workspace_ambiguous_import)). 0103 has unit coverage but no project-suite fixture. 0002 (`escapes the workspace root via '..'`), 0003 (`is not a directory under the workspace root`), and 0004 (`must be a relative non-empty path under the workspace root`) have unit-message coverage in [crates/sifr_driver/src/workspace/tests.rs:162-190](crates/sifr_driver/src/workspace/tests.rs:162) but no verification-suite snapshot in any of human/json/compact.

The unit test [crates/sifr_driver/src/tests/diagnostics.rs:39-68](crates/sifr_driver/src/tests/diagnostics.rs:39) (`test_workspace_resolution_errors_have_stable_codes_and_urls`) only exercises 0101, 0102, 0103 through `to_diagnostic`. The 0001-series codes are derived only by the prefix-dispatch in [crates/sifr_driver/src/diagnostics.rs:96-128](crates/sifr_driver/src/diagnostics.rs:96) and are not unit-tested for code+URL.

### N3. Workspace diagnostic codes are derived by string-prefix matching on the message

[crates/sifr_driver/src/diagnostics.rs:96-128](crates/sifr_driver/src/diagnostics.rs:96) recovers each `SIFR-WORKSPACE-XXXX` code by prefix-matching on `CompileError.message`. If the human-readable wording at [workspace/mod.rs:160-179](crates/sifr_driver/src/workspace/mod.rs:160) or [project/discovery.rs:273-325](crates/sifr_driver/src/project/discovery.rs:273) changes — even a casing or punctuation tweak — the diagnostic silently downgrades to `SIFR-BUILD-0001` and the URL drops to `https://sifr.sh/docs/errors/SIFR-BUILD-0001`. A more durable model would attach the code at the construction site (e.g., a `code` field on `CompileError` or a typed enum). Not blocking because the current strings are tested for one example each via the workspace 0101/0102/0103 unit, but the 0001-0004 series and any rewording of those messages have no guard.

### N4. CLI human-format label for workspace errors is `error:`, not `build error:`

`SIFR-WORKSPACE-XXXX` codes carry `CompilePhase::Build`, but [crates/sifr/src/main.rs:371-386](crates/sifr/src/main.rs:371) selects the label by `code.starts_with("SIFR-PARSE-")` etc. and falls through to severity-based `error` for `SIFR-WORKSPACE-*`. Cosmetic, but inconsistent with sibling diagnostic categories.

### N5. Unresolved-import diagnostic includes literal `./` when source root is `.`

[crates/sifr/tests/verification/project/workspace_unresolved_import/baselines/check-human.stderr.txt:1](crates/sifr/tests/verification/project/workspace_unresolved_import/baselines/check-human.stderr.txt:1) shows `…/workspace_unresolved_import/./missing/helper.sifr` because [workspace/mod.rs:145-149](crates/sifr_driver/src/workspace/mod.rs:145) normalizes empty source roots to `PathBuf::from(".")` and the resolver then `join`s that. Functionally correct and deterministic, but visually noisy. A `Path::components` clean-up before display would tidy this up.

### N6. Dead branch in `unresolved_import_message`

[crates/sifr_driver/src/project/discovery.rs:283-286](crates/sifr_driver/src/project/discovery.rs:283) handles the case where `workspace_paths.is_empty()`. `to_compile_error` is only invoked from the `resolver.has_workspace()` arms in `parse_import_closure_modules` (lines 379, 426), and a workspace always carries at least one source root (defaults to `["."]`), so this branch is unreachable. Either delete it or make the call sites consistent.

### N7. WS6 reduced-scope test selector hides the broader `sifr_driver` failure surface

The execution checklist row [line 181](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:181) `cargo test -p sifr_driver workspace -- --nocapture` filters by name substring — narrower than `cargo test -p sifr_driver`. Combined with B2, no merge-gate evidence row actually proves the full `sifr_driver` lib tests pass. Pair the broadened gate from B2 with a re-run of the full checklist before re-closing.

### N8. Architecture doc bullet still describes `[dependencies]` as live behavior

[internal_docs/architecture.md:592](internal_docs/architecture.md:592) keeps a legacy bullet ``**sifr.toml`:** project manifest with `[dependencies]` section specifying version ranges (semver)``. This phase explicitly defers `[dependencies]` semantics ([sifr_workspace_design.md:30-36](internal_docs/sifr_workspace_design.md:30)). Either drop the bullet or amend it to "reserved" so a reader doesn't expect dependency resolution to work today. Cosmetic.

---

## 3. Validation Reviewed

Run locally on `main` at `b60ff461`:

| Check | Result |
| --- | --- |
| `cargo build --release -p sifr` | pass (15.7s) |
| `cargo fmt --check` | pass |
| `cargo clippy --workspace -- -D warnings` | pass |
| `cargo test -p sifr_driver workspace` | 30 passed |
| `cargo test -p sifr_driver --lib` | **FAIL — 1 failure (B1)** |
| `cargo run -p sifr -- check audits/leetcode/0021_merge_two_sorted_lists.sifr` | "no errors found" |
| `cargo run -p sifr -- run audits/leetcode/0021_merge_two_sorted_lists.sifr` | exit 0, cache hit |
| `cargo run -p sifr -- emit audits/leetcode/0021_merge_two_sorted_lists.sifr` | nested `// src/helpers/list_node.rs` emitted, only `mod helpers;` at top level |

Closure-artifact spot checks:

- [verification/leetcode/full_corpus_current_results_20260425_workspace_closure.json](verification/leetcode/full_corpus_current_results_20260425_workspace_closure.json) summary matches checklist line 188: `case_count=411`, `PASS=208`, `NO_ORACLE=203`, no `CHECK_ERROR`/`RUN_ERROR`/`TIMEOUT`.
- [verification/leetcode/leetcode_pair_diff_scan_20260425.json](verification/leetcode/leetcode_pair_diff_scan_20260425.json) shows the 0021 pilot at `sifr_lines=9`, matching checklist line 147.
- [audits/leetcode/helpers/list_node.sifr](audits/leetcode/helpers/list_node.sifr) and migrated [audits/leetcode/0021_merge_two_sorted_lists.sifr](audits/leetcode/0021_merge_two_sorted_lists.sifr) are in place; root [sifr.toml](sifr.toml) declares `roots = ["audits/leetcode", "."]`.
- Verification fixtures [workspace_dotted_helper_run](crates/sifr/tests/verification/project/workspace_dotted_helper_run), [workspace_ambiguous_import](crates/sifr/tests/verification/project/workspace_ambiguous_import), [workspace_malformed_manifest](crates/sifr/tests/verification/project/workspace_malformed_manifest), [workspace_unresolved_import](crates/sifr/tests/verification/project/workspace_unresolved_import) exist and are registered in [verification/suites/manifest.json](verification/suites/manifest.json).
- [internal_docs/roadmap.md:56](internal_docs/roadmap.md:56) row 31.6 status is `closed`.

---

## 4. Verdict

NOT READY.

Three blockers must be resolved before re-closing:

- B1: fix the failing `sifr_driver` lib test by updating the assertion to the alphabetic top-level-namespace order (the preferred direction; reverting `assemble_project_main_rs` to compile_order would re-open the dotted-namespace dedupe behavior that WS3 deliberately introduced).
- B2: extend `scripts/run_all_tests.sh` (and the WS6 evidence row) to actually run `cargo test -p sifr_driver` (or `cargo test --workspace`), so this class of regression is caught locally next time.
- B3: implement the test-runner dotted module materialization called out in WS3/WS4, or formally amend the plan and add a hard diagnostic for dotted support modules until support lands.

Non-blocking notes N1–N8 should be tracked but do not gate closure.
