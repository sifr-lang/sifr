Reviewing the verification scope added after pass 3 and assessing phase balance.

**Scope of this review:** verification matrix coverage, requirement soundness, phase elegance.

---

## Review Findings

### Parser Coverage (Verification Matrix row 1–3, M1 lines 409–410, M2 lines 434–438)

**Row 1** (span attachment): unit tests + explicit fixture list covers the five named parser gap categories. Solid.

**Row 2** (span completeness) and **Row 3** (edge cases): M2 explicitly requires JSON field coverage (`byte_start`, `line`, `highlight_start`, etc.) and edge case tests (zero-length EOF, out-of-bounds, CRLF, non-ASCII). Both the table row and M2 are aligned. However, **Row 3** mentions CRLF and non-ASCII — check whether M2 lines436 explicitly mentions these, as they appear in the table but the table is the authoritative coverage list. **Action item:** add "CRLF source text" and "non-ASCII text" explicitly to M2's edge case list to match the verification matrix row3.

**Severity:** minor — the table governs, but stale alignment between table and M2 description is a maintenance hazard.

---

### Import Coverage (Verification Matrix rows 4–7, M3 lines 450–456)

**Row 4** (missing module imports): the parity test row explicitly requires same canonical `SIFR-IMPORT-0002` across single-file, workspace, package, and editor flows. M3 lines450–454 lists the same four flows. Consistent.

**Row 5** (missing member imports): the table says `SIFR-NAME-0004` remains distinct, no duplicate missing-module diagnostics. M3 line 449 addresses this. Good.

**Rows 6–7** (ambiguous, namespace collision): primary span + candidate/collision context + no old workspace code. M3 lines 444–445 and 456 add negative tests for old-code leakage. Good.

**Gap found:** The table covers `SIFR-IMPORT-0005` and `SIFR-IMPORT-0006` for ambiguous and collision cases, but does **not** ask for coverage across the same four flows (single-file, workspace, package, editor). The parity expectation in the table is limited to `SIFR-IMPORT-0002` missing module imports only. Should ambiguous imports and namespace collisions also be tested across flows? The phase purpose says "same user-facing problem should use the same diagnostic code regardless of flow," which implies yes — but the verification matrix stops short. **Recommendation:** either (a) extend rows 6 and 7 to include multi-flow parity, or (b) explicitly scope the parity requirement to missing module and import cycle only, to avoid an unspoken gap.

**Severity:** minor — the intent is readable from the phase purpose and M3, but the verification matrix implies a narrower scope.

---

### Cycle Coverage (Verification Matrix row 8, M4 lines 461–466)

Row8 covers primary import-edge span, related spans/edge context, cycle path JSON args, and no `SIFR-WORKSPACE-0104` emission. M3 line456 (negative tests for old codes) and M4 lines 461–466 cover exactly this. M4 also adds two-node and three-node cycle tests explicitly. No gaps.

---

### Package Coverage (Verification Matrix rows 9–10, M5 lines 475–483)

**Row 9** (package import context): JSON assertions for written module path, resolved package import path, `PackageImportOrigin`, dependency package id, resolution scope. The struct defined at lines 347–356 has exactly these fields. M3 line 455 adds JSON assertions for these fields. Aligned.

**Row 10** (package diagnostic conversion): unit tests for each `PackageDiagnosticOrigin` variant, integration tests for help preservation. M5 lines 475–483 covers all variants named in the table (lines 476–481 enumerate them explicitly), plus help preservation and spanless status. Aligned.

---

### Legacy Code Migration (Verification Matrix row 11, M3 line 458, M4 line 465)

Row 11: registry/docs tests proving new codes exist and old workspace import codes are documented as legacy/aliases but not emitted. M1 line 422 requires registry/docs placeholders for new codes. M3 line 458 and M4 line 465 cover doc updates and aliasing. **Minor gap:** "not emitted by source-level fixtures" is the behavioral test; the table does not explicitly call for a registry existence test. M1 line 422 covers placeholders; a registry-existence gate check belongs in M1 alongside the placeholder work, but it is implied rather than stated. **Recommendation:** add a line to M1 requiring that the contract checker verifies new codes are active in the registry, not just documented as placeholders.

**Severity:** minor — M1 line 422 is close, but a stricter M1 gate for registry activation is worth an explicit line.

---

### Contract Guardrail (Verification Matrix row 12)

Row 12 requires a source-canonicalization contract checker wired into `scripts/run_all_tests.sh --profile quick` with negative self-tests. M1 line 418 proposes "extend or add a sibling contract checker." Line 539 adds the specific negative categories. **Structural gap:** M1 (lines 406–428) describes a contract checker to be built, but there is no explicit milestone task that wires it into the quick profile. The M6 closeout script references the presentation contract (`check_diagnostic_presentation_rules.py`) but does not mention a source-canonicalization equivalent. **Action item:** either add a line to M6's closeout script calling the new contract checker, or add an M1 sub-task explicitly requiring the wiring into `run_all_tests.sh --profile quick`.

**Severity:** moderate — without the mechanic gating the quick profile, the coverage gap categories in row 12 would not be caught by the development loop. Worth explicit placement.

---

## Verification Matrix Completeness Check

Checking against the phase purpose categories:

| Layer | Source spans | Canonical code | Flow parity | Help/context |
|---|---|---|---|---|
| Parser | ✓ row1–3 | ✓ `SIFR-PARSE-*` | N/A | ✓ row1 |
| Missing module import | ✓ row 4 | ✓ row 4 | ✓ row4 (4 flows) | ✓ row 4 |
| Missing member import | ✓ implied | ✓ row 5 | implicit | ✓ implied |
| Ambiguous import | ✓ row 6 | ✓ row 6 | partial (gap noted) | ✓ row 6 |
| Namespace collision | ✓ row 7 | ✓ row 7 | partial (gap noted) | ✓ row 7 |
| Import cycle | ✓ row 8 | ✓ row 8 | N/A | ✓ row 8 |
| Package import context | ✓ row 9 | ✓ row 9 | N/A | ✓ row 9 |
| Package diagnostic help | ✓ row 10 | ✓ row 10 | N/A | ✓ row 10 |
| Legacy migration | N/A | ✓ row 11 | ✓ row 11 (neg tests) | ✓ row11 |
| Contract guardrail | ✓ row 12 | ✓ row 12 | ✓ row 12 | ✓ row 12 |

All major aspects are covered. No missing cell is a blocker; the gap between parity scope for ambiguous/collision cases and the lack of an explicit multi-flow assertion is the only area where the matrix could imply something different from the stated purpose.

---

## Phase Elegance After Verification Expansion

The verification matrix adds ~20 lines of table rows and M1/M2 gains ~10 lines of fixture names. This is proportionate to the phase scope. The matrix is well-structured, each row maps to milestone assignments, and there are no circular or unreachable requirements. The phase remains clean and implementation-ready.

---

## Blocker Assessment

**No blockers.**

## Findings Ordered by Severity

1. **Moderate — Contract guardrail wiring gap (row 12):** The verification matrix requires wiring a source-canonicalization contract checker into `run_all_tests.sh --profile quick` with negative self-tests, but neither M1 sub-task, M6 closeout script, nor any closeout step explicitly mentions building or wiring this checker. Recommend: add a line to M6's closeout script calling the new contract checker (e.g., `python3 verification/tooling/check_diagnostic_source_canonicalization_rules.py`).

2. **Minor — M2 edge case list partial alignment (row 3):** Row 3 of the verification matrix explicitly names CRLF source text and non-ASCII text before errors as edge cases. M2 line 436 lists zero-length EOF ranges, invalid out-of-bounds ranges, and unsupported syntax diagnostics — but does not explicitly name CRLF and non-ASCII. Recommend: expand M2 line 436 to explicitly include the CRLF and non-ASCII cases to match the matrix.

3. **Minor — Multi-flow parity scope for ambiguous/collision imports:** The verification matrix's parity requirement is scoped only to missing module imports (row 4) across four flows. But the phase purpose says same code regardless of flow, which applies equally to `SIFR-IMPORT-0005` and `SIFR-IMPORT-0006`. Recommend: either extend rows 6 and 7 with multi-flow parity, or add an explicit note scoped to missing module + import cycle only.

4. **Minor — M1 registry activation gate:** M1 line 422 ensures codes are documented/placeholdered, but row11 requires that new codes "exist" (implying active in registry). An explicit M1 sub-task that the contract checker confirms registry activation would close the loop. Low cost, worth one line.

None of these are blockers. The phase is implementation-ready with correct architecture and sufficient verification coverage. Ship it, then address findings 1–4 in M1 scoping.
