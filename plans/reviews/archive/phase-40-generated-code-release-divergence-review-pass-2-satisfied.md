I audited the full working-tree diff, read all changed and untracked files, and independently probed the guards by mutating a sandboxed copy of the area (repo files untouched).

## Pass-1 findings — all seven independently verified closed

**1. `release-full` cannot silently narrow — closed.** `release_divergence_self_test.py:108-113` now asserts the *whole* plan list as a derivation of `PROFILE_SUITES["full"]` with `clippy` → `clippy-release`. I probed two narrowing mutations in a sandbox copy: setting `("corpus", "12", None)` and deleting the `panic-scan` entry from `release-full` (`runner.py:56-64`). Both make the self-test exit 1. The self-test is registered in the blocking `coverage_matrix:readiness` suite (`coverage_matrix/manifest.json`, suite `readiness` now carries `generated_code_release_divergence`), which both nightly and release select.

**2. Governed entries must be exercised — closed.** `require_all_divergences_exercised` (`release_clippy.py:248-256`) is invoked at `release_clippy.py:303-306` on every path through the `try` body, before the `if failures: raise`. Under `SIFR_GCQ_ENTRY_IDS`/`--group` filtering the missing IDs now append a failure and raise. Belt-and-braces: `run_gate` pops both `SIFR_GCQ_MAX_ENTRIES` and `SIFR_GCQ_ENTRY_IDS` when the plan says `None` (`runner.py:236-244`), and I confirmed those are the *only* two env vars `selected_positive_entries` reads (`generated_code_quality.py:349-392`).

**3. Policy truthfulness — closed.** `profile_policy.md:103-114` now has a Release Profile section naming all three entries (`e2e-018-cpython-math-semantic-corrections`, `e2e-027-error-mixed-builtin-stdlib`, `stdlib-007-math`), the exact lint, the blocking conditions, and nightly's unchanged `full` selection. `verification/README.md:85-88` points at it consistently.

**4. Disclosure with honest blocking count — closed.** Human: `runner.py:150-159` prints `expected_failures=3 records=GENC-NAN@2026-10-31` before the summary line (and does not collide with `HARDENING_OK_RE` in `reports.py:33-36`). Machine: `summary.expected_failures` plus a full `release_divergences` array (`runner.py:111-135`), while `blocking_failures = total_failures` stays untouched at `runner.py:110`.

**5. Mutation coverage — closed.** 15 cases, verified by running it: unexpected pass and additional lint via `check_entry` injection (`release_divergence_self_test.py:207-227`), malformed header, malformed record, duplicate record, duplicate/unsorted/unknown `entry_ids`, negative-group entry, expired, wrong gate, empty lints, unbound record, entry-binding mismatch, and missing governed execution. Matrix-side drift is caught by the canonical assertion at line 87: I removed `release_divergence_entries` from the `codegen_snapshots` row in a sandbox and got `generated-code release divergence matrix mismatch: … observed=[('GENC-NAN','2026-10-31',()), …]`.

**6. Matrix binding — closed.** `validate_matrix_binding` tokenizes on commas (`release_clippy.py:140-146`) and compares `(record_id, expiry, entry_ids)` triples (149-164). I probed a comma-joined `"some_other:suite, generated_code_quality:release-full"` row: still bound, self-test still passes. A row that drops `release_suite` entirely leaves `bound` but is then caught by `coverage_matrix.py:396-399` ("diverges from nightly without release_suite").

**7. Failure collection — closed.** `release_clippy.py:234` uses `check=False` for governed entries; every entry runs inside a per-entry `try/except` that appends to `failures` and continues (282-290), with a single aggregated raise at 307-308. `timed_case` still emits a per-entry `fail` timing.

## Safety contract

- Nightly: `selected_areas` `['full']` and `legacy_facade.generated_code_quality = "full"` — unchanged.
- Release: both set to `release-full`; `profile_assignment_matrix.validate_release_suite_alignment` pins `selected_areas` against the matrix rows, and `load_release_surface_suites` requires the suite to exist in the area manifest.
- Non-governed entries in `release_clippy.check_entry:229-233` replicate `gate_clippy`'s per-entry work exactly (materialize → `cargo fmt` → `cargo clippy -- <GENERATED_CLIPPY_ARGS>` with `check=True`), plus the negative-seed assertion at 272-278. No allow, threshold, skip, fallback, generated-source change, or Rust-interop code.
- Fail-closed record validation: closed header/field sets, sorted+deduped IDs, positive groups only, `gate == "clippy"`, `date.today()` expiry, `clippy::[a-z0-9_]+` lint shape, and exact matrix cross-binding.
- Docs: index status/expiry, roadmap row, phase 40, issue, and ad hoc doc (including the "remove `release-full`, return to `full`" exit criterion) all match the implementation. No demo added or renamed. The NaN source defect remains an indexed non-prerequisite follow-up.

## Non-actionable observations

- Removing the `require_all_divergences_exercised` *call site* (keeping the function) still passes the self-test — the test covers the function as a unit, not its wiring. I confirmed this creates no live fail-open: the only narrowing vectors are the two env vars, which `run_gate` pops, and the plan itself, which the structural assertion pins.
- A governed entry that emitted the expected lint *and* a hard rustc error would be accepted by `check_entry`; the `corpus` gate's `cargo check` over all 91 entries blocks that case independently.
- `runner.release_divergence_records()` reads the document without re-validating it; a malformed document surfaces as a traceback rather than a diagnostic, but the gate reading the same file fails blocking-ly first.
- `internal_docs/generated_code_quality.md` enumerates independently-selectable gate suites only (not profile suites like `full`/`representative`), so `release-full` and the non-selectable `clippy-release` gate are consistently out of that doc's existing scope.
- The `import json` removal in `profile_runner.py:7` is unrelated cleanup; `json.` has zero remaining uses in that file.

VERDICT: SATISFIED
