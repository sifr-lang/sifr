## Identity verification

| Check | Result |
|---|---|
| local `HEAD` | `eebc715f412be91e7751a0ac56a80d0e3ca4271b` |
| `origin/codex/phase40-generated-code-release-divergence` | same SHA |
| PR #3049 `headRefOid` (base `main`, OPEN, MERGEABLE) | same SHA |
| tracked working tree | clean (only untracked `plans/reviews/active/…pass-3-exact-pr-head.md`, not in the PR) |
| PR diff ≡ `git diff origin/main...HEAD` | identical `patch-id` `6079220a65a59dee`, 20 files |

Single commit; changes confined to `plans/` and `verification/`. No `crates/`, `demos/`, `stdlib`, or generated-source edits.

## Pass-1 closures — reverified independently

1. **Plan pinning** — `release_divergence_self_test.py:108-113` asserts the *whole* `PROFILE_SUITES["release-full"]` list as a derivation of `["full"]` with `clippy`→`clippy-release`. I diffed `runner.py:44-64`: the two lists are elementwise identical except that one gate token. Breadth claim holds.
2. **Governed entries must execute** — `require_all_divergences_exercised` (`release_clippy.py:242-250`) is called at `:301-304` inside the `try`, before the aggregate raise. `run_gate` pops both `SIFR_GCQ_MAX_ENTRIES` and `SIFR_GCQ_ENTRY_IDS` when the plan says `None` (`runner.py:236-244`), and those are the only two env vars `selected_positive_entries` reads (`generated_code_quality.py:349-392`).
3. **Policy truthfulness** — `profile_policy.md:103-113` names all three entries, the exact lint, the blocking conditions, and nightly's unchanged `full`.
4. **Disclosure** — human line `runner.py:150-159`; machine `summary.expected_failures` + `release_divergences[]` (`runner.py:111-135`). `blocking_failures = total_failures` untouched at `:110`; the new line does not match `HARDENING_OK_RE` (`reports.py:33-36`).
5. **Mutation coverage** — I ran the self-test at this head: `PASS (15 mutations)`.
6. **Matrix binding** — `validate_matrix_binding` comma-tokenizes `release_suite` and compares `(record_id, expiry, entry_ids)` triples; empty `bound` also raises.
7. **Failure collection** — governed entries use `check=False`; every entry sits in a per-entry `try/except` appending to `failures` (`:281-289`) with one aggregated raise at `:305-306`.

## Safety contract — checks I re-ran at this head

- **Breadth**: 91 positive corpus entries (96 total − 5 negative-seeds), all reached by `clippy-release`; the three governed IDs are all in positive groups (`e2e-pass-representative`×2, `stdlib-flows`).
- **Nightly unmodified**: `nightly.json` still `"full"` in both `selected_areas` and `legacy_facade`; only `release.json` moved to `release-full`.
- `python3 …/release_divergence_self_test.py` → `PASS (15 mutations)`
- `python3 …/checks/profile_assignment_matrix.py` → `profile assignment matrix ok: rows=19`
- `PYTHONPATH=verification/runner python3 …/checks/coverage_matrix.py` → `coverage matrix ok: guarantees=13 surfaces=34 temporary_rows=0`
- `…/coverage_matrix_readiness_self_test.py` → `cases=24`
- `readiness` suite now has 5 cases including `generated_code_release_divergence`; all four profiles select `coverage_matrix:readiness`, so expiry (2026-10-31), `release_suite != nightly_release_suite`, and the `plans/phases/index.md` cell-3 link for `GENC-NAN` are blocking.
- `scripts/check_file_size_guardrails.py` → `PASS (2952 files, limit 900)`; new files are 334 and 246 lines.
- **Gate parity**: `release_clippy.check_entry` replicates `gate_clippy` per-entry exactly (materialize → `cargo fmt` → `cargo clippy -- GENERATED_CLIPPY_ARGS`), same negative-seed assertion, same `TARGET_ROOT`/`SIFR_GCQ_SHARED_ROOT`. `GENERATED_CLIPPY_ARGS` unchanged.
- **No allow/skip/threshold/fallback**: none in the diff; the surface-matrix surface_id rename (`generated_code_quality` → `codegen_merge_blocking` + two added rows) is required by `profile_assignment_matrix.py:81-83`, and the old id was dangling.
- **Doc mutual truth**: index status/expiry, roadmap row, phase 40, issue ledger, ad hoc doc (including the "remove `release-full`, return to `full`" exit criterion), policy, and README all agree on three entries / one lint / 2026-10-31 / nightly-unchanged.
- `import json` removal in `profile_runner.py` is safe — zero remaining `json.` uses in that file.

## Non-actionable observations

- `runner.expected_failures` is derived from the data file, not from observed run results. Not a fail-open: a governed entry that passes, drifts lint, or is skipped makes the gate exit non-zero, so `expected_failures=3` is only published alongside a genuinely green run.
- `coverage_matrix.py:validate_release_divergence` does not check `release_divergence_entries`; that direction is enforced by `release_clippy.validate_matrix_binding`, so the pair is complete.
- `ruff` and `uv` are not resolvable in this shell, so I could not re-run the Ruff/profile-schema lanes; those remain as recorded in the ledger. Everything else above I executed at this head.

No actionable finding.

VERDICT: SATISFIED
