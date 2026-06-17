

All four pass 4 findings are addressed:

1. **Contract guardrail wiring (was moderate):** M6 lines 495-496 call `check_diagnostic_source_canonicalization_rules.py` and its `--self-test`. M1 line 424 explicitly wires the checker into `run_all_tests.sh --profile quick`. Row 12 of the verification matrix mirrors this.

2. **M2 edge cases (was minor):** M2 line 438 explicitly names "CRLF source text" and "non-ASCII text before the error location" alongside the zero-length/out-of-bounds cases.

3. **Ambiguous/collision flow parity (was minor):** M3 line 457 extends parity requirements to ambiguous imports and namespace/file collisions "in every flow where those states can be constructed."

4. **Registry activation gate (was minor):** M1 line 423 requires placeholders and fails the contract if codes are "undocumented or missing from the active registry." M1 line 424 explicitly requires "active registry entries," not just doc mentions. Row 11 of the verification matrix requires the codes be "active registry entries."

**Verification matrix vs. implementation plan alignment:**

| Finding | Matrix row | Milestone line | Status |
|---|---|---|---|
| Contract wiring | row 12 | M1:424, M6:495-496 | Patched |
| CRLF/non-ASCII edge | row 3 | M2:438 | Patched |
| Ambiguous/collision parity | rows 6-7 | M3:457 | Patched |
| Registry activation | row 11 | M1:423-424 | Patched |

**No remaining blockers.** The plan is implementation-ready and structurally sound.
