Spot-checks confirm: the new evidence check verifies every claimed code is in stderr (0 mismatches across 127 pairs), all three renderers render the literal `SIFR-…` code in stderr, the SIFR-STDLIB-0001 deferral is now explicitly documented in the issue file at line 865, and the contracts suite passes.

# Wave 4 Core Semantic Compact Baselines — Review Pass 2

**Verdict: no blockers. No further review rounds needed. Ready to open the PR.**

The two pass-1 follow-ups landed cleanly and the underlying slice still validates. I re-audited the evidence path end-to-end and could not find anything that should hold the PR.

## Blockers

None.

## Pass-1 follow-up review

**A. Coverage evidence enforcement (`code_baseline_coverage.py:217`, `226-242`).** Correctly closes the loophole.
- Gated by `case is not None`, so the call is reached only when the fixture id resolves to a manifest case — no spurious errors on deferral rows.
- Reads `<entry>.parent/baselines/check-<renderer>.stderr.txt` and does `code in baseline.read_text(...)`. Diagnostic codes (`SIFR-[A-Z]+-\d{4}`) are 4-digit-bounded, and I spot-checked all three renderers — compact emits `E SIFR-…`, human emits `error[SIFR-…]`, json emits `"code": "SIFR-…"`. Substring match is safe; no realistic false-positive path.
- Renderer-agnostic and fixture-agnostic — it fires on any future code/fixture/renderer combination, not overfit to this slice.
- Two separate error messages distinguish "evidence file missing" from "code not rendered". Good operator-readability.
- I re-verified the full coverage set programmatically: 127/127 (code, fixture, renderer) tuples pass; 0 mismatches. The contracts suite (`code_baseline_coverage` case) executed in 72 ms and passed.

**B. SIFR-STDLIB-0001 deferral rationale (`plans/issues/.../gate-closure.md:865`).** Adequately documented and acceptable.
- The phase doc now states: "`e2e_bare_defaultdict_constructor_rejected` incidentally emits `SIFR-STDLIB-0001`, but that coverage row intentionally remains deferred to the stdlib-owned Wave 4 slice so stdlib diagnostics get purpose-built baseline ownership rather than incidental semantic-fixture ownership."
- I confirmed the incidental emission is real (`check-compact.stderr.txt` line 3 shows `E SIFR-STDLIB-0001`). The decision to keep the row deferred is the conservative one — it avoids creating cross-team baseline ownership where a stdlib change would silently retune a semantic-team fixture. Defensible.
- Minor optional nit (not a blocker, not a follow-up needed before PR): the `deferral.reason` string in `code_baseline_coverage.json` for `SIFR-STDLIB-0001` is still the generic boilerplate; tightening it to mention "incidental defaultdict emission deliberately not adopted; awaits purpose-built stdlib fixture" would let the row stand alone without requiring readers to chase the phase doc. Leave or apply at slice author's discretion.

## Answers to your review questions

**Q1. Does the new evidence check correctly close the pass-1 loophole without overfitting?**
Yes. See A above. The check is general (renderer- and fixture-agnostic), cheap (filesystem + substring), correctly gated, and 100% green against the current 127 pairs. It will reject any future row that claims a code its baseline doesn't render.

**Q2. Is leaving incidental SIFR-STDLIB-0001 deferred now adequately documented and acceptable?**
Yes. See B above. The phase doc explicitly names the fixture, the incidental emission, and the ownership rationale. Optional polish on the per-row deferral reason is non-blocking.

**Q3. Production blockers before opening the PR?**
None.

**Q4. Further review rounds needed before PR?**
No. The pass-1 follow-ups are applied and verified; nothing surfaced in pass-2 that needs another round.

## Residual non-blockers (carried over from pass-1, not in scope for this PR)

These remain open from pass-1 finding #3, #4, #6, #7 — none new in pass-2 — and none gate the PR:

- **Drift hazard between diagnostics copies and `crates/sifr/tests/e2e/fail/<name>.sifr`.** Not addressed; will surface as silent staleness if upstream e2e fixtures evolve without rendered-output changes. Worth a `source_origin`-link check before Wave 4 expands further.
- **Warm merge wall-time advisory** (`1016.25s` vs `900s` budget). Pre-existing, advisory-only; the policy alignment item still belongs in the PR description, not in the code.
- **Pre-existing ownership inconsistency** on package/workspace/stdlib deferral rows (`compiler/frontend`/`compiler/core-language` vs the more specific owners in `owners.json`). Pre-existing; cheap to bundle later.
- **`bless_reference` placeholder slug** — matches prior-slice convention; replace with real PR URL after open.

Open the PR.
