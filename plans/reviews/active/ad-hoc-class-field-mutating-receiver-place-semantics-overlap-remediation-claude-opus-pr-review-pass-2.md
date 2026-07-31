# Class-field mutating-receiver overlap remediation — Claude Opus PR review pass 2

## Review target

- Base: `a7a5df414b985cc95a9ad23c5b006caa84101f0d`
- Head: `92b38be705138643b23c37a425892df767beee5d`
- Pull request: #3090
- Scope: the precise unsupported-field footprint remediation, structured
  `SIFR-OWN-0002` surface, phase-fixture coverage, and the prior review
  finding.

The reviewer was instructed to remain read-only and assess the exact commit
range against the active phase plan and repository workflow.

## Validation independently inspected or run by the reviewer

- Inspected the exact two-commit implementation range and pass-1 review
  artifact.
- Ran `cargo test -p sifr_lowering unsupported_`: 21 tests passed, including
  callable/recursive overlap rejection and disjoint-sibling acceptance.
- Reproduced both new fail fixtures as one structured `SIFR-OWN-0002` at the
  annotated source location.
- Inspected the exact-head create-PR log, which passed all enforced budgets,
  crate tests, 137/137 manifest E2E fixtures, Python interop, runtime, tooling,
  and representative performance checks.

## Findings

### Blocking

1. The plan's overlap rule still said every unsupported/dynamic projection
   under the same root is conservatively overlapping. That contradicted the
   corrected implementation, which retains precise declaring-field identity
   for callable and optional/recursive field values whose base place is
   statically resolvable. The plan must distinguish those values from genuinely
   unresolvable bases and dynamic index/slice projections.
2. The issue status still described Item 2 as pending under #3082, mentioned
   neither merged remediation #3087 nor current remediation #3090, and ended
   its review ledger at pass 11. The authoritative closure record must include
   the merged history and both #3090 review rounds.

### Non-blocking

1. Direct runnable probes of the new disjoint-sibling examples exposed
   pre-existing value-codegen failures for moving callable/recursive fields
   and for mutable free-function field arguments. The same probes were already
   accepted at the detached base; these are separate codegen guarantee issues,
   not regressions in the overlap-remediation range.
2. The exact-head create-PR log did not print the Git SHA and the create-PR
   profile does not run the fail suite. The reviewer closed the semantic gap
   with direct fail-fixture checks; the evidence ledger should retain the
   reviewed SHA and those results.
3. The unresolvable-base fallback may collect the same dynamic root twice for
   index/slice-rooted field access. This is harmless because overlap is an
   `any` query, but it is redundant work.
4. The `SIFR-OWN-0002` error page mentions the pending async-generator case
   without a worked erroneous/fixed example.

## Disposition

- Verdict: **NOT SATISFIED**
- Required action: align the authoritative overlap rule with precise static
  field identities and refresh the status/review ledger for #3087 and #3090.
