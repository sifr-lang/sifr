# M17 Wave 3 whole-diff review — pass (fresh, complete)

I reviewed the full `git diff main...HEAD` (17 files, +1124/−131: `certification_ledger.py`, matrix schema-2 + validator, area runner, profiles gate, selftests, demo, and docs) with focused adversarial probes I ran to completion in this invocation. No blocker, major, or actionable minor remains.

## Re-probe of the pass-2 remediation (suite→argv→report binding)

The binding in `certification_ledger.py` holds up under direct attack:

- **Order**: `_validate_suite_report_invocations` runs at `certification_ledger.py:40`, before any report bytes are loaded in `_validate_entry` (line 78) — evidence is never read past a drifted invocation.
- **Exactness**: for every selected required suite, the union of `--report` values across *all* recorded variants of *all* cases must equal exactly the one matrix-pinned path (`observed_paths != {expected_path}` → `SystemExit`, line 162). I probed this live against the real matrix: correct absolute pinning and the real runner's relative form (`../../../target/...`) are accepted; a wrong-suite report, an extra second `--report` target, an empty `cases` list, a dangling `--report`, and non-list/malformed argv are all rejected with `invocation drift`. Fail-closed in every case.
- **Negative self-test**: lines 320–328 strip the `--report` argument from a suite's recorded argv and require rejection containing `invocation drift`. I re-ran `run_declaration_capability_self_tests` + `run_compiled_certification_self_tests` and the sifr_verify profile self-tests locally — all green.
- **Path coherence across layers**: the child writes `paths.area_root / args.report` where `area_root` is `__file__`-derived (`runner/env.py:18`), the parent unlinks `AREA_ROOT / arg` pre-run (`runner.py:323-325, 368`), and the ledger validates `area_root / arg` — all three resolve to the same file regardless of cwd. The matrix validator additionally pins `report == target/verification/areas/python_interop/{suite}.latest.json`, making suite→report bijective, and `_command_report_path` rejects any report escaping `target/`.

## Other dimensions rescanned

- **Freshness / stale / partial / failed**: pinned reports are deleted before their owning case runs, so only a current-invocation report can exist; missing report → `SystemExit`; `skipped != 0`, nonzero failures, duplicate case IDs, `python-runner` execution model, unobserved markers, and failed certification commands all reject (each covered by a shipped negative self-test). Partial row selection aborts; a failed suite yields `status: failed` with evidence withheld; missing `total_failures` defaults to failed.
- **Resource-zero truthfulness**: the ledger requires the `:resources=zero` marker suffix *and* observation in stdout, and the fixtures (e.g. `numpy_buffer/buffer_declaration_numpy.sifr:36-38`) print it only after live-object/leak-counter equality checks — the compiled binary computes it, it isn't decorative. Matrix count 4 (buffer + arrow + 2×dlpack) matches the demo's enforced `resources-zero=4`.
- **Profile completeness gate**: `create-pr`, `merge`, `nightly`, `release` all select the five owning suites; only the `selected-areas-only` live profile is exempt, which the gate deliberately allows. The new negative self-test rejecting an arrow-omitting full profile passes.
- **Demo enforcement**: `demos/python_ecosystem_certification/run.sh` hard-gates `status == "complete"` with exact 7/10/4 via `jq -er` under `set -euo pipefail` — the pass-1 fix is real. Matrix arithmetic corroborates: 7 rows with compiled evidence, 10 entries, 4 resource-zero.
- **Schemas**: matrix schema-2 is enforced in both the area validator and the profiles gate; `result.schema.json` governs the top-level report, not the area payload, so the additive `compiled_certification` key (itself versioned) breaks nothing.
- **Maintainability / docs / milestone**: all touched hand-maintained files are under 900 lines (largest: `runner/run.py` at 865, +4 this diff). `python-interop.mdx`, the area README, exit evidence, both architecture docs, and the plan's Wave 3 section accurately describe the implemented behavior, recorded timings, and both prior review passes with their remediations.

Non-actionable notes: the ledger's "exactly one report per suite" check is now redundant defense-in-depth given the matrix pin (fine to keep), and the untracked pass-3/pass-4 review artifacts in `plans/reviews/active/` are worktree housekeeping outside this diff — the invalid pass-3 is superseded by this review.

VERDICT: SATISFIED
