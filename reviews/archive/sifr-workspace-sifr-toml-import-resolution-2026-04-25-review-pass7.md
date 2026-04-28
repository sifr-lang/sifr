# External Review (pass 7): Sifr Workspace Resolution Via `sifr.toml` (post-pass-5 blocker fixes)

Verdict: READY

Reviewer: external review pass 7 (post-pass-5 blocker-fix audit)
Review date: 2026-04-25
Branch reviewed: `ad-hoc/sifr-workspace-pass5-fixes`
Base: `main` at `9a8a6f84` (post `Record Sifr workspace closure review rounds (#1646)`)
Inputs reviewed:

- `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass5.md` (the corrected pass-5 NOT READY artifact, authoritative)
- `reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass6.md` (treated as superseded — reviewed an earlier READY pass-5 draft)
- `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md` and `…-execution.md`
- Working-tree diff against `main` for: `crates/sifr_driver/src/project/mod.rs`, `crates/sifr_driver/src/test_runner/{artifacts.rs,execution.rs}`, `crates/sifr_driver/src/tests/{project_graph.rs,test_runner.rs}`, `scripts/run_all_tests.sh`, the execution checklist, the pass-5 review file
- Spot checks against `crates/sifr_driver/src/project/rust_module_layout.rs` (helpers wired into the test runner)
- Local validation re-run: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_driver --lib`, `cargo test -p sifr_driver test_run_tests_resolves_dotted_local_support_modules -- --nocapture`

Scope of this review: confirm whether B1, B2, and B3 from the corrected pass-5 review are fixed on the working branch, and sanity-check the execution-checklist additions and the supersedence note for pass 6. Per the user's instructions, only the review artifact was written; no source or doc files were modified.

---

## 1. Blocking Findings

None. B1, B2, and B3 from pass 5 are all resolved on this branch.

---

## 2. Verification of pass-5 blockers

### B1 — Resolved

[crates/sifr_driver/src/tests/project_graph.rs:333](crates/sifr_driver/src/tests/project_graph.rs:333) now asserts `"mod consumer;\nmod provider;\n\nfn main() {}\n"` (alphabetic top-level-namespace order), matching the WS3 `top_level_module_declarations` behavior at [crates/sifr_driver/src/project/rust_module_layout.rs:19-27](crates/sifr_driver/src/project/rust_module_layout.rs:19). The sibling test `test_assemble_project_main_rs_declares_dotted_modules_by_top_level_namespace` at [project_graph.rs:336-350](crates/sifr_driver/src/tests/project_graph.rs:336) is still in place and continues to guard the dotted-namespace dedupe behavior. Pass 5's "preferred direction" (update the assertion rather than revert assembly) was followed.

Verified by running `cargo test -p sifr_driver --lib`: 97 passed, 0 failed (was 94 passed / 1 failed at b60ff461).

### B2 — Resolved

[scripts/run_all_tests.sh:102-103](scripts/run_all_tests.sh:102) now adds:

```
echo "Running sifr_driver library tests"
cargo test -p sifr_driver --lib
```

That is the exact gate pass 5 prescribed (`cargo test -p sifr_driver`, narrowed safely to `--lib` since integration-test wiring isn't present for this crate). With B1 fixed and this gate in place, the same regression class — a stale assembly assertion — would now fail the local lane on the next iteration.

The execution checklist at [issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:189-192](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:189) records direct `cargo test -p sifr_driver --lib` evidence (97 passed, 0 failed) plus quick and full `run_all_tests.sh` runs that explicitly note the `cargo test -p sifr_driver --lib` step is included. This also addresses pass-5 N7 (the substring-filter `cargo test -p sifr_driver workspace` evidence row no longer stands alone).

### B3 — Resolved

The test-runner Rust materialization now uses the same module-layout helpers WS3/WS4 prescribed:

- [crates/sifr_driver/src/test_runner/artifacts.rs:10-14](crates/sifr_driver/src/test_runner/artifacts.rs:10) emits top-level `mod` declarations through `top_level_module_declarations`, dropping invalid `mod helpers.list_node;` lines for dotted IDs.
- [crates/sifr_driver/src/test_runner/execution.rs:54-79](crates/sifr_driver/src/test_runner/execution.rs:54) writes each support module file via `rust_module_file_path`, so `helpers.list_node` lands at `src/helpers/list_node.rs` (creating parents as needed) instead of the literal `helpers.list_node.rs` flat file.
- [crates/sifr_driver/src/test_runner/execution.rs:81-109](crates/sifr_driver/src/test_runner/execution.rs:81) writes the per-namespace `mod.rs` files via `namespace_module_files`, declaring each direct child as `pub mod <child>;`, mirroring the project-build assembly path.
- [crates/sifr_driver/src/project/mod.rs:17-19](crates/sifr_driver/src/project/mod.rs:17) re-exports `top_level_module_declarations` alongside `namespace_module_files` and `rust_module_file_path`, keeping the helper surface unified.

Coverage:

- Unit-level: [crates/sifr_driver/src/tests/test_runner.rs:402-418](crates/sifr_driver/src/tests/test_runner.rs:402) (`test_compose_test_runner_lib_declares_dotted_modules_by_namespace`) confirms the lib emits `mod helpers;` once and never `mod helpers.list_node;` for a mixed `["helpers.list_node", "helpers.tree_node", "math"]` set.
- End-to-end: [crates/sifr_driver/src/tests/test_runner.rs:49-85](crates/sifr_driver/src/tests/test_runner.rs:49) (`test_run_tests_resolves_dotted_local_support_modules`) is the exact pass-5 scenario — a `helpers/list_node.sifr` import from a sibling `test_*.sifr` — and it passes locally (`cargo test -p sifr_driver test_run_tests_resolves_dotted_local_support_modules -- --nocapture` → 1 passed). Without the materialization fix this test would have hit either an invalid-Rust `mod helpers.list_node;` error or a missing-file error.

Implementation aligns with pass-5's preferred resolution direction (use the helper rather than reject dotted support IDs); the plan does not need amendment.

---

## 3. Local validation re-run

Run on the working branch:

| Check | Result |
| --- | --- |
| `cargo fmt --check` | pass |
| `cargo clippy --workspace -- -D warnings` | pass |
| `cargo test -p sifr_driver --lib` | 97 passed, 0 failed |
| `cargo test -p sifr_driver test_run_tests_resolves_dotted_local_support_modules -- --nocapture` | 1 passed |

I did not re-run the full `scripts/run_all_tests.sh` lane during this pass; the execution checklist records both `--profile quick` (87.03s, 0 failures) and full (108.06s, 0 failures, 28 hardening variants) runs at [execution lines 191-192](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:191), and the lane scripts on the branch include the new `cargo test -p sifr_driver --lib` step.

---

## 4. Other observations

- **Pass-5 review file**: the working tree replaces the old READY pass-5 with the corrected NOT READY pass-5 in place. That matches the user's framing that the corrected pass 5 is authoritative. The execution checklist `External Reviews` section ([execution lines 202-203](issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md:202)) now correctly documents the pass-5 supersedence and explicitly marks pass 6 as superseded because it reviewed the earlier READY draft. Both updates are accurate and non-controversial.
- **Pass-5 non-blocking notes N1–N6, N8**: out of scope for this audit and not addressed on this branch (they were never blockers). They remain valid follow-up hygiene items: N1 (missing two of three WS4 cache regression assertions), N2 (verification-suite coverage gaps for SIFR-WORKSPACE-0002/0003/0004), N3 (string-prefix diagnostic-code recovery), N4 (`error:` vs `build error:` label), N5 (literal `./` in unresolved-import message), N6 (dead branch in `unresolved_import_message`), N8 (architecture-doc bullet still describes `[dependencies]` as live). None gate closure; track in a hygiene follow-up.
- **Latent shadow on test-runner src layout (informational, not blocking)**: if both a bare `helpers` and a dotted `helpers.list_node` ever appear in `support_module_names`, `execution.rs` would emit both `src/helpers.rs` (from `rust_module_file_path("helpers")`) and `src/helpers/mod.rs` (from `namespace_module_files`), which `rustc` rejects with a same-named-module ambiguity. This mirrors the equivalent risk on the project-assembly path and is not a regression introduced by this fix; leaving the note here for the hygiene tracker.

---

## 5. Verdict

READY. B1, B2, and B3 from the corrected pass-5 review are resolved on `ad-hoc/sifr-workspace-pass5-fixes`. No further review rounds are required to land the post-pass-5 blocker-fix work; the non-blocking pass-5 notes (N1–N6, N8) should be tracked as follow-up hygiene rather than re-reviewed here.
