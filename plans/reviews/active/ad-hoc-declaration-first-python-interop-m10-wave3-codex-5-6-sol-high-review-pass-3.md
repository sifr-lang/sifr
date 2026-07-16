# M10 Wave 3 Codex Review Pass 3

- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Scope: complete committed `main...c3760d1ee` Wave 3 diff for PR #2989
- Verdict: changes requested

## Findings

1. **Medium — CPython 3.11 compatibility was asserted but not a pinned
   executable lane.** The default verification environment could select a newer
   interpreter and miss a future minimum-version regression.
2. **Medium — exact negative-evidence ownership omitted five claimed native
   fixtures.** The evidence description named the full `python_buffer_*.sifr`
   family while the owner set locked only the identity fixture.
3. **Low — the phase tracker omitted review pass 2 and its remediation.**

## Reviewer validation

- Runtime buffer tests passed `30/30`.
- Focused lowering passed `34/34`; focused code generation passed `10/10`.
- Runner adversarial self-tests and all five compiled examples passed.
- Generated packages contained only their case-specific bridge and exact
  import/native trust roots.
- Formatting, diff, HIR maintainability, and file-size checks passed.

## Remediation

- Added a blocking CPython 3.11 compatibility suite to every delivery profile.
  It runs the five C-level exact release/pointer tests and all five compiled
  declaration-first fixtures in a minimal locked NumPy environment; CI installs
  the pinned interpreter explicitly.
- Enumerated all six native-negative buffer fixtures as exact evidence owners.
- Updated the phase ledger through review pass 3.
- Moved the test buffer format constant to satisfy Python-feature Clippy and
  corrected stale demo evidence wording.

The complete remediated diff requires a fresh whole-diff review pass.
