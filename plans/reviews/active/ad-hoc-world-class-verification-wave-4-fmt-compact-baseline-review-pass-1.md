## Findings

### Blockers
**None.**

### Medium
- **M1 — Plan-tracker phrasing (carried over from prior slices).** `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:907` reads "rendered diagnostic coverage is now 119 active codes, with 51 active codes still carrying Wave 4 deferrals." The numbers are correct (119 non-deferred + 51 deferred = 170 stable codes — confirmed against `code_baseline_coverage.json`), but the wording suggests 51 is a subset of 119. Identical phrasing exists in the seventh-slice entry, so this is a convention issue, not introduced here. Optional: clarify to "of 170 stable codes, 119 have rendered baseline coverage and 51 carry Wave 4 deferrals."

### Low / informational
- **L1 — `bless_reference` placeholder.** `verification/areas/diagnostics/data/baseline_metadata.json:2167` uses `"wave-4-fmt-compact-baseline-pr"`. Recently merged Wave 4 slices use full PR URLs (e.g. `https://github.com/sifr-lang/sifr/pull/2580`); two prior pre-PR slices used the same `wave-4-*-pr` placeholder pattern, so this matches convention. Update to the PR URL once the PR exists.
- **L2 — Parallel `BASELINE_COMMANDS` set drifts further.** `verification/runner/sifr_verify/hardening/core.py:28` keeps a separate `BASELINE_COMMANDS = {"check", "run", "build", "test"}` (already missing `lint` from before; now also missing `fmt-check`). Not a regression from this slice — but if anyone later promotes a formatter bug to the `fixedbugs` suite using `command: fmt-check`, both the metadata validator (`fixedbugs_and_crashes.py:77`) and `run_variant` in `hardening/core.py:147` would fail, because the hardening path doesn't have the alias rewrite either. Out of scope for this slice; worth tracking.
- **L3 — `--no-cache` is a no-op for `fmt --check`.** Looking at `crates/sifr/src/check_and_package_commands.rs:422-449`, the cache hit/write logic only runs in the formatter write branch, not the `--check` branch. The flag is harmless and futureproofs determinism, but is technically redundant.
- **L4 — `fmt-check` alias hides flag composition.** The only place a reader can learn that `fmt-check` expands to `fmt --check --no-cache` is `verification/runner/sifr_verify/area_adapter.py:453-456`. A generalized `extra_args` field in the manifest schema would scale better if more flag-bearing fixtures land. Acceptable for one fixture; revisit if the alias pattern proliferates.
- **L5 — Broad validation pending (by design).** The new plan entry says "locally focused-validated" and intentionally omits `scripts/run_all_tests.sh` evidence. Per `AGENTS.md`/`CLAUDE.md`, the full local gate must run before PR submission.

### Confirmed correct
- `shasum -a 256 verification/areas/diagnostics/fixtures/diagnostics/fmt_formatting_drift/main.sifr` matches `sha256:21709500…` in metadata exactly.
- `SIFR-FMT-0001` is the only active code in the FMT family (`crates/sifr_diagnostics/src/codes/registry/registry_entries/formatting_and_lint.rs:7-17`), so closing it closes the family. Reserved entry `SIFR-FMT-0000` is not active and not expected to have a baseline.
- Compact stderr text `source is not formatted with sifr fmt` matches the registry's `message_template`. Column 15 on line 2 of the fixture points at the `=` in `value: int=1` — consistent with formatter drift detection.
- Coverage matches plan note exactly: 119 non-deferred + 51 deferred (BUILD 6, ENCODING 1, INTERNAL 1, IO 2, PACKAGE 34, STDLIB 3, WORKSPACE 4). Zero non-deferred codes lack a `baseline_fixture_id`.
- Baseline-metadata counts: 145 `baselines`-suite variants (117 compact + 14 human + 14 json) = the recorded validation variants. 3 synthetic entries are still separate.
- Manifest case count = 117 (contracts 5, baselines 117) and the new case lives only under `baselines`.
- Normalizer set `["workspace-path","tmp-path","crlf","artifact-cache-lines"]` is the universal convention across all 117 compact entries and is a subset of the `NORMALIZERS` allowlist enforced by `verification/areas/diagnostics/checks/code_baseline_coverage.py:26`.
- Baseline file trio `fmt-check-compact.{stdout,stderr,exit-code}.txt` is present and matches `expected_baseline_files`'s `{command}-{renderer}` label rule.
- The diagnostics contracts suite (already passed) actively enforces `source_hash`, `normalizers`, `bless_reference`, `owner`, suite/renderer matches, and presence of the metadata/baseline pair — so the new entry passing contracts is meaningful evidence, not just shape-only.
- Unit coverage already exists: `cargo test -p sifr_format` (7 tests) covers `check_reports_formatting_drift`, the registry entry's pointed-to test.

### Recommendation
**No additional review round required before PR submission.** Run the full local gate (`scripts/run_all_tests.sh --profile create-pr` at minimum, ideally `scripts/run_all_tests.sh`) and update `bless_reference` to the PR URL once it exists. Consider the M1 wording tweak only if the convention itself is being revisited; otherwise carry forward as-is.
