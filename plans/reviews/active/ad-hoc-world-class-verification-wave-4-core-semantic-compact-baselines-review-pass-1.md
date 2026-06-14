I've audited the slice end-to-end. Writing the review now.

# Wave 4 Core Semantic Compact Baselines — Review Pass 1

**Verdict: no blockers. Ready for PR after the optional cleanups below.**

The slice does exactly what it claims: 81 e2e fail fixtures promoted into compact-only diagnostics baselines, 82 deferral rows cleared, 97/170 active stable codes now have rendered baseline coverage, validation gates green. Every claim I spot-checked holds up.

## Blockers

None.

I verified:
- All 97 claimed `code → fixture` rows have their code present in the corresponding `check-compact.stderr.txt` (auditing every row, 0 mismatches).
- All 127 renderer×fixture pairs claimed in `code_baseline_coverage.json` have the full stdout/stderr/exit-code trio on disk (0 missing files).
- All 81 new fixtures are byte-for-byte equal to `crates/sifr/tests/e2e/fail/<name>.sifr`, and all have `# expect-error[...]` markers — no lossy promotion.
- All 81 new metadata entries point at a real `main.sifr`, the recorded `source_hash` matches the file on disk, owners (`compiler/core-language`, `compiler/diagnostics`) are valid per `verification/owners.json`, normalizers are subset-valid and correctly omit `json-sort` for compact.
- Manifest case count = on-disk e2e_* fixture count = metadata `e2e_*` fixture count = 81; baseline suite case total = 14 + 81 = 95 (matches user report).
- `cargo run --locked -q -p sifr -- --diagnostic-format compact check verification/areas/diagnostics/fixtures/diagnostics/e2e_async_no_suspend_rejected/main.sifr` reproduces the baseline byte-for-byte.
- Phase doc accurately scopes this as "third Wave 4 diagnostics-baseline slice … review is in progress" with 73 deferrals still tracked; it does not claim Wave 4 closure.

## Non-blocking findings

**1. One missed deferral clearance (low-value).**
`e2e_bare_defaultdict_constructor_rejected` emits `SIFR-STDLIB-0001` in its compact stderr alongside the claimed `SIFR-NAME-0002`, but the coverage row for `SIFR-STDLIB-0001` is still deferred. You could add a second coverage row pointing `SIFR-STDLIB-0001 → e2e_bare_defaultdict_constructor_rejected` to clear it for free. Defensible to leave for the stdlib-owned slice if they want a purpose-built fixture, but worth a sentence in the phase notes either way.

**2. The coverage check does not assert "claimed code actually appears in the fixture's stderr".**
`verification/areas/diagnostics/checks/code_baseline_coverage.py:171-221` validates fixture/renderer/manifest topology but never `grep`s for `entry.code` inside the corresponding baseline. Today every claim happens to be true (I verified), but a future row claiming a code the fixture does not emit would slip through silently. Wave 4 already cares enough about this to enforce it in the recovery surfaces (`verify_recovery_codes`); extending the same `code in baseline.read_text(...)` check to per-code coverage rows would close the loophole.

**3. Source duplication / drift hazard between diagnostics and e2e-fail.**
The 81 new fixtures are byte-for-byte copies of `crates/sifr/tests/e2e/fail/<name>.sifr`. `source_hash` pins the *diagnostics copy* but does not link to the e2e original. A future edit to the e2e fixture (rename, refactor, semantic change) will only fall out of sync if the rendered compact output also changes — silent in cases where the code/position/message survives. Two non-invasive mitigations: (a) a `code_baseline_coverage.py` check that asserts `e2e_<name>/main.sifr` is byte-equal to `crates/sifr/tests/e2e/fail/<name>.sifr` when the fixture id starts with `e2e_`, or (b) record `source_origin` next to `source_hash` in `baseline_metadata.json` so divergence is auditable.

**4. Warm merge wall-time over budget, no mitigation shipped.**
Reported `wall_time=1016.25s` vs. `warm_wall_time_minutes: 15` (900s) in `verification/profiles/merge.json`. The phase doc (line 1309) says: *"If a gate-expanding wave would push merge over `warm_wall_time_minutes`… the same PR must ship deterministic sharding, bounded profile parallelism, or a documented move of broad non-merge coverage to nightly while preserving merge smoke."* Wave 4 is explicitly listed (line 1308) as a gate-expanding wave. The slice is already at the cheapest renderer (compact), so the only remaining levers are sharding/parallelism, raising the budget with rationale, or splitting the diagnostics baselines suite across profiles. Today the breach is "advisory only" and doesn't fail the run, but the policy-letter answer is that this PR should either ship one of those mitigations or amend the budget. Flag in the PR description and align with the phase owner.

**5. Wave 4 closure remains far off, not just one slice away.**
73 deferrals remain. Of those, 10 non-package semantic codes (`SIFR-FLOW-0901`, `SIFR-IMPORT-0004..0007`, `SIFR-INT-0011`, `SIFR-INTERNAL-0001`, `SIFR-RESULT-0006`, `SIFR-TYPE-0901`, `SIFR-TYPE-0902`) have no current e2e fail fixture, so copy-promotion will not work for them. Many are `Warning`/`Note` severity (e.g., `SIFR-INT-0011`, `SIFR-TYPE-0901`, `SIFR-TYPE-0902`) and have no `expect-error` evidence path at all. The phase doc correctly acknowledges this. Just calling it out so the PR description doesn't read as "almost done with Wave 4" — it's "merge-requirement progress on the semantic family; package/build/workspace/lint/fmt/io/stdlib/encoding families still owe their own slices, plus purpose-built fixtures for the 10 stragglers."

**6. Pre-existing ownership inconsistency on non-semantic deferrals (not introduced here).**
`SIFR-PACKAGE-*` and `SIFR-WORKSPACE-*` deferrals are owned by `compiler/frontend`; `SIFR-STDLIB-*` deferrals are owned by `compiler/core-language`. Per `verification/owners.json` the more specific owners (`compiler/package-management`, `stdlib/parity`) exist. This was true before this slice and is not the slice's job to fix, but a one-line `jq` sweep aligning these would be cheap to bundle.

**7. `bless_reference` is a placeholder slug.**
`wave-4-core-semantic-compact-baselines-pr` — matches the convention of prior slices (`wave-4-diagnostic-baseline-catalog-pr`, `wave-4-hir-recovery-baseline-pr`). Replace with the real PR URL after open. Consistent with prior practice; not a deviation.

## Answers to your review questions

**Q1. Are any coverage rows incorrectly cleared from deferral?**
No. Every one of the 97 claimed `(code, fixture)` pairs is backed by the code literally appearing in the fixture's `check-compact.stderr.txt`. The two cases where a single fixture covers multiple codes (`e2e_fixed_width_const_expression_out_of_range` → `SIFR-INT-0001`+`SIFR-INT-0004`; `hir_mixed_semantic_recovery` → 4 codes) both check out. The reverse miss — a code emitted incidentally but left deferred — exists for exactly one case (`SIFR-STDLIB-0001` from `e2e_bare_defaultdict_constructor_rejected`), and is conservative rather than wrong.

**Q2. Are any manifest/metadata/source-hash/ownership entries inconsistent or too weak?**
Internally consistent and validated. Two weaknesses worth noting: (a) `source_hash` pins the copy but not the link back to the e2e original (finding #3); (b) `code_baseline_coverage.py` does not assert the claimed code is actually in the baseline (finding #2). Ownership values are valid per `owners.json`; the package/workspace/stdlib ownership mismatches in deferrals (finding #6) predate this slice.

**Q3. Is copying e2e fail fixtures acceptable for this merge-required compact baseline slice?**
Yes for the merge-only "at least one rendered baseline" requirement — the compact stderr proves the diagnostic codepath emits code, position, and message text at a stable location. No for Wave 4 EXIT or nightly/release "every stable renderer for every active stable code". Copying gives you compact correctness but not curated minimal spans, no-noise message text, or suggestion-render coverage. Future slices closing nightly/release renderer coverage will need purpose-built fixtures, especially for the 10 stragglers without any e2e-fail evidence and for `Warning`/`Note` codes that the e2e-fail mechanism does not exercise.

**Q4. Are docs/status statements accurate and not overstating Wave 4 completion?**
Yes, accurate. The status line says "Wave 4 core semantic compact baseline slice locally validated and ready for review" — not "Wave 4 merged". The implementation note correctly reports 97 rendered / 73 deferred, names the 10 semantic stragglers without current e2e evidence, and explicitly defers package/build/workspace/lint/fmt/io/stdlib/encoding to "owning Wave 4 slices." No overstatement.

**Q5. Production blockers before opening the PR?**
None. The warm-budget overrun (finding #4) is the closest to a blocker on paper; in practice it is advisory and the slice already uses the cheapest renderer, so it is a phase-policy alignment item to surface with the owner rather than a fix-this-PR blocker.

## Residual risks worth tracking

- **Drift between e2e-fail and diagnostics copies** (finding #3) — will surface as silent staleness, not test failure, until something in the rendered message also changes. Worth closing before Wave 4 grows to 170 copy-fixtures.
- **Coverage check tautology** (finding #2) — the check enforces topology but not semantic correctness of the `code → fixture` mapping. Today's 97 are correct, but the cheapness of adding a `grep` makes this a no-brainer to harden.
- **Merge wall-time pressure** (finding #4) — adding the remaining 73 baselines will roughly double the per-baseline cost again unless sharding or parallelism lands. Decide the strategy before slice 4 starts, not after it lands over budget.
- **The 10 semantic stragglers** (`SIFR-FLOW-0901`, `SIFR-IMPORT-0004..0007`, `SIFR-INT-0011`, `SIFR-INTERNAL-0001`, `SIFR-RESULT-0006`, `SIFR-TYPE-0901`, `SIFR-TYPE-0902`) — several are `Warning`/`Note` severity that the e2e-fail expect-error pipeline does not exercise. Closing these will need new fixture infrastructure or a documented exception.
