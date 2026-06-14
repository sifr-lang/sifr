Local re-check confirmed: codegen `655/52/707` red, `sifr_type_system` 92, `sifr_format` 7, `sifr_lint` 22, `sifr_source` 3, `sifr_ir` 0, `verification/areas/diagnostics/` ships 2 baseline fixtures, `verification/areas/coverage_matrix/` does not exist, `verification/areas/fuzz_property/` and `algorithmic_compatibility/` do exist, `profile_runner.run_crate_tests` still hard-codes the crate list and omits the six crates the plan calls out.

---

# Findings

## P0 — Plan-blocking

None. All four pass-1 P0 items are closed:

- **P0-1 (empty-crate no-op):** Closed. The plan now records `sifr_type_system=92`, `sifr_format=7`, `sifr_lint=22`, `sifr_source=3`, `sifr_ir=0`, and only `sifr_ir` carries the `tests:none` provision.
- **P0-2 (missing `sifr_ir`/`sifr_source` rows):** Closed. Both are listed in Wave 1, and the Wave 1 guardrail explicitly catches "first-party crates with zero tests and no explicit `tests:none` coverage-matrix row."
- **P0-3 (advisory→blocking matrix):** Closed. §Decisions and Wave 0 state the matrix lands advisory with a closed `expected-missing` list, promotes to blocking in Wave 10, and forbids adding `expected-missing` rows after Wave 0 lands.
- **P0-4 (CPython catalogue not pre-committed):** Closed. Wave 6.0 ships the catalogue with two enumerated tables and a catalogue linter, and Wave 6.1 must lint generated programs against it (no after-the-fact filtering).

## P1 — Should fix before treating as final

**1. Wave 4 acceptance vs. Wave 4 tasks are softly contradictory on renderer coverage.**
Acceptance criterion: "Diagnostic rendered baselines cover every active `SIFR-*` code with a stable user-facing message **and all stable renderers**." Wave 4 task: "If the merge lane exceeds the documented budget, keep broad renderer permutations in nightly while preserving one merge-blocking baseline per active stable code." Either the criterion needs `(at merge: ≥1 renderer; at nightly/release: all stable renderers)` phrasing, or the Wave 4 fallback is forbidden. As written, a future reviewer can claim the criterion is met by `merge=1 renderer + nightly=all renderers` while the acceptance text reads as "all renderers merge-blocking."

**2. "Stderr classification" in Wave 6.1 is underdefined.**
The subset compares "stdout, stderr classification, and exit code." Sifr's error model is Result/Option (an excluded divergence), so byte-equal stderr comparison would fail constantly. The plan should spell out the classification axes — e.g., "exit code bucket (0 / non-zero), no-error vs. compile-error vs. runtime-error, message-presence not message-equality" — or defer stderr comparison to a follow-up. Otherwise Wave 6.1 implementers will either over-strict (false positives) or invent a private contract.

**3. Multi-error recovery coverage rule is unspecified.**
Acceptance: "Multi-error diagnostic recovery is baseline-tested." Wave 4 task: "Add multi-error recovery fixtures where one source file intentionally triggers several independent diagnostics." There is no numeric or surface-based rule. A single fixture would technically satisfy both. Either tie it to a list of recovery contracts in `crates/sifr_diagnostics` (one fixture per recovery surface) or state a minimum like "every parser/HIR/type recovery boundary documented in `verification/policy/...md` has ≥1 multi-error fixture."

**4. The Wave 10 promotion of profile-assignment table → profile JSONs has no check.**
Acceptance: "Profile assignment in the decisions table is reflected by `verification/profiles/{create-pr,merge,nightly,release}.json`." Wave 10's validation block runs the four profile commands but does not include a script that diffs the §Decisions table against the profile JSONs. Without a check this is reviewer-judgment and will drift. Either add `verification/policy/checks/profile_assignment_matrix.py` (or equivalent) to Wave 0's coverage-matrix check or Wave 10's validation block.

**5. Wave 0's "one-to-one to Waves 1-9" mapping for `expected-missing` is ambiguous.**
The §Decisions text says rows are "mapped one-to-one to Waves 1-9." Does that mean one row per wave, or each row is tagged with exactly one wave? The latter is presumably intended. Rephrase to: "each `expected-missing` row carries a `closes_in_wave` field referencing exactly one wave in 1-9."

## P2 — Tightening, not blocking

**6. The "Reported by the verification reviews and still to re-measure" subsection in §Existing Facts is now stale.**
`sifr_codegen` 655/52/707 was just verified again locally; the diagnostics baseline count of 2 is verified. Only the e2e corpus sizes and host warm/cold wall time remain unverified. Move the codegen count and diagnostic-baseline count up into "Verified during this planning pass" so the implementer doesn't re-run them unnecessarily.

**7. "Once stable" thresholds are subjective.**
Wave 6.1: "Add a small deterministic seed set to merge once stable." Wave 9: "Add package-management integration suites to nightly at minimum and merge once stable." Define "stable" — e.g., "no flakes across 20 consecutive nightly runs" — or set explicit promotion criteria in §Decisions.

**8. Wave 7 doesn't define the fuzz input for the diagnostic renderer entrypoint.**
Renderer fuzzing needs a structured input (a `Diagnostic` value or its JSON), not raw source. The other four entrypoints are clearly source-driven. Note this explicitly so the implementer doesn't accidentally write a duplicate parse-fuzz with extra steps.

**9. Wave 5's "parsed source shape where Sifr owns behavior above the parser fork" is fuzzy.**
The pipeline note in `AGENTS.md` lists `sifr_python_parser`/`sifr_python_ast` as Ruff fork. The plan should name the concrete Sifr layer being snapshotted (probably `sifr_syntax` / `sifr_frontend` surface above the AST) so implementers don't argue about what to snapshot.

**10. The diagnostics baseline coverage check file location is unspecified.**
Wave 4 introduces "a diagnostics baseline coverage check that fails when a new diagnostic code has no rendered baseline." This should live next to its peers: `verification/areas/diagnostics/checks/code_baseline_coverage.py`, parallel to the existing `code_coverage.py` and `baseline_hygiene.py`.

**11. Acceptance criterion "every gate-expanding wave records measured warm/cold merge wall time" is missing Wave 2.final from the §Required Tracking Updates enumeration.**
§Required Tracking Updates says "Gate-expanding waves include at least Waves 1, 2, 3, 4, and 7." Wave 2 is split into 2.0/2.1..N/2.final; only 2.final flips the merge gate. Spell out that the wall-time measurement is required at the 2.final boundary, not 2.0 or each per-classification PR.

---

# Required Edits

These are the text-level changes I'd want before calling this implementation-ready.

**§Existing Facts To Verify — move codegen and diagnostic-baseline counts up to "Verified":**

Replace the "Reported by the verification reviews and still to re-measure" list with:

```
Verified during this planning pass (re-verify only on environment change):
- cargo test -p sifr_codegen: 655 passed, 52 failed, 707 total (red, excluded from merge).
- verification/areas/diagnostics/manifest.json ships exactly two rendered baseline fixtures (decimal_invalid_literal, multiline_span_rendering).

Re-measure at implementation start (host-dependent):
- E2E corpus size under verification/areas/core_language/.
- Current warm/cold merge wall time on the implementer's host.
```

**§Decisions — clarify the matrix `expected-missing` mapping:**

Replace "with a closed list of `expected-missing` rows mapped one-to-one to Waves 1-9" with:

```
with a closed list of `expected-missing` rows. Each row carries a `closes_in_wave` field naming exactly one wave in 1-9. The same wave may close several rows, but no row may be open after its named wave merges.
```

**§Wave 4 — reconcile merge vs. nightly renderer coverage:**

Add a bullet under tasks:

```
- Define renderer coverage rule per profile: merge-blocking requires ≥1 baseline per active stable SIFR-* code with a stable user-facing message; nightly/release require all stable renderers (human, compact, JSON). Update §Acceptance Criteria to reflect this profile split.
```

And in §Acceptance Criteria, replace "Diagnostic rendered baselines cover every active `SIFR-*` code with a stable user-facing message and all stable renderers." with:

```
- Merge: every active SIFR-* code with a stable user-facing message has ≥1 rendered baseline.
- Nightly and release: every active SIFR-* code with a stable user-facing message has rendered baselines for every stable renderer (human, compact, JSON).
- The diagnostics baseline coverage check (verification/areas/diagnostics/checks/code_baseline_coverage.py) enforces both rules and fails on undocumented gaps.
```

**§Wave 4 — also add the multi-error rule:**

Replace "Add multi-error recovery fixtures where one source file intentionally triggers several independent diagnostics." with:

```
- For every parser, HIR, name-resolution, and type-checker recovery surface listed in `verification/policy/suite_taxonomy.md` (extend that doc if no list exists), add at least one multi-error recovery fixture exercising that surface. The diagnostics baseline coverage check fails if a documented recovery surface has zero multi-error fixtures.
```

**§Wave 6.1 — pin stderr semantics:**

Replace "Compare stdout, stderr classification, and exit code according to the subset contract." with:

```
Compare:
- stdout: byte-equal (after deterministic-output normalization documented in the subset contract).
- exit code: bucketed as 0 / non-zero (precise integer equality only for documented exit-code-stable programs).
- error presence: no-error vs. compile-error vs. runtime-error; message text is not compared because Sifr's error model is an excluded divergence (Result/Option vs. exceptions).
- Per-program timeout and overall suite timeout are recorded in the suite manifest.
```

**§Wave 7 — clarify renderer fuzz input:**

Add a bullet:

```
- Diagnostic renderer fuzz target consumes structured Diagnostic values (or their JSON serialization), not source code; document the input grammar in verification/policy/fuzz_property.md so it does not duplicate parser fuzzing.
```

**§Wave 5 — name the parsed-shape layer:**

Replace "parsed source shape where Sifr owns behavior above the parser fork" with:

```
parsed-source shape at the Sifr-owned boundary (sifr_syntax / sifr_frontend surface above sifr_python_parser / sifr_python_ast)
```

**§Wave 9 / Wave 6 — define "once stable":**

Replace "once stable" in both waves with:

```
once the suite has 20 consecutive nightly green runs with no quarantine entries and no flaky retries
```

(or substitute whatever threshold the verification policy already uses; the point is to remove the word "stable" as a subjective gate).

**§Wave 10 — add a profile-assignment consistency check:**

Add a task:

```
- Add and run `verification/policy/checks/profile_assignment_matrix.py` (or equivalent area-check entry) that diffs the §Decisions profile-assignment table against verification/profiles/{create-pr,merge,nightly,release}.json and fails on drift. Promotion to blocking happens with the coverage-matrix promotion.
```

**§Required Tracking Updates Per Wave — pin wall-time measurement to 2.final, not all of Wave 2:**

Replace "Gate-expanding waves include at least Waves 1, 2, 3, 4, and 7." with:

```
Gate-expanding waves include at least Wave 1, Wave 2.final, Wave 3, Wave 4, and Wave 7. Wave 2.0 (inventory) and Wave 2.1..N (per-classification fixes) do not require wall-time measurement; Wave 2.final must record before/after merge wall time when sifr_codegen is added to the merge crate list.
```

---

# Verdict

**Ready after minor edits.**

Pass 1's P0/P1 set is genuinely closed — the plan now reflects the real test landscape, the matrix lands advisory and promotes, Wave 2 splits into inventory + per-classification + flip, Wave 5/9 are sub-PR'd, Wave 6.0 pre-commits the divergence catalogue, the profile-assignment table is in place, gate-expanding waves carry warm/cold measurement, and fuzz scaffolding extends the existing `fuzz_property` area rather than forking it. The remaining P1 items (renderer-coverage profile split, stderr semantics, multi-error rule, profile-assignment check, `expected-missing` mapping shape) are text-level fixes; none of them require restructuring waves. After the edits above land, this is implementation-ready without a third review pass.
