Pass-3 review of the Wave 4 workspace manifest/source-root compact baselines slice on branch `codex/wave-4-workspace-compact-baselines`. Focus: confirm the pass-2 blocker (rationale text landed on the wrong rows) is resolved.

## Findings (ordered by severity)

### 1. BLOCKER — Pass-2 fix was misapplied again: the "Legacy workspace graph" rationale is now on SIFR-BUILD-0006, SIFR-BUILD-0901, SIFR-ENCODING-0803, and SIFR-FMT-0001; SIFR-WORKSPACE-0101..0104 still hold the original boilerplate

Severity: high (same governance-honesty class as pass-2 finding #1)

The post-pass-2 summary from the implementer states two things were done: (a) restore SIFR-BUILD-0002..0005 deferral.reason to the boilerplate, and (b) apply the legacy-workspace rationale to SIFR-WORKSPACE-0101..0104. Step (a) was done correctly. Step (b) was again misapplied to four entirely different unrelated codes.

Concretely, `git diff verification/areas/diagnostics/data/code_baseline_coverage.json` against HEAD shows exactly four `reason:` replacements introducing the new rationale text, and they sit at:

- `verification/areas/diagnostics/data/code_baseline_coverage.json:127-132` — `SIFR-BUILD-0006`, owner `compiler/frontend`. This is the project/build "no source roots discovered" code, emitted from build orchestration, not from `sifr_driver::workspace`.
- `verification/areas/diagnostics/data/code_baseline_coverage.json:140-145` — `SIFR-BUILD-0901`, owner `compiler/frontend`. Project-bootstrap code, unrelated to the legacy workspace graph.
- `verification/areas/diagnostics/data/code_baseline_coverage.json:347-352` — `SIFR-ENCODING-0803`, owner `compiler/core-language`. An encoding-family diagnostic; not a workspace import graph code at all.
- `verification/areas/diagnostics/data/code_baseline_coverage.json:452-457` — `SIFR-FMT-0001`, owner `compiler/diagnostics`. Formatter-family diagnostic.

None of these four codes are listed in `LEGACY_WORKSPACE_IMPORT_CODES` (`crates/sifr_driver/src/bin/diagnostic_rendering_harness.rs:13-18`); none of them are intentionally replaced on public CLI paths by source-spanned SIFR-IMPORT diagnostics. The new rationale text — "Legacy workspace graph diagnostic remains active, but current public project import paths intentionally render source-spanned SIFR-IMPORT replacements. Wave 4 follow-up must either add a lower-level rendered harness for this legacy code or make an explicit coverage-policy decision." — is materially wrong for each of them.

Meanwhile the intended targets remain untouched. `code_baseline_coverage.json:1839-1888` shows all four `SIFR-WORKSPACE-0101..0104` entries still carry the original "Rendered baseline fixture expansion is staged inside Wave 4..." boilerplate — exactly the text the pass-1 review flagged as dishonest for this code group and that the pass-2 review re-flagged.

Validation does not catch this because `deferral.reason` is free text; diagnostics contracts and `git diff --check` still pass. The defect is purely semantic.

Required fix before PR:
1. Revert the four `deferral.reason` strings at lines 132, 145, 352, and 457 of `code_baseline_coverage.json` back to the prior boilerplate ("Rendered baseline fixture expansion is staged inside Wave 4; this code is active and tracked by registry/docs/e2e coverage until its rendered baseline lands.").
2. Apply the workspace-graph rationale to the four entries at lines 1844, 1857, 1870, and 1883 — `SIFR-WORKSPACE-0101..0104`, the rows the pass-1 and pass-2 reviews and the tracker actually identified.
3. Re-run `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` and `git diff --check` to confirm no regression.
4. Before staging again, grep the file for the new rationale and confirm it shows up at exactly four lines and they are the `SIFR-WORKSPACE-010x` rows:
   - `grep -n "Legacy workspace graph diagnostic remains active" verification/areas/diagnostics/data/code_baseline_coverage.json` → should return 4 lines.
   - For each hit, the next surrounding `"code":` entry must be one of `SIFR-WORKSPACE-0101`, `0102`, `0103`, `0104`. This is the check that has been missed twice in a row.

### 2. PASS — SIFR-BUILD-0002..0005 deferral.reason is correctly restored

`code_baseline_coverage.json:75-123` shows all four BUILD codes carry the boilerplate again, matching their pre-slice state. `git diff` confirms no residual diff on these rows. This half of the post-pass-2 fix is good.

### 3. PASS (carried from pass 2) — Each new SIFR-WORKSPACE-0001..0004 baseline still covers its intended code and is reachable through the public `check` command

No regressions in the slice's primary deliverable. Fixture set, manifest entries, baseline metadata, and coverage flips for SIFR-WORKSPACE-0001..0004 are unchanged from pass 2 and remain correct.

### 4. PASS — Manifest, coverage, and baseline metadata counts remain internally consistent

`verification/areas/diagnostics/manifest.json` baselines suite = 116 cases; contracts suite = 5. `code_baseline_coverage.json` totals 170 codes (118 active, 52 deferred) with the family deferral breakdown matching the tracker. `verification/areas/diagnostics/data/baseline_metadata.json` carries 147 entries (144 baselines + 3 synthetic_baselines). No drift since pass 2.

### 5. PASS — Validations re-run after the post-pass-2 fix

Per the summary, `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` returned `variants=5, failures=0, blocking_failures=0` and `git diff --check` passed. Both are consistent with what I'd expect — the bug is in free-text metadata only, so neither check can catch it.

## Verifications performed in this pass

- `git diff --stat` confirmed only the expected files are touched (tracker, manifest, coverage, metadata, four new fixture dirs, three review markdowns).
- `git diff verification/areas/diagnostics/data/code_baseline_coverage.json` enumerated every changed `reason:` row.
- `grep -n "Legacy workspace graph diagnostic"` against `code_baseline_coverage.json` shows the rationale at lines 132, 145, 352, 457 — confirming the misattribution.
- Read each of `code_baseline_coverage.json:125-150`, `:345-360`, `:450-465`, and `:1835-1890` to verify which `code:` field each updated `reason:` actually belongs to and to confirm WORKSPACE-0101..0104 are still on boilerplate.
- Cross-checked the four wrongly-targeted codes against `LEGACY_WORKSPACE_IMPORT_CODES` in `crates/sifr_driver/src/bin/diagnostic_rendering_harness.rs:13-18` — none of them are listed.

## Verdict

**Blocked.** The pass-2 blocker is *not* resolved. The fix landed on yet another set of four wrong rows — SIFR-BUILD-0006, SIFR-BUILD-0901, SIFR-ENCODING-0803, SIFR-FMT-0001 — while SIFR-WORKSPACE-0101..0104 (the intended targets, called out by name in both the pass-1 and pass-2 reviews and in the tracker) still hold the original boilerplate. This makes the tracker's "Review" bullet false for the second time and now misdescribes a *different* group of four codes than pass 2 did.

This is the same class of error as last round, and it has already slipped twice. Before the next attempt, please grep-verify the rationale lands on the named codes (`grep -n "Legacy workspace graph diagnostic remains active" verification/areas/diagnostics/data/code_baseline_coverage.json` must return exactly the four `SIFR-WORKSPACE-010x` rows around lines 1844, 1857, 1870, 1883) and revert the misattributed text on lines 132, 145, 352, and 457. **Another review round is required after the fix** to confirm the rationale finally lands on the intended rows.
