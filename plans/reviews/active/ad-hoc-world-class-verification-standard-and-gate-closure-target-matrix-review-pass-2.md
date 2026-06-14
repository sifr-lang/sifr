# Final Review — Verification Target Matrix (Pass 2)

## Findings

### P0
None.

### P1
None. All pass 1 mandatory edits (E1–E5) and polish (E6–E8) are correctly incorporated.

- E1 — CLI row at line 116 ✓
- E2 — Project/workspace row at line 121 ✓
- E3 — Incremental/determinism row at line 143 ✓
- E4 — Wave 9.6 added (line 823) with task block (lines 868–872) and acceptance criterion (line 995) ✓
- E5 — `lowering_layer_inventory.json` and `codegen_construct_inventory.json` pinned at lines 128–129; Wave 5 task lines 589–592 ✓
- E6 — Acceptance criterion rephrased cleanly at line 970 ✓
- E7 — Local/CI parity inventory cell now `verification/profiles/*.json` + plan-equivalence artifact ✓
- E8 — `machine_applicable` consistent across matrix (line 125), Wave 4 task list (line 521), and Wave 4 validation (line 550) ✓

### P2 — polish only

1. **CLI exit-code contract list lacks a pinned file path.** Every other row's source-of-truth points at a concrete file. The CLI row (line 116) names `sifr` integration tests by binary target but the "plus CLI exit-code contract list" half is unscoped — no path, no owner-of-creation task. The minimum-content rule "exit-code contract without integration test fails" therefore has no machine-checkable input until Wave 0 implicitly creates one as part of the compiler surface matrix.
   - Optional fix: pin to e.g. `verification/areas/developer_tooling/data/cli_exit_code_contracts.json` and add a Wave 0 or Wave 1 sub-task to populate it from current `sifr` integration test expectations.

2. **Project/workspace row lacks an explicit "create the contract enumeration" task.** Similar to #1: the row promises "every shipped workspace and graph-isolation contract has a blocking suite row," but no wave is explicitly tasked with enumerating the contract set. Wave 0's compiler-surface matrix will absorb it, but it's left implicit.

Both are absorbed by Wave 0's `compiler_surface_matrix.json` work — implementers can correctly infer it — but pinning would mirror how the snapshot rows were tightened in pass 1.

## Required Edits

None blocking. Two optional polish items above; the document is internally consistent without them.

## Verdict

**Elegant / implementation-ready.**

The target matrix now covers every surface enumerated in the World-Class Verification Standard (CLI, project/workspace, incremental/determinism added; algorithmic compatibility now owned by Wave 9.6). All "every X" targets are tied to a pinned inventory file with one exception (CLI exit-code contract list, P2). Snapshot inventory paths are concrete, Wave 5 has the task to create them, and the acceptance criterion at line 970 closes the loop by demanding inventory-backed checks. No P0/P1 blockers; no contradictions between matrix, profile-assignment table, waves, and acceptance criteria. Ready to proceed to implementation without another review pass.
