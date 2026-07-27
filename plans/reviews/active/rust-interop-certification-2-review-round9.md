# Rust Interop `certification_2` Review — Round 9

Date: 2026-07-27

Reviewer: Claude Opus 5 (`medium`)

Reviewed head: `32379e3bbc936d67be55752cf9fe7c5ef8023558`

Verdict: `SATISFIED`

## Scope

Independent final merge-readiness adjudication after the authoritative merge
profile reached the representative performance budget subset. The review
rechecked the exact PR head, mergeability, all earlier certification review
artifacts, the functional lane results, the unchanged performance contracts,
and a retained pre-certification compiler control.

## Findings

No actionable PR-attributable finding remains.

- The worktree was clean, the local head exactly matched PR #3031, and GitHub
  reported the PR cleanly mergeable.
- The exact head differs from the round-8 implementation head only by review
  and tracking documentation.
- The create-PR lane passed all 24 steps. The merge lane passed every
  functional step and stopped only at the representative performance budget
  subset.
- The three timing cases use `check`, contain no Rust-interop declarations, and
  terminate on the frontend-diagnostics path before code generation or
  Rust-interop bridge planning.
- The PR changes no parser, HIR, lowering, frontend, performance-budget, or
  performance-baseline file.
- A retained compiler executable whose filesystem timestamp predates both
  `certification_1` merge and every `certification_2` commit was exercised
  through the same five-sample benchmark implementation. It missed the same
  three unchanged thresholds, and was slower than the current compiler on the
  arithmetic case.
- Repeated current-head and retained-control measurements overlap and reorder
  under sustained unrelated host load. This demonstrates host-wide timing
  drift rather than an implementation regression.
- The separately tracked demo/algorithm failures do not enter this lane and
  remain outside this Rust-interop item as directed.

## Preserved evidence

- `target/validation_lane_reports/create-pr.latest.json`
- `target/validation_lane_reports/merge.latest.json`
- `target/performance/representative.budget.latest.json`
- `target/performance/cert2-three-case.latest.json`
- `target/performance/cert2-preexisting-binary.latest.json`

The reviewer concluded that the environmental performance miss is fully
preserved, independently controlled, and is not an actionable blocker for
PR #3031.
