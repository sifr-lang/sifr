## Review findings — Wave 4 package library diagnostic synthetic baselines

**No blocking findings.** Another review round is not required after fixes. Below are non-blocking observations ordered by severity.

### Verified invariants

- **Source-hash binding**: `shasum -a 256 verification/areas/diagnostics/fixtures/diagnostics/package_library_diagnostic_rules_cases/main.sifr` = `aa2229e7c6b1f24cc8ee802bfa7607c69c8fafe0f68134dfcd31b64bef60df96`, matching `baseline_metadata.json:616`. Source-hash mismatch will fail `validate_baseline_metadata` at `code_baseline_coverage.py:333`.
- **Baseline trio complete**: `check-compact.stdout.txt` (1 B), `check-compact.stderr.txt` (1.1 KB, 9 codes), `check-compact.exit-code.txt` (`1\n`). Matches the trio expected by `validate_baseline_metadata` (`code_baseline_coverage.py:300-303`).
- **stderr evidence**: All 9 codes (`SIFR-PACKAGE-0001/0101/0103/0104/0204/0402/0404/0603/0604`) appear as substrings in `check-compact.stderr.txt`, satisfying `validate_coverage_baseline_evidence` (`code_baseline_coverage.py:262-271`). Substring matching is safe because `CODE_RE` (`code_baseline_coverage.py:27`) enforces exactly 4 digits — no code can prefix another.
- **Renderer support cross-check**: catalog entries for all 9 codes list `compact` in `renderer_support`, so `validate_coverage` line 243 passes.
- **Coverage counts match tracker**: `total=170, covered=158, deferred=12 → BUILD 5, INTERNAL 1, STDLIB 2, WORKSPACE 4`. Tracker text on lines 1046–1047 is accurate.
- **Math vs prior slice**: Twenty-second slice (PR #2612) closed 3 PACKAGE codes leaving `149 covered / 21 deferred` with `PACKAGE 9` outstanding. Twenty-third closes exactly those 9 → `158 covered / 12 deferred`. ✓
- **Catalog owner alignment**: All 9 codes are `stability=stable`, `severity=Error`, `owner=compiler/frontend`. Metadata `owner` field matches (`baseline_metadata.json:614`).
- **Precedent parity**: `presentation_rules_cases` already establishes the `synthetic=true` pattern — fabricated stderr, identical normalizers, identical bless-reference scheme. The new fixture is a faithful replication.
- **File-size guardrail**: `code_baseline_coverage.py` at 442 lines is well under 900. JSON files exempt.
- **Orphan/missing hygiene**: `validate_baseline_metadata` (`code_baseline_coverage.py:294-309`) unions `expected_files | synthetic_files` so the new fixture is allow-listed; absent files would surface as `missing required baseline file`, and stray files as `no owning manifest fixture`. Both checks still trigger correctly.
- **Hardcoded `command="check"` consistency**: `synthetic_baseline_cases` (`code_baseline_coverage.py:104`) and the `synthetic_files` block (`code_baseline_coverage.py:300`) both hardcode `check-{renderer}`, so the two sides agree and the new fixture's `check-compact.*` files are correctly admitted.

### Non-blocking notes

1. **Follow-up opportunity (informational, not in this PR's scope)** — `SIFR-INTERNAL-0001` already appears in `presentation_rules_cases/baselines/check-compact.stderr.txt:3` as `E SIFR-INTERNAL-0001 <unknown> spanless internal diagnostic`, but the coverage row at `code_baseline_coverage.json` still carries a Wave 4 deferral pointing at `null`. The new synthetic-baseline mechanism in this PR makes it possible to close that out by setting `baseline_fixture_id="presentation_rules_cases"` and `renderer_formats=["compact"]` (and likely `human`/`json` too). This PR correctly scoped to PACKAGE codes only; the INTERNAL closure should be a separate slice owned by `compiler/core-language` per existing deferral metadata.

2. **Placeholder `bless_reference`** — `baseline_metadata.json:606` uses `wave-4-package-library-diagnostic-baselines-pr` and the task description acknowledges this is to be replaced post-PR. Consistent with the precedent used by other Wave 4 entries.

3. **Synthetic stderr is fabricated text, not compiler output** — for 8 of the 9 codes, the stderr lines use `<unknown>` span stubs and hand-written messages (`SIFR-PACKAGE-0204` even synthesizes a TID identity pair that the actual renderer would derive at runtime). This is policy-acceptable under the existing "library-level or Cargo-environment dependent" carve-out (matches `presentation_rules_cases` precedent) and is constrained by source-hash + trio + metadata enforcement, but it is intentionally a weaker contract than fixtures that exercise the renderer through `sifr check`. The tracker phrasing in line 1045 correctly characterizes the scope.

4. **`command="check"` is an implicit, undocumented invariant** — both `synthetic_baseline_cases()` and `validate_baseline_metadata`'s `synthetic_files` block assume the command label is `check`. They agree, so today it's correct. If a future synthetic baseline ever needs a non-`check` command (e.g., `package`, `build`), both call sites would need to change in lockstep. No code change requested; just a thing to watch.

### Tracker accuracy

The Twenty-third slice entry on lines 1042–1047 of the phase tracker is accurate against the current state of `code_baseline_coverage.json`, `baseline_metadata.json`, and the fixture trio. The lead Status line on line 3 is also reasonable.

### Recommendation

Ready to proceed to create-pr / merge-gate validation without further review. After PR creation, replace the placeholder `bless_reference` and consider opening a follow-up slice to close `SIFR-INTERNAL-0001` using the now-extended synthetic mechanism.
