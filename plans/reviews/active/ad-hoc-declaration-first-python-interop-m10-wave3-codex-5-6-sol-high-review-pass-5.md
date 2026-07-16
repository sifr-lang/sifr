# M10 Wave 3 Codex Review Pass 5

- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Scope: complete committed `main...6c84cdacb` Wave 3 diff for PR #2989
- Verdict: changes requested

## Findings

1. **Medium — duplicate runtime results still satisfied the exact-five
   requirement.** Parsing successful Cargo test lines into a set erased a
   duplicated sixth observation.
2. **Medium — the positive evidence ledger overstated its exact runtime owners
   and pointer-width coverage.** The raw format owner was unlisted, pointer-width
   formats were absent from its family table, and no runtime test invoked the
   pointer-width accessors.

## Reviewer validation

- Empty, missing, and duplicate compiled-case result sets were rejected.
- Runner and compatibility self-tests passed.
- Python, JSON, shell, and diff syntax checks passed.

## Remediation

- Preserve the runtime observation list, require exactly five observations and
  the exact five unique names, and self-test zero, missing, duplicate, and exact
  result sets.
- Register the raw format owner, add pointer-width formats to its native-endian
  matrix, and add C-level acquisition/read/write/copy round trips for every
  supported fixed-width, pointer-width, and float element type.

The complete remediated diff requires a fresh whole-diff review pass.
