
All three pass 6 findings are resolved:

1. **M4 cycle-trace scope** — M4 (line 467) now explicitly requires choosing and documenting the import-edge extraction approach before implementation: "extend compile-order inputs to carry `ImportDependency` records, add a parallel range-aware compile-order path, or re-derive import ranges at cycle-diagnostic time with tests documenting any fallback limitations."

2. **Contract checker pre-flight** — Lines 4–6 now state: "Pre-flight requirement: implement M1 first. All later milestones depend on the source-canonicalization contract checker existing, being wired into `scripts/run_all_tests.sh --profile quick`, and failing against the current known gaps before producer implementation begins."

3. **Ambiguous/collision flow parity scope** — M3 (lines 459–460) now explicitly lists: "single-file mode is exempt because it has no workspace/package discovery. Workspace/project mode and package mode are in scope across CLI paths and driver/frontend API paths that perform discovery."

**No blockers remain.** The phase is ready to implement.
