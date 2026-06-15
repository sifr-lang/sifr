# Wave 6.0 Review — CPython Divergence Catalogue and Hand-Seeded Merge Smoke (Pass 1)

Reviewer: ad-hoc-world-class-verification reviewer.
Branch: `codex/wave-6-0-cpython-differential-smoke`.
Scope: `verification/policy/cpython_differential.md`, `verification/areas/cpython_differential/**`, `verification/runner/sifr_verify/profile_runner.py`, `verification/profiles/merge.json`, `verification/areas/coverage_matrix/compiler_surface_matrix.json`, and the Wave 6.0 tracker slice.

## Verdict

**Acceptable for PR** with non-blocking follow-ups noted below. No blocking issues found.

The policy satisfies every Wave 6.0 decision: supported constructs table, excluded divergences table with required divergence axes, exit-code-stable table, CPython version policy tied to `requires-python`, canonical one-JSON-line serializer contract, explicit cross-Python-minor non-comparability, no `repr`/exception-message comparison, and unsupported semantics modeled as catalogued exclusions rather than post-generation skips. The hand-seeded smoke runs CPython under `sys.executable` and Sifr under the `cargo run` CLI and compares JSON values, exit codes, and canonical-normalized stdout — this is genuine CPython-vs-Sifr differential evidence, not a Sifr self-test. Profile wiring is concrete: `profile_runner.run_cpython_differential_suites` is a non-conditional step that returns non-zero on suite failure, so `scripts/run_all_tests.sh --profile merge` is genuinely gated on both `policy` and `hand_seeded_merge`. Coverage matrix closure flips only `cpython_hand_seeded_differential` to `blocking`; `cpython_generated_differential` correctly remains `expected-missing` with `closes_in_wave: 6, closes_in_subwave: 1`, so generated coverage is not overclaimed.

## Findings (Severity-ordered)

### Blocking

None.

### Non-blocking — required-quality polish

1. **Linter does not validate the `supported_constructs` field** — `verification/areas/cpython_differential/checks/catalogue_lint.py:196-218`. `validate_case` checks `python`/`sifr`/`allowed_exit_codes`/`excluded_divergences` but never reads `supported_constructs`. Result: stale or invented construct ids in the manifest go undetected. Concrete drift already present: `bounded_int_arithmetic` claims `[2, 7, 9]` but does not use construct 7 (`if`/`for`); `boolean_string_logic` claims `[3, 4, 8, 9]` but also uses construct 5 (list literal); `list_iteration_indexing` claims `[5, 7, 9]` but also uses construct 2 (bounded ints); `dict_lookup_order_independent` claims `[6, 9]` but also uses constructs 3 and 8. Either drop the field or add a cross-check that every claim is a valid Table 1 row number (a fixture-content check is out of scope for a Wave 6.0 lint).
2. **Augmented assignment is used by a hand-seeded case but is not in Table 1** — `verification/areas/cpython_differential/fixtures/hand_seeded/list_iteration_indexing/main.sifr:8` (`total += value`) and `case.py:8`. Table 1 row 2 enumerates `+, -, *, //, %, unary -, integer comparisons` but does not name `+=`/augmented assignment. Either add augmented assignment for supported integer operators explicitly to Table 1 row 2, or rewrite the fixture as `total = total + value` to stay strictly inside the declared subset.
3. **Catalogue lint's "exactly one JSON line" and "requires-python" gates are substring presence checks** — `catalogue_lint.py:78-81`. A future rewording of the policy that drops the exact substring would silently pass. Promote each to a structural check (e.g., assert a labeled requirement bullet or a specific Table 3 caption) so the contract is rephrasing-safe.
4. **`sys.version` is logged only to stdout, not embedded in the area result artifact** — `verification/areas/cpython_differential/checks/hand_seeded_merge.py:47`. Policy line 7 requires oracle reports to "include the exact `sys.version`." Logging satisfies the human form but not a durable machine-readable artifact. Add `sys.version` to the case-level JSON output that the area adapter writes to `target/verification/areas/cpython-differential-results.json` (or the actual_root tree) so historic merge logs do not need to be re-mined to recover the interpreter version.
5. **Smoke runner does not enforce the v1 value grammar at runtime** — `hand_seeded_merge.py:117-135` only validates that stdout is exactly one parseable JSON line. The bounded-integer range `[-1000000, 1000000]`, no-floats rule, container-depth ≤ 4, string-keyed dicts, and homogeneous lists are not checked. For hand-seeded fixtures under reviewer inspection this is acceptable; flag this as a Wave 6.1 prerequisite because generated programs cannot rely on human review for grammar conformance.
6. **Linter does not reject orphan Table 3 rows** — `catalogue_lint.py:190-192` only checks `set(case_ids) - set(exit_codes_by_program)`; the reverse (table rows with no manifest case) is silently allowed. Minor; mirror the symmetric check so removing a smoke case forces a Table 3 update.
7. **`nightly_release_suite` for `cpython_hand_seeded_differential` omits the `policy` suite** — `verification/areas/coverage_matrix/compiler_surface_matrix.json:323`. Merge runs both `policy` and `hand_seeded_merge`, but nightly only re-runs `hand_seeded_merge`. The catalogue linter is cheap and re-running it in nightly hardens against policy drift on long-running branches. Add `cpython_differential:policy` to the nightly cell.
8. **Tracker has no dedicated `Wave 6.0 Implementation Notes` section** — `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:1163-1215`. Earlier waves (0, 1, 2.0-2.final, 3, 4, 5.1-5.3) each got an Implementation Notes block with status, scope, validation, artifacts, review. The Wave 6.0 status currently lives only in the top-line status sentence and the Implementation slice paragraph at line 1167-1169. Acceptable while in progress; add the section before merge so the closeout audit trail matches the rest of the phase.
9. **Hand-seeded smoke uses `cargo run` per case** — `hand_seeded_merge.py:73`. Four sequential `cargo run -q -p sifr -- run` invocations rely on the build cache; cold-cache compilation could push close to the 90s per-case `TIMEOUT_SECONDS` on slower hosts. Wave 6.1 plans the release-binary optimization (`cargo build --release` once); keeping `cargo run` for Wave 6.0 is fine. No action needed beyond noting the dependency so a future budget regression is not blamed on this lane.
10. **`run_command` raises `subprocess.TimeoutExpired` rather than producing a `RuntimeResult`** — `hand_seeded_merge.py:98-114`. If the 90s timeout fires, the suite crashes with an unhandled exception instead of recording a deterministic failure for the case. Trap `TimeoutExpired` and append a structured failure (`f"{case_id} {runtime} timed out after 90s"`) so the merge gate emits a deterministic failure record. Minor; only matters once a hand-seeded program regresses.

### File-size and maintainability

All Wave 6.0 first-party files are well under the 900-line cap: `catalogue_lint.py` 237, `hand_seeded_merge.py` 144, `runner.py` 56, `manifest.json` 39, `hand_seeded_manifest.json` 58, `cpython_differential.md` 67. The profile runner gained a single 11-line `run_cpython_differential_suites` method at `profile_runner.py:279-288`. No maintainability risk.

## Required Follow-up Before PR

None. All findings are non-blocking.

## Optional Follow-up

Cleanest split:

- **Apply in this PR (small, low-risk):** findings 1 (lint `supported_constructs` against Table 1 row indices), 2 (either rewrite the fixture to `total = total + value` or add augmented-int assignment to Table 1), 7 (nightly suite cell), 8 (Wave 6.0 Implementation Notes section), 10 (`TimeoutExpired` trap).
- **Track as Wave 6.1 prerequisites:** findings 3 (structural rephrasing-safe lint), 4 (embed `sys.version` in the result artifact), 5 (runtime value-grammar enforcement once generated programs land), 6 (symmetric Table 3 orphan check).

## Another Opus Review Round Required After Fixes?

Not required. The findings are mechanical and individually verifiable from the diff; a self-check by the implementer plus the validation commands listed in the tracker slice (`areas check`, `areas run --area cpython_differential --suite policy --suite hand_seeded_merge`, `profiles plan --profile merge`, `--self-test`, `check_file_size_guardrails.py`) is sufficient. Request another Opus round only if findings 1 or 2 are resolved by changing policy semantics rather than tightening the lint or rewriting the fixture.
