## Pass 2 review — cleanup is clean

Reviewed both files end-to-end against the four cleanup items.

**1. Exception handling in `run_evidence_case` (lines 367-383)** — Order is correct: `EvidenceFailure` → `subprocess.TimeoutExpired` → broad `Exception`. All three branches:
- compute `elapsed_ms` from the same `started` reference,
- emit a stderr line and a `[sifr-case-timing] … status=fail` line,
- return a `fail_variant`, so JSON always lands.

The broad `Exception` catch is appropriately scoped (`# noqa: BLE001` with a justification) and uses `f"{type(exc).__name__}: {exc}"` so the failing exception class is preserved in `mismatches`. `KeyboardInterrupt` (BaseException) still escapes — good.

**2. `run_and_check_duration` rename (line 386)** — Accurate. The function is a post-hoc check that runs the callback to completion before measuring; the new name removes the misleading "timeout" framing. The hard timeout for the only long probe (`install-distribution-smoke`) comes from `subprocess.run(..., timeout=60)`, not this wrapper, which is the right division.

**3. `support_matrix_variants` timings emitted after construction (lines 321-322)** — Loop emits one `[sifr-case-timing] … status=<variant status>` per variant; status is read from the constructed variant rather than guessed. Consistent with the evidence-suite emission shape.

**4. Timeout raised to 60s** — Manifest (`platform_evidence_manifest.json:203`) and subprocess (`check_platform_evidence.py:549`) match. Schema validator (line 269-271) accepts 60 as positive int. No drift.

**No blockers, no regressions.** Self-test mutations cover the three invariants you'd expect (forbidden skip, external network, missing case); manifest required-cases set still includes all 12. Reviewer satisfied — ship it.
