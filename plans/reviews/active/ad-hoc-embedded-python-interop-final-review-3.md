The two targeted patches are confirmed in place:

- `docs/python-interop.mdx:103` now has `@blocking_io` on `fetch_status_sync`, matching the contract enforced by `SIFR-ASYNC-0005` and the positive fixture `verification/python_interop/fixtures/async_blocking/offloaded_python_calls.sifr:14,26` (no import needed; `@blocking_io` is a built-in, same as in the fixture). The surrounding async offload shape (`task.spawn_blocking`, `await handle.join()`) also matches the canonical fixture and `crates/sifr/tests/e2e/fail/spawn_blocking_unannotated_rejected.sifr`.
- `internal_docs/python_interop_architecture.md:70` now reads `--group dataframes`, consistent with `verification/python_interop/README.md:38`, `docs/python-interop.mdx:324`, `verification/python_interop/reports/python_interop_exit_evidence.md:52,78`, and the canonical group list in the phase tracker.

Focused checks all clean:
- `rg` for `--group data\b`, raw `try` expressions, and `phase_closeout` returns no matches in scope.
- `git diff --check` clean.
- `docs.json` parses.

Items previously verified clean in final-review-2 (phase tracker / roadmap / exit-evidence status strings; `SIFR-PYENV-0001..0011` and `SIFR-PYTRUST-0001..0004` diagnostic registration; reserved family list `PYIMP/PYCALL/PYCONV/PYRES/PYZC/PYCB`; quiet support scoped to `build`/`run` in `area_adapter.py:397-411`, schema `area.schema.json:162-164`, and manifest `project_workspace/manifest.json:48,67`; public-example symbol coverage in `sifr_stdlib`; local validation evidence 132/0 create-pr, 651/0 merge gate with only advisory wall-time/group-skew) remain valid — no scope was touched outside the two B1/group-rename patches, so those judgments still apply.

reviewer satisfied: no blockers
