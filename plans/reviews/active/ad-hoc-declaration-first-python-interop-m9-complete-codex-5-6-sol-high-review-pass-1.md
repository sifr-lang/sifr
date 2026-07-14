# M9 complete implementation — Codex 5.6 Sol high review pass 1

Reviewer configuration: `gpt-5.6-sol`, high reasoning, fast service tier,
read-only review of the complete merged M9 range from pre-M9 base
`62878588d79bdee0667123ed246235b1eddcf958` through merge
`2c5c6555faf475c7bfbc62c0a3ab8e079d3bef11`.

## Verdict: blocked

The runtime, lowering, code generation, callback lifecycle, cancellation,
drain, finalization, shutdown, and sendability mechanisms satisfy the reviewed
M9 contracts. Milestone closure is blocked by one false required live-evidence
claim and the resulting stale completion tracking.

## Findings

1. **P1 — required `dispatch=current` compiled/live evidence is missing and
   falsely attributed.** The capability ledger names the compiled CFFI example
   as current-thread evidence, but the only compiled CFFI declaration uses
   `dispatch=foreign` and its Python bridge invokes the callback from a worker
   thread. Add a separate compiled same-thread `dispatch=current` fixture,
   register it in the callback suite, correct the capability evidence owner and
   README, and retain the existing CFFI fixture as foreign evidence.
2. **P2 — merged Wave 3 remains recorded as incomplete.** Mark Wave 3 complete
   after correcting the evidence gap, while keeping the milestone-review item
   open until the remediation is merged and the complete M9 review is
   satisfied.
