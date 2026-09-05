I have enough to write the pass 3 report.

## Findings

### Pass-2 findings — all resolved

1. **Phase plan now reflects the six Tier 1b matrix entries.** `plans/issues/active/ad-hoc-embedded-python-interop.md`:
   - line 578 lists `grpcio` (HTTP/async/networking)
   - line 580 lists `pika` and `moto` (Databases/queues/brokers)
   - line 583 lists `Pillow`, `tensorflow`, `scikit-learn` (Data/binary/parser)
   
   Cross-check: `--gate tier1b` returns 25 packages and all six appear in the report. Set-equality between matrix-derived Tier 1b and the plan bullets is now intact for the six entries pass 2 flagged.

2. **Runner enforces tier ↔ gate consistency.** `verification/python_interop/runner/run.py:136-139`:
   - `entry.tier == "tier1" and entry.gate is None` → `tier1 package X must declare gate = "tier1a" or "tier1b"`
   - `entry.tier != "tier1" and entry.gate is not None` → `non-tier1 package X must not declare certification gate`
   
   I independently exercised both branches plus the pre-existing unknown-gate branch with synthetic `PackageEntry` values — all reject with the expected `SystemExit` message.

3. **Self-tests cover the new branches.** `run.py:191-207`: unknown gate (`tier99`), tier1 without gate, non-tier1 with gate. Each negative test runs `validate_*` and re-raises if the bad input was *accepted*. `--self-test` exits 0 cleanly and writes no report. Confirmed: `reports/` stays empty after self-test.

### Spot-checked, still clean

- 21+25 Tier 1 entries (`tier1.toml` 21, plus 25 across `native`/`async`/`data`/`cloud`/`brokers`) all carry `gate = "tier1a"|"tier1b"`; 27 non-tier1 entries (`tier2`/`tier3`/`tier4`) carry none — validator invariant holds across the whole matrix.
- `--gate tier1a` selects exactly 21, `--gate tier1b` exactly 25, `--gate tier99` rejected with the expected message.
- `KNOWN_GATES = {"tier1a", "tier1b"}` (`smoke_matrix.py:23`) and `PackageEntry.gate: str | None` (`import_matrix.py:13`) unchanged in shape from pass 2; pass-2's spot-check observations on registry/docs/IPC lowering are untouched and remain fine.
- Touched files all under the 900-line guardrail (plan file at 914 lines is a doc, not first-party source — guardrail script reportedly passes; verified by the user, no reason to recheck).

### Minor non-blocker (pre-existing, not a pass-3 regression)

- **Case-skew on `Pillow` vs `pillow`.** Plan line 583 says `Pillow`; `data.toml` says `name = "pillow"`. PyPI is case-insensitive so the cert target is unambiguous, but `--package Pillow` would miss the entry because the runner compares name strings literally. Cosmetic for py_0 scaffold; worth normalizing one side before milestone_py_11 makes `--package` load-bearing. Did NOT exist as a pass-2 finding — flagging for awareness only.

## Open Questions

None for py_0. Pass-2 open questions #1 (plan/matrix membership) and #2 (gate enforcement) are now closed. Pass-2 open question #3 (host-dependent `skip_when`/`evidence` structured field) remains explicitly deferred to milestone_py_11 — unchanged.

## Verdict

**No blocking issues. milestone_py_0 is ready to ship.** All three pass-2 findings are resolved: the six Tier 1b matrix entries (`grpcio`, `pika`, `moto`, `Pillow`, `tensorflow`, `scikit-learn`) now appear in the corresponding phase-plan category bullets; `validate_matrix_entries` rejects both tier1-without-gate and non-tier1-with-gate; the self-test suite covers unknown-gate, missing-tier1-gate, and non-tier1-gate paths. Independent reproduction confirms `--self-test`, `--gate tier1a` (21/21), `--gate tier1b` (25/25), and `--gate tier99` (rejected) all behave as described, and synthetic `PackageEntry` calls exercise the new validator branches end-to-end. The remaining `Pillow`/`pillow` case-skew is cosmetic and pre-existing — handle when `--package` becomes load-bearing, not now.
