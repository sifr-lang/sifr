## Findings

### P1 — must fix before elegant

1. **Missing matrix row: CLI behavior and exit codes.**
   World-Class Verification Standard line 51 enumerates "CLI behavior and exit codes" as a required compiler surface, but the target matrix has no row for it. The Discovery Snapshot already lists `sifr` integration tests `e2e`, `validation_contracts`, and `build_output_contracts`; the merge command already runs them, so the inventory exists in spirit but is not pinned in the matrix.

2. **Missing matrix row: project/workspace behavior.**
   Discovery Snapshot line 247 ships a `project_workspace` area with four suites (`frontend_mode_parity`, `phase23_graph_isolation`, `baselines`, `audit-fixtures`). The matrix omits this surface entirely, leaving the area's existing merge/nightly assignments uncodified.

3. **Missing matrix row: incremental / clean-rebuild / report-determinism equivalence.**
   Wave 8 is titled "Incremental, Determinism, and Performance Trend Evidence" and explicitly produces clean-vs-incremental equivalence checks, edit-run fixtures, and report determinism artifacts. The matrix has a Performance trends row but no Incremental/Determinism row, so Wave 8's two largest deliverables have no inventory pointer or minimum-content rule.

4. **Algorithmic compatibility row has no owning wave.**
   The matrix promises a "representative LeetCode/algorithm subset" in merge with a "taxonomy row required for each included problem/category" rule. But Wave 9's sub-PR list (lines 811–815) covers LSP / ecosystem / package / stdlib / runtime-platform only — algorithmic compatibility is not assigned. Closeout (line 967 acceptance criterion) likewise omits it.

5. **Snapshot rows have unpinned inventory locations.**
   Every other row cites a concrete path (`verification/areas/coverage_matrix/shipped_guarantees.json`, etc.). The HIR/name/type/CFG row says only "layer inventories owned by `core_language`, `sifr_lowering`, and `sifr_analysis`"; the Codegen snapshots row says "codegen IR/emitted-Rust stable construct inventory" — no file paths. Wave 5 also has no task to create these inventory files. Without paths, "every stable lowering/analysis contract has at least one snapshot" cannot be mechanized.

### P2 — polish

6. **Acceptance criterion (line 956) is awkward.** "every `every` target has an inventory-backed check that fails missing evidence" repeats "every" with shifting meaning.

7. **Local/CI parity row deviates from the inventory pattern.** Its "Source-of-truth inventory" is "emitted local and CI profile plans" — runtime artifacts, not a checked-in source of truth. Other rows point at static files; the asymmetry is mildly confusing.

8. **Field-name drift between matrix and Wave 4 catalog.** Matrix uses `machine_applicable`; Wave 4 task list (line 518) phrases it as "machine-application applicability" and Wave 4 validation phrasing is "machine-applicable in the diagnostic code catalog". Pick one and use it in both places.

## Required Edits

**E1. Insert a CLI surfaces row.** Place under the matrix between "First-party crates" and "Cargo features and targets" (or wherever fits the ordering — current ordering is by area-of-the-compiler, so here is natural):
```
| CLI behavior and exit codes | `sifr` integration test inventory (`e2e`, `validation_contracts`, `build_output_contracts`) and a documented CLI exit-code contract list | every documented CLI exit-code contract has a corresponding integration test | full CLI behavior matrix and broader exit-code scenarios | an exit-code contract without an integration test fails |
```

**E2. Insert a project/workspace row.** For example:
```
| Project/workspace behavior | `verification/areas/project_workspace` suite manifests (`frontend_mode_parity`, `phase23_graph_isolation`, `baselines`, `audit-fixtures`) | every shipped workspace/graph-isolation contract has a blocking suite row | broader project graph and workspace scenarios | workspace contracts without a suite row fail |
```

**E3. Insert an incremental / determinism row.** For example:
```
| Incremental and determinism equivalence | clean-vs-incremental contract list owned by Wave 8 plus `check_report_determinism.sh` and `check_sequential_parallel_equivalence.sh` outputs | every shipped cache/query/incremental behavior has an equivalence fixture; report and parallel determinism remain blocking | full edit-run scenario matrix and long-session repeated-build stress | an incremental contract without an equivalence fixture fails; nondeterministic output fails |
```

**E4. Assign algorithmic compatibility to a wave.** Either add `9.6 Algorithmic compatibility broader corpus` to the Wave 9 sub-PR list (line 811) with a matching task block, or move it into Wave 3 as a parser/semantic-adjacent corpus. Then add an acceptance-criteria line at line ~980 stating "Algorithmic compatibility taxonomy and corpus has profile-owned commands and structured evidence."

**E5. Pin snapshot inventory paths.** Replace the inventory cells for the two snapshot rows with concrete paths and add a Wave 5 task to create them. Suggested:
- HIR/name/type/CFG row: `verification/areas/core_language/data/lowering_layer_inventory.json` (or similar split by layer).
- Codegen snapshots row: `verification/areas/generated_code_quality/data/codegen_construct_inventory.json`.
- Add task to Wave 5 (before sub-PRs 5.2–5.7): "Create per-layer snapshot inventory files (HIR, name resolution, type/ownership, CFG, codegen IR, emitted Rust); each entry declares the stable contract and its snapshot id."

**E6. Rephrase the acceptance criterion** at line 956:
> The verification target matrix is reflected by machine-checked inventories: every matrix target phrased with "every" has an inventory-backed check that fails when an inventory entry lacks the required evidence.

**E7. Clarify the Local/CI parity row.** Change the Source-of-truth inventory cell to: `verification/profiles/*.json` plus the emitted local-vs-CI profile plan equivalence artifact. That keeps the static source of truth distinct from the runtime artifact and reads parallel to the rest of the matrix.

**E8. Align suggestion field name.** In Wave 4 (line 518), use `machine_applicable` verbatim as the catalog field name, and either drop "machine-application applicability" or note it as a synonym in one place only.

## Verdict

**Ready after minor edits.** The matrix is structurally sound, the inventory-derived approach is elegant, and integration with profiles/waves/decisions is consistent. But three missing surface rows (CLI, project_workspace, incremental/determinism), one missing wave assignment (algorithmic compatibility), and two unpinned inventory paths (snapshot rows) must land before this can be called elegant/implementation-ready. Edits E1–E5 are mandatory; E6–E8 are polish.
