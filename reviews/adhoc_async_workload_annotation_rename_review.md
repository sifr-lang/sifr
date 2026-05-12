

Reviewing the staged diff for annotation vocabulary change (`io_bound`/`cpu_bound` → `blocking_io`/`cpu_heavy`):

**Finding 1 — Severity: Info (not blocking)**
`crates/sifr_driver/src/tests/single_file_frontend.rs:267` — Test function `test_type_check_source_surfaces_workload_annotation_warning` verifies the new `@blocking_io` annotation. The test also checks that the diagnostic's `workload` arg is `"blocking_io"`. This is correct.

**Finding 2 — Severity: Pass**
`crates/sifr_hir/src/lower/workload_annotations.rs` — The `WorkloadKind` enum and `annotation_for_decorators` function have been updated:
- `IoBound` → `BlockingIo`, `CpuBound` → `CpuHeavy`
- `label()` returns `"blocking_io"` and `"cpu_heavy"` 
- Parser accepts `"blocking_io"` and `"cpu_heavy"` strings
- Suggestions remain unchanged (correct, they are action-oriented, not vocabulary)
This is internally consistent.

**Finding 3 — Severity: Pass**
E2E fixture renames and content:
- `io_bound_annotation_warning.sifr` → `blocking_io_annotation_warning.sifr` — `@io_bound` replaced with `@blocking_io`
- `cpu_bound_annotation_warning.sifr` → `cpu_heavy_annotation_warning.sifr` — `@cpu_bound` replaced with `@cpu_heavy`
Both renames are clean.

**Finding 4 — Severity: Pass**
`demos/blocking_offload_demo/main.sifr` — Both decorator usages updated. Demo is consistent with new vocabulary.

**Finding 5 — Severity: Pass**
`internal_docs/phases/32_async_ecosystem.md` and `issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md` — Documentation updated. The issue tracker note about the 2026-05-12 annotation vocabulary slice is accurate and complete.

**Finding 6 — Severity: Pass**
`verification/validation_lanes/quick_e2e_manifest.json` — Fixture entries updated to `blocking_io_annotation_warning` and `cpu_heavy_annotation_warning`.

**Finding 7 — Severity: Pass**
User confirmed an `rg` scan for legacy spellings (`io_bound`, `cpu_bound`) in `crates`, `demos`, `internal_docs`, `issues`, `verification`, `docs` returned no matches, excluding historical review artifacts and `host_io_bound` resource-classification names. This is the correct exclusion set.

---

**Summary**: The diff is internally consistent. All four annotation occurrences (`@io_bound`, `@cpu_bound`) have been renamed across code, tests, fixtures, demo, and docs. The HIR parser, enum variants, diagnostic labels, and test expectations are all updated in lockstep. No stale references remain.

**No blocking findings. The change is ready to PR/merge.**
