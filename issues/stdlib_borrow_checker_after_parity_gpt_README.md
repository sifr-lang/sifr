# Stdlib Borrow Checker Audit (Post-Parity)

This directory contains the borrow/ownership-focused stdlib audit requested for the post-parity state.

## Files

- `AUDIT_SUMMARY.md` - executive summary and top findings.
- `CONTRADICTIONS_AND_REMEDIATION.md` - evidence-backed contradictions and prioritized remediation plan.
- `MODULE_OWNERSHIP_MATRIX.md` - detailed per-module ownership/mutability/risk matrix across all 37 stdlib modules.

## Focus

- Borrow-by-default behavior in stdlib API design.
- Ownership transfer and mutable-borrow coverage in public stdlib signatures.
- Receiver mutability and class API behavior.
- Cross-check against architecture-level ownership and safety contracts.
