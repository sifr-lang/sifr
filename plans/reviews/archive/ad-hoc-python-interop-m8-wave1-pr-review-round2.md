# M8 Wave 1 PR review — round 2

PR: #2970 at `c497dd0c4`

Reviewer: Claude Opus 4.7 (`xhigh`)

Verdict: **SATISFIED**

## Verification performed

- Read `gh pr view 2970` and the complete `gh pr diff 2970`.
- Read the async-context codegen and cancellation primitive end to end.
- Traced the child-carrier notification through async submission, terminal completion, and cancellation propagation.
- Read the new regression, the existing generated-Rust syntax test, and the round-1 review artifact.

## Findings

No blocking findings.

1. The enter-failure path is move-safe. `scope.notification()` only borrows the scope and returns a cloned sticky notification. The cancellation branch then consumes the scope through `release_and_resume_parent()` and immediately returns, while the ordinary-error branch drops the still-live scope normally.
2. Active cancellation reaches `async_cancellation::propagate`, which synchronously invokes the child fallback before the enter await returns. Therefore the sticky notification is already set when generated code observes the cancelled enter error.
3. Ordinary Python enter failures do not invoke cancellation propagation, so their notification remains unset and they preserve the ordinary converted-error path.
4. `release_and_resume_parent()` clears the exact claim before invoking the parent fallback. The parent's `fallback_resumed` guard and mutually exclusive, scope-consuming branches enforce exact-once resumption.
5. Cancellation maps to the internal `SifrPythonAsyncContextError`, while an ordinary enter failure retains the declared Python error conversion. This matches the accepted cancellation-arm contract.
6. Reusing the rendered parent-resume sequence at the enter-failure and body-cancellation sites is sound: both sites own an unconsumed scope and immediately return after the sequence.
7. The round-1 artifact accurately records the prior verdict, findings, and maintainer disposition.

Non-blocking test nit: the focused regression asserts the required tokens are present in the enter-failure slice but does not structurally assert their nesting order. Existing syntax parsing plus Wave 2 compiled-code evidence make this acceptable for the Wave 1 gate.

## Final assessment

The reviewer concluded that the corrected enter-failure branch is cancellation-safe, preserves ordinary Python error behavior, resumes the parent fallback exactly once, and leaves the complete PR safe to merge.

**PR #2970 is safe to merge.**
