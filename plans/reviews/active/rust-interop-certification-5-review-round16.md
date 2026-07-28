# Certification 5 Review — Round 16

## Verdict

**NOT SATISFIED**

This review inspected the exact PR #3042 diff against current `origin/main`,
all implementation and evidence surfaces, prior rounds 1–15, and read-only
mergeability with the unrelated Phase 40 release work. The reviewer confirmed
that the PR is mergeable without file overlap and that the unrelated modified
`editor_integrations` submodule and untracked `.cert5probe/` directory are
excluded.

## Confirmed strengths

- Independently recomputed inventory counts match the plan.
- File-size and HIR maintainability guardrails pass.
- The round-14 runtime-evidence and temporary-prompt findings remain closed.
- Lifecycle timeouts, drop guards, tracked-task accounting, client-before-join
  cleanup, and panic-payload redaction are sound.
- Ownership propagation, re-export coverage, and plan-digest invalidation are
  correctly implemented.

## Actionable findings

### Medium: positive subscription fixture missed the new consuming annotation

`callback_subscription_ecosystem/positive/subscription_cancel_shutdown.sifr`
accepts a borrowed `Subscription` and calls consuming `aclose()`. It now fails
with `SIFR-OWN-0003`, while five sibling fixtures were updated to accept
`own`. Update this positive fixture consistently.

### Medium: three passing negatives fail before their claimed evidence

The negative fixtures for `close_after_use`, `opaque_resource_core`, and
`opaque_handle_tokenizer` still declare opaque `close(self) -> None`. The new
validation rejects them with `SIFR-RUST-CONFIG-0001` and `SIFR-OWN-0003`
before their claimed diagnostic/runtime family is reachable. Update them to
the new `own self -> Result[...]` cleanup shape and preserve their intended
evidence.

### Medium: merge-gate provenance no longer pins the integration base

Phase 40 PR #3041 landed after the recorded run and added a protected-release
drill to the merge profile. Integrate current `origin/main`, rerun the full
merge facade, and record the exact base commit SHA with the resulting counts.

### Low: non-union poisoned-state fallback lacks codegen coverage

The `Self.*` poisoned-state fallback for a plain declared message error is
redaction-safe, but all new emission tests use a union containing
`RustPanicError`. Add a focused non-union codegen test.

