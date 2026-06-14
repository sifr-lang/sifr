## Review pass 2 — Wave 2.final

Verified the four follow-up changes against the working tree (no edits made):

- `verification/areas/coverage_matrix/data/cargo_metadata_classification.json:25` — `sifr_codegen` lib target now `profile_assignment: "merge"`. No remaining row uses `merge-red-blocker`.
- `verification/areas/coverage_matrix/compiler_surface_matrix.json:140` — `surface_id` is `codegen_merge_blocking`.
- `verification/areas/coverage_matrix/shipped_guarantees.json:49` — `merge_surface: "codegen_merge_blocking"` cross-references the renamed matrix row.
- `verification/runner/sifr_verify/selftest.py:105-115` — loops all four profiles (`create-pr`, `merge`, `nightly`, `release`) and asserts `sifr_codegen` is present in full-mode `crate_test_membership` with `status="blocking"` and `executed_in_merge=true`.
- `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json:5` — `updated_on: "2026-06-14 (post-Wave-2.final)"`.
- `crates/sifr_stdlib/tests/ipc_process_pipe_fixture.rs:37` — `WORKER_STARTUP_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())`. Standard poison recovery; safe because the guarded state is `()` (pure mutual exclusion), so an upstream panic can't leave corrupted data behind.

Remaining `merge-red-blocker` mentions live only in (a) the vocabulary enum `coverage_matrix.py`, (b) `verification/policy/profile_policy.md` wording, and (c) historical plan/review notes. None are data rows assigning the value to a real target — the vocabulary is kept available for future suites, which matches the documented policy.

### Answers

1. **B1 fixed end-to-end?** Yes. Cargo classification, compiler surface matrix, and shipped-guarantee `merge_surface` are all aligned on `codegen_merge_blocking`/`merge`. No data file still labels `sifr_codegen` as `merge-red-blocker`.

2. **Selftest + profile metadata prevent stale red-blocker semantics?** Yes. The new four-profile loop in `selftest.py` makes any future regression that demotes `sifr_codegen` to non-executed/red-blocker fail self-test in all four profiles, not just `merge`. Combined with the `updated_on` timestamp on the inventory, the post-closure semantics are pinned.

3. **New blocking issues introduced?** None. Selftest expansion is additive and consistent with prior assertions; the matrix/guarantee rename is a pure relabel cross-checked by both files; the inventory date is metadata-only; the IPC poison handler uses the canonical pattern over a unit-typed mutex.

4. **Wave 2.final is approved for PR/merge.**
